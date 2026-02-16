//! Streaming API calls
//!
//! Handles SSE streaming responses from different providers.

use anyhow::Result;
use futures::{SinkExt, StreamExt};
use serde_json::Value;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tracing::{debug, error, info, warn};
use url::Url;

use super::config::CallOptions;
use super::core::{AiClient, KRUSTY_SYSTEM_PROMPT};
use crate::ai::format::anthropic::AnthropicFormat;
use crate::ai::format::google::GoogleFormat;
use crate::ai::format::openai::OpenAIFormat;
use crate::ai::format::FormatHandler;
use crate::ai::parsers::{AnthropicParser, GoogleParser, OpenAIParser};
use crate::ai::providers::{ProviderCapabilities, ReasoningFormat};
use crate::ai::reasoning::{ReasoningConfig, DEFAULT_THINKING_BUDGET};
use crate::ai::sse::{
    create_streaming_channels, spawn_buffer_processor, SseParser, SseStreamProcessor,
};
use crate::ai::streaming::StreamPart;
use crate::ai::transform::build_provider_params;
use crate::ai::types::{Content, ModelMessage, Role};

/// Spawn a stream processing task for an HTTP SSE response.
///
/// Handles the common pattern of reading bytes from a response stream,
/// parsing SSE events, and forwarding them through channels. Sends an
/// explicit error signal if the stream fails, ensuring the receiver
/// never waits on a silently-dead channel.
fn spawn_sse_stream_task<S, P>(
    stream: S,
    mut processor: SseStreamProcessor,
    parser: P,
    tx_err: mpsc::UnboundedSender<StreamPart>,
    label: &'static str,
) where
    S: futures::Stream<Item = reqwest::Result<bytes::Bytes>> + Send + 'static,
    P: SseParser + 'static,
{
    tokio::spawn(async move {
        tokio::pin!(stream);
        let mut chunk_count: u64 = 0;
        let mut had_error = false;

        while let Some(chunk) = stream.next().await {
            chunk_count += 1;
            match chunk {
                Ok(bytes) => {
                    if let Err(e) = processor.process_chunk(bytes, &parser).await {
                        warn!("{} chunk #{} parse error: {}", label, chunk_count, e);
                        let _ = tx_err.send(StreamPart::Error {
                            error: format!("{} parse error: {}", label, e),
                        });
                        had_error = true;
                        break;
                    }
                }
                Err(e) => {
                    error!("{} read error at chunk #{}: {}", label, chunk_count, e);
                    let _ = tx_err.send(StreamPart::Error {
                        error: format!("{} read error: {}", label, e),
                    });
                    had_error = true;
                    break;
                }
            }
        }

        if !had_error {
            info!("{} stream ended after {} chunks", label, chunk_count);
        }
        processor.finish().await;
    });
}

fn first_text_block(content: &[Content]) -> Option<&str> {
    content.iter().find_map(|block| match block {
        Content::Text { text } => Some(text.as_str()),
        _ => None,
    })
}

fn collect_system_message_text(messages: &[ModelMessage]) -> String {
    let mut combined = String::new();
    for message in messages.iter().filter(|m| m.role == Role::System) {
        if let Some(text) = first_text_block(&message.content) {
            if !combined.is_empty() {
                combined.push_str("\n\n");
            }
            combined.push_str(text);
        }
    }
    combined
}

fn collect_message_text(content: &[Content], separator: &str) -> String {
    let mut combined = String::new();
    for block in content {
        if let Content::Text { text } = block {
            if !combined.is_empty() {
                combined.push_str(separator);
            }
            combined.push_str(text);
        }
    }
    combined
}

fn build_default_system_prompt(messages: &[ModelMessage], options: &CallOptions) -> String {
    let system = collect_system_message_text(messages);
    if !system.is_empty() {
        format!("{}\n\n{}", KRUSTY_SYSTEM_PROMPT, system)
    } else if let Some(custom) = &options.system_prompt {
        custom.clone()
    } else {
        KRUSTY_SYSTEM_PROMPT.to_string()
    }
}

async fn ensure_success_stream_response(
    response: reqwest::Response,
    call_start: Instant,
    response_label: &str,
    error_label: &str,
) -> Result<reqwest::Response> {
    let status = response.status();
    info!(
        "{}: {} in {:?}",
        response_label,
        status,
        call_start.elapsed()
    );

    if status.is_success() {
        return Ok(response);
    }

    let error_text = response
        .text()
        .await
        .unwrap_or_else(|_| "Unknown error".to_string());
    error!("{}: {} - {}", error_label, status, error_text);
    Err(anyhow::anyhow!(
        "{}: {} - {}",
        error_label,
        status,
        error_text
    ))
}

