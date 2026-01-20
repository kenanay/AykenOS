//! Security validation and context management
//!
//! This module implements comprehensive security validation for execution plans,
//! ensuring that all operations comply with security policies and isolation requirements.

use crate::types::*;
use crate::error::CompilationError;
use std::collections::HashMap;
use tracing::{debug, warn, info};
use chrono::{DateTime, Utc};

/// Security validator for execution plans and commands
pub struct SecurityValidator {
    /// Security policies
    security_policies: SecurityPolicies,
    /// Trust levels for different operation types
    trust_levels: HashMap<ActionType, TrustLevel>,
    /// Security event log
    security_events: Vec<SecurityEvent>,
}

/// Security policies configuration
#[derive(Debug, Clone)]
pub struct SecurityPolicies {
    /// Default isolation level for new operations
    pub default_isolation_level: IsolationLevel,
    /// Whether to require approval for all operations
    pub require_approval_for_all: bool,
    /// Maximum risk level allowed without approval
    pub max_risk_without_approval: RiskLevel,
    /// Whether to enable strict permission checking
    pub strict_permission_checking: bool,
    /// Allowed file system paths
    pub allowed_filesystem_paths: Vec<String>,
    /// Blocked command patterns
    pub blocked_command_patterns: Vec<String>,
}

/// Trust level for operations
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum TrustLevel {
    Untrusted,
    Limited,
    Trusted,
    FullyTrusted,
}

/// Security event for audit logging
#[derive(Debug, Clone)]
pub struct SecurityEvent {
    /// Event ID
    pub id: uuid::Uuid,
    /// Event type
    pub event_type: SecurityEventType,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Description
    pub description: String,
    /// Associated plan ID if applicable
    pub plan_id: Option<uuid::Uuid>,
    /// Risk level
    pub risk_level: RiskLevel,
    /// Action taken
    pub action_taken: SecurityAction,
}

/// Types of security events
#[derive(Debug, Clone)]
pub enum SecurityEventType {
    PlanValidation,
    PermissionCheck,
    IsolationViolation,
    RiskAssessment,
    ApprovalRequired,
    SecurityViolation,
}

/// Security actions taken
#[derive(Debug, Clone)]
pub enum SecurityAction {
    Allowed,
    Blocked,
    RequiresApproval,
    Escalated,
}

impl Default for SecurityPolicies {
    fn default() -> Self {
        Self {
            default_isolation_level: IsolationLevel::Sandboxed,
            require_approval_for_all: false,
            max_risk_without_approval: RiskLevel::Low,
            strict_permission_checking: true,
            allowed_filesystem_paths: vec![
                "/tmp".to_string(),
                "/var/tmp".to_string(),
                "/home".to_string(),
                "/opt/ayken".to_string(),
            ],
            blocked_command_patterns: vec![
                "rm -rf /".to_string(),
                "dd if=/dev/zero".to_string(),
                ":(){ :|:& };:".to_string(), // Fork bomb
                "chmod 777 /".to_string(),
                "chown root:root /".to_string(),
                "mkfs".to_string(),
                "fdisk".to_string(),
            ],
        }
    }
}

impl SecurityValidator {
    /// Create a new security validator
    pub fn new() -> Self {
        let mut trust_levels = HashMap::new();
        trust_levels.insert(ActionType::Query, TrustLevel::Trusted);
        trust_levels.insert(ActionType::Analysis, TrustLevel::Trusted);
        trust_levels.insert(ActionType::Monitoring, TrustLevel::Limited);
        trust_levels.insert(ActionType::Command, TrustLevel::Limited);
        trust_levels.insert(ActionType::Configuration, TrustLevel::Untrusted);
        trust_levels.insert(ActionType::FileOperation, TrustLevel::Limited);
        trust_levels.insert(ActionType::ProcessManagement, TrustLevel::Untrusted);

        Self {
            security_policies: SecurityPolicies::default(),
            trust_levels,
            security_events: Vec::new(),
        }
    }

    /// Create validator with custom policies
    pub fn with_policies(policies: SecurityPolicies) -> Self {
        let mut validator = Self::new();
        validator.security_policies = policies;
        validator
    }

