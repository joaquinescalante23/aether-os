//! Chronos Tool Execution Domain
//!
//! Created by Joaquín Escalante (https://github.com/joaquinescalante23)

use crate::domain::error::DomainResult;
use async_trait::async_trait;
use serde_json::Value;

/// Represents a request from an agent to execute a tool.
#[derive(Debug, Clone)]
pub struct ToolCallRequest {
    pub tool_name: String,
    pub arguments: Value,
}

/// Represents the output of a tool execution to be fed back to the agent.
#[derive(Debug, Clone)]
pub struct ToolCallResult {
    pub tool_name: String,
    pub result: String,
    pub is_error: bool,
}

/// A generic interface for any tool that an agent can execute.
#[async_trait]
pub trait Tool: Send + Sync {
    /// The unique name of the tool (e.g., "shell_execute", "fs_read").
    fn name(&self) -> &str;

    /// A JSON Schema string describing the tool's expected arguments.
    /// This is injected into the LLM's system prompt.
    fn schema(&self) -> &str;

    /// Executes the tool with the given JSON arguments.
    async fn execute(&self, arguments: Value) -> DomainResult<ToolCallResult>;
}

/// The Kernel's registry of available tools for agents.
pub struct ToolRegistry {
    tools: std::collections::HashMap<String, Box<dyn Tool>>,
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self {
            tools: std::collections::HashMap::new(),
        }
    }

    /// Registers a new tool in the kernel.
    pub fn register(&mut self, tool: Box<dyn Tool>) {
        self.tools.insert(tool.name().to_string(), tool);
    }

    /// Retrieves a tool by name.
    pub fn get(&self, name: &str) -> Option<&dyn Tool> {
        self.tools.get(name).map(|t| t.as_ref())
    }

    /// Generates the combined schema for all registered tools.
    pub fn combined_schema(&self) -> String {
        let schemas: Vec<String> = self.tools.values().map(|t| t.schema().to_string()).collect();
        format!("[{}]", schemas.join(","))
    }
}

// Created by Joaquín Escalante (https://github.com/joaquinescalante23)
