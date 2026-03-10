//! Chronos LLM Provider Abstraction
//!
//! Created by Joaquín Escalante (https://github.com/joaquinescalante23)

use async_trait::async_trait;
use crate::domain::checkpoint::Message;
use crate::domain::error::DomainResult;

/// Response from an LLM provider including usage statistics.
pub struct LlmResponse {
    pub content: String,
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_cost_usd: f64,
}

/// A generic interface for Large Language Model providers.
#[async_trait]
pub trait LlmProvider: Send + Sync {
    /// Sends a sequence of messages to the model and returns the response.
    async fn completion(
        &self,
        model_id: &str,
        messages: &[Message],
    ) -> DomainResult<LlmResponse>;
}

// Created by Joaquín Escalante (https://github.com/joaquinescalante23)
