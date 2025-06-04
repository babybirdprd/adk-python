//! LLM-based agent implementation

use crate::{
    agents::{BaseAgent, InvocationContext},
    error::Result,
    events::Event,
    tools::BaseTool,
    types::{AgentId, Metadata},
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

    async fn run_async(&self, _ctx: InvocationContext) -> Result<EventStream> {
        // TODO: Implement actual LLM agent execution
        let events = vec![
            Event::text_response(&self.name, "Hello from LLM agent!"),
        ];
        Ok(Box::pin(stream! {
            for event in events {
                yield Ok(event);
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
