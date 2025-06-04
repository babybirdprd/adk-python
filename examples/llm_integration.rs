//! LLM integration example demonstrating real Google AI API usage

use google_adk::{
    agents::{BaseAgent, LlmAgent, base_agent::AgentBuilder, InvocationContextBuilder},
    models::{create_model, get_model_info, list_available_models},
    sessions::{SessionService, InMemorySessionService},
    tools::google_search,
    types::SessionState,
};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the ADK with logging
    google_adk::init()?;
    
    println!("🚀 Google ADK - LLM Integration Example");
    println!("========================================\n");

    // 1. List available models
    println!("📋 Available Model Patterns:");
    let models = list_available_models().await;
    for model in &models {
        println!("  - {}", model);
    }
    println!();

    // 2. Get model information
    let model_name = "gemini-2.0-flash";
    println!("🔍 Model Information for '{}':", model_name);
    match get_model_info(model_name).await {
        Ok(info) => {
            println!("  - Supports Streaming: {}", info.supports_streaming);
            println!("  - Supports Function Calling: {}", info.supports_function_calling);
            println!("  - Supports Multimodal: {}", info.supports_multimodal);
            println!("  - Supports Live: {}", info.supports_live);
        }
        Err(e) => println!("  Error: {}", e),
    }
    println!();

    // 3. Create a model instance
    println!("🤖 Creating Model Instance:");
    let model = match create_model(model_name).await {
        Ok(model) => {
            println!("  ✅ Successfully created model: {}", model.model_name());
            model
        }
        Err(e) => {
            println!("  ❌ Failed to create model: {}", e);
            println!("  💡 Make sure to set GOOGLE_API_KEY environment variable");
            return Ok(());
        }
    };
    println!();

    // 4. Create tools
    println!("🔧 Creating Tools:");
    let search_tool = if std::env::var("GOOGLE_SEARCH_API_KEY").is_ok() || 
                         std::env::var("GOOGLE_API_KEY").is_ok() {
        println!("  ✅ Google Search tool with real API");
        google_search()
    } else {
        println!("  ⚠️  Google Search tool with mock responses (set GOOGLE_SEARCH_API_KEY for real search)");
        google_search()
    };
    println!();

    // 5. Create an agent
    println!("🤖 Creating LLM Agent:");
    let agent = LlmAgent::builder()
        .name("research_assistant")
        .model(model_name)
        .instruction("You are a helpful research assistant. Use Google Search to find current information when needed. Provide comprehensive and accurate answers.")
        .description("An AI assistant that can search the web and provide research help.")
        .tool(search_tool)
        .build()?;

    println!("  ✅ Agent '{}' created successfully", agent.name());
    println!("  📝 Description: {}", agent.description());
    println!();

    // 6. Set up session management
    println!("💾 Setting up Session Management:");
    let session_service: Arc<dyn SessionService> = Arc::new(InMemorySessionService::new());
    println!("  ✅ In-memory session service created");
    println!();

    // 7. Create invocation context
    println!("🎯 Creating Invocation Context:");
    let context = InvocationContextBuilder::new()
        .session_id("example_session_001".to_string())
        .user_id("user_123".to_string())
        .app_name("llm_integration_example".to_string())
        .state(SessionState::new())
        .session_service(session_service.clone())
        .timeout_seconds(30)
        .build()?;

    println!("  ✅ Context created for session: {}", context.session_id);
    println!();

    // 8. Test basic agent execution
    println!("🎬 Testing Agent Execution:");
    println!("  Query: 'What is the latest news about Rust programming language?'");
    
    // Add a user message to the session
    let user_message = google_adk::events::Event::user_input(
        "What is the latest news about Rust programming language?",
        context.invocation_id,
    );
    
    session_service.append_event(&context.session_id, user_message).await?;

    // Run the agent
    match agent.run_async(context.clone()).await {
        Ok(mut event_stream) => {
            println!("  📡 Receiving agent responses:");
            
            use futures::StreamExt;
            while let Some(event_result) = event_stream.next().await {
                match event_result {
                    Ok(event) => {
                        if let Some(text) = event.get_text() {
                            println!("    🤖 {}: {}", event.author, text);
                        }
                    }
                    Err(e) => {
                        println!("    ❌ Error: {}", e);
                        break;
                    }
                }
            }
        }
        Err(e) => {
            println!("  ❌ Agent execution failed: {}", e);
            
            // Check for common issues
            if e.to_string().contains("API key") {
                println!("  💡 Tip: Set GOOGLE_API_KEY environment variable");
            }
            if e.to_string().contains("quota") {
                println!("  💡 Tip: Check your Google AI API quota and billing");
            }
        }
    }
    println!();

    // 9. Environment variable guidance
    println!("🔧 Environment Variables for Full Functionality:");
    println!("  GOOGLE_API_KEY          - Required for Google AI/Gemini API");
    println!("  GOOGLE_SEARCH_API_KEY   - Required for real Google Search");
    println!("  GOOGLE_SEARCH_ENGINE_ID - Required for Google Custom Search");
    println!("  GOOGLE_CLOUD_PROJECT    - Optional for Vertex AI");
    println!("  GOOGLE_CLOUD_REGION     - Optional for Vertex AI");
    println!();

    // 10. Show current environment status
    println!("📊 Current Environment Status:");
    println!("  Google AI API Key: {}", 
        if std::env::var("GOOGLE_API_KEY").is_ok() { "✅ Set" } else { "❌ Not set" });
    println!("  Google Search API Key: {}", 
        if std::env::var("GOOGLE_SEARCH_API_KEY").is_ok() { "✅ Set" } else { "❌ Not set" });
    println!("  Google Search Engine ID: {}", 
        if std::env::var("GOOGLE_SEARCH_ENGINE_ID").is_ok() { "✅ Set" } else { "❌ Not set" });
    println!("  Google Cloud Project: {}", 
        if std::env::var("GOOGLE_CLOUD_PROJECT").is_ok() { "✅ Set" } else { "❌ Not set" });

    println!("\n🎉 Example completed successfully!");
    Ok(())
}
