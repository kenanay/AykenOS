//! Context Data Loaders
//!
//! This module implements data loaders for different context types.
//! In Phase 3.5.1.a, all loaders return mock data for testing purposes.

use crate::context::ContextData;
use crate::error::Result;
use serde_json::json;
use std::time::{Duration, Instant};

/// Trait for context data loaders
pub trait ContextLoader: Send + Sync {
    /// Load context data
    fn load(&self) -> Result<ContextData>;
    
    /// Get loader name for debugging
    fn name(&self) -> &str;
}

/// Mock user data loader (simulates database)
pub struct MockUserLoader {
    name: String,
}

impl MockUserLoader {
    pub fn new() -> Self {
        Self {
            name: "MockUserLoader".to_string(),
        }
    }
}

impl ContextLoader for MockUserLoader {
    fn load(&self) -> Result<ContextData> {
        // Simulate database query latency (5-50ms)
        let latency = Duration::from_millis(5 + (rand::random::<u64>() % 45));
        std::thread::sleep(latency);

        let users = vec![
            json!({
                "id": "user_001",
                "name": "Alice Johnson",
                "age": 28,
                "email": "alice@example.com",
                "active": true,
                "roles": ["admin", "developer"]
            }),
            json!({
                "id": "user_002", 
                "name": "Bob Smith",
                "age": 34,
                "email": "bob@example.com",
                "active": true,
                "roles": ["developer"]
            }),
            json!({
                "id": "user_003",
                "name": "Carol Davis",
                "age": 29,
                "email": "carol@example.com", 
                "active": false,
                "roles": ["analyst"]
            }),
            json!({
                "id": "user_004",
                "name": "David Wilson",
                "age": 42,
                "email": "david@example.com",
                "active": true,
                "roles": ["manager", "admin"]
            }),
            json!({
                "id": "user_005",
                "name": "Eve Brown",
                "age": 26,
                "email": "eve@example.com",
                "active": true,
                "roles": ["developer", "tester"]
            }),
        ];

        Ok(ContextData::new(users))
    }

    fn name(&self) -> &str {
        &self.name
    }
}

/// Mock log data loader (simulates log files)
pub struct MockLogLoader {
    name: String,
}

impl MockLogLoader {
    pub fn new() -> Self {
        Self {
            name: "MockLogLoader".to_string(),
        }
    }
}

impl ContextLoader for MockLogLoader {
    fn load(&self) -> Result<ContextData> {
        // Simulate log file reading latency (10-80ms)
        let latency = Duration::from_millis(10 + (rand::random::<u64>() % 70));
        std::thread::sleep(latency);

        let logs = vec![
            json!({
                "id": "log_001",
                "timestamp": "2026-01-15T10:30:00Z",
                "level": "INFO",
                "message": "System startup completed successfully",
                "source": "kernel",
                "metadata": {
                    "boot_time": "2.3s",
                    "modules_loaded": 42
                }
            }),
            json!({
                "id": "log_002",
                "timestamp": "2026-01-15T10:31:15Z", 
                "level": "WARN",
                "message": "High memory usage detected",
                "source": "memory_manager",
                "metadata": {
                    "usage_percent": 85,
                    "available_mb": 512
                }
            }),
            json!({
                "id": "log_003",
                "timestamp": "2026-01-15T10:32:30Z",
                "level": "ERROR",
                "message": "Failed to connect to external service",
                "source": "network_service",
                "metadata": {
                    "service": "api.example.com",
                    "retry_count": 3,
                    "error_code": "TIMEOUT"
                }
            }),
            json!({
                "id": "log_004",
                "timestamp": "2026-01-15T10:33:45Z",
                "level": "INFO",
                "message": "User authentication successful",
                "source": "auth_service",
                "metadata": {
                    "user_id": "user_001",
                    "session_id": "sess_abc123",
                    "ip_address": "192.168.1.100"
                }
            }),
            json!({
                "id": "log_005",
                "timestamp": "2026-01-15T10:35:00Z",
                "level": "DEBUG",
                "message": "Cache invalidation completed",
                "source": "cache_manager",
                "metadata": {
                    "cache_type": "user_sessions",
                    "entries_cleared": 15,
                    "duration_ms": 23
                }
            }),
        ];

        Ok(ContextData::new(logs))
    }

