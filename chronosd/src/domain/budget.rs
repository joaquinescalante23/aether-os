//! Chronos Agent Budget Domain Model
//!
//! Created by Joaquín Escalante (https://github.com/joaquinescalante23)

use crate::domain::error::{DomainError, DomainResult};
use serde::{Deserialize, Serialize};

/// Represents the resource limits and current usage for an agent process.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Budget {
    /// Maximum allowed cost in USD.
    pub max_cost_usd: f64,
    /// Maximum allowed tokens.
    pub max_tokens: u32,
    /// Current accumulated cost in USD.
    pub current_cost_usd: f64,
    /// Current accumulated tokens used.
    pub current_tokens: u32,
}

impl Budget {
    /// Creates a new budget with specified limits.
    pub fn new(max_cost_usd: f64, max_tokens: u32) -> Self {
        Self {
            max_cost_usd,
            max_tokens,
            current_cost_usd: 0.0,
            current_tokens: 0.0 as u32,
        }
    }

    /// Records the consumption of tokens and cost, checking against limits.
    pub fn consume(&mut self, agent_id: uuid::Uuid, tokens: u32, cost: f64) -> DomainResult<()> {
        let new_tokens = self.current_tokens + tokens;
        let new_cost = self.current_cost_usd + cost;

        if new_tokens > self.max_tokens {
            return Err(DomainError::BudgetExceeded(
                agent_id,
                format!("Token limit {} exceeded (attempted to use {})", self.max_tokens, new_tokens),
            ));
        }

        if new_cost > self.max_cost_usd {
            return Err(DomainError::BudgetExceeded(
                agent_id,
                format!("Cost limit ${:.4} exceeded (attempted to spend ${:.4})", self.max_cost_usd, new_cost),
            ));
        }

        self.current_tokens = new_tokens;
        self.current_cost_usd = new_cost;
        Ok(())
    }

    /// Returns the remaining token budget.
    pub fn remaining_tokens(&self) -> u32 {
        self.max_tokens.saturating_sub(self.current_tokens)
    }

    /// Returns the remaining cost budget in USD.
    pub fn remaining_cost(&self) -> f64 {
        (self.max_cost_usd - self.current_cost_usd).max(0.0)
    }
}

// Created by Joaquín Escalante (https://github.com/joaquinescalante23)
