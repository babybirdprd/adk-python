//! Google Search tool implementation

use crate::{
    error::Result,
    tools::{BaseTool, FunctionTool},
    types::FunctionDeclaration,
};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{collections::HashMap, sync::Arc, time::Duration};
use tracing::{debug, error, warn};

/// Google Custom Search API response
#[derive(Debug, Deserialize)]
struct GoogleSearchResponse {
    items: Option<Vec<GoogleSearchItem>>,
    #[serde(rename = "searchInformation")]
    search_information: Option<GoogleSearchInfo>,
}

#[derive(Debug, Deserialize)]
struct GoogleSearchItem {
    title: String,
    link: String,
    snippet: Option<String>,
    #[serde(rename = "displayLink")]
    display_link: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GoogleSearchInfo {
    #[serde(rename = "totalResults")]
    total_results: String,
    #[serde(rename = "searchTime")]
    search_time: f64,
}

/// Perform actual Google Custom Search API call
async fn perform_google_search(query: &str) -> Result<Value> {
    // Check for required environment variables
    let api_key = std::env::var("GOOGLE_SEARCH_API_KEY")
        .or_else(|_| std::env::var("GOOGLE_API_KEY"))
        .map_err(|_| crate::adk_error!(
            ToolError,
            "Google Search API key not found. Set GOOGLE_SEARCH_API_KEY or GOOGLE_API_KEY environment variable"
        ))?;

    let search_engine_id = std::env::var("GOOGLE_SEARCH_ENGINE_ID")
        .or_else(|_| std::env::var("GOOGLE_CSE_ID"))
        .map_err(|_| crate::adk_error!(
            ToolError,
            "Google Search Engine ID not found. Set GOOGLE_SEARCH_ENGINE_ID or GOOGLE_CSE_ID environment variable"
        ))?;

    debug!("Performing Google search for query: {}", query);

    let client = Client::builder()
        .timeout(Duration::from_secs(10))
        .build()?;

    let url = "https://www.googleapis.com/customsearch/v1";
    let response = client
        .get(url)
        .query(&[
            ("key", api_key.as_str()),
            ("cx", search_engine_id.as_str()),
            ("q", query),
            ("num", "5"), // Limit to 5 results
        ])
        .send()
        .await?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_default();
        error!("Google Search API error: {} - {}", status, error_text);
        return Err(crate::adk_error!(
            ToolError,
            "Google Search API error: {} - {}",
            status,
            error_text
        ));
    }

    let search_response: GoogleSearchResponse = response.json().await?;

    let results = search_response.items.unwrap_or_default()
        .into_iter()
        .map(|item| serde_json::json!({
            "title": item.title,
            "url": item.link,
            "snippet": item.snippet.unwrap_or_default(),
            "display_link": item.display_link.unwrap_or_default()
        }))
        .collect::<Vec<_>>();

    let total_results = search_response.search_information
        .as_ref()
        .map(|info| info.total_results.clone())
        .unwrap_or_default();

    let search_time = search_response.search_information
        .as_ref()
        .map(|info| info.search_time)
        .unwrap_or_default();

    debug!("Google search completed: {} results in {}s", results.len(), search_time);

    Ok(serde_json::json!({
        "results": results,
        "query": query,
        "total_results": total_results,
        "search_time": search_time
    }))
}

/// Create a Google Search tool
pub fn google_search() -> Arc<dyn BaseTool> {
    let tool = FunctionTool::new(
        "google_search",
        "Search the web using Google Custom Search API",
        |args: HashMap<String, Value>| async move {
            let query = args
                .get("query")
                .and_then(|v| v.as_str())
                .ok_or_else(|| crate::adk_error!(ToolError, "Missing 'query' parameter"))?;

            // Check if we have API credentials
            if std::env::var("GOOGLE_SEARCH_API_KEY").is_err() &&
               std::env::var("GOOGLE_API_KEY").is_err() {
                warn!("Google Search API key not configured, returning mock results");

                // Return mock response when API key is not available
                return Ok(serde_json::json!({
                    "results": [
                        {
                            "title": format!("Mock search result for: {}", query),
                            "url": "https://example.com",
                            "snippet": format!("This is a mock search result for the query: {}", query),
                            "display_link": "example.com"
                        }
                    ],
                    "query": query,
                    "total_results": "1",
                    "search_time": 0.1,
                    "note": "Mock results - configure GOOGLE_SEARCH_API_KEY for real search"
                }));
            }

            // Perform actual search
            perform_google_search(query).await
        },
    )
    .with_declaration(FunctionDeclaration {
        name: "google_search".to_string(),
        description: "Search the web using Google Custom Search API. Returns a list of search results with titles, URLs, and snippets.".to_string(),
        parameters: serde_json::json!({
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "The search query to execute"
                }
            },
            "required": ["query"]
        }),
    });

    Arc::new(tool)
}

/// Create a Google Search tool with custom configuration
pub fn google_search_with_config(api_key: String, search_engine_id: String) -> Arc<dyn BaseTool> {
    let tool = FunctionTool::new(
        "google_search",
        "Search the web using Google Custom Search API",
        move |args: HashMap<String, Value>| {
            let api_key = api_key.clone();
            let search_engine_id = search_engine_id.clone();

            async move {
                let query = args
                    .get("query")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| crate::adk_error!(ToolError, "Missing 'query' parameter"))?;

                debug!("Performing Google search with custom config for query: {}", query);

                let client = Client::builder()
                    .timeout(Duration::from_secs(10))
                    .build()?;

                let url = "https://www.googleapis.com/customsearch/v1";
                let response = client
                    .get(url)
                    .query(&[
                        ("key", api_key.as_str()),
                        ("cx", search_engine_id.as_str()),
                        ("q", query),
                        ("num", "5"),
                    ])
                    .send()
                    .await?;

                if !response.status().is_success() {
                    let status = response.status();
                    let error_text = response.text().await.unwrap_or_default();
                    return Err(crate::adk_error!(
                        ToolError,
                        "Google Search API error: {} - {}",
                        status,
                        error_text
                    ));
                }

                let search_response: GoogleSearchResponse = response.json().await?;

                let results = search_response.items.unwrap_or_default()
                    .into_iter()
                    .map(|item| serde_json::json!({
                        "title": item.title,
                        "url": item.link,
                        "snippet": item.snippet.unwrap_or_default(),
                        "display_link": item.display_link.unwrap_or_default()
                    }))
                    .collect::<Vec<_>>();

                Ok(serde_json::json!({
                    "results": results,
                    "query": query
                }))
            }
        },
    )
    .with_declaration(FunctionDeclaration {
        name: "google_search".to_string(),
        description: "Search the web using Google Custom Search API".to_string(),
        parameters: serde_json::json!({
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "The search query to execute"
                }
            },
            "required": ["query"]
        }),
    });

    Arc::new(tool)
}
