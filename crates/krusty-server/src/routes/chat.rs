//! Chat endpoint with SSE streaming and tool loop.

use std::collections::HashMap;
use std::convert::Infallible;
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
    AiToolCall, Content, FinishReason, ModelMessage, Role, ThinkingConfig,
};
use krusty_core::process::ProcessRegistry;
use krusty_core::storage::Database;
use krusty_core::tools::registry::{
    tool_category, PermissionMode, ToolCategory, ToolContext, ToolRegistry,
};
use krusty_core::SessionManager;

use crate::auth::CurrentUser;
use crate::error::AppError;
use crate::push::{PushPayload, PushService};
use crate::types::{AgenticEvent, ChatRequest, ToolApprovalRequest, ToolResultRequest};
use crate::AppState;

const MAX_ITERATIONS: usize = 50;
const SSE_CHANNEL_BUFFER: usize = 256;
const MAX_TOOL_OUTPUT_CHARS: usize = 30_000;
const AI_STREAM_TIMEOUT: Duration = Duration::from_secs(120);
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
const EXPLORATION_BUDGET_SOFT: usize = 15;
const EXPLORATION_BUDGET_HARD: usize = 30;
const APPROVAL_TIMEOUT: Duration = Duration::from_secs(300);
const SESSION_LOCK_MAX_ENTRIES: usize = 1000;
const SESSION_LOCK_MAX_AGE: Duration = Duration::from_secs(3600);

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
    user_id: Option<String>,
    guard: OwnedMutexGuard<()>,
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

    let permission_mode = req.permission_mode;

    let first_message_for_title = if is_first_message {
        Some(req.message.clone())
    } else {
        None
    };

    let user_content = vec![Content::Text {
        text: req.message.clone(),
    }];
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
        run_agentic_loop(
            ai_client,
            tool_registry,
            process_registry,
            sse_tx,
            conversation,
            options,
            ctx_session_id,
            db_path,
            working_dir,
            ctx_user_id,
            permission_mode,
            pending_approvals,
            push_service,
        )
        .await;
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

    let has_thinking = ctx.conversation.iter().any(|msg| {
        msg.content
            .iter()
            .any(|c| matches!(c, Content::Thinking { .. }))
    });
    if has_thinking {
        apply_thinking_config(&ctx.ai_client, &mut ctx.options);
    }

    let tool_result_content = vec![Content::ToolResult {
        tool_use_id: req.tool_call_id.clone(),
        output: serde_json::Value::String(req.result),
        is_error: None,
    }];
    let tool_result_json = serde_json::to_string(&tool_result_content)?;

    ctx.conversation.push(ModelMessage {
        role: Role::User,
        content: tool_result_content,
    });
    ctx.session_manager
        .save_message(&req.session_id, "user", &tool_result_json)?;

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
        user_id,
        guard,
        ..
    } = ctx;

    tokio::spawn(async move {
        let _guard = guard;
        run_agentic_loop(
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
            PermissionMode::Autonomous,
            pending_approvals,
            push_service,
        )
        .await;
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

async fn run_agentic_loop(
    ai_client: Arc<AiClient>,
    tool_registry: Arc<ToolRegistry>,
    process_registry: Arc<ProcessRegistry>,
    sse_tx: mpsc::Sender<Result<Event, Infallible>>,
    mut conversation: Vec<ModelMessage>,
    options: CallOptions,
    session_id: String,
    db_path: Arc<PathBuf>,
    working_dir: PathBuf,
    user_id: Option<String>,
    permission_mode: PermissionMode,
    pending_approvals: Arc<RwLock<HashMap<String, oneshot::Sender<bool>>>>,
    push_service: Option<Arc<PushService>>,
) {
    let db = match Database::new(&db_path) {
        Ok(db) => db,
        Err(e) => {
            send_event(
                &sse_tx,
                AgenticEvent::Error {
                    error: format!("Database error: {}", e),
                },
            )
            .await;
            return;
        }
    };
    let session_manager = SessionManager::new(db);

    let mut client_connected = true;
    let mut last_token_count = 0usize;
    let mut exploration_budget_count = 0usize;
    let _ = session_manager.set_agent_state(&session_id, "streaming");

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
                    let _ = session_manager.update_token_count(&session_id, last_token_count);
                }
                let _ = session_manager.set_agent_state(&session_id, "error");
                if !client_connected {
                    fire_push(
                        &push_service,
                        user_id.as_deref(),
                        PushPayload {
                            title: "Krusty".into(),
                            body: "Session encountered an error".into(),
                            session_id: Some(session_id),
                            tag: None,
                        },
                    );
                }
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
            save_message(&session_manager, &session_id, &assistant_msg);
        }

        if tool_calls.is_empty() {
            if let Some(plan) = try_detect_plan(&text) {
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
                            title: plan.title,
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
                    let _ = session_manager.update_token_count(&session_id, last_token_count);
                }
                let _ = session_manager.set_agent_state(&session_id, "awaiting_input");
                if client_connected {
                    send_event(
                        &sse_tx,
                        AgenticEvent::Finish {
                            session_id: session_id.clone(),
                        },
                    )
                    .await;
                } else {
                    fire_push(
                        &push_service,
                        user_id.as_deref(),
                        PushPayload {
                            title: "Krusty".into(),
                            body: "Krusty needs your input".into(),
                            session_id: Some(session_id),
                            tag: None,
                        },
                    );
                }
                return;
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

        if client_connected {
            for call in tool_calls.iter().filter(|t| t.name == "enter_plan_mode") {
                let reason = call
                    .arguments
                    .get("reason")
                    .and_then(|v| v.as_str())
                    .map(String::from);
                send_event(
                    &sse_tx,
                    AgenticEvent::ModeChange {
                        mode: "plan".to_string(),
                        reason,
                    },
                )
                .await;
            }
        }

        let ask_user_calls: Vec<_> = tool_calls
            .iter()
            .filter(|t| t.name == "AskUserQuestion")
            .collect();

        if !ask_user_calls.is_empty() {
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
                let _ = session_manager.update_token_count(&session_id, last_token_count);
            }
            let _ = session_manager.set_agent_state(&session_id, "awaiting_input");
            if client_connected {
                send_event(
                    &sse_tx,
                    AgenticEvent::Finish {
                        session_id: session_id.clone(),
                    },
                )
                .await;
            } else {
                fire_push(
                    &push_service,
                    user_id.as_deref(),
                    PushPayload {
                        title: "Krusty".into(),
                        body: "Krusty needs your input".into(),
                        session_id: Some(session_id),
                        tag: None,
                    },
                );
            }
            return;
        }

        let all_readonly = tool_calls
            .iter()
            .all(|t| matches!(t.name.as_str(), "read" | "glob" | "grep"));
        let has_action = tool_calls.iter().any(|t| {
            matches!(
                t.name.as_str(),
                "edit" | "write" | "bash" | "build" | "task_start" | "task_complete"
            )
        });
        if has_action {
            exploration_budget_count = 0;
        } else if all_readonly {
            exploration_budget_count += tool_calls.len();
        }

        let _ = session_manager.set_agent_state(&session_id, "tool_executing");
        let tool_results = execute_tools(
            &tool_registry,
            &tool_calls,
            &working_dir,
            &process_registry,
            &sse_tx,
            user_id.as_deref(),
            permission_mode,
            &pending_approvals,
        )
        .await;

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
        save_message(&session_manager, &session_id, &tool_msg);

        let _ = session_manager.set_agent_state(&session_id, "streaming");

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
        let _ = session_manager.update_token_count(&session_id, last_token_count);
    }
    let _ = session_manager.set_agent_state(&session_id, "idle");

    if client_connected && !sse_tx.is_closed() {
        send_event(&sse_tx, AgenticEvent::Finish { session_id }).await;
    } else {
        // Client disconnected mid-run — get session title for the notification
        let title = session_manager
            .get_session(&session_id)
            .ok()
            .flatten()
            .map(|s| s.title)
            .unwrap_or_else(|| "Session".to_string());

        fire_push(
            &push_service,
            user_id.as_deref(),
            PushPayload {
                title: "Krusty".into(),
                body: format!("{title} is complete"),
                session_id: Some(session_id.clone()),
                tag: Some(format!("session-{session_id}")),
            },
        );
    }
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

    loop {
        let part = match tokio::time::timeout(AI_STREAM_TIMEOUT, api_rx.recv()).await {
            Ok(Some(part)) => part,
            Ok(None) => break,
            Err(_) => {
                send_event(
                    sse_tx,
                    AgenticEvent::Error {
                        error: "AI stream timeout: no data received for 120 seconds".to_string(),
                    },
                )
                .await;
                break;
            }
        };

        match &part {
            StreamPart::TextDelta { delta } => {
                text_buffer.push_str(delta);
                send_event(
                    sse_tx,
                    AgenticEvent::TextDelta {
                        delta: delta.clone(),
                    },
                )
                .await;
            }
            StreamPart::ThinkingDelta { thinking, .. } => {
                send_event(
                    sse_tx,
                    AgenticEvent::ThinkingDelta {
                        thinking: thinking.clone(),
                    },
                )
                .await;
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
                send_event(
                    sse_tx,
                    AgenticEvent::ToolCallStart {
                        id: id.clone(),
                        name: name.clone(),
                    },
                )
                .await;
            }
            StreamPart::ToolCallComplete { tool_call } => {
                tool_calls.push(tool_call.clone());
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
            StreamPart::Finish { reason } => finish_reason = reason.clone(),
            StreamPart::Usage { usage } => {
                prompt_tokens = usage.prompt_tokens + usage.completion_tokens;
                send_event(
                    sse_tx,
                    AgenticEvent::Usage {
                        prompt_tokens,
                        completion_tokens: usage.completion_tokens,
                    },
                )
                .await;
            }
            StreamPart::Error { error } => {
                send_event(
                    sse_tx,
                    AgenticEvent::Error {
                        error: error.clone(),
                    },
                )
                .await;
            }
            _ => {}
        }

        if sse_tx.is_closed() {
            break;
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

async fn execute_tools(
    tool_registry: &ToolRegistry,
    tool_calls: &[AiToolCall],
    working_dir: &Path,
    process_registry: &Arc<ProcessRegistry>,
    sse_tx: &mpsc::Sender<Result<Event, Infallible>>,
    user_id: Option<&str>,
    permission_mode: PermissionMode,
    pending_approvals: &Arc<RwLock<HashMap<String, oneshot::Sender<bool>>>>,
) -> Vec<Content> {
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

        if call.name == "enter_plan_mode" {
            let reason = call
                .arguments
                .get("reason")
                .and_then(|v| v.as_str())
                .unwrap_or("Starting planning phase");

            let output = format!(
                "Now in Plan mode. {}\n\nCreate a phase-based checkbox plan before making changes.",
                reason
            );

            send_event(
                sse_tx,
                AgenticEvent::ToolResult {
                    id: call.id.clone(),
                    output: output.clone(),
                    is_error: false,
                },
            )
            .await;

            results.push(Content::ToolResult {
                tool_use_id: call.id.clone(),
                output: serde_json::Value::String(output),
                is_error: None,
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
            plan_mode: false,
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

    results
}

fn build_assistant_message(
    text: &str,
    thinking_blocks: &[ThinkingBlock],
    tool_calls: &[AiToolCall],
) -> ModelMessage {
    let mut content = Vec::new();

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

fn save_message(session_manager: &SessionManager, session_id: &str, message: &ModelMessage) {
    let role = match message.role {
        Role::User => "user",
        Role::Assistant => "assistant",
        _ => return,
    };

    match serde_json::to_string(&message.content) {
        Ok(json) => {
            if let Err(e) = session_manager.save_message(session_id, role, &json) {
                tracing::error!("Failed to save message: {}", e);
            }
        }
        Err(e) => tracing::error!("Failed to serialize message: {}", e),
    }
}

fn truncate_output(output: &str) -> String {
    if output.len() <= MAX_TOOL_OUTPUT_CHARS {
        return output.to_string();
    }

    let truncated = &output[..MAX_TOOL_OUTPUT_CHARS];
    let break_point = truncated.rfind('\n').unwrap_or(MAX_TOOL_OUTPUT_CHARS);
    let clean = &output[..break_point];
    format!(
        "{}\n\n[... OUTPUT TRUNCATED: {} chars -> {} chars ...]",
        clean,
        output.len(),
        clean.len()
    )
}

/// Fire a push notification in a background task (non-blocking, failure only logged).
fn fire_push(push_service: &Option<Arc<PushService>>, user_id: Option<&str>, payload: PushPayload) {
    if let Some(svc) = push_service.clone() {
        let uid = user_id.map(String::from);
        tokio::spawn(async move {
            svc.notify_user(uid.as_deref(), payload).await;
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
}

fn try_detect_plan(text: &str) -> Option<ParsedPlan> {
    let plan_patterns = [
        r"(?m)^#{1,3}\s*Plan:\s*(.+)$",
        r"(?m)^#{1,3}\s*([\w\s]+Plan[\w\s]*)$",
        r"(?m)^#{1,3}\s*Plan\s*$",
    ];

    let mut title = String::new();
    for pattern in &plan_patterns {
        if let Ok(regex) = regex::Regex::new(pattern) {
            if let Some(captures) = regex.captures(text) {
                title = captures
                    .get(1)
                    .map(|m| m.as_str().trim().to_string())
                    .unwrap_or_else(|| "Implementation Plan".to_string());
                break;
            }
        }
    }

    if title.is_empty() {
        let phase_regex = regex::Regex::new(r"(?m)^#{2,3}\s*Phase\s+\d").ok()?;
        if phase_regex.is_match(text) {
            title = "Implementation Plan".to_string();
        }
    }
    if title.is_empty() {
        return None;
    }

    let task_regex = regex::Regex::new(r"(?m)^[\s]*-\s*\[([ xX])\]\s*(.+)$").ok()?;
    let mut tasks = Vec::new();
    for cap in task_regex.captures_iter(text) {
        let completed = cap
            .get(1)
            .map(|m| m.as_str().eq_ignore_ascii_case("x"))
            .unwrap_or(false);
        let content = cap
            .get(2)
            .map(|m| m.as_str().trim().to_string())
            .unwrap_or_default();
        if !content.is_empty() {
            tasks.push(crate::types::PlanItem { content, completed });
        }
    }

    if tasks.is_empty() {
        None
    } else {
        Some(ParsedPlan { title, tasks })
    }
}
