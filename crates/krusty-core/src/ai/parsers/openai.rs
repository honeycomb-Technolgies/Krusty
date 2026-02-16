//! OpenAI-compatible SSE parser for chat/completions format

use std::collections::HashMap;

use anyhow::Result;
use serde_json::Value;

use crate::ai::sse::{SseEvent, SseParser, ToolCallAccumulator};
use crate::ai::types::{AiToolCall, FinishReason, Usage};

/// OpenAI-compatible SSE parser for chat/completions format
pub struct OpenAIParser {
    /// Track tool calls being accumulated
    tool_accumulators: std::sync::Mutex<HashMap<String, ToolCallAccumulator>>,
    /// Preserve tool call ordering for deterministic completion
    tool_order: std::sync::Mutex<Vec<String>>,
    /// Map Responses API item ids to call ids for interleaved argument deltas
    response_item_to_call: std::sync::Mutex<HashMap<String, String>>,
}

impl OpenAIParser {
    pub fn new() -> Self {
        Self {
            tool_accumulators: std::sync::Mutex::new(HashMap::new()),
            tool_order: std::sync::Mutex::new(Vec::new()),
            response_item_to_call: std::sync::Mutex::new(HashMap::new()),
        }
    }

    /// Lock tool accumulators with proper error handling
    fn lock_tool_accumulators(
        &self,
    ) -> anyhow::Result<std::sync::MutexGuard<'_, HashMap<String, ToolCallAccumulator>>> {
        self.tool_accumulators
            .lock()
            .map_err(|e| anyhow::anyhow!("Tool accumulators lock poisoned: {}", e))
    }

    fn lock_tool_order(&self) -> anyhow::Result<std::sync::MutexGuard<'_, Vec<String>>> {
        self.tool_order
            .lock()
            .map_err(|e| anyhow::anyhow!("Tool order lock poisoned: {}", e))
    }

    fn lock_response_item_map(
        &self,
    ) -> anyhow::Result<std::sync::MutexGuard<'_, HashMap<String, String>>> {
        self.response_item_to_call
            .lock()
            .map_err(|e| anyhow::anyhow!("Response item map lock poisoned: {}", e))
    }

    fn register_tool_call(
        &self,
        key: String,
        id: &str,
        name: &str,
        item_id: Option<String>,
    ) -> anyhow::Result<bool> {
        let mut inserted = false;
        {
            let mut accumulators = self.lock_tool_accumulators()?;
            if !accumulators.contains_key(&key) {
                accumulators.insert(
                    key.clone(),
                    ToolCallAccumulator::new(id.to_string(), name.to_string()),
                );
                inserted = true;
            }
        }

        if inserted {
            let mut order = self.lock_tool_order()?;
            if !order.contains(&key) {
                order.push(key.clone());
            }
        }

        if let Some(item_id) = item_id {
            if !item_id.is_empty() {
                let mut map = self.lock_response_item_map()?;
                map.insert(item_id, key);
            }
        }

        Ok(inserted)
    }

    fn append_tool_arguments(&self, key: &str, delta: &str) -> anyhow::Result<Option<String>> {
        let mut accumulators = self.lock_tool_accumulators()?;
        if let Some(acc) = accumulators.get_mut(key) {
            acc.add_arguments(delta);
            return Ok(Some(acc.id.clone()));
        }
        Ok(None)
    }

    fn resolve_responses_tool_key(&self, json: &Value) -> anyhow::Result<Option<String>> {
        if let Some(call_id) = json
            .get("call_id")
            .and_then(|id| id.as_str())
            .filter(|id| !id.is_empty())
        {
            return Ok(Some(call_id.to_string()));
        }

        if let Some(item) = json.get("item") {
            if let Some(call_id) = item
                .get("call_id")
                .and_then(|id| id.as_str())
                .filter(|id| !id.is_empty())
            {
                return Ok(Some(call_id.to_string()));
            }
            if let Some(item_id) = item
                .get("id")
                .and_then(|id| id.as_str())
                .filter(|id| !id.is_empty())
            {
                if let Some(key) = self.lock_response_item_map()?.get(item_id).cloned() {
                    return Ok(Some(key));
                }
            }
        }

        if let Some(item_id) = json
            .get("item_id")
            .and_then(|id| id.as_str())
            .filter(|id| !id.is_empty())
        {
            if let Some(key) = self.lock_response_item_map()?.get(item_id).cloned() {
                return Ok(Some(key));
            }
        }

        // Some providers omit call_id/item_id in deltas when there is only one in-flight call.
        let accumulators = self.lock_tool_accumulators()?;
        if accumulators.len() == 1 {
            return Ok(accumulators.keys().next().cloned());
        }

        Ok(None)
    }

    fn drain_tool_calls(&self) -> anyhow::Result<Vec<AiToolCall>> {
        let keys = {
            let mut order = self.lock_tool_order()?;
            std::mem::take(&mut *order)
        };

        let mut accumulators = self.lock_tool_accumulators()?;
        let mut tool_calls = Vec::new();

        for key in keys {
            if let Some(mut acc) = accumulators.remove(&key) {
                tool_calls.push(acc.force_complete());
            }
        }

        for (_, mut acc) in accumulators.drain() {
            tool_calls.push(acc.force_complete());
        }

        self.lock_response_item_map()?.clear();
        Ok(tool_calls)
    }

    fn parse_responses_usage(usage_obj: &Value) -> Option<Usage> {
        let input = usage_obj
            .get("input_tokens")
            .or_else(|| usage_obj.get("input"))
            .and_then(|t| t.as_u64())
            .unwrap_or(0) as usize;

        let output = usage_obj
            .get("output_tokens")
            .or_else(|| usage_obj.get("output"))
            .and_then(|t| t.as_u64())
            .unwrap_or(0) as usize;

        let cached = usage_obj
            .get("cached_input")
            .or_else(|| usage_obj.get("cache_read_input_tokens"))
            .or_else(|| {
                usage_obj
                    .get("input_tokens_details")
                    .and_then(|d| d.get("cached_tokens"))
            })
            .and_then(|t| t.as_u64())
            .unwrap_or(0) as usize;

        if input == 0 && output == 0 && cached == 0 {
            return None;
        }

        Some(Usage {
            prompt_tokens: input,
            completion_tokens: output,
            total_tokens: input + output,
            cache_creation_input_tokens: 0,
            cache_read_input_tokens: cached,
        })
    }

    fn responses_finish_reason(response: Option<&Value>) -> FinishReason {
        let Some(response) = response else {
            return FinishReason::Stop;
        };

        let Some(status) = response.get("status").and_then(|s| s.as_str()) else {
            return FinishReason::Stop;
        };

        match status {
            "completed" => FinishReason::Stop,
            "incomplete" => {
                let reason = response
                    .get("incomplete_details")
                    .and_then(|d| d.get("reason"))
                    .and_then(|r| r.as_str())
                    .unwrap_or("incomplete");
                match reason {
                    "max_output_tokens" | "max_tokens" | "length" => FinishReason::Length,
                    "tool_calls" | "tool_use" => FinishReason::ToolCalls,
                    other => FinishReason::Other(format!("incomplete:{other}")),
                }
            }
            "cancelled" => FinishReason::Other("cancelled".to_string()),
            "failed" => FinishReason::Other("failed".to_string()),
            other => FinishReason::Other(other.to_string()),
        }
    }

    /// Parse OpenAI Responses API event format
    /// Used by GPT-5 models via OpenCode Zen and ChatGPT Codex
    fn parse_responses_api_event(
        &self,
        json: &Value,
        event_type: &str,
    ) -> anyhow::Result<SseEvent> {
        tracing::debug!("Responses API event: {} - {:?}", event_type, json);

        match event_type {
            // Text content delta
            "response.output_text.delta" => {
                if let Some(delta) = json.get("delta").and_then(|d| d.as_str()) {
                    if !delta.is_empty() {
                        return Ok(SseEvent::TextDelta(delta.to_string()));
                    }
                }
            }

            // Reasoning/thinking deltas (OpenAI o1, codex, GPT-5 reasoning models)
            "response.reasoning_summary_text.delta" | "response.reasoning_text.delta" => {
                if let Some(delta) = json.get("delta").and_then(|d| d.as_str()) {
                    if !delta.is_empty() {
                        tracing::debug!(
                            "Reasoning delta: {}...",
                            &delta.chars().take(50).collect::<String>()
                        );
                        return Ok(SseEvent::ThinkingDelta {
                            index: 0,
                            thinking: delta.to_string(),
                        });
                    }
                }
            }

            // Reasoning part started - emit ThinkingStart
            "response.reasoning_summary_part.added" => {
                tracing::info!("Reasoning block started");
                return Ok(SseEvent::ThinkingStart { index: 0 });
            }

            // Reasoning complete
            "response.reasoning_summary_text.done"
            | "response.reasoning_text.done"
            | "response.reasoning_summary_part.done" => {
                tracing::info!("Reasoning block complete");
                return Ok(SseEvent::ThinkingComplete {
                    index: 0,
                    thinking: String::new(), // Already sent via deltas
                    signature: String::new(),
                });
            }

            // Response completed - check for accumulated tool calls and extract usage
            "response.done" | "response.completed" => {
                let usage = json
                    .get("response")
                    .and_then(|r| r.get("usage"))
                    .and_then(Self::parse_responses_usage);

                if let Some(usage) = &usage {
                    tracing::info!(
                        "Responses API usage: input={}, output={}, cached={}",
                        usage.prompt_tokens,
                        usage.completion_tokens,
                        usage.cache_read_input_tokens
                    );
                }

                let tool_calls = self.drain_tool_calls()?;
                if !tool_calls.is_empty() {
                    tracing::info!(
                        "Responses API completing with {} tool calls",
                        tool_calls.len()
                    );
                    return Ok(SseEvent::FinishWithToolCalls { tool_calls, usage });
                }

                return Ok(SseEvent::Finish {
                    reason: Self::responses_finish_reason(json.get("response")),
                    usage,
                });
            }

            // Function/tool call start - multiple event types for different APIs
            "response.function_call_arguments.start"
            | "response.output_item.added"
            | "response.output_item.done" => {
                if let Some(item) = json.get("item") {
                    if item.get("type").and_then(|t| t.as_str()) == Some("function_call") {
                        let call_id = item
                            .get("call_id")
                            .and_then(|i| i.as_str())
                            .unwrap_or("")
                            .to_string();
                        let item_id = item.get("id").and_then(|i| i.as_str()).map(str::to_string);
                        let name = item
                            .get("name")
                            .and_then(|n| n.as_str())
                            .unwrap_or("")
                            .to_string();

                        let key = if !call_id.is_empty() {
                            call_id
                        } else if let Some(item_id) = item_id.as_deref().filter(|id| !id.is_empty())
                        {
                            format!("item:{}", item_id)
                        } else {
                            format!("tool-{}", self.lock_tool_order()?.len())
                        };

                        let tool_id = key.clone();

                        if !name.is_empty() {
                            let inserted =
                                self.register_tool_call(key.clone(), &tool_id, &name, item_id)?;

                            if let Some(arguments) = item.get("arguments").and_then(|a| a.as_str())
                            {
                                let _ = self.append_tool_arguments(&key, arguments)?;
                            }

                            tracing::info!(
                                "Responses API tool call start: id={}, name={}",
                                tool_id,
                                name
                            );
                            if inserted {
                                return Ok(SseEvent::ToolCallStart { id: tool_id, name });
                            }
                        }
                    }
                }
            }

            // Function arguments delta
            "response.function_call_arguments.delta" => {
                if let Some(delta) = json.get("delta").and_then(|d| d.as_str()) {
                    if let Some(key) = self.resolve_responses_tool_key(json)? {
                        if let Some(id) = self.append_tool_arguments(&key, delta)? {
                            return Ok(SseEvent::ToolCallDelta {
                                id,
                                delta: delta.to_string(),
                            });
                        }
                    }
                }
            }

            // Function call done - arguments are complete
            "response.function_call_arguments.done" => {
                // Arguments complete, tool call will be finalized on response.done
                if let Some(arguments) = json.get("arguments").and_then(|a| a.as_str()) {
                    if let Some(key) = self.resolve_responses_tool_key(json)? {
                        let mut accumulators = self.lock_tool_accumulators()?;
                        if let Some(acc) = accumulators.get_mut(&key) {
                            if acc.arguments.is_empty() {
                                acc.add_arguments(arguments);
                            }
                            tracing::debug!(
                                "Tool call arguments complete: id={}, args_len={}",
                                acc.id,
                                acc.arguments.len()
                            );
                        }
                    }
                }
            }

            // Usage info - handle both standard Responses API and Codex field names
            "response.usage" => {
                // Try nested "usage" object first, then top-level fields
                let usage_obj = json.get("usage").unwrap_or(json);

                if let Some(usage) = Self::parse_responses_usage(usage_obj) {
                    tracing::info!(
                        "Responses API usage: input={}, output={}, cached={}",
                        usage.prompt_tokens,
                        usage.completion_tokens,
                        usage.cache_read_input_tokens
                    );
                    return Ok(SseEvent::Usage(usage));
                }
            }

            // Other events we can skip
            _ => {
                tracing::trace!("Skipping Responses API event: {}", event_type);
            }
        }

        Ok(SseEvent::Skip)
    }
}

