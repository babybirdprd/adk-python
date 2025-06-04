//! HTTP API handlers

use crate::{
    agents::{BaseAgent, InvocationContextBuilder},
    events::Event,
    models::list_available_models,
    types::{SessionState},
    web::ServerState,
};
use axum::{
    extract::{Path, Query, State, WebSocketUpgrade},
    http::StatusCode,
    response::{Json, Response, Html},
};
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{error, warn};
use uuid::Uuid;

/// Health check response
#[derive(Serialize)]
pub struct HealthResponse {
    status: String,
    version: String,
    timestamp: chrono::DateTime<chrono::Utc>,
}

/// Agent information response
#[derive(Serialize)]
pub struct AgentInfo {
    name: String,
    description: String,
    metadata: HashMap<String, serde_json::Value>,
}

/// Agent run request
#[derive(Deserialize)]
pub struct AgentRunRequest {
    message: String,
    session_id: Option<String>,
    user_id: Option<String>,
    stream: Option<bool>,
    temperature: Option<f32>,
    max_tokens: Option<i32>,
}

/// Agent run response
#[derive(Serialize)]
pub struct AgentRunResponse {
    response: String,
    session_id: String,
    events: Vec<EventResponse>,
    metadata: HashMap<String, serde_json::Value>,
}

/// Event response
#[derive(Serialize)]
pub struct EventResponse {
    id: String,
    author: String,
    content: Option<String>,
    timestamp: chrono::DateTime<chrono::Utc>,
    metadata: HashMap<String, serde_json::Value>,
}

/// Session information
#[derive(Serialize)]
pub struct SessionInfo {
    id: String,
    user_id: String,
    app_name: String,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
    event_count: usize,
}

/// Model information response
#[derive(Serialize)]
pub struct ModelInfoResponse {
    name: String,
    supports_streaming: bool,
    supports_function_calling: bool,
    supports_multimodal: bool,
    supports_live: bool,
}

/// Query parameters for listing
#[derive(Deserialize)]
pub struct ListQuery {
    limit: Option<usize>,
    offset: Option<usize>,
    user_id: Option<String>,
}

/// Health check endpoint
pub async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
        version: crate::VERSION.to_string(),
        timestamp: chrono::Utc::now(),
    })
}