fn start_sse_stream<P>(
    response: reqwest::Response,
    parser: P,
    label: &'static str,
) -> mpsc::UnboundedReceiver<StreamPart>
where
    P: SseParser + 'static,
{
    let (tx, rx, buffer_tx, buffer_rx) = create_streaming_channels();
    spawn_buffer_processor(buffer_rx, tx.clone());

    let processor = SseStreamProcessor::new(tx.clone(), buffer_tx);
    spawn_sse_stream_task(response.bytes_stream(), processor, parser, tx, label);

    rx
}

enum CodexPayloadState {
    Continue,
    Complete,
    Error,
}

async fn process_codex_ws_payload(
    payload: &str,
    parser: &OpenAIParser,
    processor: &mut SseStreamProcessor,
    tx_err: &mpsc::UnboundedSender<StreamPart>,
) -> CodexPayloadState {
    if let Ok(json) = serde_json::from_str::<Value>(payload) {
        let event_type = json.get("type").and_then(|t| t.as_str()).unwrap_or("");
        if matches!(event_type, "error" | "response.failed") || event_type.contains("error") {
            let detail = AiClient::codex_ws_error_message(&json)
                .unwrap_or_else(|| "unknown websocket error".to_string());
            let _ = tx_err.send(StreamPart::Error {
                error: format!("Codex websocket API error: {}", detail),
            });
            return CodexPayloadState::Error;
        }
        if matches!(event_type, "response.done" | "response.completed") {
            if let Err(e) = processor.process_sse_data(payload, parser).await {
                let _ = tx_err.send(StreamPart::Error {
                    error: format!("Codex websocket parsing error: {}", e),
                });
                return CodexPayloadState::Error;
            }
            return CodexPayloadState::Complete;
        }
    }

    if let Err(e) = processor.process_sse_data(payload, parser).await {
        let _ = tx_err.send(StreamPart::Error {
            error: format!("Codex websocket parsing error: {}", e),
        });
        return CodexPayloadState::Error;
    }

    CodexPayloadState::Continue
}

impl AiClient {
    /// Call the API with streaming response
    pub async fn call_streaming(
        &self,
        messages: Vec<ModelMessage>,
        options: &CallOptions,
    ) -> Result<mpsc::UnboundedReceiver<StreamPart>> {
        let call_start = Instant::now();
        info!("=== API CALL START ===");
        info!(
            "Model: {}, Messages: {}, Tools: {}, Thinking: {}, Format: {:?}",
            self.config().model,
            messages.len(),
            options.tools.as_ref().map(|t| t.len()).unwrap_or(0),
            options.thinking.is_some(),
            self.config().api_format
        );

        // Route to appropriate format handler based on API format
        if self.config().uses_openai_format() {
            return self
                .call_streaming_openai(messages, options, call_start)
                .await;
        }

        if self.config().uses_google_format() {
            return self
                .call_streaming_google(messages, options, call_start)
                .await;
        }

        // Anthropic format (default)
        self.call_streaming_anthropic(messages, options, call_start)
            .await
    }

