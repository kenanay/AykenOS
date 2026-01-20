//! Main compiler implementation
//!
//! This module provides the main compiler implementation that orchestrates
//! the compilation process using validation, security, and policy components.

use crate::types::*;
use crate::error::CompilationError;
use crate::compiler::{SecurityValidator, PolicyEngine, CommandValidator, SecurityPolicies};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Configuration for the command compiler
#[derive(Debug, Clone)]
pub struct CompilerConfig {
    /// Whether to require explicit approval for all commands
    pub require_explicit_approval: bool,
    /// Maximum number of commands per compilation
    pub max_commands_per_compilation: usize,
    /// Default isolation level
    pub default_isolation_level: IsolationLevel,
    /// Whether to enable strict security mode
    pub strict_security_mode: bool,
}

impl Default for CompilerConfig {
    fn default() -> Self {
        Self {
            require_explicit_approval: true,
            max_commands_per_compilation: 20,
            default_isolation_level: IsolationLevel::Sandboxed,
            strict_security_mode: true,
        }
    }
}

/// Main command compiler that orchestrates the compilation process
pub struct CommandCompiler {
    /// Security validator
    security_validator: Arc<RwLock<SecurityValidator>>,
    /// Policy engine
    policy_engine: Arc<RwLock<PolicyEngine>>,
    /// Command validator
    command_validator: Arc<RwLock<CommandValidator>>,
    /// Compiler configuration
    config: CompilerConfig,
}

impl CommandCompiler {
    /// Create a new command compiler
    pub fn new() -> Self {
        let config = CompilerConfig::default();
        Self::with_config(config)
    }

    /// Create compiler with custom configuration
    pub fn with_config(config: CompilerConfig) -> Self {
        // Create security policies that match the compiler config
        let security_policies = SecurityPolicies {
            default_isolation_level: config.default_isolation_level,
            require_approval_for_all: config.require_explicit_approval,
            max_risk_without_approval: if config.strict_security_mode { RiskLevel::Low } else { RiskLevel::Medium },
            strict_permission_checking: config.strict_security_mode,
            allowed_filesystem_paths: vec![
                "/tmp".to_string(),
                "/var/tmp".to_string(),
                "/home".to_string(),
                "/opt/ayken".to_string(),
            ],
            blocked_command_patterns: if config.strict_security_mode {
                vec![
                    "rm -rf /".to_string(),
                    "dd if=/dev/zero".to_string(),
                    ":(){ :|:& };:".to_string(),
                    "chmod 777 /".to_string(),
                    "chown root:root /".to_string(),
                    "mkfs".to_string(),
                    "fdisk".to_string(),
                ]
            } else {
                vec![]
            },
        };

        Self {
            security_validator: Arc::new(RwLock::new(SecurityValidator::with_policies(security_policies))),
            policy_engine: Arc::new(RwLock::new(PolicyEngine::new())),
            command_validator: Arc::new(RwLock::new(CommandValidator::new())),
            config,
        }
    }

    /// Compile execution plan into validated commands
    /// This is the main entry point that implements the security boundary
    pub async fn compile_plan(&self, plan: &ExecutionPlan) -> Result<CompiledCommands, CompilationError> {
        info!("Starting compilation for plan: {:?}", plan.id);

        // Step 1: Basic plan validation
        self.validate_plan_structure(plan)?;

        // Step 2: Security validation - CRITICAL SECURITY BOUNDARY
        let mut security_validator = self.security_validator.write().await;
        let security_context = security_validator.validate_plan_security(plan).await?;
        drop(security_validator); // Release lock early

        // Step 3: Policy evaluation for approval requirements
        let mut policy_engine = self.policy_engine.write().await;
        let approval_requirements = policy_engine.evaluate_approval_requirements(plan, &security_context).await?;
        drop(policy_engine); // Release lock early

        // Step 4: Command validation against system capabilities
        let command_validator = self.command_validator.read().await;
        let validated_commands = command_validator.validate_commands(plan, &security_context).await?;
        drop(command_validator); // Release lock early

        // Step 5: Final security check - ensure no security boundary bypass
        self.final_security_check(plan, &security_context, &validated_commands)?;

        // Step 6: Generate compilation metadata
        let metadata = self.generate_compilation_metadata(plan, &security_context);

        // Step 7: Determine final approval requirement
        let approval_required = self.determine_final_approval_requirement(
            plan,
            &approval_requirements,
            &security_context,
        );

        let compiled = CompiledCommands {
            commands: validated_commands,
            security_context,
            approval_required,
            metadata,
        };

        info!("Compilation completed for plan: {:?}, approval_required: {}", 
              plan.id, approval_required);

        Ok(compiled)
    }

