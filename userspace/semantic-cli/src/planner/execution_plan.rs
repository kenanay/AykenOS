//! Execution plan generation and management
//!
//! This module provides functionality for creating, validating, and managing
//! execution plans with detailed step decomposition and metadata.

use crate::types::*;
use crate::error::PlanningError;
use std::collections::HashMap;
use uuid::Uuid;
use tracing::debug;

/// Execution plan generator with advanced decomposition capabilities
pub struct ExecutionPlanGenerator {
    /// Configuration for plan generation
    config: PlanGenerationConfig,
}

/// Configuration for execution plan generation
#[derive(Debug, Clone)]
pub struct PlanGenerationConfig {
    /// Maximum number of steps per plan
    pub max_steps: usize,
    /// Default step timeout
    pub default_timeout: std::time::Duration,
    /// Whether to generate detailed preconditions
    pub detailed_preconditions: bool,
    /// Whether to generate rollback plans
    pub generate_rollback: bool,
}

impl Default for PlanGenerationConfig {
    fn default() -> Self {
        Self {
            max_steps: 50,
            default_timeout: std::time::Duration::from_secs(60),
            detailed_preconditions: true,
            generate_rollback: true,
        }
    }
}

impl ExecutionPlanGenerator {
    /// Create a new execution plan generator
    pub fn new() -> Self {
        Self {
            config: PlanGenerationConfig::default(),
        }
    }

    /// Create generator with custom configuration
    pub fn with_config(config: PlanGenerationConfig) -> Self {
        Self { config }
    }

    /// Generate a comprehensive execution plan from an intent
    pub async fn generate_comprehensive_plan(&self, intent: &Intent) -> Result<ExecutionPlan, PlanningError> {
        debug!("Generating comprehensive plan for intent: {:?}", intent.id);

        let mut plan = ExecutionPlan::new(intent.id);

        // Generate steps based on intent complexity
        let steps = self.generate_steps_for_intent(intent).await?;
        
        // Validate step count
        if steps.len() > self.config.max_steps {
            return Err(PlanningError::TooManySteps {
                requested: steps.len(),
                maximum: self.config.max_steps,
            });
        }

        plan.steps = steps;

        // Generate dependencies if we have multiple steps
        if plan.steps.len() > 1 {
            plan.dependencies = self.generate_step_dependencies(&plan.steps).await?;
        }

        // Estimate execution time
        plan.estimated_time = self.calculate_execution_time(&plan.steps, &plan.dependencies).await?;

        // Generate resource requirements
        plan.resource_requirements = self.generate_resource_requirements(&plan.steps).await?;

        // Generate risk assessment
        plan.risk_assessment = self.generate_risk_assessment(intent, &plan.steps).await?;

        // Generate rollback plan if configured and needed
        if self.config.generate_rollback && plan.risk_assessment.risk_level != RiskLevel::Low {
            plan.rollback_plan = Some(self.generate_rollback_plan(&plan.steps).await?);
        }

        debug!("Generated plan with {} steps and {} dependencies", 
               plan.steps.len(), plan.dependencies.len());

        Ok(plan)
    }

    /// Generate steps for a specific intent
    async fn generate_steps_for_intent(&self, intent: &Intent) -> Result<Vec<PlanStep>, PlanningError> {
        match intent.action {
            ActionType::Query => self.generate_query_steps(intent).await,
            ActionType::Command => self.generate_command_steps(intent).await,
            ActionType::Configuration => self.generate_configuration_steps(intent).await,
            ActionType::Analysis => self.generate_analysis_steps(intent).await,
            ActionType::Monitoring => self.generate_monitoring_steps(intent).await,
            ActionType::FileOperation => self.generate_file_operation_steps(intent).await,
            ActionType::ProcessManagement => self.generate_process_management_steps(intent).await,
        }
    }

