//! Base tool trait and implementations

use crate::{
    error::Result,
    types::FunctionDeclaration,
};
use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;

/// Base trait for all tools
#[async_trait]
pub trait BaseTool: Send + Sync {
    /// Get the tool's name
    fn name(&self) -> &str;

    /// Get the tool's description
    fn description(&self) -> &str;

    /// Get the function declaration for this tool
    fn get_declaration(&self) -> Option<FunctionDeclaration> {
        None
    }

    /// Run the tool with the given arguments
    async fn run_async(
        &self,
        args: HashMap<String, Value>,
    ) -> Result<Value>;
}