    /// Validate basic plan structure
    fn validate_plan_structure(&self, plan: &ExecutionPlan) -> Result<(), CompilationError> {
        debug!("Validating plan structure for plan: {:?}", plan.id);

        // Check command count limits
        if plan.steps.len() > self.config.max_commands_per_compilation {
            return Err(CompilationError::TooManyCommands {
                requested: plan.steps.len(),
                maximum: self.config.max_commands_per_compilation,
            });
        }

        // Validate that all steps have valid commands
        for step in &plan.steps {
            if step.command.trim().is_empty() {
                return Err(CompilationError::InvalidCommand {
                    step_id: step.id,
                    reason: "Empty command".to_string(),
                });
            }

            // Check for potentially dangerous command patterns in strict mode
            if self.config.strict_security_mode {
                self.validate_command_safety(&step.command, step.id)?;
            }
        }

        // Validate dependencies are consistent
        self.validate_dependencies(plan)?;

        Ok(())
    }

    /// Validate command safety in strict security mode
    fn validate_command_safety(&self, command: &str, step_id: uuid::Uuid) -> Result<(), CompilationError> {
        let dangerous_patterns = [
            "rm -rf /",
            "dd if=/dev/zero",
            ":(){ :|:& };:",  // Fork bomb
            "chmod 777 /",
            "chown root:root /",
            "mkfs",
            "fdisk",
            "format",
            "> /dev/sda",
            "shutdown",
            "reboot",
            "halt",
        ];

        for pattern in &dangerous_patterns {
            if command.contains(pattern) {
                warn!("Dangerous command pattern detected: {} in command: {}", pattern, command);
                return Err(CompilationError::DangerousCommand {
                    command: command.to_string(),
                    reason: format!("Contains dangerous pattern: {}", pattern),
                });
            }
        }

        // Check for suspicious command combinations
        if command.contains("sudo") && (command.contains("rm") || command.contains("chmod")) {
            warn!("Suspicious sudo command combination detected: {}", command);
            return Err(CompilationError::DangerousCommand {
                command: command.to_string(),
                reason: "Suspicious sudo command combination".to_string(),
            });
        }

        Ok(())
    }

    /// Validate plan dependencies are consistent
    fn validate_dependencies(&self, plan: &ExecutionPlan) -> Result<(), CompilationError> {
        let step_ids: std::collections::HashSet<_> = plan.steps.iter().map(|s| s.id).collect();

        for dependency in &plan.dependencies {
            if !step_ids.contains(&dependency.prerequisite) {
                return Err(CompilationError::InvalidDependency {
                    dependency_id: format!("{:?}", dependency),
                    reason: "Prerequisite step not found in plan".to_string(),
                });
            }

            if !step_ids.contains(&dependency.dependent) {
                return Err(CompilationError::InvalidDependency {
                    dependency_id: format!("{:?}", dependency),
                    reason: "Dependent step not found in plan".to_string(),
                });
            }

            if dependency.prerequisite == dependency.dependent {
                return Err(CompilationError::InvalidDependency {
                    dependency_id: format!("{:?}", dependency),
                    reason: "Step cannot depend on itself".to_string(),
                });
            }
        }

        // Check for circular dependencies (simple check)
        self.check_circular_dependencies(plan)?;

        Ok(())
    }

