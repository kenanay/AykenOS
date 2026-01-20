//! Risk assessment for execution plans
//!
//! This module provides functionality for assessing the risk level of
//! execution plans and generating appropriate mitigation strategies.

use crate::types::*;
use crate::error::PlanningError;
use std::collections::HashMap;
use tracing::debug;

/// Risk assessor for execution plans
pub struct RiskAssessor {
    /// Configuration for risk assessment
    config: RiskAssessmentConfig,
    /// Risk rules database
    risk_rules: HashMap<String, RiskRule>,
    /// Security policies
    security_policies: SecurityPolicies,
}

/// Configuration for risk assessment
#[derive(Debug, Clone)]
pub struct RiskAssessmentConfig {
    /// Default risk level for unknown operations
    pub default_risk_level: RiskLevel,
    /// Whether to require approval for medium risk operations
    pub require_approval_medium_risk: bool,
    /// Whether to require approval for high risk operations
    pub require_approval_high_risk: bool,
    /// Maximum allowed risk level without explicit approval
    pub max_auto_approve_risk: RiskLevel,
    /// Risk escalation thresholds
    pub escalation_thresholds: RiskEscalationThresholds,
}

impl Default for RiskAssessmentConfig {
    fn default() -> Self {
        Self {
            default_risk_level: RiskLevel::Low,
            require_approval_medium_risk: true,
            require_approval_high_risk: true,
            max_auto_approve_risk: RiskLevel::Low,
            escalation_thresholds: RiskEscalationThresholds::default(),
        }
    }
}

/// Risk escalation thresholds
#[derive(Debug, Clone)]
pub struct RiskEscalationThresholds {
    /// Number of steps that escalates risk
    pub step_count_threshold: usize,
    /// Execution time that escalates risk (in seconds)
    pub execution_time_threshold: u64,
    /// Memory usage that escalates risk (in bytes)
    pub memory_usage_threshold: u64,
    /// Number of exclusive resources that escalates risk
    pub exclusive_resource_threshold: usize,
}

impl Default for RiskEscalationThresholds {
    fn default() -> Self {
        Self {
            step_count_threshold: 10,
            execution_time_threshold: 300, // 5 minutes
            memory_usage_threshold: 1024 * 1024 * 1024, // 1GB
            exclusive_resource_threshold: 3,
        }
    }
}

/// Risk rule for specific operations
#[derive(Debug, Clone)]
pub struct RiskRule {
    /// Command pattern this rule applies to
    pub command_pattern: String,
    /// Base risk level for this operation
    pub base_risk_level: RiskLevel,
    /// Potential impacts
    pub potential_impacts: Vec<Impact>,
    /// Required mitigation strategies
    pub required_mitigations: Vec<Mitigation>,
    /// Whether approval is always required
    pub always_require_approval: bool,
    /// Risk factors that can escalate the risk
    pub escalation_factors: Vec<RiskEscalationFactor>,
}

/// Risk escalation factor
#[derive(Debug, Clone)]
pub struct RiskEscalationFactor {
    /// Description of the factor
    pub description: String,
    /// Condition that triggers escalation
    pub condition: EscalationCondition,
    /// Risk level increase
    pub risk_increase: RiskLevelIncrease,
}

/// Escalation condition
#[derive(Debug, Clone)]
pub enum EscalationCondition {
    /// Parameter contains specific value
    ParameterContains { key: String, value: String },
    /// Target type matches
    TargetTypeMatches(TargetType),
    /// Step count exceeds threshold
    StepCountExceeds(usize),
    /// Execution time exceeds threshold
    ExecutionTimeExceeds(std::time::Duration),
    /// Multiple exclusive resources
    MultipleExclusiveResources,
}

/// Risk level increase
#[derive(Debug, Clone)]
pub enum RiskLevelIncrease {
    /// Increase by one level
    OneLevel,
    /// Increase by two levels
    TwoLevels,
    /// Set to specific level
    SetTo(RiskLevel),
}

/// Security policies for risk assessment
#[derive(Debug, Clone)]
pub struct SecurityPolicies {
    /// Forbidden command patterns
    pub forbidden_commands: Vec<String>,
    /// Restricted file paths
    pub restricted_paths: Vec<String>,
    /// Allowed process operations
    pub allowed_process_operations: Vec<String>,
    /// Configuration change policies
    pub config_change_policies: Vec<ConfigChangePolicy>,
}

impl Default for SecurityPolicies {
    fn default() -> Self {
        Self {
            forbidden_commands: vec![
                "rm -rf /".to_string(),
                "format".to_string(),
                "delete_all".to_string(),
            ],
            restricted_paths: vec![
                "/etc/passwd".to_string(),
                "/etc/shadow".to_string(),
                "/boot/".to_string(),
                "/sys/".to_string(),
            ],
            allowed_process_operations: vec![
                "start".to_string(),
                "stop".to_string(),
                "restart".to_string(),
                "status".to_string(),
            ],
            config_change_policies: vec![
                ConfigChangePolicy {
                    config_type: "system".to_string(),
                    requires_backup: true,
                    requires_approval: true,
                    max_changes_per_session: 5,
                },
            ],
        }
    }
}

