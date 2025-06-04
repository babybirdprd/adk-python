//! Web server implementation with HTTP API and WebSocket support

use crate::{
    agents::BaseAgent,
    error::Result,
    runners::Runner,
    sessions::{SessionService, InMemorySessionService},
    web::{handlers, middleware, WebSocketHandler},
};
use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, net::SocketAddr, sync::Arc, time::Duration};
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
    timeout::TimeoutLayer,
};
use tracing::{info, warn};

/// Web server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// Host to bind to
    pub host: String,
    
    /// Port to bind to
    pub port: u16,
    
    /// CORS allowed origins
    pub cors_origins: Vec<String>,
    
    /// Request timeout in seconds
    pub timeout_seconds: u64,
    
    /// Maximum request body size in bytes
    pub max_body_size: usize,
    
    /// Enable WebSocket support
    pub enable_websockets: bool,
    
    /// Enable API documentation
    pub enable_docs: bool,
    
    /// Static file serving directory
    pub static_dir: Option<String>,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 8000,
            cors_origins: vec!["*".to_string()],
            timeout_seconds: 30,
            max_body_size: 16 * 1024 * 1024, // 16MB
            enable_websockets: true,
            enable_docs: true,
            static_dir: None,
        }
    }
}

impl ServerConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_host(mut self, host: impl Into<String>) -> Self {
        self.host = host.into();
        self
    }

    pub fn with_port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    pub fn with_cors_origins(mut self, origins: Vec<String>) -> Self {
        self.cors_origins = origins;
        self
    }

    pub fn with_timeout(mut self, timeout_seconds: u64) -> Self {
        self.timeout_seconds = timeout_seconds;
        self
    }

    pub fn disable_websockets(mut self) -> Self {
        self.enable_websockets = false;
        self
    }

    pub fn disable_docs(mut self) -> Self {
        self.enable_docs = false;
        self
    }

    pub fn with_static_dir(mut self, dir: impl Into<String>) -> Self {
        self.static_dir = Some(dir.into());
        self
    }

    pub fn socket_addr(&self) -> SocketAddr {
        format!("{}:{}", self.host, self.port)
            .parse()
            .expect("Invalid host:port combination")
    }
}

/// Server state shared across handlers
#[derive(Clone)]
pub struct ServerState {
    /// Available agents
    pub agents: Arc<HashMap<String, Arc<dyn BaseAgent>>>,
    
    /// Session service
    pub session_service: Arc<dyn SessionService>,
    
    /// Active runners
    pub runners: Arc<tokio::sync::RwLock<HashMap<String, Arc<Runner>>>>,
    
    /// Server configuration
    pub config: ServerConfig,
    
    /// WebSocket handler
    pub websocket_handler: Arc<WebSocketHandler>,
}

impl ServerState {
    pub fn new(config: ServerConfig) -> Self {
        let session_service: Arc<dyn SessionService> = Arc::new(InMemorySessionService::new());
        let websocket_handler = Arc::new(WebSocketHandler::new());
        
        Self {
            agents: Arc::new(HashMap::new()),
            session_service,
            runners: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            config,
            websocket_handler,
        }
    }

    pub fn with_agents(mut self, agents: HashMap<String, Arc<dyn BaseAgent>>) -> Self {
        self.agents = Arc::new(agents);
        self
    }

    pub fn with_session_service(mut self, service: Arc<dyn SessionService>) -> Self {
        self.session_service = service;
        self
    }
}

/// Web server for the ADK
pub struct WebServer {
    config: ServerConfig,
    state: ServerState,
}

impl WebServer {
    /// Create a new web server
    pub fn new(config: ServerConfig) -> Self {
        let state = ServerState::new(config.clone());
        
        Self { config, state }
    }

    /// Add an agent to the server
    pub fn add_agent(mut self, name: impl Into<String>, agent: Arc<dyn BaseAgent>) -> Self {
        let mut agents = (*self.state.agents).clone();
        agents.insert(name.into(), agent);
        self.state.agents = Arc::new(agents);
        self
    }

    /// Set session service
    pub fn with_session_service(mut self, service: Arc<dyn SessionService>) -> Self {
        self.state.session_service = service;
        self
    }

