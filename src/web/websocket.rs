//! WebSocket handler for real-time agent communication

use crate::{
    agents::{BaseAgent, InvocationContextBuilder},
    events::Event,
    types::{SessionState},
    web::ServerState,
};
use axum::extract::ws::{Message, WebSocket};
use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::broadcast;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// WebSocket message types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WebSocketMessage {
    /// User message to agent
    UserMessage {
        message: String,
        session_id: Option<String>,
        user_id: Option<String>,
        metadata: Option<HashMap<String, serde_json::Value>>,
    },
    
    /// Agent response
    AgentResponse {
        message: String,
        session_id: String,
        author: String,
        timestamp: chrono::DateTime<chrono::Utc>,
        is_partial: bool,
        metadata: HashMap<String, serde_json::Value>,
    },
    
    /// System message
    SystemMessage {
        message: String,
        level: String, // info, warning, error
    },
    
    /// Connection status
    ConnectionStatus {
        status: String, // connected, disconnected, error
        agent_name: String,
        session_id: Option<String>,
    },
    
    /// Ping/Pong for keepalive
    Ping { timestamp: chrono::DateTime<chrono::Utc> },
    Pong { timestamp: chrono::DateTime<chrono::Utc> },
    
    /// Error message
    Error {
        error: String,
        code: Option<String>,
    },
}

/// WebSocket connection state
#[derive(Debug, Clone)]
pub struct ConnectionState {
    pub session_id: String,
    pub user_id: String,
    pub agent_name: String,
    pub connected_at: chrono::DateTime<chrono::Utc>,
}

/// WebSocket handler
pub struct WebSocketHandler {
    /// Active connections
    connections: Arc<tokio::sync::RwLock<HashMap<String, ConnectionState>>>,
    
    /// Broadcast channel for system messages
    broadcast_tx: broadcast::Sender<WebSocketMessage>,
}

impl WebSocketHandler {
    /// Create a new WebSocket handler
    pub fn new() -> Self {
        let (broadcast_tx, _) = broadcast::channel(1000);
        
        Self {
            connections: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            broadcast_tx,
        }
    }

