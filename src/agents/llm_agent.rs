//! LLM-based agent implementation

use crate::{
    agents::{BaseAgent, InvocationContext},
    error::Result,
    events::Event,
    models::{create_model, LlmRequest},
    tools::BaseTool,
    types::{AgentId, Content, Metadata},
};
use async_stream::stream;
use async_trait::async_trait;
use std::{collections::HashMap, sync::Arc};

use super::base_agent::{AgentBuilder, EventStream};

/// LLM-based agent
// Note: Debug not derived due to trait objects
pub struct LlmAgent {
    id: AgentId,
    name: String,
    description: String,
    model: String,
    instruction: String,
    tools: Vec<Arc<dyn BaseTool>>,
    sub_agents: Vec<Box<dyn BaseAgent>>,
    metadata: Metadata,
}

impl LlmAgent {
    /// Create a new builder for LlmAgent
    pub fn builder() -> LlmAgentBuilder {
        LlmAgentBuilder::new()
    }
}

#[async_trait]
impl BaseAgent for LlmAgent {
    fn id(&self) -> &AgentId {
        &self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn metadata(&self) -> &Metadata {
        &self.metadata
    }

    fn parent(&self) -> Option<&dyn BaseAgent> {
        None // TODO: Implement parent tracking
    }

    fn sub_agents(&self) -> &[Box<dyn BaseAgent>] {
        &self.sub_agents
    }

    async fn run_async(&self, ctx: InvocationContext) -> Result<EventStream> {
        let agent_name = self.name.clone();
        let model_name = self.model.clone();
        let instruction = self.instruction.clone();
        let tools = self.tools.clone();

        Ok(Box::pin(stream! {
            // Create the LLM model
            let model = match create_model(&model_name).await {
                Ok(model) => model,
                Err(e) => {
                    yield Err(e);
                    return;
                }
            };

            // Build the conversation history from session
            let mut conversation_history = Vec::new();

            // Add system instruction if provided
            if !instruction.is_empty() {
                conversation_history.push(Content::user_text(format!("System: {}", instruction)));
            }

            // Add conversation history from session events
            for event in &ctx.session_service.get_session(&ctx.app_name, &ctx.user_id, &ctx.session_id)
                .await
                .unwrap_or_default()
                .unwrap_or_else(|| crate::sessions::Session::new(ctx.app_name.clone(), ctx.user_id.clone(), ctx.session_id.clone()))
                .events {
                if let Some(content) = &event.content {
                    conversation_history.push(content.clone());
                }
            }

            // Create LLM request
            let mut request = LlmRequest::new(&model_name);
            for content in conversation_history {
                request = request.add_content(content);
            }

            // Add tools if available
            if !tools.is_empty() {
                request = request.add_tools(tools.clone());
            }

            // Generate response
            match model.generate_content(request.clone()).await {
                Ok(response) => {
                    // Handle function calls
                    if response.has_function_calls() {
                        for function_call in &response.function_calls {
                            yield Ok(Event::text_response(&agent_name, &format!("Calling function: {}", function_call.name)));

                            // Execute the function call
                            if let Some(tool) = request.get_tool(&function_call.name) {
                                let args: HashMap<String, serde_json::Value> = match serde_json::from_value(function_call.args.clone()) {
                                    Ok(args) => args,
                                    Err(e) => {
                                        yield Ok(Event::text_response(&agent_name, &format!("Error parsing function arguments: {}", e)));
                                        continue;
                                    }
                                };

                                match tool.run_async(args).await {
                                    Ok(result) => {
                                        yield Ok(Event::text_response(&agent_name, &format!("Function result: {}", result)));

                                        // Add function result to conversation and continue
                                        let mut follow_up_request = request.clone();
                                        follow_up_request = follow_up_request.add_model_message(format!("Function {} returned: {}", function_call.name, result));

                                        match model.generate_content(follow_up_request).await {
                                            Ok(final_response) => {
                                                if let Some(text) = final_response.get_text() {
                                                    yield Ok(Event::text_response(&agent_name, text));
                                                }
                                            }
                                            Err(e) => {
                                                yield Err(e);
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        yield Ok(Event::text_response(&agent_name, &format!("Function execution error: {}", e)));
                                    }
                                }
                            } else {
                                yield Ok(Event::text_response(&agent_name, &format!("Unknown function: {}", function_call.name)));
                            }
                        }
                    } else if let Some(text) = response.get_text() {
                        // Regular text response
                        yield Ok(Event::text_response(&agent_name, text));
                    } else {
                        yield Ok(Event::text_response(&agent_name, "No response generated"));
                    }
                }
                Err(e) => {
                    yield Err(e);
                }
            }
        }))
    }

    async fn run_live(&self, ctx: InvocationContext) -> Result<EventStream> {
        // TODO: Implement live mode
        self.run_async(ctx).await
    }
}

/// Builder for LlmAgent
pub struct LlmAgentBuilder {
    name: Option<String>,
    description: String,
    model: Option<String>,
    instruction: String,
    tools: Vec<Arc<dyn BaseTool>>,
    sub_agents: Vec<Box<dyn BaseAgent>>,
    metadata: Metadata,
}

impl LlmAgentBuilder {
    pub fn new() -> Self {
        Self {
            name: None,
            description: String::new(),
            model: None,
            instruction: String::new(),
            tools: Vec::new(),
            sub_agents: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    pub fn model(mut self, model: impl Into<String>) -> Self {
        self.model = Some(model.into());
        self
    }

    pub fn instruction(mut self, instruction: impl Into<String>) -> Self {
        self.instruction = instruction.into();
        self
    }

    pub fn tool(mut self, tool: Arc<dyn BaseTool>) -> Self {
        self.tools.push(tool);
        self
    }

    pub fn sub_agent(mut self, agent: Box<dyn BaseAgent>) -> Self {
        self.sub_agents.push(agent);
        self
    }
}

impl AgentBuilder<LlmAgent> for LlmAgentBuilder {
    fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    fn description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    fn metadata(mut self, metadata: Metadata) -> Self {
        self.metadata = metadata;
        self
    }

    fn build(self) -> Result<LlmAgent> {
        let name = self.name.ok_or_else(|| {
            crate::adk_error!(ValidationError, "Agent name is required")
        })?;
        
        let model = self.model.ok_or_else(|| {
            crate::adk_error!(ValidationError, "Model is required")
        })?;

        Ok(LlmAgent {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            description: self.description,
            model,
            instruction: self.instruction,
            tools: self.tools,
            sub_agents: self.sub_agents,
            metadata: self.metadata,
        })
    }
}

impl Default for LlmAgentBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Type alias for backward compatibility
pub type Agent = LlmAgent;
