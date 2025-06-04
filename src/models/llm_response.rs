//! LLM response types

use crate::types::Content;
use serde::{Deserialize, Serialize};

/// Response from an LLM
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmResponse {
    /// Content of the response
    pub content: Option<Content>,
}

impl LlmResponse {
    /// Create a text response
    pub fn text(text: impl Into<String>) -> Self {
        Self {
            content: Some(Content::model_text(text)),
        }
    }

    /// Get text content
    pub fn get_text(&self) -> Option<String> {
        self.content.as_ref().map(|c| c.get_text())
    }
}
