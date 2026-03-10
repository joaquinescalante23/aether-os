//! Chronos Daemon (Kernel) Entry Point
//!
//! Created by Joaquín Escalante (https://github.com/joaquinescalante23)

use chronosd::application::CognitiveLoop;
use chronosd::domain::ToolRegistry;
use chronosd::infrastructure::tools::{FileSystemTool, ShellTool};
use chronosd::infrastructure::SqliteAgentRepository;
use chronosd::presentation::grpc_server::chronos_proto::chronos_kernel_server::ChronosKernelServer;
use chronosd::presentation::ChronosServer;
use sqlx::sqlite::SqlitePoolOptions;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;
use tonic::transport::Server;
use tracing::{info, warn, Level};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Initialize Logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    info!("Starting Chronos Daemon (chronosd)...");

    // 2. Initialize Database
    let db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:chronos.db?mode=rwc".to_string());
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await?;

    // 3. Run Migrations
    info!("Running database migrations...");
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await?;

    // 4. Initialize Tools (The "Execution Environment")
    info!("Initializing Tool Registry...");
    let mut tool_registry = ToolRegistry::new();
    tool_registry.register(Box::new(ShellTool));
    tool_registry.register(Box::new(FileSystemTool));
    let tool_registry = Arc::new(RwLock::new(tool_registry));

    // 5. Initialize Infrastructure & Application
    let repository = Arc::new(SqliteAgentRepository::new(pool));

    // Choose LLM Provider based on env vars
    let api_key = std::env::var("CHRONOS_LLM_API_KEY");
    let base_url = std::env::var("CHRONOS_LLM_BASE_URL").unwrap_or_else(|_| "https://api.openai.com/v1".to_string());

    let llm_provider: Arc<dyn chronosd::domain::LlmProvider> = if let Ok(key) = api_key {
        info!("Using Generic OpenAI-compatible Provider with endpoint: {}", base_url);
        Arc::new(chronosd::infrastructure::llm::openai::GenericOpenAiProvider::new(key, base_url))
    } else {
        warn!("CHRONOS_LLM_API_KEY not found. Falling back to MockLlmProvider.");
        Arc::new(chronosd::infrastructure::llm::mock_provider::MockLlmProvider)
    };

    let cognitive_loop = Arc::new(CognitiveLoop::new(
        Arc::clone(&repository),
        llm_provider,
        Arc::clone(&tool_registry),
    ));

    // 6. Initialize Presentation
    let chronos_service = ChronosServer::new(repository, cognitive_loop);

    // 7. Start gRPC Server
    let addr: SocketAddr = "[::1]:50051".parse()?;
    info!("Chronos Kernel listening on {}", addr);

    Server::builder()
        .add_service(ChronosKernelServer::new(chronos_service))
        .serve(addr)
        .await?;

    Ok(())
}

// Created by Joaquín Escalante (https://github.com/joaquinescalante23)