    /// Streaming call using Anthropic format
    async fn call_streaming_anthropic(
        &self,
        messages: Vec<ModelMessage>,
        options: &CallOptions,
        call_start: Instant,
    ) -> Result<mpsc::UnboundedReceiver<StreamPart>> {
        let format_handler = AnthropicFormat::new();
        let anthropic_messages =
            format_handler.convert_messages(&messages, Some(self.provider_id()));

        // Extract any system messages from conversation (e.g., pinch context)
        let injected_context = collect_system_message_text(&messages);

        // Build system prompt
        let mut system = if let Some(custom) = &options.system_prompt {
            custom.clone()
        } else {
            KRUSTY_SYSTEM_PROMPT.to_string()
        };

        // Handle injected context
        if !injected_context.is_empty() {
            system.push_str("\n\n---\n\n");
            system.push_str(&injected_context);
            info!(
                "Injected {} chars of context into system prompt",
                injected_context.len()
            );
        }

        // Determine max_tokens based on reasoning format
        let fallback_tokens = options.max_tokens.unwrap_or(self.config().max_tokens) as u32;
        let legacy_thinking = options.thinking.is_some();
        let max_tokens = ReasoningConfig::max_tokens_for_format(
            options.reasoning_format,
            fallback_tokens,
            legacy_thinking,
        );

        // Build request body
        let mut body = serde_json::json!({
            "model": self.config().model,
            "messages": anthropic_messages,
            "max_tokens": max_tokens,
            "stream": true,
        });

        // Add system prompt with cache control
        if !system.is_empty() {
            if options.enable_caching {
                body["system"] = serde_json::json!([{
                    "type": "text",
                    "text": system,
                    "cache_control": {"type": "ephemeral"}
                }]);
                debug!("Cache breakpoint added to system prompt");
            } else {
                body["system"] = Value::String(system);
            }
        }

        // Temperature incompatible with reasoning - only add if reasoning is off
        let reasoning_enabled = options.reasoning_format.is_some() || options.thinking.is_some();
        if !reasoning_enabled {
            if let Some(temp) = options.temperature {
                body["temperature"] = serde_json::json!(temp);
            }
        }

        // Build tools array
        let mut all_tools: Vec<Value> = Vec::new();

        if let Some(tools) = &options.tools {
            for tool in tools {
                all_tools.push(serde_json::json!({
                    "name": tool.name,
                    "description": tool.description,
                    "input_schema": tool.input_schema,
                }));
            }
        }

        // Add server-executed tools based on provider capabilities
        let capabilities = ProviderCapabilities::for_provider(self.provider_id());
        self.add_server_tools(&mut all_tools, &mut body, options, &capabilities);

        // Add all tools to body with cache breakpoint on last one
        if !all_tools.is_empty() {
            if options.enable_caching {
                if let Some(last) = all_tools.last_mut() {
                    last["cache_control"] = serde_json::json!({"type": "ephemeral"});
                }
                debug!("Cache breakpoint added to last tool");
            }
            body["tools"] = Value::Array(all_tools);
        }

        // Add reasoning/thinking config
        self.add_reasoning_config(&mut body, options, reasoning_enabled);

        // Add context management
        self.add_context_management(&mut body, options);

        // Add provider-specific parameters
        self.add_provider_params(&mut body, reasoning_enabled);

        debug!("Calling {} API with streaming", self.provider_id());

        // Build beta headers
        let beta_headers = self.build_beta_headers(options);
        let request = self.build_request_with_beta(&self.config().api_url(), &beta_headers);

        // Send request
        info!("Sending API request...");
        let response = request.json(&body).send().await?;
        let response =
            ensure_success_stream_response(response, call_start, "API response", "API error")
                .await?;

        info!("Starting Anthropic stream processing task");
        Ok(start_sse_stream(
            response,
            AnthropicParser::new(),
            "Anthropic",
        ))
    }

    /// Streaming call using OpenAI format
    async fn call_streaming_openai(
        &self,
        messages: Vec<ModelMessage>,
        options: &CallOptions,
        call_start: Instant,
    ) -> Result<mpsc::UnboundedReceiver<StreamPart>> {
        // Check if we're using ChatGPT Codex API (OAuth) vs standard OpenAI API
        let is_chatgpt_codex = self
            .config()
            .base_url
            .as_ref()
            .map(|url| url.contains("chatgpt.com"))
            .unwrap_or(false);

        if is_chatgpt_codex {
            info!(
                "Using ChatGPT Codex format for {} (OAuth)",
                self.config().model
            );
            return self
                .call_streaming_chatgpt_codex_ws(messages, options, call_start)
                .await;
        } else {
            info!(
                "Using OpenAI chat/completions format for {}",
                self.config().model
            );
        }

        let format_handler = OpenAIFormat::new(self.config().api_format);

        let system_prompt = build_default_system_prompt(&messages, options);

        let max_tokens = options.max_tokens.unwrap_or(self.config().max_tokens);

        // Standard OpenAI format (Chat Completions or Responses API)
        let openai_messages = format_handler.convert_messages(&messages, Some(self.provider_id()));

        // Responses API uses "input", Chat Completions uses "messages"
        let (messages_key, max_tokens_key) = if matches!(
            self.config().api_format,
            crate::ai::models::ApiFormat::OpenAIResponses
        ) {
            ("input", "max_output_tokens")
        } else {
            ("messages", "max_tokens")
        };

        let mut body = serde_json::json!({
            "model": self.config().model,
            "stream": true,
        });
        body[max_tokens_key] = serde_json::json!(max_tokens);
        body[messages_key] = serde_json::json!(openai_messages);

        // Add system message at the start
        if let Some(msgs) = body.get_mut(messages_key).and_then(|m| m.as_array_mut()) {
            msgs.insert(
                0,
                serde_json::json!({
                    "role": "system",
                    "content": system_prompt
                }),
            );
        }

        // Add temperature
        if options.thinking.is_none() {
            if let Some(temp) = options.temperature {
                body["temperature"] = serde_json::json!(temp);
            }
        }

        // Add tools
        if let Some(tools) = &options.tools {
            let openai_tools = format_handler.convert_tools(tools);
            if !openai_tools.is_empty() {
                body["tools"] = serde_json::json!(openai_tools);
            }
        }

        debug!("OpenAI request to: {}", self.config().api_url());

        let request = self.build_request(&self.config().api_url());

        info!("Sending OpenAI format request...");
        let response = request.json(&body).send().await?;
        let response =
            ensure_success_stream_response(response, call_start, "API response", "API error")
                .await?;

        info!("Starting OpenAI stream processing task");
        Ok(start_sse_stream(response, OpenAIParser::new(), "OpenAI"))
    }

