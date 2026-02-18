//! Chat endpoint with SSE streaming and tool loop.

use std::collections::HashMap;
use std::convert::Infallible;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};

use axum::{
    extract::State,
    response::sse::{Event, KeepAlive, Sse},
    routing::post,
    Json, Router,
};
use futures::stream::Stream;
use serde_json::json;
use tokio::sync::{mpsc, oneshot, Mutex, OwnedMutexGuard, RwLock};
use tokio_stream::wrappers::ReceiverStream;

use krusty_core::ai::client::{AiClient, CallOptions, CodexReasoningEffort};
use krusty_core::ai::providers::ProviderId;
use krusty_core::ai::streaming::StreamPart;
use krusty_core::ai::title::generate_title;
use krusty_core::ai::types::{
    AiToolCall, Content, FinishReason, ImageContent, ModelMessage, Role, ThinkingConfig,
};
use krusty_core::plan::{PlanFile, PlanManager, TaskStatus};
use krusty_core::process::ProcessRegistry;
use krusty_core::storage::{Database, WorkMode};
use krusty_core::tools::registry::{
    tool_category, PermissionMode, ToolCategory, ToolContext, ToolRegistry,
};
use krusty_core::SessionManager;

use crate::auth::CurrentUser;
use crate::error::AppError;
use crate::push::{PushEventType, PushPayload, PushService};
use crate::types::{
    AgenticEvent, ChatRequest, ContentBlock, ToolApprovalRequest, ToolResultRequest,
};
use crate::AppState;

const MAX_ITERATIONS: usize = 50;
const SSE_CHANNEL_BUFFER: usize = 256;
const MAX_TOOL_OUTPUT_CHARS: usize = 30_000;
const AI_STREAM_TIMEOUT: Duration = Duration::from_secs(120);
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
const EXPLORATION_BUDGET_SOFT: usize = 15;
const EXPLORATION_BUDGET_HARD: usize = 30;
const APPROVAL_TIMEOUT: Duration = Duration::from_secs(300);
const REPEATED_TOOL_FAILURE_THRESHOLD: usize = 2;
const SESSION_LOCK_MAX_ENTRIES: usize = 1000;
const SESSION_LOCK_MAX_AGE: Duration = Duration::from_secs(3600);

type SseSender = mpsc::Sender<Result<Event, Infallible>>;
type PendingApprovals = Arc<RwLock<HashMap<String, oneshot::Sender<bool>>>>;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", post(chat))
        .route("/tool-result", post(tool_result))
        .route("/tool-approval", post(tool_approval))
}

struct ChatSessionContext {
    ai_client: Arc<AiClient>,
    base_ai_client: Arc<AiClient>,
    options: CallOptions,
    conversation: Vec<ModelMessage>,
    session_id: String,
    session_manager: SessionManager,
    working_dir: PathBuf,
    work_mode: WorkMode,
    user_id: Option<String>,
    guard: OwnedMutexGuard<()>,
}

struct AgenticLoopContext {
    ai_client: Arc<AiClient>,
    tool_registry: Arc<ToolRegistry>,
    process_registry: Arc<ProcessRegistry>,
    sse_tx: SseSender,
    conversation: Vec<ModelMessage>,
    options: CallOptions,
    session_id: String,
    db_path: Arc<PathBuf>,
    working_dir: PathBuf,
    user_id: Option<String>,
    work_mode: WorkMode,
    permission_mode: PermissionMode,
    pending_approvals: PendingApprovals,
    push_service: Option<Arc<PushService>>,
}

struct ExecuteToolsContext<'a> {
    tool_registry: &'a ToolRegistry,
    tool_calls: &'a [AiToolCall],
    working_dir: &'a Path,
    process_registry: &'a Arc<ProcessRegistry>,
    sse_tx: &'a SseSender,
    user_id: Option<&'a str>,
    permission_mode: PermissionMode,
    pending_approvals: &'a PendingApprovals,
    session_id: &'a str,
    db_path: &'a Path,
    current_mode: WorkMode,
}

/// Build user message content from content blocks (images) and text message.
/// Processes content blocks first (images), then appends text content.
fn build_user_content(message: &str, content_blocks: &[ContentBlock]) -> Vec<Content> {
    let mut contents: Vec<Content> = Vec::new();

    // Process content blocks first (images)
    for block in content_blocks {
        match block {
            ContentBlock::Text { text } => {
                contents.push(Content::Text { text: text.clone() });
            }
            ContentBlock::Image { source } => {
                let image_content = match source {
                    crate::types::ImageSource::Base64 { media_type, data } => {
                        Some(Content::Image {
                            image: ImageContent {
                                base64: Some(data.clone()),
                                url: None,
                                media_type: Some(media_type.clone()),
                            },
                            detail: None,
                        })
                    }
                    crate::types::ImageSource::Url { url } => Some(Content::Image {
                        image: ImageContent {
                            base64: None,
                            url: Some(url.clone()),
                            media_type: None,
                        },
                        detail: None,
                    }),
                };
                if let Some(img) = image_content {
                    contents.push(img);
                }
            }
        }
    }

    // If no content blocks or message has text, add the message as text
    // (content blocks take precedence if present)
    if contents.is_empty() || !message.is_empty() {
        // Check if we already have a text block from content_blocks
        let has_text = contents.iter().any(|c| matches!(c, Content::Text { .. }));
        if !message.is_empty() && !has_text {
            contents.push(Content::Text {
                text: message.to_string(),
            });
        }
    }

    // If nothing was added, at least include the message
    if contents.is_empty() {
        contents.push(Content::Text {
            text: message.to_string(),
        });
    }

    contents
}

async fn setup_chat_session(
    state: &AppState,
    user: Option<&CurrentUser>,
    session_id: &str,
    model_override: Option<&str>,
    enable_thinking: bool,
) -> Result<ChatSessionContext, AppError> {
    let base_ai_client = state
        .ai_client
        .as_ref()
        .cloned()
        .ok_or_else(|| AppError::BadRequest("No AI credentials configured".to_string()))?;

    let user_id = user.and_then(|u| u.0.user_id.clone());
    let user_home_dir = user.and_then(|u| u.0.home_dir.clone());
    let default_working_dir = user_home_dir
        .clone()
        .unwrap_or_else(|| (*state.working_dir).clone());

    let db = Database::new(&state.db_path)?;
    let session_manager = SessionManager::new(db);

    if !session_manager.verify_session_ownership(session_id, user_id.as_deref())? {
        return Err(AppError::NotFound(format!(
            "Session {} not found",
            session_id
        )));
    }

    let session = session_manager
        .get_session(session_id)?
        .ok_or_else(|| AppError::NotFound(format!("Session {} not found", session_id)))?;

    let working_dir = session
        .working_dir
        .as_deref()
        .map(PathBuf::from)
        .unwrap_or(default_working_dir);

    let session_lock = {
        let mut locks = state.session_locks.write().await;
        if locks.len() > SESSION_LOCK_MAX_ENTRIES {
            locks.retain(|_, (lock, created_at)| {
                created_at.elapsed() < SESSION_LOCK_MAX_AGE || Arc::strong_count(lock) > 1
            });
        }
        let (lock, _) = locks
            .entry(session_id.to_string())
            .or_insert_with(|| (Arc::new(Mutex::new(())), Instant::now()));
        lock.clone()
    };
    let guard = Arc::clone(&session_lock)
        .try_lock_owned()
        .map_err(|_| AppError::Conflict(format!("Session {} is busy", session_id)))?;

    let raw_messages = session_manager.load_session_messages(session_id)?;
    let conversation: Vec<ModelMessage> = raw_messages
        .into_iter()
        .filter_map(|(role_str, content_json)| {
            let role = match role_str.as_str() {
                "user" => Role::User,
                "assistant" => Role::Assistant,
                _ => return None,
            };
            serde_json::from_str(&content_json)
                .ok()
                .map(|content| ModelMessage { role, content })
        })
        .collect();

    let ai_client = if let Some(requested_model) = model_override {
        let mut cfg = base_ai_client.config().clone();
        cfg.model = requested_model.to_string();
        Arc::new(AiClient::new(cfg, base_ai_client.api_key().to_string()))
    } else {
        base_ai_client.clone()
    };

    let ai_tools = state.tool_registry.get_ai_tools().await;
    let mut options = CallOptions {
        tools: Some(ai_tools),
        session_id: Some(session_id.to_string()),
        codex_parallel_tool_calls: true,
        ..Default::default()
    };
    if enable_thinking {
        apply_thinking_config(&ai_client, &mut options);
    }

    Ok(ChatSessionContext {
        ai_client,
        base_ai_client,
        options,
        conversation,
        session_id: session_id.to_string(),
        session_manager,
        working_dir,
        work_mode: session.work_mode,
        user_id,
        guard,
    })
}

