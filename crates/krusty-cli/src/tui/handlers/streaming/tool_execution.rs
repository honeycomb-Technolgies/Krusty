//! Tool execution and result handling
//!
//! Handles the execution of AI tool calls and processing of results.

use std::{
    borrow::Cow,
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    time::Duration,
};

use tokio::sync::{mpsc, oneshot};

use crate::agent::subagent::AgentProgress;
use crate::ai::types::{AiToolCall, Content};
use crate::tools::{ToolContext, ToolOutputChunk};
use crate::tui::app::App;
use crate::tui::components::{PromptOption, PromptQuestion};

const MAX_ITERATIONS: usize = 50;
const MAX_TOOL_OUTPUT_CHARS: usize = 30_000;
const APPROVAL_TIMEOUT: Duration = Duration::from_secs(300);
const REPEATED_TOOL_FAILURE_THRESHOLD: usize = 2;

impl App {
    /// Handle mode-switch tools (`set_work_mode` and legacy `enter_plan_mode`)
    pub(super) fn handle_mode_switch_tools(&mut self, tool_calls: Vec<AiToolCall>) {
        use crate::tui::app::WorkMode;

        let mut results = Vec::new();

        for tool_call in tool_calls {
            tracing::info!(
                "Handling mode switch tool call: {} ({})",
                tool_call.id,
                tool_call.name
            );

            let target_mode = if tool_call.name == "enter_plan_mode" {
                WorkMode::Plan
            } else {
                match tool_call.arguments.get("mode").and_then(|v| v.as_str()) {
                    Some("plan") => WorkMode::Plan,
                    Some("build") => WorkMode::Build,
                    Some(other) => {
                        results.push(Content::ToolResult {
                            tool_use_id: tool_call.id.clone(),
                            output: serde_json::Value::String(format!(
                                "Error: invalid mode '{}'. Use 'build' or 'plan'.",
                                other
                            )),
                            is_error: Some(true),
                        });
                        continue;
                    }
                    None => {
                        results.push(Content::ToolResult {
                            tool_use_id: tool_call.id.clone(),
                            output: serde_json::Value::String(
                                "Error: mode parameter is required (build|plan)".to_string(),
                            ),
                            is_error: Some(true),
                        });
                        continue;
                    }
                }
            };

            let default_reason = if target_mode == WorkMode::Plan {
                "Starting planning phase"
            } else {
                "Starting implementation phase"
            };
            let reason = tool_call
                .arguments
                .get("reason")
                .and_then(|v| v.as_str())
                .unwrap_or(default_reason)
                .to_string();

            let clear_existing = tool_call
                .arguments
                .get("clear_existing")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            if clear_existing && target_mode == WorkMode::Plan {
                self.clear_plan();
                tracing::info!("Cleared existing plan");
            }
            self.ui.work_mode = target_mode;
            tracing::info!("Switched to {:?} mode: {}", target_mode, reason);

            let output = if target_mode == WorkMode::Plan {
                format!(
                    "Now in Plan mode. {}. Create a plan using the standard format (# Plan: Title, ## Phase N: Name, - [ ] Task). The user will review and approve before implementation.",
                    reason
                )
            } else {
                format!(
                    "Now in Build mode. {}. Proceed with implementation and keep plan task status updated.",
                    reason
                )
            };
            results.push(Content::ToolResult {
                tool_use_id: tool_call.id.clone(),
                output: serde_json::Value::String(output),
                is_error: None,
            });
        }

        if !results.is_empty() {
            self.runtime.pending_tool_results.extend(results);
        }
    }

