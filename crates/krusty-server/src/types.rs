//! Request and response types for the API

use krusty_core::storage::SessionInfo;
use serde::{Deserialize, Serialize};

// ============================================================================
// Session Types
// ============================================================================

#[derive(Deserialize)]
pub struct CreateSessionRequest {
    pub title: Option<String>,
    pub model: Option<String>,
    pub working_dir: Option<String>,
}

#[derive(Deserialize)]
pub struct UpdateSessionRequest {
    pub title: String,
}

#[derive(Deserialize)]
pub struct PinchRequest {
    /// Optional hints about what to preserve
    pub preservation_hints: Option<String>,
    /// Optional direction for the new session
    pub direction: Option<String>,
}

#[derive(Serialize)]
pub struct PinchResponse {
    /// The new child session
    pub session: SessionResponse,
    /// Summary of what was preserved
    pub summary: String,
    /// Key decisions preserved
    pub key_decisions: Vec<String>,
    /// Pending tasks carried forward
    pub pending_tasks: Vec<String>,
}

#[derive(Serialize)]
pub struct SessionResponse {
    pub id: String,
    pub title: String,
    pub updated_at: String,
    pub token_count: Option<usize>,
    pub parent_session_id: Option<String>,
    pub working_dir: Option<String>,
}

impl From<SessionInfo> for SessionResponse {
    fn from(s: SessionInfo) -> Self {
        Self {
            id: s.id,
            title: s.title,
            updated_at: s.updated_at.to_rfc3339(),
            token_count: s.token_count,
            parent_session_id: s.parent_session_id,
            working_dir: s.working_dir,
        }
    }
}

#[derive(Serialize)]
pub struct SessionWithMessagesResponse {
    pub session: SessionResponse,
    pub messages: Vec<MessageResponse>,
}

/// Agent execution state for a session
#[derive(Serialize)]
pub struct SessionStateResponse {
    /// Session ID
    pub id: String,
    /// Agent state: "idle", "streaming", "tool_executing", "awaiting_input", "error"
    pub agent_state: String,
    /// When the agent started (if not idle)
    pub started_at: Option<String>,
    /// Last event timestamp (for activity tracking)
    pub last_event_at: Option<String>,
}

#[derive(Serialize)]
pub struct MessageResponse {
    pub role: String,
    pub content: serde_json::Value,
}

// ============================================================================
// Chat Types
// ============================================================================

#[derive(Deserialize)]
pub struct ChatRequest {
    /// Session ID (creates new session if not provided)
    pub session_id: Option<String>,
    /// User message content
    pub message: String,
    /// Model override
    pub model: Option<String>,
    /// Enable extended thinking
    #[serde(default)]
    pub thinking_enabled: bool,
}

#[derive(Deserialize)]
pub struct ToolResultRequest {
    /// Session ID
    pub session_id: String,
    /// Tool use ID to respond to
    pub tool_call_id: String,
    /// Tool result content (JSON string)
    pub result: String,
}

// ============================================================================
// Model Types
// ============================================================================

#[derive(Serialize)]
pub struct ModelResponse {
    pub id: String,
    pub display_name: String,
    pub provider: String,
    pub context_window: usize,
    pub max_output: usize,
    pub supports_thinking: bool,
    pub supports_tools: bool,
}

#[derive(Serialize)]
pub struct ModelsListResponse {
    pub models: Vec<ModelResponse>,
    pub default_model: String,
}

// ============================================================================
// Tool Types
// ============================================================================

#[derive(Deserialize)]
pub struct ToolExecuteRequest {
    pub tool_name: String,
    pub params: serde_json::Value,
    /// Optional working directory override
    pub working_dir: Option<String>,
}

#[derive(Serialize)]
pub struct ToolExecuteResponse {
    pub output: String,
    pub is_error: bool,
}

// ============================================================================
// File Types
// ============================================================================

#[derive(Deserialize)]
pub struct FileQuery {
    pub path: String,
}

#[derive(Deserialize)]
pub struct TreeQuery {
    pub root: Option<String>,
    #[serde(default = "default_depth", deserialize_with = "clamp_depth")]
    pub depth: usize,
}

fn default_depth() -> usize {
    3
}

/// Maximum tree depth to prevent DoS via deep recursion
const MAX_TREE_DEPTH: usize = 10;

fn clamp_depth<'de, D>(deserializer: D) -> Result<usize, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value: usize = serde::Deserialize::deserialize(deserializer)?;
    Ok(value.min(MAX_TREE_DEPTH))
}

#[derive(Serialize)]
pub struct FileResponse {
    pub path: String,
    pub content: String,
    pub size: u64,
}

#[derive(Deserialize)]
pub struct FileWriteRequest {
    pub content: String,
}

#[derive(Serialize)]
pub struct FileWriteResponse {
    pub path: String,
    pub bytes_written: usize,
}

#[derive(Serialize)]
pub struct TreeEntry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<TreeEntry>>,
}

#[derive(Serialize)]
pub struct TreeResponse {
    pub root: String,
    pub entries: Vec<TreeEntry>,
}

#[derive(Deserialize)]
pub struct BrowseQuery {
    /// Directory to list (defaults to home directory)
    pub path: Option<String>,
}

#[derive(Serialize)]
pub struct BrowseEntry {
    pub name: String,
    pub path: String,
}

#[derive(Serialize)]
pub struct BrowseResponse {
    pub current: String,
    pub parent: Option<String>,
    pub directories: Vec<BrowseEntry>,
}

// ============================================================================
// Plan Types
// ============================================================================

#[derive(Debug, Clone, Serialize)]
pub struct PlanItem {
    pub content: String,
    pub completed: bool,
}

// ============================================================================
// Agentic SSE Events
// ============================================================================

/// Events sent to the client during agentic chat loop
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AgenticEvent {
    /// Text content delta from AI
    TextDelta { delta: String },
    /// Extended thinking delta
    ThinkingDelta { thinking: String },
    /// AI is starting a tool call
    ToolCallStart { id: String, name: String },
    /// Tool call complete with arguments
    ToolCallComplete {
        id: String,
        name: String,
        arguments: serde_json::Value,
    },
    /// Server is executing a tool
    ToolExecuting { id: String, name: String },
    /// Tool execution result
    ToolResult {
        id: String,
        output: String,
        is_error: bool,
    },
    /// Waiting for user input (AskUserQuestion)
    AwaitingInput {
        tool_call_id: String,
        tool_name: String,
    },
    /// Mode change (enter_plan_mode tool)
    ModeChange {
        mode: String,
        reason: Option<String>,
    },
    /// Plan tasks update - sent when plan is detected
    PlanUpdate { items: Vec<PlanItem> },
    /// Plan detected in AI response - awaiting confirmation
    PlanComplete {
        tool_call_id: String,
        title: String,
        task_count: usize,
    },
    /// An agentic turn completed
    TurnComplete { turn: usize, has_more: bool },
    /// Token usage information
    Usage {
        prompt_tokens: usize,
        completion_tokens: usize,
    },
    /// Agentic loop finished
    Finish { session_id: String },
    /// Session title updated (from Haiku)
    TitleUpdate { title: String },
    /// Error occurred
    Error { error: String },
}
