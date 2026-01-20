//! AI Planner module for execution plan generation
//!
//! This module provides the AI planner functionality that converts structured intents
//! into detailed execution plans with step decomposition, dependency analysis,
//! resource estimation, and risk assessment capabilities.

pub mod execution_plan;
pub mod replay;
pub mod dependency;
pub mod resource_estimation;
pub mod risk_assessment;

pub use execution_plan::*;
pub use replay::*;
pub use dependency::*;
pub use resource_estimation::*;
pub use risk_assessment::*;

use crate::types::*;
use crate::error::PlanningError;
use std::collections::HashMap;
use tracing::{debug, info};
use uuid::Uuid;

/// Enhanced AI planner with step decomposition and dependency analysis
pub struct EnhancedAIPlanner {
    /// Replay controller for deterministic execution
    replay_controller: ReplayController,
    /// Dependency analyzer for step relationships
    dependency_analyzer: DependencyAnalyzer,
    /// Resource estimator for plan requirements
    resource_estimator: ResourceEstimator,
    /// Risk assessor for security and safety evaluation
    risk_assessor: RiskAssessor,
    /// Planning context and configuration
    planning_context: PlanningContext,
}

/// Planning context and configuration
#[derive(Debug, Clone)]
pub struct PlanningContext {
    /// Whether deterministic replay mode is enabled
    pub deterministic_mode: bool,
    /// Maximum number of steps allowed in a plan
    pub max_steps: usize,
    /// Default timeout for steps
    pub default_timeout: std::time::Duration,
    /// Planning session ID for replay
    pub session_id: String,
    /// User context for planning decisions
    pub user_context: UserContext,
}

impl Default for PlanningContext {
    fn default() -> Self {
        Self {
            deterministic_mode: false,
            max_steps: 50,
            default_timeout: std::time::Duration::from_secs(60),
            session_id: Uuid::new_v4().to_string(),
            user_context: UserContext::default(),
        }
    }
}

impl EnhancedAIPlanner {
    /// Create a new enhanced AI planner
    pub fn new() -> Self {
        Self {
            replay_controller: ReplayController::new(),
            dependency_analyzer: DependencyAnalyzer::new(),
            resource_estimator: ResourceEstimator::new(),
            risk_assessor: RiskAssessor::new(),
            planning_context: PlanningContext::default(),
        }
    }

    /// Create planner with custom context
    pub fn with_context(context: PlanningContext) -> Self {
        let mut planner = Self::new();
        planner.planning_context = context;
        planner
    }

    /// Enable deterministic replay mode for audit and debugging
    pub fn enable_replay_mode(&mut self, deterministic: bool) {
        info!("Setting deterministic replay mode: {}", deterministic);
        self.planning_context.deterministic_mode = deterministic;
        self.replay_controller.set_deterministic_mode(deterministic);
    }

    /// Update planning context
    pub fn update_context(&mut self, context: PlanningContext) {
        self.planning_context = context;
    }

    /// Generate execution plan from intent with full decomposition and analysis
    pub async fn generate_plan(&mut self, intent: &Intent) -> Result<ExecutionPlan, PlanningError> {
        info!("Generating execution plan for intent: {:?}", intent.id);
        debug!("Intent action: {:?}, targets: {:?}", intent.action, intent.targets);

        // Check replay mode first
        if self.planning_context.deterministic_mode {
            if let Some(cached_plan) = self.replay_controller.get_cached_plan(intent).await? {
                info!("Using cached plan from replay mode");
                return Ok(cached_plan);
            }
        }

        // Create base execution plan
        let mut plan = ExecutionPlan::new(intent.id);

        // Step 1: Decompose intent into atomic steps
        let atomic_steps = self.decompose_intent_to_steps(intent).await?;
        debug!("Decomposed intent into {} atomic steps", atomic_steps.len());

        // Step 2: Analyze dependencies between steps
        let dependencies = self.dependency_analyzer.analyze_dependencies(&atomic_steps).await?;
        debug!("Identified {} dependencies", dependencies.len());

        // Step 3: Estimate resource requirements
        let resource_requirements = self.resource_estimator.estimate_plan_resources(&atomic_steps).await?;
        debug!("Estimated resource requirements: {:?}", resource_requirements);

        // Step 4: Perform risk assessment
        let risk_assessment = self.risk_assessor.assess_plan_risk(intent, &atomic_steps).await?;
        debug!("Risk assessment: {:?}", risk_assessment.risk_level);

        // Step 5: Generate rollback plan if needed
        let rollback_plan = if risk_assessment.risk_level != RiskLevel::Low {
            Some(self.generate_rollback_plan(&atomic_steps).await?)
        } else {
            None
        };

        // Step 6: Calculate estimated execution time
        let estimated_time = self.calculate_total_execution_time(&atomic_steps, &dependencies).await?;

        // Assemble the complete plan
        plan.steps = atomic_steps;
        plan.dependencies = dependencies;
        plan.resource_requirements = resource_requirements;
        plan.risk_assessment = risk_assessment;
        plan.rollback_plan = rollback_plan;
        plan.estimated_time = estimated_time;

        // Validate the plan
        self.validate_plan(&plan).await?;

        // Cache plan for replay if in deterministic mode
        if self.planning_context.deterministic_mode {
            self.replay_controller.cache_plan(intent, &plan).await?;
        }

        info!("Successfully generated execution plan with {} steps", plan.steps.len());
        Ok(plan)
    }