    /// Build the router with all routes
    fn build_router(&self) -> Router {
        let mut router = Router::new()
            // Health check
            .route("/health", get(handlers::health_check))
            .route("/", get(handlers::root))
            
            // Agent management
            .route("/api/agents", get(handlers::list_agents))
            .route("/api/agents/:agent_name", get(handlers::get_agent))
            
            // Agent execution
            .route("/api/agents/:agent_name/run", post(handlers::run_agent))
            .route("/api/agents/:agent_name/stream", post(handlers::stream_agent))
            
            // Session management
            .route("/api/sessions", get(handlers::list_sessions))
            .route("/api/sessions/:session_id", get(handlers::get_session))
            .route("/api/sessions/:session_id", post(handlers::update_session))
            .route("/api/sessions/:session_id/events", get(handlers::get_session_events))
            
            // Model information
            .route("/api/models", get(handlers::list_models))
            .route("/api/models/:model_name", get(handlers::get_model_info));

        // Add WebSocket support if enabled
        if self.config.enable_websockets {
            router = router.route("/ws/:agent_name", get(handlers::websocket_handler));
        }

        // Add API documentation if enabled
        if self.config.enable_docs {
            router = router
                .route("/docs", get(handlers::api_docs))
                .route("/openapi.json", get(handlers::openapi_spec));
        }

        // Add static file serving if configured
        if let Some(static_dir) = &self.config.static_dir {
            router = router.nest_service("/static", 
                tower_http::services::ServeDir::new(static_dir));
        }

        // Add middleware
        let cors = if self.config.cors_origins.contains(&"*".to_string()) {
            CorsLayer::permissive()
        } else {
            CorsLayer::new()
                .allow_origin(self.config.cors_origins.iter().map(|origin| {
                    origin.parse().unwrap_or_else(|_| "http://localhost:3000".parse().unwrap())
                }).collect::<Vec<_>>())
                .allow_methods(Any)
                .allow_headers(Any)
        };

        router
            .layer(
                ServiceBuilder::new()
                    .layer(TraceLayer::new_for_http())
                    .layer(TimeoutLayer::new(Duration::from_secs(self.config.timeout_seconds)))
                    .layer(cors)
                    .layer(middleware::request_id::RequestIdLayer::new())
                    .layer(middleware::logging::LoggingLayer::new())
            )
            .with_state(self.state.clone())
    }

    /// Start the web server
    pub async fn start(self) -> Result<()> {
        let addr = self.config.socket_addr();
        let router = self.build_router();

        info!("Starting web server on {}", addr);
        info!("WebSocket support: {}", self.config.enable_websockets);
        info!("API documentation: {}", self.config.enable_docs);
        
        if let Some(static_dir) = &self.config.static_dir {
            info!("Static files served from: {}", static_dir);
        }

        let listener = TcpListener::bind(addr).await?;
        
        info!("🚀 ADK Web Server running on http://{}", addr);
        info!("📚 API Documentation: http://{}/docs", addr);
        info!("🔌 WebSocket endpoint: ws://{}/ws/{{agent_name}}", addr);

        axum::serve(listener, router)
            .await
            .map_err(|e| crate::adk_error!(NetworkError, "Server error: {}", e))?;

        Ok(())
    }

    /// Start the server with graceful shutdown
    pub async fn start_with_shutdown(
        self,
        shutdown_signal: impl std::future::Future<Output = ()> + Send + 'static,
    ) -> Result<()> {
        let addr = self.config.socket_addr();
        let router = self.build_router();

        info!("Starting web server with graceful shutdown on {}", addr);

        let listener = TcpListener::bind(addr).await?;
        
        info!("🚀 ADK Web Server running on http://{}", addr);

        axum::serve(listener, router)
            .with_graceful_shutdown(shutdown_signal)
            .await
            .map_err(|e| crate::adk_error!(NetworkError, "Server error: {}", e))?;

        info!("Server shut down gracefully");
        Ok(())
    }
}

/// Builder for web server
pub struct WebServerBuilder {
    config: ServerConfig,
    agents: HashMap<String, Arc<dyn BaseAgent>>,
    session_service: Option<Arc<dyn SessionService>>,
}

impl WebServerBuilder {
    pub fn new() -> Self {
        Self {
            config: ServerConfig::default(),
            agents: HashMap::new(),
            session_service: None,
        }
    }

    pub fn config(mut self, config: ServerConfig) -> Self {
        self.config = config;
        self
    }

    pub fn host(mut self, host: impl Into<String>) -> Self {
        self.config.host = host.into();
        self
    }

    pub fn port(mut self, port: u16) -> Self {
        self.config.port = port;
        self
    }

    pub fn add_agent(mut self, name: impl Into<String>, agent: Arc<dyn BaseAgent>) -> Self {
        self.agents.insert(name.into(), agent);
        self
    }

    pub fn session_service(mut self, service: Arc<dyn SessionService>) -> Self {
        self.session_service = Some(service);
        self
    }

    pub fn build(self) -> WebServer {
        let mut server = WebServer::new(self.config);
        
        server.state.agents = Arc::new(self.agents);
        
        if let Some(session_service) = self.session_service {
            server.state.session_service = session_service;
        }

        server
    }
}

impl Default for WebServerBuilder {
    fn default() -> Self {
        Self::new()
    }
}
