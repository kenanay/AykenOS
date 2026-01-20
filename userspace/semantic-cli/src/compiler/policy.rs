//! Policy engine for approval requirements and governance
//!
//! This module implements the policy engine that determines when explicit approval
//! is required for command execution based on risk assessment and organizational policies.

use crate::types::*;
use crate::error::CompilationError;
use std::collections::HashMap;
use tracing::{debug, info};
use chrono::{DateTime, Utc, Datelike, Timelike};

/// Policy engine for determining approval requirements
pub struct PolicyEngine {
    /// Policy rules configuration
    policy_rules: PolicyRules,
    /// Approval history for learning
    approval_history: Vec<ApprovalRecord>,
    /// Policy evaluation cache
    evaluation_cache: HashMap<String, ApprovalRequirements>,
}

/// Policy rules configuration
#[derive(Debug, Clone)]
pub struct PolicyRules {
    /// Risk-based approval thresholds
    pub risk_thresholds: RiskThresholds,
    /// Action-based approval requirements
    pub action_requirements: HashMap<ActionType, ApprovalLevel>,
    /// Command-specific approval rules
    pub command_rules: HashMap<String, CommandApprovalRule>,
    /// Time-based restrictions
    pub time_restrictions: TimeRestrictions,
    /// User-based policies
    pub user_policies: UserPolicies,
}

/// Risk-based approval thresholds
#[derive(Debug, Clone)]
pub struct RiskThresholds {
    /// Minimum risk level requiring approval
    pub min_risk_for_approval: RiskLevel,
    /// Risk level requiring elevated approval
    pub elevated_approval_threshold: RiskLevel,
    /// Risk level requiring administrator approval
    pub admin_approval_threshold: RiskLevel,
}

/// Approval level requirements
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub enum ApprovalLevel {
    None,
    User,
    Supervisor,
    Administrator,
    Security,
}

/// Command-specific approval rule
#[derive(Debug, Clone)]
pub struct CommandApprovalRule {
    /// Required approval level
    pub approval_level: ApprovalLevel,
    /// Additional conditions
    pub conditions: Vec<ApprovalCondition>,
    /// Whether rule can be overridden
    pub overridable: bool,
}

/// Approval condition
#[derive(Debug, Clone)]
pub struct ApprovalCondition {
    /// Condition type
    pub condition_type: ConditionType,
    /// Condition description
    pub description: String,
    /// Required value
    pub required_value: serde_json::Value,
}

/// Time-based restrictions
#[derive(Debug, Clone)]
pub struct TimeRestrictions {
    /// Business hours (24-hour format)
    pub business_hours: (u32, u32), // (start_hour, end_hour)
    /// Days of week when restrictions apply (0 = Sunday)
    pub restricted_days: Vec<u32>,
    /// Whether to require approval outside business hours
    pub require_approval_after_hours: bool,
}

/// User-based policies
#[derive(Debug, Clone)]
pub struct UserPolicies {
    /// Default user approval level
    pub default_user_level: ApprovalLevel,
    /// User-specific approval levels
    pub user_levels: HashMap<String, ApprovalLevel>,
    /// Group-based policies
    pub group_policies: HashMap<String, ApprovalLevel>,
}

/// Approval requirements result
#[derive(Debug, Clone)]
pub struct ApprovalRequirements {
    /// Whether approval is required
    pub requires_approval: bool,
    /// Required approval level
    pub approval_level: ApprovalLevel,
    /// Reasons for requiring approval
    pub reasons: Vec<String>,
    /// Estimated approval time
    pub estimated_approval_time: std::time::Duration,
    /// Approval workflow steps
    pub workflow_steps: Vec<ApprovalStep>,
}

/// Approval workflow step
#[derive(Debug, Clone)]
pub struct ApprovalStep {
    /// Step name
    pub name: String,
    /// Required approver role
    pub approver_role: String,
    /// Step description
    pub description: String,
    /// Whether step can be skipped
    pub optional: bool,
}

