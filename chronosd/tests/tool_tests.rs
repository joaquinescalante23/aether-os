#[cfg(test)]
mod tests {
    use chronosd::domain::Tool;
    use chronosd::infrastructure::tools::{FileSystemTool, ShellTool};
    use serde_json::json;
    use std::fs;

    #[tokio::test]
    async fn test_filesystem_tool_write() {
        let tool = FileSystemTool;
        let path = "test_file.txt";
        let content = "Hello AetherOS";
        
        let args = json!({
            "path": path,
            "content": content
        });

        let result = tool.execute(args).await.expect("Should execute write");
        assert!(!result.is_error);
        
        // Verify file exists
        let read_content = fs::read_to_string(path).unwrap();
        assert_eq!(read_content, content);
        
        // Cleanup
        fs::remove_file(path).unwrap();
    }

    #[tokio::test]
    async fn test_shell_tool_execute() {
        let tool = ShellTool;
        
        // Test success
        let args = json!({ "command": "echo 'test'" });
        let result = tool.execute(args).await.expect("Should execute echo");
        assert!(!result.is_error);
        assert!(result.result.contains("test"));

        // Test failure
        let args = json!({ "command": "non_existent_command_123" });
        let result = tool.execute(args).await.expect("Should execute and return error");
        assert!(result.is_error);
        assert!(result.result.contains("STDERR"));
    }
}