    fn name(&self) -> &str {
        &self.name
    }
}

/// Mock process data loader (simulates /proc filesystem)
pub struct MockProcessLoader {
    name: String,
}

impl MockProcessLoader {
    pub fn new() -> Self {
        Self {
            name: "MockProcessLoader".to_string(),
        }
    }
}

impl ContextLoader for MockProcessLoader {
    fn load(&self) -> Result<ContextData> {
        // Simulate /proc reading latency (2-20ms)
        let latency = Duration::from_millis(2 + (rand::random::<u64>() % 18));
        std::thread::sleep(latency);

        let processes = vec![
            json!({
                "pid": 1,
                "name": "init",
                "cpu_usage": 0.1,
                "memory_usage": 2048,
                "running": true,
                "command": "/sbin/init"
            }),
            json!({
                "pid": 123,
                "name": "kernel_worker",
                "cpu_usage": 5.2,
                "memory_usage": 8192,
                "running": true,
                "command": "[kernel_worker]"
            }),
            json!({
                "pid": 456,
                "name": "semantic_cli",
                "cpu_usage": 12.8,
                "memory_usage": 16384,
                "running": true,
                "command": "/usr/bin/semantic_cli --repl"
            }),
            json!({
                "pid": 789,
                "name": "orchestrator",
                "cpu_usage": 8.5,
                "memory_usage": 32768,
                "running": true,
                "command": "/usr/bin/orchestrator --daemon"
            }),
            json!({
                "pid": 1001,
                "name": "agent_manager",
                "cpu_usage": 3.2,
                "memory_usage": 12288,
                "running": true,
                "command": "/usr/bin/agent_manager --config /etc/agents.conf"
            }),
            json!({
                "pid": 1234,
                "name": "log_service",
                "cpu_usage": 1.8,
                "memory_usage": 4096,
                "running": true,
                "command": "/usr/bin/log_service --syslog"
            }),
        ];

        Ok(ContextData::new(processes))
    }

    fn name(&self) -> &str {
        &self.name
    }
}

/// Mock agent data loader (simulates orchestrator API)
pub struct MockAgentLoader {
    name: String,
}

impl MockAgentLoader {
    pub fn new() -> Self {
        Self {
            name: "MockAgentLoader".to_string(),
        }
    }
}

impl ContextLoader for MockAgentLoader {
    fn load(&self) -> Result<ContextData> {
        // Simulate orchestrator API call latency (15-60ms)
        let latency = Duration::from_millis(15 + (rand::random::<u64>() % 45));
        std::thread::sleep(latency);

        let agents = vec![
            json!({
                "id": "agent_001",
                "name": "FileSystemAgent",
                "status": "active",
                "active": true,
                "load": 25.5,
                "capabilities": ["file_read", "file_write", "directory_list"]
            }),
            json!({
                "id": "agent_002",
                "name": "NetworkAgent", 
                "status": "active",
                "active": true,
                "load": 42.8,
                "capabilities": ["http_request", "tcp_connect", "dns_resolve"]
            }),
            json!({
                "id": "agent_003",
                "name": "DatabaseAgent",
                "status": "idle",
                "active": true,
                "load": 8.2,
                "capabilities": ["sql_query", "transaction", "backup"]
            }),
            json!({
                "id": "agent_004",
                "name": "SecurityAgent",
                "status": "active",
                "active": true,
                "load": 15.7,
                "capabilities": ["auth_check", "permission_verify", "audit_log"]
            }),
            json!({
                "id": "agent_005",
                "name": "MonitoringAgent",
                "status": "maintenance",
                "active": false,
                "load": 0.0,
                "capabilities": ["metric_collect", "alert_send", "health_check"]
            }),
        ];

        Ok(ContextData::new(agents))
    }

