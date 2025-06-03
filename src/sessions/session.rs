//! Session management

use crate::{
    events::Event,
    types::{SessionId, SessionState, Timestamp, UserId},
};
use chrono::Utc;
use serde::{Deserialize, Serialize};

/// Session for managing conversation state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    /// Unique session identifier
    pub id: SessionId,
    
    /// User identifier
    pub user_id: UserId,
    
    /// Application name
    pub app_name: String,
    
    /// Session state
    pub state: SessionState,
    
    /// Events in this session
    pub events: Vec<Event>,
    
    /// When the session was created
    pub created_at: Timestamp,
    
    /// When the session was last updated
    pub updated_at: Timestamp,
}

impl Session {
    /// Create a new session
    pub fn new(app_name: String, user_id: UserId, session_id: SessionId) -> Self {
        let now = Utc::now();
        Self {
            id: session_id,
            user_id,
            app_name,
            state: SessionState::new(),
            events: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }

    /// Add an event to the session
    pub fn add_event(&mut self, event: Event) {
        self.events.push(event);
        self.updated_at = Utc::now();
    }
}