/// Configuration change policy
#[derive(Debug, Clone)]
pub struct ConfigChangePolicy {
    /// Type of configuration
    pub config_type: String,
    /// Whether backup is required
    pub requires_backup: bool,
    /// Whether approval is required
    pub requires_approval: bool,
    /// Maximum changes allowed per session
    pub max_changes_per_session: u32,
}

impl RiskAssessor {
    /// Create a new risk assessor
    pub fn new() -> Self {
        let mut assessor = Self {
            config: RiskAssessmentConfig::default(),
            risk_rules: HashMap::new(),
            security_policies: SecurityPolicies::default(),
        };
        
        assessor.initialize_default_risk_rules();
        assessor
    }

    /// Create assessor with custom configuration
    pub fn with_config(config: RiskAssessmentConfig) -> Self {
        let mut assessor = Self::new();
        assessor.config = config;
        assessor
    }

    /// Assess risk for an execution plan
    pub async fn assess_plan_risk(
        &self,
        intent: &Intent,
        steps: &[PlanStep],
    ) -> Result<RiskAssessment, PlanningError> {
        debug!("Assessing risk for plan with {} steps", steps.len());

        // Start with base risk assessment
        let mut risk_level = self.assess_base_risk_level(intent, steps)?;
        let mut potential_impacts = Vec::new();
        let mut mitigation_strategies = Vec::new();
        let mut approval_required = false;

        // Assess each step individually
        for step in steps {
            let step_assessment = self.assess_step_risk(step)?;
            
            // Escalate risk level if needed
            if step_assessment.risk_level > risk_level {
                risk_level = step_assessment.risk_level;
            }
            
            potential_impacts.extend(step_assessment.potential_impacts);
            mitigation_strategies.extend(step_assessment.mitigation_strategies);
            
            if step_assessment.approval_required {
                approval_required = true;
            }
        }

        // Apply escalation factors
        risk_level = self.apply_escalation_factors(risk_level, intent, steps)?;

        // Check security policies
        self.check_security_policies(intent, steps)?;

        // Determine if approval is required
        approval_required = approval_required || self.requires_approval(risk_level);

        // Remove duplicate impacts and mitigations
        potential_impacts.sort_by(|a, b| a.description.cmp(&b.description));
        potential_impacts.dedup_by(|a, b| a.description == b.description);
        
        mitigation_strategies.sort_by(|a, b| a.description.cmp(&b.description));
        mitigation_strategies.dedup_by(|a, b| a.description == b.description);

        debug!("Risk assessment complete: {:?}, approval required: {}", risk_level, approval_required);

        Ok(RiskAssessment {
            risk_level,
            potential_impacts,
            mitigation_strategies,
            approval_required,
        })
    }

    /// Assess base risk level for intent and steps
    fn assess_base_risk_level(&self, intent: &Intent, steps: &[PlanStep]) -> Result<RiskLevel, PlanningError> {
        let mut base_risk = match intent.action {
            ActionType::Query => RiskLevel::Low,
            ActionType::Command => RiskLevel::Medium,
            ActionType::Configuration => RiskLevel::High,
            ActionType::Analysis => RiskLevel::Low,
            ActionType::Monitoring => RiskLevel::Low,
            ActionType::FileOperation => RiskLevel::Medium,
            ActionType::ProcessManagement => RiskLevel::High,
        };

        // Escalate based on step count
        if steps.len() > self.config.escalation_thresholds.step_count_threshold {
            base_risk = self.escalate_risk_level(base_risk, RiskLevelIncrease::OneLevel);
        }

        // Escalate based on destructive keywords in input
        let input_lower = intent.raw_input.to_lowercase();
        if input_lower.contains("delete") || input_lower.contains("remove") || input_lower.contains("destroy") {
            base_risk = self.escalate_risk_level(base_risk, RiskLevelIncrease::OneLevel);
        }

        Ok(base_risk)
    }

