//! Context Registry
//!
//! Manages context metadata, schemas, and loaders.

use crate::context::loaders::{ContextLoader, MockUserLoader, MockLogLoader, MockProcessLoader, MockAgentLoader};
use crate::context::ContextData;
use crate::error::{ErrorCode, Result, SemanticCLIError};
use std::collections::HashMap;

/// Context registry that manages all available contexts
pub struct ContextRegistry {
    contexts: HashMap<String, ContextMetadata>,
}

/// Metadata for a context including schema and loader
pub struct ContextMetadata {
    pub path: String,
    pub schema: ContextSchema,
    pub loader: Box<dyn ContextLoader>,
}

/// Schema definition for a context
#[derive(Debug, Clone)]
pub struct ContextSchema {
    pub fields: HashMap<String, FieldType>,
    pub description: String,
}

/// Field type information
#[derive(Debug, Clone, PartialEq)]
pub enum FieldType {
    String,
    Number,
    Boolean,
    Array,
    Object,
}

impl ContextRegistry {
    /// Create a new context registry with default contexts
    pub fn new() -> Self {
        let mut registry = Self {
            contexts: HashMap::new(),
        };

        // Register default contexts for Phase 3.5.1.a
        registry.register_context("data.users", Self::users_schema(), Box::new(MockUserLoader::new()));
        registry.register_context("data.logs", Self::logs_schema(), Box::new(MockLogLoader::new()));
        registry.register_context("fs.logs", Self::fs_logs_schema(), Box::new(MockLogLoader::new()));
        registry.register_context("system.processes", Self::processes_schema(), Box::new(MockProcessLoader::new()));
        registry.register_context("system.agents", Self::agents_schema(), Box::new(MockAgentLoader::new()));

        registry
    }

    /// Register a new context
    fn register_context(&mut self, path: &str, schema: ContextSchema, loader: Box<dyn ContextLoader>) {
        let metadata = ContextMetadata {
            path: path.to_string(),
            schema,
            loader,
        };
        self.contexts.insert(path.to_string(), metadata);
    }

    /// Check if a context exists
    pub fn context_exists(&self, path: &str) -> bool {
        self.contexts.contains_key(path)
    }

    /// Load context data
    pub fn load_context(&self, path: &str) -> Result<ContextData> {
        let metadata = self.contexts.get(path)
            .ok_or_else(|| SemanticCLIError::context_error(
                format!("Context '{}' not found", path),
                ErrorCode::E500,
            ))?;

        metadata.loader.load()
    }

    /// List all available contexts
    pub fn list_contexts(&self) -> Vec<String> {
        self.contexts.keys().cloned().collect()
    }

    /// Get context schema
    pub fn get_context_schema(&self, path: &str) -> Result<HashMap<String, String>> {
        let metadata = self.contexts.get(path)
            .ok_or_else(|| SemanticCLIError::context_error(
                format!("Context '{}' not found", path),
                ErrorCode::E500,
            ))?;

        let mut schema_map = HashMap::new();
        for (field, field_type) in &metadata.schema.fields {
            schema_map.insert(field.clone(), format!("{:?}", field_type));
        }
        Ok(schema_map)
    }

    /// Get context metadata
    pub fn get_metadata(&self, path: &str) -> Option<&ContextMetadata> {
        self.contexts.get(path)
    }

    // Schema definitions for default contexts

    fn users_schema() -> ContextSchema {
        let mut fields = HashMap::new();
        fields.insert("id".to_string(), FieldType::String);
        fields.insert("name".to_string(), FieldType::String);
        fields.insert("age".to_string(), FieldType::Number);
        fields.insert("email".to_string(), FieldType::String);
        fields.insert("active".to_string(), FieldType::Boolean);
        fields.insert("roles".to_string(), FieldType::Array);

        ContextSchema {
            fields,
            description: "User data context with user profiles and information".to_string(),
        }
    }