    /// Streaming call for ChatGPT Codex over WebSocket (no SSE fallback).
    async fn call_streaming_chatgpt_codex_ws(
        &self,
        messages: Vec<ModelMessage>,
        options: &CallOptions,
        call_start: Instant,
    ) -> Result<mpsc::UnboundedReceiver<StreamPart>> {
        let format_handler = OpenAIFormat::new(self.config().api_format);

        let system_prompt = build_default_system_prompt(&messages, options);

        let max_tokens = options.max_tokens.unwrap_or(self.config().max_tokens);
        let body = self.build_chatgpt_codex_body(
            &messages,
            &system_prompt,
            max_tokens,
            options,
            &format_handler,
        );

        let ws_url = Self::resolve_codex_ws_url(&self.config().api_url())?;
        let mut request = self.build_websocket_request(
            ws_url.as_str(),
            &[
                ("OpenAI-Beta", "responses_websockets=2026-02-06"),
                ("originator", "krusty"),
            ],
        )?;
        if let Some(session_id) = &options.session_id {
            match session_id.parse::<tokio_tungstenite::tungstenite::http::HeaderValue>() {
                Ok(value) => {
                    request.headers_mut().insert("session_id", value);
                }
                Err(e) => {
                    warn!("Invalid Codex session_id header '{}': {}", session_id, e);
                }
            }
        }

        info!("Connecting ChatGPT Codex websocket: {}", ws_url);
        let (mut ws_stream, _) = match connect_async(request).await {
            Ok(pair) => pair,
            Err(e) => {
                warn!(
                    "ChatGPT Codex websocket connect failed ({}), falling back to HTTP streaming",
                    e
                );
                return self
                    .call_streaming_chatgpt_codex_http(body, call_start)
                    .await;
            }
        };
        info!(
            "ChatGPT Codex websocket connected in {:?}",
            call_start.elapsed()
        );

        let create_payload = Self::codex_ws_create_payload(body.clone());
        if let Err(e) = ws_stream
            .send(Message::Text(create_payload.to_string()))
            .await
        {
            warn!(
                "ChatGPT Codex websocket send failed ({}), falling back to HTTP streaming",
                e
            );
            return self
                .call_streaming_chatgpt_codex_http(body, call_start)
                .await;
        }

        let first_ws_message = (tokio::time::timeout(Duration::from_secs(2), ws_stream.next())
            .await)
            .unwrap_or_default();

        if matches!(
            first_ws_message,
            Some(Ok(Message::Close(_))) | Some(Err(_)) | None
        ) {
            warn!(
                "ChatGPT Codex websocket closed before first event, falling back to HTTP streaming"
            );
            return self
                .call_streaming_chatgpt_codex_http(body, call_start)
                .await;
        }

        let (tx, rx, buffer_tx, buffer_rx) = create_streaming_channels();
        spawn_buffer_processor(buffer_rx, tx.clone());
        let tx_err = tx.clone();

        let mut processor = SseStreamProcessor::new(tx, buffer_tx);
        let parser = OpenAIParser::new();

        tokio::spawn(async move {
            let (_write, mut read) = ws_stream.split();

            let mut pending_first = first_ws_message;

            loop {
                let msg = if let Some(msg) = pending_first.take() {
                    msg
                } else {
                    match read.next().await {
                        Some(msg) => msg,
                        None => break,
                    }
                };

                match msg {
                    Ok(Message::Text(text)) => {
                        let payload = text.to_string();
                        match process_codex_ws_payload(&payload, &parser, &mut processor, &tx_err)
                            .await
                        {
                            CodexPayloadState::Continue => {}
                            CodexPayloadState::Complete => {
                                break;
                            }
                            CodexPayloadState::Error => break,
                        }
                    }
                    Ok(Message::Binary(bytes)) => {
                        let payload = String::from_utf8_lossy(&bytes);
                        match process_codex_ws_payload(
                            payload.as_ref(),
                            &parser,
                            &mut processor,
                            &tx_err,
                        )
                        .await
                        {
                            CodexPayloadState::Continue => {}
                            CodexPayloadState::Complete => {
                                break;
                            }
                            CodexPayloadState::Error => break,
                        }
                    }
                    Ok(Message::Close(frame)) => {
                        let (code, reason) = frame
                            .as_ref()
                            .map(|f| (f.code.to_string(), f.reason.to_string()))
                            .unwrap_or_else(|| {
                                ("no close code".to_string(), "no close reason".to_string())
                            });
                        let _ = tx_err.send(StreamPart::Error {
                            error: format!(
                                "Codex websocket closed before completion (websocket-only mode): code={}, reason={}",
                                code, reason
                            ),
                        });
                        break;
                    }
                    Ok(Message::Ping(_)) | Ok(Message::Pong(_)) => {}
                    Ok(Message::Frame(_)) => {}
                    Err(e) => {
                        let _ = tx_err.send(StreamPart::Error {
                            error: format!("Codex websocket stream error: {}", e),
                        });
                        break;
                    }
                }
            }

            processor.finish().await;
        });

        Ok(rx)
    }