/// Approval record for history tracking
#[derive(Debug, Clone)]
pub struct ApprovalRecord {
    /// Record ID
    pub id: uuid::Uuid,
    /// Plan ID that was approved
    pub plan_id: uuid::Uuid,
    /// Approval decision
    pub decision: ApprovalDecision,
    /// Approver information
    pub approver: String,
    /// Approval timestamp
    pub timestamp: DateTime<Utc>,
    /// Approval reason/comments
    pub comments: String,
    /// Risk level at time of approval
    pub risk_level: RiskLevel,
}

/// Approval decision
#[derive(Debug, Clone, PartialEq)]
pub enum ApprovalDecision {
    Approved,
    Rejected,
    ConditionallyApproved,
    Escalated,
}

impl Default for PolicyRules {
    fn default() -> Self {
        let mut action_requirements = HashMap::new();
        action_requirements.insert(ActionType::Query, ApprovalLevel::None);
        action_requirements.insert(ActionType::Analysis, ApprovalLevel::None);
        action_requirements.insert(ActionType::Monitoring, ApprovalLevel::User);
        action_requirements.insert(ActionType::Command, ApprovalLevel::User);
        action_requirements.insert(ActionType::Configuration, ApprovalLevel::Supervisor);
        action_requirements.insert(ActionType::FileOperation, ApprovalLevel::User);
        action_requirements.insert(ActionType::ProcessManagement, ApprovalLevel::Administrator);

        let mut command_rules = HashMap::new();
        
        // Safe commands - no approval needed
        command_rules.insert("echo".to_string(), CommandApprovalRule {
            approval_level: ApprovalLevel::None,
            conditions: vec![],
            overridable: false,
        });
        
        command_rules.insert("ls".to_string(), CommandApprovalRule {
            approval_level: ApprovalLevel::None,
            conditions: vec![],
            overridable: false,
        });

        // Potentially dangerous commands - require approval
        command_rules.insert("rm".to_string(), CommandApprovalRule {
            approval_level: ApprovalLevel::User,
            conditions: vec![
                ApprovalCondition {
                    condition_type: ConditionType::Custom,
                    description: "File deletion requires confirmation".to_string(),
                    required_value: serde_json::Value::Bool(true),
                }
            ],
            overridable: true,
        });

        command_rules.insert("kill".to_string(), CommandApprovalRule {
            approval_level: ApprovalLevel::Supervisor,
            conditions: vec![
                ApprovalCondition {
                    condition_type: ConditionType::ProcessRunning,
                    description: "Process must exist before termination".to_string(),
                    required_value: serde_json::Value::Bool(true),
                }
            ],
            overridable: false,
        });

        command_rules.insert("chmod".to_string(), CommandApprovalRule {
            approval_level: ApprovalLevel::User,
            conditions: vec![],
            overridable: true,
        });

        // System administration commands - require high-level approval
        command_rules.insert("sudo".to_string(), CommandApprovalRule {
            approval_level: ApprovalLevel::Administrator,
            conditions: vec![],
            overridable: false,
        });

        Self {
            risk_thresholds: RiskThresholds {
                min_risk_for_approval: RiskLevel::Medium,
                elevated_approval_threshold: RiskLevel::High,
                admin_approval_threshold: RiskLevel::Critical,
            },
            action_requirements,
            command_rules,
            time_restrictions: TimeRestrictions {
                business_hours: (9, 17), // 9 AM to 5 PM
                restricted_days: vec![0, 6], // Sunday and Saturday
                require_approval_after_hours: true,
            },
            user_policies: UserPolicies {
                default_user_level: ApprovalLevel::User,
                user_levels: HashMap::new(),
                group_policies: HashMap::new(),
            },
        }
    }
}

impl PolicyEngine {
    /// Create a new policy engine
    pub fn new() -> Self {
        Self {
            policy_rules: PolicyRules::default(),
            approval_history: Vec::new(),
            evaluation_cache: HashMap::new(),
        }
    }

    /// Create policy engine with custom rules
    pub fn with_rules(rules: PolicyRules) -> Self {
        Self {
            policy_rules: rules,
            approval_history: Vec::new(),
            evaluation_cache: HashMap::new(),
        }
    }

