//! Developer mode implementation for semantic CLI
//!
//! Provides plan generation without execution, manual overrides,
//! and safe testing capabilities for semantic operations.

use crate::types::*;
use crate::error::*;
use super::DebugError;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

/// Developer mode controller that manages plan generation without execution
pub struct DeveloperController {
    /// Current developer session state
    session_state: DeveloperSessionState,
    /// Plan generation history
    plan_history: Vec<GeneratedPlan>,
    /// Manual overrides for testing
    manual_overrides: HashMap<String, serde_json::Value>,
    /// Safety checks enabled
    safety_checks_enabled: bool,
}

/// State for developer mode session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeveloperSessionState {
    /// Session ID
    pub session_id: Uuid,
    /// Whether plan generation is enabled
    pub plan_generation_enabled: bool,
    /// Whether execution is blocked
    pub execution_blocked: bool,
    /// Current test scenario
    pub test_scenario: Option<String>,
    /// Session start time
    pub start_time: DateTime<Utc>,
}

/// Generated plan with metadata for developer mode
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedPlan {
    /// Plan ID
    pub plan_id: PlanId,
    /// Associated intent ID
    pub intent_id: IntentId,
    /// The generated execution plan
    pub plan: ExecutionPlan,
    /// Generation timestamp
    pub generated_at: DateTime<Utc>,
    /// Generation time
    pub generation_time: Duration,
    /// Whether this was a test generation
    pub is_test: bool,
    /// Developer notes
    pub notes: Option<String>,
}

/// Manual override for testing specific scenarios
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManualOverride {
    /// Override key/identifier
    pub key: String,
    /// Override value
    pub value: serde_json::Value,
    /// Description of what this override does
    pub description: String,
    /// Whether this override is active
    pub active: bool,
}

impl DeveloperController {
    /// Create a new developer controller
    pub fn new() -> Self {
        Self {
            session_state: DeveloperSessionState {
                session_id: Uuid::new_v4(),
                plan_generation_enabled: true,
                execution_blocked: true, // Safety: block execution by default
                test_scenario: None,
                start_time: Utc::now(),
            },
            plan_history: Vec::new(),
            manual_overrides: HashMap::new(),
            safety_checks_enabled: true,
        }
    }

    /// Generate execution plan without executing it (core developer mode feature)
    pub async fn generate_plan_without_execution(&mut self, intent: &Intent) -> Result<ExecutionPlan, DebugError> {
        info!("Developer mode: Generating plan for intent {} without execution", intent.id);
        
        // Safety check: ensure execution is blocked
        if !self.session_state.execution_blocked {
            warn!("Developer mode safety violation: execution not blocked");
            return Err(DebugError::DeveloperModeError(
                "Execution must be blocked in developer mode".to_string()
            ));
        }

        let start_time = Instant::now();
        
        // Generate plan using mock planner (safe simulation)
        let plan = self.simulate_plan_generation(intent).await?;
        
        let generation_time = start_time.elapsed();
        
        // Store in history (move plan instead of cloning)
        let generated_plan = GeneratedPlan {
            plan_id: plan.id,
            intent_id: intent.id,
            plan: plan, // Move instead of clone
            generated_at: Utc::now(),
            generation_time,
            is_test: self.session_state.test_scenario.is_some(),
            notes: None,
        };
        
        // Get reference to the plan for return value (avoid clone)
        let plan_ref = &generated_plan.plan;
        self.plan_history.push(generated_plan);
        
        debug!("Plan generated in {}ms without execution", generation_time.as_millis());
        
        // Return a new plan with same data (single clone instead of storing clone)
        Ok(ExecutionPlan {
            id: plan_ref.id,
            intent_id: plan_ref.intent_id,
            steps: plan_ref.steps.clone(), // Only necessary clone
            dependencies: plan_ref.dependencies.clone(), // Only necessary clone
            risk_assessment: plan_ref.risk_assessment.clone(), // Only necessary clone
            created_at: plan_ref.created_at,
            estimated_duration: plan_ref.estimated_duration,
        })
    }

