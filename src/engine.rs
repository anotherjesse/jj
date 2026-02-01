use anyhow::{anyhow, Context, Result};
use serde_json::Value;
use std::env;

/// A tool call returned by the LLM.
#[derive(Debug, Clone)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub arguments: Value,
}

/// Response from a chat completion call.
#[derive(Debug, Clone)]
pub struct ChatResponse {
    pub content: Option<String>,
    pub tool_calls: Vec<ToolCall>,
}

/// Which wire protocol to use.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EngineKind {
    OpenAI,
    Anthropic,
    Gemini,
}

/// Chat completion engine — one implementation per wire protocol.
pub trait Engine: Send + Sync {
    fn chat(&self, messages: &[Value], tools: &[Value]) -> Result<ChatResponse>;
    fn set_model(&mut self, model: String);
    fn model(&self) -> &str;
}

/// Resolve which engine kind to use from environment.
pub fn resolve_engine_kind() -> Result<EngineKind> {
    match env::var("LLM_ENGINE").ok().as_deref() {
        Some("openai") => Ok(EngineKind::OpenAI),
        Some("anthropic") => Ok(EngineKind::Anthropic),
        Some("gemini") => Ok(EngineKind::Gemini),
        Some(other) => Err(anyhow!(
            "invalid LLM_ENGINE={other:?}. Valid options: openai, anthropic, gemini"
        )),
        None => Ok(EngineKind::OpenAI), // backward compatible default
    }
}

/// Resolve API key, base URL, and model for the given engine kind.
/// Generic LLM_* env vars override provider-specific ones.
pub struct EngineConfig {
    pub api_key: String,
    pub base_url: String,
    pub model: String,
}

impl EngineConfig {
    pub fn from_env(kind: EngineKind) -> Result<Self> {
        let (provider_key_var, provider_url_var, provider_model_var, default_url, default_model) =
            match kind {
                EngineKind::OpenAI => (
                    "OPENAI_API_KEY",
                    "OPENAI_BASE_URL",
                    "OPENAI_MODEL",
                    "https://api.openai.com",
                    "gpt-5-mini-2025-08-07",
                ),
                EngineKind::Anthropic => (
                    "ANTHROPIC_API_KEY",
                    "ANTHROPIC_BASE_URL",
                    "ANTHROPIC_MODEL",
                    "https://api.anthropic.com",
                    "claude-sonnet-4-20250514",
                ),
                EngineKind::Gemini => (
                    "GEMINI_API_KEY",
                    "GEMINI_BASE_URL",
                    "GEMINI_MODEL",
                    "https://generativelanguage.googleapis.com",
                    "gemini-2.0-flash",
                ),
            };

        let api_key = env::var("LLM_API_KEY")
            .or_else(|_| env::var(provider_key_var))
            .with_context(|| {
                format!("set LLM_API_KEY or {provider_key_var} for engine {kind:?}")
            })?;

        let base_url = env::var("LLM_BASE_URL")
            .or_else(|_| env::var(provider_url_var))
            .unwrap_or_else(|_| default_url.to_string());

        let model = env::var("LLM_MODEL")
            .or_else(|_| env::var(provider_model_var))
            .unwrap_or_else(|_| default_model.to_string());

        Ok(Self {
            api_key,
            base_url,
            model,
        })
    }
}

/// Build an engine from environment variables.
pub fn create_engine() -> Result<Box<dyn Engine>> {
    let kind = resolve_engine_kind()?;
    create_engine_of_kind(kind)
}

/// Build an engine of a specific kind from environment variables.
pub fn create_engine_of_kind(kind: EngineKind) -> Result<Box<dyn Engine>> {
    let config = EngineConfig::from_env(kind)?;
    match kind {
        EngineKind::OpenAI => {
            let client =
                crate::openai::OpenAIClient::new(config.api_key, config.base_url, config.model);
            Ok(Box::new(client))
        }
        EngineKind::Anthropic => {
            let max_tokens: usize = env::var("ANTHROPIC_MAX_TOKENS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(8192);
            let client = crate::anthropic::AnthropicClient::new(
                config.api_key,
                config.base_url,
                config.model,
                max_tokens,
            );
            Ok(Box::new(client))
        }
        EngineKind::Gemini => {
            let client = crate::gemini_chat::GeminiChatClient::new(
                config.api_key,
                config.base_url,
                config.model,
            );
            Ok(Box::new(client))
        }
    }
}

/// Build an engine from explicit parameters (for callers that already have key/url/model).
#[allow(dead_code)]
pub fn create_engine_with(
    kind: EngineKind,
    api_key: String,
    base_url: String,
    model: String,
) -> Result<Box<dyn Engine>> {
    match kind {
        EngineKind::OpenAI => {
            let client = crate::openai::OpenAIClient::new(api_key, base_url, model);
            Ok(Box::new(client))
        }
        EngineKind::Anthropic => {
            let max_tokens: usize = env::var("ANTHROPIC_MAX_TOKENS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(8192);
            let client =
                crate::anthropic::AnthropicClient::new(api_key, base_url, model, max_tokens);
            Ok(Box::new(client))
        }
        EngineKind::Gemini => {
            let client = crate::gemini_chat::GeminiChatClient::new(api_key, base_url, model);
            Ok(Box::new(client))
        }
    }
}
