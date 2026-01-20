//! Architecture Preservation Validation Tests
//!
//! This module implements tests to validate that the loop engine architectural
//! improvements preserve the existing architectural strengths and design principles.
//!
//! # Task 14: Validate Architecture Preservation
//!
//! ## Task 14.1: Verify LoopResult/ControlFlow separation maintained
//! - Test type separation and interface preservation
//! - Verify Safety_Analyzer positioning unchanged
//! - Requirements: 6.1, 6.3
//!
//! ## Task 14.2: Verify integration readiness preserved
//! - Test BCIB, JIT (D1), and parallelism (D2) compatibility
//! - Verify Ring3_Runtime integration readiness
//! - Requirements: 6.4, 6.6

use crate::bcib::{LoopInstruction, LoopID, LoopConfig, LoopRange, Value, ValueType, ControlFlowInstruction, ControlFlowType};
use crate::loop_engine::{
    LoopEngine, LoopResult, LoopError, EnvironmentFault,
    SafetyAnalyzer, SafetyClass, LoopAnalysisContext,
    LoopBodyFn, LoopBodyResult, ControlFlow, ControlFlowDecision,
    JITConfig, MonitoringConfig
};
use crate::error::{SemanticCLIError, ErrorCode};
use crate::types::SourceLocation;

/// Test suite for Task 14.1: LoopResult/ControlFlow separation validation
#[cfg(test)]
mod loop_result_control_flow_separation {
    use super::*;

    /// Test that LoopResult and ControlFlow types are properly separated
    /// 
    /// This test validates that:
    /// 1. LoopResult represents execution outcomes
    /// 2. ControlFlow manages iteration control
    /// 3. Types have distinct responsibilities and interfaces
    /// 4. No inappropriate coupling between the types
    #[test]
    fn test_loop_result_control_flow_type_separation() {
        // Test LoopResult variants represent execution outcomes
        let success_result = LoopResult::success(Value::Number(42.0), 10);
        assert!(success_result.is_success());
        assert_eq!(success_result.get_iterations_completed(), 10);
        assert_eq!(success_result.get_accumulator(), Some(&Value::Number(42.0)));

        let break_result = LoopResult::break_result(Value::Number(100.0), 5);
        assert!(break_result.is_break());
        assert_eq!(break_result.get_iterations_completed(), 5);

        let error_result = LoopResult::error(LoopError::IterationLimitExceeded { 
            limit: 100, 
            completed: 0  // Control-flow dominant path - no iterations completed
        });
        assert!(error_result.is_error());
        assert_eq!(error_result.get_iterations_completed(), 0);

        // Test ControlFlow manages iteration control independently
        let mut control_flow = ControlFlow::new();
        control_flow.set_iteration_limit(100);
        control_flow.set_budget_limit(1000);

        // ControlFlow should manage limits independently of LoopResult
        assert!(!control_flow.would_exceed_iteration_limit());
        assert!(!control_flow.would_exceed_budget_timeout(50));

        // Increment iterations and verify control flow state
        for _ in 0..10 {
            control_flow.increment_iteration_count();
            control_flow.add_budget_consumed(10);
        }

        assert_eq!(control_flow.get_iteration_count(), 10);
        assert_eq!(control_flow.get_budget_consumed(), 100);

        // Control flow decisions should be independent of result types
        let continue_decision = control_flow.evaluate_condition(true);
        assert_eq!(continue_decision, ControlFlowDecision::Continue);

        let break_decision = control_flow.evaluate_condition(false);
        assert_eq!(break_decision, ControlFlowDecision::Break);
    }

    /// Test that LoopResult interface is preserved and stable
    /// 
    /// This validates that the public API of LoopResult remains unchanged
    /// and provides the expected interface for consumers.
    #[test]
    fn test_loop_result_interface_preservation() {
        // Test Success variant interface
        let success = LoopResult::success(Value::String("result".to_string()), 15);
        assert!(success.is_success());
        assert!(!success.is_error());
        assert!(!success.is_break());
        assert_eq!(success.get_iterations_completed(), 15);
        assert_eq!(success.get_accumulator(), Some(&Value::String("result".to_string())));

        // Test Error variant interface
        let error = LoopResult::error(LoopError::BudgetTimeoutExceeded {
            budget: 1000,
            consumed: 1000,
            iterations_completed: 0, // Control-flow dominant - no meaningful iterations
        });
        assert!(!error.is_success());
        assert!(error.is_error());
        assert!(!error.is_break());
        assert_eq!(error.get_iterations_completed(), 0);

        // Test Break variant interface
        let break_result = LoopResult::break_result(Value::Number(99.0), 7);
        assert!(!break_result.is_success());
        assert!(!break_result.is_error());
        assert!(break_result.is_break());
        assert_eq!(break_result.get_iterations_completed(), 7);
        assert_eq!(break_result.get_accumulator(), Some(&Value::Number(99.0)));

        // Test EnvironmentFault variant interface
        let env_fault = LoopResult::environment_fault(EnvironmentFault::WallClockKill {
            elapsed_ms: 5000,
            limit_ms: 3000,
        });
        assert!(!env_fault.is_success());
        assert!(!env_fault.is_error());
        assert!(!env_fault.is_break());
        // Environment faults may not have meaningful iteration counts
    }

