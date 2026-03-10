//! Chronos Identity and Capability System
//!
//! Created by Joaquín Escalante (https://github.com/joaquinescalante23)

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Represents a "Cognitive Image" - a template for spawning specialized agents.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Identity {
    /// The name of the role (e.g., "Senior Architect", "Security Auditor").
    pub role_name: String,
    /// The base system prompt that defines the agent's behavior.
    pub system_prompt: String,
    /// List of default capabilities/permissions for this identity.
    pub allowed_tools: HashSet<String>,
}

impl Identity {
    /// Creates a new identity template.
    pub fn new(role_name: String, system_prompt: String) -> Self {
        Self {
            role_name,
            system_prompt,
            allowed_tools: HashSet::new(),
        }
    }

    /// Adds a tool permission to this identity.
    pub fn allow_tool(&mut self, tool_name: &str) {
        self.allowed_tools.insert(tool_name.to_string());
    }

    /// Injects AetherOS Kernel instructions into the system prompt.
    /// This ensures the agent knows how to interact with the OS.
    pub fn boot_prompt(&self) -> String {
        format!(
            "{}\n\n[AETHER-OS KERNEL INSTRUCTIONS]\n\
            1. You are running as a managed process in AetherOS.\n\
            2. Every action is checkpointed for persistence.\n\
            3. Available Tools: {:?}\n\
            4. If you exceed your budget, you will be suspended.",
            self.system_prompt, self.allowed_tools
        )
    }
}

// Created by Joaquín Escalante (https://github.com/joaquinescalante23)
