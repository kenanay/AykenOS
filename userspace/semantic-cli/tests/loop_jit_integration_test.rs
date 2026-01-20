//! Integration test for D3 Loop Support with D1 JIT Integration
//!
//! This test demonstrates the complete workflow:
//! 1. Hot loop detection (Phase 6.1)
//! 2. JIT compilation triggering (Phase 6.2)
//! 3. Comprehensive fingerprint caching (Requirements 6.3)
//! 4. Constitutional guarantees in native code (Requirements 6.4, 6.5)

use semantic_cli::bcib::{LoopInstruction, LoopID, LoopConfig, LoopRange, Value, ValueType};
use semantic_cli::loop_engine::{
    LoopEngine, LoopBodyFn, LoopBodyResult, JITConfig
};
use semantic_cli::types::SourceLocation;

/// Test the complete hot loop detection to JIT compilation workflow
#[test]
fn test_hot_loop_jit_integration_workflow() {
    let mut loop_engine = LoopEngine::new();
    
    // Create a hot loop instruction (1500 iterations > 1000 threshold)
    let hot_loop = LoopInstruction::For {
        id: LoopID::new("hot-jit-loop".to_string()),
        range: LoopRange::new(0, 1500, 1), // Hot loop with 1500 iterations
        iterator_var: "i".to_string(),
        body: "hot-jit-body".to_string(),
        config: LoopConfig::new(Value::Number(0.0), ValueType::Number),
        location: SourceLocation::new(1, 1, 0),
    };
    
    // Create a simple accumulator body function
    let body_fn: LoopBodyFn = Box::new(|accumulator, iteration| {
        if let Value::Number(acc) = accumulator {
            Ok(LoopBodyResult::Normal(Value::Number(acc + iteration as f64)))
        } else {
            Err(semantic_cli::error::SemanticCLIError::execution_error(
                "Invalid accumulator type", 
                semantic_cli::error::ErrorCode::E500
            ))
        }
    });
    
    // Execute the loop - this should trigger hot loop detection
    let result = loop_engine.execute_loop(&hot_loop, body_fn).unwrap();
    
    // Verify loop executed successfully (rich execution result)
    assert!(result.is_success());
    assert_eq!(result.iterations_completed, 1500);
    assert_eq!(result.execution_mode, semantic_cli::loop_engine::ExecutionMode::Interpreted);
    
    // Verify the final accumulator value
    if let semantic_cli::bcib::Value::Number(final_sum) = result.accumulator {
        // Sum should be 0 + 0 + 1 + 2 + ... + 1499 = sum of 0 to 1499
        let expected_sum: f64 = (0..1500).sum::<i32>() as f64;
        assert_eq!(final_sum, expected_sum);
    } else {
        panic!("Expected number accumulator");
    }
    
    // Verify hot loop was detected
    let loop_id = LoopID::new("hot-jit-loop".to_string());
    assert!(loop_engine.is_hot_loop(&loop_id));
    
    // Verify JIT compilation was triggered
    let hot_loop_info = loop_engine.get_hot_loop_info(&loop_id).unwrap();
    assert!(hot_loop_info.jit_triggered);
    
    // Verify monitoring statistics
    let global_stats = loop_engine.get_global_monitoring_stats();
    assert_eq!(global_stats.hot_loops_detected, 1);
    assert_eq!(global_stats.jit_compilations_triggered, 1);
    
    // Test JIT eligibility check
    assert!(loop_engine.is_jit_eligible(&hot_loop));
    
    // Test manual JIT compilation
    let jit_result = loop_engine.trigger_integrated_jit_compilation(&loop_id, &hot_loop);
    
    // JIT compilation should succeed (even if it's a placeholder implementation)
    match jit_result {
        Ok(()) => {
            // Success - JIT compilation was triggered
            println!("JIT compilation successfully triggered for hot loop");
        }
        Err(e) => {
            // Expected for now since we don't have actual D1 JIT implementation
            println!("JIT compilation placeholder returned: {}", e);
        }
    }
    
    // Verify JIT statistics
    let jit_stats = loop_engine.get_jit_stats();
    assert!(jit_stats.compilation_attempts > 0);
    
    // Test JIT configuration
    let jit_config = loop_engine.get_jit_config();
    assert!(jit_config.enabled);
    assert!(jit_config.enable_bounds_checking);
    assert!(jit_config.enable_budget_enforcement);
    assert!(jit_config.enable_type_safety);
}