    /// Validate execution plan against system capabilities
    pub async fn validate_plan(&self, plan: &ExecutionPlan) -> Result<ValidationResult, PlanningError> {
        debug!("Validating execution plan: {:?}", plan.id);

        let mut validation_result = ValidationResult {
            is_valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
        };

        // Check step count limit
        if plan.steps.len() > self.planning_context.max_steps {
            validation_result.is_valid = false;
            validation_result.errors.push(format!(
                "Plan exceeds maximum step limit: {} > {}",
                plan.steps.len(),
                self.planning_context.max_steps
            ));
        }

        // Validate each step
        for step in &plan.steps {
            if let Err(step_error) = self.validate_step(step).await {
                validation_result.warnings.push(format!(
                    "Step {} validation warning: {}",
                    step.id,
                    step_error
                ));
            }
        }

        // Validate dependencies
        if let Err(dep_error) = self.dependency_analyzer.validate_dependencies(&plan.dependencies, &plan.steps).await {
            validation_result.is_valid = false;
            validation_result.errors.push(format!("Dependency validation failed: {}", dep_error));
        }

        // Validate resource requirements
        if let Err(resource_error) = self.resource_estimator.validate_resources(&plan.resource_requirements).await {
            validation_result.warnings.push(format!("Resource validation warning: {}", resource_error));
        }

        Ok(validation_result)
    }

    /// Estimate resource requirements for a plan
    pub async fn estimate_resources(&self, plan: &ExecutionPlan) -> Result<ResourceEstimate, PlanningError> {
        debug!("Estimating resources for plan: {:?}", plan.id);

        let cpu_usage = self.resource_estimator.estimate_cpu_usage(&plan.steps).await?;
        let memory_usage = self.resource_estimator.estimate_memory_usage(&plan.steps).await?;
        let disk_io = self.resource_estimator.estimate_disk_io(&plan.steps).await?;
        let network_io = self.resource_estimator.estimate_network_io(&plan.steps).await?;
        let estimated_duration = plan.estimated_time;

        // Calculate confidence interval (±20% for now)
        let duration_secs = estimated_duration.as_secs_f64();
        let confidence_interval = (
            std::time::Duration::from_secs_f64(duration_secs * 0.8),
            std::time::Duration::from_secs_f64(duration_secs * 1.2),
        );

        Ok(ResourceEstimate {
            cpu_usage,
            memory_usage,
            disk_io,
            network_io,
            estimated_duration,
            confidence_interval,
        })
    }

    /// Generate alternative plans for failed steps
    pub async fn generate_alternatives(&self, failed_step: &PlanStep) -> Result<Vec<ExecutionPlan>, PlanningError> {
        debug!("Generating alternatives for failed step: {:?}", failed_step.id);

        let mut alternatives = Vec::new();

        // Generate alternative approaches based on step type
        match failed_step.command.as_str() {
            "file_operation" => {
                alternatives.extend(self.generate_file_operation_alternatives(failed_step).await?);
            },
            "process_management" => {
                alternatives.extend(self.generate_process_alternatives(failed_step).await?);
            },
            "query" => {
                alternatives.extend(self.generate_query_alternatives(failed_step).await?);
            },
            _ => {
                // Generic fallback alternatives
                alternatives.extend(self.generate_generic_alternatives(failed_step).await?);
            }
        }

        // Limit alternatives to avoid overwhelming the user
        alternatives.truncate(3);

        Ok(alternatives)
    }

