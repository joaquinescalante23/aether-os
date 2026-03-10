//! Chronos Multi-Agent Orchestrator
//!
//! Created by Joaquín Escalante (https://github.com/joaquinescalante23)

use crate::domain::identity::Identity;
use crate::domain::mission::{Mission, MissionStatus};
use crate::domain::agent::Agent;
use crate::infrastructure::SqliteAgentRepository;
use crate::application::CognitiveLoop;
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

/// Orchestrates multiple specialized agents to complete a complex mission.
pub struct Orchestrator {
    repository: Arc<SqliteAgentRepository>,
    cognitive_loop: Arc<CognitiveLoop>,
}

impl Orchestrator {
    pub fn new(repository: Arc<SqliteAgentRepository>, cognitive_loop: Arc<CognitiveLoop>) -> Self {
        Self { repository, cognitive_loop }
    }

    /// Spawns a standard "Development Squad" for a programming mission.
    pub async fn start_dev_mission(&self, title: String, goal: String) -> Result<Uuid, Box<dyn std::error::Error>> {
        info!("Orchestrator: Starting Dev Mission - {}", title);

        let mut mission = Mission::new(title, goal);

        // 1. Define Identities (Cognitive Images)
        let architect_id = Identity::new(
            "Lead Architect".to_string(),
            "You are the Lead Architect. Break down the user's goal into small, actionable coding tasks.".to_string()
        );

        let coder_id = Identity::new(
            "Senior Developer".to_string(),
            "You are a Senior Developer. Implement the tasks assigned by the Architect with high-quality code.".to_string()
        );

        // 2. Spawn Agent Processes
        let architect_agent = Agent::new(
            format!("{}-Architect", mission.title),
            "gpt-4".to_string(),
            5.0, // USD Budget
            50000 // Token Budget
        );

        let coder_agent = Agent::new(
            format!("{}-Coder", mission.title),
            "gpt-4".to_string(),
            10.0,
            100000
        );

        // 3. Persist and Track
        self.repository.save_agent(&architect_agent).await?;
        self.repository.save_agent(&coder_agent).await?;

        mission.add_agent(architect_agent.id);
        mission.add_agent(coder_agent.id);
        mission.status = MissionStatus::Active;

        // 4. Launch Cognitive Loops
        let loop_handle = Arc::clone(&self.cognitive_loop);
        
        let arch_id = architect_agent.id;
        tokio::spawn(async move {
            loop_handle.start_background_loop(arch_id).await;
        });

        info!("Orchestrator: Mission {} started with {} agents.", mission.id, mission.agent_ids.len());
        Ok(mission.id)
    }
}

// Created by Joaquín Escalante (https://github.com/joaquinescalante23)