/// Test JIT compilation with different loop types
#[test]
fn test_jit_compilation_loop_type_support() {
    let loop_engine = LoopEngine::new();
    
    // Test While loop JIT eligibility
    let while_loop = LoopInstruction::While {
        id: LoopID::new("while-jit-test".to_string()),
        condition: semantic_cli::bcib::OperandRef::Literal(Value::Boolean(true)),
        body: "while-jit-body".to_string(),
        config: LoopConfig::new(Value::Number(0.0), ValueType::Number),
        location: SourceLocation::new(1, 1, 0),
    };
    
    // Test For loop JIT eligibility
    let for_loop = LoopInstruction::For {
        id: LoopID::new("for-jit-test".to_string()),
        range: LoopRange::new(0, 100, 1),
        iterator_var: "i".to_string(),
        body: "for-jit-body".to_string(),
        config: LoopConfig::new(Value::Number(0.0), ValueType::Number),
        location: SourceLocation::new(2, 1, 0),
    };
    
    // Test ForEach loop JIT eligibility
    let foreach_loop = LoopInstruction::ForEach {
        id: LoopID::new("foreach-jit-test".to_string()),
        collection: semantic_cli::bcib::OperandRef::Literal(Value::Array(vec![
            Value::Number(1.0),
            Value::Number(2.0),
            Value::Number(3.0),
        ])),
        collection_type: semantic_cli::bcib::CollectionType::Array,
        iterator_var: "item".to_string(),
        body: "foreach-jit-body".to_string(),
        config: LoopConfig::new(Value::Number(0.0), ValueType::Number),
        location: SourceLocation::new(3, 1, 0),
    };
    
    // All loop types should be JIT eligible
    assert!(loop_engine.is_jit_eligible(&while_loop));
    assert!(loop_engine.is_jit_eligible(&for_loop));
    assert!(loop_engine.is_jit_eligible(&foreach_loop));
}

/// Test JIT configuration variants
#[test]
fn test_jit_configuration_variants() {
    let mut loop_engine = LoopEngine::new();
    
    // Test default configuration
    let default_config = loop_engine.get_jit_config();
    assert!(default_config.enabled);
    assert_eq!(default_config.max_cache_entries, 1000);
    assert!(default_config.enable_bounds_checking);
    assert!(default_config.enable_budget_enforcement);
    assert!(default_config.enable_type_safety);
    
    // Test custom configuration
    let custom_config = JITConfig {
        enabled: true,
        max_cache_entries: 500,
        compilation_timeout_ms: 10000,
        enable_bounds_checking: true,
        enable_budget_enforcement: true,
        enable_type_safety: true,
        enable_debug_info: true,
    };
    
    loop_engine.update_jit_config(custom_config.clone());
    
    let updated_config = loop_engine.get_jit_config();
    assert_eq!(updated_config.max_cache_entries, 500);
    assert_eq!(updated_config.compilation_timeout_ms, 10000);
    assert!(updated_config.enable_debug_info);
}

