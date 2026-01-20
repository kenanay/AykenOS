//! Command validation against system capabilities
//!
//! This module provides comprehensive validation of commands against available
//! system capabilities, ensuring that only valid and safe commands are executed.

use crate::types::*;
use crate::error::CompilationError;
use std::collections::{HashMap, HashSet};
use tracing::{debug, warn};
use uuid::Uuid;

/// Command validator that checks commands against system capabilities
pub struct CommandValidator {
    /// Available system commands
    available_commands: HashSet<String>,
    /// Command capability mappings
    command_capabilities: HashMap<String, CommandCapability>,
    /// System resource limits
    system_limits: SystemLimits,
}

/// Capability information for a command
#[derive(Debug, Clone)]
pub struct CommandCapability {
    /// Required permissions
    pub required_permissions: Vec<Permission>,
    /// Resource requirements
    pub resource_requirements: ResourceRequirements,
    /// Security constraints
    pub security_constraints: Vec<SecurityConstraint>,
    /// Whether command is considered safe
    pub is_safe: bool,
    /// Isolation level required
    pub required_isolation: IsolationLevel,
}

/// System resource limits
#[derive(Debug, Clone)]
pub struct SystemLimits {
    /// Maximum CPU usage allowed
    pub max_cpu_usage: f32,
    /// Maximum memory usage in bytes
    pub max_memory_usage: u64,
    /// Maximum disk space usage in bytes
    pub max_disk_usage: u64,
    /// Maximum network bandwidth in bytes/sec
    pub max_network_bandwidth: u64,
}

impl Default for SystemLimits {
    fn default() -> Self {
        Self {
            max_cpu_usage: 0.8, // 80% CPU
            max_memory_usage: 8 * 1024 * 1024 * 1024, // 8GB
            max_disk_usage: 100 * 1024 * 1024 * 1024, // 100GB
            max_network_bandwidth: 100 * 1024 * 1024, // 100MB/s
        }
    }
}

impl CommandValidator {
    /// Create a new command validator
    pub fn new() -> Self {
        let mut validator = Self {
            available_commands: HashSet::new(),
            command_capabilities: HashMap::new(),
            system_limits: SystemLimits::default(),
        };

        validator.initialize_default_commands();
        validator
    }