    /// Handle task_complete tool calls to update plan immediately
    /// ENFORCES: Task must be InProgress (started) before it can be completed
    /// ENFORCES: Only ONE task per call (no batch completion)
    /// ENFORCES: Result parameter required
    pub(super) fn handle_task_complete_tools(&mut self, tool_calls: Vec<AiToolCall>) {
        use crate::plan::TaskStatus;
        let mut results = Vec::new();

        for tool_call in tool_calls {
            tracing::info!("Handling task_complete tool call: {}", tool_call.id);

            // Extract required result parameter
            let result_text = tool_call
                .arguments
                .get("result")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            if result_text.is_empty() {
                results.push(Content::ToolResult {
                    tool_use_id: tool_call.id.clone(),
                    output: serde_json::Value::String(
                        "Error: 'result' parameter is required. Describe what you accomplished for this specific task.".to_string(),
                    ),
                    is_error: Some(true),
                });
                continue;
            }

            // HARD CONSTRAINT: Only single task_id allowed (no batch)
            let task_id = tool_call.arguments.get("task_id").and_then(|v| v.as_str());
            let task_ids = tool_call
                .arguments
                .get("task_ids")
                .and_then(|v| v.as_array());

            if task_ids.is_some() {
                results.push(Content::ToolResult {
                    tool_use_id: tool_call.id.clone(),
                    output: serde_json::Value::String(
                        "Error: Batch completion (task_ids) is not allowed. Complete ONE task at a time with task_id. This ensures focused, quality work.".to_string(),
                    ),
                    is_error: Some(true),
                });
                continue;
            }

            let Some(task_id) = task_id.filter(|s| !s.is_empty()) else {
                results.push(Content::ToolResult {
                    tool_use_id: tool_call.id.clone(),
                    output: serde_json::Value::String(
                        "Error: task_id required. Specify which task you're completing."
                            .to_string(),
                    ),
                    is_error: Some(true),
                });
                continue;
            };

            let Some(plan) = &mut self.runtime.active_plan else {
                results.push(Content::ToolResult {
                    tool_use_id: tool_call.id.clone(),
                    output: serde_json::Value::String(
                        "Error: No active plan. Create a plan first.".to_string(),
                    ),
                    is_error: Some(true),
                });
                continue;
            };

            // HARD CONSTRAINT: Task must be InProgress to complete
            let task_status = plan.find_task(task_id).map(|t| t.status);
            match task_status {
                None => {
                    results.push(Content::ToolResult {
                        tool_use_id: tool_call.id.clone(),
                        output: serde_json::Value::String(format!(
                            "Error: Task '{}' not found in plan.",
                            task_id
                        )),
                        is_error: Some(true),
                    });
                    continue;
                }
                Some(TaskStatus::Completed) => {
                    results.push(Content::ToolResult {
                        tool_use_id: tool_call.id.clone(),
                        output: serde_json::Value::String(format!(
                            "Error: Task '{}' is already completed.",
                            task_id
                        )),
                        is_error: Some(true),
                    });
                    continue;
                }
                Some(TaskStatus::Blocked) => {
                    results.push(Content::ToolResult {
                        tool_use_id: tool_call.id.clone(),
                        output: serde_json::Value::String(format!(
                            "Error: Task '{}' is blocked. Complete its dependencies first, then use task_start.",
                            task_id
                        )),
                        is_error: Some(true),
                    });
                    continue;
                }
                Some(TaskStatus::Pending) => {
                    results.push(Content::ToolResult {
                        tool_use_id: tool_call.id.clone(),
                        output: serde_json::Value::String(format!(
                            "Error: Task '{}' was not started. Use task_start(\"{}\") first, do the work, then complete it.",
                            task_id, task_id
                        )),
                        is_error: Some(true),
                    });
                    continue;
                }
                Some(TaskStatus::InProgress) => {
                    // Good - task is in progress, can be completed
                }
            }

            // Complete the task
            if let Err(e) = plan.complete_task(task_id, &result_text) {
                results.push(Content::ToolResult {
                    tool_use_id: tool_call.id.clone(),
                    output: serde_json::Value::String(format!("Error: {}", e)),
                    is_error: Some(true),
                });
                continue;
            }

            if let Some(ref pm) = self.services.plan_manager {
                if let Err(e) = pm.save_plan(plan) {
                    tracing::error!("Failed to save plan after task completion: {}", e);
                }
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

            tracing::info!("{}", msg);

            results.push(Content::ToolResult {
                tool_use_id: tool_call.id.clone(),
                output: serde_json::Value::String(msg),
                is_error: None,
            });
        }

        if !results.is_empty() {
            self.runtime.pending_tool_results.extend(results);
        }
    }

    /// Handle task_start tool calls to mark tasks as in-progress
    pub(super) fn handle_task_start_tools(&mut self, tool_calls: Vec<AiToolCall>) {
        let mut results = Vec::new();

        for tool_call in tool_calls {
            tracing::info!("Handling task_start tool call: {}", tool_call.id);

            let Some(task_id) = tool_call.arguments.get("task_id").and_then(|v| v.as_str()) else {
                results.push(Content::ToolResult {
                    tool_use_id: tool_call.id.clone(),
                    output: serde_json::Value::String("Error: task_id required".to_string()),
                    is_error: Some(true),
                });
                continue;
            };

            let Some(plan) = &mut self.runtime.active_plan else {
                results.push(Content::ToolResult {
                    tool_use_id: tool_call.id.clone(),
                    output: serde_json::Value::String(
                        "Error: No active plan. Create a plan first.".to_string(),
                    ),
                    is_error: Some(true),
                });
                continue;
            };

            match plan.start_task(task_id) {
                Ok(()) => {
                    if let Some(ref pm) = self.services.plan_manager {
                        if let Err(e) = pm.save_plan(plan) {
                            tracing::error!("Failed to save plan after task start: {}", e);
                        }
                    }
                    results.push(Content::ToolResult {
                        tool_use_id: tool_call.id.clone(),
                        output: serde_json::Value::String(format!(
                            "Started task {}. Status: in_progress",
                            task_id
                        )),
                        is_error: None,
                    });
                }
                Err(e) => {
                    results.push(Content::ToolResult {
                        tool_use_id: tool_call.id.clone(),
                        output: serde_json::Value::String(format!("Error: {}", e)),
                        is_error: Some(true),
                    });
                }
            }
        }

        if !results.is_empty() {
            self.runtime.pending_tool_results.extend(results);
        }
    }

    /// Handle add_subtask tool calls to create subtasks
    pub(super) fn handle_add_subtask_tools(&mut self, tool_calls: Vec<AiToolCall>) {
        let mut results = Vec::new();

        for tool_call in tool_calls {
            tracing::info!("Handling add_subtask tool call: {}", tool_call.id);

            let parent_id = tool_call
                .arguments
                .get("parent_id")
                .and_then(|v| v.as_str());
            let description = tool_call
                .arguments
                .get("description")
                .and_then(|v| v.as_str());
            let context = tool_call.arguments.get("context").and_then(|v| v.as_str());

            let (Some(parent_id), Some(description)) = (parent_id, description) else {
                results.push(Content::ToolResult {
                    tool_use_id: tool_call.id.clone(),
                    output: serde_json::Value::String(
                        "Error: parent_id and description required".to_string(),
                    ),
                    is_error: Some(true),
                });
                continue;
            };

            let Some(plan) = &mut self.runtime.active_plan else {
                results.push(Content::ToolResult {
                    tool_use_id: tool_call.id.clone(),
                    output: serde_json::Value::String(
                        "Error: No active plan. Create a plan first.".to_string(),
                    ),
                    is_error: Some(true),
                });
                continue;
            };

            match plan.add_subtask(parent_id, description, context) {
                Ok(subtask_id) => {
                    if let Some(ref pm) = self.services.plan_manager {
                        if let Err(e) = pm.save_plan(plan) {
                            tracing::error!("Failed to save plan after adding subtask: {}", e);
                        }
                    }
                    results.push(Content::ToolResult {
                        tool_use_id: tool_call.id.clone(),
                        output: serde_json::Value::String(format!(
                            "Created subtask {} under {}",
                            subtask_id, parent_id
                        )),
                        is_error: None,
                    });
                }
                Err(e) => {
                    results.push(Content::ToolResult {
                        tool_use_id: tool_call.id.clone(),
                        output: serde_json::Value::String(format!("Error: {}", e)),
                        is_error: Some(true),
                    });
                }
            }
        }

        if !results.is_empty() {
            self.runtime.pending_tool_results.extend(results);
        }
    }

    /// Handle set_dependency tool calls to create task dependencies
    pub(super) fn handle_set_dependency_tools(&mut self, tool_calls: Vec<AiToolCall>) {
        let mut results = Vec::new();

        for tool_call in tool_calls {
            tracing::info!("Handling set_dependency tool call: {}", tool_call.id);

            let task_id = tool_call.arguments.get("task_id").and_then(|v| v.as_str());
            let blocked_by = tool_call
                .arguments
                .get("blocked_by")
                .and_then(|v| v.as_str());

            let (Some(task_id), Some(blocked_by)) = (task_id, blocked_by) else {
                results.push(Content::ToolResult {
                    tool_use_id: tool_call.id.clone(),
                    output: serde_json::Value::String(
                        "Error: task_id and blocked_by required".to_string(),
                    ),
                    is_error: Some(true),
                });
                continue;
            };

            let Some(plan) = &mut self.runtime.active_plan else {
                results.push(Content::ToolResult {
                    tool_use_id: tool_call.id.clone(),
                    output: serde_json::Value::String(
                        "Error: No active plan. Create a plan first.".to_string(),
                    ),
                    is_error: Some(true),
                });
                continue;
            };

            match plan.add_dependency(task_id, blocked_by) {
                Ok(()) => {
                    if let Some(ref pm) = self.services.plan_manager {
                        if let Err(e) = pm.save_plan(plan) {
                            tracing::error!("Failed to save plan after adding dependency: {}", e);
                        }
                    }
                    results.push(Content::ToolResult {
                        tool_use_id: tool_call.id.clone(),
                        output: serde_json::Value::String(format!(
                            "Task {} is now blocked by {}",
                            task_id, blocked_by
                        )),
                        is_error: None,
                    });
                }
                Err(e) => {
                    results.push(Content::ToolResult {
                        tool_use_id: tool_call.id.clone(),
                        output: serde_json::Value::String(format!("Error: {}", e)),
                        is_error: Some(true),
                    });
                }
            }
        }

        if !results.is_empty() {
            self.runtime.pending_tool_results.extend(results);
        }
    }

    /// Handle AskUserQuestion tool calls via UI instead of registry
    pub(super) fn handle_ask_user_question_tools(&mut self, tool_calls: Vec<AiToolCall>) {
        let Some(tool_call) = tool_calls.into_iter().next() else {
            return;
        };

        tracing::info!("Handling AskUserQuestion tool call: {}", tool_call.id);

        let questions_arg = tool_call.arguments.get("questions");
        let Some(questions_array) = questions_arg.and_then(|v| v.as_array()) else {
            tracing::warn!("AskUserQuestion missing questions array");
            return;
        };

        let mut prompt_questions: Vec<PromptQuestion> = Vec::new();

        for q in questions_array {
            let question = q.get("question").and_then(|v| v.as_str()).unwrap_or("");
            let header = q.get("header").and_then(|v| v.as_str()).unwrap_or("Q");
            let multi_select = q
                .get("multiSelect")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            let mut options: Vec<PromptOption> = Vec::new();
            if let Some(opts) = q.get("options").and_then(|v| v.as_array()) {
                for opt in opts {
                    let label = opt.get("label").and_then(|v| v.as_str()).unwrap_or("");
                    let description = opt.get("description").and_then(|v| v.as_str());
                    options.push(PromptOption {
                        label: label.to_string(),
                        description: description.map(|s| s.to_string()),
                    });
                }
            }

            // Add "Other" option for custom input
            options.push(PromptOption {
                label: "Other".to_string(),
                description: Some("Enter custom response".to_string()),
            });

            prompt_questions.push(PromptQuestion {
                question: question.to_string(),
                header: header.to_string(),
                options,
                multi_select,
            });
        }

        if prompt_questions.is_empty() {
            tracing::warn!("AskUserQuestion has no valid questions");
            return;
        }

        // Remove the "Preparing questions..." message
        if let Some((tag, _)) = self.runtime.chat.messages.last() {
            if tag == "tool" {
                self.runtime.chat.messages.pop();
            }
        }

        self.ui
            .decision_prompt
            .show_ask_user(prompt_questions, tool_call.id);
    }

    /// Spawn tool execution as a background task for non-blocking streaming
    pub fn spawn_tool_execution(&mut self, tool_calls: Vec<AiToolCall>) {
        let tool_names: Vec<_> = tool_calls.iter().map(|t| t.name.as_str()).collect();
        tracing::info!(
            "spawn_tool_execution: {} tools to execute: {:?}",
            tool_calls.len(),
            tool_names
        );

        // Track exploration budget: count consecutive read-only tool calls
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
            self.runtime.exploration_budget_count = 0;
        } else if all_readonly {
            self.runtime.exploration_budget_count += tool_calls.len();
        }

        if tool_calls.is_empty() {
            return;
        }

        // Intercept AskUserQuestion tool
        let (ask_user_tools, tool_calls): (Vec<_>, Vec<_>) = tool_calls
            .into_iter()
            .partition(|t| t.name == "AskUserQuestion");

        let has_ask_user = !ask_user_tools.is_empty();
        if has_ask_user {
            self.handle_ask_user_question_tools(ask_user_tools);
        }

        // Intercept task_complete tool
        let (task_complete_tools, tool_calls): (Vec<_>, Vec<_>) = tool_calls
            .into_iter()
            .partition(|t| t.name == "task_complete");

        let has_task_complete = !task_complete_tools.is_empty();
        if has_task_complete {
            self.handle_task_complete_tools(task_complete_tools);
        }

        // Intercept task_start tool
        let (task_start_tools, tool_calls): (Vec<_>, Vec<_>) =
            tool_calls.into_iter().partition(|t| t.name == "task_start");

        let has_task_start = !task_start_tools.is_empty();
        if has_task_start {
            self.handle_task_start_tools(task_start_tools);
        }

        // Intercept add_subtask tool
        let (add_subtask_tools, tool_calls): (Vec<_>, Vec<_>) = tool_calls
            .into_iter()
            .partition(|t| t.name == "add_subtask");

        let has_add_subtask = !add_subtask_tools.is_empty();
        if has_add_subtask {
            self.handle_add_subtask_tools(add_subtask_tools);
        }

        // Intercept set_dependency tool
        let (set_dependency_tools, tool_calls): (Vec<_>, Vec<_>) = tool_calls
            .into_iter()
            .partition(|t| t.name == "set_dependency");

        let has_set_dependency = !set_dependency_tools.is_empty();
        if has_set_dependency {
            self.handle_set_dependency_tools(set_dependency_tools);
        }

        // Intercept set_work_mode and legacy enter_plan_mode tools
        let (plan_mode_tools, tool_calls): (Vec<_>, Vec<_>) = tool_calls
            .into_iter()
            .partition(|t| t.name == "set_work_mode" || t.name == "enter_plan_mode");

        let has_plan_mode = !plan_mode_tools.is_empty();
        if has_plan_mode {
            self.handle_mode_switch_tools(plan_mode_tools);
        }

        let has_plan_tools = has_task_complete
            || has_task_start
            || has_add_subtask
            || has_set_dependency
            || has_plan_mode;

        // Supervised mode: intercept Write-category tools for approval
        if self.runtime.permission_mode == krusty_core::tools::registry::PermissionMode::Supervised
        {
            let write_tools: Vec<_> = tool_calls
                .iter()
                .filter(|t| {
                    krusty_core::tools::registry::tool_category(&t.name)
                        == krusty_core::tools::registry::ToolCategory::Write
                })
                .collect();

            if !write_tools.is_empty() {
                let names: Vec<String> = write_tools.iter().map(|t| t.name.clone()).collect();
                let ids: Vec<String> = write_tools.iter().map(|t| t.id.clone()).collect();

                // Store the full tool_calls for later execution after approval
                self.runtime.queued_tools.extend(tool_calls);

                // Show approval prompt and track when it was requested
                self.ui.decision_prompt.show_tool_approval(names, ids);
                self.runtime.approval_requested_at = Some(std::time::Instant::now());
                return;
            }
        }

        if tool_calls.is_empty() {
            if has_ask_user {
                self.stop_streaming();
                return;
            }

            if has_plan_tools {
                let results = std::mem::take(&mut self.runtime.pending_tool_results);
                if !results.is_empty() {
                    self.stop_streaming();
                    self.handle_tool_results(results);
                }
                return;
            }
            return;
        }

        // Check if there's an explore/Task tool in the batch
        let has_explore = tool_calls
            .iter()
            .any(|t| t.name == "explore" || t.name == "Task");
        let has_build = tool_calls.iter().any(|t| t.name == "build");

        // If explore tool is present, queue non-explore tools for later
        let tools_to_execute = if has_explore {
            let (explore_tools, other_tools): (Vec<_>, Vec<_>) = tool_calls
                .into_iter()
                .partition(|t| t.name == "explore" || t.name == "Task");

            if !other_tools.is_empty() {
                tracing::info!(
                    "spawn_tool_execution: queuing {} tools until explore completes",
                    other_tools.len()
                );
                self.runtime.queued_tools.extend(other_tools);
            }

            explore_tools
        } else {
            tool_calls
        };

        if tools_to_execute.is_empty() {
            return;
        }

        // Create streaming output channel for bash
        // Unbounded sender for krusty-core API compat, bounded receiver for backpressure
        let (output_tx, unbounded_output_rx) = mpsc::unbounded_channel::<ToolOutputChunk>();
        let (bounded_output_tx, bounded_output_rx) = mpsc::channel::<ToolOutputChunk>(1024);
        self.runtime.channels.bash_output = Some(bounded_output_rx);
        tokio::spawn(async move {
            let mut rx = unbounded_output_rx;
            while let Some(chunk) = rx.recv().await {
                if bounded_output_tx.send(chunk).await.is_err() {
                    break;
                }
            }
        });

        // Create explore progress channel if any explore tools
        let explore_progress_tx = if has_explore {
            let (tx, unbounded_rx) = mpsc::unbounded_channel::<AgentProgress>();
            let (bounded_tx, bounded_rx) = mpsc::channel::<AgentProgress>(1024);
            self.runtime.channels.explore_progress = Some(bounded_rx);
            tokio::spawn(async move {
                let mut rx = unbounded_rx;
                while let Some(progress) = rx.recv().await {
                    if bounded_tx.send(progress).await.is_err() {
                        break;
                    }
                }
            });
            Some(tx)
        } else {
            None
        };

        // Create build progress channel if any build tools
        let build_progress_tx = if has_build {
            let (tx, unbounded_rx) = mpsc::unbounded_channel::<AgentProgress>();
            let (bounded_tx, bounded_rx) = mpsc::channel::<AgentProgress>(1024);
            self.runtime.channels.build_progress = Some(bounded_rx);
            tokio::spawn(async move {
                let mut rx = unbounded_rx;
                while let Some(progress) = rx.recv().await {
                    if bounded_tx.send(progress).await.is_err() {
                        break;
                    }
                }
            });
            Some(tx)
        } else {
            None
        };

        // Create result channel
        let (result_tx, result_rx) = oneshot::channel();
        self.runtime.channels.tool_results = Some(result_rx);

        self.start_tool_execution();

        // Create blocks for visual feedback
        self.create_tool_blocks(&tools_to_execute);

        // Clone what we need for the spawned task
        let tool_registry = self.services.tool_registry.clone();
        let process_registry = self.runtime.process_registry.clone();
        let skills_manager = self.services.skills_manager.clone();
        let cancel_token = self.runtime.cancellation.child_token();
        let plan_mode = self.ui.work_mode == crate::tui::app::WorkMode::Plan;
        let current_model = self.runtime.current_model.clone();

        // Spawn tool execution in background.
        // JoinHandle is dropped - results communicated via channel.
        // If task panics, caller will hang waiting for channel (rare edge case).
        let _handle = tokio::spawn(async move {
            let mut tool_results: Vec<Content> = Vec::new();

            for tool_call in tools_to_execute {
                if cancel_token.is_cancelled() {
                    tracing::info!("Tool execution cancelled before running {}", tool_call.name);
                    break;
                }

                let tool_name = tool_call.name.clone();

                let working_dir =
                    std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));

                let mut ctx =
                    ToolContext::with_process_registry(working_dir, process_registry.clone())
                        .with_skills_manager(skills_manager.clone())
                        .with_current_model(current_model.clone());
                ctx.plan_mode = plan_mode;

                if tool_name == "bash" {
                    ctx = ctx.with_output_stream(output_tx.clone(), tool_call.id.clone());
                }

                if tool_name == "explore" || tool_name == "Task" {
                    ctx.timeout = Some(std::time::Duration::from_secs(600));
                    if let Some(ref tx) = explore_progress_tx {
                        ctx = ctx.with_explore_progress(tx.clone());
                    }
                }

                if tool_name == "build" {
                    ctx.timeout = Some(std::time::Duration::from_secs(900));
                    if let Some(ref tx) = build_progress_tx {
                        ctx = ctx.with_build_progress(tx.clone());
                    }
                }

                let result = tokio::select! {
                    _ = cancel_token.cancelled() => {
                        tracing::info!("Tool execution cancelled during {}", tool_name);
                        Some(crate::tools::registry::ToolResult {
                            output: "Cancelled by user".to_string(),
                            is_error: true,
                        })
                    }
                    result = tool_registry.execute(&tool_call.name, tool_call.arguments.clone(), &ctx) => {
                        result
                    }
                };

                if let Some(result) = result {
                    let output = truncate_tool_output(&result.output);
                    tool_results.push(Content::ToolResult {
                        tool_use_id: tool_call.id.clone(),
                        output: serde_json::Value::String(output),
                        is_error: if result.is_error { Some(true) } else { None },
                    });
                } else {
                    tool_results.push(Content::ToolResult {
                        tool_use_id: tool_call.id.clone(),
                        output: serde_json::Value::String(format!(
                            "Error: Unknown tool '{}'",
                            tool_name
                        )),
                        is_error: Some(true),
                    });
                }

                if cancel_token.is_cancelled() {
                    break;
                }
            }

            let _ = result_tx.send(tool_results);
        });
    }

    /// Create visual blocks for tool calls
    fn create_tool_blocks(&mut self, tools: &[AiToolCall]) {
        for tool_call in tools {
            let tool_name = &tool_call.name;

            if tool_name == "bash" {
                let command = tool_call
                    .arguments
                    .get("command")
                    .and_then(|v| v.as_str())
                    .unwrap_or("bash")
                    .to_string();
                self.runtime
                    .blocks
                    .bash
                    .push(crate::tui::blocks::BashBlock::with_tool_id(
                        command,
                        tool_call.id.clone(),
                    ));
                self.runtime
                    .chat
                    .messages
                    .push(("bash".to_string(), tool_call.id.clone()));
            }

            if tool_name == "grep" || tool_name == "glob" {
                let pattern = tool_call
                    .arguments
                    .get("pattern")
                    .and_then(|v| v.as_str())
                    .unwrap_or("*")
                    .to_string();
                self.runtime
                    .blocks
                    .tool_result
                    .push(crate::tui::blocks::ToolResultBlock::new(
                        tool_call.id.clone(),
                        tool_name.clone(),
                        pattern,
                    ));
                self.runtime
                    .chat
                    .messages
                    .push(("tool_result".to_string(), tool_call.id.clone()));
            }

            if tool_name == "read" {
                let file_path = tool_call
                    .arguments
                    .get("file_path")
                    .and_then(|v| v.as_str())
                    .unwrap_or("file")
                    .to_string();
                self.runtime
                    .blocks
                    .read
                    .push(crate::tui::blocks::ReadBlock::new(
                        tool_call.id.clone(),
                        file_path,
                    ));
                self.runtime
                    .chat
                    .messages
                    .push(("read".to_string(), tool_call.id.clone()));
            }

            if tool_name == "edit" {
                let file_path = tool_call
                    .arguments
                    .get("file_path")
                    .and_then(|v| v.as_str())
                    .unwrap_or("file")
                    .to_string();
                let old_string = tool_call
                    .arguments
                    .get("old_string")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let new_string = tool_call
                    .arguments
                    .get("new_string")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let start_line = 1;

                if let Some(block) = self.runtime.blocks.edit.last_mut() {
                    if block.is_pending() {
                        block.set_diff_data(file_path, old_string, new_string, start_line);
                    }
                }
            }

            if tool_name == "write" {
                let file_path = tool_call
                    .arguments
                    .get("file_path")
                    .and_then(|v| v.as_str())
                    .unwrap_or("file")
                    .to_string();
                let content = tool_call
                    .arguments
                    .get("content")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();

                if let Some(block) = self.runtime.blocks.write.last_mut() {
                    if block.is_pending() {
                        block.set_content(file_path, content);
                    }
                }
            }

            if tool_name == "explore" || tool_name == "Task" {
                let prompt = tool_call
                    .arguments
                    .get("prompt")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Exploring...")
                    .to_string();
                tracing::info!(
                    "spawn_tool_execution: creating ExploreBlock for '{}' with id={}",
                    tool_name,
                    tool_call.id
                );
                self.runtime
                    .blocks
                    .explore
                    .push(crate::tui::blocks::ExploreBlock::with_tool_id(
                        prompt,
                        tool_call.id.clone(),
                    ));
                self.runtime
                    .chat
                    .messages
                    .push(("explore".to_string(), tool_call.id.clone()));
                if self.ui.scroll_system.scroll.auto_scroll {
                    self.ui.scroll_system.scroll.request_scroll_to_bottom();
                }
            }

            if tool_name == "build" {
                let prompt = tool_call
                    .arguments
                    .get("prompt")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Building...")
                    .to_string();
                tracing::info!(
                    "spawn_tool_execution: creating BuildBlock for 'build' with id={}",
                    tool_call.id
                );
                self.runtime
                    .blocks
                    .build
                    .push(crate::tui::blocks::BuildBlock::with_tool_id(
                        prompt,
                        tool_call.id.clone(),
                    ));
                self.runtime
                    .chat
                    .messages
                    .push(("build".to_string(), tool_call.id.clone()));
                if self.ui.scroll_system.scroll.auto_scroll {
                    self.ui.scroll_system.scroll.request_scroll_to_bottom();
                }
            }
        }
    }

    /// Handle tool approval decision from DecisionPrompt
    pub(crate) fn handle_tool_approval_answer(
        &mut self,
        answers: &[crate::tui::components::PromptAnswer],
    ) {
        use crate::tui::components::PromptAnswer;

        self.runtime.approval_requested_at = None;
        let approved = matches!(answers.first(), Some(PromptAnswer::Selected(0)));
        let queued = std::mem::take(&mut self.runtime.queued_tools);

        if approved {
            // Re-run spawn_tool_execution; the supervised check will be skipped
            // because queued_tools is now empty and we temporarily set Autonomous
            let prev = self.runtime.permission_mode;
            self.runtime.permission_mode = krusty_core::tools::registry::PermissionMode::Autonomous;
            self.spawn_tool_execution(queued);
            self.runtime.permission_mode = prev;
        } else {
            // Denied - generate error results for each queued write tool
            let mut results: Vec<Content> = Vec::new();
            for tool in &queued {
                results.push(Content::ToolResult {
                    tool_use_id: tool.id.clone(),
                    output: serde_json::Value::String("Denied by user".to_string()),
                    is_error: Some(true),
                });
            }
            // Include any pending results from intercepted plan tools
            let pending = std::mem::take(&mut self.runtime.pending_tool_results);
            results.extend(pending);
            if !results.is_empty() {
                self.stop_streaming();
                self.handle_tool_results(results);
            }
        }
    }

    /// Handle completed tool results
    pub fn handle_tool_results(&mut self, tool_results: Vec<Content>) {
        if tool_results.is_empty() {
            return;
        }

        tracing::info!(
            result_count = tool_results.len(),
            explore_block_count = self.runtime.blocks.explore.len(),
            "handle_tool_results called"
        );

        // Update blocks with results
        for result in &tool_results {
            if let Content::ToolResult {
                tool_use_id,
                output,
                ..
            } = result
            {
                let output_str = match output {
                    serde_json::Value::String(s) => s.as_str(),
                    _ => "",
                };

                tracing::info!(
                    tool_use_id = %tool_use_id,
                    output_len = output_str.len(),
                    has_summary = output_str.contains("**Summary**"),
                    "Processing tool result"
                );

                self.update_tool_result_block(tool_use_id, output_str);
                self.update_read_block(tool_use_id, output_str);
                self.update_bash_block(tool_use_id, output_str);
                self.update_explore_block(tool_use_id, output_str);
                self.update_build_block(tool_use_id, output_str);
            }
        }

        // Combine with any pending results
        let mut all_results = std::mem::take(&mut self.runtime.pending_tool_results);
        all_results.extend(tool_results);

        // Process queued tools if any explore tools completed
        if !self.runtime.queued_tools.is_empty() {
            let queued = std::mem::take(&mut self.runtime.queued_tools);
            tracing::info!(
                "handle_tool_results: processing {} queued tools",
                queued.len()
            );
            // Clear executing state so UI shows idle while queued tools wait to run
            self.runtime.chat.is_executing_tools = false;
            self.spawn_tool_execution(queued);
            // Store results for later
            self.runtime.pending_tool_results = all_results;
            return;
        }

        // If decision prompt is visible, defer tool results until user decides
        // This prevents the AI from continuing while waiting for user input
        if self.ui.decision_prompt.visible {
            tracing::info!(
                "Decision prompt visible - deferring {} tool results",
                all_results.len()
            );
            self.runtime.pending_tool_results = all_results;
            self.stop_tool_execution();
            return;
        }

        // Keep exploration budget tracking internal. Do not inject warning text into model context.
        const EXPLORATION_BUDGET_SOFT: usize = 15;
        const EXPLORATION_BUDGET_HARD: usize = 30;
        if self.runtime.exploration_budget_count >= EXPLORATION_BUDGET_HARD {
            tracing::warn!(
                exploration_budget_count = self.runtime.exploration_budget_count,
                "Exploration budget hard threshold reached"
            );
        } else if self.runtime.exploration_budget_count >= EXPLORATION_BUDGET_SOFT {
            tracing::info!(
                exploration_budget_count = self.runtime.exploration_budget_count,
                "Exploration budget soft threshold reached"
            );
        }

        // Add tool results to conversation
        let tool_result_msg = crate::ai::types::ModelMessage {
            role: crate::ai::types::Role::User,
            content: all_results,
        };

        self.stop_tool_execution();
        self.runtime.chat.conversation.push(tool_result_msg.clone());
        self.save_model_message(&tool_result_msg);

        if let Some(diagnostic) = self.detect_repeated_tool_failures(&tool_result_msg.content) {
            tracing::warn!(
                threshold = REPEATED_TOOL_FAILURE_THRESHOLD,
                diagnostic = %diagnostic,
                "Fail-fast: stopping repeated tool failure loop"
            );
            self.runtime
                .chat
                .messages
                .push(("system".to_string(), diagnostic));
            self.stop_streaming();
            return;
        }

        // Enforce agentic loop iteration limit
        if self.runtime.agent_state.current_turn >= MAX_ITERATIONS {
            tracing::warn!(
                "Agentic loop hit iteration limit ({}), stopping",
                MAX_ITERATIONS
            );
            self.runtime.chat.messages.push((
                "system".to_string(),
                format!(
                    "Reached maximum agentic loop iterations ({}). Stopping.",
                    MAX_ITERATIONS
                ),
            ));
            return;
        }

        // Continue conversation with AI
        self.send_to_ai();
    }

    /// Update ToolResultBlock with output
    fn update_tool_result_block(&mut self, tool_use_id: &str, output_str: &str) {
        for block in &mut self.runtime.blocks.tool_result {
            if block.tool_use_id() == tool_use_id {
                block.set_results(output_str);
                block.complete();
                break;
            }
        }
    }

    /// Update ReadBlock with content
    fn update_read_block(&mut self, tool_use_id: &str, output_str: &str) {
        for block in &mut self.runtime.blocks.read {
            if block.tool_use_id() == tool_use_id {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(output_str) {
                    let content = json.get("content").and_then(|v| v.as_str()).unwrap_or("");
                    let total_lines = json
                        .get("total_lines")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0) as usize;
                    let lines_returned = json
                        .get("lines_returned")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0) as usize;
                    block.set_content(content.to_string(), total_lines, lines_returned);
                } else {
                    let line_count = output_str.lines().count();
                    block.set_content(output_str.to_string(), line_count, line_count);
                }
                break;
            }
        }
    }

    /// Update BashBlock for background processes
    fn update_bash_block(&mut self, tool_use_id: &str, output_str: &str) {
        for block in &mut self.runtime.blocks.bash {
            if block.tool_use_id() == Some(tool_use_id) {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(output_str) {
                    if let Some(process_id) = json.get("processId").and_then(|v| v.as_str()) {
                        block.set_background_process_id(process_id.to_string());
                        tracing::info!(
                            tool_use_id = %tool_use_id,
                            process_id = %process_id,
                            "BashBlock converted to background process"
                        );
                    }
                }
                break;
            }
        }
    }

    /// Update ExploreBlock with results
    fn update_explore_block(&mut self, tool_use_id: &str, output_str: &str) {
        tracing::info!(
            explore_blocks = self.runtime.blocks.explore.len(),
            tool_use_id = %tool_use_id,
            "Looking for matching ExploreBlock"
        );
        for block in &mut self.runtime.blocks.explore {
            if block.tool_use_id() == Some(tool_use_id) {
                tracing::info!(
                    tool_use_id = %tool_use_id,
                    output_len = output_str.len(),
                    "Found matching ExploreBlock, completing with output"
                );
                block.complete(output_str.to_string());
                break;
            }
        }
    }

    /// Update BuildBlock with results
    fn update_build_block(&mut self, tool_use_id: &str, output_str: &str) {
        for block in &mut self.runtime.blocks.build {
            if block.tool_use_id() == Some(tool_use_id) {
                tracing::info!(
                    tool_use_id = %tool_use_id,
                    output_len = output_str.len(),
                    "Found matching BuildBlock, completing with output"
                );
                block.complete(output_str.to_string());
                break;
            }
        }
    }

    fn detect_repeated_tool_failures(&mut self, tool_results: &[Content]) -> Option<String> {
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

            let output_str = match output {
                serde_json::Value::String(s) => Cow::Borrowed(s.as_str()),
                other => Cow::Owned(other.to_string()),
            };

            let Some((tool_name, args_hash)) = self.find_tool_use_metadata(tool_use_id) else {
                continue;
            };

            let (error_code, error_fingerprint) = extract_error_signature(output_str.as_ref());
            let signature = format!(
                "{}|{}|{}|{}",
                tool_name, error_code, error_fingerprint, args_hash
            );
            let count = self
                .runtime
                .tool_failure_signatures
                .entry(signature)
                .and_modify(|c| *c += 1)
                .or_insert(1);

            if *count >= REPEATED_TOOL_FAILURE_THRESHOLD {
                return Some(format!(
                    "Stopping tool loop: '{}' failed {} times with the same '{}' error. I need a different tool strategy.",
                    tool_name, *count, error_code
                ));
            }
        }

        if saw_success {
            self.runtime.tool_failure_signatures.clear();
        }

        None
    }

    fn find_tool_use_metadata(&self, tool_use_id: &str) -> Option<(String, u64)> {
        for message in self.runtime.chat.conversation.iter().rev() {
            for content in &message.content {
                let Content::ToolUse { id, name, input } = content else {
                    continue;
                };
                if id != tool_use_id {
                    continue;
                }

                let mut hasher = DefaultHasher::new();
                input.to_string().hash(&mut hasher);
                return Some((name.clone(), hasher.finish()));
            }
        }

        None
    }

    /// Check if a tool approval prompt has timed out and auto-reject if so
    pub fn check_approval_timeout(&mut self) {
        if self.ui.decision_prompt.prompt_type != crate::tui::components::PromptType::ToolApproval
            || !self.ui.decision_prompt.visible
        {
            return;
        }

        let Some(requested_at) = self.runtime.approval_requested_at else {
            return;
        };

        if requested_at.elapsed() >= APPROVAL_TIMEOUT {
            tracing::warn!("Tool approval timed out after {:?}", APPROVAL_TIMEOUT);
            self.runtime.approval_requested_at = None;
            self.ui.decision_prompt.hide();
            self.runtime.chat.messages.push((
                "system".to_string(),
                "Tool approval timed out (5 min). Automatically denied.".to_string(),
            ));

            // Auto-reject all queued tools
            let queued = std::mem::take(&mut self.runtime.queued_tools);
            let mut results: Vec<Content> = Vec::new();
            for tool in &queued {
                results.push(Content::ToolResult {
                    tool_use_id: tool.id.clone(),
                    output: serde_json::Value::String(
                        "Tool approval timed out after 5 minutes".to_string(),
                    ),
                    is_error: Some(true),
                });
            }
            let pending = std::mem::take(&mut self.runtime.pending_tool_results);
            results.extend(pending);
            if !results.is_empty() {
                self.stop_streaming();
                self.handle_tool_results(results);
            }
        }
    }
}

