#[cfg(test)]
mod tests {
    use crate::domain::agent::{Agent, AgentState};
    use crate::domain::budget::Budget;
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
}