    /// Initialize default system commands and their capabilities
    fn initialize_default_commands(&mut self) {
        // Safe query commands
        self.register_command("echo", CommandCapability {
            required_permissions: vec![],
            resource_requirements: ResourceRequirements {
                cpu_usage: 0.01,
                memory_usage: 1024 * 1024, // 1MB
                disk_space: 0,
                network_bandwidth: 0,
                exclusive_resources: vec![],
            },
            security_constraints: vec![],
            is_safe: true,
            required_isolation: IsolationLevel::None,
        });

        self.register_command("ls", CommandCapability {
            required_permissions: vec![
                Permission {
                    resource: "filesystem".to_string(),
                    access_type: AccessType::Read,
                }
            ],
            resource_requirements: ResourceRequirements {
                cpu_usage: 0.05,
                memory_usage: 10 * 1024 * 1024, // 10MB
                disk_space: 0,
                network_bandwidth: 0,
                exclusive_resources: vec![],
            },
            security_constraints: vec![
                SecurityConstraint {
                    constraint_type: ConstraintType::FileAccess,
                    description: "Read-only file system access".to_string(),
                    enforced: true,
                }
            ],
            is_safe: true,
            required_isolation: IsolationLevel::None,
        });

        self.register_command("cat", CommandCapability {
            required_permissions: vec![
                Permission {
                    resource: "filesystem".to_string(),
                    access_type: AccessType::Read,
                }
            ],
            resource_requirements: ResourceRequirements {
                cpu_usage: 0.05,
                memory_usage: 50 * 1024 * 1024, // 50MB
                disk_space: 0,
                network_bandwidth: 0,
                exclusive_resources: vec![],
            },
            security_constraints: vec![
                SecurityConstraint {
                    constraint_type: ConstraintType::FileAccess,
                    description: "Read-only file access".to_string(),
                    enforced: true,
                }
            ],
            is_safe: true,
            required_isolation: IsolationLevel::None,
        });

        // Potentially dangerous commands requiring higher security
        self.register_command("rm", CommandCapability {
            required_permissions: vec![
                Permission {
                    resource: "filesystem".to_string(),
                    access_type: AccessType::Delete,
                }
            ],
            resource_requirements: ResourceRequirements {
                cpu_usage: 0.1,
                memory_usage: 20 * 1024 * 1024, // 20MB
                disk_space: 0,
                network_bandwidth: 0,
                exclusive_resources: vec!["filesystem_write".to_string()],
            },
            security_constraints: vec![
                SecurityConstraint {
                    constraint_type: ConstraintType::FileAccess,
                    description: "File deletion requires explicit approval".to_string(),
                    enforced: true,
                }
            ],
            is_safe: false,
            required_isolation: IsolationLevel::Sandboxed,
        });

        self.register_command("chmod", CommandCapability {
            required_permissions: vec![
                Permission {
                    resource: "filesystem".to_string(),
                    access_type: AccessType::Write,
                }
            ],
            resource_requirements: ResourceRequirements {
                cpu_usage: 0.05,
                memory_usage: 10 * 1024 * 1024, // 10MB
                disk_space: 0,
                network_bandwidth: 0,
                exclusive_resources: vec!["filesystem_permissions".to_string()],
            },
            security_constraints: vec![
                SecurityConstraint {
                    constraint_type: ConstraintType::FileAccess,
                    description: "Permission changes require approval".to_string(),
                    enforced: true,
                }
            ],
            is_safe: false,
            required_isolation: IsolationLevel::Sandboxed,
        });

        // System management commands
        self.register_command("ps", CommandCapability {
            required_permissions: vec![
                Permission {
                    resource: "process_info".to_string(),
                    access_type: AccessType::Read,
                }
            ],
            resource_requirements: ResourceRequirements {
                cpu_usage: 0.1,
                memory_usage: 20 * 1024 * 1024, // 20MB
                disk_space: 0,
                network_bandwidth: 0,
                exclusive_resources: vec![],
            },
            security_constraints: vec![],
            is_safe: true,
            required_isolation: IsolationLevel::None,
        });

        self.register_command("kill", CommandCapability {
            required_permissions: vec![
                Permission {
                    resource: "process_management".to_string(),
                    access_type: AccessType::Execute,
                }
            ],
            resource_requirements: ResourceRequirements {
                cpu_usage: 0.05,
                memory_usage: 5 * 1024 * 1024, // 5MB
                disk_space: 0,
                network_bandwidth: 0,
                exclusive_resources: vec!["process_control".to_string()],
            },
            security_constraints: vec![
                SecurityConstraint {
                    constraint_type: ConstraintType::ProcessExecution,
                    description: "Process termination requires approval".to_string(),
                    enforced: true,
                }
            ],
            is_safe: false,
            required_isolation: IsolationLevel::FullyIsolated,
        });

        // Network commands
        self.register_command("ping", CommandCapability {
            required_permissions: vec![
                Permission {
                    resource: "network".to_string(),
                    access_type: AccessType::Read,
                }
            ],
            resource_requirements: ResourceRequirements {
                cpu_usage: 0.1,
                memory_usage: 10 * 1024 * 1024, // 10MB
                disk_space: 0,
                network_bandwidth: 1024 * 1024, // 1MB/s
                exclusive_resources: vec![],
            },
            security_constraints: vec![
                SecurityConstraint {
                    constraint_type: ConstraintType::NetworkAccess,
                    description: "Network access monitoring required".to_string(),
                    enforced: true,
                }
            ],
            is_safe: true,
            required_isolation: IsolationLevel::Sandboxed,
        });

        // Semantic CLI specific commands
        self.register_command("validate_query_parameters", CommandCapability {
            required_permissions: vec![],
            resource_requirements: ResourceRequirements {
                cpu_usage: 0.05,
                memory_usage: 10 * 1024 * 1024, // 10MB
                disk_space: 0,
                network_bandwidth: 0,
                exclusive_resources: vec![],
            },
            security_constraints: vec![],
            is_safe: true,
            required_isolation: IsolationLevel::None,
        });

        self.register_command("execute_query", CommandCapability {
            required_permissions: vec![
                Permission {
                    resource: "query_engine".to_string(),
                    access_type: AccessType::Execute,
                }
            ],
            resource_requirements: ResourceRequirements {
                cpu_usage: 0.2,
                memory_usage: 100 * 1024 * 1024, // 100MB
                disk_space: 0,
                network_bandwidth: 0,
                exclusive_resources: vec![],
            },
            security_constraints: vec![],
            is_safe: true,
            required_isolation: IsolationLevel::None,
        });

        self.register_command("validate_command_security", CommandCapability {
            required_permissions: vec![
                Permission {
                    resource: "security_validator".to_string(),
                    access_type: AccessType::Execute,
                }
            ],
            resource_requirements: ResourceRequirements {
                cpu_usage: 0.1,
                memory_usage: 50 * 1024 * 1024, // 50MB
                disk_space: 0,
                network_bandwidth: 0,
                exclusive_resources: vec!["security_engine".to_string()],
            },
            security_constraints: vec![],
            is_safe: true,
            required_isolation: IsolationLevel::None,
        });

        self.register_command("execute_command", CommandCapability {
            required_permissions: vec![
                Permission {
                    resource: "command_execution".to_string(),
                    access_type: AccessType::Execute,
                }
            ],
            resource_requirements: ResourceRequirements {
                cpu_usage: 0.3,
                memory_usage: 200 * 1024 * 1024, // 200MB
                disk_space: 0,
                network_bandwidth: 0,
                exclusive_resources: vec!["execution_engine".to_string()],
            },
            security_constraints: vec![
                SecurityConstraint {
                    constraint_type: ConstraintType::ProcessExecution,
                    description: "Command execution requires validation".to_string(),
                    enforced: true,
                }
            ],
            is_safe: false,
            required_isolation: IsolationLevel::Sandboxed,
        });
    }