    /// Check for circular dependencies in the plan
    fn check_circular_dependencies(&self, plan: &ExecutionPlan) -> Result<(), CompilationError> {
        use std::collections::{HashMap, HashSet};

        let mut graph: HashMap<uuid::Uuid, Vec<uuid::Uuid>> = HashMap::new();
        
        // Build dependency graph
        for step in &plan.steps {
            graph.insert(step.id, Vec::new());
        }
        
        for dependency in &plan.dependencies {
            graph.entry(dependency.prerequisite)
                .or_insert_with(Vec::new)
                .push(dependency.dependent);
        }

        // Simple cycle detection using DFS
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();

        for step_id in graph.keys() {
            if !visited.contains(step_id) {
                if self.has_cycle(&graph, *step_id, &mut visited, &mut rec_stack) {
                    return Err(CompilationError::InvalidDependency {
                        dependency_id: format!("circular_dependency_{}", step_id),
                        reason: "Circular dependency detected in plan".to_string(),
                    });
                }
            }
        }

        Ok(())
    }

    /// Helper function for cycle detection
    fn has_cycle(
        &self,
        graph: &std::collections::HashMap<uuid::Uuid, Vec<uuid::Uuid>>,
        node: uuid::Uuid,
        visited: &mut std::collections::HashSet<uuid::Uuid>,
        rec_stack: &mut std::collections::HashSet<uuid::Uuid>,
    ) -> bool {
        visited.insert(node);
        rec_stack.insert(node);

        if let Some(neighbors) = graph.get(&node) {
            for &neighbor in neighbors {
                if !visited.contains(&neighbor) {
                    if self.has_cycle(graph, neighbor, visited, rec_stack) {
                        return true;
                    }
                } else if rec_stack.contains(&neighbor) {
                    return true;
                }
            }
        }

        rec_stack.remove(&node);
        false
    }

    /// Final security check to ensure no security boundary bypass
    fn final_security_check(
        &self,
        plan: &ExecutionPlan,
        security_context: &SecurityContext,
        validated_commands: &[ValidatedCommand],
    ) -> Result<(), CompilationError> {
        debug!("Performing final security check for plan: {:?}", plan.id);

        // Ensure all commands have appropriate security constraints
        for (_i, command) in validated_commands.iter().enumerate() {
            // Safe commands don't need security constraints
            if self.is_command_safe(&command.command) {
                continue;
            }
            
            // Non-safe commands or commands requiring isolation need security constraints
            let needs_constraints = !self.is_command_safe(&command.command) || 
                                  security_context.isolation_level == IsolationLevel::FullyIsolated;
            
            if command.security_constraints.is_empty() && needs_constraints {
                warn!("Command {} lacks security constraints despite isolation requirements", command.command);
                return Err(CompilationError::SecurityViolation {
                    reason: format!("Command '{}' lacks required security constraints", command.command),
                    plan_id: plan.id,
                });
            }

            // Verify working directory is safe
            if !self.is_safe_working_directory(&command.working_directory, security_context) {
                println!("Working directory rejected: {}, isolation: {:?}", 
                        command.working_directory, security_context.isolation_level);
                return Err(CompilationError::SecurityViolation {
                    reason: format!("Unsafe working directory: {}", command.working_directory),
                    plan_id: plan.id,
                });
            }

            // Check for environment variable injection
            for (key, value) in &command.environment {
                if self.is_dangerous_env_var(key, value) {
                    return Err(CompilationError::SecurityViolation {
                        reason: format!("Dangerous environment variable: {}={}", key, value),
                        plan_id: plan.id,
                    });
                }
            }
        }

        // Verify resource limits are appropriate for risk level
        let risk_level = plan.risk_assessment.risk_level;
        if risk_level >= RiskLevel::High && security_context.isolation_level == IsolationLevel::None {
            return Err(CompilationError::SecurityViolation {
                reason: "High risk operations require isolation".to_string(),
                plan_id: plan.id,
            });
        }

        debug!("Final security check passed for plan: {:?}", plan.id);
        Ok(())
    }

    /// Check if a command is considered safe
    fn is_command_safe(&self, command: &str) -> bool {
        let safe_commands = ["echo", "ls", "cat", "pwd", "whoami", "date", "ps"];
        safe_commands.contains(&command)
    }