    /// Decompose intent into atomic executable steps
    async fn decompose_intent_to_steps(&self, intent: &Intent) -> Result<Vec<PlanStep>, PlanningError> {
        debug!("Decomposing intent: {:?}", intent.action);

        let mut steps = Vec::new();

        match intent.action {
            ActionType::Query => {
                steps.extend(self.decompose_query_intent(intent).await?);
            },
            ActionType::Command => {
                steps.extend(self.decompose_command_intent(intent).await?);
            },
            ActionType::Configuration => {
                steps.extend(self.decompose_config_intent(intent).await?);
            },
            ActionType::Analysis => {
                steps.extend(self.decompose_analysis_intent(intent).await?);
            },
            ActionType::Monitoring => {
                steps.extend(self.decompose_monitoring_intent(intent).await?);
            },
            ActionType::FileOperation => {
                steps.extend(self.decompose_file_operation_intent(intent).await?);
            },
            ActionType::ProcessManagement => {
                steps.extend(self.decompose_process_management_intent(intent).await?);
            },
        }

        if steps.is_empty() {
            return Err(PlanningError::NoStepsGenerated);
        }

        Ok(steps)
    }

    /// Decompose query intent into steps
    async fn decompose_query_intent(&self, intent: &Intent) -> Result<Vec<PlanStep>, PlanningError> {
        let mut steps = Vec::new();

        // Add validation step
        steps.push(PlanStep {
            id: Uuid::new_v4(),
            command: "validate_query".to_string(),
            parameters: intent.parameters.clone(),
            preconditions: Vec::new(),
            postconditions: vec![
                Condition {
                    description: "Query parameters validated".to_string(),
                    condition_type: ConditionType::Custom,
                    expected: serde_json::Value::Bool(true),
                }
            ],
            timeout: std::time::Duration::from_secs(10),
            description: "Validate query parameters and permissions".to_string(),
        });

        // Add execution step
        steps.push(PlanStep {
            id: Uuid::new_v4(),
            command: "execute_query".to_string(),
            parameters: intent.parameters.clone(),
            preconditions: vec![
                Condition {
                    description: "Query parameters validated".to_string(),
                    condition_type: ConditionType::Custom,
                    expected: serde_json::Value::Bool(true),
                }
            ],
            postconditions: Vec::new(),
            timeout: self.planning_context.default_timeout,
            description: "Execute the query operation".to_string(),
        });

        Ok(steps)
    }

    /// Decompose command intent into steps
    async fn decompose_command_intent(&self, intent: &Intent) -> Result<Vec<PlanStep>, PlanningError> {
        let mut steps = Vec::new();

        // Add pre-execution validation
        steps.push(PlanStep {
            id: Uuid::new_v4(),
            command: "validate_command".to_string(),
            parameters: intent.parameters.clone(),
            preconditions: Vec::new(),
            postconditions: vec![
                Condition {
                    description: "Command validated and safe to execute".to_string(),
                    condition_type: ConditionType::Custom,
                    expected: serde_json::Value::Bool(true),
                }
            ],
            timeout: std::time::Duration::from_secs(15),
            description: "Validate command safety and permissions".to_string(),
        });

        // Add resource preparation if needed
        if self.requires_resource_preparation(intent) {
            steps.push(PlanStep {
                id: Uuid::new_v4(),
                command: "prepare_resources".to_string(),
                parameters: HashMap::new(),
                preconditions: vec![
                    Condition {
                        description: "Command validated and safe to execute".to_string(),
                        condition_type: ConditionType::Custom,
                        expected: serde_json::Value::Bool(true),
                    }
                ],
                postconditions: vec![
                    Condition {
                        description: "Resources prepared for execution".to_string(),
                        condition_type: ConditionType::ResourceAvailable,
                        expected: serde_json::Value::Bool(true),
                    }
                ],
                timeout: std::time::Duration::from_secs(30),
                description: "Prepare necessary resources for command execution".to_string(),
            });
        }

        // Add main execution step
        steps.push(PlanStep {
            id: Uuid::new_v4(),
            command: "execute_command".to_string(),
            parameters: intent.parameters.clone(),
            preconditions: if self.requires_resource_preparation(intent) {
                vec![
                    Condition {
                        description: "Resources prepared for execution".to_string(),
                        condition_type: ConditionType::ResourceAvailable,
                        expected: serde_json::Value::Bool(true),
                    }
                ]
            } else {
                vec![
                    Condition {
                        description: "Command validated and safe to execute".to_string(),
                        condition_type: ConditionType::Custom,
                        expected: serde_json::Value::Bool(true),
                    }
                ]
            },
            postconditions: Vec::new(),
            timeout: self.planning_context.default_timeout,
            description: "Execute the main command operation".to_string(),
        });

        Ok(steps)
    }