    /// Register a command with its capabilities
    pub fn register_command(&mut self, command: &str, capability: CommandCapability) {
        self.available_commands.insert(command.to_string());
        self.command_capabilities.insert(command.to_string(), capability);
    }

    /// Validate commands in an execution plan
    pub async fn validate_commands(
        &self,
        plan: &ExecutionPlan,
        security_context: &SecurityContext,
    ) -> Result<Vec<ValidatedCommand>, CompilationError> {
        debug!("Validating commands for plan: {:?}", plan.id);

        let mut validated_commands = Vec::new();

        for step in &plan.steps {
            let validated_command = self.validate_single_command(step, security_context).await?;
            validated_commands.push(validated_command);
        }

        // Validate aggregate resource requirements
        self.validate_aggregate_resources(&validated_commands)?;

        debug!("Successfully validated {} commands", validated_commands.len());
        Ok(validated_commands)
    }

    /// Validate a single command step
    async fn validate_single_command(
        &self,
        step: &PlanStep,
        security_context: &SecurityContext,
    ) -> Result<ValidatedCommand, CompilationError> {
        debug!("Validating command: {}", step.command);

        // Extract base command (first word)
        let base_command = step.command.split_whitespace().next()
            .ok_or_else(|| CompilationError::InvalidCommand {
                step_id: step.id,
                reason: "Empty command".to_string(),
            })?;

        // Check if command is available
        if !self.available_commands.contains(base_command) {
            return Err(CompilationError::CommandNotAvailable {
                command: base_command.to_string(),
                available_commands: self.available_commands.iter().cloned().collect(),
            });
        }

        // Get command capability
        let capability = self.command_capabilities.get(base_command)
            .ok_or_else(|| CompilationError::InvalidCommand {
                step_id: step.id,
                reason: format!("No capability information for command: {}", base_command),
            })?;

        // Validate permissions
        self.validate_command_permissions(capability, security_context)?;

        // Validate resource requirements
        self.validate_command_resources(capability)?;

        // Validate isolation requirements
        self.validate_isolation_requirements(capability, security_context)?;

        // Parse command arguments
        let arguments: Vec<String> = step.command.split_whitespace()
            .skip(1)
            .map(|s| s.to_string())
            .collect();

        // Generate security constraints
        let mut security_constraints = capability.security_constraints.clone();
        
        // Add step-specific constraints
        if !capability.is_safe {
            security_constraints.push(SecurityConstraint {
                constraint_type: ConstraintType::ProcessExecution,
                description: format!("Unsafe command {} requires monitoring", base_command),
                enforced: true,
            });
        }

        // Create validated command
        let validated_command = ValidatedCommand {
            command: base_command.to_string(),
            arguments,
            environment: self.generate_safe_environment(step),
            working_directory: self.determine_safe_working_directory(step, security_context),
            security_constraints,
        };

        debug!("Successfully validated command: {}", base_command);
        Ok(validated_command)
    }

