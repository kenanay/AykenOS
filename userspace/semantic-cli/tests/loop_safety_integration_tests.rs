//! Integration tests for loop safety analysis system
//!
//! These tests demonstrate the complete integration of the safety analysis system
//! with the loop engine, validating Requirements 10.1, 10.2, 10.3.

use semantic_cli::loop_engine::{
    safety_analyzer::{
        DependencyType, IOOperationType, LoopCarriedDependency, SideEffect, VariableScope,
    },
    LoopAnalysisContext, LoopEngine, SafetyClass,
};

#[test]
fn test_loop_engine_safety_integration() {
    let mut loop_engine = LoopEngine::new();

    // Create analysis context
    let mut context = LoopAnalysisContext::new();
    context.add_loop_variable("i".to_string(), "number".to_string());
    context.add_loop_variable("accumulator".to_string(), "number".to_string());
    context.add_external_variable("global_var".to_string(), "number".to_string());

    // Test safe loop body
    let safe_body = "accumulator = accumulator + i * 2";
    let result = loop_engine
        .analyze_loop_safety(safe_body, &context)
        .unwrap();

    assert_eq!(result.classification, SafetyClass::Safe);
    assert!(result.side_effects.is_empty());
    assert!(result.dependencies.is_empty());
    assert!(result.reason.contains("No side effects"));
}

#[test]
fn test_io_side_effect_detection() {
    let mut loop_engine = LoopEngine::new();
    let context = LoopAnalysisContext::new();

    // Test file I/O detection
    let file_io_body = "file_write('output.txt', data); accumulator = accumulator + i";
    let result = loop_engine
        .analyze_loop_safety(file_io_body, &context)
        .unwrap();

    assert_eq!(result.classification, SafetyClass::Unsafe);
    assert!(!result.side_effects.is_empty());

    // Check that file I/O was detected
    let has_file_io = result.side_effects.iter().any(|effect| {
        matches!(
            effect,
            SideEffect::IOOperation {
                operation_type: IOOperationType::FileSystem,
                ..
            }
        )
    });
    assert!(has_file_io);
}

#[test]
fn test_external_mutation_detection() {
    let mut loop_engine = LoopEngine::new();
    let mut context = LoopAnalysisContext::new();
    context.add_external_variable("global_counter".to_string(), "number".to_string());

    // Test external mutation detection
    let mutation_body = "global_counter = global_counter + 1; accumulator = accumulator + i";
    let result = loop_engine
        .analyze_loop_safety(mutation_body, &context)
        .unwrap();

    assert_eq!(result.classification, SafetyClass::Unsafe);
    assert!(!result.side_effects.is_empty());

    // Check that external mutation was detected
    let has_mutation = result.side_effects.iter().any(|effect| {
        matches!(effect, SideEffect::ExternalMutation { 
            variable, 
            scope: VariableScope::External, 
            .. 
        } if variable == "global_counter")
    });
    assert!(has_mutation);
}

#[test]
fn test_external_call_detection() {
    let mut loop_engine = LoopEngine::new();
    let context = LoopAnalysisContext::new();

    // Test external call detection
    let call_body = "result = unknown_function(i); accumulator = accumulator + result";
    let result = loop_engine
        .analyze_loop_safety(call_body, &context)
        .unwrap();

    assert_eq!(result.classification, SafetyClass::Unsafe);
    assert!(!result.side_effects.is_empty());

    // Check that external call was detected
    let has_call = result.side_effects.iter().any(|effect| {
        matches!(effect, SideEffect::ExternalCall { 
            function_name, 
            known_side_effects: true, 
            .. 
        } if function_name == "unknown_function")
    });
    assert!(has_call);
}

#[test]
fn test_loop_carried_dependency_detection() {
    let mut loop_engine = LoopEngine::new();
    let mut context = LoopAnalysisContext::new();
    context.add_loop_variable("prev_value".to_string(), "number".to_string());

    // Test dependency detection
    let dependency_body = "current = prev_value + i; prev_value = current";
    let result = loop_engine
        .analyze_loop_safety(dependency_body, &context)
        .unwrap();

    assert_eq!(result.classification, SafetyClass::Unsafe);
    assert!(!result.dependencies.is_empty());

    // Check that dependency was detected
    let has_dependency = result.dependencies.iter().any(|dep| {
        dep.variable == "prev_value"
            && matches!(dep.dependency_type, DependencyType::ReadAfterWrite)
    });
    assert!(has_dependency);
}

#[test]
fn test_multiple_side_effects() {
    let mut loop_engine = LoopEngine::new();
    let mut context = LoopAnalysisContext::new();
    context.add_external_variable("global_var".to_string(), "number".to_string());

    // Test multiple side effects
    let complex_body = "file_write('log.txt', i); print('Processing:', i); global_var = global_var + 1; unknown_function(i)";
    let result = loop_engine
        .analyze_loop_safety(complex_body, &context)
        .unwrap();

    assert_eq!(result.classification, SafetyClass::Unsafe);
    assert!(result.side_effects.len() >= 4); // File I/O, console, mutation, external call

    // Check for different types of side effects
    let has_file_io = result.side_effects.iter().any(|e| {
        matches!(
            e,
            SideEffect::IOOperation {
                operation_type: IOOperationType::FileSystem,
                ..
            }
        )
    });
    let has_console = result.side_effects.iter().any(|e| {
        matches!(
            e,
            SideEffect::IOOperation {
                operation_type: IOOperationType::Console,
                ..
            }
        )
    });
    let has_mutation = result
        .side_effects
        .iter()
        .any(|e| matches!(e, SideEffect::ExternalMutation { .. }));
    let has_call = result
        .side_effects
        .iter()
        .any(|e| matches!(e, SideEffect::ExternalCall { .. }));

    assert!(has_file_io);
    assert!(has_console);
    assert!(has_mutation);
    assert!(has_call);
}