    /// Validate plan security and generate security context
    pub async fn validate_plan_security(&mut self, plan: &ExecutionPlan) -> Result<SecurityContext, CompilationError> {
        info!("Validating security for plan: {:?}", plan.id);

        // Step 1: Assess overall plan risk
        let plan_risk = self.assess_plan_risk(plan).await?;

        // Step 2: Determine required isolation level
        let isolation_level = self.determine_isolation_level(plan, &plan_risk).await?;

        // Step 3: Generate required permissions
        let permissions = self.generate_required_permissions(plan).await?;

        // Step 4: Set resource limits based on risk and isolation
        let resource_limits = self.generate_resource_limits(plan, &plan_risk, &isolation_level).await?;

        // Step 5: Validate against security policies
        self.validate_against_policies(plan, &isolation_level, &permissions).await?;

        // Step 6: Log security event
        self.log_security_event(SecurityEvent {
            id: uuid::Uuid::new_v4(),
            event_type: SecurityEventType::PlanValidation,
            timestamp: Utc::now(),
            description: format!("Plan {} validated with isolation level {:?}", plan.id, isolation_level),
            plan_id: Some(plan.id),
            risk_level: plan_risk,
            action_taken: SecurityAction::Allowed,
        });

        let security_context = SecurityContext {
            permissions,
            isolation_level,
            resource_limits,
        };

        info!("Security validation completed for plan: {:?}", plan.id);
        Ok(security_context)
    }

    /// Assess the overall risk level of a plan
    async fn assess_plan_risk(&mut self, plan: &ExecutionPlan) -> Result<RiskLevel, CompilationError> {
        debug!("Assessing risk for plan: {:?}", plan.id);

        let mut max_risk = RiskLevel::Low;

        // Start with the plan's own risk assessment
        max_risk = std::cmp::max(max_risk, plan.risk_assessment.risk_level);

        // Assess risk based on action type
        for step in &plan.steps {
            let step_risk = self.assess_step_risk(step).await?;
            max_risk = std::cmp::max(max_risk, step_risk);
        }

        // Increase risk based on plan complexity
        if plan.steps.len() > 10 {
            max_risk = match max_risk {
                RiskLevel::Low => RiskLevel::Medium,
                RiskLevel::Medium => RiskLevel::High,
                RiskLevel::High => RiskLevel::Critical,
                RiskLevel::Critical => RiskLevel::Critical,
            };
        }

        // Check for dangerous command patterns
        for step in &plan.steps {
            if self.contains_dangerous_patterns(&step.command) {
                max_risk = RiskLevel::Critical;
                
                self.log_security_event(SecurityEvent {
                    id: uuid::Uuid::new_v4(),
                    event_type: SecurityEventType::SecurityViolation,
                    timestamp: Utc::now(),
                    description: format!("Dangerous command pattern detected in step: {}", step.command),
                    plan_id: Some(plan.id),
                    risk_level: RiskLevel::Critical,
                    action_taken: SecurityAction::Blocked,
                });

                return Err(CompilationError::SecurityViolation {
                    reason: format!("Dangerous command pattern detected: {}", step.command),
                    plan_id: plan.id,
                });
            }
        }

        debug!("Plan risk assessment: {:?}", max_risk);
        Ok(max_risk)
    }

    /// Assess risk for an individual step
    async fn assess_step_risk(&self, step: &PlanStep) -> Result<RiskLevel, CompilationError> {
        let command = step.command.split_whitespace().next().unwrap_or("");

        let risk = match command {
            // Safe commands
            "echo" | "cat" | "ls" | "pwd" | "whoami" | "date" => RiskLevel::Low,
            
            // Query and analysis commands
            "validate_query_parameters" | "execute_query" | "format_query_results" => RiskLevel::Low,
            "collect_analysis_data" | "perform_analysis" => RiskLevel::Low,
            
            // Monitoring commands
            "ps" | "top" | "df" | "free" | "ping" => RiskLevel::Low,
            
            // File operations (potentially risky)
            "cp" | "mv" | "mkdir" | "touch" => RiskLevel::Medium,
            "rm" | "rmdir" => RiskLevel::High,
            
            // System modification commands (high risk)
            "chmod" | "chown" | "mount" | "umount" => RiskLevel::High,
            
            // Process management (high risk)
            "kill" | "killall" | "pkill" => RiskLevel::High,
            
            // System administration (critical risk)
            "sudo" | "su" | "passwd" | "useradd" | "userdel" => RiskLevel::Critical,
            
            // Network commands (medium risk)
            "wget" | "curl" | "ssh" | "scp" => RiskLevel::Medium,
            
            // Compilation and execution (high risk)
            "gcc" | "make" | "execute_command" => RiskLevel::High,
            
            // Configuration changes (high risk)
            "apply_configuration" | "backup_current_configuration" => RiskLevel::High,
            
            // Unknown commands (medium risk by default)
            _ => RiskLevel::Medium,
        };

        Ok(risk)
    }

