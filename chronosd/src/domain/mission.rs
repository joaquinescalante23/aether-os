//! Chronos Multi-Agent Mission Domain
//!
//! Created by Joaquín Escalante (https://github.com/joaquinescalante23)

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::collections::HashSet;

/// Status of a multi-agent mission.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MissionStatus {
    Queued,
    Active,
    Completed,
    Failed,
}

/// Represents a complex task requiring multiple specialized agents.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mission {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub status: MissionStatus,
    /// List of agent IDs participating in this mission.
    pub agent_ids: HashSet<Uuid>,
    /// The global "Shared Memory" ID for this mission.
    pub shared_context_id: Uuid,
}

impl Mission {
    pub fn new(title: String, description: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            title,
            description,
            status: MissionStatus::Queued,
            agent_ids: HashSet::new(),
            shared_context_id: Uuid::new_v4(),
        }
    }

    pub fn add_agent(&mut self, agent_id: Uuid) {
        self.agent_ids.insert(agent_id);
    }
}

// Created by Joaquín Escalante (https://github.com/joaquinescalante23)