    /// Decompose configuration intent into steps
    async fn decompose_config_intent(&self, intent: &Intent) -> Result<Vec<PlanStep>, PlanningError> {
        let mut steps = Vec::new();

        // Backup current configuration
        steps.push(PlanStep {
            id: Uuid::new_v4(),
            command: "backup_config".to_string(),
            parameters: HashMap::new(),
            preconditions: Vec::new(),
            postconditions: vec![
                Condition {
                    description: "Configuration backed up".to_string(),
                    condition_type: ConditionType::FileExists,
                    expected: serde_json::Value::String("config_backup".to_string()),
                }
            ],
            timeout: std::time::Duration::from_secs(30),
            description: "Backup current configuration before changes".to_string(),
        });

        // Validate new configuration
        steps.push(PlanStep {
            id: Uuid::new_v4(),
            command: "validate_config".to_string(),
            parameters: intent.parameters.clone(),
            preconditions: vec![
                Condition {
                    description: "Configuration backed up".to_string(),
                    condition_type: ConditionType::FileExists,
                    expected: serde_json::Value::String("config_backup".to_string()),
                }
            ],
            postconditions: vec![
                Condition {
                    description: "New configuration validated".to_string(),
                    condition_type: ConditionType::Custom,
                    expected: serde_json::Value::Bool(true),
                }
            ],
            timeout: std::time::Duration::from_secs(20),
            description: "Validate new configuration parameters".to_string(),
        });

        // Apply configuration
        steps.push(PlanStep {
            id: Uuid::new_v4(),
            command: "apply_config".to_string(),
            parameters: intent.parameters.clone(),
            preconditions: vec![
                Condition {
                    description: "New configuration validated".to_string(),
                    condition_type: ConditionType::Custom,
                    expected: serde_json::Value::Bool(true),
                }
            ],
            postconditions: Vec::new(),
            timeout: std::time::Duration::from_secs(60),
            description: "Apply the new configuration".to_string(),
        });

        Ok(steps)
    }

    /// Decompose analysis intent into steps
    async fn decompose_analysis_intent(&self, intent: &Intent) -> Result<Vec<PlanStep>, PlanningError> {
        let mut steps = Vec::new();

        // Collect data for analysis
        steps.push(PlanStep {
            id: Uuid::new_v4(),
            command: "collect_analysis_data".to_string(),
            parameters: intent.parameters.clone(),
            preconditions: Vec::new(),
            postconditions: vec![
                Condition {
                    description: "Analysis data collected".to_string(),
                    condition_type: ConditionType::Custom,
                    expected: serde_json::Value::Bool(true),
                }
            ],
            timeout: std::time::Duration::from_secs(45),
            description: "Collect data required for analysis".to_string(),
        });

        // Perform analysis
        steps.push(PlanStep {
            id: Uuid::new_v4(),
            command: "perform_analysis".to_string(),
            parameters: intent.parameters.clone(),
            preconditions: vec![
                Condition {
                    description: "Analysis data collected".to_string(),
                    condition_type: ConditionType::Custom,
                    expected: serde_json::Value::Bool(true),
                }
            ],
            postconditions: Vec::new(),
            timeout: std::time::Duration::from_secs(120),
            description: "Perform the requested analysis".to_string(),
        });

        Ok(steps)
    }

    /// Decompose monitoring intent into steps
    async fn decompose_monitoring_intent(&self, intent: &Intent) -> Result<Vec<PlanStep>, PlanningError> {
        let mut steps = Vec::new();

        // Setup monitoring
        steps.push(PlanStep {
            id: Uuid::new_v4(),
            command: "setup_monitoring".to_string(),
            parameters: intent.parameters.clone(),
            preconditions: Vec::new(),
            postconditions: vec![
                Condition {
                    description: "Monitoring configured".to_string(),
                    condition_type: ConditionType::Custom,
                    expected: serde_json::Value::Bool(true),
                }
            ],
            timeout: std::time::Duration::from_secs(30),
            description: "Configure monitoring parameters".to_string(),
        });

        // Start monitoring
        steps.push(PlanStep {
            id: Uuid::new_v4(),
            command: "start_monitoring".to_string(),
            parameters: intent.parameters.clone(),
            preconditions: vec![
                Condition {
                    description: "Monitoring configured".to_string(),
                    condition_type: ConditionType::Custom,
                    expected: serde_json::Value::Bool(true),
                }
            ],
            postconditions: Vec::new(),
            timeout: std::time::Duration::from_secs(15),
            description: "Start the monitoring process".to_string(),
        });

        Ok(steps)
    }

