//! Chronos gRPC Server Implementation
//!
//! Created by Joaquín Escalante (https://github.com/joaquinescalante23)

use crate::application::CognitiveLoop;
use crate::domain::agent::{Agent, AgentState as DomainAgentState};
use crate::infrastructure::SqliteAgentRepository;
use std::sync::Arc;
use tonic::{Request, Response, Status};
use uuid::Uuid;

// Include the generated gRPC types
pub mod chronos_proto {
    tonic::include_proto!("chronos");
}

use chronos_proto::chronos_kernel_server::ChronosKernel;
use chronos_proto::*;

/// The concrete implementation of the Chronos gRPC Service.
pub struct ChronosServer {
    repository: Arc<SqliteAgentRepository>,
    cognitive_loop: Arc<CognitiveLoop>,
}

impl ChronosServer {
    /// Creates a new gRPC server with the given dependencies.
    pub fn new(repository: Arc<SqliteAgentRepository>, cognitive_loop: Arc<CognitiveLoop>) -> Self {
        Self {
            repository,
            cognitive_loop,
        }
    }
}

#[tonic::async_trait]
impl ChronosKernel for ChronosServer {
    /// Spawns a new agent process in the kernel.
    async fn spawn_agent(
        &self,
        request: Request<SpawnAgentRequest>,
    ) -> Result<Response<SpawnAgentResponse>, Status> {
        let req = request.into_inner();
        let budget = req.budget.unwrap_or(Budget {
            max_cost_usd: 1.0,
            max_tokens: 10000,
        });

        // 1. Create domain agent in Running state
        let mut agent = Agent::new(
            req.name,
            req.model_id,
            budget.max_cost_usd,
            budget.max_tokens,
        );
        agent.start().map_err(|e| Status::internal(e.to_string()))?;

        // 2. Persist to database
        self.repository
            .save_agent(&agent)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        // 3. Start background cognitive loop
        let loop_handle = Arc::clone(&self.cognitive_loop);
        let agent_id = agent.id;
        tokio::spawn(async move {
            loop_handle.start_background_loop(agent_id).await;
        });

        Ok(Response::new(SpawnAgentResponse {
            agent_id: agent.id.to_string(),
            state: AgentState::Running as i32,
            created_at: Some(prost_types::Timestamp {
                seconds: agent.created_at.timestamp(),
                nanos: agent.created_at.timestamp_subsec_nanos() as i32,
            }),
        }))
    }

    /// Lists all managed agents.
    async fn list_agents(
        &self,
        _request: Request<ListAgentsRequest>,
    ) -> Result<Response<ListAgentsResponse>, Status> {
        let agents = self.repository
            .list_agents()
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        let summaries = agents
            .into_iter()
            .map(|a| AgentSummary {
                agent_id: a.id.to_string(),
                name: a.name,
                state: match a.state {
                    DomainAgentState::Pending => AgentState::Pending as i32,
                    DomainAgentState::Running => AgentState::Running as i32,
                    DomainAgentState::Suspended => AgentState::Suspended as i32,
                    DomainAgentState::Terminated => AgentState::Terminated as i32,
                    DomainAgentState::Error => AgentState::Error as i32,
                },
                stats: Some(Stats {
                    tokens_consumed: a.budget.current_tokens,
                    cost_usd: a.budget.current_cost_usd,
                    step_count: 0, 
                }),
            })
            .collect();

        Ok(Response::new(ListAgentsResponse { agents: summaries }))
    }

    /// Stops an agent gracefully.
    async fn stop_agent(
        &self,
        request: Request<StopAgentRequest>,
    ) -> Result<Response<StopAgentResponse>, Status> {
        let agent_id = Uuid::parse_str(&request.get_ref().agent_id)
            .map_err(|_| Status::invalid_argument("Invalid Agent ID"))?;

        let mut agent = self.repository.find_agent(agent_id)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        agent.stop();
        self.repository.save_agent(&agent)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(StopAgentResponse {
            agent_id: agent.id.to_string(),
            final_stats: Some(Stats {
                tokens_consumed: agent.budget.current_tokens,
                cost_usd: agent.budget.current_cost_usd,
                step_count: 0,
            }),
        }))
    }

    /// Pauses an agent process.
    async fn pause_agent(
        &self,
        request: Request<PauseAgentRequest>,
    ) -> Result<Response<PauseAgentResponse>, Status> {
        let agent_id = Uuid::parse_str(&request.get_ref().agent_id)
            .map_err(|_| Status::invalid_argument("Invalid Agent ID"))?;

        let mut agent = self.repository.find_agent(agent_id)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        agent.pause().map_err(|e| Status::internal(e.to_string()))?;
        self.repository.save_agent(&agent)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(PauseAgentResponse {
            checkpoint_id: "latest".to_string(),
            paused_at: Some(prost_types::Timestamp {
                seconds: agent.updated_at.timestamp(),
                nanos: agent.updated_at.timestamp_subsec_nanos() as i32,
            }),
        }))
    }

    /// Resumes an agent from its last checkpoint.
    async fn resume_agent(
        &self,
        request: Request<ResumeAgentRequest>,
    ) -> Result<Response<ResumeAgentResponse>, Status> {
        let agent_id = Uuid::parse_str(&request.get_ref().agent_id)
            .map_err(|_| Status::invalid_argument("Invalid Agent ID"))?;

        let mut agent = self.repository.find_agent(agent_id)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        agent.start().map_err(|e| Status::internal(e.to_string()))?;
        self.repository.save_agent(&agent)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        // Re-start the background loop
        let loop_handle = Arc::clone(&self.cognitive_loop);
        tokio::spawn(async move {
            loop_handle.start_background_loop(agent_id).await;
        });

        Ok(Response::new(ResumeAgentResponse {
            agent_id: agent.id.to_string(),
            state: AgentState::Running as i32,
        }))
    }

    async fn inspect_agent(
        &self,
        _request: Request<InspectAgentRequest>,
    ) -> Result<Response<InspectAgentResponse>, Status> {
        Err(Status::unimplemented("InspectAgent is not yet implemented"))
    }

    type MonitorAgentStream = tokio_stream::wrappers::ReceiverStream<Result<AgentEvent, Status>>;

    async fn monitor_agent(
        &self,
        _request: Request<MonitorAgentRequest>,
    ) -> Result<Response<Self::MonitorAgentStream>, Status> {
        Err(Status::unimplemented("MonitorAgent is not yet implemented"))
    }
}

// Created by Joaquín Escalante (https://github.com/joaquinescalante23)
