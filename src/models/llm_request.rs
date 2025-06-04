//! LLM request types and builders

use crate::{
    tools::BaseTool,
    types::{Content, GenerateContentConfig, Tool},
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};

/// Request to send to an LLM
#[derive(Clone, Serialize, Deserialize)]
pub struct LlmRequest {
    /// Model name to use
    pub model: String,

    /// Content to send to the model
    pub contents: Vec<Content>,

    /// Configuration for content generation
    pub config: GenerateContentConfig,

    /// Tools available to the model (not serialized)
    #[serde(skip)]
    pub tools_dict: HashMap<String, Arc<dyn BaseTool>>,
}

impl std::fmt::Debug for LlmRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LlmRequest")
            .field("model", &self.model)
            .field("contents", &self.contents)
            .field("config", &self.config)
            .field("tools_count", &self.tools_dict.len())
            .finish()
    }
}

impl LlmRequest {
    /// Create a new LLM request
    pub fn new(model: impl Into<String>) -> Self {
        Self {
            model: model.into(),
            contents: Vec::new(),
            config: GenerateContentConfig::default(),
            tools_dict: HashMap::new(),
        }
    }

    /// Add content to the request
    pub fn add_content(mut self, content: Content) -> Self {
        self.contents.push(content);
        self
    }

    /// Add multiple contents to the request
    pub fn add_contents(mut self, contents: Vec<Content>) -> Self {
        self.contents.extend(contents);
        self
    }

    /// Add a user message
    pub fn add_user_message(mut self, text: impl Into<String>) -> Self {
        self.contents.push(Content::user_text(text));
        self
    }

    /// Add a model message
    pub fn add_model_message(mut self, text: impl Into<String>) -> Self {
        self.contents.push(Content::model_text(text));
        self
    }

    /// Set the generation config
    pub fn with_config(mut self, config: GenerateContentConfig) -> Self {
        self.config = config;
        self
    }

    /// Set temperature
    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.config.temperature = Some(temperature);
        self
    }

    /// Set max output tokens
    pub fn with_max_tokens(mut self, max_tokens: i32) -> Self {
        self.config.max_output_tokens = Some(max_tokens);
        self
    }

    /// Set top-p
    pub fn with_top_p(mut self, top_p: f32) -> Self {
        self.config.top_p = Some(top_p);
        self
    }

    /// Set top-k
    pub fn with_top_k(mut self, top_k: i32) -> Self {
        self.config.top_k = Some(top_k);
        self
    }

    /// Add stop sequences
    pub fn with_stop_sequences(mut self, sequences: Vec<String>) -> Self {
        self.config.stop_sequences = sequences;
        self
    }

    /// Set response schema for structured output
    pub fn with_response_schema(mut self, schema: serde_json::Value) -> Self {
        self.config.response_schema = Some(schema);
        self.config.response_mime_type = Some("application/json".to_string());
        self
    }

    /// Add tools to the request
    pub fn add_tools(mut self, tools: Vec<Arc<dyn BaseTool>>) -> Self {
        if tools.is_empty() {
            return self;
        }

        let mut declarations = Vec::new();
        for tool in tools {
            if let Some(declaration) = tool.get_declaration() {
                declarations.push(declaration);
                self.tools_dict.insert(tool.name().to_string(), tool);
            }
        }

        if !declarations.is_empty() {
            self.config.tools.push(Tool { function_declarations: declarations });
        }

        self
    }

    /// Add a single tool to the request
    pub fn add_tool(mut self, tool: Arc<dyn BaseTool>) -> Self {
        if let Some(declaration) = tool.get_declaration() {
            // Find existing tool or create new one
            if let Some(existing_tool) = self.config.tools.last_mut() {
                existing_tool.function_declarations.push(declaration);
            } else {
                self.config.tools.push(Tool {
                    function_declarations: vec![declaration],
                });
            }
            self.tools_dict.insert(tool.name().to_string(), tool);
        }
        self
    }

    /// Get a tool by name
    pub fn get_tool(&self, name: &str) -> Option<&Arc<dyn BaseTool>> {
        self.tools_dict.get(name)
    }

    /// Check if the request has any tools
    pub fn has_tools(&self) -> bool {
        !self.config.tools.is_empty()
    }

    /// Get the last user message
    pub fn last_user_message(&self) -> Option<&Content> {
        self.contents
            .iter()
            .rev()
            .find(|content| content.role == "user")
    }

    /// Get the last model message
    pub fn last_model_message(&self) -> Option<&Content> {
        self.contents
            .iter()
            .rev()
            .find(|content| content.role == "model")
    }

    /// Clear all contents
    pub fn clear_contents(mut self) -> Self {
        self.contents.clear();
        self
    }

    /// Clear all tools
    pub fn clear_tools(mut self) -> Self {
        self.config.tools.clear();
        self.tools_dict.clear();
        self
    }

    /// Validate the request
    pub fn validate(&self) -> crate::error::Result<()> {
        if self.model.is_empty() {
            return Err(crate::adk_error!(ValidationError, "Model name is required"));
        }

        if self.contents.is_empty() {
            return Err(crate::adk_error!(ValidationError, "At least one content is required"));
        }

        // Validate temperature range
        if let Some(temp) = self.config.temperature {
            if !(0.0..=2.0).contains(&temp) {
                return Err(crate::adk_error!(
                    ValidationError,
                    "Temperature must be between 0.0 and 2.0"
                ));
            }
        }

        // Validate top_p range
        if let Some(top_p) = self.config.top_p {
            if !(0.0..=1.0).contains(&top_p) {
                return Err(crate::adk_error!(
                    ValidationError,
                    "Top-p must be between 0.0 and 1.0"
                ));
            }
        }

        // Validate max_output_tokens
        if let Some(max_tokens) = self.config.max_output_tokens {
            if max_tokens <= 0 {
                return Err(crate::adk_error!(
                    ValidationError,
                    "Max output tokens must be positive"
                ));
            }
        }

        Ok(())
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

    pub fn content(mut self, content: Content) -> Self {
        self.request = self.request.add_content(content);
        self
    }

    pub fn user_message(mut self, text: impl Into<String>) -> Self {
        self.request = self.request.add_user_message(text);
        self
    }

    pub fn model_message(mut self, text: impl Into<String>) -> Self {
        self.request = self.request.add_model_message(text);
        self
    }

    pub fn temperature(mut self, temperature: f32) -> Self {
        self.request = self.request.with_temperature(temperature);
        self
    }

    pub fn max_tokens(mut self, max_tokens: i32) -> Self {
        self.request = self.request.with_max_tokens(max_tokens);
        self
    }

    pub fn tools(mut self, tools: Vec<Arc<dyn BaseTool>>) -> Self {
        self.request = self.request.add_tools(tools);
        self
    }

    pub fn build(self) -> crate::error::Result<LlmRequest> {
        self.request.validate()?;
        Ok(self.request)
    }
}