    /// Handle a new WebSocket connection
    pub async fn handle_connection(
        &self,
        socket: WebSocket,
        agent_name: String,
        state: ServerState,
    ) {
        let connection_id = Uuid::new_v4().to_string();
        let session_id = Uuid::new_v4().to_string();
        let user_id = "websocket_user".to_string();

        info!("New WebSocket connection: {} for agent: {}", connection_id, agent_name);

        // Register connection
        {
            let mut connections = self.connections.write().await;
            connections.insert(connection_id.clone(), ConnectionState {
                session_id: session_id.clone(),
                user_id: user_id.clone(),
                agent_name: agent_name.clone(),
                connected_at: chrono::Utc::now(),
            });
        }

        // Split socket into sender and receiver
        let (mut sender, mut receiver) = socket.split();

        // Send connection status
        let status_msg = WebSocketMessage::ConnectionStatus {
            status: "connected".to_string(),
            agent_name: agent_name.clone(),
            session_id: Some(session_id.clone()),
        };

        if let Err(e) = sender.send(Message::Text(serde_json::to_string(&status_msg).unwrap())).await {
            error!("Failed to send connection status: {}", e);
            return;
        }

        // Get agent
        let agent = match state.agents.get(&agent_name) {
            Some(agent) => agent.clone(),
            None => {
                let error_msg = WebSocketMessage::Error {
                    error: format!("Agent '{}' not found", agent_name),
                    code: Some("AGENT_NOT_FOUND".to_string()),
                };
                let _ = sender.send(Message::Text(serde_json::to_string(&error_msg).unwrap())).await;
                return;
            }
        };

        // Create broadcast receiver for system messages
        let mut broadcast_rx = self.broadcast_tx.subscribe();

        // Handle incoming messages
        let connections_clone = self.connections.clone();
        let agent_clone = agent.clone();
        let state_clone = state.clone();
        let session_id_clone = session_id.clone();
        let user_id_clone = user_id.clone();
        let agent_name_clone = agent_name.clone();

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    // Handle incoming WebSocket messages
                    msg = receiver.next() => {
                        match msg {
                            Some(Ok(Message::Text(text))) => {
                                debug!("Received WebSocket message: {}", text);
                                
                                match serde_json::from_str::<WebSocketMessage>(&text) {
                                    Ok(ws_msg) => {
                                        if let Err(e) = Self::handle_message(
                                            ws_msg,
                                            &mut sender,
                                            &agent_clone,
                                            &state_clone,
                                            &session_id_clone,
                                            &user_id_clone,
                                            &agent_name_clone,
                                        ).await {
                                            error!("Error handling WebSocket message: {}", e);
                                            let error_msg = WebSocketMessage::Error {
                                                error: e.to_string(),
                                                code: Some("MESSAGE_HANDLING_ERROR".to_string()),
                                            };
                                            let _ = sender.send(Message::Text(serde_json::to_string(&error_msg).unwrap())).await;
                                        }
                                    }
                                    Err(e) => {
                                        warn!("Failed to parse WebSocket message: {}", e);
                                        let error_msg = WebSocketMessage::Error {
                                            error: "Invalid message format".to_string(),
                                            code: Some("INVALID_MESSAGE".to_string()),
                                        };
                                        let _ = sender.send(Message::Text(serde_json::to_string(&error_msg).unwrap())).await;
                                    }
                                }
                            }
                            Some(Ok(Message::Ping(data))) => {
                                debug!("Received ping, sending pong");
                                if let Err(e) = sender.send(Message::Pong(data)).await {
                                    error!("Failed to send pong: {}", e);
                                    break;
                                }
                            }
                            Some(Ok(Message::Close(_))) => {
                                info!("WebSocket connection closed by client");
                                break;
                            }
                            Some(Err(e)) => {
                                error!("WebSocket error: {}", e);
                                break;
                            }
                            None => {
                                debug!("WebSocket stream ended");
                                break;
                            }
                            _ => {
                                debug!("Received other WebSocket message type");
                            }
                        }
                    }
                    
                    // Handle broadcast messages
                    broadcast_msg = broadcast_rx.recv() => {
                        match broadcast_msg {
                            Ok(msg) => {
                                if let Err(e) = sender.send(Message::Text(serde_json::to_string(&msg).unwrap())).await {
                                    error!("Failed to send broadcast message: {}", e);
                                    break;
                                }
                            }
                            Err(broadcast::error::RecvError::Lagged(_)) => {
                                warn!("Broadcast receiver lagged, continuing");
                            }
                            Err(broadcast::error::RecvError::Closed) => {
                                debug!("Broadcast channel closed");
                                break;
                            }
                        }
                    }
                }
            }

            // Cleanup connection
            {
                let mut connections = connections_clone.write().await;
                connections.remove(&connection_id);
            }

            info!("WebSocket connection {} closed", connection_id);
        });
    }

    /// Handle a WebSocket message
    async fn handle_message(
        message: WebSocketMessage,
        sender: &mut futures::stream::SplitSink<WebSocket, Message>,
        agent: &Arc<dyn BaseAgent>,
        state: &ServerState,
        session_id: &str,
        user_id: &str,
        agent_name: &str,
    ) -> crate::error::Result<()> {
        match message {
            WebSocketMessage::UserMessage { message, session_id: msg_session_id, user_id: msg_user_id, metadata } => {
                let effective_session_id = msg_session_id.unwrap_or_else(|| session_id.to_string());
                let effective_user_id = msg_user_id.unwrap_or_else(|| user_id.to_string());

                // Create invocation context
                let context = InvocationContextBuilder::new()
                    .session_id(effective_session_id.clone())
                    .user_id(effective_user_id.clone())
                    .app_name(agent_name.to_string())
                    .state(SessionState::new())
                    .session_service(state.session_service.clone())
                    .timeout_seconds(30)
                    .build()?;

                // Add user message to session
                let user_event = Event::user_input(&message, context.invocation_id);
                state.session_service.append_event(&effective_session_id, user_event).await?;

                // Run agent and stream responses
                let mut event_stream = agent.run_async(context).await?;
                
                while let Some(event_result) = event_stream.next().await {
                    match event_result {
                        Ok(event) => {
                            if let Some(text) = event.get_text() {
                                let response_msg = WebSocketMessage::AgentResponse {
                                    message: text,
                                    session_id: effective_session_id.clone(),
                                    author: event.author,
                                    timestamp: event.timestamp,
                                    is_partial: event.is_partial,
                                    metadata: event.metadata,
                                };

                                sender.send(Message::Text(serde_json::to_string(&response_msg)?)).await
                                    .map_err(|e| crate::adk_error!(NetworkError, "Failed to send response: {}", e))?;
                            }
                        }
                        Err(e) => {
                            let error_msg = WebSocketMessage::Error {
                                error: e.to_string(),
                                code: Some("AGENT_EXECUTION_ERROR".to_string()),
                            };
                            sender.send(Message::Text(serde_json::to_string(&error_msg)?)).await
                                .map_err(|e| crate::adk_error!(NetworkError, "Failed to send error: {}", e))?;
                            break;
                        }
                    }
                }
            }
            
            WebSocketMessage::Ping { timestamp: _ } => {
                let pong_msg = WebSocketMessage::Pong {
                    timestamp: chrono::Utc::now(),
                };
                sender.send(Message::Text(serde_json::to_string(&pong_msg)?)).await
                    .map_err(|e| crate::adk_error!(NetworkError, "Failed to send pong: {}", e))?;
            }
            
            _ => {
                debug!("Received unhandled message type");
            }
        }

        Ok(())
    }

    /// Broadcast a system message to all connections
    pub async fn broadcast_system_message(&self, message: String, level: String) {
        let msg = WebSocketMessage::SystemMessage { message, level };
        if let Err(e) = self.broadcast_tx.send(msg) {
            error!("Failed to broadcast system message: {}", e);
        }
    }

    /// Get connection count
    pub async fn connection_count(&self) -> usize {
        self.connections.read().await.len()
    }

    /// Get active connections
    pub async fn get_connections(&self) -> Vec<ConnectionState> {
        self.connections.read().await.values().cloned().collect()
    }
}

impl Default for WebSocketHandler {
    fn default() -> Self {
        Self::new()
    }
}