    fn logs_schema() -> ContextSchema {
        let mut fields = HashMap::new();
        fields.insert("id".to_string(), FieldType::String);
        fields.insert("timestamp".to_string(), FieldType::String);
        fields.insert("level".to_string(), FieldType::String);
        fields.insert("message".to_string(), FieldType::String);
        fields.insert("source".to_string(), FieldType::String);
        fields.insert("metadata".to_string(), FieldType::Object);

        ContextSchema {
            fields,
            description: "System logs context with log entries and metadata".to_string(),
        }
    }

    fn fs_logs_schema() -> ContextSchema {
        let mut fields = HashMap::new();
        fields.insert("path".to_string(), FieldType::String);
        fields.insert("size".to_string(), FieldType::Number);
        fields.insert("modified".to_string(), FieldType::String);
        fields.insert("readable".to_string(), FieldType::Boolean);
        fields.insert("permissions".to_string(), FieldType::String);

        ContextSchema {
            fields,
            description: "Filesystem logs context with file information".to_string(),
        }
    }

    fn processes_schema() -> ContextSchema {
        let mut fields = HashMap::new();
        fields.insert("pid".to_string(), FieldType::Number);
        fields.insert("name".to_string(), FieldType::String);
        fields.insert("cpu_usage".to_string(), FieldType::Number);
        fields.insert("memory_usage".to_string(), FieldType::Number);
        fields.insert("running".to_string(), FieldType::Boolean);
        fields.insert("command".to_string(), FieldType::String);

        ContextSchema {
            fields,
            description: "System processes context with process information".to_string(),
        }
    }

    fn agents_schema() -> ContextSchema {
        let mut fields = HashMap::new();
        fields.insert("id".to_string(), FieldType::String);
        fields.insert("name".to_string(), FieldType::String);
        fields.insert("status".to_string(), FieldType::String);
        fields.insert("active".to_string(), FieldType::Boolean);
        fields.insert("load".to_string(), FieldType::Number);
        fields.insert("capabilities".to_string(), FieldType::Array);

        ContextSchema {
            fields,
            description: "System agents context with agent status and capabilities".to_string(),
        }
    }
}

impl Default for ContextRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_creation() {
        let registry = ContextRegistry::new();
        
        assert!(registry.context_exists("data.users"));
        assert!(registry.context_exists("data.logs"));
        assert!(registry.context_exists("fs.logs"));
        assert!(registry.context_exists("system.processes"));
        assert!(registry.context_exists("system.agents"));
        assert!(!registry.context_exists("invalid.context"));
    }

    #[test]
    fn test_list_contexts() {
        let registry = ContextRegistry::new();
        let contexts = registry.list_contexts();
        
        assert!(contexts.contains(&"data.users".to_string()));
        assert!(contexts.contains(&"data.logs".to_string()));
        assert!(contexts.contains(&"fs.logs".to_string()));
        assert!(contexts.contains(&"system.processes".to_string()));
        assert!(contexts.contains(&"system.agents".to_string()));
    }

    #[test]
    fn test_context_schema() {
        let registry = ContextRegistry::new();
        
        let schema = registry.get_context_schema("data.users");
        assert!(schema.is_ok());
        
        let schema_map = schema.unwrap();
        assert!(schema_map.contains_key("id"));
        assert!(schema_map.contains_key("name"));
        assert!(schema_map.contains_key("age"));
        assert!(schema_map.contains_key("email"));
        assert!(schema_map.contains_key("active"));
    }

    #[test]
    fn test_load_context() {
        let registry = ContextRegistry::new();
        
        let result = registry.load_context("data.users");
        assert!(result.is_ok());
        
        let data = result.unwrap();
        assert!(!data.items.is_empty());
    }

    #[test]
    fn test_load_invalid_context() {
        let registry = ContextRegistry::new();
        
        let result = registry.load_context("invalid.context");
        assert!(result.is_err());
        
        if let Err(SemanticCLIError::ContextError { .. }) = result {
            // Expected context error
        } else {
            panic!("Expected ContextError");
        }
    }

    #[test]
    fn test_get_metadata() {
        let registry = ContextRegistry::new();
        
        let metadata = registry.get_metadata("data.users");
        assert!(metadata.is_some());
        
        let metadata = metadata.unwrap();
        assert_eq!(metadata.path, "data.users");
        assert_eq!(metadata.schema.description, "User data context with user profiles and information");
    }
}