    /// Check if working directory is safe
    fn is_safe_working_directory(&self, dir: &str, security_context: &SecurityContext) -> bool {
        // System directories that require full isolation
        // Note: We need to be careful with "/" - it should only match the root directory exactly
        // or when followed by a path separator
        let system_dirs = ["/bin", "/sbin", "/usr", "/etc", "/boot", "/sys", "/proc"];
        
        // Check for system directories
        for &sys_dir in &system_dirs {
            if dir.starts_with(sys_dir) && (dir.len() == sys_dir.len() || dir.chars().nth(sys_dir.len()) == Some('/')) {
                return security_context.isolation_level == IsolationLevel::FullyIsolated;
            }
        }
        
        // Special case for root directory "/"
        if dir == "/" {
            return security_context.isolation_level == IsolationLevel::FullyIsolated;
        }

        // Allow safe directories
        dir.starts_with("/tmp") || 
        dir.starts_with("/home") || 
        dir.starts_with("/var/tmp") ||
        dir.starts_with("/opt/ayken")
    }

    /// Check if environment variable is dangerous
    fn is_dangerous_env_var(&self, key: &str, value: &str) -> bool {
        let dangerous_vars = ["LD_PRELOAD", "LD_LIBRARY_PATH"];
        
        if dangerous_vars.contains(&key) {
            return true;
        }

        // Check for injection attempts
        if value.contains("$(") || value.contains("`") || value.contains(";") {
            return true;
        }

        false
    }

    /// Generate compilation metadata
    fn generate_compilation_metadata(&self, plan: &ExecutionPlan, security_context: &SecurityContext) -> CompilationMetadata {
        let mut flags = Vec::new();

        flags.push(format!("plan_id:{}", plan.id));
        flags.push(format!("step_count:{}", plan.steps.len()));
        flags.push(format!("risk_level:{:?}", plan.risk_assessment.risk_level));
        flags.push(format!("isolation_level:{:?}", security_context.isolation_level));

        if self.config.strict_security_mode {
            flags.push("strict_security".to_string());
        }

        if self.config.require_explicit_approval {
            flags.push("explicit_approval_required".to_string());
        }

        CompilationMetadata {
            timestamp: chrono::Utc::now(),
            compiler_version: env!("CARGO_PKG_VERSION").to_string(),
            flags,
        }
    }

    /// Determine final approval requirement
    fn determine_final_approval_requirement(
        &self,
        plan: &ExecutionPlan,
        approval_requirements: &crate::compiler::ApprovalRequirements,
        security_context: &SecurityContext,
    ) -> bool {
        // Always require approval if configured
        if self.config.require_explicit_approval {
            return true;
        }

        // Require approval if policy engine determined it's needed
        if approval_requirements.requires_approval {
            return true;
        }

        // Require approval if plan's risk assessment says so
        if plan.risk_assessment.approval_required {
            return true;
        }

        // Require approval for full isolation
        if security_context.isolation_level == IsolationLevel::FullyIsolated {
            return true;
        }

        // Require approval for high-risk operations
        if plan.risk_assessment.risk_level >= RiskLevel::High {
            return true;
        }

        false
    }

    /// Get compiler configuration
    pub fn get_config(&self) -> &CompilerConfig {
        &self.config
    }

    /// Update compiler configuration
    pub fn update_config(&mut self, config: CompilerConfig) {
        info!("Updating compiler configuration");
        self.config = config;
    }

    /// Get security validator (for testing/debugging)
    pub async fn get_security_validator(&self) -> tokio::sync::RwLockReadGuard<SecurityValidator> {
        self.security_validator.read().await
    }

    /// Get policy engine (for testing/debugging)
    pub async fn get_policy_engine(&self) -> tokio::sync::RwLockReadGuard<PolicyEngine> {
        self.policy_engine.read().await
    }

    /// Get command validator (for testing/debugging)
    pub async fn get_command_validator(&self) -> tokio::sync::RwLockReadGuard<CommandValidator> {
        self.command_validator.read().await
    }
}

impl Default for CommandCompiler {
    fn default() -> Self {
        Self::new()
    }
}