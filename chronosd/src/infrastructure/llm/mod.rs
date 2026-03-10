//! Chronos LLM Infrastructure Adaptors
//!
//! Created by Joaquín Escalante (https://github.com/joaquinescalante23)

pub mod mock_provider;
pub mod openai;

pub use mock_provider::MockLlmProvider;
pub use openai::GenericOpenAiProvider;

// Created by Joaquín Escalante (https://github.com/joaquinescalante23)