    /// Generate steps for query operations
    async fn generate_query_steps(&self, intent: &Intent) -> Result<Vec<PlanStep>, PlanningError> {
        let mut steps = Vec::new();

        // Step 1: Validate query parameters
        steps.push(self.create_step(
            "validate_query_parameters",
            "Validate query parameters and check permissions",
            &intent.parameters,
            Vec::new(),
            vec![self.create_condition("query_validated", ConditionType::Custom, true)],
            std::time::Duration::from_secs(10),
        ));

        // Step 2: Prepare query context
        steps.push(self.create_step(
            "prepare_query_context",
            "Prepare execution context for query",
            &HashMap::new(),
            vec![self.create_condition("query_validated", ConditionType::Custom, true)],
            vec![self.create_condition("context_prepared", ConditionType::Custom, true)],
            std::time::Duration::from_secs(15),
        ));

        // Step 3: Execute query
        steps.push(self.create_step(
            "execute_query",
            "Execute the query operation",
            &intent.parameters,
            vec![self.create_condition("context_prepared", ConditionType::Custom, true)],
            vec![self.create_condition("query_completed", ConditionType::Custom, true)],
            self.config.default_timeout,
        ));

        // Step 4: Format results
        steps.push(self.create_step(
            "format_query_results",
            "Format and present query results",
            &HashMap::new(),
            vec![self.create_condition("query_completed", ConditionType::Custom, true)],
            Vec::new(),
            std::time::Duration::from_secs(5),
        ));

        Ok(steps)
    }

    /// Generate steps for command operations
    async fn generate_command_steps(&self, intent: &Intent) -> Result<Vec<PlanStep>, PlanningError> {
        let mut steps = Vec::new();

        // Step 1: Security validation
        steps.push(self.create_step(
            "validate_command_security",
            "Validate command security and permissions",
            &intent.parameters,
            Vec::new(),
            vec![self.create_condition("security_validated", ConditionType::Custom, true)],
            std::time::Duration::from_secs(20),
        ));

        // Step 2: Resource check
        steps.push(self.create_step(
            "check_command_resources",
            "Check available resources for command execution",
            &HashMap::new(),
            vec![self.create_condition("security_validated", ConditionType::Custom, true)],
            vec![self.create_condition("resources_available", ConditionType::ResourceAvailable, true)],
            std::time::Duration::from_secs(10),
        ));

        // Step 3: Prepare execution environment
        steps.push(self.create_step(
            "prepare_execution_environment",
            "Prepare isolated execution environment",
            &HashMap::new(),
            vec![self.create_condition("resources_available", ConditionType::ResourceAvailable, true)],
            vec![self.create_condition("environment_ready", ConditionType::Custom, true)],
            std::time::Duration::from_secs(30),
        ));

        // Step 4: Execute command
        steps.push(self.create_step(
            "execute_command",
            "Execute the command in prepared environment",
            &intent.parameters,
            vec![self.create_condition("environment_ready", ConditionType::Custom, true)],
            vec![self.create_condition("command_completed", ConditionType::Custom, true)],
            self.config.default_timeout,
        ));

        // Step 5: Cleanup
        steps.push(self.create_step(
            "cleanup_execution_environment",
            "Clean up execution environment and resources",
            &HashMap::new(),
            vec![self.create_condition("command_completed", ConditionType::Custom, true)],
            Vec::new(),
            std::time::Duration::from_secs(15),
        ));

        Ok(steps)
    }

    /// Generate steps for configuration operations
    async fn generate_configuration_steps(&self, intent: &Intent) -> Result<Vec<PlanStep>, PlanningError> {
        let mut steps = Vec::new();

        // Step 1: Backup current configuration
        steps.push(self.create_step(
            "backup_current_configuration",
            "Create backup of current configuration",
            &HashMap::new(),
            Vec::new(),
            vec![self.create_condition("config_backed_up", ConditionType::FileExists, "backup_created")],
            std::time::Duration::from_secs(30),
        ));

        // Step 2: Validate new configuration
        steps.push(self.create_step(
            "validate_new_configuration",
            "Validate new configuration parameters",
            &intent.parameters,
            vec![self.create_condition("config_backed_up", ConditionType::FileExists, "backup_created")],
            vec![self.create_condition("config_validated", ConditionType::Custom, true)],
            std::time::Duration::from_secs(25),
        ));

        // Step 3: Test configuration compatibility
        steps.push(self.create_step(
            "test_configuration_compatibility",
            "Test configuration compatibility with system",
            &intent.parameters,
            vec![self.create_condition("config_validated", ConditionType::Custom, true)],
            vec![self.create_condition("config_compatible", ConditionType::Custom, true)],
            std::time::Duration::from_secs(40),
        ));

        // Step 4: Apply configuration
        steps.push(self.create_step(
            "apply_configuration",
            "Apply the new configuration",
            &intent.parameters,
            vec![self.create_condition("config_compatible", ConditionType::Custom, true)],
            vec![self.create_condition("config_applied", ConditionType::Custom, true)],
            std::time::Duration::from_secs(60),
        ));

        // Step 5: Verify configuration
        steps.push(self.create_step(
            "verify_configuration",
            "Verify configuration was applied correctly",
            &HashMap::new(),
            vec![self.create_condition("config_applied", ConditionType::Custom, true)],
            Vec::new(),
            std::time::Duration::from_secs(20),
        ));

        Ok(steps)
    }

