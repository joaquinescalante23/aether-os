//! Chronos CLI - The Developer Control Interface
//!
//! Created by Joaquín Escalante (https://github.com/joaquinescalante23)

use clap::{Parser, Subcommand};
use chronos_proto::chronos_kernel_client::ChronosKernelClient;
use chronos_proto::{SpawnAgentRequest, ListAgentsRequest, StopAgentRequest, PauseAgentRequest, ResumeAgentRequest};

pub mod chronos_proto {
    tonic::include_proto!("chronos");
}

#[derive(Parser)]
#[command(name = "chronos")]
#[command(about = "Chronos Agent OS CLI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Spawns a new agent process in the kernel.
    Spawn {
        /// Name of the agent.
        name: String,
        /// Model ID to use.
        #[arg(short, long, default_value = "mock/model")]
        model: String,
    },
    /// Lists all managed agent processes.
    List,
    /// Stops an agent process permanently.
    Stop {
        /// Agent ID to stop.
        agent_id: String,
    },
    /// Pauses an agent process (Cognitive Checkpoint).
    Pause {
        /// Agent ID to pause.
        agent_id: String,
    },
    /// Resumes a paused agent process.
    Resume {
        /// Agent ID to resume.
        agent_id: String,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    let mut client = ChronosKernelClient::connect("http://[::1]:50051").await?;

    match cli.command {
        Commands::Spawn { name, model } => {
            let request = tonic::Request::new(SpawnAgentRequest {
                name,
                model_id: model,
                initial_prompt: "".to_string(),
                budget: None,
            });

            let response = client.spawn_agent(request).await?;
            println!("Agent spawned successfully!");
            println!("ID: {}", response.into_inner().agent_id);
        }
        Commands::List => {
            let request = tonic::Request::new(ListAgentsRequest {});
            let response = client.list_agents(request).await?;
            
            println!("{:<38} {:<15} {:<10} {:<15}", "AGENT ID", "NAME", "STATE", "COST (USD)");
            println!("{:-<38} {:-<15} {:-<10} {:-<15}", "", "", "", "");
            
            for agent in response.into_inner().agents {
                let state_str = match agent.state {
                    0 => "PENDING",
                    1 => "RUNNING",
                    2 => "SUSPENDED",
                    3 => "TERMINATED",
                    4 => "ERROR",
                    _ => "UNKNOWN",
                };
                let cost = agent.stats.map(|s| s.cost_usd).unwrap_or(0.0);
                println!("{:<38} {:<15} {:<10} ${:<14.4}", agent.agent_id, agent.name, state_str, cost);
            }
        }
        Commands::Stop { agent_id } => {
            let request = tonic::Request::new(StopAgentRequest { agent_id: agent_id.clone() });
            let _ = client.stop_agent(request).await?;
            println!("Agent {} stopped.", agent_id);
        }
        Commands::Pause { agent_id } => {
            let request = tonic::Request::new(PauseAgentRequest { agent_id: agent_id.clone() });
            let _ = client.pause_agent(request).await?;
            println!("Agent {} paused (Checkpoint created).", agent_id);
        }
        Commands::Resume { agent_id } => {
            let request = tonic::Request::new(ResumeAgentRequest { 
                agent_id: agent_id.clone(),
                checkpoint_id: "".to_string() 
            });
            let _ = client.resume_agent(request).await?;
            println!("Agent {} resumed.", agent_id);
        }
    }

    Ok(())
}

// Created by Joaquín Escalante (https://github.com/joaquinescalante23)
