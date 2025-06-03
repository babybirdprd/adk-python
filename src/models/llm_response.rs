//! LLM response types

use crate::types::{Content, FunctionCall};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Response from an LLM
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmResponse {
    /// Content of the response
    pub content: Option<Content>,

    /// Function calls requested by the model
    pub function_calls: Vec<FunctionCall>,

    /// Whether this is a partial response (streaming)
    pub is_partial: bool,

    /// Finish reason for the response
    pub finish_reason: Option<FinishReason>,

    /// Usage statistics
    pub usage: Option<Usage>,

    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Reason why the model finished generating
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum FinishReason {
    /// Natural stop
    Stop,
    /// Maximum tokens reached
    MaxTokens,
    /// Safety filter triggered
    Safety,
    /// Recitation filter triggered
    Recitation,
    /// Function call requested
    FunctionCall,
    /// Other reason
    Other,
}

/// Token usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Usage {
    /// Number of tokens in the prompt
    pub prompt_tokens: Option<u32>,

    /// Number of tokens in the response
    pub completion_tokens: Option<u32>,

    /// Total number of tokens
    pub total_tokens: Option<u32>,
}

impl LlmResponse {
    /// Create a new response
    pub fn new() -> Self {
        Self {
            content: None,
            function_calls: Vec::new(),
            is_partial: false,
            finish_reason: None,
            usage: None,
            metadata: HashMap::new(),
        }
    }

    /// Create a text response
    pub fn text(text: impl Into<String>) -> Self {
        Self {
            content: Some(Content::model_text(text)),
            function_calls: Vec::new(),
            is_partial: false,
            finish_reason: Some(FinishReason::Stop),
            usage: None,
            metadata: HashMap::new(),
        }
    }

    /// Create a function call response
    pub fn function_call(name: impl Into<String>, args: serde_json::Value) -> Self {
        Self {
            content: None,
            function_calls: vec![FunctionCall {
                name: name.into(),
                args,
            }],
            is_partial: false,
            finish_reason: Some(FinishReason::FunctionCall),
            usage: None,
            metadata: HashMap::new(),
        }
    }

    /// Create a partial response for streaming
    pub fn partial_text(text: impl Into<String>) -> Self {
        Self {
            content: Some(Content::model_text(text)),
            function_calls: Vec::new(),
            is_partial: true,
            finish_reason: None,
            usage: None,
            metadata: HashMap::new(),
        }
    }

    /// Set content
    pub fn with_content(mut self, content: Content) -> Self {
        self.content = Some(content);
        self
    }

    /// Add function call
    pub fn with_function_call(mut self, function_call: FunctionCall) -> Self {
        self.function_calls.push(function_call);
        self
    }

    /// Set as partial
    pub fn as_partial(mut self) -> Self {
        self.is_partial = true;
        self
    }

    /// Set finish reason
    pub fn with_finish_reason(mut self, reason: FinishReason) -> Self {
        self.finish_reason = Some(reason);
        self
    }

    /// Set usage statistics
    pub fn with_usage(mut self, usage: Usage) -> Self {
        self.usage = Some(usage);
        self
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.metadata.insert(key.into(), value);
        self
    }

    /// Get text content
    pub fn get_text(&self) -> Option<String> {
        self.content.as_ref().map(|c| c.get_text())
    }

    /// Check if response has function calls
    pub fn has_function_calls(&self) -> bool {
        !self.function_calls.is_empty()
    }

    /// Check if response is complete (not partial)
    pub fn is_complete(&self) -> bool {
        !self.is_partial
    }

    /// Check if response was stopped due to safety
    pub fn is_safety_filtered(&self) -> bool {
        matches!(self.finish_reason, Some(FinishReason::Safety))
    }

    /// Check if response was stopped due to max tokens
    pub fn is_max_tokens(&self) -> bool {
        matches!(self.finish_reason, Some(FinishReason::MaxTokens))
    }

    /// Merge with another response (for streaming)
    pub fn merge(mut self, other: LlmResponse) -> Self {
        // Merge content
        if let Some(other_content) = other.content {
            if let Some(ref mut content) = self.content {
                // Merge text parts
                let self_text = content.get_text();
                let other_text = other_content.get_text();
                *content = Content::model_text(format!("{}{}", self_text, other_text));
            } else {
                self.content = Some(other_content);
            }
        }

        // Merge function calls
        self.function_calls.extend(other.function_calls);

        // Update finish reason if other has one
        if other.finish_reason.is_some() {
            self.finish_reason = other.finish_reason;
        }

        // Update usage if other has it
        if other.usage.is_some() {
            self.usage = other.usage;
        }

        // Merge metadata
        self.metadata.extend(other.metadata);

        // Update partial status
        self.is_partial = other.is_partial;

        self
    }
}

impl Default for LlmResponse {
    fn default() -> Self {
        Self::new()
    }
}

impl Usage {
    pub fn new() -> Self {
        Self {
            prompt_tokens: None,
            completion_tokens: None,
            total_tokens: None,
        }
    }

    pub fn with_prompt_tokens(mut self, tokens: u32) -> Self {
        self.prompt_tokens = Some(tokens);
        self.total_tokens = Some(
            tokens + self.completion_tokens.unwrap_or(0)
        );
        self
    }

    pub fn with_completion_tokens(mut self, tokens: u32) -> Self {
        self.completion_tokens = Some(tokens);
        self.total_tokens = Some(
            self.prompt_tokens.unwrap_or(0) + tokens
        );
        self
    }

    pub fn with_total_tokens(mut self, tokens: u32) -> Self {
        self.total_tokens = Some(tokens);
        self
    }
}

impl Default for Usage {
    fn default() -> Self {
        Self::new()
    }
}