    /// Validate command permissions against security context
    fn validate_command_permissions(
        &self,
        capability: &CommandCapability,
        security_context: &SecurityContext,
    ) -> Result<(), CompilationError> {
        for required_permission in &capability.required_permissions {
            let has_permission = security_context.permissions.iter()
                .any(|p| p.resource == required_permission.resource && 
                        p.access_type == required_permission.access_type);

            if !has_permission {
                return Err(CompilationError::InsufficientPermissions {
                    required_permission: required_permission.clone(),
                    available_permissions: security_context.permissions.clone(),
                });
            }
        }

        Ok(())
    }

    /// Validate command resource requirements
    fn validate_command_resources(&self, capability: &CommandCapability) -> Result<(), CompilationError> {
        let req = &capability.resource_requirements;

        if req.cpu_usage > self.system_limits.max_cpu_usage {
            return Err(CompilationError::ResourceLimitExceeded {
                resource_type: "CPU".to_string(),
                requested: req.cpu_usage as u64,
                available: (self.system_limits.max_cpu_usage * 100.0) as u64,
            });
        }

        if req.memory_usage > self.system_limits.max_memory_usage {
            return Err(CompilationError::ResourceLimitExceeded {
                resource_type: "Memory".to_string(),
                requested: req.memory_usage,
                available: self.system_limits.max_memory_usage,
            });
        }

        if req.disk_space > self.system_limits.max_disk_usage {
            return Err(CompilationError::ResourceLimitExceeded {
                resource_type: "Disk".to_string(),
                requested: req.disk_space,
                available: self.system_limits.max_disk_usage,
            });
        }

        if req.network_bandwidth > self.system_limits.max_network_bandwidth {
            return Err(CompilationError::ResourceLimitExceeded {
                resource_type: "Network".to_string(),
                requested: req.network_bandwidth,
                available: self.system_limits.max_network_bandwidth,
            });
        }

        Ok(())
    }

    /// Validate isolation requirements
    fn validate_isolation_requirements(
        &self,
        capability: &CommandCapability,
        security_context: &SecurityContext,
    ) -> Result<(), CompilationError> {
        // Check if current isolation level meets requirements
        let current_level = &security_context.isolation_level;
        let required_level = &capability.required_isolation;

        let isolation_hierarchy = [
            IsolationLevel::None,
            IsolationLevel::Sandboxed,
            IsolationLevel::FullyIsolated,
        ];

        let current_index = isolation_hierarchy.iter().position(|l| l == current_level)
            .unwrap_or(0);
        let required_index = isolation_hierarchy.iter().position(|l| l == required_level)
            .unwrap_or(0);

        if current_index < required_index {
            return Err(CompilationError::InsufficientIsolation {
                required: required_level.clone(),
                current: current_level.clone(),
            });
        }

        Ok(())
    }

