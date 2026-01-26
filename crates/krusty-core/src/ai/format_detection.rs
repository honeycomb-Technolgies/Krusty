//! API format detection for multi-provider routing
//!
//! Determines the correct API format for a provider/model combination.
//! Used by both ACP and TUI to route requests correctly.

use super::models::ApiFormat;
use super::providers::ProviderId;

/// Detect the appropriate API format for a provider/model combination
///
/// This is the canonical format detection logic used across Krusty.
/// Provider-specific routing:
/// - OpenCodeZen: model-based detection (Claude→Anthropic, GPT-5→OpenAIResponses, Gemini→Google, etc)
/// - Kimi, OpenAI: OpenAI chat/completions format
/// - All others (Anthropic, OpenRouter, MiniMax, ZAI): Anthropic format
pub fn detect_api_format(provider: ProviderId, model: &str) -> ApiFormat {
    match provider {
        ProviderId::OpenCodeZen => detect_opencodezen_format(model),
        ProviderId::Kimi | ProviderId::OpenAI => ApiFormat::OpenAI,
        _ => ApiFormat::Anthropic,
    }
}

/// Detect API format for OpenCode Zen based on model ID
///
/// Based on OpenCode Zen official documentation:
/// - Claude models + MiniMax M2.1 → Anthropic format (/v1/messages)
/// - GPT-5 models → OpenAI Responses format (/v1/responses)
/// - Gemini models → Google format (/v1/models/{model})
/// - GLM, Kimi, Qwen, Grok, Big Pickle → OpenAI-compatible (/v1/chat/completions)
pub fn detect_opencodezen_format(model_id: &str) -> ApiFormat {
    let id = model_id.to_lowercase();

    // Anthropic format: Claude models AND MiniMax M2.1
    if id.starts_with("claude") || id.starts_with("minimax") {
        return ApiFormat::Anthropic;
    }

    // GPT-5 uses OpenAI Responses format (/v1/responses)
    if id.starts_with("gpt-5") {
        return ApiFormat::OpenAIResponses;
    }

    // Gemini uses Google format (/v1/models/{model})
    if id.starts_with("gemini") {
        return ApiFormat::Google;
    }

    // Everything else uses OpenAI chat/completions:
    // GLM, Kimi, Qwen, Grok, Big Pickle
    ApiFormat::OpenAI
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_api_format_anthropic_provider() {
        assert!(matches!(
            detect_api_format(ProviderId::Anthropic, "claude-sonnet-4"),
            ApiFormat::Anthropic
        ));
    }

    #[test]
    fn test_detect_api_format_kimi_provider() {
        assert!(matches!(
            detect_api_format(ProviderId::Kimi, "moonshot-v1"),
            ApiFormat::OpenAI
        ));
    }

    #[test]
    fn test_detect_api_format_openai_provider() {
        assert!(matches!(
            detect_api_format(ProviderId::OpenAI, "gpt-4"),
            ApiFormat::OpenAI
        ));
    }

    #[test]
    fn test_detect_opencodezen_claude() {
        assert!(matches!(
            detect_opencodezen_format("claude-opus-4-5"),
            ApiFormat::Anthropic
        ));
        assert!(matches!(
            detect_opencodezen_format("claude-sonnet-4"),
            ApiFormat::Anthropic
        ));
    }

    #[test]
    fn test_detect_opencodezen_minimax() {
        assert!(matches!(
            detect_opencodezen_format("minimax-m2.1-free"),
            ApiFormat::Anthropic
        ));
    }

    #[test]
    fn test_detect_opencodezen_gpt5() {
        assert!(matches!(
            detect_opencodezen_format("gpt-5.1-codex"),
            ApiFormat::OpenAIResponses
        ));
        assert!(matches!(
            detect_opencodezen_format("gpt-5-nano"),
            ApiFormat::OpenAIResponses
        ));
    }

    #[test]
    fn test_detect_opencodezen_gemini() {
        assert!(matches!(
            detect_opencodezen_format("gemini-3-pro"),
            ApiFormat::Google
        ));
    }

    #[test]
    fn test_detect_opencodezen_openai_format() {
        assert!(matches!(
            detect_opencodezen_format("glm-4.6"),
            ApiFormat::OpenAI
        ));
        assert!(matches!(
            detect_opencodezen_format("kimi-k2"),
            ApiFormat::OpenAI
        ));
        assert!(matches!(
            detect_opencodezen_format("grok-code"),
            ApiFormat::OpenAI
        ));
        assert!(matches!(
            detect_opencodezen_format("big-pickle"),
            ApiFormat::OpenAI
        ));
    }
}
