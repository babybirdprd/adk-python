//! CLI command implementations

use crate::error::Result;
use clap::Args;
use std::path::PathBuf;

/// Create a new agent project
#[derive(Args)]
pub struct CreateCommand {
    /// Project name
    pub name: String,
}

impl CreateCommand {
    pub async fn execute(self) -> Result<()> {
        println!("Creating project: {}", self.name);
        Ok(())
    }
}

/// Run an interactive CLI for an agent
#[derive(Args)]
pub struct RunCommand {
    /// Path to the agent source code folder
    pub agent: PathBuf,
}

impl RunCommand {
    pub async fn execute(self) -> Result<()> {
        println!("Running agent from: {}", self.agent.display());
        Ok(())
    }
}

/// Evaluate an agent
#[derive(Args)]
pub struct EvalCommand {
    /// Path to the agent module
    pub agent_module_file_path: PathBuf,
}

impl EvalCommand {
    pub async fn execute(self) -> Result<()> {
        println!("Evaluating agent: {}", self.agent_module_file_path.display());
        Ok(())
    }
}

/// Start a web server with UI for agents
#[derive(Args)]
pub struct WebCommand {
    /// Port to run the server on
    #[arg(short, long, default_value = "8000")]
    pub port: u16,

    /// Host to bind to
    #[arg(long, default_value = "0.0.0.0")]
    pub host: String,

    /// Disable WebSocket support
    #[arg(long)]
    pub no_websockets: bool,

    /// Disable API documentation
    #[arg(long)]
    pub no_docs: bool,

    /// Static files directory
    #[arg(long)]
    pub static_dir: Option<String>,

    /// Request timeout in seconds
    #[arg(long, default_value = "30")]
    pub timeout: u64,

    /// CORS allowed origins (comma-separated)
    #[arg(long, default_value = "*")]
    pub cors_origins: String,

    /// API key for authentication
    #[arg(long, env = "ADK_API_KEY")]
    pub api_key: Option<String>,
}