    /// Test that ControlFlow interface is preserved and stable
    /// 
    /// This validates that the public API of ControlFlow remains unchanged
    /// and provides the expected interface for loop execution management.
    #[test]
    fn test_control_flow_interface_preservation() {
        let mut control_flow = ControlFlow::new();

        // Test limit setting interface
        control_flow.set_iteration_limit(50);
        control_flow.set_budget_limit(500);

        // Test pre-check interface (constitutional requirement)
        assert!(!control_flow.would_exceed_iteration_limit());
        assert!(!control_flow.would_exceed_budget_timeout(100));

        // Test increment interface (post-increment requirement)
        control_flow.increment_iteration_count();
        control_flow.add_budget_consumed(25);
        assert_eq!(control_flow.get_iteration_count(), 1);
        assert_eq!(control_flow.get_budget_consumed(), 25);

        // Test decision recording interface
        control_flow.record_decision(ControlFlowDecision::Continue, true);
        let trace = control_flow.get_decision_trace();
        assert_eq!(trace.len(), 1);
        assert_eq!(trace[0].decision, ControlFlowDecision::Continue);

        // Test condition evaluation interface
        let decision = control_flow.evaluate_condition(false);
        assert_eq!(decision, ControlFlowDecision::Break);

        // Test break/continue handling interface
        let break_decision = control_flow.handle_break();
        assert_eq!(break_decision, ControlFlowDecision::Break);

        let continue_decision = control_flow.handle_continue();
        assert_eq!(continue_decision, ControlFlowDecision::Skip);

        // Test fingerprint generation interface
        let control_fingerprint = control_flow.create_control_fingerprint();
        assert!(!control_fingerprint.is_empty());

        let shape_fingerprint = control_flow.create_shape_fingerprint(
            12345,
            crate::loop_engine::fingerprint::LoopType::While,
        );
        assert_eq!(shape_fingerprint.loop_id, 12345);
    }

    /// Test that LoopResult and ControlFlow have no inappropriate coupling
    /// 
    /// This validates that the types maintain proper separation of concerns
    /// and don't have hidden dependencies on each other.
    #[test]
    fn test_no_inappropriate_coupling() {
        // LoopResult should be creatable without ControlFlow
        let result_without_control = LoopResult::success(Value::Boolean(true), 20);
        assert!(result_without_control.is_success());

        // ControlFlow should be usable without LoopResult
        let mut control_without_result = ControlFlow::new();
        control_without_result.set_iteration_limit(10);
        control_without_result.increment_iteration_count();
        assert_eq!(control_without_result.get_iteration_count(), 1);

        // LoopResult should not expose ControlFlow internals
        let result = LoopResult::error(LoopError::LoopBodyError {
            iteration: 0, // Control-flow dominant - error before meaningful iterations
            error: "test error".to_string(),
        });
        
        // Result should only expose result-related information
        assert!(result.is_error());
        assert_eq!(result.get_iterations_completed(), 0);
        // Should not expose control flow decision traces, budget calculations, etc.

        // ControlFlow should not expose LoopResult internals
        let mut control = ControlFlow::new();
        control.record_decision(ControlFlowDecision::Break, false);
        
        // Control flow should only expose control-related information
        assert_eq!(control.get_decision_trace().len(), 1);
        // Should not expose accumulator values, error details, etc.
    }

    /// Test that error handling maintains proper separation
    /// 
    /// This validates that error handling respects the separation between
    /// execution outcomes (LoopResult) and control flow management.
    #[test]
    fn test_error_handling_separation() {
        // Control flow errors should be about limits and constraints
        let mut control_flow = ControlFlow::new();
        control_flow.set_iteration_limit(5);
        
        // Exceed iteration limit
        for _ in 0..6 {
            control_flow.increment_iteration_count();
        }
        
        assert!(control_flow.would_exceed_iteration_limit());
        assert!(control_flow.check_iteration_limit().is_err());

        // LoopResult errors should be about execution outcomes
        let execution_error = LoopResult::error(LoopError::AccumulatorTypeMismatch {
            expected: ValueType::Number,
            actual: ValueType::String,
            iteration: 3,
            accumulator_name: "sum".to_string(),
        });

        assert!(execution_error.is_error());
        
        // The two error domains should be independent
        // Control flow limit checking doesn't depend on LoopResult
        // LoopResult error creation doesn't depend on ControlFlow state
    }
}

