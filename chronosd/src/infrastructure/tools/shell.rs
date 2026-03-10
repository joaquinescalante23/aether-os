//! Chronos Shell Execution Tool
//!
//! Created by Joaquín Escalante (https://github.com/joaquinescalante23)

use crate::domain::error::{DomainError, DomainResult};
use crate::domain::tool::{Tool, ToolCallResult};
use async_trait::async_trait;
use serde_json::Value;
use std::process::Stdio;
use tokio::process::Command;

pub struct ShellTool;

#[async_trait]
impl Tool for ShellTool {
    fn name(&self) -> &str {
        "shell_execute"
    }

    fn schema(&self) -> &str {
        r#"{
            "name": "shell_execute",
            "description": "Executes a shell command on the host system.",
            "parameters": {
                "type": "object",
                "properties": {
                    "command": {
                        "type": "string",
                        "description": "The bash command to execute."
                    }
                },
                "required": ["command"]
            }
        }"#
    }

    async fn execute(&self, arguments: Value) -> DomainResult<ToolCallResult> {
        let command_str = arguments.get("command")
            .and_then(|v| v.as_str())
            .ok_or_else(|| DomainError::Internal("Missing 'command' argument".to_string()))?;

        // TODO: In a production "AetherOS", this MUST run inside a WebAssembly sandbox or Docker container.
        // For this MVP, we execute it locally with a strong warning.
        tracing::warn!("Executing local shell command: {}", command_str);

        let output = Command::new("bash")
            .arg("-c")
            .arg(command_str)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .map_err(|e| DomainError::Internal(format!("Failed to execute command: {}", e)))?;

        let mut result_str = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr_str = String::from_utf8_lossy(&output.stderr).to_string();

        let mut is_error = false;
        if !output.status.success() || !stderr_str.is_empty() {
            result_str = format!("STDOUT:\n{}\nSTDERR:\n{}", result_str, stderr_str);
            is_error = !output.status.success();
        }

        Ok(ToolCallResult {
            tool_name: self.name().to_string(),
            result: result_str.trim().to_string(),
            is_error,
        })
    }
}

// Created by Joaquín Escalante (https://github.com/joaquinescalante23)
