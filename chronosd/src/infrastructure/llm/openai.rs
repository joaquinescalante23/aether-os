//! Chronos Generic OpenAI-compatible LLM Provider
//!
//! Created by Joaquín Escalante (https://github.com/joaquinescalante23)

use async_trait::async_trait;
use crate::domain::checkpoint::{Message, MessageRole};
use crate::domain::error::{DomainError, DomainResult};
use crate::domain::llm::{LlmProvider, LlmResponse};
use serde::{Deserialize, Serialize};
use tracing::info;

/// A provider that talks to any OpenAI-compatible API (OpenAI, Groq, Mistral, Ollama, etc.)
pub struct GenericOpenAiProvider {
    pub api_key: String,
    pub base_url: String,
    client: reqwest::Client,
}

#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
}

#[derive(Serialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
    usage: Option<Usage>,
}

#[derive(Deserialize)]
struct Choice {
    message: ChatMessageResponse,
}

#[derive(Deserialize)]
struct ChatMessageResponse {
    content: Option<String>,
}

#[derive(Deserialize)]
struct Usage {
    prompt_tokens: u32,
    completion_tokens: u32,
}

impl GenericOpenAiProvider {
    /// Creates a new generic provider.
    pub fn new(api_key: String, base_url: String) -> Self {
        Self {
            api_key,
            base_url,
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl LlmProvider for GenericOpenAiProvider {
    async fn completion(
        &self,
        model_id: &str,
        messages: &[Message],
    ) -> DomainResult<LlmResponse> {
        info!("Calling OpenAI-compatible API at {} for model {}", self.base_url, model_id);

        let chat_messages: Vec<ChatMessage> = messages.iter().map(|m| ChatMessage {
            role: match m.role {
                MessageRole::System => "system".to_string(),
                MessageRole::User => "user".to_string(),
                MessageRole::Assistant => "assistant".to_string(),
                MessageRole::Tool => "user".to_string(), // In standard API, tool results go as user for simplicity in MVP
            },
            content: m.content.clone(),
        }).collect();

        let request = ChatRequest {
            model: model_id.to_string(),
            messages: chat_messages,
        };

        let response = self.client.post(format!("{}/chat/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&request)
            .send()
            .await
            .map_err(|e| DomainError::Internal(format!("HTTP request failed: {}", e)))?;

        if !response.status().is_success() {
            let err_text = response.text().await.unwrap_or_default();
            return Err(DomainError::Internal(format!("API error: {}", err_text)));
        }

        let body: ChatResponse = response.json().await
            .map_err(|e| DomainError::Internal(format!("Failed to parse response: {}", e)))?;

        let choice = body.choices.get(0)
            .ok_or_else(|| DomainError::Internal("No choices returned from LLM".to_string()))?;

        let content = choice.message.content.clone()
            .ok_or_else(|| DomainError::Internal("Empty message content returned from LLM".to_string()))?;

        let usage = body.usage.unwrap_or(Usage { prompt_tokens: 0, completion_tokens: 0 });

        // Simple cost estimation if not provided by the API (TODO: Use a real pricing table)
        let total_cost = (usage.prompt_tokens as f64 * 0.000001) + (usage.completion_tokens as f64 * 0.000002);

        Ok(LlmResponse {
            content,
            prompt_tokens: usage.prompt_tokens,
            completion_tokens: usage.completion_tokens,
            total_cost_usd: total_cost,
        })
    }
}

// Created by Joaquín Escalante (https://github.com/joaquinescalante23)
