//! # Google Agent Development Kit (ADK) - Rust
//!
//! A flexible and modular framework for developing and deploying AI agents.
//! While optimized for Gemini and the Google ecosystem, ADK is model-agnostic,
//! deployment-agnostic, and is built for compatibility with other frameworks.
//!
//! ## Features
//!
//! - **Rich Tool Ecosystem**: Utilize pre-built tools, custom functions,
//!   OpenAPI specs, or integrate existing tools to give agents diverse
//!   capabilities, all for tight integration with the Google ecosystem.
//!
//! - **Code-First Development**: Define agent logic, tools, and orchestration
//!   directly in Rust for ultimate flexibility, testability, and versioning.
//!
//! - **Modular Multi-Agent Systems**: Design scalable applications by composing
//!   multiple specialized agents into flexible hierarchies.
//!
//! - **Deploy Anywhere**: Easily containerize and deploy agents on Cloud Run or
//!   scale seamlessly with Vertex AI Agent Engine.
//!
//! ## Quick Start
//!
//! ```rust
//! use google_adk::agents::LlmAgent;
//! use google_adk::tools::google_search;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let agent = LlmAgent::builder()
//!         .name("search_assistant")
//!         .model("gemini-2.0-flash")
//!         .instruction("You are a helpful assistant. Answer user questions using Google Search when needed.")
//!         .description("An assistant that can search the web.")
//!         .tool(google_search())
//!         .build()?;
//!
//!     // Use the agent...
//!     Ok(())
//! }
//! ```

pub mod agents;
pub mod artifacts;
pub mod cli;
pub mod error;
pub mod events;
pub mod evaluation;
pub mod memory;
pub mod models;
pub mod runners;
pub mod sessions;
pub mod tools;
pub mod types;
pub mod utils;
pub mod web;

// Re-export commonly used types
pub use agents::{Agent, BaseAgent, LlmAgent};
pub use error::{AdkError, Result};
pub use events::Event;
pub use models::{BaseLlm, LlmRequest, LlmResponse};
pub use runners::Runner;
pub use sessions::Session;
pub use tools::{BaseTool, FunctionTool};
pub use types::*;

/// The version of the ADK library
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Initialize the ADK library with default configuration
pub fn init() -> Result<()> {
    tracing_subscriber::fmt::init();
    Ok(())
}

/// Initialize the ADK library with custom tracing configuration
pub fn init_with_tracing(subscriber: impl tracing::Subscriber + Send + Sync) -> Result<()> {
    tracing::subscriber::set_global_default(subscriber)
        .map_err(|e| AdkError::InitializationError(e.to_string()))?;
    Ok(())
}
