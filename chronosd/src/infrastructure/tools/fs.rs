//! Chronos FileSystem Tool
//!
//! Created by Joaquín Escalante (https://github.com/joaquinescalante23)

use crate::domain::error::{DomainError, DomainResult};
use crate::domain::tool::{Tool, ToolCallResult};
use async_trait::async_trait;
use serde_json::Value;
use tokio::fs;

pub struct FileSystemTool;

#[async_trait]
impl Tool for FileSystemTool {
    fn name(&self) -> &str {
        "fs_write"
    }

    fn schema(&self) -> &str {
        r#"{
            "name": "fs_write",
            "description": "Writes content to a file on the local filesystem.",
            "parameters": {
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "The relative or absolute path to the file."
                    },
                    "content": {
                        "type": "string",
                        "description": "The content to write into the file."
                    }
                },
                "required": ["path", "content"]
            }
        }"#
    }

    async fn execute(&self, arguments: Value) -> DomainResult<ToolCallResult> {
        let path = arguments.get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| DomainError::Internal("Missing 'path' argument".to_string()))?;

        let content = arguments.get("content")
            .and_then(|v| v.as_str())
            .ok_or_else(|| DomainError::Internal("Missing 'content' argument".to_string()))?;

        tracing::info!("Writing to file: {}", path);

        match fs::write(path, content).await {
            Ok(_) => Ok(ToolCallResult {
                tool_name: self.name().to_string(),
                result: format!("Successfully wrote to {}", path),
                is_error: false,
            }),
            Err(e) => Ok(ToolCallResult {
                tool_name: self.name().to_string(),
                result: format!("Error writing file: {}", e),
                is_error: true,
            })
        }
    }
}

// Created by Joaquín Escalante (https://github.com/joaquinescalante23)