async fn chat(
    State(state): State<AppState>,
    user: Option<CurrentUser>,
    Json(req): Json<ChatRequest>,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, AppError> {
    let user_id = user.as_ref().and_then(|u| u.0.user_id.clone());
    let default_working_dir = user
        .as_ref()
        .and_then(|u| u.0.home_dir.clone())
        .unwrap_or_else(|| (*state.working_dir).clone());

    let (session_id, is_first_message) = match req.session_id {
        Some(id) => {
            let db = Database::new(&state.db_path)?;
            let sm = SessionManager::new(db);
            if !sm.verify_session_ownership(&id, user_id.as_deref())? {
                return Err(AppError::NotFound(format!("Session {} not found", id)));
            }
            // Update model if provided in request
            if let Some(ref model) = req.model {
                let normalized = if model.is_empty() {
                    None
                } else {
                    Some(model.as_str())
                };
                sm.update_session_model(&id, normalized)?;
            }
            let messages = sm.load_session_messages(&id)?;
            (id, messages.is_empty())
        }
        None => {
            let db = Database::new(&state.db_path)?;
            let sm = SessionManager::new(db);
            let title = SessionManager::generate_title_from_content(&req.message);
            let working_dir_str = default_working_dir.to_string_lossy().to_string();
            let id = sm.create_session_for_user(
                &title,
                req.model.as_deref(),
                Some(working_dir_str.as_str()),
                user_id.as_deref(),
            )?;
            (id, true)
        }
    };

    let mut ctx = setup_chat_session(
        &state,
        user.as_ref(),
        &session_id,
        req.model.as_deref(),
        req.thinking_enabled,
    )
    .await?;

    let mut work_mode = ctx.work_mode;
    if let Some(requested_mode) = req.mode {
        if requested_mode != work_mode {
            ctx.session_manager
                .update_session_work_mode(&session_id, requested_mode)?;
            work_mode = requested_mode;
        }
    }

    let permission_mode = req.permission_mode;

    let first_message_for_title = if is_first_message {
        Some(req.message.clone())
    } else {
        None
    };

    // Build user content from content blocks (images) + message (text fallback)
    let user_content = build_user_content(&req.message, &req.content);
    let user_content_json = serde_json::to_string(&user_content)?;

    ctx.conversation.push(ModelMessage {
        role: Role::User,
        content: user_content,
    });
    ctx.session_manager
        .save_message(&session_id, "user", &user_content_json)?;

    let (sse_tx, sse_rx) = mpsc::channel::<Result<Event, Infallible>>(SSE_CHANNEL_BUFFER);
    let title_sse_tx = if first_message_for_title.is_some() {
        Some(sse_tx.clone())
    } else {
        None
    };

    let tool_registry = Arc::clone(&state.tool_registry);
    let process_registry = Arc::clone(&state.process_registry);
    let db_path = Arc::clone(&state.db_path);
    let pending_approvals = Arc::clone(&state.pending_approvals);
    let push_service = state.push_service.clone();

    let ChatSessionContext {
        ai_client,
        base_ai_client,
        options,
        conversation,
        session_id: ctx_session_id,
        working_dir,
        user_id: ctx_user_id,
        guard,
        ..
    } = ctx;

    tokio::spawn(async move {
        let _guard = guard;
        run_agentic_loop(AgenticLoopContext {
            ai_client,
            tool_registry,
            process_registry,
            sse_tx,
            conversation,
            options,
            session_id: ctx_session_id,
            db_path,
            working_dir,
            user_id: ctx_user_id,
            work_mode,
            permission_mode,
            pending_approvals,
            push_service,
        })
        .await
    });

    if let (Some(first_message), Some(title_tx)) = (first_message_for_title, title_sse_tx) {
        let title_ai_client = state.ai_client.as_ref().cloned().unwrap_or_else(|| {
            Arc::new(AiClient::new(
                base_ai_client.config().clone(),
                base_ai_client.api_key().to_string(),
            ))
        });
        let title_db_path = Arc::clone(&state.db_path);
        let title_session_id = session_id;

        tokio::spawn(async move {
            let title = generate_title(&title_ai_client, &first_message).await;
            match Database::new(&title_db_path) {
                Ok(db) => {
                    let sm = SessionManager::new(db);
                    if let Err(e) = sm.update_session_title(&title_session_id, &title) {
                        tracing::error!(
                            session_id = %title_session_id,
                            "Failed to update session title: {}", e
                        );
                    }
                }
                Err(e) => {
                    tracing::error!(
                        session_id = %title_session_id,
                        "Failed to open database for title update: {}", e
                    );
                }
            }
            let event = AgenticEvent::TitleUpdate { title };
            match serde_json::to_string(&event) {
                Ok(json) => {
                    let _ = title_tx.send(Ok(Event::default().data(json))).await;
                }
                Err(e) => {
                    tracing::error!("Failed to serialize title update event: {}", e);
                }
            }
        });
    }

    let stream = ReceiverStream::new(sse_rx);
    Ok(Sse::new(stream).keep_alive(KeepAlive::default()))
}

async fn tool_result(
    State(state): State<AppState>,
    user: Option<CurrentUser>,
    Json(req): Json<ToolResultRequest>,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, AppError> {
    let mut ctx = setup_chat_session(&state, user.as_ref(), &req.session_id, None, false).await?;

    if req.tool_call_id.starts_with("plan-confirm-") {
        if let Some(choice) = parse_plan_confirm_choice(&req.result) {
            if choice == "execute" {
                ctx.session_manager
                    .update_session_work_mode(&req.session_id, WorkMode::Build)?;
                ctx.work_mode = WorkMode::Build;
            } else if choice == "abandon" {
                if let Ok(plan_manager) = PlanManager::new((*state.db_path).clone()) {
                    let _ = plan_manager.abandon_plan(&req.session_id);
                }
            }
        }
    }

    let has_thinking = ctx.conversation.iter().any(|msg| {
        msg.content
            .iter()
            .any(|c| matches!(c, Content::Thinking { .. }))
    });
    if has_thinking {
        apply_thinking_config(&ctx.ai_client, &mut ctx.options);
    }

    // Check if the last user message already has a placeholder tool_result for
    // this tool_use_id (happens when AskUser was batched with other tools).
    // If so, replace the placeholder in-place instead of creating a new message.
    let merged = if let Some(last_msg) = ctx.conversation.last_mut() {
        if last_msg.role == Role::User
            && last_msg.content.iter().any(|c| {
                matches!(c, Content::ToolResult { tool_use_id, .. } if tool_use_id == &req.tool_call_id)
            })
        {
            // Replace the placeholder with the real answer
            for c in &mut last_msg.content {
                if let Content::ToolResult {
                    tool_use_id,
                    output,
                    ..
                } = c
                {
                    if tool_use_id == &req.tool_call_id {
                        *output = serde_json::Value::String(req.result.clone());
                        break;
                    }
                }
            }
            // Re-save the updated message
            let updated_json = serde_json::to_string(&last_msg.content)?;
            ctx.session_manager
                .update_last_message(&req.session_id, "user", &updated_json)?;
            true
        } else {
            false
        }
    } else {
        false
    };

    if !merged {
        let tool_result_content = vec![Content::ToolResult {
            tool_use_id: req.tool_call_id.clone(),
            output: serde_json::Value::String(req.result.clone()),
            is_error: None,
        }];
        let tool_result_json = serde_json::to_string(&tool_result_content)?;
        ctx.conversation.push(ModelMessage {
            role: Role::User,
            content: tool_result_content,
        });
        ctx.session_manager
            .save_message(&req.session_id, "user", &tool_result_json)?;
    }

    let (sse_tx, sse_rx) = mpsc::channel::<Result<Event, Infallible>>(SSE_CHANNEL_BUFFER);

    let tool_registry = Arc::clone(&state.tool_registry);
    let process_registry = Arc::clone(&state.process_registry);
    let db_path = Arc::clone(&state.db_path);
    let pending_approvals = Arc::clone(&state.pending_approvals);
    let push_service = state.push_service.clone();

    let ChatSessionContext {
        ai_client,
        options,
        conversation,
        session_id,
        working_dir,
        work_mode,
        user_id,
        guard,
        ..
    } = ctx;

    tokio::spawn(async move {
        let _guard = guard;
        run_agentic_loop(AgenticLoopContext {
            ai_client,
            tool_registry,
            process_registry,
            sse_tx,
            conversation,
            options,
            session_id,
            db_path,
            working_dir,
            user_id,
            work_mode,
            permission_mode: PermissionMode::Autonomous,
            pending_approvals,
            push_service,
        })
        .await
    });

    let stream = ReceiverStream::new(sse_rx);
    Ok(Sse::new(stream).keep_alive(KeepAlive::default()))
}

async fn tool_approval(
    State(state): State<AppState>,
    Json(req): Json<ToolApprovalRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let mut approvals = state.pending_approvals.write().await;
    let sender = approvals
        .remove(&req.tool_call_id)
        .ok_or_else(|| AppError::NotFound("No pending approval".into()))?;
    let _ = sender.send(req.approved);
    Ok(Json(json!({"status": "ok"})))
}

fn apply_thinking_config(ai_client: &AiClient, options: &mut CallOptions) {
    let cfg = ai_client.config();
    let is_codex =
        cfg.provider_id == ProviderId::OpenAI && cfg.model.to_ascii_lowercase().contains("codex");

    if is_codex {
        options.codex_reasoning_effort = Some(CodexReasoningEffort::High);
    } else {
        options.thinking = Some(ThinkingConfig {
            budget_tokens: 32000,
        });
    }
}

async fn run_agentic_loop(ctx: AgenticLoopContext) {
    let AgenticLoopContext {
        ai_client,
        tool_registry,
        process_registry,
        sse_tx,
        mut conversation,
        options,
        session_id,
        db_path,
        working_dir,
        user_id,
        mut work_mode,
        permission_mode,
        pending_approvals,
        push_service,
    } = ctx;

    if let Err(e) = Database::new(&db_path) {
        send_event(
            &sse_tx,
            AgenticEvent::Error {
                error: format!("Database error: {}", e),
            },
        )
        .await;
        return;
    }

    let mut client_connected = true;
    let mut last_token_count = 0usize;
    let mut exploration_budget_count = 0usize;
    let mut tool_failure_signatures: HashMap<String, usize> = HashMap::new();
    set_agent_state(db_path.as_ref(), &session_id, "streaming");

    for iteration in 1..=MAX_ITERATIONS {
        if client_connected && sse_tx.is_closed() {
            client_connected = false;
        }

        let api_rx = match ai_client
            .call_streaming(conversation.clone(), &options)
            .await
        {
            Ok(rx) => rx,
            Err(e) => {
                if client_connected {
                    send_event(
                        &sse_tx,
                        AgenticEvent::Error {
                            error: format!("AI error: {}", e),
                        },
                    )
                    .await;
                }
                if last_token_count > 0 {
                    update_token_count(db_path.as_ref(), &session_id, last_token_count);
                }
                set_agent_state(db_path.as_ref(), &session_id, "error");
                fire_push(
                    &push_service,
                    user_id.as_deref(),
                    PushPayload {
                        title: "Krusty".into(),
                        body: "Session encountered an error".into(),
                        session_id: Some(session_id),
                        tag: None,
                    },
                    PushEventType::Error,
                );
                return;
            }
        };

        let (text, thinking_blocks, tool_calls, _finish_reason, prompt_tokens) =
            process_stream(api_rx, &sse_tx).await;

        if prompt_tokens > 0 {
            last_token_count = prompt_tokens;
        }

        let assistant_msg = build_assistant_message(&text, &thinking_blocks, &tool_calls);
        if !assistant_msg.content.is_empty() {
            conversation.push(assistant_msg.clone());
            save_message(db_path.as_ref(), &session_id, &assistant_msg);
        }

        if tool_calls.is_empty() {
            if work_mode == WorkMode::Plan {
                if let Some(mut plan) = try_detect_plan(&text) {
                    plan.plan_file.session_id = Some(session_id.clone());
                    plan.plan_file.working_dir = Some(working_dir.to_string_lossy().to_string());

                    match PlanManager::new((*db_path).clone()) {
                        Ok(plan_manager) => {
                            if let Err(e) =
                                plan_manager.save_plan_for_session(&session_id, &plan.plan_file)
                            {
                                tracing::warn!(
                                    session_id = %session_id,
                                    "Failed to save detected plan: {}", e
                                );
                            }
                        }
                        Err(e) => {
                            tracing::warn!(
                                session_id = %session_id,
                                "Failed to initialize plan manager for detected plan: {}", e
                            );
                        }
                    }

                    if client_connected {
                        send_event(
                            &sse_tx,
                            AgenticEvent::PlanUpdate {
                                items: plan.tasks.clone(),
                            },
                        )
                        .await;
                        let tool_call_id = format!("plan-confirm-{}", uuid::Uuid::new_v4());
                        send_event(
                            &sse_tx,
                            AgenticEvent::PlanComplete {
                                tool_call_id: tool_call_id.clone(),
                                title: plan.title.clone(),
                                task_count: plan.tasks.len(),
                            },
                        )
                        .await;
                        send_event(
                            &sse_tx,
                            AgenticEvent::AwaitingInput {
                                tool_call_id,
                                tool_name: "PlanConfirm".to_string(),
                            },
                        )
                        .await;
                    }
                    if last_token_count > 0 {
                        update_token_count(db_path.as_ref(), &session_id, last_token_count);
                    }
                    set_agent_state(db_path.as_ref(), &session_id, "awaiting_input");
                    if client_connected {
                        send_event(
                            &sse_tx,
                            AgenticEvent::Finish {
                                session_id: session_id.clone(),
                            },
                        )
                        .await;
                    }
                    fire_push(
                        &push_service,
                        user_id.as_deref(),
                        PushPayload {
                            title: "Krusty".into(),
                            body: "Krusty needs your input".into(),
                            session_id: Some(session_id),
                            tag: None,
                        },
                        PushEventType::AwaitingInput,
                    );
                    return;
                }
            }

            if client_connected {
                send_event(
                    &sse_tx,
                    AgenticEvent::TurnComplete {
                        turn: iteration,
                        has_more: false,
                    },
                )
                .await;
            }
            break;
        }

        let (ask_user_calls, non_ask_user_calls): (Vec<_>, Vec<_>) = tool_calls
            .iter()
            .partition::<Vec<_>, _>(|t| t.name == "AskUserQuestion");

        if !ask_user_calls.is_empty() {
            // When AskUser appears alongside other tool calls, execute the
            // others first and save all results (including a placeholder for
            // AskUser) in a single user message. This keeps every tool_use
            // paired with a tool_result, satisfying the Anthropic API.
            // The real AskUser answer arrives later via /tool-result which
            // merges it into this message, replacing the placeholder.
            let mut all_results: Vec<Content> = Vec::new();

            if !non_ask_user_calls.is_empty() {
                let other_calls: Vec<AiToolCall> =
                    non_ask_user_calls.into_iter().cloned().collect();
                set_agent_state(db_path.as_ref(), &session_id, "tool_executing");
                let (other_results, _next) = execute_tools(ExecuteToolsContext {
                    tool_registry: &tool_registry,
                    tool_calls: &other_calls,
                    working_dir: &working_dir,
                    process_registry: &process_registry,
                    sse_tx: &sse_tx,
                    user_id: user_id.as_deref(),
                    permission_mode,
                    pending_approvals: &pending_approvals,
                    session_id: &session_id,
                    db_path: db_path.as_ref().as_path(),
                    current_mode: work_mode,
                })
                .await;
                all_results.extend(other_results);
            }

            // Add placeholder results for each AskUser call so the
            // conversation has a complete set of tool_results.
            for call in &ask_user_calls {
                all_results.push(Content::ToolResult {
                    tool_use_id: call.id.clone(),
                    output: serde_json::Value::String("Awaiting user response...".to_string()),
                    is_error: None,
                });
            }

            let tool_msg = ModelMessage {
                role: Role::User,
                content: all_results,
            };
            conversation.push(tool_msg.clone());
            save_message(db_path.as_ref(), &session_id, &tool_msg);

            if client_connected {
                for call in &ask_user_calls {
                    send_event(
                        &sse_tx,
                        AgenticEvent::AwaitingInput {
                            tool_call_id: call.id.clone(),
                            tool_name: call.name.clone(),
                        },
                    )
                    .await;
                }
            }
            if last_token_count > 0 {
                update_token_count(db_path.as_ref(), &session_id, last_token_count);
            }
            set_agent_state(db_path.as_ref(), &session_id, "awaiting_input");
            if client_connected {
                send_event(
                    &sse_tx,
                    AgenticEvent::Finish {
                        session_id: session_id.clone(),
                    },
                )
                .await;
            }
            fire_push(
                &push_service,
                user_id.as_deref(),
                PushPayload {
                    title: "Krusty".into(),
                    body: "Krusty needs your input".into(),
                    session_id: Some(session_id),
                    tag: None,
                },
                PushEventType::AwaitingInput,
            );
            return;
        }

        let all_readonly = tool_calls
            .iter()
            .all(|t| matches!(t.name.as_str(), "read" | "glob" | "grep"));
        let has_action = tool_calls.iter().any(|t| {
            matches!(
                t.name.as_str(),
                "edit"
                    | "write"
                    | "bash"
                    | "build"
                    | "task_start"
                    | "task_complete"
                    | "add_subtask"
                    | "set_dependency"
                    | "set_work_mode"
                    | "enter_plan_mode"
            )
        });
        if has_action {
            exploration_budget_count = 0;
        } else if all_readonly {
            exploration_budget_count += tool_calls.len();
        }

        set_agent_state(db_path.as_ref(), &session_id, "tool_executing");
        let (tool_results, next_work_mode) = execute_tools(ExecuteToolsContext {
            tool_registry: &tool_registry,
            tool_calls: &tool_calls,
            working_dir: &working_dir,
            process_registry: &process_registry,
            sse_tx: &sse_tx,
            user_id: user_id.as_deref(),
            permission_mode,
            pending_approvals: &pending_approvals,
            session_id: &session_id,
            db_path: db_path.as_ref().as_path(),
            current_mode: work_mode,
        })
        .await;
        work_mode = next_work_mode;

        let fail_fast_diagnostic =
            detect_repeated_tool_failures(&mut tool_failure_signatures, &tool_calls, &tool_results);

        if exploration_budget_count >= EXPLORATION_BUDGET_HARD {
            tracing::warn!(
                exploration_budget_count,
                "Exploration budget hard threshold reached"
            );
        } else if exploration_budget_count >= EXPLORATION_BUDGET_SOFT {
            tracing::info!(
                exploration_budget_count,
                "Exploration budget soft threshold reached"
            );
        }

        let tool_msg = ModelMessage {
            role: Role::User,
            content: tool_results,
        };
        conversation.push(tool_msg.clone());
        save_message(db_path.as_ref(), &session_id, &tool_msg);

        if let Some(diagnostic) = fail_fast_diagnostic {
            tracing::warn!(
                threshold = REPEATED_TOOL_FAILURE_THRESHOLD,
                iteration,
                session_id = %session_id,
                diagnostic = %diagnostic,
                "Fail-fast: stopping repeated tool failure loop"
            );
            set_agent_state(db_path.as_ref(), &session_id, "idle");

            if client_connected && !sse_tx.is_closed() {
                send_event(
                    &sse_tx,
                    AgenticEvent::Error {
                        error: diagnostic.clone(),
                    },
                )
                .await;
                send_event(
                    &sse_tx,
                    AgenticEvent::TurnComplete {
                        turn: iteration,
                        has_more: false,
                    },
                )
                .await;
            }
            break;
        }

        set_agent_state(db_path.as_ref(), &session_id, "streaming");

        if client_connected && !sse_tx.is_closed() {
            send_event(
                &sse_tx,
                AgenticEvent::TurnComplete {
                    turn: iteration,
                    has_more: true,
                },
            )
            .await;
        } else {
            client_connected = false;
        }
    }

    if last_token_count > 0 {
        update_token_count(db_path.as_ref(), &session_id, last_token_count);
    }
    set_agent_state(db_path.as_ref(), &session_id, "idle");

    if client_connected && !sse_tx.is_closed() {
        send_event(
            &sse_tx,
            AgenticEvent::Finish {
                session_id: session_id.clone(),
            },
        )
        .await;
    }

    let title = session_title(db_path.as_ref(), &session_id);

    fire_push(
        &push_service,
        user_id.as_deref(),
        PushPayload {
            title: "Krusty".into(),
            body: format!("{title} is complete"),
            session_id: Some(session_id.clone()),
            tag: Some(format!("session-{session_id}")),
        },
        PushEventType::Completion,
    );
}

struct ThinkingBlock {
    thinking: String,
    signature: String,
}

async fn process_stream(
    mut api_rx: mpsc::UnboundedReceiver<StreamPart>,
    sse_tx: &mpsc::Sender<Result<Event, Infallible>>,
) -> (
    String,
    Vec<ThinkingBlock>,
    Vec<AiToolCall>,
    FinishReason,
    usize,
) {
    let mut text_buffer = String::new();
    let mut thinking_blocks = Vec::new();
    let mut tool_calls = Vec::new();
    let mut finish_reason = FinishReason::Stop;
    let mut prompt_tokens = 0usize;
    let mut client_alive = true;

    loop {
        let part = match tokio::time::timeout(AI_STREAM_TIMEOUT, api_rx.recv()).await {
            Ok(Some(part)) => part,
            Ok(None) => break,
            Err(_) => {
                if client_alive {
                    send_event(
                        sse_tx,
                        AgenticEvent::Error {
                            error: "AI stream timeout: no data received for 120 seconds"
                                .to_string(),
                        },
                    )
                    .await;
                }
                break;
            }
        };

        if client_alive && sse_tx.is_closed() {
            client_alive = false;
        }

        match &part {
            StreamPart::TextDelta { delta } => {
                text_buffer.push_str(delta);
                if client_alive {
                    send_event(
                        sse_tx,
                        AgenticEvent::TextDelta {
                            delta: delta.clone(),
                        },
                    )
                    .await;
                }
            }
            StreamPart::ThinkingDelta { thinking, .. } => {
                if client_alive {
                    send_event(
                        sse_tx,
                        AgenticEvent::ThinkingDelta {
                            thinking: thinking.clone(),
                        },
                    )
                    .await;
                }
            }
            StreamPart::ThinkingComplete {
                thinking,
                signature,
                ..
            } => {
                thinking_blocks.push(ThinkingBlock {
                    thinking: thinking.clone(),
                    signature: signature.clone(),
                });
            }
            StreamPart::ToolCallStart { id, name } => {
                if client_alive {
                    send_event(
                        sse_tx,
                        AgenticEvent::ToolCallStart {
                            id: id.clone(),
                            name: name.clone(),
                        },
                    )
                    .await;
                }
            }
            StreamPart::ToolCallComplete { tool_call } => {
                tool_calls.push(tool_call.clone());
                if client_alive {
                    send_event(
                        sse_tx,
                        AgenticEvent::ToolCallComplete {
                            id: tool_call.id.clone(),
                            name: tool_call.name.clone(),
                            arguments: tool_call.arguments.clone(),
                        },
                    )
                    .await;
                }
            }
            StreamPart::Finish { reason } => finish_reason = reason.clone(),
            StreamPart::Usage { usage } => {
                prompt_tokens = usage.prompt_tokens + usage.completion_tokens;
                if client_alive {
                    send_event(
                        sse_tx,
                        AgenticEvent::Usage {
                            prompt_tokens,
                            completion_tokens: usage.completion_tokens,
                        },
                    )
                    .await;
                }
            }
            StreamPart::Error { error } => {
                if client_alive {
                    send_event(
                        sse_tx,
                        AgenticEvent::Error {
                            error: error.clone(),
                        },
                    )
                    .await;
                }
            }
            _ => {}
        }
    }

    (
        text_buffer,
        thinking_blocks,
        tool_calls,
        finish_reason,
        prompt_tokens,
    )
}

async fn execute_tools(ctx: ExecuteToolsContext<'_>) -> (Vec<Content>, WorkMode) {
    let ExecuteToolsContext {
        tool_registry,
        tool_calls,
        working_dir,
        process_registry,
        sse_tx,
        user_id,
        permission_mode,
        pending_approvals,
        session_id,
        db_path,
        current_mode,
    } = ctx;

    let mut work_mode = current_mode;
    let mut results = Vec::new();

    for call in tool_calls {
        let category = tool_category(&call.name);

        if permission_mode == PermissionMode::Supervised && category == ToolCategory::Write {
            send_event(
                sse_tx,
                AgenticEvent::ToolApprovalRequired {
                    id: call.id.clone(),
                    name: call.name.clone(),
                    arguments: call.arguments.clone(),
                },
            )
            .await;

            let (tx, rx) = oneshot::channel();
            {
                let mut approvals = pending_approvals.write().await;
                approvals.retain(|_, sender| !sender.is_closed());
                approvals.insert(call.id.clone(), tx);
            }

            let approved = match tokio::time::timeout(APPROVAL_TIMEOUT, rx).await {
                Ok(Ok(approved)) => approved,
                Ok(Err(_)) => false,
                Err(_) => {
                    let output = "Tool approval timed out after 5 minutes".to_string();
                    send_event(
                        sse_tx,
                        AgenticEvent::ToolDenied {
                            id: call.id.clone(),
                        },
                    )
                    .await;
                    send_event(
                        sse_tx,
                        AgenticEvent::ToolResult {
                            id: call.id.clone(),
                            output: output.clone(),
                            is_error: true,
                        },
                    )
                    .await;
                    results.push(Content::ToolResult {
                        tool_use_id: call.id.clone(),
                        output: serde_json::Value::String(output),
                        is_error: Some(true),
                    });
                    let mut approvals = pending_approvals.write().await;
                    approvals.remove(&call.id);
                    continue;
                }
            };

            if !approved {
                let output = "Tool execution denied by user".to_string();
                send_event(
                    sse_tx,
                    AgenticEvent::ToolDenied {
                        id: call.id.clone(),
                    },
                )
                .await;
                send_event(
                    sse_tx,
                    AgenticEvent::ToolResult {
                        id: call.id.clone(),
                        output: output.clone(),
                        is_error: true,
                    },
                )
                .await;
                results.push(Content::ToolResult {
                    tool_use_id: call.id.clone(),
                    output: serde_json::Value::String(output),
                    is_error: Some(true),
                });
                continue;
            }

            send_event(
                sse_tx,
                AgenticEvent::ToolApproved {
                    id: call.id.clone(),
                },
            )
            .await;
        }

        send_event(
            sse_tx,
            AgenticEvent::ToolExecuting {
                id: call.id.clone(),
                name: call.name.clone(),
            },
        )
        .await;

        if call.name == "set_work_mode" || call.name == "enter_plan_mode" {
            let (result, next_mode, mode_change_reason) =
                handle_mode_switch_tool_call(call, session_id, db_path, work_mode);
            work_mode = next_mode;

            if let Some(reason) = mode_change_reason {
                send_event(
                    sse_tx,
                    AgenticEvent::ModeChange {
                        mode: work_mode.to_string(),
                        reason: Some(reason),
                    },
                )
                .await;
            }

            let output = truncate_output(&result.output);
            send_event(
                sse_tx,
                AgenticEvent::ToolResult {
                    id: call.id.clone(),
                    output: output.clone(),
                    is_error: result.is_error,
                },
            )
            .await;
            results.push(Content::ToolResult {
                tool_use_id: call.id.clone(),
                output: serde_json::Value::String(output),
                is_error: if result.is_error { Some(true) } else { None },
            });
            continue;
        }

        if matches!(
            call.name.as_str(),
            "task_start" | "task_complete" | "add_subtask" | "set_dependency"
        ) {
            let result = handle_plan_task_tool_call(call, session_id, db_path);
            let output = truncate_output(&result.output);
            send_event(
                sse_tx,
                AgenticEvent::ToolResult {
                    id: call.id.clone(),
                    output: output.clone(),
                    is_error: result.is_error,
                },
            )
            .await;
            results.push(Content::ToolResult {
                tool_use_id: call.id.clone(),
                output: serde_json::Value::String(output),
                is_error: if result.is_error { Some(true) } else { None },
            });
            continue;
        }

        let (output_tx, mut output_rx) =
            mpsc::unbounded_channel::<krusty_core::tools::registry::ToolOutputChunk>();

        let forwarder_sse_tx = sse_tx.clone();
        let forwarder_tool_id = call.id.clone();
        let forwarder_tool_name = call.name.clone();
        let forwarder_handle = tokio::spawn(async move {
            let mut heartbeat_interval = tokio::time::interval(HEARTBEAT_INTERVAL);
            heartbeat_interval.tick().await;

            loop {
                tokio::select! {
                    chunk = output_rx.recv() => {
                        match chunk {
                            Some(chunk) => {
                                if !chunk.chunk.is_empty() {
                                    send_event(
                                        &forwarder_sse_tx,
                                        AgenticEvent::ToolOutputDelta {
                                            id: forwarder_tool_id.clone(),
                                            delta: chunk.chunk,
                                        },
                                    )
                                    .await;
                                }
                                if chunk.is_complete {
                                    break;
                                }
                            }
                            None => break,
                        }
                    }
                    _ = heartbeat_interval.tick() => {
                        if !send_event(
                            &forwarder_sse_tx,
                            AgenticEvent::ToolExecuting {
                                id: forwarder_tool_id.clone(),
                                name: forwarder_tool_name.clone(),
                            },
                        )
                        .await
                        {
                            break;
                        }
                    }
                }
            }
        });

        let ctx = ToolContext {
            working_dir: working_dir.to_path_buf(),
            process_registry: Some(process_registry.clone()),
            plan_mode: work_mode == WorkMode::Plan,
            user_id: user_id.map(ToString::to_string),
            sandbox_root: Some(working_dir.to_path_buf()),
            ..Default::default()
        }
        .with_output_stream(output_tx, call.id.clone());

        let result = tool_registry
            .execute(&call.name, call.arguments.clone(), &ctx)
            .await
            .unwrap_or_else(|| krusty_core::tools::registry::ToolResult {
                output: format!("Unknown tool: {}", call.name),
                is_error: true,
            });

        // Drop ctx so the forwarder task sees channel closed and exits.
        // Without this, non-streaming tools deadlock: forwarder_handle.await
        // waits for the forwarder, but the forwarder waits for output_rx to
        // close, which requires output_tx (in ctx) to be dropped.
        drop(ctx);

        let _ = forwarder_handle.await;

        let output = truncate_output(&result.output);

        send_event(
            sse_tx,
            AgenticEvent::ToolResult {
                id: call.id.clone(),
                output: output.clone(),
                is_error: result.is_error,
            },
        )
        .await;

        results.push(Content::ToolResult {
            tool_use_id: call.id.clone(),
            output: serde_json::Value::String(output),
            is_error: if result.is_error { Some(true) } else { None },
        });
    }

    (results, work_mode)
}

fn handle_mode_switch_tool_call(
    call: &AiToolCall,
    session_id: &str,
    db_path: &Path,
    current_mode: WorkMode,
) -> (
    krusty_core::tools::registry::ToolResult,
    WorkMode,
    Option<String>,
) {
    let clear_existing = call
        .arguments
        .get("clear_existing")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let (target_mode, fallback_reason) = if call.name == "enter_plan_mode" {
        (WorkMode::Plan, "Starting planning phase")
    } else {
        let Some(mode) = call.arguments.get("mode").and_then(|v| v.as_str()) else {
            return (
                krusty_core::tools::registry::ToolResult {
                    output: "Error: mode parameter is required (build|plan)".to_string(),
                    is_error: true,
                },
                current_mode,
                None,
            );
        };
        let parsed_mode = match mode {
            "build" => WorkMode::Build,
            "plan" => WorkMode::Plan,
            other => {
                return (
                    krusty_core::tools::registry::ToolResult {
                        output: format!("Error: invalid mode '{}'. Use 'build' or 'plan'.", other),
                        is_error: true,
                    },
                    current_mode,
                    None,
                );
            }
        };
        let fallback_reason = if parsed_mode == WorkMode::Plan {
            "Starting planning phase"
        } else {
            "Starting implementation phase"
        };
        (parsed_mode, fallback_reason)
    };

    let reason = call
        .arguments
        .get("reason")
        .and_then(|v| v.as_str())
        .unwrap_or(fallback_reason)
        .to_string();

    let mut clear_plan_note = String::new();
    if clear_existing && target_mode == WorkMode::Plan {
        match PlanManager::new(db_path.to_path_buf()) {
            Ok(plan_manager) => {
                if let Err(e) = plan_manager.abandon_plan(session_id) {
                    clear_plan_note = format!("\n\nWarning: failed to clear existing plan: {}", e);
                } else {
                    clear_plan_note = "\n\nCleared any existing active plan.".to_string();
                }
            }
            Err(e) => {
                clear_plan_note = format!("\n\nWarning: failed to initialize plan manager: {}", e);
            }
        }
    }

    let mut next_mode = current_mode;
    let mut mode_change_reason = None;
    if target_mode != current_mode {
        let session_manager = match Database::new(db_path) {
            Ok(db) => SessionManager::new(db),
            Err(e) => {
                return (
                    krusty_core::tools::registry::ToolResult {
                        output: format!("Error: failed to open database for mode switch: {}", e),
                        is_error: true,
                    },
                    current_mode,
                    None,
                );
            }
        };
        if let Err(e) = session_manager.update_session_work_mode(session_id, target_mode) {
            return (
                krusty_core::tools::registry::ToolResult {
                    output: format!("Error: failed to switch work mode: {}", e),
                    is_error: true,
                },
                current_mode,
                None,
            );
        }
        next_mode = target_mode;
        mode_change_reason = Some(reason.clone());
    }

    let output = if target_mode == WorkMode::Plan {
        format!(
            "Now in Plan mode. {}\n\nCreate a phase-based checkbox plan before making changes.{}",
            reason, clear_plan_note
        )
    } else {
        format!(
            "Now in Build mode. {}\n\nProceed with implementation and keep plan task status updated.{}",
            reason, clear_plan_note
        )
    };

    (
        krusty_core::tools::registry::ToolResult {
            output,
            is_error: false,
        },
        next_mode,
        mode_change_reason,
    )
}

fn handle_plan_task_tool_call(
    call: &AiToolCall,
    session_id: &str,
    db_path: &Path,
) -> krusty_core::tools::registry::ToolResult {
    let plan_manager = match PlanManager::new(db_path.to_path_buf()) {
        Ok(manager) => manager,
        Err(e) => {
            return krusty_core::tools::registry::ToolResult {
                output: format!("Error: failed to initialize plan manager: {}", e),
                is_error: true,
            };
        }
    };

    let mut plan = match plan_manager.get_plan(session_id) {
        Ok(Some(plan)) => plan,
        Ok(None) => {
            return krusty_core::tools::registry::ToolResult {
                output: "Error: No active plan. Create a plan first.".to_string(),
                is_error: true,
            };
        }
        Err(e) => {
            return krusty_core::tools::registry::ToolResult {
                output: format!("Error: failed to load plan: {}", e),
                is_error: true,
            };
        }
    };

    match call.name.as_str() {
        "task_start" => {
            let Some(task_id) = call.arguments.get("task_id").and_then(|v| v.as_str()) else {
                return krusty_core::tools::registry::ToolResult {
                    output: "Error: task_id required".to_string(),
                    is_error: true,
                };
            };

            match plan.start_task(task_id) {
                Ok(()) => {
                    if let Err(e) = plan_manager.save_plan_for_session(session_id, &plan) {
                        return krusty_core::tools::registry::ToolResult {
                            output: format!("Error: failed to save plan: {}", e),
                            is_error: true,
                        };
                    }
                    krusty_core::tools::registry::ToolResult {
                        output: format!("Started task {}. Status: in_progress", task_id),
                        is_error: false,
                    }
                }
                Err(e) => krusty_core::tools::registry::ToolResult {
                    output: format!("Error: {}", e),
                    is_error: true,
                },
            }
        }
        "task_complete" => {
            let result_text = call
                .arguments
                .get("result")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            if result_text.is_empty() {
                return krusty_core::tools::registry::ToolResult {
                    output: "Error: 'result' parameter is required. Describe what you accomplished for this specific task.".to_string(),
                    is_error: true,
                };
            }

            if call
                .arguments
                .get("task_ids")
                .and_then(|v| v.as_array())
                .is_some()
            {
                return krusty_core::tools::registry::ToolResult {
                    output: "Error: Batch completion (task_ids) is not allowed. Complete ONE task at a time with task_id. This ensures focused, quality work.".to_string(),
                    is_error: true,
                };
            }

            let Some(task_id) = call.arguments.get("task_id").and_then(|v| v.as_str()) else {
                return krusty_core::tools::registry::ToolResult {
                    output: "Error: task_id required. Specify which task you're completing."
                        .to_string(),
                    is_error: true,
                };
            };

            let task_status = plan.find_task(task_id).map(|t| t.status);
            match task_status {
                None => {
                    return krusty_core::tools::registry::ToolResult {
                        output: format!("Error: Task '{}' not found in plan.", task_id),
                        is_error: true,
                    };
                }
                Some(TaskStatus::Completed) => {
                    return krusty_core::tools::registry::ToolResult {
                        output: format!("Error: Task '{}' is already completed.", task_id),
                        is_error: true,
                    };
                }
                Some(TaskStatus::Blocked) => {
                    return krusty_core::tools::registry::ToolResult {
                        output: format!(
                            "Error: Task '{}' is blocked. Complete its dependencies first, then use task_start.",
                            task_id
                        ),
                        is_error: true,
                    };
                }
                Some(TaskStatus::Pending) => {
                    return krusty_core::tools::registry::ToolResult {
                        output: format!(
                            "Error: Task '{}' was not started. Use task_start(\"{}\") first, do the work, then complete it.",
                            task_id, task_id
                        ),
                        is_error: true,
                    };
                }
                Some(TaskStatus::InProgress) => {}
            }

            if let Err(e) = plan.complete_task(task_id, &result_text) {
                return krusty_core::tools::registry::ToolResult {
                    output: format!("Error: {}", e),
                    is_error: true,
                };
            }
            if let Err(e) = plan_manager.save_plan_for_session(session_id, &plan) {
                return krusty_core::tools::registry::ToolResult {
                    output: format!("Error: failed to save plan: {}", e),
                    is_error: true,
                };
            }

            let (completed, total) = plan.progress();
            let mut msg = format!(
                "Completed task {}. Progress: {}/{}",
                task_id, completed, total
            );
            if completed == total {
                msg.push_str("\n\nAll tasks complete. Plan finished.");
            } else {
                let ready = plan.get_ready_tasks();
                if !ready.is_empty() {
                    msg.push_str("\n\nReady to work on next:");
                    for task in &ready {
                        msg.push_str(&format!("\n  → Task {}: {}", task.id, task.description));
                    }
                    msg.push_str("\n\nPick one and call task_start immediately.");
                } else {
                    msg.push_str("\n\nNo tasks currently unblocked. Check dependencies.");
                }
            }

            krusty_core::tools::registry::ToolResult {
                output: msg,
                is_error: false,
            }
        }
        "add_subtask" => {
            let parent_id = call
                .arguments
                .get("parent_id")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let description = call
                .arguments
                .get("description")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let context = call.arguments.get("context").and_then(|v| v.as_str());

            if parent_id.is_empty() || description.is_empty() {
                return krusty_core::tools::registry::ToolResult {
                    output: "Error: parent_id and description required".to_string(),
                    is_error: true,
                };
            }

            match plan.add_subtask(parent_id, description, context) {
                Ok(subtask_id) => {
                    if let Err(e) = plan_manager.save_plan_for_session(session_id, &plan) {
                        return krusty_core::tools::registry::ToolResult {
                            output: format!("Error: failed to save plan: {}", e),
                            is_error: true,
                        };
                    }
                    krusty_core::tools::registry::ToolResult {
                        output: format!("Created subtask {} under {}", subtask_id, parent_id),
                        is_error: false,
                    }
                }
                Err(e) => krusty_core::tools::registry::ToolResult {
                    output: format!("Error: {}", e),
                    is_error: true,
                },
            }
        }
        "set_dependency" => {
            let task_id = call
                .arguments
                .get("task_id")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let blocked_by = call
                .arguments
                .get("blocked_by")
                .and_then(|v| v.as_str())
                .unwrap_or("");

            if task_id.is_empty() || blocked_by.is_empty() {
                return krusty_core::tools::registry::ToolResult {
                    output: "Error: task_id and blocked_by required".to_string(),
                    is_error: true,
                };
            }

            match plan.add_dependency(task_id, blocked_by) {
                Ok(()) => {
                    if let Err(e) = plan_manager.save_plan_for_session(session_id, &plan) {
                        return krusty_core::tools::registry::ToolResult {
                            output: format!("Error: failed to save plan: {}", e),
                            is_error: true,
                        };
                    }
                    krusty_core::tools::registry::ToolResult {
                        output: format!("Task {} is now blocked by {}", task_id, blocked_by),
                        is_error: false,
                    }
                }
                Err(e) => krusty_core::tools::registry::ToolResult {
                    output: format!("Error: {}", e),
                    is_error: true,
                },
            }
        }
        _ => krusty_core::tools::registry::ToolResult {
            output: format!("Error: unsupported plan tool '{}'", call.name),
            is_error: true,
        },
    }
}

fn build_assistant_message(
    text: &str,
    thinking_blocks: &[ThinkingBlock],
    tool_calls: &[AiToolCall],
) -> ModelMessage {
    let mut content = Vec::with_capacity(
        thinking_blocks.len() + tool_calls.len() + usize::from(!text.is_empty()),
    );

    for block in thinking_blocks {
        content.push(Content::Thinking {
            thinking: block.thinking.clone(),
            signature: block.signature.clone(),
        });
    }

    if !text.is_empty() {
        content.push(Content::Text {
            text: text.to_string(),
        });
    }

    for call in tool_calls {
        content.push(Content::ToolUse {
            id: call.id.clone(),
            name: call.name.clone(),
            input: call.arguments.clone(),
        });
    }

    ModelMessage {
        role: Role::Assistant,
        content,
    }
}

fn save_message(db_path: &Path, session_id: &str, message: &ModelMessage) {
    let role = match message.role {
        Role::User => "user",
        Role::Assistant => "assistant",
        _ => return,
    };

    match serde_json::to_string(&message.content) {
        Ok(json) => match Database::new(db_path) {
            Ok(db) => {
                let session_manager = SessionManager::new(db);
                if let Err(e) = session_manager.save_message(session_id, role, &json) {
                    tracing::error!("Failed to save message: {}", e);
                }
            }
            Err(e) => tracing::error!("Failed to open database while saving message: {}", e),
        },
        Err(e) => tracing::error!("Failed to serialize message: {}", e),
    }
}

fn set_agent_state(db_path: &Path, session_id: &str, state: &str) {
    match Database::new(db_path) {
        Ok(db) => {
            let session_manager = SessionManager::new(db);
            if let Err(e) = session_manager.set_agent_state(session_id, state) {
                tracing::warn!(
                    session_id = %session_id,
                    "Failed to set agent state '{state}': {}", e
                );
            }
        }
        Err(e) => tracing::error!("Failed to open database while setting agent state: {}", e),
    }
}

fn update_token_count(db_path: &Path, session_id: &str, token_count: usize) {
    match Database::new(db_path) {
        Ok(db) => {
            let session_manager = SessionManager::new(db);
            if let Err(e) = session_manager.update_token_count(session_id, token_count) {
                tracing::warn!(
                    session_id = %session_id,
                    "Failed to update token count: {}", e
                );
            }
        }
        Err(e) => tracing::error!("Failed to open database while updating token count: {}", e),
    }
}

fn session_title(db_path: &Path, session_id: &str) -> String {
    match Database::new(db_path) {
        Ok(db) => {
            let session_manager = SessionManager::new(db);
            match session_manager.get_session(session_id) {
                Ok(Some(session)) => session.title,
                Ok(None) => "Session".to_string(),
                Err(e) => {
                    tracing::warn!(
                        session_id = %session_id,
                        "Failed to load session title: {}", e
                    );
                    "Session".to_string()
                }
            }
        }
        Err(e) => {
            tracing::error!("Failed to open database while loading session title: {}", e);
            "Session".to_string()
        }
    }
}

fn truncate_output(output: &str) -> String {
    if output.len() <= MAX_TOOL_OUTPUT_CHARS {
        return output.to_string();
    }

    let truncated_len = floor_char_boundary(output, MAX_TOOL_OUTPUT_CHARS);
    let truncated = &output[..truncated_len];
    let break_point = truncated.rfind('\n').unwrap_or(truncated_len);
    let clean = &output[..break_point];
    format!(
        "{}\n\n[... OUTPUT TRUNCATED: {} chars -> {} chars ...]",
        clean,
        output.len(),
        clean.len()
    )
}

fn floor_char_boundary(text: &str, index: usize) -> usize {
    let mut boundary = index.min(text.len());
    while boundary > 0 && !text.is_char_boundary(boundary) {
        boundary -= 1;
    }
    boundary
}

/// Fire a push notification in a background task (non-blocking, failure only logged).
fn fire_push(
    push_service: &Option<Arc<PushService>>,
    user_id: Option<&str>,
    payload: PushPayload,
    event_type: PushEventType,
) {
    if let Some(svc) = push_service.clone() {
        let uid = user_id.map(String::from);
        tokio::spawn(async move {
            let stats = svc.notify_user(uid.as_deref(), payload, event_type).await;
            tracing::info!(
                event_type = event_type.as_str(),
                attempted = stats.attempted,
                sent = stats.sent,
                stale_removed = stats.stale_removed,
                failed = stats.failed,
                "Push event dispatched"
            );
        });
    }
}

async fn send_event(sse_tx: &mpsc::Sender<Result<Event, Infallible>>, event: AgenticEvent) -> bool {
    let sse_event = Event::default()
        .json_data(&event)
        .unwrap_or_else(|_| Event::default().data("error"));
    sse_tx.send(Ok(sse_event)).await.is_ok()
}

struct ParsedPlan {
    title: String,
    tasks: Vec<crate::types::PlanItem>,
    plan_file: PlanFile,
}

fn try_detect_plan(text: &str) -> Option<ParsedPlan> {
    let plan_file = PlanFile::try_parse_from_response(text)?;
    let title = if plan_file.title.trim().is_empty() {
        "Implementation Plan".to_string()
    } else {
        plan_file.title.clone()
    };

    let tasks: Vec<crate::types::PlanItem> = plan_file
        .phases
        .iter()
        .flat_map(|phase| phase.tasks.iter())
        .filter_map(|task| {
            let content = task.description.trim().to_string();
            if content.is_empty() {
                None
            } else {
                Some(crate::types::PlanItem {
                    content,
                    completed: task.completed || task.status == TaskStatus::Completed,
                })
            }
        })
        .collect();

    if tasks.is_empty() {
        return None;
    }

    Some(ParsedPlan {
        title,
        tasks,
        plan_file,
    })
}

fn parse_plan_confirm_choice(raw: &str) -> Option<String> {
    if let Ok(value) = serde_json::from_str::<serde_json::Value>(raw) {
        if let Some(choice) = value.get("choice").and_then(|v| v.as_str()) {
            let normalized = choice.trim().to_ascii_lowercase();
            if normalized == "execute" || normalized == "abandon" {
                return Some(normalized);
            }
        }
    }

    let normalized = raw.trim().to_ascii_lowercase();
    if normalized.contains("execute") {
        Some("execute".to_string())
    } else if normalized.contains("abandon") {
        Some("abandon".to_string())
    } else {
        None
    }
}

fn detect_repeated_tool_failures(
    counters: &mut HashMap<String, usize>,
    tool_calls: &[AiToolCall],
    tool_results: &[Content],
) -> Option<String> {
    let mut call_meta: HashMap<&str, (String, u64)> = HashMap::new();
    for call in tool_calls {
        call_meta.insert(
            call.id.as_str(),
            (call.name.clone(), hash_tool_arguments(&call.arguments)),
        );
    }

    let mut saw_success = false;

    for result in tool_results {
        let Content::ToolResult {
            tool_use_id,
            output,
            is_error,
        } = result
        else {
            continue;
        };

        if !is_error.unwrap_or(false) {
            saw_success = true;
            continue;
        }

        let Some((tool_name, args_hash)) = call_meta.get(tool_use_id.as_str()) else {
            continue;
        };

        let output_str = match output {
            serde_json::Value::String(s) => s.clone(),
            other => other.to_string(),
        };
        let (error_code, error_fingerprint) = extract_error_signature(&output_str);
        let signature = format!(
            "{}|{}|{}|{}",
            tool_name, error_code, error_fingerprint, args_hash
        );
        let count = counters
            .entry(signature)
            .and_modify(|c| *c += 1)
            .or_insert(1);

        if *count >= REPEATED_TOOL_FAILURE_THRESHOLD {
            return Some(format!(
                "Stopping tool loop: '{}' failed {} times with the same '{}' error. A different strategy is required.",
                tool_name, *count, error_code
            ));
        }
    }

    if saw_success {
        counters.clear();
    }

    None
}

fn hash_tool_arguments(arguments: &serde_json::Value) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    arguments.to_string().hash(&mut hasher);
    hasher.finish()
}

fn extract_error_signature(output_str: &str) -> (String, String) {
    if let Ok(value) = serde_json::from_str::<serde_json::Value>(output_str) {
        if let Some(error) = value.get("error") {
            if let Some(error_obj) = error.as_object() {
                let message = error_obj
                    .get("message")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default();
                let code = error_obj
                    .get("code")
                    .and_then(|v| v.as_str())
                    .map(|c| c.to_ascii_lowercase())
                    .filter(|c| !c.is_empty())
                    .unwrap_or_else(|| classify_error_code(message).to_string());
                return (code, normalize_error_fingerprint(message));
            }

            if let Some(message) = error.as_str() {
                return (
                    classify_error_code(message).to_string(),
                    normalize_error_fingerprint(message),
                );
            }
        }
    }

    (
        classify_error_code(output_str).to_string(),
        normalize_error_fingerprint(output_str),
    )
}

fn classify_error_code(message: &str) -> &'static str {
    let lower = message.to_ascii_lowercase();
    if lower.contains("invalid parameters")
        || lower.contains("missing field")
        || lower.contains("unknown field")
    {
        "invalid_parameters"
    } else if lower.contains("unknown tool") {
        "unknown_tool"
    } else if lower.contains("access denied") || lower.contains("outside workspace") {
        "access_denied"
    } else if lower.contains("timed out") || lower.contains("timeout") {
        "timeout"
    } else if lower.contains("denied") {
        "permission_denied"
    } else {
        "tool_error"
    }
}