    /// Simulate plan generation safely (no system state changes)
    /// 
    /// **Optimization:** Use parameter references instead of cloning where possible
    async fn simulate_plan_generation(&self, intent: &Intent) -> Result<ExecutionPlan, DebugError> {
        // Create a safe simulation of plan generation
        let mut plan = ExecutionPlan::new(intent.id);
        
        // Pre-allocate parameters to avoid multiple clones
        let parameters = &intent.parameters; // Use reference
        let raw_input = &intent.raw_input;   // Use reference
        
        // Generate steps based on intent action type
        match intent.action {
            ActionType::Query => {
                plan.steps.push(PlanStep {
                    id: Uuid::new_v4(),
                    command: "query".to_string(),
                    parameters: parameters.clone(), // Single clone per step
                    preconditions: vec![],
                    postconditions: vec![],
                    timeout: Duration::from_secs(30),
                    description: format!("Query operation for: {}", raw_input),
                });
            },
            ActionType::Command => {
                plan.steps.push(PlanStep {
                    id: Uuid::new_v4(),
                    command: "execute".to_string(),
                    parameters: parameters.clone(), // Single clone per step
                    preconditions: vec![],
                    postconditions: vec![],
                    timeout: Duration::from_secs(60),
                    description: format!("Command execution for: {}", raw_input),
                });
            },
            ActionType::FileOperation => {
                plan.steps.push(PlanStep {
                    id: Uuid::new_v4(),
                    command: "file_op".to_string(),
                    parameters: parameters.clone(), // Single clone per step
                    preconditions: vec![Condition {
                        description: "File system access available".to_string(),
                        condition_type: ConditionType::ResourceAvailable,
                        expected: serde_json::Value::Bool(true),
                    }],
                    postconditions: vec![],
                    timeout: Duration::from_secs(30),
                    description: format!("File operation for: {}", raw_input),
                });
            },
            _ => {
                plan.steps.push(PlanStep {
                    id: Uuid::new_v4(),
                    command: "generic".to_string(),
                    parameters: parameters.clone(), // Single clone per step
                    preconditions: vec![],
                    postconditions: vec![],
                    timeout: Duration::from_secs(30),
                    description: format!("Generic operation for: {}", intent.raw_input),
                });
            }
        }
        
        // Apply manual overrides after steps are created
        self.apply_manual_overrides(&mut plan)?;
        
        // Set safe resource requirements (no actual resources consumed)
        plan.resource_requirements = ResourceRequirements {
            cpu_usage: 0.0, // No actual CPU usage in simulation
            memory_usage: 0, // No actual memory usage
            disk_space: 0,   // No actual disk usage
            network_bandwidth: 0, // No actual network usage
            exclusive_resources: vec![], // No exclusive resources needed
        };
        
        // Mark as low risk since it's simulation only
        plan.risk_assessment = RiskAssessment {
            risk_level: RiskLevel::Low,
            potential_impacts: vec![],
            mitigation_strategies: vec![],
            approval_required: false, // No approval needed for simulation
        };
        
        Ok(plan)
    }

    /// Apply manual overrides for testing scenarios
    fn apply_manual_overrides(&self, plan: &mut ExecutionPlan) -> Result<(), DebugError> {
        for (key, value) in &self.manual_overrides {
            match key.as_str() {
                "force_timeout" => {
                    if let Some(timeout_secs) = value.as_u64() {
                        for step in &mut plan.steps {
                            step.timeout = Duration::from_secs(timeout_secs);
                        }
                    }
                },
                "add_precondition" => {
                    if let Some(condition_desc) = value.as_str() {
                        let condition = Condition {
                            description: condition_desc.to_string(),
                            condition_type: ConditionType::Custom,
                            expected: serde_json::Value::Bool(true),
                        };
                        for step in &mut plan.steps {
                            step.preconditions.push(condition.clone());
                        }
                    }
                },
                "set_risk_level" => {
                    if let Some(risk_str) = value.as_str() {
                        let risk_level = match risk_str {
                            "low" => RiskLevel::Low,
                            "medium" => RiskLevel::Medium,
                            "high" => RiskLevel::High,
                            "critical" => RiskLevel::Critical,
                            _ => RiskLevel::Low,
                        };
                        plan.risk_assessment.risk_level = risk_level;
                    }
                },
                _ => {
                    debug!("Unknown manual override: {}", key);
                }
            }
        }
        Ok(())
    }

    /// Add manual override for testing
    /// 
    /// **Optimization:** Move values instead of cloning when possible
    pub fn add_manual_override(&mut self, key: String, value: serde_json::Value, description: String) {
        info!("Adding manual override: {} = {:?}", key, value);
        // Move key and value instead of cloning
        self.manual_overrides.insert(key, value);
    }

    /// Remove manual override
    pub fn remove_manual_override(&mut self, key: &str) -> bool {
        self.manual_overrides.remove(key).is_some()
    }

    /// Get all active manual overrides
    pub fn get_manual_overrides(&self) -> Vec<ManualOverride> {
        self.manual_overrides
            .iter()
            .map(|(key, value)| ManualOverride {
                key: key.clone(),
                value: value.clone(),
                description: format!("Manual override for {}", key),
                active: true,
            })
            .collect()
    }

    /// Set test scenario
    pub fn set_test_scenario(&mut self, scenario: Option<String>) {
        self.session_state.test_scenario = scenario;
        if let Some(ref scenario) = self.session_state.test_scenario {
            info!("Developer mode: Set test scenario to '{}'", scenario);
        } else {
            info!("Developer mode: Cleared test scenario");
        }
    }

    /// Get plan generation history
    pub fn get_plan_history(&self) -> &[GeneratedPlan] {
        &self.plan_history
    }

    /// Get specific generated plan by ID
    pub fn get_generated_plan(&self, plan_id: PlanId) -> Option<&GeneratedPlan> {
        self.plan_history.iter().find(|p| p.plan_id == plan_id)
    }

