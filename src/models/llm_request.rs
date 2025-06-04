//! LLM request types

use crate::types::Content;
use serde::{Deserialize, Serialize};

/// Request to send to an LLM
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmRequest {
    /// Model name to use
    pub model: String,
    
    /// Content to send to the model
    pub contents: Vec<Content>,
}

impl LlmRequest {
    /// Create a new LLM request
    pub fn new(model: impl Into<String>) -> Self {
        Self {
            model: model.into(),
            contents: Vec::new(),
        }
    }
}

/// Builder for LLM requests
pub struct LlmRequestBuilder {
    request: LlmRequest,
}

impl LlmRequestBuilder {
    pub fn new(model: impl Into<String>) -> Self {
        Self {
            request: LlmRequest::new(model),
        }
    }

    pub fn user_message(mut self, text: impl Into<String>) -> Self {
        self.request.contents.push(Content::user_text(text));
        self
    }

    pub fn build(self) -> crate::error::Result<LlmRequest> {
        Ok(self.request)
    }
}