    /// Check if command contains dangerous patterns
    fn contains_dangerous_patterns(&self, command: &str) -> bool {
        for pattern in &self.security_policies.blocked_command_patterns {
            if command.contains(pattern) {
                warn!("Dangerous pattern detected: {} in command: {}", pattern, command);
                return true;
            }
        }
        false
    }

    /// Determine required isolation level based on plan and risk
    async fn determine_isolation_level(&self, plan: &ExecutionPlan, risk: &RiskLevel) -> Result<IsolationLevel, CompilationError> {
        debug!("Determining isolation level for risk: {:?}", risk);

        // Base isolation on risk level
        let mut isolation = match risk {
            RiskLevel::Low => IsolationLevel::None,
            RiskLevel::Medium => IsolationLevel::Sandboxed,
            RiskLevel::High => IsolationLevel::FullyIsolated,
            RiskLevel::Critical => IsolationLevel::FullyIsolated,
        };

        // Increase isolation based on action types
        for step in &plan.steps {
            let command = step.command.split_whitespace().next().unwrap_or("");
            
            match command {
                "kill" | "killall" | "pkill" | "sudo" | "su" => {
                    isolation = IsolationLevel::FullyIsolated;
                },
                "rm" | "chmod" | "chown" | "mount" | "umount" => {
                    isolation = std::cmp::max(isolation, IsolationLevel::Sandboxed);
                },
                _ => {}
            }
        }

        // Apply policy defaults
        isolation = std::cmp::max(isolation, self.security_policies.default_isolation_level);

        debug!("Determined isolation level: {:?}", isolation);
        Ok(isolation)
    }

    /// Generate required permissions for the plan
    async fn generate_required_permissions(&self, plan: &ExecutionPlan) -> Result<Vec<Permission>, CompilationError> {
        debug!("Generating permissions for plan: {:?}", plan.id);

        let mut permissions = Vec::new();
        let mut seen_permissions = std::collections::HashSet::new();

        for step in &plan.steps {
            let step_permissions = self.get_step_permissions(step).await?;
            
            for permission in step_permissions {
                let key = format!("{}:{:?}", permission.resource, permission.access_type);
                if !seen_permissions.contains(&key) {
                    seen_permissions.insert(key);
                    permissions.push(permission);
                }
            }
        }

        debug!("Generated {} permissions", permissions.len());
        Ok(permissions)
    }

    /// Get permissions required for a specific step
    async fn get_step_permissions(&self, step: &PlanStep) -> Result<Vec<Permission>, CompilationError> {
        let command = step.command.split_whitespace().next().unwrap_or("");
        
        let permissions = match command {
            // File system operations
            "ls" | "cat" | "find" | "grep" => vec![
                Permission {
                    resource: "filesystem".to_string(),
                    access_type: AccessType::Read,
                }
            ],
            
            "cp" | "mv" | "mkdir" | "touch" => vec![
                Permission {
                    resource: "filesystem".to_string(),
                    access_type: AccessType::Read,
                },
                Permission {
                    resource: "filesystem".to_string(),
                    access_type: AccessType::Write,
                },
                Permission {
                    resource: "filesystem".to_string(),
                    access_type: AccessType::Create,
                }
            ],
            
            "rm" | "rmdir" => vec![
                Permission {
                    resource: "filesystem".to_string(),
                    access_type: AccessType::Delete,
                }
            ],
            
            "chmod" | "chown" => vec![
                Permission {
                    resource: "filesystem".to_string(),
                    access_type: AccessType::Write,
                }
            ],
            
            // Process management
            "ps" | "top" => vec![
                Permission {
                    resource: "process_info".to_string(),
                    access_type: AccessType::Read,
                }
            ],
            
            "kill" | "killall" | "pkill" => vec![
                Permission {
                    resource: "process_management".to_string(),
                    access_type: AccessType::Execute,
                }
            ],
            
            // Network operations
            "ping" | "wget" | "curl" => vec![
                Permission {
                    resource: "network".to_string(),
                    access_type: AccessType::Read,
                }
            ],
            
            // System information
            "df" | "free" | "uname" => vec![
                Permission {
                    resource: "system_info".to_string(),
                    access_type: AccessType::Read,
                }
            ],
            
            // Semantic CLI operations
            "validate_query_parameters" | "execute_query" => vec![
                Permission {
                    resource: "query_engine".to_string(),
                    access_type: AccessType::Execute,
                }
            ],
            
            "validate_command_security" => vec![
                Permission {
                    resource: "security_validator".to_string(),
                    access_type: AccessType::Execute,
                }
            ],
            
            "execute_command" => vec![
                Permission {
                    resource: "command_execution".to_string(),
                    access_type: AccessType::Execute,
                }
            ],
            
            "apply_configuration" => vec![
                Permission {
                    resource: "system_config".to_string(),
                    access_type: AccessType::Write,
                }
            ],
            
            // Default: no specific permissions
            _ => vec![]
        };

        Ok(permissions)
    }