    /// Evaluate approval requirements for a plan
    pub async fn evaluate_approval_requirements(
        &mut self,
        plan: &ExecutionPlan,
        security_context: &SecurityContext,
    ) -> Result<ApprovalRequirements, CompilationError> {
        info!("Evaluating approval requirements for plan: {:?}", plan.id);

        // Check cache first
        let cache_key = self.generate_cache_key(plan, security_context);
        if let Some(cached_result) = self.evaluation_cache.get(&cache_key) {
            debug!("Using cached approval requirements");
            return Ok(cached_result.clone());
        }

        let mut requirements = ApprovalRequirements {
            requires_approval: false,
            approval_level: ApprovalLevel::None,
            reasons: Vec::new(),
            estimated_approval_time: std::time::Duration::from_secs(0),
            workflow_steps: Vec::new(),
        };

        // Step 1: Evaluate risk-based requirements
        self.evaluate_risk_based_approval(plan, &mut requirements).await?;

        // Step 2: Evaluate action-based requirements
        self.evaluate_action_based_approval(plan, &mut requirements).await?;

        // Step 3: Evaluate command-specific requirements
        self.evaluate_command_based_approval(plan, &mut requirements).await?;

        // Step 4: Evaluate time-based restrictions
        self.evaluate_time_based_approval(&mut requirements).await?;

        // Step 5: Evaluate security context requirements
        self.evaluate_security_context_approval(security_context, &mut requirements).await?;

        // Step 6: Generate workflow steps
        self.generate_approval_workflow(&mut requirements).await?;

        // Step 7: Estimate approval time
        self.estimate_approval_time(&mut requirements).await?;

        // Cache the result
        self.evaluation_cache.insert(cache_key, requirements.clone());

        info!("Approval evaluation completed: requires_approval={}, level={:?}", 
              requirements.requires_approval, requirements.approval_level);

        Ok(requirements)
    }

    /// Generate cache key for approval requirements
    fn generate_cache_key(&self, plan: &ExecutionPlan, security_context: &SecurityContext) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        plan.id.hash(&mut hasher);
        plan.risk_assessment.risk_level.hash(&mut hasher);
        security_context.isolation_level.hash(&mut hasher);
        
        for step in &plan.steps {
            step.command.hash(&mut hasher);
        }