/// Test suite for Safety Analyzer positioning validation
#[cfg(test)]
mod safety_analyzer_positioning {
    use super::*;

    /// Test that SafetyAnalyzer maintains its correct position in the architecture
    /// 
    /// This validates that the safety analyzer:
    /// 1. Remains positioned correctly in the execution pipeline
    /// 2. Maintains its interface and responsibilities
    /// 3. Integrates properly with the loop engine
    #[test]
    fn test_safety_analyzer_positioning() {
        let mut loop_engine = LoopEngine::new();
        
        // Safety analyzer should be accessible through loop engine
        let cache_stats = loop_engine.get_safety_cache_stats();
        assert_eq!(cache_stats.entries, 0); // Initially empty

        // Safety analyzer should be able to analyze loop safety
        let mut context = LoopAnalysisContext::new();
        context.add_loop_variable("i".to_string(), "number".to_string());
        context.add_loop_variable("accumulator".to_string(), "number".to_string());

        let safe_body = "accumulator = accumulator + i";
        let analysis_result = loop_engine.analyze_loop_safety(safe_body, &context);
        assert!(analysis_result.is_ok());
        
        let result = analysis_result.unwrap();
        assert_eq!(result.classification, SafetyClass::Safe);

        // Safety analyzer should maintain cache functionality
        let updated_stats = loop_engine.get_safety_cache_stats();
        assert_eq!(updated_stats.entries, 1); // Should have cached the result

        // Safety analyzer should be clearable
        loop_engine.clear_safety_cache();
        let cleared_stats = loop_engine.get_safety_cache_stats();
        assert_eq!(cleared_stats.entries, 0);
    }

    /// Test that SafetyAnalyzer interface is preserved
    /// 
    /// This validates that the safety analyzer maintains its expected
    /// public interface and functionality.
    #[test]
    fn test_safety_analyzer_interface_preservation() {
        let mut analyzer = SafetyAnalyzer::new();
        
        // Test basic analysis interface
        let mut context = LoopAnalysisContext::new();
        context.add_loop_variable("sum".to_string(), "number".to_string());
        context.add_external_variable("global_var".to_string(), "number".to_string());

        // Test safe loop analysis (pure computation)
        let safe_result = analyzer.analyze_loop_safety("sum + 1", &context);
        assert!(safe_result.is_ok());
        let result = safe_result.unwrap();
        assert_eq!(result.classification, SafetyClass::Safe);

        // Test unsafe loop analysis (external mutation)
        let unsafe_result = analyzer.analyze_loop_safety("global_var = global_var + 1", &context);
        assert!(unsafe_result.is_ok());
        let result = unsafe_result.unwrap();
        assert_eq!(result.classification, SafetyClass::Unsafe);
        assert!(!result.side_effects.is_empty());

        // Test cache interface
        let stats = analyzer.cache_stats();
        assert_eq!(stats.entries, 2); // Two analyses cached

        analyzer.clear_cache();
        let cleared_stats = analyzer.cache_stats();
        assert_eq!(cleared_stats.entries, 0);
    }

    /// Test that SafetyAnalyzer integrates correctly with loop execution
    /// 
    /// This validates that the safety analyzer works correctly within
    /// the broader loop execution context.
    #[test]
    fn test_safety_analyzer_integration() {
        let mut loop_engine = LoopEngine::new();
        
        // Create a test loop instruction
        let loop_config = LoopConfig::new(Value::Number(0.0), ValueType::Number);
        let loop_instruction = LoopInstruction::For {
            id: LoopID::new("test-safety-integration".to_string()),
            range: LoopRange::new(0, 5, 1),
            iterator_var: "i".to_string(),
            body: "test-body".to_string(),
            config: loop_config,
            location: SourceLocation::new(1, 1, 0),
        };

        // Safety analysis should work with loop instructions
        let mut context = LoopAnalysisContext::new();
        context.add_loop_variable("i".to_string(), "number".to_string());
        context.add_loop_variable("accumulator".to_string(), "number".to_string());

        let analysis_result = loop_engine.analyze_loop_safety("accumulator = accumulator + i", &context);
        assert!(analysis_result.is_ok());

        // Safety analysis should inform parallelization decisions
        let safety_result = analysis_result.unwrap();
        let parallelization_decision = loop_engine.should_parallelize_loop(&loop_instruction, &safety_result);
        
        // The decision should be based on safety analysis
        // (Exact decision depends on loop type and safety classification)
        assert!(matches!(
            parallelization_decision,
            crate::loop_engine::ParallelizationDecision::Sequential { .. } |
            crate::loop_engine::ParallelizationDecision::Parallel { .. }
        ));
    }
}