    /// Generate steps for analysis operations
    async fn generate_analysis_steps(&self, intent: &Intent) -> Result<Vec<PlanStep>, PlanningError> {
        let mut steps = Vec::new();

        // Step 1: Collect analysis data
        steps.push(self.create_step(
            "collect_analysis_data",
            "Collect data required for analysis",
            &intent.parameters,
            Vec::new(),
            vec![self.create_condition("data_collected", ConditionType::Custom, true)],
            std::time::Duration::from_secs(60),
        ));

        // Step 2: Prepare analysis tools
        steps.push(self.create_step(
            "prepare_analysis_tools",
            "Prepare and configure analysis tools",
            &HashMap::new(),
            vec![self.create_condition("data_collected", ConditionType::Custom, true)],
            vec![self.create_condition("tools_ready", ConditionType::Custom, true)],
            std::time::Duration::from_secs(30),
        ));

        // Step 3: Perform analysis
        steps.push(self.create_step(
            "perform_analysis",
            "Execute the analysis operation",
            &intent.parameters,
            vec![self.create_condition("tools_ready", ConditionType::Custom, true)],
            vec![self.create_condition("analysis_completed", ConditionType::Custom, true)],
            std::time::Duration::from_secs(120),
        ));

        // Step 4: Generate report
        steps.push(self.create_step(
            "generate_analysis_report",
            "Generate analysis report and recommendations",
            &HashMap::new(),
            vec![self.create_condition("analysis_completed", ConditionType::Custom, true)],
            Vec::new(),
            std::time::Duration::from_secs(30),
        ));

        Ok(steps)
    }

    /// Generate steps for monitoring operations
    async fn generate_monitoring_steps(&self, intent: &Intent) -> Result<Vec<PlanStep>, PlanningError> {
        let mut steps = Vec::new();

        // Step 1: Configure monitoring parameters
        steps.push(self.create_step(
            "configure_monitoring",
            "Configure monitoring parameters and thresholds",
            &intent.parameters,
            Vec::new(),
            vec![self.create_condition("monitoring_configured", ConditionType::Custom, true)],
            std::time::Duration::from_secs(20),
        ));

        // Step 2: Initialize monitoring infrastructure
        steps.push(self.create_step(
            "initialize_monitoring_infrastructure",
            "Initialize monitoring infrastructure and collectors",
            &HashMap::new(),
            vec![self.create_condition("monitoring_configured", ConditionType::Custom, true)],
            vec![self.create_condition("infrastructure_ready", ConditionType::Custom, true)],
            std::time::Duration::from_secs(45),
        ));

        // Step 3: Start monitoring
        steps.push(self.create_step(
            "start_monitoring",
            "Start the monitoring process",
            &intent.parameters,
            vec![self.create_condition("infrastructure_ready", ConditionType::Custom, true)],
            Vec::new(),
            std::time::Duration::from_secs(10),
        ));

        Ok(steps)
    }