    /// Assess risk for individual step
    fn assess_step_risk(&self, step: &PlanStep) -> Result<RiskAssessment, PlanningError> {
        // Find matching risk rule
        if let Some(rule) = self.find_matching_risk_rule(&step.command) {
            let mut risk_level = rule.base_risk_level;
            let mut approval_required = rule.always_require_approval;

            // Apply escalation factors
            for factor in &rule.escalation_factors {
                if self.check_escalation_condition(&factor.condition, step) {
                    risk_level = self.escalate_risk_level(risk_level, factor.risk_increase.clone());
                }
            }

            // Check if approval is required based on risk level
            if !approval_required {
                approval_required = self.requires_approval(risk_level);
            }

            Ok(RiskAssessment {
                risk_level,
                potential_impacts: rule.potential_impacts.clone(),
                mitigation_strategies: rule.required_mitigations.clone(),
                approval_required,
            })
        } else {
            // Use default assessment for unknown commands
            Ok(RiskAssessment {
                risk_level: self.config.default_risk_level,
                potential_impacts: vec![
                    Impact {
                        description: "Unknown operation may have unpredictable effects".to_string(),
                        severity: self.config.default_risk_level,
                        affected_components: vec!["system".to_string()],
                    }
                ],
                mitigation_strategies: vec![
                    Mitigation {
                        description: "Manual review of unknown operation".to_string(),
                        implementation: "Require explicit approval for unknown commands".to_string(),
                    }
                ],
                approval_required: true,
            })
        }
    }

    /// Find matching risk rule for a command
    fn find_matching_risk_rule(&self, command: &str) -> Option<&RiskRule> {
        // Try exact match first
        if let Some(rule) = self.risk_rules.get(command) {
            return Some(rule);
        }

        // Try pattern matching
        for (pattern, rule) in &self.risk_rules {
            if command.starts_with(pattern) || pattern.contains("*") {
                return Some(rule);
            }
        }

        None
    }

    /// Apply escalation factors based on plan characteristics
    fn apply_escalation_factors(
        &self,
        mut risk_level: RiskLevel,
        intent: &Intent,
        steps: &[PlanStep],
    ) -> Result<RiskLevel, PlanningError> {
        // Escalate based on step count
        if steps.len() > self.config.escalation_thresholds.step_count_threshold {
            risk_level = self.escalate_risk_level(risk_level, RiskLevelIncrease::OneLevel);
        }

        // Escalate based on total execution time
        let total_time: std::time::Duration = steps.iter().map(|s| s.timeout).sum();
        if total_time.as_secs() > self.config.escalation_thresholds.execution_time_threshold {
            risk_level = self.escalate_risk_level(risk_level, RiskLevelIncrease::OneLevel);
        }

        // Escalate based on target sensitivity
        for target in &intent.targets {
            match target.target_type {
                TargetType::System => {
                    risk_level = self.escalate_risk_level(risk_level, RiskLevelIncrease::OneLevel);
                },
                TargetType::Configuration => {
                    risk_level = self.escalate_risk_level(risk_level, RiskLevelIncrease::OneLevel);
                },
                _ => {}
            }
        }

        Ok(risk_level)
    }

    /// Check escalation condition
    fn check_escalation_condition(&self, condition: &EscalationCondition, step: &PlanStep) -> bool {
        match condition {
            EscalationCondition::ParameterContains { key, value } => {
                if let Some(param_value) = step.parameters.get(key) {
                    if let Ok(param_str) = serde_json::from_value::<String>(param_value.clone()) {
                        return param_str.contains(value);
                    }
                }
                false
            },
            EscalationCondition::StepCountExceeds(_threshold) => {
                // This would be checked at the plan level, not step level
                false
            },
            EscalationCondition::ExecutionTimeExceeds(threshold) => {
                step.timeout > *threshold
            },
            EscalationCondition::MultipleExclusiveResources => {
                // This would be checked at the plan level
                false
            },
            EscalationCondition::TargetTypeMatches(_) => {
                // This would need target information from the intent
                false
            },
        }
    }

    /// Escalate risk level
    fn escalate_risk_level(&self, current: RiskLevel, increase: RiskLevelIncrease) -> RiskLevel {
        match increase {
            RiskLevelIncrease::OneLevel => {
                match current {
                    RiskLevel::Low => RiskLevel::Medium,
                    RiskLevel::Medium => RiskLevel::High,
                    RiskLevel::High => RiskLevel::Critical,
                    RiskLevel::Critical => RiskLevel::Critical,
                }
            },
            RiskLevelIncrease::TwoLevels => {
                match current {
                    RiskLevel::Low => RiskLevel::High,
                    RiskLevel::Medium => RiskLevel::Critical,
                    RiskLevel::High => RiskLevel::Critical,
                    RiskLevel::Critical => RiskLevel::Critical,
                }
            },
            RiskLevelIncrease::SetTo(level) => level,
        }
    }

    /// Check if approval is required for risk level
    fn requires_approval(&self, risk_level: RiskLevel) -> bool {
        match risk_level {
            RiskLevel::Low => false,
            RiskLevel::Medium => self.config.require_approval_medium_risk,
            RiskLevel::High => self.config.require_approval_high_risk,
            RiskLevel::Critical => true,
        }
    }