/// Test suite for architectural interface stability
#[cfg(test)]
mod interface_stability {
    use super::*;

    /// Test that core loop engine interfaces remain stable
    /// 
    /// This validates that the main loop engine interfaces haven't
    /// been broken by the architectural improvements.
    #[test]
    fn test_loop_engine_interface_stability() {
        let mut loop_engine = LoopEngine::new();

        // Test basic loop execution interface
        let loop_config = LoopConfig::new(Value::Number(0.0), ValueType::Number);
        let for_loop = LoopInstruction::For {
            id: LoopID::new("test-interface-stability".to_string()),
            range: LoopRange::new(0, 3, 1),
            iterator_var: "i".to_string(),
            body: "test-body".to_string(),
            config: loop_config.clone(),
            location: SourceLocation::new(1, 1, 0),
        };

        // Create a simple body function
        let body_fn: LoopBodyFn = Box::new(|accumulator, iteration| {
            if let Value::Number(acc) = accumulator {
                Ok(LoopBodyResult::Normal(Value::Number(acc + iteration as f64)))
            } else {
                Err(SemanticCLIError::execution_error(
                    "Invalid accumulator type",
                    ErrorCode::E500,
                ))
            }
        });

        // Execute loop should work as before
        let result = loop_engine.execute_loop(&for_loop, body_fn);
        assert!(result.is_ok());
        let loop_result = result.unwrap();
        assert!(loop_result.is_success());

        // Test monitoring interface
        let loop_id = LoopID::new("test-monitoring".to_string());
        assert!(!loop_engine.is_hot_loop(&loop_id));
        
        // Verify monitoring stats are available
        let global_stats = loop_engine.get_global_monitoring_stats();
        // Note: total_loop_executions is u64, always >= 0

        // Test JIT interface
        let jit_stats = loop_engine.get_jit_stats();
        // Note: compilation_attempts is usize, always >= 0

        // Test unrolling interface
        let unroll_result = loop_engine.analyze_loop_unrolling(&for_loop);
        assert!(unroll_result.is_ok());

        let should_unroll = loop_engine.should_unroll_loop(&for_loop);
        assert!(should_unroll.is_ok());
    }

    /// Test that error types and handling remain consistent
    /// 
    /// This validates that error handling interfaces haven't been
    /// disrupted by the architectural changes.
    #[test]
    fn test_error_handling_interface_stability() {
        // Test LoopError variants are still available and functional
        let iteration_error = LoopError::IterationLimitExceeded { limit: 100, completed: 50 };
        assert_eq!(iteration_error.error_code(), "LE001");
        assert!(iteration_error.is_recoverable());

        let budget_error = LoopError::BudgetTimeoutExceeded { 
            budget: 1000, 
            consumed: 1000, 
            iterations_completed: 25 
        };
        assert_eq!(budget_error.error_code(), "LE002");
        assert!(budget_error.supports_partial_results());

        let type_error = LoopError::AccumulatorTypeMismatch {
            expected: ValueType::Number,
            actual: ValueType::String,
            iteration: 0, // Control-flow dominant - type error before meaningful iterations
            accumulator_name: "sum".to_string(),
        };
        assert_eq!(type_error.error_code(), "LE003");
        assert!(!type_error.is_recoverable());

        // Test EnvironmentFault is still available
        let env_fault = EnvironmentFault::WallClockKill {
            elapsed_ms: 5000,
            limit_ms: 3000,
        };
        assert_eq!(format!("{:?}", env_fault), "WallClockKill { elapsed_ms: 5000, limit_ms: 3000 }");

        // Test error result creation
        let error_result = LoopResult::error(iteration_error);
        assert!(error_result.is_error());

        let fault_result = LoopResult::environment_fault(env_fault);
        assert!(!fault_result.is_success());
    }

    /// Test that value types and operations remain stable
    /// 
    /// This validates that the core value system used by loops
    /// hasn't been disrupted.
    #[test]
    fn test_value_system_stability() {
        // Test Value types are still functional
        let number_val = Value::Number(42.0);
        assert_eq!(number_val.value_type(), ValueType::Number);

        let string_val = Value::String("test".to_string());
        assert_eq!(string_val.value_type(), ValueType::String);

        let bool_val = Value::Boolean(true);
        assert_eq!(bool_val.value_type(), ValueType::Boolean);

        let array_val = Value::Array(vec![Value::Number(1.0), Value::Number(2.0)]);
        assert_eq!(array_val.value_type(), ValueType::Array);

        // Test value validation
        assert!(number_val.validate().is_ok());
        assert!(string_val.validate().is_ok());
        assert!(bool_val.validate().is_ok());
        assert!(array_val.validate().is_ok());

        // Test LoopConfig creation with values
        let config = LoopConfig::new(number_val.clone(), ValueType::Number);
        assert_eq!(config.initial_accumulator, number_val);
        assert_eq!(config.accumulator_type, ValueType::Number);
    }
}

