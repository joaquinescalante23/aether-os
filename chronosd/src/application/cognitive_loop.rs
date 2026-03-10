//! Chronos Cognitive Loop Application Service
//!
//! Created by Joaquín Escalante (https://github.com/joaquinescalante23)

use crate::domain::agent::AgentState;
use crate::domain::checkpoint::{Checkpoint, Message, MessageRole};
use crate::domain::error::DomainResult;
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

    /// Runs a single step of the agent's thought process. Returns true if active work was done.
    pub async fn run_step(&self, agent_id: Uuid) -> DomainResult<bool> {
        let mut agent = self.repository.find_agent(agent_id).await?;
        if agent.state != AgentState::Running {
            return Ok(false);
        }

        // 1. Fetch History or create Genesis Checkpoint
        let mut checkpoint = match self.repository.get_latest_checkpoint(agent_id).await {
            Ok(cp) => cp,
            Err(_) => {
                info!("CognitiveLoop: Initializing Genesis Checkpoint for {}", agent.id);
                let registry = self.tool_registry.read().await;
                let tools_schema = registry.combined_schema();
                let initial_messages = vec![
                    Message {
                        role: MessageRole::System,
                        content: format!("You are AetherOS Agent '{}'.\n\nYou have access to the following tools:\n{}\n\nTo execute a tool, you MUST reply with a JSON block in this exact format:\n```json\n{{\"tool_call\": {{\"tool_name\": \"name\", \"arguments\": {{\"key\": \"value\"}}}}}}\n```\nIf you do not need to use a tool, just reply normally.", agent.name, tools_schema),
                        timestamp: Utc::now(),
                    },
                    Message {
                        role: MessageRole::User,
                        content: "System boot complete. Awaiting instructions.".to_string(),
                        timestamp: Utc::now(),
                    }
                ];
                let cp = Checkpoint::new(agent.id, initial_messages, vec![]);
                self.repository.save_checkpoint(&cp).await?;
                cp
            }
        };

        // 2. Idle Check: If the last message was from the assistant and wasn't a tool call, wait for user input.
        if let Some(last_msg) = checkpoint.messages.last() {
            if last_msg.role == MessageRole::Assistant {
                return Ok(false); // Idle
            }
        }

        info!("CognitiveLoop: Processing thought for agent {}", agent_id);

        // 3. Call LLM with full context
        let response = self.llm_provider.completion(&agent.model_id, &checkpoint.messages).await?;
        agent.budget.consume(agent.id, response.prompt_tokens + response.completion_tokens, response.total_cost_usd)?;

        let mut executed_tool = false;
        let content = response.content.clone();
        
        checkpoint.messages.push(Message {
            role: MessageRole::Assistant,
            content: content.clone(),
            timestamp: Utc::now(),
        });

        // 4. Robust JSON Parsing for Tool Calls
        let json_str = if let Some(start) = content.find("```json") {
            if let Some(end) = content[start + 7..].find("```") {
                &content[start + 7..start + 7 + end]
            } else {
                &content
            }
        } else if let Some(start) = content.find('{') {
             if let Some(end) = content.rfind('}') {
                 &content[start..=end]
             } else {
                 &content
             }
        } else {
            &content
        };

        if let Ok(json_resp) = serde_json::from_str::<serde_json::Value>(json_str) {
            if let Some(tool_call) = json_resp.get("tool_call") {
                if let (Some(tool_name), Some(args)) = (tool_call.get("tool_name").and_then(|v| v.as_str()), tool_call.get("arguments")) {
                    info!("Agent {} executing tool: {}", agent.id, tool_name);
                    
                    let registry = self.tool_registry.read().await;
                    let result_msg = match registry.get(tool_name) {
                        Some(tool) => match tool.execute(args.clone()).await {
                            Ok(res) => {
                                if res.is_error {
                                    format!("Tool Execution Failed:\n{}", res.result)
                                } else {
                                    format!("Tool Execution Success:\n{}", res.result)
                                }
                            }
                            Err(e) => format!("Kernel Error executing tool: {}", e),
                        },
                        None => format!("Error: Tool '{}' not found in registry.", tool_name),
                    };

                    checkpoint.messages.push(Message {
                        role: MessageRole::Tool,
                        content: result_msg,
                        timestamp: Utc::now(),
                    });
                    
                    executed_tool = true;
                }
            }
        }

        // 5. Persist State
        let new_checkpoint = Checkpoint::new(agent.id, checkpoint.messages, vec![]);
        self.repository.save_agent(&agent).await?;
        self.repository.save_checkpoint(&new_checkpoint).await?;

        Ok(executed_tool)
    }

    /// Background task that keeps the agent running until paused or terminated.
    pub async fn start_background_loop(self: Arc<Self>, agent_id: Uuid) {
        info!("CognitiveLoop: Starting background process for agent {}", agent_id);
        
        loop {
            let agent = match self.repository.find_agent(agent_id).await {
                Ok(a) => a,
                Err(e) => {
                    error!("CognitiveLoop: Error fetching agent {}: {}", agent_id, e);
                    break;
                }
            };

            if agent.state != AgentState::Running {
                break;
            }

            let did_work = match self.run_step(agent_id).await {
                Ok(work) => work,
                Err(e) => {
                    error!("CognitiveLoop: Critical error in agent {}: {}", agent_id, e);
                    let mut agent = agent;
                    agent.fail();
                    let _ = self.repository.save_agent(&agent).await;
                    break;
                }
            };

            if did_work {
                // If a tool was executed, immediately process the next thought
                tokio::task::yield_now().await;
            } else {
                // If idle or waiting for user input, sleep to save CPU
                sleep(Duration::from_millis(1000)).await;
            }
        }
        
        info!("CognitiveLoop: Halted for agent {}", agent_id);
    }
}

// Created by Joaquín Escalante (https://github.com/joaquinescalante23)