impl WebCommand {
    pub async fn execute(self) -> Result<()> {
        use crate::{
            agents::{LlmAgent, base_agent::AgentBuilder},
            sessions::{SessionService, InMemorySessionService},
            tools::google_search,
            web::{ServerConfig, WebServerBuilder},
        };
        use std::sync::Arc;
        use tokio::signal;
        use tracing::{info, warn};

        info!("Starting Google ADK Web Server");

        // Parse CORS origins
        let cors_origins: Vec<String> = self.cors_origins
            .split(',')
            .map(|s| s.trim().to_string())
            .collect();

        // Create server configuration
        let mut config = ServerConfig::new()
            .with_host(self.host)
            .with_port(self.port)
            .with_cors_origins(cors_origins)
            .with_timeout(self.timeout);

        if self.no_websockets {
            config = config.disable_websockets();
        }

        if self.no_docs {
            config = config.disable_docs();
        }

        if let Some(static_dir) = self.static_dir {
            config = config.with_static_dir(static_dir);
        }

        // Create default agents
        info!("Creating default agents");

        let research_agent = LlmAgent::builder()
            .name("research_assistant")
            .model("gemini-2.0-flash")
            .instruction("You are a helpful research assistant. Use Google Search to find current information when needed. Provide comprehensive and accurate answers.")
            .description("An AI assistant that can search the web and provide research help.")
            .tool(google_search())
            .build()?;

        let chat_agent = LlmAgent::builder()
            .name("chat_assistant")
            .model("gemini-pro")
            .instruction("You are a friendly conversational AI assistant. Be helpful, informative, and engaging.")
            .description("A general-purpose conversational AI assistant.")
            .build()?;

        let code_agent = LlmAgent::builder()
            .name("code_assistant")
            .model("gemini-2.0-flash")
            .instruction("You are a programming assistant. Help users with coding questions, debugging, and software development best practices.")
            .description("An AI assistant specialized in programming and software development.")
            .build()?;

        // Create session service
        let session_service: Arc<dyn SessionService> = Arc::new(InMemorySessionService::new());

        // Build web server
        let server = WebServerBuilder::new()
            .config(config.clone())
            .add_agent("research_assistant", Arc::new(research_agent))
            .add_agent("chat_assistant", Arc::new(chat_agent))
            .add_agent("code_assistant", Arc::new(code_agent))
            .session_service(session_service)
            .build();

        // Display server information
        println!("🚀 Google ADK Web Server");
        println!("========================");
        println!();
        println!("📡 Server Configuration:");
        println!("  Host: {}", config.host);
        println!("  Port: {}", config.port);
        println!("  WebSocket: {}", config.enable_websockets);
        println!("  API Docs: {}", config.enable_docs);
        println!("  Timeout: {}s", config.timeout_seconds);
        if let Some(static_dir) = &config.static_dir {
            println!("  Static files: {}", static_dir);
        }
        println!();

        println!("🤖 Available Agents:");
        println!("  - research_assistant (with Google Search)");
        println!("  - chat_assistant (general conversation)");
        println!("  - code_assistant (programming help)");
        println!();

        println!("🔗 Endpoints:");
        println!("  📊 Health:      GET  http://{}:{}/health", config.host, config.port);
        println!("  🏠 Home:        GET  http://{}:{}/", config.host, config.port);
        println!("  📋 Agents:      GET  http://{}:{}/api/agents", config.host, config.port);
        println!("  🤖 Run Agent:   POST http://{}:{}/api/agents/{{name}}/run", config.host, config.port);
        println!("  📡 Stream:      POST http://{}:{}/api/agents/{{name}}/stream", config.host, config.port);
        if config.enable_docs {
            println!("  📚 API Docs:    GET  http://{}:{}/docs", config.host, config.port);
        }
        if config.enable_websockets {
            println!("  🔌 WebSocket:   WS   ws://{}:{}/ws/{{agent_name}}", config.host, config.port);
        }
        println!();

        // Environment status
        println!("📊 Environment Status:");
        println!("  Google AI API Key: {}",
            if std::env::var("GOOGLE_API_KEY").is_ok() { "✅ Set" } else { "❌ Not set" });
        println!("  Google Search API Key: {}",
            if std::env::var("GOOGLE_SEARCH_API_KEY").is_ok() { "✅ Set" } else { "❌ Not set" });

        if self.api_key.is_some() {
            println!("  ADK API Key: ✅ Set");
        }
        println!();

        if std::env::var("GOOGLE_API_KEY").is_err() {
            warn!("⚠️  GOOGLE_API_KEY not set - agents will return mock responses");
            println!("💡 Set GOOGLE_API_KEY environment variable for real LLM integration");
            println!();
        }

        println!("🚀 Starting server... (Press Ctrl+C to stop)");
        println!();

        // Set up graceful shutdown
        let shutdown_signal = async {
            signal::ctrl_c()
                .await
                .expect("Failed to install Ctrl+C handler");

            info!("Shutdown signal received, stopping server...");
        };

        // Start the server
        match server.start_with_shutdown(shutdown_signal).await {
            Ok(()) => {
                info!("Server stopped gracefully");
            }
            Err(e) => {
                eprintln!("❌ Server error: {}", e);

                // Provide helpful error messages
                if e.to_string().contains("Address already in use") {
                    eprintln!("💡 Port {} is already in use. Try a different port with --port", config.port);
                }
                if e.to_string().contains("Permission denied") {
                    eprintln!("💡 Permission denied. You may need elevated permissions for port {}", config.port);
                }

                return Err(e);
            }
        }

        Ok(())
    }
}

/// Start a FastAPI server for agents
#[derive(Args)]
pub struct ApiServerCommand {
    /// Directory containing agents
    pub agents_dir: PathBuf,
}

impl ApiServerCommand {
    pub async fn execute(self) -> Result<()> {
        println!("Starting API server for: {}", self.agents_dir.display());
        Ok(())
    }
}

/// Deploy agents to hosted environments
#[derive(Args)]
pub struct DeployCommand {
    /// Target platform
    pub target: String,
}

impl DeployCommand {
    pub async fn execute(self) -> Result<()> {
        println!("Deploying to: {}", self.target);
        Ok(())
    }
}