impl Default for OpenAIParser {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl SseParser for OpenAIParser {
    async fn parse_event(&self, json: &Value) -> Result<SseEvent> {
        // Check for error response first
        // OpenAI format: {"error": {"message": "...", "type": "...", "code": "..."}}
        if let Some(error) = json.get("error") {
            let message = error
                .get("message")
                .and_then(|m| m.as_str())
                .unwrap_or("Unknown error");
            let error_type = error
                .get("type")
                .and_then(|t| t.as_str())
                .unwrap_or("unknown");
            return Err(anyhow::anyhow!(
                "OpenAI API error ({}): {}",
                error_type,
                message
            ));
        }

        // Check for Responses API format (has "type" field)
        if let Some(event_type) = json.get("type").and_then(|t| t.as_str()) {
            // Check for error events in Responses API
            if event_type == "error" || event_type.contains("error") {
                let message = json
                    .get("message")
                    .and_then(|m| m.as_str())
                    .or_else(|| json.get("error").and_then(|e| e.as_str()))
                    .unwrap_or("Unknown error");
                return Err(anyhow::anyhow!("OpenAI Responses API error: {}", message));
            }
            return self.parse_responses_api_event(json, event_type);
        }

        // OpenAI Chat Completions format: {"choices": [{"index": 0, "delta": {...}, "finish_reason": null}]}
        let choices = json.get("choices").and_then(|c| c.as_array());

        if let Some(choices) = choices {
            if let Some(choice) = choices.first() {
                // Check for finish_reason
                if let Some(reason) = choice.get("finish_reason").and_then(|r| r.as_str()) {
                    if reason == "stop" || reason == "end_turn" {
                        return Ok(SseEvent::Finish {
                            reason: FinishReason::Stop,
                            usage: None,
                        });
                    }
                    if reason == "tool_calls" {
                        // Complete all accumulated tool calls
                        let tool_calls = self.drain_tool_calls()?;

                        if tool_calls.is_empty() {
                            return Ok(SseEvent::Finish {
                                reason: FinishReason::ToolCalls,
                                usage: None,
                            });
                        }
                        return Ok(SseEvent::FinishWithToolCalls {
                            tool_calls,
                            usage: None,
                        });
                    }
                    if reason == "length" || reason == "max_tokens" {
                        return Ok(SseEvent::Finish {
                            reason: FinishReason::Length,
                            usage: None,
                        });
                    }
                }

                // Check for delta content
                if let Some(delta) = choice.get("delta") {
                    // Regular text content
                    if let Some(content) = delta.get("content").and_then(|c| c.as_str()) {
                        if !content.is_empty() {
                            return Ok(SseEvent::TextDelta(content.to_string()));
                        }
                    }

                    // Reasoning content (GLM-style thinking)
                    if let Some(reasoning) = delta.get("reasoning_content").and_then(|r| r.as_str())
                    {
                        if !reasoning.is_empty() {
                            // Treat reasoning as thinking delta
                            return Ok(SseEvent::ThinkingDelta {
                                index: 0,
                                thinking: reasoning.to_string(),
                            });
                        }
                    }

                    // Tool calls
                    if let Some(tool_calls) = delta.get("tool_calls").and_then(|t| t.as_array()) {
                        for tool_call in tool_calls {
                            let index = tool_call.get("index").and_then(|i| i.as_u64()).unwrap_or(0)
                                as usize;
                            let key = format!("chat-index:{}", index);

                            // Check for function info (start of tool call)
                            if let Some(function) = tool_call.get("function") {
                                let id = tool_call
                                    .get("id")
                                    .and_then(|i| i.as_str())
                                    .unwrap_or("")
                                    .to_string();
                                let tool_id = if id.is_empty() { key.clone() } else { id };
                                let mut emitted_event: Option<SseEvent> = None;

                                if let Some(name) = function.get("name").and_then(|n| n.as_str()) {
                                    // New tool call starting
                                    let inserted =
                                        self.register_tool_call(key.clone(), &tool_id, name, None)?;
                                    if inserted {
                                        emitted_event = Some(SseEvent::ToolCallStart {
                                            id: tool_id.clone(),
                                            name: name.to_string(),
                                        });
                                    }
                                }

                                if let Some(args) =
                                    function.get("arguments").and_then(|a| a.as_str())
                                {
                                    if let Some(id) = self.append_tool_arguments(&key, args)? {
                                        if emitted_event.is_none() {
                                            emitted_event = Some(SseEvent::ToolCallDelta {
                                                id,
                                                delta: args.to_string(),
                                            });
                                        }
                                    }
                                }

                                if let Some(event) = emitted_event {
                                    return Ok(event);
                                }
                            }
                        }
                    }
                }
            }
        }

        // Check for usage info
        if let Some(usage) = json.get("usage") {
            let prompt_tokens = usage
                .get("prompt_tokens")
                .and_then(|t| t.as_u64())
                .unwrap_or(0) as usize;
            let completion_tokens = usage
                .get("completion_tokens")
                .and_then(|t| t.as_u64())
                .unwrap_or(0) as usize;
            if prompt_tokens > 0 || completion_tokens > 0 {
                return Ok(SseEvent::Usage(Usage {
                    prompt_tokens,
                    completion_tokens,
                    total_tokens: prompt_tokens + completion_tokens,
                    cache_creation_input_tokens: 0,
                    cache_read_input_tokens: 0,
                }));
            }
        }

        // Check for [DONE] marker (OpenAI uses this)
        // This is handled at the SSE line level, but just in case
        Ok(SseEvent::Skip)
    }
}
