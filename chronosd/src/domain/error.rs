//! Chronos Domain Error Types
//!
//! Created by Joaquín Escalante (https://github.com/joaquinescalante23)

use thiserror::Error;
use uuid::Uuid;

/// Core domain errors for the Chronos Kernel.
#[derive(Debug, Error)]
pub enum DomainError {
    /// Attempted to interact with an agent that does not exist.
    #[error("Agent with ID {0} not found")]
    AgentNotFound(Uuid),

    /// Attempted an invalid state transition (e.g., Resume a Running agent).
    #[error("Invalid state transition for agent {0}: {1}")]
    InvalidStateTransition(Uuid, String),

    /// The agent has exceeded its allocated token or cost budget.
    #[error("Agent {0} has exceeded its budget: {1}")]
    BudgetExceeded(Uuid, String),

    /// An error occurred during state serialization or deserialization.
    #[error("State persistence error for agent {0}: {1}")]
    PersistenceError(Uuid, String),

    /// General internal domain error.
    #[error("Internal domain error: {0}")]
    Internal(String),
}

/// A specialized Result type for Chronos domain operations.
pub type DomainResult<T> = Result<T, DomainError>;

// Created by Joaquín Escalante (https://github.com/joaquinescalante23)
