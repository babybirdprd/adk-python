//! Base LLM trait and connection management

use crate::{
    error::Result,
    types::{Blob, Content},
};
use async_trait::async_trait;
use futures::Stream;
use serde::{Deserialize, Serialize};
use std::pin::Pin;

use super::{LlmRequest, LlmResponse};

/// Base trait for all LLM implementations
#[async_trait]
pub trait BaseLlm: Send + Sync {
    /// Get the model name
    fn model_name(&self) -> &str;

    /// Get supported model patterns (regex)
    fn supported_models() -> Vec<String>
    where
        Self: Sized;

    /// Generate content from a request
    async fn generate_content(&self, request: LlmRequest) -> Result<LlmResponse>;

    /// Generate streaming content from a request
    async fn generate_content_stream(
        &self,
        request: LlmRequest,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<LlmResponse>> + Send>>>;

    /// Check if the model supports streaming
    fn supports_streaming(&self) -> bool {
        true
    }

    /// Check if the model supports function calling
    fn supports_function_calling(&self) -> bool {
        true
    }

    /// Check if the model supports multimodal input
    fn supports_multimodal(&self) -> bool {
        false
    }

    /// Check if the model supports live (realtime) conversation
    fn supports_live(&self) -> bool {
        false
    }

    /// Create a live connection for realtime conversation
    async fn create_live_connection(&self) -> Result<Box<dyn LlmConnection>> {
        Err(crate::adk_error!(
            ModelError,
            "Live connections not supported by this model"
        ))
    }

    /// Validate the model configuration
    fn validate(&self) -> Result<()> {
        Ok(())
    }
}

/// Connection for live/realtime LLM interactions
#[async_trait]
pub trait LlmConnection: Send + Sync {
    /// Send a message to the model
    async fn send_message(&mut self, content: Content) -> Result<()>;

    /// Send realtime data (audio/video) to the model
    async fn send_realtime(&mut self, blob: Blob) -> Result<()>;

    /// Receive responses from the model
    async fn receive(&mut self) -> Result<Option<LlmResponse>>;

    /// Close the connection
    async fn close(&mut self) -> Result<()>;

    /// Check if the connection is still active
    fn is_active(&self) -> bool;
}

/// Configuration for LLM models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    /// Model name or identifier
    pub model: String,

    /// API endpoint URL
    pub endpoint: Option<String>,

    /// API key for authentication
    pub api_key: Option<String>,

    /// Project ID (for Google Cloud)
    pub project_id: Option<String>,

    /// Region (for Google Cloud)
    pub region: Option<String>,

    /// Temperature for response generation
    pub temperature: Option<f32>,

    /// Top-p for nucleus sampling
    pub top_p: Option<f32>,

    /// Top-k for top-k sampling
    pub top_k: Option<i32>,

    /// Maximum output tokens
    pub max_output_tokens: Option<i32>,

    /// Stop sequences
    pub stop_sequences: Vec<String>,

    /// Request timeout in seconds
    pub timeout_seconds: Option<u64>,

    /// Additional model-specific parameters
    pub additional_params: serde_json::Value,
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            model: String::new(),
            endpoint: None,
            api_key: None,
            project_id: None,
            region: None,
            temperature: None,
            top_p: None,
            top_k: None,
            max_output_tokens: None,
            stop_sequences: Vec::new(),
            timeout_seconds: Some(30),
            additional_params: serde_json::Value::Null,
        }
    }
}

impl LlmConfig {
    pub fn new(model: impl Into<String>) -> Self {
        Self {
            model: model.into(),
            ..Default::default()
        }
    }

    pub fn with_api_key(mut self, api_key: impl Into<String>) -> Self {
        self.api_key = Some(api_key.into());
        self
    }

    pub fn with_project_id(mut self, project_id: impl Into<String>) -> Self {
        self.project_id = Some(project_id.into());
        self
    }

    pub fn with_region(mut self, region: impl Into<String>) -> Self {
        self.region = Some(region.into());
        self
    }

    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = Some(temperature);
        self
    }

    pub fn with_max_tokens(mut self, max_tokens: i32) -> Self {
        self.max_output_tokens = Some(max_tokens);
        self
    }

    pub fn with_timeout(mut self, timeout_seconds: u64) -> Self {
        self.timeout_seconds = Some(timeout_seconds);
        self
    }
}

/// Helper trait for model builders
pub trait LlmBuilder<T> {
    fn model(self, model: impl Into<String>) -> Self;
    fn config(self, config: LlmConfig) -> Self;
    fn build(self) -> Result<T>;
}
