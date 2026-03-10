//! Chronos SQLite Agent Repository
//!
//! Created by Joaquín Escalante (https://github.com/joaquinescalante23)

use crate::domain::agent::{Agent, AgentState};
use crate::domain::budget::Budget;
use crate::domain::checkpoint::Checkpoint;
use crate::domain::error::{DomainError, DomainResult};
use sqlx::{Pool, Sqlite, Row};
use uuid::Uuid;

/// Concrete implementation of an Agent Repository using SQLite and sqlx.
pub struct SqliteAgentRepository {
    pool: Pool<Sqlite>,
}

impl SqliteAgentRepository {
    /// Creates a new repository with an existing connection pool.
    pub fn new(pool: Pool<Sqlite>) -> Self {
        Self { pool }
    }

    /// Persists a new agent to the database.
    pub async fn save_agent(&self, agent: &Agent) -> DomainResult<()> {
        sqlx::query(
            r#"
            INSERT INTO agents (id, name, model_id, state, max_cost_usd, max_tokens, current_cost_usd, current_tokens, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(id) DO UPDATE SET
                state = excluded.state,
                current_cost_usd = excluded.current_cost_usd,
                current_tokens = excluded.current_tokens,
                updated_at = excluded.updated_at
            "#
        )
        .bind(agent.id.to_string())
        .bind(&agent.name)
        .bind(&agent.model_id)
        .bind(format!("{:?}", agent.state))
        .bind(agent.budget.max_cost_usd)
        .bind(agent.budget.max_tokens as i64)
        .bind(agent.budget.current_cost_usd)
        .bind(agent.budget.current_tokens as i64)
        .bind(agent.created_at)
        .bind(agent.updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::PersistenceError(agent.id, e.to_string()))?;

        Ok(())
    }

    /// Retrieves an agent by its unique identifier.
    pub async fn find_agent(&self, id: Uuid) -> DomainResult<Agent> {
        let row = sqlx::query(r#"SELECT * FROM agents WHERE id = ?"#)
            .bind(id.to_string())
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| DomainError::PersistenceError(id, e.to_string()))?
            .ok_or(DomainError::AgentNotFound(id))?;

        let state_str: String = row.get("state");
        let state = match state_str.as_str() {
            "Pending" => AgentState::Pending,
            "Running" => AgentState::Running,
            "Suspended" => AgentState::Suspended,
            "Terminated" => AgentState::Terminated,
            "Error" => AgentState::Error,
            _ => return Err(DomainError::Internal(format!("Unknown agent state: {}", state_str))),
        };

        Ok(Agent {
            id: Uuid::parse_str(row.get("id")).unwrap_or_default(),
            name: row.get("name"),
            model_id: row.get("model_id"),
            state,
            budget: Budget {
                max_cost_usd: row.get("max_cost_usd"),
                max_tokens: row.get::<i64, _>("max_tokens") as u32,
                current_cost_usd: row.get("current_cost_usd"),
                current_tokens: row.get::<i64, _>("current_tokens") as u32,
            },
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        })
    }

    /// Fetches all managed agents.
    pub async fn list_agents(&self) -> DomainResult<Vec<Agent>> {
        let rows = sqlx::query(r#"SELECT * FROM agents ORDER BY created_at DESC"#)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        let mut agents = Vec::new();
        for row in rows {
            let state_str: String = row.get("state");
            let state = match state_str.as_str() {
                "Pending" => AgentState::Pending,
                "Running" => AgentState::Running,
                "Suspended" => AgentState::Suspended,
                "Terminated" => AgentState::Terminated,
                "Error" => AgentState::Error,
                _ => continue,
            };

            agents.push(Agent {
                id: Uuid::parse_str(row.get("id")).unwrap_or_default(),
                name: row.get("name"),
                model_id: row.get("model_id"),
                state,
                budget: Budget {
                    max_cost_usd: row.get("max_cost_usd"),
                    max_tokens: row.get::<i64, _>("max_tokens") as u32,
                    current_cost_usd: row.get("current_cost_usd"),
                    current_tokens: row.get::<i64, _>("current_tokens") as u32,
                },
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            });
        }
        Ok(agents)
    }

    /// Retrieves the most recent cognitive checkpoint for an agent.
    pub async fn get_latest_checkpoint(&self, agent_id: Uuid) -> DomainResult<Checkpoint> {
        let row = sqlx::query(r#"SELECT * FROM checkpoints WHERE agent_id = ? ORDER BY created_at DESC LIMIT 1"#)
            .bind(agent_id.to_string())
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| DomainError::PersistenceError(agent_id, e.to_string()))?
            .ok_or_else(|| DomainError::Internal(format!("No checkpoints found for agent {}", agent_id)))?;

        let messages_json: String = row.get("messages_json");
        let tools_json: String = row.get("tools_json");

        let messages = serde_json::from_str(&messages_json)
            .map_err(|e| DomainError::Internal(format!("Failed to parse messages: {}", e)))?;
        let tools = serde_json::from_str(&tools_json)
            .map_err(|e| DomainError::Internal(format!("Failed to parse tools: {}", e)))?;

        Ok(Checkpoint {
            id: Uuid::parse_str(row.get("id")).unwrap_or_default(),
            agent_id,
            messages,
            tools,
            created_at: row.get("created_at"),
        })
    }

    /// Persists a new cognitive checkpoint (Thought Snapshot).
    pub async fn save_checkpoint(&self, checkpoint: &Checkpoint) -> DomainResult<()> {
        let messages_json = serde_json::to_string(&checkpoint.messages)
            .map_err(|e| DomainError::PersistenceError(checkpoint.agent_id, e.to_string()))?;
        let tools_json = serde_json::to_string(&checkpoint.tools)
            .map_err(|e| DomainError::PersistenceError(checkpoint.agent_id, e.to_string()))?;

        sqlx::query(
            r#"
            INSERT INTO checkpoints (id, agent_id, messages_json, tools_json, created_at)
            VALUES (?, ?, ?, ?, ?)
            "#
        )
        .bind(checkpoint.id.to_string())
        .bind(checkpoint.agent_id.to_string())
        .bind(messages_json)
        .bind(tools_json)
        .bind(checkpoint.created_at)
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::PersistenceError(checkpoint.agent_id, e.to_string()))?;

        Ok(())
    }
}

// Created by Joaquín Escalante (https://github.com/joaquinescalante23)