    async fn call_streaming_chatgpt_codex_http(
        &self,
        body: Value,
        call_start: Instant,
    ) -> Result<mpsc::UnboundedReceiver<StreamPart>> {
        let request = self
            .build_request(&self.config().api_url())
            .header("OpenAI-Beta", "responses=experimental");

        info!("Falling back to ChatGPT Codex HTTP streaming");
        let response = request.json(&body).send().await?;
        let response = ensure_success_stream_response(
            response,
            call_start,
            "ChatGPT Codex HTTP response",
            "ChatGPT Codex HTTP fallback error",
        )
        .await?;

        Ok(start_sse_stream(
            response,
            OpenAIParser::new(),
            "Codex HTTP",
        ))
    }

    /// Streaming call using Google format
    async fn call_streaming_google(
        &self,
        messages: Vec<ModelMessage>,
        options: &CallOptions,
        call_start: Instant,
    ) -> Result<mpsc::UnboundedReceiver<StreamPart>> {
        info!("Using Google/Gemini format for {}", self.config().model);

        let format_handler = GoogleFormat::new();
        let contents = format_handler.convert_messages(&messages, Some(self.provider_id()));

        let system_instruction = build_default_system_prompt(&messages, options);

        let max_tokens = options.max_tokens.unwrap_or(self.config().max_tokens);

        let mut body = serde_json::json!({
            "contents": contents,
            "generationConfig": {
                "maxOutputTokens": max_tokens,
            }
        });

        body["systemInstruction"] = serde_json::json!({
            "parts": [{"text": system_instruction}]
        });

        if let Some(temp) = options.temperature {
            body["generationConfig"]["temperature"] = serde_json::json!(temp);
        }

        if let Some(tools) = &options.tools {
            let google_tools = format_handler.convert_tools(tools);
            if !google_tools.is_empty() {
                body["tools"] = serde_json::json!([{
                    "functionDeclarations": google_tools
                }]);
            }
        }

        debug!("Google request to: {}", self.config().api_url());

        let request = self.build_request(&self.config().api_url());

        info!("Sending Google format request...");
        let response = request.json(&body).send().await?;
        let response =
            ensure_success_stream_response(response, call_start, "API response", "API error")
                .await?;

        info!("Starting Google stream processing task");
        Ok(start_sse_stream(response, GoogleParser::new(), "Google"))
    }

    /// Add server-executed tools (web search, web fetch) to the request
    fn add_server_tools(
        &self,
        all_tools: &mut Vec<Value>,
        body: &mut Value,
        options: &CallOptions,
        capabilities: &ProviderCapabilities,
    ) {
        // Anthropic server-executed web tools
        if capabilities.web_search {
            if let Some(search) = &options.web_search {
                let mut spec = serde_json::json!({
                    "type": "web_search_20250305",
                    "name": "web_search",
                });
                if let Some(max_uses) = search.max_uses {
                    spec["max_uses"] = serde_json::json!(max_uses);
                }
                all_tools.push(spec);
                debug!("Web search tool enabled (server-side)");
            }
        }

        if capabilities.web_fetch {
            if let Some(fetch) = &options.web_fetch {
                let mut spec = serde_json::json!({
                    "type": "web_fetch_20250910",
                    "name": "web_fetch",
                    "citations": { "enabled": fetch.citations_enabled },
                });
                if let Some(max_uses) = fetch.max_uses {
                    spec["max_uses"] = serde_json::json!(max_uses);
                }
                if let Some(max_tokens) = fetch.max_content_tokens {
                    spec["max_content_tokens"] = serde_json::json!(max_tokens);
                }
                all_tools.push(spec);
                debug!("Web fetch tool enabled (server-side)");
            }
        }

        // OpenRouter web search: append :online suffix to model name
        if capabilities.web_plugins && options.web_search.is_some() {
            if let Some(model) = body.get("model").and_then(|m| m.as_str()) {
                if !model.ends_with(":online") {
                    let online_model = format!("{}:online", model);
                    body["model"] = serde_json::json!(online_model);
                    info!(
                        "OpenRouter web search enabled via model suffix: {}",
                        online_model
                    );
                }
            }
        }
    }