    /// Validate aggregate resource requirements across all commands
    fn validate_aggregate_resources(&self, commands: &[ValidatedCommand]) -> Result<(), CompilationError> {
        let mut total_cpu = 0.0;
        let mut total_memory = 0u64;
        let mut total_disk = 0u64;
        let mut total_network = 0u64;

        for command in commands {
            if let Some(capability) = self.command_capabilities.get(&command.command) {
                total_cpu += capability.resource_requirements.cpu_usage;
                total_memory += capability.resource_requirements.memory_usage;
                total_disk += capability.resource_requirements.disk_space;
                total_network += capability.resource_requirements.network_bandwidth;
            }
        }

        // Check aggregate limits (with some buffer for coordination overhead)
        if total_cpu > self.system_limits.max_cpu_usage * 0.9 {
            return Err(CompilationError::ResourceLimitExceeded {
                resource_type: "Aggregate CPU".to_string(),
                requested: (total_cpu * 100.0) as u64,
                available: (self.system_limits.max_cpu_usage * 90.0) as u64,
            });
        }

        if total_memory > self.system_limits.max_memory_usage / 2 {
            return Err(CompilationError::ResourceLimitExceeded {
                resource_type: "Aggregate Memory".to_string(),
                requested: total_memory,
                available: self.system_limits.max_memory_usage / 2,
            });
        }

        Ok(())
    }

    /// Generate safe environment variables for command execution
    fn generate_safe_environment(&self, step: &PlanStep) -> HashMap<String, String> {
        let mut env = HashMap::new();

        // Add safe default environment variables
        env.insert("PATH".to_string(), "/usr/local/bin:/usr/bin:/bin".to_string());
        env.insert("LANG".to_string(), "C.UTF-8".to_string());
        env.insert("TERM".to_string(), "xterm".to_string());

        // Add step-specific environment variables from parameters
        for (key, value) in &step.parameters {
            if let Some(str_value) = value.as_str() {
                // Only allow safe environment variable names
                if self.is_safe_env_var(key) {
                    env.insert(key.clone(), str_value.to_string());
                }
            }
        }

        env
    }

    /// Check if environment variable name is safe
    fn is_safe_env_var(&self, name: &str) -> bool {
        // Allow alphanumeric and underscore, but not system-critical variables
        let safe_pattern = regex::Regex::new(r"^[A-Za-z_][A-Za-z0-9_]*$").unwrap();
        let dangerous_vars = ["LD_PRELOAD", "LD_LIBRARY_PATH", "PATH", "HOME", "USER"];

        safe_pattern.is_match(name) && !dangerous_vars.contains(&name)
    }

    /// Determine safe working directory for command execution
    fn determine_safe_working_directory(&self, step: &PlanStep, security_context: &SecurityContext) -> String {
        // Default to a safe sandbox directory
        let default_dir = "/tmp/semantic_cli_sandbox";

        // Check if step specifies a working directory
        if let Some(dir_value) = step.parameters.get("working_directory") {
            if let Some(dir_str) = dir_value.as_str() {
                // Validate the directory is safe
                if self.is_safe_directory(dir_str, security_context) {
                    return dir_str.to_string();
                } else {
                    warn!("Unsafe working directory specified: {}, using default", dir_str);
                }
            }
        }

        default_dir.to_string()
    }

    /// Check if directory is safe for command execution
    fn is_safe_directory(&self, dir: &str, security_context: &SecurityContext) -> bool {
        // Prevent access to system directories unless explicitly allowed
        let system_dirs = ["/", "/bin", "/sbin", "/usr", "/etc", "/boot", "/sys", "/proc"];
        
        // Allow access to system directories only with full isolation
        if system_dirs.iter().any(|&sys_dir| dir.starts_with(sys_dir)) {
            return security_context.isolation_level == IsolationLevel::FullyIsolated;
        }

        // Allow access to user directories and tmp
        dir.starts_with("/tmp") || 
        dir.starts_with("/home") || 
        dir.starts_with("/var/tmp") ||
        dir.starts_with("/opt/ayken")
    }

    /// Get available commands
    pub fn get_available_commands(&self) -> &HashSet<String> {
        &self.available_commands
    }

    /// Get command capability information
    pub fn get_command_capability(&self, command: &str) -> Option<&CommandCapability> {
        self.command_capabilities.get(command)
    }

    /// Update system limits
    pub fn update_system_limits(&mut self, limits: SystemLimits) {
        self.system_limits = limits;
    }
}

impl Default for CommandValidator {
    fn default() -> Self {
        Self::new()
    }
}