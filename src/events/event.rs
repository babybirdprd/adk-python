//! Event types for agent communication

use crate::types::{Content, InvocationId, StateDelta, Timestamp};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Event generated during agent execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    /// Unique identifier for this event
    pub id: String,
    
    /// The agent that generated this event
    pub author: String,
    
    /// Content of the event (text, images, etc.)
    pub content: Option<Content>,
    
    /// Actions to be performed as a result of this event
    pub actions: EventAction,
    
    /// Timestamp when the event was created
    pub timestamp: Timestamp,
    
    /// Invocation ID this event belongs to
    pub invocation_id: InvocationId,
    
    /// Whether this is a partial response (streaming)
    pub is_partial: bool,
    
    /// Metadata associated with the event
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Actions that can be performed as a result of an event
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EventAction {
    /// Changes to apply to the session state
    pub state_delta: StateDelta,
    
    /// Whether to escalate to parent agent
    pub escalate: bool,
    
    /// Whether to transfer to another agent
    pub transfer_to: Option<String>,
    
    /// Whether to end the current conversation
    pub end_conversation: bool,
    
    /// Custom actions
    pub custom: HashMap<String, serde_json::Value>,
}

impl Event {
    /// Create a text response event
    pub fn text_response(
        author: impl Into<String>,
        text: impl Into<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            author: author.into(),
            content: Some(Content::model_text(text)),
            actions: EventAction::default(),
            timestamp: Utc::now(),
            invocation_id: Uuid::new_v4(),
            is_partial: false,
            metadata: HashMap::new(),
        }
    }

    /// Create a user input event
    pub fn user_input(
        text: impl Into<String>,
        invocation_id: InvocationId,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            author: "user".to_string(),
            content: Some(Content::user_text(text)),
            actions: EventAction::default(),
            timestamp: Utc::now(),
            invocation_id,
            is_partial: false,
            metadata: HashMap::new(),
        }
    }
}

/// Builder for creating events
pub struct EventBuilder {
    event: Event,
}

impl EventBuilder {
    pub fn new(author: impl Into<String>, invocation_id: InvocationId) -> Self {
        Self {
            event: Event {
                id: Uuid::new_v4().to_string(),
                author: author.into(),
                content: None,
                actions: EventAction::default(),
                timestamp: Utc::now(),
                invocation_id,
                is_partial: false,
                metadata: HashMap::new(),
            },
        }
    }

    pub fn build(self) -> Event {
        self.event
    }
}
