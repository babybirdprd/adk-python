//! Base LLM trait

use crate::error::Result;
use async_trait::async_trait;

use super::{LlmRequest, LlmResponse};

/// Base trait for all LLM implementations
#[async_trait]
pub trait BaseLlm: Send + Sync {
    /// Get the model name
    fn model_name(&self) -> &str;

    /// Generate content from a request
    async fn generate_content(&self, request: LlmRequest) -> Result<LlmResponse>;
}
