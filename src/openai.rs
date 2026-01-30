use anyhow::{anyhow, Context, Result};
use reqwest::blocking::Client;
use serde_json::{json, Value};

#[derive(Debug, Clone)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub arguments: Value,
}

#[derive(Debug, Clone)]
pub struct ChatResponse {
    pub content: Option<String>,
    pub tool_calls: Vec<ToolCall>,
}

pub struct OpenAIClient {
    api_key: String,
    base_url: String,
    http: Client,
    model: String,
}

impl OpenAIClient {
    pub fn new(api_key: String, base_url: String, model: String) -> Self {
        Self {
            api_key,
            base_url,
            http: Client::new(),
            model,
        }
    }

    pub fn set_model(&mut self, model: String) {
        self.model = model;
    }

    pub fn chat(&self, messages: &[Value], tools: &[Value]) -> Result<ChatResponse> {
        let url = format!("{}/v1/chat/completions", self.base_url.trim_end_matches('/'));
        let body = json!({
            "model": self.model,
            "messages": messages,
            "tools": tools,
            "tool_choice": "auto"
        });

        let resp: Value = self
            .http
            .post(url)
            .bearer_auth(&self.api_key)
            .json(&body)
            .send()
            .context("send chat completion")?
            .error_for_status()
            .context("chat completion status")?
            .json()
            .context("parse chat completion response")?;

        let message = resp
            .get("choices")
            .and_then(|choices| choices.get(0))
            .and_then(|choice| choice.get("message"))
            .ok_or_else(|| anyhow!("missing message in response"))?;

        let content = message
            .get("content")
            .and_then(|val| val.as_str())
            .map(|s| s.to_string());

        let tool_calls = parse_tool_calls(message)?;

        Ok(ChatResponse { content, tool_calls })
    }
}

fn parse_tool_calls(message: &Value) -> Result<Vec<ToolCall>> {
    let mut calls = Vec::new();
    let list = match message.get("tool_calls") {
        Some(Value::Array(items)) => items,
        _ => return Ok(calls),
    };
    for item in list {
        let id = item
            .get("id")
            .and_then(|val| val.as_str())
            .ok_or_else(|| anyhow!("tool call missing id"))?;
        let function = item
            .get("function")
            .ok_or_else(|| anyhow!("tool call missing function"))?;
        let name = function
            .get("name")
            .and_then(|val| val.as_str())
            .ok_or_else(|| anyhow!("tool call missing name"))?;
        let args_str = function
            .get("arguments")
            .and_then(|val| val.as_str())
            .unwrap_or("{}");
        let arguments: Value =
            serde_json::from_str(args_str).context("parse tool arguments as JSON")?;
        calls.push(ToolCall {
            id: id.to_string(),
            name: name.to_string(),
            arguments,
        });
    }
    Ok(calls)
}