#[test]
fn test_safety_analysis_caching() {
    let mut loop_engine = LoopEngine::new();
    let context = LoopAnalysisContext::new();

    let loop_body = "accumulator = accumulator + i";

    // First analysis
    let result1 = loop_engine
        .analyze_loop_safety(loop_body, &context)
        .unwrap();
    let cache_stats1 = loop_engine.get_safety_cache_stats();

    // Second analysis - should use cache
    let result2 = loop_engine
        .analyze_loop_safety(loop_body, &context)
        .unwrap();
    let cache_stats2 = loop_engine.get_safety_cache_stats();

    // Results should be identical
    assert_eq!(result1.classification, result2.classification);
    assert_eq!(result1.cache_key, result2.cache_key);

    // Cache should have one entry
    assert_eq!(cache_stats1.entries, 1);
    assert_eq!(cache_stats2.entries, 1);
}

#[test]
fn test_known_safe_functions() {
    let mut loop_engine = LoopEngine::new();
    let context = LoopAnalysisContext::new();

    // Test with known safe function
    let safe_function_body = "result = math_add(accumulator, i); accumulator = result";
    let result = loop_engine
        .analyze_loop_safety(safe_function_body, &context)
        .unwrap();

    // Known safe functions should not cause unsafe classification
    assert_eq!(result.classification, SafetyClass::Safe);
    assert!(result.side_effects.is_empty());
}

#[test]
fn test_cache_clearing() {
    let mut loop_engine = LoopEngine::new();
    let context = LoopAnalysisContext::new();

    let loop_body = "accumulator = accumulator + i";

    // Analyze to populate cache
    loop_engine
        .analyze_loop_safety(loop_body, &context)
        .unwrap();
    assert_eq!(loop_engine.get_safety_cache_stats().entries, 1);

    // Clear cache
    loop_engine.clear_safety_cache();
    assert_eq!(loop_engine.get_safety_cache_stats().entries, 0);
}

#[test]
fn test_conservative_mode_behavior() {
    let mut loop_engine = LoopEngine::new();
    let context = LoopAnalysisContext::new();

    // Test with unknown function - should be treated as unsafe in conservative mode
    let unknown_body =
        "result = completely_unknown_function(i); accumulator = accumulator + result";
    let result = loop_engine
        .analyze_loop_safety(unknown_body, &context)
        .unwrap();

    // In conservative mode, unknown functions should be treated as unsafe
    assert_eq!(result.classification, SafetyClass::Unsafe);
    assert!(!result.side_effects.is_empty());
}

#[test]
fn test_network_io_detection() {
    let mut loop_engine = LoopEngine::new();
    let context = LoopAnalysisContext::new();

    // Test network I/O detection
    let network_body = "response = http_request('api.example.com', data); accumulator = accumulator + response.length";
    let result = loop_engine
        .analyze_loop_safety(network_body, &context)
        .unwrap();

    assert_eq!(result.classification, SafetyClass::Unsafe);

    // Check that network I/O was detected
    let has_network_io = result.side_effects.iter().any(|effect| {
        matches!(
            effect,
            SideEffect::IOOperation {
                operation_type: IOOperationType::Network,
                ..
            }
        )
    });
    assert!(has_network_io);
}

#[test]
fn test_database_io_detection() {
    let mut loop_engine = LoopEngine::new();
    let context = LoopAnalysisContext::new();

    // Test database I/O detection
    let db_body = "result = db_query('SELECT * FROM table WHERE id = ?', i); accumulator = accumulator + result.count";
    let result = loop_engine.analyze_loop_safety(db_body, &context).unwrap();

    assert_eq!(result.classification, SafetyClass::Unsafe);

    // Check that database I/O was detected
    let has_db_io = result.side_effects.iter().any(|effect| {
        matches!(
            effect,
            SideEffect::IOOperation {
                operation_type: IOOperationType::Database,
                ..
            }
        )
    });
    assert!(has_db_io);
}

#[test]
fn test_system_call_detection() {
    let mut loop_engine = LoopEngine::new();
    let context = LoopAnalysisContext::new();

    // Test system call detection
    let syscall_body =
        "result = system_call('ls', ['-la']); accumulator = accumulator + result.exit_code";
    let result = loop_engine
        .analyze_loop_safety(syscall_body, &context)
        .unwrap();

    assert_eq!(result.classification, SafetyClass::Unsafe);

    // Check that system call was detected
    let has_syscall = result.side_effects.iter().any(|effect| {
        matches!(
            effect,
            SideEffect::IOOperation {
                operation_type: IOOperationType::SystemCall,
                ..
            }
        )
    });
    assert!(has_syscall);
}