    /// Generate resource limits based on risk and isolation
    async fn generate_resource_limits(
        &self,
        plan: &ExecutionPlan,
        risk: &RiskLevel,
        isolation: &IsolationLevel,
    ) -> Result<ResourceLimits, CompilationError> {
        debug!("Generating resource limits for risk: {:?}, isolation: {:?}", risk, isolation);

        // Base limits on risk level
        let (base_cpu, base_memory, base_time) = match risk {
            RiskLevel::Low => (0.3, 512u64 * 1024 * 1024, 300), // 30% CPU, 512MB, 5min
            RiskLevel::Medium => (0.5, 1024u64 * 1024 * 1024, 600), // 50% CPU, 1GB, 10min
            RiskLevel::High => (0.7, 2048u64 * 1024 * 1024, 1200), // 70% CPU, 2GB, 20min
            RiskLevel::Critical => (0.8, 4096u64 * 1024 * 1024, 1800), // 80% CPU, 4GB, 30min
        };

        // Adjust based on isolation level
        let (cpu_multiplier, memory_multiplier, time_multiplier) = match isolation {
            IsolationLevel::None => (1.0, 1.0, 1.0),
            IsolationLevel::Sandboxed => (0.8, 0.8, 1.2), // Reduce resources, increase time
            IsolationLevel::FullyIsolated => (0.6, 0.6, 1.5), // Further reduce resources
        };

        // Consider plan complexity
        let complexity_factor = if plan.steps.len() > 5 {
            1.0 + (plan.steps.len() as f32 - 5.0) * 0.1
        } else {
            1.0
        };

        let final_cpu: f32 = if base_cpu * cpu_multiplier > 1.0 { 1.0 } else { base_cpu * cpu_multiplier };
        
        // Use checked arithmetic to prevent overflow
        let calculated_memory = (base_memory as f64 * memory_multiplier as f64 * complexity_factor as f64) as u64;
        let max_memory = calculated_memory.min(8 * 1024 * 1024 * 1024); // Max 8GB
        
        let calculated_time = (base_time as f64 * time_multiplier as f64 * complexity_factor as f64) as u64;
        let max_time_secs = calculated_time.min(3600); // Max 1 hour
        
        let limits = ResourceLimits {
            max_cpu: final_cpu,
            max_memory,
            max_time: std::time::Duration::from_secs(max_time_secs),
        };

        debug!("Generated resource limits: CPU: {:.2}, Memory: {}MB, Time: {}s", 
               limits.max_cpu, limits.max_memory / (1024 * 1024), limits.max_time.as_secs());

        Ok(limits)
    }