    /// Generate steps for file operations
    async fn generate_file_operation_steps(&self, intent: &Intent) -> Result<Vec<PlanStep>, PlanningError> {
        let mut steps = Vec::new();

        // Step 1: Validate file permissions
        steps.push(self.create_step(
            "validate_file_permissions",
            "Validate file access permissions and ownership",
            &intent.parameters,
            Vec::new(),
            vec![self.create_condition("permissions_validated", ConditionType::Custom, true)],
            std::time::Duration::from_secs(10),
        ));

        // Step 2: Check file system space
        steps.push(self.create_step(
            "check_filesystem_space",
            "Check available file system space",
            &HashMap::new(),
            vec![self.create_condition("permissions_validated", ConditionType::Custom, true)],
            vec![self.create_condition("space_available", ConditionType::ResourceAvailable, true)],
            std::time::Duration::from_secs(5),
        ));

        // Step 3: Create backup if needed
        if self.requires_backup(intent) {
            steps.push(self.create_step(
                "create_file_backup",
                "Create backup of files before operation",
                &intent.parameters,
                vec![self.create_condition("space_available", ConditionType::ResourceAvailable, true)],
                vec![self.create_condition("backup_created", ConditionType::FileExists, "backup")],
                std::time::Duration::from_secs(60),
            ));
        }

        // Step 4: Perform file operation
        let preconditions = if self.requires_backup(intent) {
            vec![self.create_condition("backup_created", ConditionType::FileExists, "backup")]
        } else {
            vec![self.create_condition("space_available", ConditionType::ResourceAvailable, true)]
        };

        steps.push(self.create_step(
            "perform_file_operation",
            "Perform the requested file operation",
            &intent.parameters,
            preconditions,
            vec![self.create_condition("operation_completed", ConditionType::Custom, true)],
            self.config.default_timeout,
        ));

        // Step 5: Verify operation
        steps.push(self.create_step(
            "verify_file_operation",
            "Verify file operation completed successfully",
            &HashMap::new(),
            vec![self.create_condition("operation_completed", ConditionType::Custom, true)],
            Vec::new(),
            std::time::Duration::from_secs(15),
        ));

        Ok(steps)
    }

    /// Generate steps for process management operations
    async fn generate_process_management_steps(&self, intent: &Intent) -> Result<Vec<PlanStep>, PlanningError> {
        let mut steps = Vec::new();

        // Step 1: Validate process permissions
        steps.push(self.create_step(
            "validate_process_permissions",
            "Validate process management permissions",
            &intent.parameters,
            Vec::new(),
            vec![self.create_condition("permissions_validated", ConditionType::Custom, true)],
            std::time::Duration::from_secs(10),
        ));

        // Step 2: Identify target processes
        steps.push(self.create_step(
            "identify_target_processes",
            "Identify and validate target processes",
            &intent.parameters,
            vec![self.create_condition("permissions_validated", ConditionType::Custom, true)],
            vec![self.create_condition("processes_identified", ConditionType::ProcessRunning, "target")],
            std::time::Duration::from_secs(15),
        ));

        // Step 3: Perform process operation
        steps.push(self.create_step(
            "perform_process_operation",
            "Perform the requested process management operation",
            &intent.parameters,
            vec![self.create_condition("processes_identified", ConditionType::ProcessRunning, "target")],
            vec![self.create_condition("operation_completed", ConditionType::Custom, true)],
            self.config.default_timeout,
        ));

        // Step 4: Verify operation
        steps.push(self.create_step(
            "verify_process_operation",
            "Verify process operation completed successfully",
            &HashMap::new(),
            vec![self.create_condition("operation_completed", ConditionType::Custom, true)],
            Vec::new(),
            std::time::Duration::from_secs(10),
        ));

        Ok(steps)
    }

    /// Create a plan step with the given parameters
    fn create_step(
        &self,
        command: &str,
        description: &str,
        parameters: &HashMap<String, serde_json::Value>,
        preconditions: Vec<Condition>,
        postconditions: Vec<Condition>,
        timeout: std::time::Duration,
    ) -> PlanStep {
        PlanStep {
            id: Uuid::new_v4(),
            command: command.to_string(),
            parameters: parameters.clone(),
            preconditions,
            postconditions,
            timeout,
            description: description.to_string(),
        }
    }

    /// Create a condition with the given parameters
    fn create_condition<T: Into<serde_json::Value>>(
        &self,
        description: &str,
        condition_type: ConditionType,
        expected: T,
    ) -> Condition {
        Condition {
            description: description.to_string(),
            condition_type,
            expected: expected.into(),
        }
    }

