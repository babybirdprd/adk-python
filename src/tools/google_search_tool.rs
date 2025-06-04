//! Google Search tool implementation

use crate::{
    error::Result,
    tools::{BaseTool, FunctionTool},
    types::FunctionDeclaration,
};
use serde_json::Value;
use std::{collections::HashMap, sync::Arc};

/// Create a Google Search tool
pub fn google_search() -> Arc<dyn BaseTool> {
    let tool = FunctionTool::new(
        "google_search",
        "Search the web using Google Search",
        |args: HashMap<String, Value>| async move {
            let query = args
                .get("query")
                .and_then(|v| v.as_str())
                .ok_or_else(|| crate::adk_error!(ToolError, "Missing 'query' parameter"))?;

            // TODO: Implement actual Google Search API call
            // For now, return a mock response
            Ok(serde_json::json!({
                "results": [
                    {
                        "title": format!("Search result for: {}", query),
                        "url": "https://example.com",
                        "snippet": format!("This is a mock search result for the query: {}", query)
                    }
                ],
                "query": query
            }))
        },
    )
    .with_declaration(FunctionDeclaration {
        name: "google_search".to_string(),
        description: "Search the web using Google Search".to_string(),
        parameters: serde_json::json!({
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "The search query"
                }
            },
            "required": ["query"]
        }),
    });

    Arc::new(tool)
}
