#[cfg(test)]
mod tests {
    use chronosd::application::{CognitiveLoop, Orchestrator, ShadowService};
    use chronosd::domain::ToolRegistry;
    use chronosd::infrastructure::llm::MockLlmProvider;
    use chronosd::infrastructure::SqliteAgentRepository;
    use sqlx::sqlite::SqlitePoolOptions;
    use std::sync::Arc;
    use tokio::sync::RwLock;

    async fn setup_test_env() -> (Arc<SqliteAgentRepository>, Arc<MockLlmProvider>, Arc<CognitiveLoop>) {
        let pool = SqlitePoolOptions::new()
            .connect("sqlite::memory:")
            .await
            .unwrap();

        sqlx::query(include_str!("../migrations/202603100001_initial_schema.sql"))
            .execute(&pool)
            .await
            .unwrap();

        let repo = Arc::new(SqliteAgentRepository::new(pool));
        let llm = Arc::new(MockLlmProvider);
        let registry = Arc::new(RwLock::new(ToolRegistry::new()));
        
        let cognitive_loop = Arc::new(CognitiveLoop::new(
            Arc::clone(&repo),
            llm.clone(),
            registry,
        ));

        (repo, llm, cognitive_loop)
    }

    #[tokio::test]
    async fn test_orchestrator_mission_spawn() {
        let (repo, _, cognitive_loop) = setup_test_env().await;
        let orchestrator = Orchestrator::new(repo.clone(), cognitive_loop);

        let mission_id = orchestrator
            .start_dev_mission("Test Mission".to_string(), "Build a test".to_string())
            .await
            .expect("Should start mission");

        // Verify agents were created
        let agents = repo.list_agents().await.unwrap();
        assert_eq!(agents.len(), 2, "Should have spawned 2 agents (Architect + Coder)");
        
        let architect = agents.iter().find(|a| a.name.contains("Architect")).unwrap();
        assert_eq!(architect.budget.max_tokens, 50000);
    }

    #[tokio::test]
    async fn test_shadow_service_audit() {
        let (repo, llm, _) = setup_test_env().await;
        let shadow_service = ShadowService::new(repo, llm);

        let agent_id = uuid::Uuid::new_v4();
        let checkpoint_id = uuid::Uuid::new_v4();

        // The MockLlmProvider always returns "I am a simulated response..." 
        // This won't contain '"verdict": "pass"', so it should fail the audit.
        let report = shadow_service
            .audit_execution(agent_id, checkpoint_id, "Some bad code")
            .await
            .expect("Should run audit");

        assert!(!report.is_success());
    }
}