    /// Clear plan history (for memory management)
    pub fn clear_plan_history(&mut self) {
        info!("Developer mode: Clearing plan history ({} plans)", self.plan_history.len());
        self.plan_history.clear();
    }

    /// Ensure execution is blocked (safety check)
    pub fn ensure_execution_blocked(&mut self) -> Result<(), DebugError> {
        if !self.session_state.execution_blocked {
            self.session_state.execution_blocked = true;
            warn!("Developer mode: Execution was not blocked, now blocking for safety");
        }
        Ok(())
    }

    /// Check if execution is safely blocked
    pub fn is_execution_blocked(&self) -> bool {
        self.session_state.execution_blocked
    }

    /// Get developer session state
    pub fn get_session_state(&self) -> &DeveloperSessionState {
        &self.session_state
    }

    /// Generate developer mode summary
    pub fn generate_summary(&self) -> DeveloperModeSummary {
        DeveloperModeSummary {
            session_id: self.session_state.session_id,
            plans_generated: self.plan_history.len(),
            test_scenario: self.session_state.test_scenario.clone(),
            manual_overrides_count: self.manual_overrides.len(),
            execution_blocked: self.session_state.execution_blocked,
            safety_checks_enabled: self.safety_checks_enabled,
            session_duration: Utc::now().signed_duration_since(self.session_state.start_time),
        }
    }
}

/// Summary of developer mode session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeveloperModeSummary {
    /// Session ID
    pub session_id: Uuid,
    /// Number of plans generated
    pub plans_generated: usize,
    /// Current test scenario
    pub test_scenario: Option<String>,
    /// Number of manual overrides
    pub manual_overrides_count: usize,
    /// Whether execution is blocked
    pub execution_blocked: bool,
    /// Whether safety checks are enabled
    pub safety_checks_enabled: bool,
    /// Session duration
    pub session_duration: chrono::Duration,
}

impl Default for DeveloperController {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_developer_controller_creation() {
        let controller = DeveloperController::new();
        assert!(controller.session_state.execution_blocked);
        assert!(controller.session_state.plan_generation_enabled);
        assert!(controller.safety_checks_enabled);
    }

    #[tokio::test]
    async fn test_plan_generation_without_execution() {
        let mut controller = DeveloperController::new();
        let intent = Intent::new("test command".to_string(), ActionType::Command);
        
        let result = controller.generate_plan_without_execution(&intent).await;
        assert!(result.is_ok());
        
        let plan = result.unwrap();
        assert_eq!(plan.intent_id, intent.id);
        assert!(!plan.steps.is_empty());
        
        // Verify plan was stored in history
        assert_eq!(controller.plan_history.len(), 1);
        assert_eq!(controller.plan_history[0].intent_id, intent.id);
    }

    #[tokio::test]
    async fn test_execution_blocked_safety() {
        let mut controller = DeveloperController::new();
        
        // Execution should be blocked by default
        assert!(controller.is_execution_blocked());
        
        // Ensure it stays blocked
        controller.ensure_execution_blocked().unwrap();
        assert!(controller.is_execution_blocked());
    }

    #[test]
    fn test_manual_overrides() {
        let mut controller = DeveloperController::new();
        
        // Add override
        controller.add_manual_override(
            "test_key".to_string(),
            serde_json::Value::String("test_value".to_string()),
            "Test override".to_string()
        );
        
        let overrides = controller.get_manual_overrides();
        assert_eq!(overrides.len(), 1);
        assert_eq!(overrides[0].key, "test_key");
        
        // Remove override
        assert!(controller.remove_manual_override("test_key"));
        assert_eq!(controller.get_manual_overrides().len(), 0);
    }

    #[test]
    fn test_test_scenario_management() {
        let mut controller = DeveloperController::new();
        
        // Initially no scenario
        assert!(controller.session_state.test_scenario.is_none());
        
        // Set scenario
        controller.set_test_scenario(Some("test_scenario_1".to_string()));
        assert_eq!(controller.session_state.test_scenario, Some("test_scenario_1".to_string()));
        
        // Clear scenario
        controller.set_test_scenario(None);
        assert!(controller.session_state.test_scenario.is_none());
    }

    #[tokio::test]
    async fn test_plan_generation_with_overrides() {
        let mut controller = DeveloperController::new();
        
        // Add timeout override
        controller.add_manual_override(
            "force_timeout".to_string(),
            serde_json::Value::Number(serde_json::Number::from(120)),
            "Force 120 second timeout".to_string()
        );
        
        let intent = Intent::new("test command".to_string(), ActionType::Command);
        let plan = controller.generate_plan_without_execution(&intent).await.unwrap();
        
        // Check that timeout was applied
        for step in &plan.steps {
            assert_eq!(step.timeout, Duration::from_secs(120));
        }
    }

    #[test]
    fn test_developer_mode_summary() {
        let controller = DeveloperController::new();
        let summary = controller.generate_summary();
        
        assert_eq!(summary.plans_generated, 0);
        assert!(summary.execution_blocked);
        assert!(summary.safety_checks_enabled);
        assert_eq!(summary.manual_overrides_count, 0);
    }
}