    /// Validate plan against security policies
    async fn validate_against_policies(
        &mut self,
        plan: &ExecutionPlan,
        isolation: &IsolationLevel,
        permissions: &[Permission],
    ) -> Result<(), CompilationError> {
        debug!("Validating plan against security policies");

        // Check if approval is required by policy
        if self.security_policies.require_approval_for_all {
            self.log_security_event(SecurityEvent {
                id: uuid::Uuid::new_v4(),
                event_type: SecurityEventType::ApprovalRequired,
                timestamp: Utc::now(),
                description: "Approval required by security policy".to_string(),
                plan_id: Some(plan.id),
                risk_level: plan.risk_assessment.risk_level,
                action_taken: SecurityAction::RequiresApproval,
            });
        }

        // Validate file system access
        for step in &plan.steps {
            self.validate_filesystem_access(step, permissions).await?;
        }

        // Validate isolation requirements
        if *isolation == IsolationLevel::None && plan.risk_assessment.risk_level > RiskLevel::Low {
            return Err(CompilationError::SecurityViolation {
                reason: "Insufficient isolation for risk level".to_string(),
                plan_id: plan.id,
            });
        }

        // Check permission requirements
        if self.security_policies.strict_permission_checking {
            self.validate_strict_permissions(plan, permissions).await?;
        }

        debug!("Plan passed all security policy validations");
        Ok(())
    }

    /// Validate file system access against allowed paths
    async fn validate_filesystem_access(&mut self, step: &PlanStep, permissions: &[Permission]) -> Result<(), CompilationError> {
        // Check if step requires filesystem access
        let has_fs_permission = permissions.iter().any(|p| p.resource == "filesystem");
        
        if !has_fs_permission {
            return Ok(()); // No filesystem access required
        }

        // Extract potential file paths from step parameters
        for (key, value) in &step.parameters {
            if key.contains("path") || key.contains("file") || key.contains("directory") {
                if let Some(path_str) = value.as_str() {
                    if !self.is_path_allowed(path_str) {
                        self.log_security_event(SecurityEvent {
                            id: uuid::Uuid::new_v4(),
                            event_type: SecurityEventType::SecurityViolation,
                            timestamp: Utc::now(),
                            description: format!("Unauthorized file system access: {}", path_str),
                            plan_id: None,
                            risk_level: RiskLevel::High,
                            action_taken: SecurityAction::Blocked,
                        });

                        return Err(CompilationError::SecurityViolation {
                            reason: format!("Unauthorized file system access: {}", path_str),
                            plan_id: step.id,
                        });
                    }
                }
            }
        }

        Ok(())
    }

    /// Check if a file path is allowed by security policies
    fn is_path_allowed(&self, path: &str) -> bool {
        // Always block access to critical system paths
        let blocked_paths = ["/etc/passwd", "/etc/shadow", "/boot", "/sys", "/proc/sys"];
        
        for blocked in &blocked_paths {
            if path.starts_with(blocked) {
                return false;
            }
        }

        // Check against allowed paths
        for allowed in &self.security_policies.allowed_filesystem_paths {
            if path.starts_with(allowed) {
                return true;
            }
        }

        false
    }

    /// Validate strict permission requirements
    async fn validate_strict_permissions(&self, plan: &ExecutionPlan, permissions: &[Permission]) -> Result<(), CompilationError> {
        // In strict mode, ensure all required permissions are explicitly granted
        for step in &plan.steps {
            let required_permissions = self.get_step_permissions(step).await?;
            
            for required in &required_permissions {
                let has_permission = permissions.iter().any(|p| {
                    p.resource == required.resource && p.access_type == required.access_type
                });

                if !has_permission {
                    return Err(CompilationError::InsufficientPermissions {
                        required_permission: required.clone(),
                        available_permissions: permissions.to_vec(),
                    });
                }
            }
        }

        Ok(())
    }

    /// Log a security event
    fn log_security_event(&mut self, event: SecurityEvent) {
        info!("Security event: {:?} - {}", event.event_type, event.description);
        self.security_events.push(event);
    }

    /// Get security events
    pub fn get_security_events(&self) -> &[SecurityEvent] {
        &self.security_events
    }

    /// Clear security events (for testing or maintenance)
    pub fn clear_security_events(&mut self) {
        self.security_events.clear();
    }

    /// Update security policies
    pub fn update_policies(&mut self, policies: SecurityPolicies) {
        info!("Updating security policies");
        self.security_policies = policies;
    }

    /// Get current security policies
    pub fn get_policies(&self) -> &SecurityPolicies {
        &self.security_policies
    }
}

impl Default for SecurityValidator {
    fn default() -> Self {
        Self::new()
    }
}