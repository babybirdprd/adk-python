//! Session service implementations

use crate::{
    error::Result,
    events::Event,
    types::{SessionId, SessionState, UserId},
};
use async_trait::async_trait;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;

use super::session::Session;

/// Service for managing sessions
#[async_trait]
pub trait SessionService: Send + Sync {
    /// Get a session by ID
    async fn get_session(
        &self,
        app_name: &str,
        user_id: &UserId,
        session_id: &SessionId,
    ) -> Result<Option<Session>>;

    /// Update session state
    async fn update_session_state(
        &self,
        session_id: &SessionId,
        state: &SessionState,
    ) -> Result<()>;

    /// Append an event to a session
    async fn append_event(&self, session_id: &SessionId, event: Event) -> Result<()>;
}

/// In-memory session service implementation
#[derive(Debug)]
pub struct InMemorySessionService {
    sessions: Arc<RwLock<HashMap<SessionId, Session>>>,
}

impl InMemorySessionService {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl Default for InMemorySessionService {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SessionService for InMemorySessionService {
    async fn get_session(
        &self,
        _app_name: &str,
        _user_id: &UserId,
        session_id: &SessionId,
    ) -> Result<Option<Session>> {
        let sessions = self.sessions.read().await;
        Ok(sessions.get(session_id).cloned())
    }

    async fn update_session_state(
        &self,
        session_id: &SessionId,
        state: &SessionState,
    ) -> Result<()> {
        let mut sessions = self.sessions.write().await;
        if let Some(session) = sessions.get_mut(session_id) {
            session.state = state.clone();
            session.updated_at = chrono::Utc::now();
        }
        Ok(())
    }

    async fn append_event(&self, session_id: &SessionId, event: Event) -> Result<()> {
        let mut sessions = self.sessions.write().await;
        if let Some(session) = sessions.get_mut(session_id) {
            session.add_event(event);
        }
        Ok(())
    }
}