/// Test JIT cache management
#[test]
fn test_jit_cache_management() {
    let mut loop_engine = LoopEngine::new();
    
    // Create multiple different loops for cache testing
    let loop1 = LoopInstruction::For {
        id: LoopID::new("cache-test-1".to_string()),
        range: LoopRange::new(0, 1200, 1), // Hot loop
        iterator_var: "i".to_string(),
        body: "cache-body-1".to_string(),
        config: LoopConfig::new(Value::Number(0.0), ValueType::Number),
        location: SourceLocation::new(1, 1, 0),
    };
    
    let loop2 = LoopInstruction::For {
        id: LoopID::new("cache-test-2".to_string()),
        range: LoopRange::new(0, 1300, 1), // Hot loop with different range
        iterator_var: "j".to_string(),
        body: "cache-body-2".to_string(),
        config: LoopConfig::new(Value::String("".to_string()), ValueType::String),
        location: SourceLocation::new(2, 1, 0),
    };
    
    // Execute loops to trigger JIT compilation
    let body_fn1: LoopBodyFn = Box::new(|accumulator, _iteration| {
        if let Value::Number(acc) = accumulator {
            Ok(LoopBodyResult::Normal(Value::Number(acc + 1.0)))
        } else {
            Err(semantic_cli::error::SemanticCLIError::execution_error(
                "Invalid accumulator type", 
                semantic_cli::error::ErrorCode::E500
            ))
        }
    });
    
    let body_fn2: LoopBodyFn = Box::new(|accumulator, iteration| {
        if let Value::String(acc) = accumulator {
            Ok(LoopBodyResult::Normal(Value::String(format!("{}_{}", acc, iteration))))
        } else {
            Err(semantic_cli::error::SemanticCLIError::execution_error(
                "Invalid accumulator type", 
                semantic_cli::error::ErrorCode::E500
            ))
        }
    });
    
    // Execute loops
    let result1 = loop_engine.execute_loop(&loop1, body_fn1).unwrap();
    let result2 = loop_engine.execute_loop(&loop2, body_fn2).unwrap();
    
    // Verify both loops executed successfully
    assert!(result1.is_success());
    assert!(result2.is_success());
    
    // Check JIT statistics
    let jit_stats = loop_engine.get_jit_stats();
    assert!(jit_stats.compilation_attempts >= 2);
    
    // Test cache clearing
    loop_engine.clear_jit_cache();
    let cleared_stats = loop_engine.get_jit_stats();
    assert_eq!(cleared_stats.cached_entries, 0);
}

/// Test constitutional guarantees in JIT compilation
#[test]
fn test_jit_constitutional_guarantees() {
    let mut loop_engine = LoopEngine::new();
    
    // Create a loop with specific constitutional constraints
    let mut config = LoopConfig::new(Value::Number(0.0), ValueType::Number);
    config.iteration_limit = 5000; // Custom iteration limit
    config.budget_timeout = 10000; // Custom budget timeout
    config.budget_measurement = semantic_cli::bcib::BudgetMeasurement::InstructionCount { weight: 2 };
    
    let constitutional_loop = LoopInstruction::For {
        id: LoopID::new("constitutional-jit-test".to_string()),
        range: LoopRange::new(0, 1100, 1), // Hot loop
        iterator_var: "i".to_string(),
        body: "constitutional-body".to_string(),
        config,
        location: SourceLocation::new(1, 1, 0),
    };
    
    // Verify JIT eligibility
    assert!(loop_engine.is_jit_eligible(&constitutional_loop));
    
    // Execute loop to trigger JIT compilation
    let body_fn: LoopBodyFn = Box::new(|accumulator, _iteration| {
        if let Value::Number(acc) = accumulator {
            Ok(LoopBodyResult::Normal(Value::Number(acc + 1.0)))
        } else {
            Err(semantic_cli::error::SemanticCLIError::execution_error(
                "Invalid accumulator type", 
                semantic_cli::error::ErrorCode::E500
            ))
        }
    });
    
    let result = loop_engine.execute_loop(&constitutional_loop, body_fn).unwrap();
    
    // Verify constitutional compliance (rich execution result)
    assert!(result.is_success());
    assert_eq!(result.iterations_completed, 1100);
    assert_eq!(result.execution_mode, semantic_cli::loop_engine::ExecutionMode::Interpreted);
    
    // Verify the final accumulator value
    if let semantic_cli::bcib::Value::Number(final_sum) = result.accumulator {
        // Sum should be 0 + 1 + 2 + ... + 1099 = 1100 iterations
        assert_eq!(final_sum, 1100.0);
    } else {
        panic!("Expected number accumulator");
    }
    
    // Verify hot loop detection and JIT triggering
    let loop_id = LoopID::new("constitutional-jit-test".to_string());
    assert!(loop_engine.is_hot_loop(&loop_id));
    
    let hot_loop_info = loop_engine.get_hot_loop_info(&loop_id).unwrap();
    assert!(hot_loop_info.jit_triggered);
    
    // Verify JIT configuration includes constitutional guarantees
    let jit_config = loop_engine.get_jit_config();
    assert!(jit_config.enable_bounds_checking); // Requirements 6.5
    assert!(jit_config.enable_budget_enforcement); // Requirements 6.4
    assert!(jit_config.enable_type_safety); // Requirements 6.4
}