fn normalize_error_fingerprint(message: &str) -> String {
    let mut compact = String::new();
    for part in message.split_whitespace() {
        if !compact.is_empty() {
            compact.push(' ');
        }
        compact.push_str(part);
    }

    if compact.is_empty() {
        return "unknown".to_string();
    }

    compact.make_ascii_lowercase();
    compact.chars().take(160).collect()
}

#[cfg(test)]
mod tests {
    use super::{
        classify_error_code, detect_repeated_tool_failures, extract_error_signature,
        normalize_error_fingerprint, truncate_output, AiToolCall, Content, MAX_TOOL_OUTPUT_CHARS,
    };
    use std::collections::HashMap;

    #[test]
    fn extract_error_signature_handles_structured_and_legacy_formats() {
        let structured = r#"{"error":{"code":"invalid_parameters","message":"Invalid parameters: missing field `prompt`"}}"#;
        let (code, fingerprint) = extract_error_signature(structured);
        assert_eq!(code, "invalid_parameters");
        assert!(fingerprint.contains("missing field"));

        let legacy = r#"{"error":"Invalid parameters: missing field `pattern`"}"#;
        let (code, fingerprint) = extract_error_signature(legacy);
        assert_eq!(code, "invalid_parameters");
        assert!(fingerprint.contains("missing field"));
    }