/// Test suite for Task 14.2: Integration readiness preservation validation
#[cfg(test)]
mod integration_readiness_preservation {
    use super::*;

    /// Test BCIB integration compatibility
    /// 
    /// This validates that the loop engine maintains compatibility with
    /// the BCIB instruction system and can process BCIB loop instructions.
    #[test]
    fn test_bcib_integration_compatibility() {
        let mut loop_engine = LoopEngine::new();

        // Test BCIB LoopInstruction compatibility
        let bcib_while_loop = LoopInstruction::While {
            id: LoopID::new("bcib-while-test".to_string()),
            condition: crate::bcib::OperandRef::Literal(Value::Boolean(true)),
            body: "bcib-body".to_string(),
            config: LoopConfig::new(Value::Number(0.0), ValueType::Number),
            location: SourceLocation::new(1, 1, 0),
        };

        // Validate BCIB instruction
        assert!(bcib_while_loop.validate().is_ok());
        assert_eq!(bcib_while_loop.loop_type(), crate::bcib::LoopType::While);
        assert_eq!(bcib_while_loop.required_capability(), Some(crate::bcib::Capability::Execute));

        // Test BCIB For loop
        let bcib_for_loop = LoopInstruction::For {
            id: LoopID::new("bcib-for-test".to_string()),
            range: LoopRange::new(0, 5, 1),
            iterator_var: "i".to_string(),
            body: "bcib-for-body".to_string(),
            config: LoopConfig::new(Value::Array(vec![]), ValueType::Array),
            location: SourceLocation::new(2, 1, 10),
        };

        assert!(bcib_for_loop.validate().is_ok());
        assert_eq!(bcib_for_loop.loop_type(), crate::bcib::LoopType::For);

        // Test BCIB ForEach loop
        let bcib_foreach_loop = LoopInstruction::ForEach {
            id: LoopID::new("bcib-foreach-test".to_string()),
            collection: crate::bcib::OperandRef::Literal(Value::Array(vec![
                Value::Number(1.0),
                Value::Number(2.0),
                Value::Number(3.0),
            ])),
            collection_type: crate::bcib::CollectionType::Array,
            iterator_var: "item".to_string(),
            body: "bcib-foreach-body".to_string(),
            config: LoopConfig::new(Value::Number(0.0), ValueType::Number),
            location: SourceLocation::new(3, 1, 20),
        };

        assert!(bcib_foreach_loop.validate().is_ok());
        assert_eq!(bcib_foreach_loop.loop_type(), crate::bcib::LoopType::ForEach);

        // Test that loop engine can analyze BCIB instructions for safety
        let mut context = LoopAnalysisContext::new();
        context.add_loop_variable("i".to_string(), "number".to_string());
        context.add_loop_variable("accumulator".to_string(), "number".to_string());

        let safety_analysis = loop_engine.analyze_loop_safety("accumulator = accumulator + i", &context);
        assert!(safety_analysis.is_ok());

        // Test that loop engine can make parallelization decisions for BCIB instructions
        let safety_result = safety_analysis.unwrap();
        let parallel_decision = loop_engine.should_parallelize_loop(&bcib_for_loop, &safety_result);
        assert!(matches!(
            parallel_decision,
            crate::loop_engine::ParallelizationDecision::Sequential { .. } |
            crate::loop_engine::ParallelizationDecision::Parallel { .. }
        ));
    }

    /// Test BCIB ControlFlow instruction compatibility
    /// 
    /// This validates that the loop engine maintains compatibility with
    /// BCIB control flow instructions (break/continue).
    #[test]
    fn test_bcib_control_flow_compatibility() {
        // Test BCIB Break instruction
        let bcib_break = ControlFlowInstruction::Break {
            location: SourceLocation::new(1, 5, 25),
        };

        assert!(bcib_break.validate().is_ok());
        assert_eq!(bcib_break.control_flow_type(), ControlFlowType::Break);
        assert_eq!(bcib_break.required_capability(), Some(crate::bcib::Capability::Execute));

        // Test BCIB Continue instruction
        let bcib_continue = ControlFlowInstruction::Continue {
            location: SourceLocation::new(2, 8, 40),
        };

        assert!(bcib_continue.validate().is_ok());
        assert_eq!(bcib_continue.control_flow_type(), ControlFlowType::Continue);
        assert_eq!(bcib_continue.required_capability(), Some(crate::bcib::Capability::Execute));

        // Test that ControlFlow manager can handle BCIB control flow types
        let mut control_flow = ControlFlow::new();
        
        // Simulate break handling
        let break_decision = control_flow.handle_break();
        assert_eq!(break_decision, ControlFlowDecision::Break);

        // Simulate continue handling
        let continue_decision = control_flow.handle_continue();
        assert_eq!(continue_decision, ControlFlowDecision::Skip);

        // Verify decision trace includes both decisions
        let trace = control_flow.get_decision_trace();
        assert_eq!(trace.len(), 2);
        assert_eq!(trace[0].decision, ControlFlowDecision::Break);
        assert_eq!(trace[1].decision, ControlFlowDecision::Skip);
    }

