//! Agent runners for executing agents

use crate::{
    agents::{BaseAgent, InvocationContext},
    error::Result,
    events::Event,
    sessions::{Session, SessionService},
    types::{Content, SessionId, UserId},
};
use async_trait::async_trait;
use futures::Stream;
use std::{pin::Pin, sync::Arc};
use tracing::{info, instrument};

/// Stream of events from runner execution
pub type RunnerEventStream = Pin<Box<dyn Stream<Item = Result<Event>> + Send>>;

/// Runner for executing agents
pub struct Runner {
    app_name: String,
    agent: Arc<dyn BaseAgent>,
    session_service: Arc<dyn SessionService>,
}

impl Runner {
    /// Create a new runner
    pub fn new(
        app_name: impl Into<String>,
        agent: Arc<dyn BaseAgent>,
        session_service: Arc<dyn SessionService>,
    ) -> Self {
        Self {
            app_name: app_name.into(),
            agent,
            session_service,
        }
    }

    /// Run the agent with a new message
    #[instrument(skip(self, new_message))]
    pub async fn run_async(
        &self,
        user_id: UserId,
        session_id: SessionId,
        new_message: Content,
    ) -> Result<RunnerEventStream> {
        info!("Running agent for session: {}", session_id);

        // Get or create session
        let session = self
            .session_service
            .get_session(&self.app_name, &user_id, &session_id)
            .await?
            .unwrap_or_else(|| {
                // Create new session if not found
                Session::new(self.app_name.clone(), user_id.clone(), session_id.clone())
            });

        // Create invocation context
        let context = InvocationContext::new(
            session.id.clone(),
            session.user_id.clone(),
            session.app_name.clone(),
            session.state.clone(),
            self.session_service.clone(),
        );

        // Add the new message to session
        let user_event = Event::user_input(new_message.get_text(), context.invocation_id);
        self.session_service
            .append_event(&session.id, user_event)
            .await?;

        // Run the agent
        self.agent.run_async(context).await
    }

    /// Run the agent in live mode
    #[instrument(skip(self))]
    pub async fn run_live(
        &self,
        user_id: UserId,
        session_id: SessionId,
    ) -> Result<RunnerEventStream> {
        info!("Running agent in live mode for session: {}", session_id);

        // Get or create session
        let session = self
            .session_service
            .get_session(&self.app_name, &user_id, &session_id)
            .await?
            .unwrap_or_else(|| {
                Session::new(self.app_name.clone(), user_id.clone(), session_id.clone())
            });

        // Create invocation context for live mode
        let mut context = InvocationContext::new(
            session.id.clone(),
            session.user_id.clone(),
            session.app_name.clone(),
            session.state.clone(),
            self.session_service.clone(),
        );
        context.is_live = true;

        // Run the agent in live mode
        self.agent.run_live(context).await
    }

    /// Close the runner and cleanup resources
    pub async fn close(&self) -> Result<()> {
        info!("Closing runner for app: {}", self.app_name);
        // TODO: Implement cleanup logic
        Ok(())
    }
}

/// Builder for creating runners
pub struct RunnerBuilder {
    app_name: Option<String>,
    agent: Option<Arc<dyn BaseAgent>>,
    session_service: Option<Arc<dyn SessionService>>,
}

impl RunnerBuilder {
    pub fn new() -> Self {
        Self {
            app_name: None,
            agent: None,
            session_service: None,
        }
    }

    pub fn app_name(mut self, name: impl Into<String>) -> Self {
        self.app_name = Some(name.into());
        self
    }

    pub fn agent(mut self, agent: Arc<dyn BaseAgent>) -> Self {
        self.agent = Some(agent);
        self
    }

    pub fn session_service(mut self, service: Arc<dyn SessionService>) -> Self {
        self.session_service = Some(service);
        self
    }



    pub fn build(self) -> Result<Runner> {
        let app_name = self.app_name.ok_or_else(|| {
            crate::adk_error!(ValidationError, "app_name is required")
        })?;
        let agent = self.agent.ok_or_else(|| {
            crate::adk_error!(ValidationError, "agent is required")
        })?;
        let session_service = self.session_service.ok_or_else(|| {
            crate::adk_error!(ValidationError, "session_service is required")
        })?;

        Ok(Runner::new(app_name, agent, session_service))
    }
}

impl Default for RunnerBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_runner_builder() {
        let builder = RunnerBuilder::new()
            .app_name("test_app");

        // Should fail without required fields
        assert!(builder.build().is_err());
    }
}
