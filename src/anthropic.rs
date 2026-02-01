use anyhow::{Context, Result};
use reqwest::blocking::Client;
use serde_json::{json, Value};
use std::time::Duration;

use crate::engine::{ChatResponse, Engine, ToolCall};

pub struct AnthropicClient {
    api_key: String,
    base_url: String,
    http: Client,
    model: String,
    max_tokens: usize,
}

impl AnthropicClient {
    pub fn new(api_key: String, base_url: String, model: String, max_tokens: usize) -> Self {
        let http = Client::builder()
            .timeout(Duration::from_secs(300))
            .build()
            .expect("build http client");
        Self {
            api_key,
            base_url,
            http,
            model,
            max_tokens,
        }
    }
}

impl Engine for AnthropicClient {
    fn chat(&self, messages: &[Value], tools: &[Value]) -> Result<ChatResponse> {
        let url = format!("{}/v1/messages", self.base_url.trim_end_matches('/'));

        // Separate system messages from conversation messages.
        // OpenAI format puts system as role:"system" in messages array.
        // Anthropic wants a top-level "system" string parameter.
        let mut system_parts: Vec<String> = Vec::new();
        let mut converted_messages: Vec<Value> = Vec::new();

        for msg in messages {
            let role = msg.get("role").and_then(|v| v.as_str()).unwrap_or("");
            let content = msg.get("content").and_then(|v| v.as_str()).unwrap_or("");

            match role {
                "system" => {
                    system_parts.push(content.to_string());
                }
                "user" => {
                    converted_messages.push(json!({
                        "role": "user",
                        "content": content,
                    }));
                }
                "assistant" => {
                    // Check if this is an assistant message with tool_calls
                    if let Some(tool_calls) = msg.get("tool_calls").and_then(|v| v.as_array()) {
                        let mut content_blocks: Vec<Value> = Vec::new();
                        // Include text content if present
                        if !content.is_empty() {
                            content_blocks.push(json!({ "type": "text", "text": content }));
                        }
                        // Convert tool_calls to tool_use content blocks
                        for tc in tool_calls {
                            let id = tc.get("id").and_then(|v| v.as_str()).unwrap_or("");
                            let func = tc.get("function").unwrap_or(&Value::Null);
                            let name = func.get("name").and_then(|v| v.as_str()).unwrap_or("");
                            let args_str =
                                func.get("arguments").and_then(|v| v.as_str()).unwrap_or("{}");
                            let input: Value =
                                serde_json::from_str(args_str).unwrap_or(json!({}));
                            content_blocks.push(json!({
                                "type": "tool_use",
                                "id": id,
                                "name": name,
                                "input": input,
                            }));
                        }
                        converted_messages.push(json!({
                            "role": "assistant",
                            "content": content_blocks,
                        }));
                    } else {
                        converted_messages.push(json!({
                            "role": "assistant",
                            "content": content,
                        }));
                    }
                }
                "tool" => {
                    // OpenAI tool results: {"role":"tool","tool_call_id":"...","content":"..."}
                    // Anthropic: user message with tool_result content block
                    let tool_call_id = msg
                        .get("tool_call_id")
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    converted_messages.push(json!({
                        "role": "user",
                        "content": [{
                            "type": "tool_result",
                            "tool_use_id": tool_call_id,
                            "content": content,
                        }],
                    }));
                }
                _ => {
                    // Skip unknown roles
                }
            }
        }

        // Merge consecutive same-role messages (Anthropic requires alternating roles)
        let merged = merge_consecutive_roles(converted_messages);

        // Convert OpenAI tool schemas to Anthropic format
        let anthropic_tools: Vec<Value> = tools.iter().filter_map(convert_tool_schema).collect();

        let mut body = json!({
            "model": self.model,
            "max_tokens": self.max_tokens,
            "messages": merged,
        });

        if !system_parts.is_empty() {
            body["system"] = Value::String(system_parts.join("\n\n"));
        }
        if !anthropic_tools.is_empty() {
            body["tools"] = Value::Array(anthropic_tools);
        }

        let resp: Value = self
            .http
            .post(url)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&body)
            .send()
            .context("send anthropic messages")?
            .error_for_status()
            .context("anthropic messages status")?
            .json()
            .context("parse anthropic messages response")?;

        // Parse response: content is an array of blocks
        let content_blocks = resp
            .get("content")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();

        let mut text_parts: Vec<String> = Vec::new();
        let mut tool_calls: Vec<ToolCall> = Vec::new();

        for block in &content_blocks {
            let block_type = block.get("type").and_then(|v| v.as_str()).unwrap_or("");
            match block_type {
                "text" => {
                    if let Some(text) = block.get("text").and_then(|v| v.as_str()) {
                        text_parts.push(text.to_string());
                    }
                }
                "tool_use" => {
                    let id = block
                        .get("id")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    let name = block
                        .get("name")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    let input = block.get("input").cloned().unwrap_or(json!({}));
                    tool_calls.push(ToolCall {
                        id,
                        name,
                        arguments: input,
                    });
                }
                _ => {}
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

/// Convert an OpenAI-format tool schema to Anthropic format.
/// OpenAI: {"type":"function","function":{"name":"...","description":"...","parameters":{...}}}
/// Anthropic: {"name":"...","description":"...","input_schema":{...}}
fn convert_tool_schema(tool: &Value) -> Option<Value> {
    let func = tool.get("function")?;
    let name = func.get("name")?;
    let description = func.get("description").cloned().unwrap_or(Value::Null);
    let parameters = func.get("parameters").cloned().unwrap_or(json!({"type": "object"}));

    Some(json!({
        "name": name,
        "description": description,
        "input_schema": parameters,
    }))
}

/// Merge consecutive messages with the same role.
/// Anthropic requires strictly alternating user/assistant messages.
fn merge_consecutive_roles(messages: Vec<Value>) -> Vec<Value> {
    let mut merged: Vec<Value> = Vec::new();

    for msg in messages {
        let role = msg.get("role").and_then(|v| v.as_str()).unwrap_or("");
        let prev_role = merged
            .last()
            .and_then(|m| m.get("role"))
            .and_then(|v| v.as_str())
            .unwrap_or("");

        if role == prev_role && !merged.is_empty() {
            // Merge into previous message
            let prev = merged.last_mut().unwrap();
            let prev_content = prev.get("content").cloned().unwrap_or(Value::Null);
            let new_content = msg.get("content").cloned().unwrap_or(Value::Null);

            // Convert both to arrays of content blocks, then concatenate
            let mut blocks = to_content_blocks(prev_content);
            blocks.extend(to_content_blocks(new_content));
            prev["content"] = Value::Array(blocks);
        } else {
            merged.push(msg);
        }
    }

    merged
}

/// Convert a content value to an array of content blocks.
fn to_content_blocks(content: Value) -> Vec<Value> {
    match content {
        Value::Array(blocks) => blocks,
        Value::String(text) => vec![json!({"type": "text", "text": text})],
        Value::Null => vec![],
        other => vec![json!({"type": "text", "text": other.to_string()})],
    }
}
