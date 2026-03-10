//! Chronos Cognitive Checkpointing Domain Model
//!
//! Created by Joaquín Escalante (https://github.com/joaquinescalante23)

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// A role in a conversation (System, User, Assistant, Tool).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum MessageRole {
    System,
    User,
    Assistant,
    Tool,
}

/// A single message in the agent's context window.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: MessageRole,
    pub content: String,
    pub timestamp: DateTime<Utc>,
}

/// Represents the state of a tool at checkpoint time.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolState {
    pub name: String,
    pub status: String,
}

/// A complete snapshot of an agent's cognitive state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Checkpoint {
    /// Unique identifier for this checkpoint.
    pub id: Uuid,
    /// Reference to the agent this checkpoint belongs to.
    pub agent_id: Uuid,
    /// The conversation history at this moment.
    pub messages: Vec<Message>,
    /// State of all tools connected to the agent.
    pub tools: Vec<ToolState>,
    /// When this checkpoint was created.
    pub created_at: DateTime<Utc>,
}

impl Checkpoint {
    /// Creates a new checkpoint for an agent.
    pub fn new(agent_id: Uuid, messages: Vec<Message>, tools: Vec<ToolState>) -> Self {
        Self {
            id: Uuid::new_v4(),
            agent_id,
            messages,
            tools,
            created_at: Utc::now(),
        }
    }
}

// Created by Joaquín Escalante (https://github.com/joaquinescalante23)