    /// Decompose file operation intent into steps
    async fn decompose_file_operation_intent(&self, intent: &Intent) -> Result<Vec<PlanStep>, PlanningError> {
        let mut steps = Vec::new();

        // Validate file permissions
        steps.push(PlanStep {
            id: Uuid::new_v4(),
            command: "validate_file_permissions".to_string(),
            parameters: intent.parameters.clone(),
            preconditions: Vec::new(),
            postconditions: vec![
                Condition {
                    description: "File permissions validated".to_string(),
                    condition_type: ConditionType::Custom,
                    expected: serde_json::Value::Bool(true),
                }
            ],
            timeout: std::time::Duration::from_secs(10),
            description: "Validate file access permissions".to_string(),
        });

        // Perform file operation
        steps.push(PlanStep {
            id: Uuid::new_v4(),
            command: "file_operation".to_string(),
            parameters: intent.parameters.clone(),
            preconditions: vec![
                Condition {
                    description: "File permissions validated".to_string(),
                    condition_type: ConditionType::Custom,
                    expected: serde_json::Value::Bool(true),
                }
            ],
            postconditions: Vec::new(),
            timeout: self.planning_context.default_timeout,
            description: "Perform the requested file operation".to_string(),
        });

        Ok(steps)
    }

    /// Decompose process management intent into steps
    async fn decompose_process_management_intent(&self, intent: &Intent) -> Result<Vec<PlanStep>, PlanningError> {
        let mut steps = Vec::new();

        // Validate process permissions
        steps.push(PlanStep {
            id: Uuid::new_v4(),
            command: "validate_process_permissions".to_string(),
            parameters: intent.parameters.clone(),
            preconditions: Vec::new(),
            postconditions: vec![
                Condition {
                    description: "Process permissions validated".to_string(),
                    condition_type: ConditionType::Custom,
                    expected: serde_json::Value::Bool(true),
                }
            ],
            timeout: std::time::Duration::from_secs(10),
            description: "Validate process management permissions".to_string(),
        });

        // Perform process operation
        steps.push(PlanStep {
            id: Uuid::new_v4(),
            command: "process_management".to_string(),
            parameters: intent.parameters.clone(),
            preconditions: vec![
                Condition {
                    description: "Process permissions validated".to_string(),
                    condition_type: ConditionType::Custom,
                    expected: serde_json::Value::Bool(true),
                }
            ],
            postconditions: Vec::new(),
            timeout: self.planning_context.default_timeout,
            description: "Perform the requested process management operation".to_string(),
        });

        Ok(steps)
    }

    /// Check if intent requires resource preparation
    fn requires_resource_preparation(&self, intent: &Intent) -> bool {
        // Check if the intent involves operations that need resource preparation
        intent.raw_input.to_lowercase().contains("large") ||
        intent.raw_input.to_lowercase().contains("heavy") ||
        intent.parameters.len() > 5 ||
        intent.targets.len() > 3
    }

    /// Generate rollback plan for risky operations
    async fn generate_rollback_plan(&self, steps: &[PlanStep]) -> Result<RollbackPlan, PlanningError> {
        debug!("Generating rollback plan for {} steps", steps.len());

        let mut rollback_steps = Vec::new();

        // Generate rollback steps in reverse order
        for step in steps.iter().rev() {
            if let Some(rollback_step) = self.generate_rollback_step(step).await? {
                rollback_steps.push(rollback_step);
            }
        }

        Ok(RollbackPlan {
            steps: rollback_steps,
            description: "Rollback plan to undo changes if execution fails".to_string(),
        })
    }

    /// Generate rollback step for a given step
    async fn generate_rollback_step(&self, step: &PlanStep) -> Result<Option<PlanStep>, PlanningError> {
        match step.command.as_str() {
            "apply_config" => {
                Some(PlanStep {
                    id: Uuid::new_v4(),
                    command: "restore_config".to_string(),
                    parameters: HashMap::new(),
                    preconditions: Vec::new(),
                    postconditions: Vec::new(),
                    timeout: std::time::Duration::from_secs(30),
                    description: "Restore previous configuration".to_string(),
                })
            },
            "file_operation" => {
                Some(PlanStep {
                    id: Uuid::new_v4(),
                    command: "undo_file_operation".to_string(),
                    parameters: step.parameters.clone(),
                    preconditions: Vec::new(),
                    postconditions: Vec::new(),
                    timeout: std::time::Duration::from_secs(30),
                    description: "Undo file operation changes".to_string(),
                })
            },
            _ => None,
        }.map(Ok).transpose()
    }

