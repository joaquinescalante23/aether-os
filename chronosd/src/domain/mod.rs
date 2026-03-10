//! Chronos Domain Layer
//!
//! Created by Joaquín Escalante (https://github.com/joaquinescalante23)

pub mod agent;
pub mod audit;
pub mod budget;
pub mod checkpoint;
pub mod error;
pub mod identity;
pub mod llm;
pub mod mission;
pub mod tool;

#[cfg(test)]
mod tests;

pub use agent::{Agent, AgentState};
pub use audit::{AuditReport, AuditVerdict};
pub use budget::Budget;
pub use checkpoint::{Checkpoint, Message, MessageRole, ToolState};
pub use error::{DomainError, DomainResult};
pub use identity::Identity;
pub use llm::{LlmProvider, LlmResponse};
pub use mission::{Mission, MissionStatus};
pub use tool::{Tool, ToolCallRequest, ToolCallResult, ToolRegistry};

// Created by Joaquín Escalante (https://github.com/joaquinescalante23)