    #[test]
    fn normalize_error_fingerprint_collapses_whitespace() {
        let normalized = normalize_error_fingerprint("  A   spaced\n error\tmessage  ");
        assert_eq!(normalized, "a spaced error message");
    }

    #[test]
    fn classify_error_code_matches_invalid_params() {
        assert_eq!(
            classify_error_code("Invalid parameters: missing field `pattern`"),
            "invalid_parameters"
        );
    }

    #[test]
    fn truncate_output_handles_utf8_boundaries() {
        let prefix = "a".repeat(MAX_TOOL_OUTPUT_CHARS - 1);
        let output = format!("{prefix}🙂tail");
        let truncated = truncate_output(&output);
        assert!(truncated.starts_with(&prefix));
        assert!(truncated.contains("OUTPUT TRUNCATED"));
    }

    #[test]
    fn detect_repeated_tool_failures_trips_at_threshold() {
        let call = AiToolCall {
            id: "call_1".to_string(),
            name: "glob".to_string(),
            arguments: serde_json::json!({"pattern":"**/*"}),
        };
        let result = Content::ToolResult {
            tool_use_id: "call_1".to_string(),
            output: serde_json::Value::String(
                r#"{"error":"Invalid parameters: missing field `pattern`"}"#.to_string(),
            ),
            is_error: Some(true),
        };

        let mut counters = HashMap::new();
        let first = detect_repeated_tool_failures(
            &mut counters,
            std::slice::from_ref(&call),
            std::slice::from_ref(&result),
        );
        assert!(first.is_none());

        let second = detect_repeated_tool_failures(
            &mut counters,
            std::slice::from_ref(&call),
            std::slice::from_ref(&result),
        );
        assert!(second.is_some());
    }
}