fn truncate_tool_output(output: &str) -> String {
    if output.len() <= MAX_TOOL_OUTPUT_CHARS {
        return output.to_string();
    }

    let truncated_len = floor_char_boundary(output, MAX_TOOL_OUTPUT_CHARS);
    let truncated = &output[..truncated_len];
    let break_point = truncated.rfind('\n').unwrap_or(truncated_len);
    let clean = &output[..break_point];
    format!(
        "{}\n\n[Output truncated: {} chars, showing first {}]",
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
        classify_error_code, extract_error_signature, normalize_error_fingerprint,
        truncate_tool_output, MAX_TOOL_OUTPUT_CHARS,
    };

    #[test]
    fn extract_error_signature_supports_structured_error_object() {
        let output = r#"{"error":{"code":"invalid_parameters","message":"Invalid parameters: missing field `prompt`"}}"#;
        let (code, fingerprint) = extract_error_signature(output);
        assert_eq!(code, "invalid_parameters");
        assert!(fingerprint.contains("missing field"));
    }

    #[test]
    fn extract_error_signature_supports_legacy_error_string() {
        let output = r#"{"error":"Invalid parameters: missing field `pattern`"}"#;
        let (code, fingerprint) = extract_error_signature(output);
        assert_eq!(code, "invalid_parameters");
        assert!(fingerprint.contains("missing field"));
    }

    #[test]
    fn classify_error_code_covers_common_categories() {
        assert_eq!(
            classify_error_code("Invalid parameters: missing field `pattern`"),
            "invalid_parameters"
        );
        assert_eq!(classify_error_code("Unknown tool: nope"), "unknown_tool");
        assert_eq!(
            classify_error_code("Access denied: path is outside workspace"),
            "access_denied"
        );
    }

    #[test]
    fn normalize_error_fingerprint_collapses_whitespace() {
        let normalized = normalize_error_fingerprint("  A   spaced\n error\tmessage  ");
        assert_eq!(normalized, "a spaced error message");
    }

    #[test]
    fn truncate_tool_output_handles_utf8_boundaries() {
        let prefix = "a".repeat(MAX_TOOL_OUTPUT_CHARS - 1);
        let output = format!("{prefix}🙂tail");
        let truncated = truncate_tool_output(&output);
        assert!(truncated.starts_with(&prefix));
        assert!(truncated.contains("Output truncated"));
    }
}
