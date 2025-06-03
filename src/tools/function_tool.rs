//! Function tool implementation

use crate::{
    error::Result,
    tools::BaseTool,
    types::FunctionDeclaration,
};
use async_trait::async_trait;
use serde_json::Value;
use std::{collections::HashMap, future::Future, pin::Pin, sync::Arc};

/// Type alias for async function
pub type AsyncToolFunction = Arc<
    dyn Fn(HashMap<String, Value>) -> Pin<Box<dyn Future<Output = Result<Value>> + Send>>
        + Send
        + Sync,
>;

/// Tool that wraps a user-defined function
pub struct FunctionTool {
    name: String,
    description: String,
    function: AsyncToolFunction,
    declaration: Option<FunctionDeclaration>,
}

impl FunctionTool {
    /// Create a new function tool
    pub fn new<F, Fut>(
        name: impl Into<String>,
        description: impl Into<String>,
        function: F,
    ) -> Self
    where
        F: Fn(HashMap<String, Value>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<Value>> + Send + 'static,
    {
        let name = name.into();
        let description = description.into();
        
        // Create the declaration
        let declaration = FunctionDeclaration {
            name: name.clone(),
            description: description.clone(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
        };

        Self {
            name,
            description,
            function: Arc::new(move |args| Box::pin(function(args))),
            declaration: Some(declaration),
        }
    }

    /// Create with custom declaration
    pub fn with_declaration(mut self, declaration: FunctionDeclaration) -> Self {
        self.declaration = Some(declaration);
        self
    }
}

impl std::fmt::Debug for FunctionTool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FunctionTool")
            .field("name", &self.name)
            .field("description", &self.description)
            .finish()
    }
}

#[async_trait]
impl BaseTool for FunctionTool {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn get_declaration(&self) -> Option<FunctionDeclaration> {
        self.declaration.clone()
    }

    async fn run_async(
        &self,
        args: HashMap<String, Value>,
    ) -> Result<Value> {
        (self.function)(args).await
    }
}
