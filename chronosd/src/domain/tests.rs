#[cfg(test)]
mod tests {
    use crate::domain::agent::{Agent, AgentState};
    use crate::domain::budget::Budget;
    use crate::domain::identity::Identity;
    use crate::domain::mission::{Mission, MissionStatus};
    use uuid::Uuid;

    #[test]
    fn test_agent_state_transitions() {
        let mut agent = Agent::new("Test".to_string(), "model".to_string(), 1.0, 1000);
        
        assert_eq!(agent.state, AgentState::Pending);
        
        // Start agent
        agent.start().expect("Should start");
        assert_eq!(agent.state, AgentState::Running);
        
        // Pause agent
        agent.pause().expect("Should pause");
        assert_eq!(agent.state, AgentState::Suspended);
        
        // Resume agent
        agent.start().expect("Should resume");
        assert_eq!(agent.state, AgentState::Running);
        
        // Stop agent
        agent.stop();
        assert_eq!(agent.state, AgentState::Terminated);
    }

    #[test]
    fn test_budget_consumption() {
        let mut budget = Budget::new(0.01, 100);
        let id = Uuid::new_v4();

        // Valid consumption
        budget.consume(id, 50, 0.005).expect("Should consume");
        assert_eq!(budget.remaining_tokens(), 50);
        assert_eq!(budget.remaining_cost(), 0.005);

        // Exceed tokens
        let result = budget.consume(id, 60, 0.001);
        assert!(result.is_err(), "Should fail due to token limit");

        // Exceed cost
        let result = budget.consume(id, 1, 0.01);
        assert!(result.is_err(), "Should fail due to cost limit");
    }

    #[test]
    fn test_identity_boot_prompt() {
        let mut identity = Identity::new("Architect".to_string(), "You design systems.".to_string());
        identity.allow_tool("shell_execute");
        
        let prompt = identity.boot_prompt();
        assert!(prompt.contains("Architect"));
        assert!(prompt.contains("AETHER-OS KERNEL INSTRUCTIONS"));
        assert!(prompt.contains("shell_execute"));
    }

    #[test]
    fn test_mission_management() {
        let mut mission = Mission::new("Build App".to_string(), "Create a React app.".to_string());
        let agent_id = Uuid::new_v4();
        
        assert_eq!(mission.status, MissionStatus::Queued);
        mission.add_agent(agent_id);
        assert!(mission.agent_ids.contains(&agent_id));
    }

    #[test]
    fn test_tool_registry() {
        use crate::domain::tool::{Tool, ToolCallResult, ToolRegistry};
        use crate::domain::error::DomainResult;
        use async_trait::async_trait;
        use serde_json::Value;

        struct MockTool;
        #[async_trait]
        impl Tool for MockTool {
            fn name(&self) -> &str { "mock" }
            fn schema(&self) -> &str { "{}" }
            async fn execute(&self, _: Value) -> DomainResult<ToolCallResult> {
                Ok(ToolCallResult { tool_name: "mock".to_string(), result: "ok".to_string(), is_error: false })
            }
        }

        let mut registry = ToolRegistry::new();
        registry.register(Box::new(MockTool));
        
        assert!(registry.get("mock").is_some());
        assert!(registry.get("unknown").is_none());
        assert!(registry.combined_schema().contains("{}"));
    }
}
