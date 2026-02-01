use anyhow::{Context, Result};
use reqwest::blocking::Client;
use serde_json::{json, Value};
use std::time::Duration;

use crate::engine::{ChatResponse, Engine, ToolCall};

pub struct GeminiChatClient {
    api_key: String,
    base_url: String,
    http: Client,
    model: String,
}

impl GeminiChatClient {
    pub fn new(api_key: String, base_url: String, model: String) -> Self {
        let http = Client::builder()
            .timeout(Duration::from_secs(300))
            .build()
            .expect("build http client");
        Self {
            api_key,
            base_url,
            http,
            model,
        }
    }
}

impl Engine for GeminiChatClient {
    fn chat(&self, messages: &[Value], tools: &[Value]) -> Result<ChatResponse> {
        let url = format!(
            "{}/v1beta/models/{}:generateContent?key={}",
            self.base_url.trim_end_matches('/'),
            self.model,
            self.api_key,
        );

        // Separate system messages from conversation
        let mut system_parts: Vec<String> = Vec::new();
        let mut contents: Vec<Value> = Vec::new();

        for msg in messages {
            let role = msg.get("role").and_then(|v| v.as_str()).unwrap_or("");
            let content = msg.get("content").and_then(|v| v.as_str()).unwrap_or("");

            match role {
                "system" => {
                    system_parts.push(content.to_string());
                }
                "user" => {
                    contents.push(json!({
                        "role": "user",
                        "parts": [{"text": content}],
                    }));
                }
                "assistant" => {
                    // Check for tool_calls (OpenAI format in history)
                    if let Some(tool_calls) = msg.get("tool_calls").and_then(|v| v.as_array()) {
                        let mut parts: Vec<Value> = Vec::new();
                        if !content.is_empty() {
                            parts.push(json!({"text": content}));
                        }
                        for tc in tool_calls {
                            let func = tc.get("function").unwrap_or(&Value::Null);
                            let name =
                                func.get("name").and_then(|v| v.as_str()).unwrap_or("");
                            let args_str =
                                func.get("arguments").and_then(|v| v.as_str()).unwrap_or("{}");
                            let args: Value =
                                serde_json::from_str(args_str).unwrap_or(json!({}));
                            parts.push(json!({
                                "functionCall": {
                                    "name": name,
                                    "args": args,
                                }
                            }));
                        }
                        contents.push(json!({
                            "role": "model",
                            "parts": parts,
                        }));
                    } else {
                        contents.push(json!({
                            "role": "model",
                            "parts": [{"text": content}],
                        }));
                    }
                }
                "tool" => {
                    // Tool results become user messages with functionResponse parts
                    let tool_call_id = msg
                        .get("tool_call_id")
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown");
                    // Try to parse content as JSON for structured response
                    let response_value: Value =
                        serde_json::from_str(content).unwrap_or(json!({"result": content}));
                    contents.push(json!({
                        "role": "user",
                        "parts": [{
                            "functionResponse": {
                                "name": tool_call_id,
                                "response": response_value,
                            }
                        }],
                    }));
                }
                _ => {}
            }
        }

        // Merge consecutive same-role messages (Gemini requires alternating user/model)
        let merged = merge_consecutive_parts(contents);

        // Convert OpenAI tool schemas to Gemini functionDeclarations
        let declarations: Vec<Value> = tools.iter().filter_map(convert_to_declaration).collect();

        let mut body = json!({ "contents": merged });

        if !system_parts.is_empty() {
            body["systemInstruction"] = json!({
                "parts": [{"text": system_parts.join("\n\n")}]
            });
        }
        if !declarations.is_empty() {
            body["tools"] = json!([{ "functionDeclarations": declarations }]);
        }

        let resp: Value = self
            .http
            .post(&url)
            .header("content-type", "application/json")
            .json(&body)
            .send()
            .context("send gemini generateContent")?
            .error_for_status()
            .context("gemini generateContent status")?
            .json()
            .context("parse gemini generateContent response")?;

        // Parse response: candidates[0].content.parts[]
        let parts = resp
            .get("candidates")
            .and_then(|c| c.get(0))
            .and_then(|c| c.get("content"))
            .and_then(|c| c.get("parts"))
            .and_then(|p| p.as_array())
            .cloned()
            .unwrap_or_default();

        let mut text_parts: Vec<String> = Vec::new();
        let mut tool_calls: Vec<ToolCall> = Vec::new();

        for part in &parts {
            if let Some(text) = part.get("text").and_then(|v| v.as_str()) {
                text_parts.push(text.to_string());
            }
            if let Some(fc) = part.get("functionCall") {
                let name = fc
                    .get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let args = fc.get("args").cloned().unwrap_or(json!({}));
                // Gemini doesn't provide tool call IDs; generate one
                let id = format!("gemini_{}", ulid::Ulid::new());
                tool_calls.push(ToolCall {
                    id,
                    name,
                    arguments: args,
                });
            }
        }

        let content = if text_parts.is_empty() {
            None
        } else {
            Some(text_parts.join(""))
        };

        Ok(ChatResponse {
            content,
            tool_calls,
        })
    }

    fn set_model(&mut self, model: String) {
        self.model = model;
    }

    fn model(&self) -> &str {
        &self.model
    }
}

/// Convert an OpenAI tool schema to a Gemini functionDeclaration.
fn convert_to_declaration(tool: &Value) -> Option<Value> {
    let func = tool.get("function")?;
    let name = func.get("name")?;
    let description = func.get("description").cloned().unwrap_or(Value::Null);
    let parameters = func.get("parameters").cloned().unwrap_or(json!({"type": "object"}));

    Some(json!({
        "name": name,
        "description": description,
        "parameters": parameters,
    }))
}

/// Merge consecutive messages with the same role.
fn merge_consecutive_parts(messages: Vec<Value>) -> Vec<Value> {
    let mut merged: Vec<Value> = Vec::new();

    for msg in messages {
        let role = msg.get("role").and_then(|v| v.as_str()).unwrap_or("");
        let prev_role = merged
            .last()
            .and_then(|m| m.get("role"))
            .and_then(|v| v.as_str())
            .unwrap_or("");

        if role == prev_role && !merged.is_empty() {
            let prev = merged.last_mut().unwrap();
            let mut prev_parts = prev
                .get("parts")
                .and_then(|v| v.as_array())
                .cloned()
                .unwrap_or_default();
            if let Some(new_parts) = msg.get("parts").and_then(|v| v.as_array()) {
                prev_parts.extend(new_parts.clone());
            }
            prev["parts"] = Value::Array(prev_parts);
        } else {
            merged.push(msg);
        }
    }

    merged
}
