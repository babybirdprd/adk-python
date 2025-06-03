//! Simple agent example

use google_adk::{
    agents::{BaseAgent, LlmAgent, base_agent::AgentBuilder},
    tools::google_search,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the ADK
    google_adk::init()?;
    
    // Create a simple agent
    let agent = LlmAgent::builder()
        .name("search_assistant")
        .model("gemini-2.0-flash")
        .instruction("You are a helpful assistant. Answer user questions using Google Search when needed.")
        .description("An assistant that can search the web.")
        .tool(google_search())
        .build()?;

    println!("Agent '{}' created successfully!", agent.name());
    println!("Description: {}", agent.description());
    
    Ok(())
}
