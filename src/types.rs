//! Common types used throughout the ADK library

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Unique identifier for agents
pub type AgentId = String;

/// Unique identifier for sessions
pub type SessionId = String;

/// Unique identifier for users
pub type UserId = String;

/// Unique identifier for invocations
pub type InvocationId = Uuid;

/// Content part that can contain text, images, or other media
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentPart {
    Text { text: String },
    Image { data: Vec<u8>, mime_type: String },
    Video { data: Vec<u8>, mime_type: String },
    Audio { data: Vec<u8>, mime_type: String },
    File { data: Vec<u8>, mime_type: String, filename: String },
}

impl ContentPart {
    /// Create a text content part
    pub fn text(text: impl Into<String>) -> Self {
        Self::Text { text: text.into() }
    }

    /// Create an image content part
    pub fn image(data: Vec<u8>, mime_type: impl Into<String>) -> Self {
        Self::Image {
            data,
            mime_type: mime_type.into(),
        }
    }

    /// Get text content if this is a text part
    pub fn as_text(&self) -> Option<&str> {
        match self {
            Self::Text { text } => Some(text),
            _ => None,
        }
    }
}

/// Content with role and parts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Content {
    pub role: String,
    pub parts: Vec<ContentPart>,
}

impl Content {
    /// Create user content with text
    pub fn user_text(text: impl Into<String>) -> Self {
        Self {
            role: "user".to_string(),
            parts: vec![ContentPart::text(text)],
        }
    }

    /// Create model content with text
    pub fn model_text(text: impl Into<String>) -> Self {
        Self {
            role: "model".to_string(),
            parts: vec![ContentPart::text(text)],
        }
    }

    /// Get all text from content parts
    pub fn get_text(&self) -> String {
        self.parts
            .iter()
            .filter_map(|part| part.as_text())
            .collect::<Vec<_>>()
            .join("")
    }
}

/// Function declaration for tools
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionDeclaration {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

/// Tool definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    pub function_declarations: Vec<FunctionDeclaration>,
}

/// Function call from the model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionCall {
    pub name: String,
    pub args: serde_json::Value,
}

/// Configuration for content generation
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GenerateContentConfig {
    pub tools: Vec<Tool>,
    pub response_schema: Option<serde_json::Value>,
    pub response_mime_type: Option<String>,
    pub temperature: Option<f32>,
    pub top_p: Option<f32>,
    pub top_k: Option<i32>,
    pub max_output_tokens: Option<i32>,
    pub stop_sequences: Vec<String>,
}

/// State delta for session updates
pub type StateDelta = HashMap<String, serde_json::Value>;

/// Session state
pub type SessionState = HashMap<String, serde_json::Value>;

/// Streaming mode for agent responses
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum StreamingMode {
    #[default]
    Off,
    On,
    OnWithToolCalls,
}

/// Run configuration for agents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunConfig {
    pub streaming_mode: StreamingMode,
    pub max_iterations: Option<u32>,
    pub timeout_seconds: Option<u64>,
}

impl Default for RunConfig {
    fn default() -> Self {
        Self {
            streaming_mode: StreamingMode::default(),
            max_iterations: None,
            timeout_seconds: None,
        }
    }
}

/// Metadata for various objects
pub type Metadata = HashMap<String, serde_json::Value>;

/// Timestamp type
pub type Timestamp = DateTime<Utc>;

/// Helper function to get current timestamp
pub fn now() -> Timestamp {
    Utc::now()
}

/// Blob data for streaming
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Blob {
    pub mime_type: String,
    pub data: Vec<u8>,
}

impl Blob {
    pub fn new(mime_type: impl Into<String>, data: Vec<u8>) -> Self {
        Self {
            mime_type: mime_type.into(),
            data,
        }
    }
}