        format!("{:x}", hasher.finish())
    }

    /// Evaluate risk-based approval requirements
    async fn evaluate_risk_based_approval(
        &self,
        plan: &ExecutionPlan,
        requirements: &mut ApprovalRequirements,
    ) -> Result<(), CompilationError> {
        let risk_level = plan.risk_assessment.risk_level;

        if risk_level >= self.policy_rules.risk_thresholds.min_risk_for_approval {
            requirements.requires_approval = true;
            requirements.reasons.push(format!("Risk level {:?} requires approval", risk_level));

            if risk_level >= self.policy_rules.risk_thresholds.admin_approval_threshold {
                requirements.approval_level = std::cmp::max(requirements.approval_level, ApprovalLevel::Administrator);
                requirements.reasons.push("Critical risk level requires administrator approval".to_string());
            } else if risk_level >= self.policy_rules.risk_thresholds.elevated_approval_threshold {
                requirements.approval_level = std::cmp::max(requirements.approval_level, ApprovalLevel::Supervisor);
                requirements.reasons.push("High risk level requires supervisor approval".to_string());
            } else {
                requirements.approval_level = std::cmp::max(requirements.approval_level, ApprovalLevel::User);
            }
        }

        Ok(())
    }

    /// Evaluate action-based approval requirements
    async fn evaluate_action_based_approval(
        &self,
        plan: &ExecutionPlan,
        requirements: &mut ApprovalRequirements,
    ) -> Result<(), CompilationError> {
        // Find the highest approval level required by any step's action type
        for step in &plan.steps {
            // Determine action type from command
            let action_type = self.infer_action_type(&step.command);
            
            if let Some(required_level) = self.policy_rules.action_requirements.get(&action_type) {
                if *required_level > ApprovalLevel::None {
                    requirements.requires_approval = true;
                    requirements.approval_level = std::cmp::max(requirements.approval_level, *required_level);
                    requirements.reasons.push(format!("Action type {:?} requires {:?} approval", action_type, required_level));
                }
            }
        }

        Ok(())
    }

    /// Infer action type from command
    fn infer_action_type(&self, command: &str) -> ActionType {
        let base_command = command.split_whitespace().next().unwrap_or("");
        
        match base_command {
            "echo" | "cat" | "ls" | "find" | "grep" | "ps" | "top" => ActionType::Query,
            "validate_query_parameters" | "execute_query" => ActionType::Query,
            "collect_analysis_data" | "perform_analysis" => ActionType::Analysis,
            "configure_monitoring" | "start_monitoring" => ActionType::Monitoring,
            "cp" | "mv" | "rm" | "mkdir" | "chmod" | "chown" => ActionType::FileOperation,
            "kill" | "killall" | "pkill" => ActionType::ProcessManagement,
            "apply_configuration" | "backup_current_configuration" => ActionType::Configuration,
            _ => ActionType::Command,
        }
    }

    /// Evaluate command-specific approval requirements
    async fn evaluate_command_based_approval(
        &self,
        plan: &ExecutionPlan,
        requirements: &mut ApprovalRequirements,
    ) -> Result<(), CompilationError> {
        for step in &plan.steps {
            let base_command = step.command.split_whitespace().next().unwrap_or("");
            
            if let Some(rule) = self.policy_rules.command_rules.get(base_command) {
                if rule.approval_level > ApprovalLevel::None {
                    requirements.requires_approval = true;
                    requirements.approval_level = std::cmp::max(requirements.approval_level, rule.approval_level.clone());
                    requirements.reasons.push(format!("Command '{}' requires {:?} approval", base_command, rule.approval_level));

                    // Check additional conditions
                    for condition in &rule.conditions {
                        requirements.reasons.push(format!("Condition: {}", condition.description));
                    }
                }
            }
        }

        Ok(())
    }

    /// Evaluate time-based approval requirements
    async fn evaluate_time_based_approval(
        &self,
        requirements: &mut ApprovalRequirements,
    ) -> Result<(), CompilationError> {
        if !self.policy_rules.time_restrictions.require_approval_after_hours {
            return Ok(());
        }

        let now = Utc::now();
        let hour = now.hour();
        let weekday = now.weekday().num_days_from_sunday();

        let (start_hour, end_hour) = self.policy_rules.time_restrictions.business_hours;
        let is_business_hours = hour >= start_hour && hour < end_hour;
        let is_restricted_day = self.policy_rules.time_restrictions.restricted_days.contains(&weekday);

        if !is_business_hours || is_restricted_day {
            requirements.requires_approval = true;
            requirements.approval_level = std::cmp::max(requirements.approval_level, ApprovalLevel::Supervisor);
            
            if !is_business_hours {
                requirements.reasons.push("Execution outside business hours requires approval".to_string());
            }
            
            if is_restricted_day {
                requirements.reasons.push("Execution on restricted day requires approval".to_string());
            }
        }

        Ok(())
    }

    /// Evaluate security context approval requirements
    async fn evaluate_security_context_approval(
        &self,
        security_context: &SecurityContext,
        requirements: &mut ApprovalRequirements,
    ) -> Result<(), CompilationError> {
        // High isolation levels may require approval
        match security_context.isolation_level {
            IsolationLevel::FullyIsolated => {
                requirements.requires_approval = true;
                requirements.approval_level = std::cmp::max(requirements.approval_level, ApprovalLevel::Supervisor);
                requirements.reasons.push("Full isolation mode requires supervisor approval".to_string());
            },
            IsolationLevel::Sandboxed => {
                // Sandboxed mode may require approval for certain permissions
                let has_dangerous_permissions = security_context.permissions.iter().any(|p| {
                    matches!(p.access_type, AccessType::Delete | AccessType::Execute) ||
                    p.resource.contains("system") || p.resource.contains("process")
                });

                if has_dangerous_permissions {
                    requirements.requires_approval = true;
                    requirements.approval_level = std::cmp::max(requirements.approval_level, ApprovalLevel::User);
                    requirements.reasons.push("Sandboxed execution with dangerous permissions requires approval".to_string());
                }
            },
            IsolationLevel::None => {
                // No isolation may require approval for safety
                if !security_context.permissions.is_empty() {
                    requirements.requires_approval = true;
                    requirements.approval_level = std::cmp::max(requirements.approval_level, ApprovalLevel::User);
                    requirements.reasons.push("Execution without isolation requires approval".to_string());
                }
            },
        }

        Ok(())
    }

    /// Generate approval workflow steps
    async fn generate_approval_workflow(
        &self,
        requirements: &mut ApprovalRequirements,
    ) -> Result<(), CompilationError> {
        if !requirements.requires_approval {
            return Ok(());
        }

        match requirements.approval_level {
            ApprovalLevel::None => {},
            ApprovalLevel::User => {
                requirements.workflow_steps.push(ApprovalStep {
                    name: "User Approval".to_string(),
                    approver_role: "user".to_string(),
                    description: "User must approve the execution plan".to_string(),
                    optional: false,
                });
            },
            ApprovalLevel::Supervisor => {
                requirements.workflow_steps.push(ApprovalStep {
                    name: "User Approval".to_string(),
                    approver_role: "user".to_string(),
                    description: "User must approve the execution plan".to_string(),
                    optional: false,
                });
                requirements.workflow_steps.push(ApprovalStep {
                    name: "Supervisor Approval".to_string(),
                    approver_role: "supervisor".to_string(),
                    description: "Supervisor must approve high-risk operations".to_string(),
                    optional: false,
                });
            },
            ApprovalLevel::Administrator => {
                requirements.workflow_steps.push(ApprovalStep {
                    name: "User Approval".to_string(),
                    approver_role: "user".to_string(),
                    description: "User must approve the execution plan".to_string(),
                    optional: false,
                });
                requirements.workflow_steps.push(ApprovalStep {
                    name: "Supervisor Approval".to_string(),
                    approver_role: "supervisor".to_string(),
                    description: "Supervisor must approve high-risk operations".to_string(),
                    optional: false,
                });
                requirements.workflow_steps.push(ApprovalStep {
                    name: "Administrator Approval".to_string(),
                    approver_role: "administrator".to_string(),
                    description: "Administrator must approve critical operations".to_string(),
                    optional: false,
                });
            },
            ApprovalLevel::Security => {
                requirements.workflow_steps.push(ApprovalStep {
                    name: "Security Review".to_string(),
                    approver_role: "security_officer".to_string(),
                    description: "Security officer must review and approve".to_string(),
                    optional: false,
                });
            },
        }

        Ok(())
    }

    /// Estimate approval time based on workflow complexity
    async fn estimate_approval_time(
        &self,
        requirements: &mut ApprovalRequirements,
    ) -> Result<(), CompilationError> {
        if !requirements.requires_approval {
            return Ok(());
        }

        let base_time_per_step = std::time::Duration::from_secs(300); // 5 minutes per step
        let total_steps = requirements.workflow_steps.len();
        
        // Add complexity factor based on approval level
        let complexity_multiplier = match requirements.approval_level {
            ApprovalLevel::None => 1.0,
            ApprovalLevel::User => 1.0,
            ApprovalLevel::Supervisor => 2.0,
            ApprovalLevel::Administrator => 3.0,
            ApprovalLevel::Security => 4.0,
        };

        let estimated_time = base_time_per_step.mul_f32(total_steps as f32 * complexity_multiplier);
        requirements.estimated_approval_time = estimated_time;

        Ok(())
    }

    /// Record an approval decision
    pub fn record_approval(&mut self, record: ApprovalRecord) {
        info!("Recording approval decision: {:?} for plan: {:?}", record.decision, record.plan_id);
        self.approval_history.push(record);
    }

    /// Get approval history
    pub fn get_approval_history(&self) -> &[ApprovalRecord] {
        &self.approval_history
    }

    /// Update policy rules
    pub fn update_rules(&mut self, rules: PolicyRules) {
        info!("Updating policy rules");
        self.policy_rules = rules;
        self.evaluation_cache.clear(); // Clear cache when rules change
    }

    /// Get current policy rules
    pub fn get_rules(&self) -> &PolicyRules {
        &self.policy_rules
    }

    /// Clear evaluation cache
    pub fn clear_cache(&mut self) {
        self.evaluation_cache.clear();
    }
}

impl Default for PolicyEngine {
    fn default() -> Self {
        Self::new()
    }
}