//! Base agent trait and implementations

use crate::{
    error::{AdkError, Result},
    events::Event,
    types::{AgentId, Metadata},
};
use async_stream::stream;
use async_trait::async_trait;
use futures::Stream;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, pin::Pin};
use tracing::{info, instrument};

use super::invocation_context::InvocationContext;

/// Stream of events from agent execution
pub type EventStream = Pin<Box<dyn Stream<Item = Result<Event>> + Send>>;

/// Base trait for all agents in the ADK
#[async_trait]
pub trait BaseAgent: Send + Sync {
    /// Get the agent's unique identifier
    fn id(&self) -> &AgentId;

    /// Get the agent's name
    fn name(&self) -> &str;

    /// Get the agent's description
    fn description(&self) -> &str;

    /// Get the agent's metadata
    fn metadata(&self) -> &Metadata;

    /// Get the agent's parent, if any
    fn parent(&self) -> Option<&dyn BaseAgent>;

    /// Get the agent's sub-agents
    fn sub_agents(&self) -> &[Box<dyn BaseAgent>];

    /// Run the agent asynchronously with text-based conversation
    async fn run_async(&self, ctx: InvocationContext) -> Result<EventStream>;

    /// Run the agent with live audio/video conversation
    async fn run_live(&self, ctx: InvocationContext) -> Result<EventStream>;

    /// Validate the agent configuration
    fn validate(&self) -> Result<()> {
        // Default implementation - agents can override for custom validation
        Ok(())
    }

    /// Check if this agent can handle the given input
    fn can_handle(&self, _input: &str) -> bool {
        // Default implementation - all agents can handle any input
        true
    }
}

/// Common agent properties
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentProperties {
    pub id: AgentId,
    pub name: String,
    pub description: String,
    pub metadata: Metadata,
    pub parent_id: Option<AgentId>,
    pub sub_agent_ids: Vec<AgentId>,
}

impl AgentProperties {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: name.into(),
            description: String::new(),
            metadata: HashMap::new(),
            parent_id: None,
            sub_agent_ids: Vec::new(),
        }
    }

    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    pub fn with_metadata(mut self, metadata: Metadata) -> Self {
        self.metadata = metadata;
        self
    }
}

/// Helper function to create an event stream from a vector of events
pub fn events_to_stream(events: Vec<Event>) -> EventStream {
    Box::pin(stream! {
        for event in events {
            yield Ok(event);
        }
    })
}

/// Trait for agents that can be used as tools
#[async_trait]
pub trait AgentTool: BaseAgent {
    /// Execute the agent as a tool with the given arguments
    async fn execute_as_tool(
        &self,
        args: serde_json::Value,
        ctx: InvocationContext,
    ) -> Result<serde_json::Value>;
}

/// Builder pattern for creating agents
pub trait AgentBuilder<T> {
    fn name(self, name: impl Into<String>) -> Self;
    fn description(self, description: impl Into<String>) -> Self;
    fn metadata(self, metadata: Metadata) -> Self;
    fn build(self) -> Result<T>;
}