/// Root endpoint with API information
pub async fn root() -> Html<&'static str> {
    Html(r#"
<!DOCTYPE html>
<html>
<head>
    <title>Google ADK API</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 40px; }
        .header { color: #1a73e8; }
        .endpoint { background: #f8f9fa; padding: 10px; margin: 10px 0; border-radius: 4px; }
        .method { font-weight: bold; color: #137333; }
    </style>
</head>
<body>
    <h1 class="header">🚀 Google ADK API</h1>
    <p>Welcome to the Google Agent Development Kit REST API</p>
    
    <h2>Available Endpoints</h2>
    <div class="endpoint"><span class="method">GET</span> /health - Health check</div>
    <div class="endpoint"><span class="method">GET</span> /api/agents - List available agents</div>
    <div class="endpoint"><span class="method">POST</span> /api/agents/{name}/run - Run an agent</div>
    <div class="endpoint"><span class="method">GET</span> /api/sessions - List sessions</div>
    <div class="endpoint"><span class="method">GET</span> /api/models - List available models</div>
    <div class="endpoint"><span class="method">GET</span> /docs - API documentation</div>
    <div class="endpoint"><span class="method">WS</span> /ws/{agent_name} - WebSocket connection</div>
    
    <h2>Documentation</h2>
    <p><a href="/docs">📚 Interactive API Documentation</a></p>
    <p><a href="/openapi.json">📄 OpenAPI Specification</a></p>
</body>
</html>
    "#)
}

/// List all available agents
pub async fn list_agents(State(state): State<ServerState>) -> Json<Vec<AgentInfo>> {
    let agents: Vec<AgentInfo> = state
        .agents
        .iter()
        .map(|(name, agent)| AgentInfo {
            name: name.clone(),
            description: agent.description().to_string(),
            metadata: agent.metadata().clone(),
        })
        .collect();

    Json(agents)
}

/// Get information about a specific agent
pub async fn get_agent(
    Path(agent_name): Path<String>,
    State(state): State<ServerState>,
) -> Result<Json<AgentInfo>, StatusCode> {
    if let Some(agent) = state.agents.get(&agent_name) {
        Ok(Json(AgentInfo {
            name: agent_name,
            description: agent.description().to_string(),
            metadata: agent.metadata().clone(),
        }))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

/// Run an agent with a message
pub async fn run_agent(
    Path(agent_name): Path<String>,
    State(state): State<ServerState>,
    Json(request): Json<AgentRunRequest>,
) -> Result<Json<AgentRunResponse>, StatusCode> {
    // Simple implementation for now
    Ok(Json(AgentRunResponse {
        response: format!("Agent {} received: {}", agent_name, request.message),
        session_id: request.session_id.unwrap_or_else(|| Uuid::new_v4().to_string()),
        events: vec![],
        metadata: HashMap::new(),
    }))
}

/// Stream agent responses (Server-Sent Events)
pub async fn stream_agent(
    Path(agent_name): Path<String>,
    State(_state): State<ServerState>,
    Json(request): Json<AgentRunRequest>,
) -> Result<Response, StatusCode> {
    // Simple SSE response for now
    let response = Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "text/event-stream")
        .header("Cache-Control", "no-cache")
        .header("Connection", "keep-alive")
        .body(axum::body::Body::from(format!(
            "data: {{\"message\": \"Streaming from {}: {}\"}}\n\ndata: [DONE]\n\n",
            agent_name, request.message
        )))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(response)
}

/// List sessions
pub async fn list_sessions(
    Query(_query): Query<ListQuery>,
    State(_state): State<ServerState>,
) -> Json<Vec<SessionInfo>> {
    warn!("Session listing not fully implemented");
    Json(vec![])
}

/// Get session information
pub async fn get_session(
    Path(_session_id): Path<String>,
    State(_state): State<ServerState>,
) -> Result<Json<SessionInfo>, StatusCode> {
    warn!("Session retrieval not fully implemented");
    Err(StatusCode::NOT_IMPLEMENTED)
}

/// Update session
pub async fn update_session(
    Path(_session_id): Path<String>,
    State(_state): State<ServerState>,
    Json(_update): Json<serde_json::Value>,
) -> Result<Json<SessionInfo>, StatusCode> {
    warn!("Session updates not fully implemented");
    Err(StatusCode::NOT_IMPLEMENTED)
}

/// Get session events
pub async fn get_session_events(
    Path(_session_id): Path<String>,
    State(_state): State<ServerState>,
) -> Result<Json<Vec<EventResponse>>, StatusCode> {
    warn!("Session event retrieval not fully implemented");
    Ok(Json(vec![]))
}

/// List available models
pub async fn list_models() -> Json<Vec<String>> {
    let models = list_available_models().await;
    Json(models)
}

/// Get model information
pub async fn get_model_info(
    Path(model_name): Path<String>,
) -> Result<Json<ModelInfoResponse>, StatusCode> {
    // Simple implementation for now
    Ok(Json(ModelInfoResponse {
        name: model_name,
        supports_streaming: true,
        supports_function_calling: true,
        supports_multimodal: false,
        supports_live: false,
    }))
}

/// WebSocket handler
pub async fn websocket_handler(
    Path(agent_name): Path<String>,
    ws: WebSocketUpgrade,
    State(state): State<ServerState>,
) -> Response {
    let handler = state.websocket_handler.clone();
    let state_clone = state.clone();
    ws.on_upgrade(move |socket| async move {
        handler.handle_connection(socket, agent_name, state_clone).await
    })
}

/// API documentation
pub async fn api_docs() -> Html<&'static str> {
    Html(r#"
<!DOCTYPE html>
<html>
<head><title>Google ADK API</title></head>
<body>
<h1>Google ADK API Documentation</h1>
<p>API documentation will be available here.</p>
</body>
</html>
    "#)
}

/// OpenAPI specification
pub async fn openapi_spec() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "openapi": "3.0.0",
        "info": {
            "title": "Google ADK API",
            "version": crate::VERSION,
            "description": "REST API for the Google Agent Development Kit"
        },
        "paths": {
            "/health": {
                "get": {
                    "summary": "Health check",
                    "responses": {
                        "200": {
                            "description": "Server is healthy"
                        }
                    }
                }
            },
            "/api/agents": {
                "get": {
                    "summary": "List all agents",
                    "responses": {
                        "200": {
                            "description": "List of available agents"
                        }
                    }
                }
            }
        }
    }))
}