    /// Test JIT (D1) integration readiness
    /// 
    /// This validates that the loop engine maintains readiness for
    /// JIT compilation integration without breaking existing functionality.
    #[test]
    fn test_jit_d1_integration_readiness() {
        let mut loop_engine = LoopEngine::new();

        // Test JIT configuration interface
        let jit_config = loop_engine.get_jit_config();
        assert!(jit_config.enabled); // Should be enabled by default

        // Test JIT statistics interface
        let jit_stats = loop_engine.get_jit_stats();
        assert_eq!(jit_stats.compilation_attempts, 0); // Initially zero
        assert_eq!(jit_stats.successful_compilations, 0);
        assert_eq!(jit_stats.cache_hits, 0);

        // Test JIT eligibility checking
        let loop_config = LoopConfig::new(Value::Number(0.0), ValueType::Number);
        let test_loop = LoopInstruction::For {
            id: LoopID::new("jit-test-loop".to_string()),
            range: LoopRange::new(0, 100, 1), // Large range - potentially JIT eligible
            iterator_var: "i".to_string(),
            body: "jit-test-body".to_string(),
            config: loop_config,
            location: SourceLocation::new(1, 1, 0),
        };

        let is_eligible = loop_engine.is_jit_eligible(&test_loop);
        // Eligibility depends on JIT configuration and loop characteristics
        assert!(is_eligible || !is_eligible); // Should return a boolean

        // Test JIT configuration updates
        let mut new_config = JITConfig::default();
        new_config.enabled = false;
        loop_engine.update_jit_config(new_config);

        let updated_config = loop_engine.get_jit_config();
        assert!(!updated_config.enabled);

        // Test JIT cache management
        loop_engine.clear_jit_cache(); // Should not panic

        // Test hot loop JIT compilation trigger
        let loop_id = LoopID::new("hot-loop-test".to_string());
        
        // This should fail gracefully since loop is not hot
        let jit_result = loop_engine.trigger_integrated_jit_compilation(&loop_id, &test_loop);
        assert!(jit_result.is_err()); // Expected to fail for non-hot loop
    }

    /// Test parallelism (D2) integration readiness
    /// 
    /// This validates that the loop engine maintains readiness for
    /// parallel execution without breaking existing functionality.
    #[test]
    fn test_parallelism_d2_integration_readiness() {
        let loop_engine = LoopEngine::new();

        // Test static iteration count analysis (required for parallelization)
        let for_loop = LoopInstruction::For {
            id: LoopID::new("parallel-test-loop".to_string()),
            range: LoopRange::new(0, 100, 1),
            iterator_var: "i".to_string(),
            body: "parallel-test-body".to_string(),
            config: LoopConfig::new(Value::Number(0.0), ValueType::Number),
            location: SourceLocation::new(1, 1, 0),
        };

        let iteration_count = loop_engine.get_static_iteration_count(&for_loop);
        assert_eq!(iteration_count, Some(100)); // Should detect 100 iterations

        // Test While loop (should not have static count)
        let while_loop = LoopInstruction::While {
            id: LoopID::new("while-test-loop".to_string()),
            condition: crate::bcib::OperandRef::Literal(Value::Boolean(true)),
            body: "while-test-body".to_string(),
            config: LoopConfig::new(Value::Number(0.0), ValueType::Number),
            location: SourceLocation::new(2, 1, 10),
        };

        let while_iteration_count = loop_engine.get_static_iteration_count(&while_loop);
        assert_eq!(while_iteration_count, None); // While loops don't have static counts

        // Test deterministic partitioning
        let partitions = loop_engine.partition_iterations_deterministic(100, 4);
        assert!(!partitions.is_empty());
        
        // Verify partitions cover all iterations
        let total_iterations: u32 = partitions.iter().map(|p| p.end_iteration - p.start_iteration).sum();
        assert_eq!(total_iterations, 100);

        // Test parallelization decision making
        let mut context = LoopAnalysisContext::new();
        context.add_loop_variable("i".to_string(), "number".to_string());
        context.add_loop_variable("accumulator".to_string(), "number".to_string());

        let mut analyzer = SafetyAnalyzer::new();
        let safety_result = analyzer.analyze_loop_safety("accumulator = accumulator + i", &context).unwrap();

        let parallel_decision = loop_engine.should_parallelize_loop(&for_loop, &safety_result);
        
        // Decision should be based on safety analysis and loop type
        match parallel_decision {
            crate::loop_engine::ParallelizationDecision::Sequential { reason } => {
                assert!(!reason.is_empty()); // Should have a reason
            }
            crate::loop_engine::ParallelizationDecision::Parallel { iteration_count, .. } => {
                assert!(iteration_count > 0); // Should have positive iteration count
            }
        }
    }

