//! Chronos Shadow Service (Deep-Heal Architecture)
//!
//! Created by Joaquín Escalante (https://github.com/joaquinescalante23)

use crate::domain::audit::{AuditReport, AuditVerdict};
use crate::domain::checkpoint::{Message, MessageRole};
use crate::domain::error::DomainResult;
use crate::domain::identity::Identity;
use crate::domain::LlmProvider;
use crate::infrastructure::SqliteAgentRepository;
use std::sync::Arc;
use tracing::{info, warn};
use uuid::Uuid;

/// Orchestrates the Shadow Audit and Cognitive Burst processes.
pub struct ShadowService {
    repository: Arc<SqliteAgentRepository>,
    llm_provider: Arc<dyn LlmProvider>,
}

impl ShadowService {
    /// Creates a new ShadowService.
    pub fn new(repository: Arc<SqliteAgentRepository>, llm_provider: Arc<dyn LlmProvider>) -> Self {
        Self {
            repository,
            llm_provider,
        }
    }

    /// Performs an audit on an agent's latest execution.
    pub async fn audit_execution(
        &self,
        worker_agent_id: Uuid,
        checkpoint_id: Uuid,
        execution_result: &str,
    ) -> DomainResult<AuditReport> {
        info!("ShadowService: Auditing execution for agent {}", worker_agent_id);

        let auditor_identity = Identity::new(
            "Kernel Shadow Auditor".to_string(),
            "You are the internal AetherOS Kernel Auditor. Your job is to verify the execution result of a worker agent. \
            If the result contains errors (like compilation failures or logical flaws), you must fail the audit and provide a fix. \
            Respond ONLY with a JSON object containing: 'verdict' (Pass/Fail), 'error_log' (string, empty if pass), and 'suggested_fix' (string, empty if pass).".to_string(),
        );

        let messages = vec![
            Message {
                role: MessageRole::System,
                content: auditor_identity.boot_prompt(),
                timestamp: chrono::Utc::now(),
            },
            Message {
                role: MessageRole::User,
                content: format!("Verify this execution result:\n{}", execution_result),
                timestamp: chrono::Utc::now(),
            },
        ];

        // We use the same LLM provider for the audit, but a dedicated model could be used here (Burst)
        let response = self.llm_provider.completion("gpt-4o", &messages).await?;

        // Simple heuristic parser for MVP (In a real scenario, use structured output parsing)
        let verdict = if response.content.to_lowercase().contains("\"verdict\": \"pass\"") {
            AuditVerdict::Pass
        } else {
            AuditVerdict::Fail {
                error_log: "Audit detected anomalies in the execution.".to_string(),
                suggested_fix: "Review the system logs and correct the syntax/logic.".to_string(),
                severity: 3,
            }
        };

        let report = AuditReport::new(worker_agent_id, checkpoint_id, verdict);
        
        // TODO: Persist the audit report in the Datahive

        Ok(report)
    }

    /// Initiates a Time-Travel rewind for an agent to a specific checkpoint.
    pub async fn rewind_agent(&self, agent_id: Uuid, _checkpoint_id: Uuid, fix_instruction: &str) -> DomainResult<()> {
        warn!("ShadowService: Triggering Rewind for agent {}", agent_id);
        
        let mut agent = self.repository.find_agent(agent_id).await?;
        
        // In a full implementation, we would restore the exact state from `checkpoint_id`.
        // For this version, we inject the correction impulse into the current state.
        
        let mut latest_checkpoint = self.repository.get_latest_checkpoint(agent_id).await?;
        
        latest_checkpoint.messages.push(Message {
            role: MessageRole::System,
            content: format!("[KERNEL DEEP-HEAL IMPULSE]\nYour last action was rejected by the Shadow Auditor. Correct your path using this instruction:\n{}", fix_instruction),
            timestamp: chrono::Utc::now(),
        });

        self.repository.save_checkpoint(&latest_checkpoint).await?;
        
        // Pause the agent so the Orchestrator/CognitiveLoop can re-evaluate
        agent.pause()?;
        self.repository.save_agent(&agent).await?;

        Ok(())
    }
}

// Created by Joaquín Escalante (https://github.com/joaquinescalante23)