    /// Generate step dependencies based on preconditions and postconditions
    async fn generate_step_dependencies(&self, steps: &[PlanStep]) -> Result<Vec<Dependency>, PlanningError> {
        let mut dependencies = Vec::new();

        // Create a map of postconditions to steps
        let mut postcondition_map: HashMap<String, &PlanStep> = HashMap::new();
        for step in steps {
            for postcondition in &step.postconditions {
                postcondition_map.insert(postcondition.description.clone(), step);
            }
        }

        // Find dependencies based on preconditions
        for step in steps {
            for precondition in &step.preconditions {
                if let Some(prerequisite_step) = postcondition_map.get(&precondition.description) {
                    if prerequisite_step.id != step.id {
                        dependencies.push(Dependency {
                            prerequisite: prerequisite_step.id,
                            dependent: step.id,
                            dependency_type: DependencyType::Success,
                        });
                    }
                }
            }
        }

        Ok(dependencies)
    }

    /// Calculate execution time considering dependencies
    async fn calculate_execution_time(
        &self,
        steps: &[PlanStep],
        _dependencies: &[Dependency],
    ) -> Result<std::time::Duration, PlanningError> {
        // Simple approach: sum all timeouts plus coordination overhead
        let total_step_time: std::time::Duration = steps.iter()
            .map(|step| step.timeout)
            .sum();

        // Add coordination overhead (2 seconds per step)
        let coordination_overhead = std::time::Duration::from_secs(steps.len() as u64 * 2);

        Ok(total_step_time + coordination_overhead)
    }

    /// Generate resource requirements for the plan
    async fn generate_resource_requirements(&self, steps: &[PlanStep]) -> Result<ResourceRequirements, PlanningError> {
        // Estimate based on step types and parameters
        let mut cpu_usage: f32 = 0.1; // Base CPU usage
        let mut memory_usage = 1024 * 1024 * 100; // Base 100MB
        let mut disk_space = 0;
        let mut network_bandwidth = 0;
        let mut exclusive_resources = Vec::new();

        for step in steps {
            match step.command.as_str() {
                "execute_command" | "perform_analysis" => {
                    cpu_usage += 0.2;
                    memory_usage += 1024 * 1024 * 200; // Additional 200MB
                },
                "perform_file_operation" | "create_file_backup" => {
                    disk_space += 1024 * 1024 * 500; // 500MB for file operations
                },
                "collect_analysis_data" => {
                    network_bandwidth += 1024 * 1024; // 1MB/s
                    memory_usage += 1024 * 1024 * 300; // Additional 300MB
                },
                "apply_configuration" => {
                    exclusive_resources.push("system_config".to_string());
                },
                _ => {
                    cpu_usage += 0.05;
                    memory_usage += 1024 * 1024 * 50; // Additional 50MB
                }
            }
        }

        Ok(ResourceRequirements {
            cpu_usage: cpu_usage.min(1.0),
            memory_usage,
            disk_space,
            network_bandwidth,
            exclusive_resources,
        })
    }

