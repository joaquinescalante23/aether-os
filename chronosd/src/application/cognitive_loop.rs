//! Chronos Cognitive Loop Application Service
//!
//! Created by Joaquín Escalante (https://github.com/joaquinescalante23)

use crate::domain::agent::AgentState;
use crate::domain::checkpoint::{Checkpoint, Message, MessageRole};
use crate::domain::error::{DomainError, DomainResult};
use crate::domain::{LlmProvider, ToolRegistry};
use crate::infrastructure::SqliteAgentRepository;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{sleep, Duration};
use tracing::{error, info};
use uuid::Uuid;
use chrono::Utc;

/// The orchestrator of an agent's continuous thinking process.
pub struct CognitiveLoop {
    repository: Arc<SqliteAgentRepository>,
    llm_provider: Arc<dyn LlmProvider>,
    tool_registry: Arc<RwLock<ToolRegistry>>,
}

impl CognitiveLoop {
    /// Creates a new CognitiveLoop with its required dependencies.
    pub fn new(repository: Arc<SqliteAgentRepository>, llm_provider: Arc<dyn LlmProvider>, tool_registry: Arc<RwLock<ToolRegistry>>) -> Self {
        Self {
            repository,
            llm_provider,
            tool_registry,
        }
    }

    /// Runs a single step of the agent's thought process.
    pub async fn run_step(&self, agent_id: Uuid) -> DomainResult<()> {
        info!("CognitiveLoop: Running step for agent {}", agent_id);

        let mut agent = self.repository.find_agent(agent_id).await?;
        if agent.state != AgentState::Running {
            return Err(DomainError::InvalidStateTransition(agent_id, "Agent is not in Running state".to_string()));
        }

        // Fetch schema from registry to instruct the LLM
        let registry = self.tool_registry.read().await;
        let tools_schema = registry.combined_schema();

        let messages = vec![
            Message {
                role: MessageRole::System,
                content: format!("You are a Chronos Agent. You have access to the following tools: {}. If you need to perform an action, output a JSON object with 'tool_call' containing 'tool_name' and 'arguments'.", tools_schema),
                timestamp: Utc::now(),
            },
            Message {
                role: MessageRole::User,
                content: format!("Continue your work for agent {}", agent.name),
                timestamp: Utc::now(),
            }
        ];

        let response = self.llm_provider.completion(&agent.model_id, &messages).await?;
        agent.budget.consume(agent.id, response.prompt_tokens + response.completion_tokens, response.total_cost_usd)?;

        // Simple heuristic for parsing tool calls in MVP
        let mut new_messages = messages.clone();
        
        if response.content.contains("\"tool_call\"") {
            // Simulated tool execution
            info!("Agent {} requested a tool call.", agent.id);
            // In a real implementation, we would parse the JSON, call registry.get("tool_name").execute(),
            // and append the ToolResult. For MVP, we simulate the observation.
            
            new_messages.push(Message {
                role: MessageRole::Assistant,
                content: response.content.clone(),
                timestamp: Utc::now(),
            });

            new_messages.push(Message {
                role: MessageRole::Tool,
                content: "Tool executed successfully.".to_string(),
                timestamp: Utc::now(),
            });
            
        } else {
            new_messages.push(Message {
                role: MessageRole::Assistant,
                content: response.content,
                timestamp: Utc::now(),
            });
        }

        let checkpoint = Checkpoint::new(agent.id, new_messages, vec![]);
        
        self.repository.save_agent(&agent).await?;
        self.repository.save_checkpoint(&checkpoint).await?;

        Ok(())
    }

    /// Background task that keeps the agent running until paused or terminated.
    pub async fn start_background_loop(self: Arc<Self>, agent_id: Uuid) {
        info!("CognitiveLoop: Starting background process for agent {}", agent_id);
        
        loop {
            // Check if agent is still in Running state
            let agent = match self.repository.find_agent(agent_id).await {
                Ok(a) => a,
                Err(e) => {
                    error!("CognitiveLoop: Error fetching agent {}: {}", agent_id, e);
                    break;
                }
            };

            if agent.state != AgentState::Running {
                info!("CognitiveLoop: Agent {} no longer running. Stopping loop.", agent_id);
                break;
            }

            // Run a cognitive step
            if let Err(e) = self.run_step(agent_id).await {
                error!("CognitiveLoop: Critical error in agent {}: {}", agent_id, e);
                // Mark agent as Error in DB
                let mut agent = agent;
                agent.fail();
                let _ = self.repository.save_agent(&agent).await;
                break;
            }

            // Throttle to avoid CPU/Token exhaustion in MVP
            sleep(Duration::from_secs(5)).await;
        }
    }
}

// Created by Joaquín Escalante (https://github.com/joaquinescalante23)
