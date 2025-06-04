//! Run configuration for agents

use crate::types::StreamingMode;
use serde::{Deserialize, Serialize};

/// Configuration for running agents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunConfig {
    /// Streaming mode for responses
    pub streaming_mode: StreamingMode,
    
    /// Maximum number of iterations
    pub max_iterations: Option<u32>,
    
    /// Timeout in seconds
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