    /// Generate risk assessment for the plan
    async fn generate_risk_assessment(&self, intent: &Intent, steps: &[PlanStep]) -> Result<RiskAssessment, PlanningError> {
        let mut risk_level = RiskLevel::Low;
        let mut potential_impacts = Vec::new();
        let mut mitigation_strategies = Vec::new();
        let mut approval_required = false;

        // Assess risk based on intent action
        match intent.action {
            ActionType::Configuration => {
                risk_level = RiskLevel::Medium;
                approval_required = true;
                potential_impacts.push(Impact {
                    description: "System configuration changes may affect stability".to_string(),
                    severity: RiskLevel::Medium,
                    affected_components: vec!["system_config".to_string()],
                });
                mitigation_strategies.push(Mitigation {
                    description: "Create configuration backup before changes".to_string(),
                    implementation: "Automated backup in backup_current_configuration step".to_string(),
                });
            },
            ActionType::ProcessManagement => {
                risk_level = RiskLevel::High;
                approval_required = true;
                potential_impacts.push(Impact {
                    description: "Process management operations may affect system stability".to_string(),
                    severity: RiskLevel::High,
                    affected_components: vec!["running_processes".to_string()],
                });
                mitigation_strategies.push(Mitigation {
                    description: "Validate process permissions and ownership".to_string(),
                    implementation: "Permission validation in validate_process_permissions step".to_string(),
                });
            },
            ActionType::FileOperation => {
                if self.is_destructive_file_operation(intent) {
                    risk_level = RiskLevel::Medium;
                    approval_required = true;
                    potential_impacts.push(Impact {
                        description: "File operations may result in data loss".to_string(),
                        severity: RiskLevel::Medium,
                        affected_components: vec!["file_system".to_string()],
                    });
                    mitigation_strategies.push(Mitigation {
                        description: "Create file backups before destructive operations".to_string(),
                        implementation: "Backup creation in create_file_backup step".to_string(),
                    });
                }
            },
            _ => {
                // Query, Analysis, Monitoring, Command are generally lower risk
                risk_level = RiskLevel::Low;
            }
        }

        // Increase risk if many steps are involved
        if steps.len() > 10 {
            risk_level = match risk_level {
                RiskLevel::Low => RiskLevel::Medium,
                RiskLevel::Medium => RiskLevel::High,
                RiskLevel::High => RiskLevel::Critical,
                RiskLevel::Critical => RiskLevel::Critical,
            };
        }

        Ok(RiskAssessment {
            risk_level,
            potential_impacts,
            mitigation_strategies,
            approval_required,
        })
    }

    /// Generate rollback plan for the steps
    async fn generate_rollback_plan(&self, steps: &[PlanStep]) -> Result<RollbackPlan, PlanningError> {
        let mut rollback_steps = Vec::new();

        // Generate rollback steps in reverse order
        for step in steps.iter().rev() {
            if let Some(rollback_step) = self.create_rollback_step(step) {
                rollback_steps.push(rollback_step);
            }
        }

        Ok(RollbackPlan {
            steps: rollback_steps,
            description: format!("Rollback plan to undo {} operations", steps.len()),
        })
    }

    /// Create rollback step for a given step
    fn create_rollback_step(&self, step: &PlanStep) -> Option<PlanStep> {
        match step.command.as_str() {
            "apply_configuration" => Some(PlanStep {
                id: Uuid::new_v4(),
                command: "restore_configuration_backup".to_string(),
                parameters: HashMap::new(),
                preconditions: Vec::new(),
                postconditions: Vec::new(),
                timeout: std::time::Duration::from_secs(60),
                description: "Restore configuration from backup".to_string(),
            }),
            "perform_file_operation" => Some(PlanStep {
                id: Uuid::new_v4(),
                command: "restore_file_backup".to_string(),
                parameters: step.parameters.clone(),
                preconditions: Vec::new(),
                postconditions: Vec::new(),
                timeout: std::time::Duration::from_secs(60),
                description: "Restore files from backup".to_string(),
            }),
            "perform_process_operation" => Some(PlanStep {
                id: Uuid::new_v4(),
                command: "restore_process_state".to_string(),
                parameters: step.parameters.clone(),
                preconditions: Vec::new(),
                postconditions: Vec::new(),
                timeout: std::time::Duration::from_secs(30),
                description: "Restore previous process state".to_string(),
            }),
            _ => None,
        }
    }

    /// Check if intent requires backup
    fn requires_backup(&self, intent: &Intent) -> bool {
        let input_lower = intent.raw_input.to_lowercase();
        input_lower.contains("delete") || 
        input_lower.contains("remove") || 
        input_lower.contains("modify") ||
        input_lower.contains("change")
    }

    /// Check if file operation is destructive
    fn is_destructive_file_operation(&self, intent: &Intent) -> bool {
        let input_lower = intent.raw_input.to_lowercase();
        input_lower.contains("delete") || 
        input_lower.contains("remove") || 
        input_lower.contains("truncate") ||
        input_lower.contains("overwrite")
    }
}

impl Default for ExecutionPlanGenerator {
    fn default() -> Self {
        Self::new()
    }
}