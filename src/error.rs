//! Error types for the ADK library

use std::fmt;

/// Result type alias for ADK operations
pub type Result<T> = std::result::Result<T, AdkError>;

/// Main error type for the ADK library
#[derive(Debug, thiserror::Error)]
pub enum AdkError {
    /// Initialization errors
    #[error("Initialization error: {0}")]
    InitializationError(String),

    /// Agent-related errors
    #[error("Agent error: {0}")]
    AgentError(String),

    /// Model-related errors
    #[error("Model error: {0}")]
    ModelError(String),

    /// Tool-related errors
    #[error("Tool error: {0}")]
    ToolError(String),

    /// Session-related errors
    #[error("Session error: {0}")]
    SessionError(String),

    /// Memory-related errors
    #[error("Memory error: {0}")]
    MemoryError(String),

    /// Artifact-related errors
    #[error("Artifact error: {0}")]
    ArtifactError(String),

    /// Evaluation-related errors
    #[error("Evaluation error: {0}")]
    EvaluationError(String),

    /// Configuration errors
    #[error("Configuration error: {0}")]
    ConfigError(String),

    /// Network/HTTP errors
    #[error("Network error: {0}")]
    NetworkError(String),

    /// Serialization/deserialization errors
    #[error("Serialization error: {0}")]
    SerializationError(String),

    /// Database errors
    #[error("Database error: {0}")]
    DatabaseError(String),

    /// File I/O errors
    #[error("IO error: {0}")]
    IoError(String),

    /// Authentication errors
    #[error("Authentication error: {0}")]
    AuthError(String),

    /// Validation errors
    #[error("Validation error: {0}")]
    ValidationError(String),

    /// Timeout errors
    #[error("Timeout error: {0}")]
    TimeoutError(String),

    /// Generic errors
    #[error("Error: {0}")]
    Other(String),
}

impl From<std::io::Error> for AdkError {
    fn from(err: std::io::Error) -> Self {
        AdkError::IoError(err.to_string())
    }
}

impl From<serde_json::Error> for AdkError {
    fn from(err: serde_json::Error) -> Self {
        AdkError::SerializationError(err.to_string())
    }
}

impl From<reqwest::Error> for AdkError {
    fn from(err: reqwest::Error) -> Self {
        AdkError::NetworkError(err.to_string())
    }
}

impl From<sqlx::Error> for AdkError {
    fn from(err: sqlx::Error) -> Self {
        AdkError::DatabaseError(err.to_string())
    }
}

impl From<anyhow::Error> for AdkError {
    fn from(err: anyhow::Error) -> Self {
        AdkError::Other(err.to_string())
    }
}

impl From<config::ConfigError> for AdkError {
    fn from(err: config::ConfigError) -> Self {
        AdkError::ConfigError(err.to_string())
    }
}

/// Helper macro for creating errors
#[macro_export]
macro_rules! adk_error {
    ($variant:ident, $msg:expr) => {
        $crate::error::AdkError::$variant($msg.to_string())
    };
    ($variant:ident, $fmt:expr, $($arg:tt)*) => {
        $crate::error::AdkError::$variant(format!($fmt, $($arg)*))
    };
}

/// Helper macro for creating results
#[macro_export]
macro_rules! adk_bail {
    ($variant:ident, $msg:expr) => {
        return Err($crate::adk_error!($variant, $msg))
    };
    ($variant:ident, $fmt:expr, $($arg:tt)*) => {
        return Err($crate::adk_error!($variant, $fmt, $($arg)*))
    };
}
