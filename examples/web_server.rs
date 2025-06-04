//! Web server example demonstrating HTTP API and WebSocket functionality

use google_adk::{
    agents::{LlmAgent, base_agent::AgentBuilder},
    sessions::{SessionService, InMemorySessionService},
    tools::google_search,
    web::{WebServer, ServerConfig, WebServerBuilder},
};
use std::sync::Arc;
use tokio::signal;
use tracing::{info, warn};

#[tokio::main]
async fn main() -> google_adk::error::Result<()> {
    // Initialize the ADK with logging
    google_adk::init()?;
    
    println!("🌐 Google ADK - Web Server Example");
    println!("==================================\n");

    // 1. Create agents
    println!("🤖 Creating Agents:");
    
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

    println!("  ✅ Created 3 agents: research_assistant, chat_assistant, code_assistant");
    println!();

    // 2. Configure server
    println!("⚙️  Configuring Web Server:");
    
    let config = ServerConfig::new()
        .with_host("0.0.0.0".to_string())
        .with_port(8000)
        .with_cors_origins(vec!["*".to_string()])
        .with_timeout(60)
        .with_static_dir("static".to_string());

    println!("  📡 Host: {}", config.host);
    println!("  🔌 Port: {}", config.port);
    println!("  🌐 CORS: {:?}", config.cors_origins);
    println!("  ⏱️  Timeout: {}s", config.timeout_seconds);
    println!("  📁 Static files: {:?}", config.static_dir);
    println!("  🔌 WebSocket: {}", config.enable_websockets);
    println!("  📚 API docs: {}", config.enable_docs);
    println!();

    // 3. Create session service
    println!("💾 Setting up Session Service:");
    let session_service: Arc<dyn SessionService> = Arc::new(InMemorySessionService::new());
    println!("  ✅ In-memory session service created");
    println!();

    // 4. Build web server
    println!("🏗️  Building Web Server:");
    let server = WebServerBuilder::new()
        .config(config)
        .add_agent("research_assistant", Arc::new(research_agent))
        .add_agent("chat_assistant", Arc::new(chat_agent))
        .add_agent("code_assistant", Arc::new(code_agent))
        .session_service(session_service)
        .build();

    println!("  ✅ Web server built with 3 agents");
    println!();

    // 5. Display available endpoints
    println!("🔗 Available Endpoints:");
    println!("  📊 Health Check:     GET  http://localhost:8000/health");
    println!("  🏠 Home Page:        GET  http://localhost:8000/");
    println!("  📋 List Agents:      GET  http://localhost:8000/api/agents");
    println!("  🤖 Run Agent:        POST http://localhost:8000/api/agents/{{name}}/run");
    println!("  📡 Stream Agent:     POST http://localhost:8000/api/agents/{{name}}/stream");
    println!("  📝 List Sessions:    GET  http://localhost:8000/api/sessions");
    println!("  🧠 List Models:      GET  http://localhost:8000/api/models");
    println!("  📚 API Docs:         GET  http://localhost:8000/docs");
    println!("  📄 OpenAPI Spec:     GET  http://localhost:8000/openapi.json");
    println!("  🔌 WebSocket:        WS   ws://localhost:8000/ws/{{agent_name}}");
    println!();

    // 6. Display example usage
    println!("💡 Example Usage:");
    println!();
    
    println!("  📡 HTTP API Examples:");
    println!("    # Health check");
    println!("    curl http://localhost:8000/health");
    println!();
    println!("    # List agents");
    println!("    curl http://localhost:8000/api/agents");
    println!();
    println!("    # Run research assistant");
    println!("    curl -X POST http://localhost:8000/api/agents/research_assistant/run \\");
    println!("      -H 'Content-Type: application/json' \\");
    println!("      -d '{{\"message\": \"What is the latest news about Rust programming?\"}}'");
    println!();
    println!("    # Stream responses");
    println!("    curl -X POST http://localhost:8000/api/agents/chat_assistant/stream \\");
    println!("      -H 'Content-Type: application/json' \\");
    println!("      -H 'Accept: text/event-stream' \\");
    println!("      -d '{{\"message\": \"Tell me a story\"}}'");
    println!();

    println!("  🔌 WebSocket Example (JavaScript):");
    println!("    const ws = new WebSocket('ws://localhost:8000/ws/research_assistant');");
    println!("    ws.onmessage = (event) => console.log(JSON.parse(event.data));");
    println!("    ws.send(JSON.stringify({{");
    println!("      type: 'UserMessage',");
    println!("      message: 'Hello, how can you help me?'");
    println!("    }}));");
    println!();

    // 7. Environment variable guidance
    println!("🔧 Environment Variables for Full Functionality:");
    println!("  GOOGLE_API_KEY          - Required for Google AI/Gemini API");
    println!("  GOOGLE_SEARCH_API_KEY   - Required for real Google Search");
    println!("  GOOGLE_SEARCH_ENGINE_ID - Required for Google Custom Search");
    println!("  GOOGLE_CLOUD_PROJECT    - Optional for Vertex AI");
    println!("  GOOGLE_CLOUD_REGION     - Optional for Vertex AI");
    println!();

    // 8. Show current environment status
    println!("📊 Current Environment Status:");
    println!("  Google AI API Key: {}", 
        if std::env::var("GOOGLE_API_KEY").is_ok() { "✅ Set" } else { "❌ Not set" });
    println!("  Google Search API Key: {}", 
        if std::env::var("GOOGLE_SEARCH_API_KEY").is_ok() { "✅ Set" } else { "❌ Not set" });
    println!("  Google Search Engine ID: {}", 
        if std::env::var("GOOGLE_SEARCH_ENGINE_ID").is_ok() { "✅ Set" } else { "❌ Not set" });
    println!();

    if std::env::var("GOOGLE_API_KEY").is_err() {
        warn!("⚠️  GOOGLE_API_KEY not set - agents will return mock responses");
        println!("  💡 Set GOOGLE_API_KEY environment variable for real LLM integration");
        println!();
    }

    // 9. Start server with graceful shutdown
    println!("🚀 Starting Web Server...");
    println!("   Press Ctrl+C to stop the server");
    println!();

    // Set up graceful shutdown
    let shutdown_signal = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
        
        println!("\n🛑 Shutdown signal received, stopping server...");
    };

    // Start the server
    match server.start_with_shutdown(shutdown_signal).await {
        Ok(()) => {
            println!("✅ Server stopped gracefully");
        }
        Err(e) => {
            eprintln!("❌ Server error: {}", e);
            
            // Check for common issues
            if e.to_string().contains("Address already in use") {
                eprintln!("💡 Tip: Port 8000 is already in use. Try a different port or stop the existing service.");
            }
            if e.to_string().contains("Permission denied") {
                eprintln!("💡 Tip: You may need elevated permissions to bind to this port.");
            }
            
            return Err(e);
        }
    }

    println!("👋 Goodbye!");
    Ok(())
}
