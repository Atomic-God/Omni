use serde::{Serialize, Deserialize};
use std::collections::HashMap;

// --- Architecture Definitions ---

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ActionType {
    Read,
    Write,
    Execute,
    Network,
    System,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Permission {
    pub action: ActionType,
    pub resource_pattern: String, // Glob or Regex
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDescriptor {
    pub name: String,
    pub description: String,
    pub capabilities: Vec<ActionType>,
    pub usage_pattern: String,
}

pub trait Tool: Send + Sync {
    fn descriptor(&self) -> ToolDescriptor;
    fn validate_permissions(&self, action: &ActionType, resource: &str, permissions: &[Permission]) -> bool;
    fn execute(&self, args: &str) -> Result<String, String>;
}

pub struct ActionContext {
    pub permissions: Vec<Permission>,
}

impl ActionContext {
    pub fn new() -> Self {
        Self { permissions: Vec::new() }
    }

    pub fn allow(&mut self, action: ActionType, pattern: &str) {
        self.permissions.push(Permission { action, resource_pattern: pattern.to_string() });
    }

    pub fn check(&self, action: &ActionType, resource: &str) -> bool {
        for perm in &self.permissions {
            if perm.action == *action {
                // Simple prefix match for now
                if resource.starts_with(&perm.resource_pattern) || perm.resource_pattern == "*" {
                    return true;
                }
            }
        }
        false
    }
}

// --- Stub Implementations ---

pub struct FileReadTool;
impl Tool for FileReadTool {
    fn descriptor(&self) -> ToolDescriptor {
        ToolDescriptor {
            name: "file_read".to_string(),
            description: "Reads content of a file".to_string(),
            capabilities: vec![ActionType::Read],
            usage_pattern: "file_read <path>".to_string(),
        }
    }

    fn validate_permissions(&self, action: &ActionType, resource: &str, permissions: &[Permission]) -> bool {
        if *action != ActionType::Read { return false; }
        // Iterate perms
        for perm in permissions {
            if perm.action == ActionType::Read && (resource.starts_with(&perm.resource_pattern) || perm.resource_pattern == "*") {
                return true;
            }
        }
        false
    }

    fn execute(&self, _args: &str) -> Result<String, String> {
        // Stub
        Ok("File content (stub)".to_string())
    }
}
