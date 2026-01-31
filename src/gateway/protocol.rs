use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Client-to-server request frame.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Request {
    /// Always "req"
    #[serde(rename = "type")]
    pub frame_type: String,
    /// Unique request ID (UUID)
    pub id: String,
    /// Method name, e.g. "session.open"
    pub method: String,
    /// Method parameters
    #[serde(default)]
    pub params: Value,
}

/// Server-to-client response frame.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    /// Always "res"
    #[serde(rename = "type")]
    pub frame_type: String,
    /// Matches the request ID
    pub id: String,
    /// Whether the request succeeded
    pub ok: bool,
    /// Success payload (when ok=true)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payload: Option<Value>,
    /// Error details (when ok=false)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ErrorPayload>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorPayload {
    pub code: String,
    pub message: String,
}

/// Server-to-client event frame (push).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    /// Always "event"
    #[serde(rename = "type")]
    pub frame_type: String,
    /// Event name, e.g. "delta", "final", "error"
    pub event: String,
    /// Which session this event belongs to
    pub session_id: String,
    /// Event-specific payload
    pub payload: Value,
}

/// Inbound frame: either a request or unknown.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum InboundFrame {
    #[serde(rename = "req")]
    Request {
        id: String,
        method: String,
        #[serde(default)]
        params: Value,
    },
}

impl Response {
    pub fn ok(id: impl Into<String>, payload: Value) -> Self {
        Self {
            frame_type: "res".into(),
            id: id.into(),
            ok: true,
            payload: Some(payload),
            error: None,
        }
    }

    pub fn err(id: impl Into<String>, code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            frame_type: "res".into(),
            id: id.into(),
            ok: false,
            payload: None,
            error: Some(ErrorPayload {
                code: code.into(),
                message: message.into(),
            }),
        }
    }
}

impl Event {
    pub fn new(event: impl Into<String>, session_id: impl Into<String>, payload: Value) -> Self {
        Self {
            frame_type: "event".into(),
            event: event.into(),
            session_id: session_id.into(),
            payload,
        }
    }
}
