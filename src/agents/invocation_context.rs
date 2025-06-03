//! Invocation context for agent execution

use crate::{
    error::Result,
    sessions::SessionService,
    types::{InvocationId, SessionId, SessionState, StateDelta, UserId},
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

/// Context for agent invocation containing session and execution state
#[derive(Clone)]
pub struct InvocationContext {
    /// Unique identifier for this invocation
    pub invocation_id: InvocationId,
    
    /// Session identifier
    pub session_id: SessionId,
    
    /// User identifier
    pub user_id: UserId,
    
    /// Application name
    pub app_name: String,
    
    /// Current session state
    pub state: SessionState,
    
    /// Session service for state management
    pub session_service: Arc<dyn SessionService>,
    
    /// Whether to end the invocation
    pub end_invocation: bool,
    
    /// Timestamp when the invocation started
    pub started_at: DateTime<Utc>,
    
    /// Maximum execution time in seconds
    pub timeout_seconds: Option<u64>,
    
    /// Whether this is a live (audio/video) session
    pub is_live: bool,
}

impl InvocationContext {
    /// Create a new invocation context
    pub fn new(
        session_id: SessionId,
        user_id: UserId,
        app_name: String,
        state: SessionState,
        session_service: Arc<dyn SessionService>,
    ) -> Self {
        Self {
            invocation_id: Uuid::new_v4(),
            session_id,
            user_id,
            app_name,
            state,
            session_service,
            end_invocation: false,
            started_at: Utc::now(),
            timeout_seconds: None,
            is_live: false,
        }
    }

    /// Get a value from the session state
    pub fn get_state_value(&self, key: &str) -> Option<&serde_json::Value> {
        self.state.get(key)
    }

    /// Set a value in the session state
    pub fn set_state_value(&mut self, key: String, value: serde_json::Value) {
        self.state.insert(key, value);
    }

    /// Apply a state delta to the current state
    pub fn apply_state_delta(&mut self, delta: StateDelta) {
        for (key, value) in delta {
            self.state.insert(key, value);
        }
    }

    /// Check if the invocation has timed out
    pub fn is_timed_out(&self) -> bool {
        if let Some(timeout) = self.timeout_seconds {
            let elapsed = Utc::now().signed_duration_since(self.started_at);
            elapsed.num_seconds() as u64 >= timeout
        } else {
            false
        }
    }

    /// Mark the invocation for termination
    pub fn terminate(&mut self) {
        self.end_invocation = true;
    }

    /// Create a child context for sub-agent execution
    pub fn create_child_context(&self, child_app_name: String) -> Self {
        Self {
            invocation_id: Uuid::new_v4(),
            session_id: self.session_id.clone(),
            user_id: self.user_id.clone(),
            app_name: child_app_name,
            state: self.state.clone(),
            session_service: self.session_service.clone(),
            end_invocation: false,
            started_at: self.started_at,
            timeout_seconds: self.timeout_seconds,
            is_live: self.is_live,
        }
    }

    /// Save the current state to the session service
    pub async fn save_state(&self) -> Result<()> {
        self.session_service
            .update_session_state(&self.session_id, &self.state)
            .await
    }
}

/// Builder for creating invocation contexts
pub struct InvocationContextBuilder {
    session_id: Option<SessionId>,
    user_id: Option<UserId>,
    app_name: Option<String>,
    state: SessionState,
    session_service: Option<Arc<dyn SessionService>>,
    timeout_seconds: Option<u64>,
    is_live: bool,
}

impl InvocationContextBuilder {
    pub fn new() -> Self {
        Self {
            session_id: None,
            user_id: None,
            app_name: None,
            state: SessionState::new(),
            session_service: None,
            timeout_seconds: None,
            is_live: false,
        }
    }

    pub fn session_id(mut self, session_id: SessionId) -> Self {
        self.session_id = Some(session_id);
        self
    }

    pub fn user_id(mut self, user_id: UserId) -> Self {
        self.user_id = Some(user_id);
        self
    }

    pub fn app_name(mut self, app_name: String) -> Self {
        self.app_name = Some(app_name);
        self
    }

    pub fn state(mut self, state: SessionState) -> Self {
        self.state = state;
        self
    }

    pub fn session_service(mut self, service: Arc<dyn SessionService>) -> Self {
        self.session_service = Some(service);
        self
    }

    pub fn timeout_seconds(mut self, timeout: u64) -> Self {
        self.timeout_seconds = Some(timeout);
        self
    }

    pub fn is_live(mut self, is_live: bool) -> Self {
        self.is_live = is_live;
        self
    }

    pub fn build(self) -> Result<InvocationContext> {
        let session_id = self.session_id.ok_or_else(|| {
            crate::adk_error!(ValidationError, "session_id is required")
        })?;
        let user_id = self.user_id.ok_or_else(|| {
            crate::adk_error!(ValidationError, "user_id is required")
        })?;
        let app_name = self.app_name.ok_or_else(|| {
            crate::adk_error!(ValidationError, "app_name is required")
        })?;
        let session_service = self.session_service.ok_or_else(|| {
            crate::adk_error!(ValidationError, "session_service is required")
        })?;

        let mut ctx = InvocationContext::new(
            session_id,
            user_id,
            app_name,
            self.state,
            session_service,
        );
        ctx.timeout_seconds = self.timeout_seconds;
        ctx.is_live = self.is_live;

        Ok(ctx)
    }
}

impl Default for InvocationContextBuilder {
    fn default() -> Self {
        Self::new()
    }
}
