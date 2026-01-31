use anyhow::{anyhow, Context, Result};
use reqwest::blocking::Client;
use serde_json::{json, Value};
use std::env;

#[derive(Debug, Clone, Copy)]
pub enum EmbeddingProvider {
    OpenAI,
    Gemini,
}

pub struct EmbeddingClient {
    provider: EmbeddingProvider,
    api_key: String,
    base_url: String,
    model: String,
    http: Client,
}

impl EmbeddingClient {
    pub fn from_env() -> Result<Self> {
        let provider = match env::var("EMBEDDING_PROVIDER").ok().as_deref() {
            Some("gemini") => EmbeddingProvider::Gemini,
            Some("openai") => EmbeddingProvider::OpenAI,
            Some(other) => return Err(anyhow!("unsupported EMBEDDING_PROVIDER: {other}")),
            None => {
                if env::var("OPENAI_API_KEY").is_ok() {
                    EmbeddingProvider::OpenAI
                } else if env::var("GEMINI_API_KEY").is_ok() {
                    EmbeddingProvider::Gemini
                } else {
                    return Err(anyhow!("no embedding provider configured"));
                }
            }
        };

        match provider {
            EmbeddingProvider::OpenAI => {
                let api_key = env::var("OPENAI_API_KEY").context("OPENAI_API_KEY is not set")?;
                let base_url =
                    env::var("OPENAI_BASE_URL").unwrap_or_else(|_| "https://api.openai.com".to_string());
                let model =
                    env::var("OPENAI_EMBED_MODEL").unwrap_or_else(|_| "text-embedding-3-small".to_string());
                Ok(Self {
                    provider,
                    api_key,
                    base_url,
                    model,
                    http: Client::new(),
                })
            }
            EmbeddingProvider::Gemini => {
                let api_key = env::var("GEMINI_API_KEY").context("GEMINI_API_KEY is not set")?;
                let base_url = env::var("GEMINI_BASE_URL")
                    .unwrap_or_else(|_| "https://generativelanguage.googleapis.com".to_string());
                let model =
                    env::var("GEMINI_EMBED_MODEL").unwrap_or_else(|_| "gemini-embedding-001".to_string());
                Ok(Self {
                    provider,
                    api_key,
                    base_url,
                    model,
                    http: Client::new(),
                })
            }
        }
    }

    pub fn provider(&self) -> EmbeddingProvider {
        self.provider
    }

    pub fn model(&self) -> &str {
        &self.model
    }

    pub fn embed_text(&self, text: &str) -> Result<Vec<f32>> {
        match self.provider {
            EmbeddingProvider::OpenAI => self.embed_openai(text),
            EmbeddingProvider::Gemini => self.embed_gemini(text),
        }
    }

    fn embed_openai(&self, text: &str) -> Result<Vec<f32>> {
        let url = format!("{}/v1/embeddings", self.base_url.trim_end_matches('/'));
        let body = json!({
            "model": self.model,
            "input": text,
            "encoding_format": "float"
        });
        let resp: Value = self
            .http
            .post(url)
            .bearer_auth(&self.api_key)
            .json(&body)
            .send()
            .context("send embeddings request")?
            .error_for_status()
            .context("embeddings status")?
            .json()
            .context("parse embeddings response")?;
        parse_openai_embedding(&resp)
    }

    fn embed_gemini(&self, text: &str) -> Result<Vec<f32>> {
        let url = format!(
            "{}/v1beta/models/{}:embedContent",
            self.base_url.trim_end_matches('/'),
            self.model
        );
        let body = json!({
            "content": {
                "parts": [
                    {"text": text}
                ]
            }
        });
        let resp: Value = self
            .http
            .post(url)
            .header("x-goog-api-key", &self.api_key)
            .json(&body)
            .send()
            .context("send gemini embeddings request")?
            .error_for_status()
            .context("gemini embeddings status")?
            .json()
            .context("parse gemini embeddings response")?;
        parse_gemini_embedding(&resp)
    }
}

fn parse_openai_embedding(resp: &Value) -> Result<Vec<f32>> {
    let embedding = resp
        .get("data")
        .and_then(|val| val.get(0))
        .and_then(|val| val.get("embedding"))
        .ok_or_else(|| anyhow!("missing embedding in response"))?;
    parse_embedding_array(embedding)
}

fn parse_gemini_embedding(resp: &Value) -> Result<Vec<f32>> {
    let embedding = resp
        .get("embedding")
        .and_then(|val| val.get("values"))
        .ok_or_else(|| anyhow!("missing gemini embedding values"))?;
    parse_embedding_array(embedding)
}

fn parse_embedding_array(value: &Value) -> Result<Vec<f32>> {
    match value {
        Value::Array(items) => items
            .iter()
            .map(|v| v.as_f64().map(|f| f as f32).ok_or_else(|| anyhow!("invalid embedding value")))
            .collect(),
        _ => Err(anyhow!("embedding is not an array")),
    }
}
