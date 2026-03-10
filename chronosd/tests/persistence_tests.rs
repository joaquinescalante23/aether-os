#[cfg(test)]
mod tests {
    use chronosd::domain::agent::Agent;
    use chronosd::domain::checkpoint::{Checkpoint, Message, MessageRole};
    use chronosd::infrastructure::SqliteAgentRepository;
    use sqlx::sqlite::SqlitePoolOptions;
    use chrono::Utc;

    #[tokio::test]
    async fn test_persistence_cycle() {
        // 1. Setup In-Memory Database
        let pool = SqlitePoolOptions::new()
            .connect("sqlite::memory:")
            .await
            .unwrap();

        // 2. Run Migrations manually for the test
        sqlx::query(include_str!("../migrations/202603100001_initial_schema.sql"))
            .execute(&pool)
            .await
            .unwrap();

        let repo = SqliteAgentRepository::new(pool);

        // 3. Create Agent
        let mut agent = Agent::new("PersistTest".to_string(), "gpt-4".to_string(), 1.0, 1000);
        agent.start().unwrap();
        let _ = repo.save_agent(&agent).await.expect("Should save agent");

        // 4. Save Checkpoint
        let messages = vec![Message {
            role: MessageRole::User,
            content: "Hello World".to_string(),
            timestamp: Utc::now(),
        }];
        let checkpoint = Checkpoint::new(agent.id, messages, vec![]);
        let _ = repo.save_checkpoint(&checkpoint).await.expect("Should save checkpoint");

        // 5. Retrieve and Verify
        let retrieved = repo.find_agent(agent.id).await.expect("Should find agent");
        assert_eq!(retrieved.name, "PersistTest");
        assert_eq!(retrieved.model_id, "gpt-4");
    }
}