    fn name(&self) -> &str {
        &self.name
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_user_loader() {
        let loader = MockUserLoader::new();
        assert_eq!(loader.name(), "MockUserLoader");
        
        let result = loader.load();
        assert!(result.is_ok());
        
        let data = result.unwrap();
        assert!(!data.items.is_empty());
        assert_eq!(data.items.len(), 5);
        
        // Check first user structure
        let first_user = &data.items[0];
        assert!(first_user.get("id").is_some());
        assert!(first_user.get("name").is_some());
        assert!(first_user.get("email").is_some());
        assert!(first_user.get("active").is_some());
        assert!(first_user.get("roles").is_some());
    }

    #[test]
    fn test_mock_log_loader() {
        let loader = MockLogLoader::new();
        assert_eq!(loader.name(), "MockLogLoader");
        
        let result = loader.load();
        assert!(result.is_ok());
        
        let data = result.unwrap();
        assert!(!data.items.is_empty());
        assert_eq!(data.items.len(), 5);
        
        // Check first log structure
        let first_log = &data.items[0];
        assert!(first_log.get("id").is_some());
        assert!(first_log.get("timestamp").is_some());
        assert!(first_log.get("level").is_some());
        assert!(first_log.get("message").is_some());
        assert!(first_log.get("source").is_some());
        assert!(first_log.get("metadata").is_some());
    }

    #[test]
    fn test_mock_process_loader() {
        let loader = MockProcessLoader::new();
        assert_eq!(loader.name(), "MockProcessLoader");
        
        let result = loader.load();
        assert!(result.is_ok());
        
        let data = result.unwrap();
        assert!(!data.items.is_empty());
        assert_eq!(data.items.len(), 6);
        
        // Check first process structure
        let first_process = &data.items[0];
        assert!(first_process.get("pid").is_some());
        assert!(first_process.get("name").is_some());
        assert!(first_process.get("cpu_usage").is_some());
        assert!(first_process.get("memory_usage").is_some());
        assert!(first_process.get("running").is_some());
        assert!(first_process.get("command").is_some());
    }

    #[test]
    fn test_mock_agent_loader() {
        let loader = MockAgentLoader::new();
        assert_eq!(loader.name(), "MockAgentLoader");
        
        let result = loader.load();
        assert!(result.is_ok());
        
        let data = result.unwrap();
        assert!(!data.items.is_empty());
        assert_eq!(data.items.len(), 5);
        
        // Check first agent structure
        let first_agent = &data.items[0];
        assert!(first_agent.get("id").is_some());
        assert!(first_agent.get("name").is_some());
        assert!(first_agent.get("status").is_some());
        assert!(first_agent.get("active").is_some());
        assert!(first_agent.get("load").is_some());
        assert!(first_agent.get("capabilities").is_some());
    }

    #[test]
    fn test_loader_performance() {
        let user_loader = MockUserLoader::new();
        let log_loader = MockLogLoader::new();
        let process_loader = MockProcessLoader::new();
        let agent_loader = MockAgentLoader::new();
        
        // Test that all loaders complete within reasonable time (< 100ms each)
        let start = Instant::now();
        let _ = user_loader.load();
        assert!(start.elapsed().as_millis() < 100);
        
        let start = Instant::now();
        let _ = log_loader.load();
        assert!(start.elapsed().as_millis() < 100);
        
        let start = Instant::now();
        let _ = process_loader.load();
        assert!(start.elapsed().as_millis() < 100);
        
        let start = Instant::now();
        let _ = agent_loader.load();
        assert!(start.elapsed().as_millis() < 100);
    }

    #[test]
    fn test_data_consistency() {
        let loader = MockUserLoader::new();
        
        // Load data multiple times and verify consistency
        let data1 = loader.load().unwrap();
        let data2 = loader.load().unwrap();
        
        // Should have same structure (though values may vary due to randomness)
        assert_eq!(data1.items.len(), data2.items.len());
        
        // Check that all required fields are present in both loads
        for (item1, item2) in data1.items.iter().zip(data2.items.iter()) {
            assert!(item1.get("id").is_some());
            assert!(item2.get("id").is_some());
            assert!(item1.get("name").is_some());
            assert!(item2.get("name").is_some());
        }
    }
}