    /// Add reasoning/thinking config to the request body
    fn add_reasoning_config(
        &self,
        body: &mut Value,
        options: &CallOptions,
        reasoning_enabled: bool,
    ) {
        let budget_tokens = options.thinking.as_ref().map(|t| t.budget_tokens);

        if let Some(reasoning_config) = ReasoningConfig::build(
            options.reasoning_format,
            reasoning_enabled,
            budget_tokens,
            None,
        ) {
            match options.reasoning_format {
                Some(ReasoningFormat::Anthropic) => {
                    body["thinking"] = reasoning_config;
                    debug!(
                        "Anthropic thinking enabled with budget: {}",
                        budget_tokens.unwrap_or(DEFAULT_THINKING_BUDGET)
                    );
                }
                Some(ReasoningFormat::OpenAI) => {
                    if let Some(obj) = reasoning_config.as_object() {
                        for (k, v) in obj {
                            body[k] = v.clone();
                        }
                    }
                    debug!("OpenAI reasoning enabled with high effort");
                }
                Some(ReasoningFormat::DeepSeek) => {
                    if let Some(obj) = reasoning_config.as_object() {
                        for (k, v) in obj {
                            body[k] = v.clone();
                        }
                    }
                    debug!("DeepSeek reasoning enabled");
                }
                None => {}
            }

            // Opus 4.5 effort config
            if let Some(effort_config) =
                ReasoningConfig::build_opus_effort(&self.config().model, reasoning_enabled)
            {
                body["output_config"] = effort_config;
                debug!("Using high effort for Opus 4.5");
            }
        } else if let Some(thinking) = &options.thinking {
            // Legacy support: if thinking is set without format, assume Anthropic
            body["thinking"] = serde_json::json!({
                "type": "enabled",
                "budget_tokens": thinking.budget_tokens
            });
            debug!(
                "Legacy thinking enabled with budget: {}",
                thinking.budget_tokens
            );

            if let Some(effort_config) =
                ReasoningConfig::build_opus_effort(&self.config().model, true)
            {
                body["output_config"] = effort_config;
            }
        }
    }

    /// Add context management to the request body
    fn add_context_management(&self, body: &mut Value, options: &CallOptions) {
        if let Some(ctx_mgmt) = &options.context_management {
            let caps = ProviderCapabilities::for_provider(self.provider_id());
            if caps.context_management {
                body["context_management"] =
                    serde_json::to_value(ctx_mgmt).unwrap_or(serde_json::Value::Null);
                info!("Context management enabled: {} edits", ctx_mgmt.edits.len());
            } else {
                debug!(
                    "Skipping context_management for provider {:?} (not supported)",
                    self.provider_id()
                );
            }
        }
    }

    /// Add provider-specific parameters to the request body
    fn add_provider_params(&self, body: &mut Value, thinking_enabled: bool) {
        let provider_params =
            build_provider_params(&self.config().model, self.provider_id(), thinking_enabled);

        // Temperature incompatible with reasoning
        if !thinking_enabled {
            if let Some(temp) = provider_params.temperature {
                body["temperature"] = Value::Number(serde_json::Number::from(temp as i32));
                debug!(
                    "Setting temperature: {} for model {}",
                    temp,
                    self.config().model
                );
            }
        }

        if let Some(top_p) = provider_params.top_p {
            if let Some(num) = serde_json::Number::from_f64(top_p as f64) {
                body["top_p"] = Value::Number(num);
                debug!("Setting top_p: {} for model {}", top_p, self.config().model);
            }
        }

        if let Some(top_k) = provider_params.top_k {
            body["top_k"] = Value::Number(serde_json::Number::from(top_k));
            debug!("Setting top_k: {} for model {}", top_k, self.config().model);
        }

        if let Some(chat_args) = provider_params.chat_template_args {
            body["chat_template_args"] = chat_args;
            info!(
                "Enabling chat_template_args for thinking model {}",
                self.config().model
            );
        }
    }

