//! Model system for LLM integration

pub mod base_llm;
pub mod google_llm;
pub mod llm_request;
pub mod llm_response;
pub mod registry;

#[cfg(feature = "anthropic")]
pub mod anthropic_llm;

pub use base_llm::{BaseLlm, LlmConnection};
pub use google_llm::GoogleLlm;
pub use llm_request::{LlmRequest, LlmRequestBuilder};
pub use llm_response::{LlmResponse, FinishReason, Usage};
pub use registry::{LlmRegistry, global_registry, create_model, get_model_info, list_available_models, ModelInfo};

#[cfg(feature = "anthropic")]
pub use anthropic_llm::AnthropicLlm;
