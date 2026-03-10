//! Chronos Mock LLM Provider Implementation
//!
//! Created by Joaquín Escalante (https://github.com/joaquinescalante23)

use async_trait::async_trait;
use crate::domain::checkpoint::Message;
use crate::domain::error::DomainResult;
use crate::domain::llm::{LlmProvider, LlmResponse};
use tracing::info;

/// A simple mock provider that simulates LLM responses for testing.
pub struct MockLlmProvider;

#[async_trait]
impl LlmProvider for MockLlmProvider {
    async fn completion(
        &self,
        model_id: &str,
        messages: &[Message],
    ) -> DomainResult<LlmResponse> {
        info!("Mock LLM: Simulating completion for model {}", model_id);

        let last_msg = messages.last()
            .map(|m| m.content.as_str())
            .unwrap_or("No input");

        let content = format!("I am a simulated response to: '{}'. I am processing your task.", last_msg);

        // Simulate usage
        Ok(LlmResponse {
            content,
            prompt_tokens: 10,
            completion_tokens: 20,
            total_cost_usd: 0.0001,
        })
    }
}

// Created by Joaquín Escalante (https://github.com/joaquinescalante23)