    /// Build beta headers based on options
    fn build_beta_headers(&self, options: &CallOptions) -> Vec<&'static str> {
        let mut beta_headers: Vec<&str> = Vec::new();

        // Add thinking beta headers for Anthropic reasoning format
        let anthropic_thinking =
            matches!(options.reasoning_format, Some(ReasoningFormat::Anthropic))
                || options.thinking.is_some();
        if anthropic_thinking {
            beta_headers.push("interleaved-thinking-2025-05-14");

            // Effort beta for Opus 4.5
            if self.config().model.contains("opus-4-5") {
                beta_headers.push("effort-2025-11-24");
            }
        }

        // Context management beta
        if options.context_management.is_some() {
            beta_headers.push("context-management-2025-06-27");
        }

        // Web tool beta headers
        let caps = ProviderCapabilities::for_provider(self.provider_id());
        if options.web_search.is_some() && caps.web_search {
            beta_headers.push("web-search-2025-03-05");
        }
        if options.web_fetch.is_some() && caps.web_fetch {
            beta_headers.push("web-fetch-2025-09-10");
        }

        beta_headers
    }

    fn resolve_codex_ws_url(api_url: &str) -> Result<Url> {
        let mut url = Url::parse(api_url)
            .map_err(|e| anyhow::anyhow!("Invalid Codex API URL '{}': {}", api_url, e))?;

        url.set_scheme(if url.scheme() == "https" { "wss" } else { "ws" })
            .map_err(|_| anyhow::anyhow!("Failed to set websocket scheme for '{}'", api_url))?;

        Ok(url)
    }

    pub(crate) fn codex_ws_create_payload(body: Value) -> Value {
        match body {
            Value::Object(mut object) => {
                object.insert(
                    "type".to_string(),
                    Value::String("response.create".to_string()),
                );
                Value::Object(object)
            }
            other => serde_json::json!({
                "type": "response.create",
                "response": other
            }),
        }
    }

    pub(crate) fn codex_ws_error_message(event: &Value) -> Option<String> {
        if let Some(message) = event.get("message").and_then(|m| m.as_str()) {
            if !message.is_empty() {
                return Some(message.to_string());
            }
        }

        if let Some(message) = event
            .pointer("/error/message")
            .and_then(|m| m.as_str())
            .or_else(|| {
                event
                    .pointer("/response/error/message")
                    .and_then(|m| m.as_str())
            })
            .or_else(|| {
                event
                    .pointer("/response/status_details/error/message")
                    .and_then(|m| m.as_str())
            })
        {
            if !message.is_empty() {
                return Some(message.to_string());
            }
        }

        if let Some(error_text) = event.get("error").and_then(|e| e.as_str()) {
            if !error_text.is_empty() {
                return Some(error_text.to_string());
            }
        }

        let error_type = event
            .pointer("/error/type")
            .and_then(|t| t.as_str())
            .or_else(|| {
                event
                    .pointer("/response/error/type")
                    .and_then(|t| t.as_str())
            });
        let error_code = event
            .pointer("/error/code")
            .and_then(|t| t.as_str())
            .or_else(|| {
                event
                    .pointer("/response/error/code")
                    .and_then(|t| t.as_str())
            });
        match (error_type, error_code) {
            (Some(error_type), Some(error_code))
                if !error_type.is_empty() || !error_code.is_empty() =>
            {
                Some(format!("{} ({})", error_type, error_code))
            }
            (Some(error_type), None) if !error_type.is_empty() => Some(error_type.to_string()),
            (None, Some(error_code)) if !error_code.is_empty() => Some(error_code.to_string()),
            _ => None,
        }
    }

    fn codex_prompt_cache_key(options: &CallOptions) -> Option<String> {
        options.session_id.clone()
    }

    /// Build request body for ChatGPT Codex API
    ///
    /// ChatGPT Codex has a completely different format than standard OpenAI APIs.
    /// Based on reverse-engineering from: https://simonwillison.net/2025/Nov/9/gpt-5-codex-mini/
    ///
    /// Key differences:
    /// - Uses "instructions" field for system prompt (required, not a message)
    /// - Messages wrapped in {"type": "message", "role": ..., "content": [...]}
    /// - Content items use {"type": "input_text", "text": ...} for user messages
    /// - No max_output_tokens parameter
    /// - Requires: store=false, tool_choice, parallel_tool_calls, reasoning, include
    fn build_chatgpt_codex_body(
        &self,
        messages: &[ModelMessage],
        system_prompt: &str,
        _max_tokens: usize, // Not used - Codex doesn't support this parameter
        options: &CallOptions,
        format_handler: &OpenAIFormat,
    ) -> Value {
        // Convert messages to Codex format
        // Each message is wrapped: {"type": "message", "role": "...", "content": [...]}
        let mut input_messages: Vec<Value> = Vec::new();

        for msg in messages.iter().filter(|m| m.role != Role::System) {
            let role = match msg.role {
                Role::User => "user",
                Role::Assistant => "assistant",
                Role::Tool => "tool",
                Role::System => continue, // Handled via instructions field
            };

            // Check for tool results
            let has_tool_results = msg
                .content
                .iter()
                .any(|c| matches!(c, Content::ToolResult { .. }));

            if has_tool_results {
                for content in &msg.content {
                    if let Content::ToolResult {
                        tool_use_id,
                        output,
                        ..
                    } = content
                    {
                        let output_str = match output {
                            Value::String(s) => s.clone(),
                            other => other.to_string(),
                        };
                        // Tool results in Codex format
                        input_messages.push(serde_json::json!({
                            "type": "function_call_output",
                            "call_id": tool_use_id,
                            "output": output_str
                        }));
                    }
                }
                continue;
            }

            // Check for tool calls (assistant requesting tool use)
            let has_tool_use = msg
                .content
                .iter()
                .any(|c| matches!(c, Content::ToolUse { .. }));

            if has_tool_use && role == "assistant" {
                // First add any text content as a message
                let text_content = collect_message_text(&msg.content, "\n");

                if !text_content.is_empty() {
                    input_messages.push(serde_json::json!({
                        "type": "message",
                        "role": "assistant",
                        "content": [{
                            "type": "output_text",
                            "text": text_content
                        }]
                    }));
                }

                // Then add each tool call as a function_call item
                for content in &msg.content {
                    if let Content::ToolUse { id, name, input } = content {
                        input_messages.push(serde_json::json!({
                            "type": "function_call",
                            "call_id": id,
                            "name": name,
                            "arguments": input.to_string()
                        }));
                    }
                }
                continue;
            }

            // Regular message - extract text
            let text = collect_message_text(&msg.content, "\n");

            if !text.is_empty() {
                // Codex format: wrapped message with typed content
                let content_type = if role == "assistant" {
                    "output_text"
                } else {
                    "input_text"
                };
                input_messages.push(serde_json::json!({
                    "type": "message",
                    "role": role,
                    "content": [{
                        "type": content_type,
                        "text": text
                    }]
                }));
            }
        }

        let prompt_cache_key = Self::codex_prompt_cache_key(options);

        // Determine if thinking/reasoning is enabled
        let thinking_enabled = options.thinking.is_some();
        let reasoning_effort = options
            .codex_reasoning_effort
            .unwrap_or(super::config::CodexReasoningEffort::Medium)
            .as_str();

        // Build Codex request body - exact format from reverse-engineering
        let mut body = serde_json::json!({
            "model": self.config().model,
            "instructions": system_prompt,
            "input": input_messages,
            "tools": [],
            "tool_choice": "auto",
            "parallel_tool_calls": options.codex_parallel_tool_calls,
            "store": false,
            "stream": true,
            "include": [],
            "text": {
                "verbosity": "medium"
            }
        });

        if let Some(cache_key) = prompt_cache_key {
            body["prompt_cache_key"] = serde_json::json!(cache_key);
        }

        // Add reasoning config based on thinking toggle
        // When enabled: map to configured Codex effort with auto summary
        // When disabled: no reasoning
        if thinking_enabled {
            body["reasoning"] = serde_json::json!({
                "effort": reasoning_effort,
                "summary": "auto"
            });
            body["include"] = serde_json::json!(["reasoning.encrypted_content"]);
            debug!(
                "ChatGPT Codex: reasoning enabled (effort={}, summary=auto)",
                reasoning_effort
            );
        } else {
            debug!("ChatGPT Codex: reasoning disabled");
        }

        // Add tools if provided
        if let Some(tools) = &options.tools {
            let codex_tools = format_handler.convert_tools(tools);
            if !codex_tools.is_empty() {
                body["tools"] = serde_json::json!(codex_tools);
            }
        }

        debug!(
            "ChatGPT Codex request: model={}, {} messages, {} tools",
            self.config().model,
            input_messages.len(),
            options.tools.as_ref().map(|t| t.len()).unwrap_or(0)
        );

        body
    }
}