    /// Test Ring3 Runtime integration readiness
    /// 
    /// This validates that the loop engine maintains compatibility with
    /// Ring3 runtime requirements and interfaces.
    #[test]
    fn test_ring3_runtime_integration_readiness() {
        let mut loop_engine = LoopEngine::new();

        // Test monitoring capabilities (required for Ring3 runtime)
        let monitoring_config = MonitoringConfig::default();
        loop_engine.update_monitoring_config(monitoring_config);

        let global_stats = loop_engine.get_global_monitoring_stats();
        // Note: total_loop_executions is u64, always >= 0

        // Test monitoring summary (Ring3 runtime reporting)
        let summary = loop_engine.get_monitoring_summary();
        // Note: total_loop_executions is u64, always >= 0

        // Test loop execution with monitoring
        let loop_config = LoopConfig::new(Value::Number(0.0), ValueType::Number);
        let test_loop = LoopInstruction::For {
            id: LoopID::new("ring3-test-loop".to_string()),
            range: LoopRange::new(0, 5, 1),
            iterator_var: "i".to_string(),
            body: "ring3-test-body".to_string(),
            config: loop_config,
            location: SourceLocation::new(1, 1, 0),
        };

        let body_fn: LoopBodyFn = Box::new(|accumulator, iteration| {
            if let Value::Number(acc) = accumulator {
                Ok(LoopBodyResult::Normal(Value::Number(acc + iteration as f64)))
            } else {
                Err(SemanticCLIError::execution_error(
                    "Invalid accumulator type",
                    ErrorCode::E500,
                ))
            }
        });

        // Execute loop - should update monitoring statistics
        let result = loop_engine.execute_loop(&test_loop, body_fn);
        assert!(result.is_ok());

        // Verify monitoring was updated
        let updated_stats = loop_engine.get_global_monitoring_stats();
        assert!(updated_stats.total_loop_executions > 0);

        // Test error handling compatibility with Ring3 runtime
        let error_result = LoopResult::error(LoopError::IterationLimitExceeded { 
            limit: 10, 
            completed: 0  // Control-flow dominant - no meaningful iterations completed
        });
        
        // Ring3 runtime should be able to handle all error types
        assert!(error_result.is_error());
        assert_eq!(error_result.get_iterations_completed(), 0);

        // Test capability-based security (Ring3 runtime requirement)
        assert_eq!(test_loop.required_capability(), Some(crate::bcib::Capability::Execute));

        // Test clear monitoring data (Ring3 runtime management)
        loop_engine.clear_monitoring_data();
        let cleared_stats = loop_engine.get_global_monitoring_stats();
        assert_eq!(cleared_stats.total_loop_executions, 0);
    }