    /// Calculate total execution time considering dependencies
    async fn calculate_total_execution_time(
        &self,
        steps: &[PlanStep],
        _dependencies: &[Dependency],
    ) -> Result<std::time::Duration, PlanningError> {
        // For now, use a simple approach: sum of all step timeouts
        // In a full implementation, this would consider parallel execution possibilities
        let total_timeout: std::time::Duration = steps.iter()
            .map(|step| step.timeout)
            .sum();

        // Add some buffer for coordination overhead
        let overhead = std::time::Duration::from_secs((steps.len() as u64) * 2);
        
        Ok(total_timeout + overhead)
    }

    /// Validate individual step
    async fn validate_step(&self, step: &PlanStep) -> Result<(), String> {
        // Basic validation
        if step.command.is_empty() {
            return Err("Step command cannot be empty".to_string());
        }

        if step.timeout.as_secs() == 0 {
            return Err("Step timeout must be greater than zero".to_string());
        }

        if step.description.is_empty() {
            return Err("Step description cannot be empty".to_string());
        }

        Ok(())
    }

    /// Generate file operation alternatives
    async fn generate_file_operation_alternatives(&self, _step: &PlanStep) -> Result<Vec<ExecutionPlan>, PlanningError> {
        // Placeholder for file operation alternatives
        Ok(Vec::new())
    }

    /// Generate process alternatives
    async fn generate_process_alternatives(&self, _step: &PlanStep) -> Result<Vec<ExecutionPlan>, PlanningError> {
        // Placeholder for process alternatives
        Ok(Vec::new())
    }

    /// Generate query alternatives
    async fn generate_query_alternatives(&self, _step: &PlanStep) -> Result<Vec<ExecutionPlan>, PlanningError> {
        // Placeholder for query alternatives
        Ok(Vec::new())
    }

    /// Generate generic alternatives
    async fn generate_generic_alternatives(&self, _step: &PlanStep) -> Result<Vec<ExecutionPlan>, PlanningError> {
        // Placeholder for generic alternatives
        Ok(Vec::new())
    }
}

/// Validation result for execution plans
#[derive(Debug, Clone)]
pub struct ValidationResult {
    /// Whether the plan is valid
    pub is_valid: bool,
    /// Validation errors
    pub errors: Vec<String>,
    /// Validation warnings
    pub warnings: Vec<String>,
}

/// Resource estimate for execution plans
#[derive(Debug, Clone)]
pub struct ResourceEstimate {
    /// CPU usage estimate
    pub cpu_usage: CpuUsage,
    /// Memory usage estimate
    pub memory_usage: MemoryUsage,
    /// Disk I/O estimate
    pub disk_io: DiskIO,
    /// Network I/O estimate
    pub network_io: NetworkIO,
    /// Estimated execution duration
    pub estimated_duration: std::time::Duration,
    /// Confidence interval for duration
    pub confidence_interval: (std::time::Duration, std::time::Duration),
}

/// CPU usage information
#[derive(Debug, Clone)]
pub struct CpuUsage {
    /// Percentage of CPU (0.0 to 1.0)
    pub percentage: f32,
    /// Number of cores needed
    pub cores_needed: u32,
}

/// Memory usage information
#[derive(Debug, Clone)]
pub struct MemoryUsage {
    /// Peak memory usage in bytes
    pub peak_bytes: u64,
    /// Average memory usage in bytes
    pub average_bytes: u64,
}

/// Disk I/O information
#[derive(Debug, Clone)]
pub struct DiskIO {
    /// Read operations per second
    pub read_ops_per_sec: u32,
    /// Write operations per second
    pub write_ops_per_sec: u32,
    /// Total bytes read
    pub total_read_bytes: u64,
    /// Total bytes written
    pub total_write_bytes: u64,
}

/// Network I/O information
#[derive(Debug, Clone)]
pub struct NetworkIO {
    /// Bytes per second incoming
    pub incoming_bps: u64,
    /// Bytes per second outgoing
    pub outgoing_bps: u64,
    /// Total bytes transferred
    pub total_bytes: u64,
}

impl Default for EnhancedAIPlanner {
    fn default() -> Self {
        Self::new()
    }
}