    /// Check security policies
    fn check_security_policies(&self, intent: &Intent, steps: &[PlanStep]) -> Result<(), PlanningError> {
        // Check for forbidden commands
        for step in steps {
            for forbidden in &self.security_policies.forbidden_commands {
                if step.command.contains(forbidden) {
                    return Err(PlanningError::ForbiddenOperation {
                        operation: step.command.clone(),
                        reason: format!("Command contains forbidden pattern: {}", forbidden),
                    });
                }
            }
        }

        // Check for restricted paths
        for target in &intent.targets {
            if target.target_type == TargetType::File {
                for restricted in &self.security_policies.restricted_paths {
                    if target.identifier.starts_with(restricted) {
                        return Err(PlanningError::ForbiddenOperation {
                            operation: format!("Access to {}", target.identifier),
                            reason: format!("Path is restricted: {}", restricted),
                        });
                    }
                }
            }
        }

        Ok(())
    }

    /// Initialize default risk rules
    fn initialize_default_risk_rules(&mut self) {
        // File operation rules
        self.risk_rules.insert("file_operation".to_string(), RiskRule {
            command_pattern: "file_operation".to_string(),
            base_risk_level: RiskLevel::Medium,
            potential_impacts: vec![
                Impact {
                    description: "File system changes may affect data integrity".to_string(),
                    severity: RiskLevel::Medium,
                    affected_components: vec!["file_system".to_string()],
                }
            ],
            required_mitigations: vec![
                Mitigation {
                    description: "Create backup before file operations".to_string(),
                    implementation: "Automated backup in file operation steps".to_string(),
                }
            ],
            always_require_approval: false,
            escalation_factors: vec![
                RiskEscalationFactor {
                    description: "Destructive file operation".to_string(),
                    condition: EscalationCondition::ParameterContains {
                        key: "operation".to_string(),
                        value: "delete".to_string(),
                    },
                    risk_increase: RiskLevelIncrease::OneLevel,
                }
            ],
        });

        // Configuration rules
        self.risk_rules.insert("apply_config".to_string(), RiskRule {
            command_pattern: "apply_config".to_string(),
            base_risk_level: RiskLevel::High,
            potential_impacts: vec![
                Impact {
                    description: "Configuration changes may affect system stability".to_string(),
                    severity: RiskLevel::High,
                    affected_components: vec!["system_configuration".to_string()],
                }
            ],
            required_mitigations: vec![
                Mitigation {
                    description: "Backup current configuration".to_string(),
                    implementation: "Configuration backup step".to_string(),
                },
                Mitigation {
                    description: "Validate new configuration".to_string(),
                    implementation: "Configuration validation step".to_string(),
                }
            ],
            always_require_approval: true,
            escalation_factors: Vec::new(),
        });

        // Process management rules
        self.risk_rules.insert("process_management".to_string(), RiskRule {
            command_pattern: "process_management".to_string(),
            base_risk_level: RiskLevel::High,
            potential_impacts: vec![
                Impact {
                    description: "Process operations may affect system stability".to_string(),
                    severity: RiskLevel::High,
                    affected_components: vec!["running_processes".to_string()],
                }
            ],
            required_mitigations: vec![
                Mitigation {
                    description: "Validate process permissions".to_string(),
                    implementation: "Process permission validation".to_string(),
                }
            ],
            always_require_approval: true,
            escalation_factors: Vec::new(),
        });

        // Query rules (low risk)
        self.risk_rules.insert("execute_query".to_string(), RiskRule {
            command_pattern: "execute_query".to_string(),
            base_risk_level: RiskLevel::Low,
            potential_impacts: vec![
                Impact {
                    description: "Query operations have minimal system impact".to_string(),
                    severity: RiskLevel::Low,
                    affected_components: vec!["query_engine".to_string()],
                }
            ],
            required_mitigations: Vec::new(),
            always_require_approval: false,
            escalation_factors: Vec::new(),
        });

        // Validation rules (low risk)
        self.risk_rules.insert("validate_".to_string(), RiskRule {
            command_pattern: "validate_".to_string(),
            base_risk_level: RiskLevel::Low,
            potential_impacts: Vec::new(),
            required_mitigations: Vec::new(),
            always_require_approval: false,
            escalation_factors: Vec::new(),
        });
    }

    /// Update security policies
    pub fn update_security_policies(&mut self, policies: SecurityPolicies) {
        self.security_policies = policies;
    }

    /// Add custom risk rule
    pub fn add_risk_rule(&mut self, pattern: String, rule: RiskRule) {
        self.risk_rules.insert(pattern, rule);
    }

    /// Get current security policies
    pub fn get_security_policies(&self) -> &SecurityPolicies {
        &self.security_policies
    }
}

impl Default for RiskAssessor {
    fn default() -> Self {
        Self::new()
    }
}