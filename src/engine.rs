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
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EngineKind {
    OpenAI,
    Anthropic,
    Gemini,
}

impl EngineKind {
    /// All known engine kinds.
    pub const ALL: [EngineKind; 3] = [EngineKind::OpenAI, EngineKind::Anthropic, EngineKind::Gemini];

    /// Check if this engine has an API key configured in the environment.
    pub fn is_available(self) -> bool {
        let key_var = match self {
            EngineKind::OpenAI => "OPENAI_API_KEY",
            EngineKind::Anthropic => "ANTHROPIC_API_KEY",
            EngineKind::Gemini => "GEMINI_API_KEY",
        };
        env::var("LLM_API_KEY").is_ok() || env::var(key_var).is_ok()
    }

    /// Return the string name for this engine kind.
    pub fn as_str(self) -> &'static str {
        match self {
            EngineKind::OpenAI => "openai",
            EngineKind::Anthropic => "anthropic",
            EngineKind::Gemini => "gemini",
        }
    }

    /// Parse from string.
    pub fn from_str_opt(s: &str) -> Option<Self> {
        match s {
            "openai" => Some(EngineKind::OpenAI),
            "anthropic" => Some(EngineKind::Anthropic),
            "gemini" => Some(EngineKind::Gemini),
            _ => None,
        }
    }

    /// Get default config for this engine (model, base_url) without requiring API key.
    pub fn defaults(self) -> (&'static str, &'static str) {
        match self {
            EngineKind::OpenAI => ("gpt-5-mini-2025-08-07", "https://api.openai.com"),
            EngineKind::Anthropic => ("claude-sonnet-4-20250514", "https://api.anthropic.com"),
            EngineKind::Gemini => ("gemini-2.0-flash", "https://generativelanguage.googleapis.com"),
        }
    }
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
