//! Chronos Agent Aggregate Root
//!
//! Created by Joaquín Escalante (https://github.com/joaquinescalante23)

use crate::domain::budget::Budget;
use crate::domain::error::{DomainError, DomainResult};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// The possible states of an agent process.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentState {
    Pending,
    Running,
    Suspended,
    Terminated,
    Error,
}

/// The core domain entity representing an autonomous agent process.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agent {
    /// Unique system identifier for the agent process.
    pub id: Uuid,
    /// Human-readable name for the agent.
    pub name: String,
    /// The specific LLM model this agent is configured to use.
    pub model_id: String,
    /// The current operational state of the agent.
    pub state: AgentState,
    /// Resource and cost management for the agent.
    pub budget: Budget,
    /// Timestamp when the agent was first spawned.
    pub created_at: DateTime<Utc>,
    /// Last time the agent's state was modified.
    pub updated_at: DateTime<Utc>,
}

impl Agent {
    /// Spawns a new agent with a defined budget.
    pub fn new(name: String, model_id: String, max_cost: f64, max_tokens: u32) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name,
            model_id,
            state: AgentState::Pending,
            budget: Budget::new(max_cost, max_tokens),
            created_at: now,
            updated_at: now,
        }
    }

    /// Transitions the agent to the Running state.
    pub fn start(&mut self) -> DomainResult<()> {
        match self.state {
            AgentState::Pending | AgentState::Suspended => {
                self.transition_to(AgentState::Running);
                Ok(())
            }
            _ => Err(DomainError::InvalidStateTransition(
                self.id,
                format!("Cannot start agent from {:?} state", self.state),
            ))
        }
    }

    /// Transitions the agent to the Suspended state (Pausing execution).
    pub fn pause(&mut self) -> DomainResult<()> {
        if self.state == AgentState::Running {
            self.transition_to(AgentState::Suspended);
            Ok(())
        } else {
            Err(DomainError::InvalidStateTransition(
                self.id,
                format!("Only running agents can be paused (current: {:?})", self.state),
            ))
        }
    }

    /// Terminates the agent process permanently.
    pub fn stop(&mut self) {
        self.transition_to(AgentState::Terminated);
    }

    /// Marks the agent as being in an error state.
    pub fn fail(&mut self) {
        self.transition_to(AgentState::Error);
    }

    /// Internal helper to manage state transitions and timestamps.
    fn transition_to(&mut self, new_state: AgentState) {
        self.state = new_state;
        self.updated_at = Utc::now();
    }
}

// Created by Joaquín Escalante (https://github.com/joaquinescalante23)