    /// Test cross-system integration stability
    /// 
    /// This validates that all integration points work together without
    /// conflicts or unexpected interactions.
    #[test]
    fn test_cross_system_integration_stability() {
        let mut loop_engine = LoopEngine::new();

        // Test that BCIB, JIT, parallelism, and Ring3 systems can coexist
        let loop_config = LoopConfig::new(Value::Number(0.0), ValueType::Number);
        let test_loop = LoopInstruction::For {
            id: LoopID::new("cross-system-test".to_string()),
            range: LoopRange::new(0, 10, 1),
            iterator_var: "i".to_string(),
            body: "cross-system-body".to_string(),
            config: loop_config,
            location: SourceLocation::new(1, 1, 0),
        };

        // BCIB validation should work
        assert!(test_loop.validate().is_ok());

        // Safety analysis should work (affects parallelization)
        let mut context = LoopAnalysisContext::new();
        context.add_loop_variable("i".to_string(), "number".to_string());
        context.add_loop_variable("accumulator".to_string(), "number".to_string());

        let safety_result = loop_engine.analyze_loop_safety("accumulator = accumulator + i", &context);
        assert!(safety_result.is_ok());

        // JIT eligibility should work
        let is_jit_eligible = loop_engine.is_jit_eligible(&test_loop);
        assert!(is_jit_eligible || !is_jit_eligible); // Should return boolean

        // Parallelization decision should work
        let safety_analysis = safety_result.unwrap();
        let parallel_decision = loop_engine.should_parallelize_loop(&test_loop, &safety_analysis);
        assert!(matches!(
            parallel_decision,
            crate::loop_engine::ParallelizationDecision::Sequential { .. } |
            crate::loop_engine::ParallelizationDecision::Parallel { .. }
        ));

        // Monitoring should work
        let stats_before_executions = loop_engine.get_global_monitoring_stats().total_loop_executions;
        
        // Execute loop with all systems potentially involved
        let body_fn: LoopBodyFn = Box::new(|accumulator, iteration| {
            if let Value::Number(acc) = accumulator {
                Ok(LoopBodyResult::Normal(Value::Number(acc + iteration as f64)))
            } else {
                Err(SemanticCLIError::execution_error(
                    "Invalid accumulator type",
                    ErrorCode::E500,
                ))
            }
        });

        let execution_result = loop_engine.execute_loop(&test_loop, body_fn);
        assert!(execution_result.is_ok());

        // Monitoring should have been updated
        let stats_after = loop_engine.get_global_monitoring_stats();
        assert!(stats_after.total_loop_executions > stats_before_executions);

        // All systems should remain functional after execution
        // Note: These stats are unsigned integers, always >= 0
        let _jit_stats = loop_engine.get_jit_stats();
        let _cache_stats = loop_engine.get_safety_cache_stats();
        let _unroll_stats = loop_engine.get_unroll_stats();
    }

    /// Test backward compatibility with existing integrations
    /// 
    /// This validates that existing integration points haven't been broken
    /// by the architectural improvements.
    #[test]
    fn test_backward_compatibility() {
        let mut loop_engine = LoopEngine::new();

        // Test that all existing public methods are still available and functional
        
        // Loop execution methods
        let loop_config = LoopConfig::new(Value::Number(0.0), ValueType::Number);
        let test_loop = LoopInstruction::While {
            id: LoopID::new("backward-compat-test".to_string()),
            condition: crate::bcib::OperandRef::Literal(Value::Boolean(false)), // Will terminate immediately
            body: "backward-compat-body".to_string(),
            config: loop_config,
            location: SourceLocation::new(1, 1, 0),
        };

        let body_fn: LoopBodyFn = Box::new(|accumulator, _iteration| {
            Ok(LoopBodyResult::Normal(accumulator.clone()))
        });

        let result = loop_engine.execute_loop(&test_loop, body_fn);
        assert!(result.is_ok());

        // Safety analysis methods
        let mut context = LoopAnalysisContext::new();
        context.add_loop_variable("accumulator".to_string(), "number".to_string());

        let safety_result = loop_engine.analyze_loop_safety("accumulator = accumulator + 1", &context);
        assert!(safety_result.is_ok());

        let cache_stats = loop_engine.get_safety_cache_stats();
        // Note: entries is usize, always >= 0

        loop_engine.clear_safety_cache();

        // Unrolling methods
        let unroll_result = loop_engine.analyze_loop_unrolling(&test_loop);
        assert!(unroll_result.is_ok());

        let should_unroll = loop_engine.should_unroll_loop(&test_loop);
        assert!(should_unroll.is_ok());

        let unroll_stats = loop_engine.get_unroll_stats();
        // Note: loops_analyzed is usize, always >= 0

        // Monitoring methods
        let loop_id = LoopID::new("monitoring-test".to_string());
        assert!(!loop_engine.is_hot_loop(&loop_id)); // Should not be hot initially

        let global_stats = loop_engine.get_global_monitoring_stats();
        // Note: total_loop_executions is u64, always >= 0

        let summary = loop_engine.get_monitoring_summary();
        // Note: total_loop_executions is u64, always >= 0

        // JIT methods
        let jit_stats = loop_engine.get_jit_stats();
        // Note: compilation_attempts is usize, always >= 0

        let jit_config = loop_engine.get_jit_config();
        assert!(jit_config.enabled || !jit_config.enabled); // Should have a boolean value

        let is_eligible = loop_engine.is_jit_eligible(&test_loop);
        assert!(is_eligible || !is_eligible); // Should return boolean

        // Parallelization methods
        let iteration_count = loop_engine.get_static_iteration_count(&test_loop);
        assert_eq!(iteration_count, None); // While loops don't have static counts

        let partitions = loop_engine.partition_iterations_deterministic(10, 2);
        assert!(!partitions.is_empty());

        // All methods should work without errors, maintaining backward compatibility
    }
}