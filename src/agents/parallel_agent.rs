//! Parallel agent implementation

use crate::{
    agents::{BaseAgent, InvocationContext},
    error::Result,
    types::{AgentId, Metadata},
};
use async_trait::async_trait;
use std::collections::HashMap;

use super::base_agent::EventStream;

/// Agent that runs sub-agents in parallel
// Note: Debug not derived due to trait objects
pub struct ParallelAgent {
    id: AgentId,
    name: String,
    description: String,
    sub_agents: Vec<Box<dyn BaseAgent>>,
    metadata: Metadata,
}

impl ParallelAgent {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: name.into(),
            description: String::new(),
            sub_agents: Vec::new(),
            metadata: HashMap::new(),
        }
    }
}

#[async_trait]
impl BaseAgent for ParallelAgent {
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
        None
    }

    fn sub_agents(&self) -> &[Box<dyn BaseAgent>] {
        &self.sub_agents
    }

    async fn run_async(&self, _ctx: InvocationContext) -> Result<EventStream> {
        // TODO: Implement parallel execution
        todo!("Parallel agent execution not implemented")
    }

    async fn run_live(&self, ctx: InvocationContext) -> Result<EventStream> {
        self.run_async(ctx).await
    }
}
