//! Loop Safety Analysis Tests - Spec Implementation (10.3)
//!
//! This test suite covers the safety analysis system for loop bodies.
//! These tests are behind the d3_loop_spec feature flag.
//!
//! Run with: cargo test -p semantic-cli --features d3_loop_spec

#![cfg(feature = "d3_loop_spec")]

use semantic_cli::loop_engine::{LoopAnalysisContext, SafetyAnalyzer, SafetyClass};

// =============================================================================
// 10.3 Test Safety Analysis System
// =============================================================================

#[cfg(all(test, feature = "d3_loop_spec"))]
mod safety_analysis_tests {
    use super::*;

    #[test]
    fn test_safe_loop_body_classification() {
        let mut analyzer = SafetyAnalyzer::new();
        let mut context = LoopAnalysisContext::new();
        context.add_loop_variable("i".to_string(), "number".to_string());
        context.add_loop_variable("accumulator".to_string(), "number".to_string());

        // Safe computation: only arithmetic operations
        let safe_body = "accumulator = accumulator + i * 2";
        let result = analyzer.analyze_loop_safety(safe_body, &context).unwrap();

        assert_eq!(result.classification, SafetyClass::Safe);
        assert!(result.side_effects.is_empty());
        assert!(result.dependencies.is_empty());
        assert!(result.reason.contains("No side effects"));
    }

    #[test]
    fn test_io_side_effect_detection() {
        let mut analyzer = SafetyAnalyzer::new();
        let context = LoopAnalysisContext::new();

        // Test file I/O detection
        let file_io_body = "file_write('output.txt', data); accumulator = accumulator + i";
        let result = analyzer
            .analyze_loop_safety(file_io_body, &context)
            .unwrap();

        assert_eq!(result.classification, SafetyClass::Unsafe);
        assert!(!result.side_effects.is_empty());

        // Check that file I/O was detected
        let has_file_io = result.side_effects.iter().any(|effect| {
            matches!(
                effect,
                semantic_cli::loop_engine::safety_analyzer::SideEffect::IOOperation {
                    operation_type:
                        semantic_cli::loop_engine::safety_analyzer::IOOperationType::FileSystem,
                    ..
                }
            )
        });
        assert!(has_file_io);
    }

    #[test]
    fn test_console_io_detection() {
        let mut analyzer = SafetyAnalyzer::new();
        let context = LoopAnalysisContext::new();

        // Test console I/O detection
        let console_body = "print('Processing:', i); accumulator = accumulator + i";
        let result = analyzer
            .analyze_loop_safety(console_body, &context)
            .unwrap();

        assert_eq!(result.classification, SafetyClass::Unsafe);
        assert!(!result.side_effects.is_empty());

        // Check that console I/O was detected
        let has_console_io = result.side_effects.iter().any(|effect| {
            matches!(
                effect,
                semantic_cli::loop_engine::safety_analyzer::SideEffect::IOOperation {
                    operation_type:
                        semantic_cli::loop_engine::safety_analyzer::IOOperationType::Console,
                    ..
                }
            )
        });
        assert!(has_console_io);
    }

    #[test]
    fn test_network_io_detection() {
        let mut analyzer = SafetyAnalyzer::new();
        let context = LoopAnalysisContext::new();

        // Test network I/O detection
        let network_body = "response = http_request('api.example.com', data); accumulator = accumulator + response.length";
        let result = analyzer
            .analyze_loop_safety(network_body, &context)
            .unwrap();

        assert_eq!(result.classification, SafetyClass::Unsafe);
        assert!(!result.side_effects.is_empty());

        // Check that network I/O was detected
        let has_network_io = result.side_effects.iter().any(|effect| {
            matches!(
                effect,
                semantic_cli::loop_engine::safety_analyzer::SideEffect::IOOperation {
                    operation_type:
                        semantic_cli::loop_engine::safety_analyzer::IOOperationType::Network,
                    ..
                }
            )
        });
        assert!(has_network_io);
    }

    #[test]
    fn test_database_io_detection() {
        let mut analyzer = SafetyAnalyzer::new();
        let context = LoopAnalysisContext::new();

        // Test database I/O detection
        let db_body = "result = db_query('SELECT * FROM table WHERE id = ?', i); accumulator = accumulator + result.count";
        let result = analyzer.analyze_loop_safety(db_body, &context).unwrap();

        assert_eq!(result.classification, SafetyClass::Unsafe);
        assert!(!result.side_effects.is_empty());

        // Check that database I/O was detected
        let has_db_io = result.side_effects.iter().any(|effect| {
            matches!(
                effect,
                semantic_cli::loop_engine::safety_analyzer::SideEffect::IOOperation {
                    operation_type:
                        semantic_cli::loop_engine::safety_analyzer::IOOperationType::Database,
                    ..
                }
            )
        });
        assert!(has_db_io);
    }

    #[test]
    fn test_external_mutation_detection() {
        let mut analyzer = SafetyAnalyzer::new();
        let mut context = LoopAnalysisContext::new();
        context.add_external_variable("global_counter".to_string(), "number".to_string());

        // Test external mutation detection
        let mutation_body = "global_counter = global_counter + 1; accumulator = accumulator + i";
        let result = analyzer
            .analyze_loop_safety(mutation_body, &context)
            .unwrap();

        assert_eq!(result.classification, SafetyClass::Unsafe);
        assert!(!result.side_effects.is_empty());

        // Check that external mutation was detected
        let has_mutation = result.side_effects.iter().any(|effect| {
            matches!(effect, semantic_cli::loop_engine::safety_analyzer::SideEffect::ExternalMutation { 
                variable, 
                scope: semantic_cli::loop_engine::safety_analyzer::VariableScope::External, 
                .. 
            } if variable == "global_counter")
        });
        assert!(has_mutation);
    }

    #[test]
    fn test_external_call_detection() {
        let mut analyzer = SafetyAnalyzer::new();
        let context = LoopAnalysisContext::new();

        // Test external call detection
        let call_body = "result = unknown_function(i); accumulator = accumulator + result";
        let result = analyzer.analyze_loop_safety(call_body, &context).unwrap();

        assert_eq!(result.classification, SafetyClass::Unsafe);
        assert!(!result.side_effects.is_empty());

        // Check that external call was detected
        let has_call = result.side_effects.iter().any(|effect| {
            matches!(effect, semantic_cli::loop_engine::safety_analyzer::SideEffect::ExternalCall { 
                function_name, 
                known_side_effects: true, 
                .. 
            } if function_name == "unknown_function")
        });
        assert!(has_call);
    }

    #[test]
    fn test_loop_carried_dependency_detection() {
        let mut analyzer = SafetyAnalyzer::new();
        let mut context = LoopAnalysisContext::new();
        context.add_loop_variable("prev_value".to_string(), "number".to_string());

        // Test dependency detection: iteration N reads value written by iteration N-1
        let dependency_body = "current = prev_value + i; prev_value = current";
        let result = analyzer
            .analyze_loop_safety(dependency_body, &context)
            .unwrap();

        assert_eq!(result.classification, SafetyClass::Unsafe);
        assert!(!result.dependencies.is_empty());

        // Check that dependency was detected
        let has_dependency = result.dependencies.iter().any(|dep| {
            dep.variable == "prev_value"
                && matches!(
                    dep.dependency_type,
                    semantic_cli::loop_engine::safety_analyzer::DependencyType::ReadAfterWrite
                )
        });
        assert!(has_dependency);
    }

    #[test]
    fn test_multiple_side_effects() {
        let mut analyzer = SafetyAnalyzer::new();
        let mut context = LoopAnalysisContext::new();
        context.add_external_variable("global_var".to_string(), "number".to_string());

        // Test multiple side effects in one loop body
        let complex_body = "file_write('log.txt', i); print('Processing:', i); global_var = global_var + 1; unknown_function(i)";
        let result = analyzer
            .analyze_loop_safety(complex_body, &context)
            .unwrap();

        assert_eq!(result.classification, SafetyClass::Unsafe);
        assert!(result.side_effects.len() >= 4); // File I/O, console, mutation, external call

        // Check for different types of side effects
        let has_file_io = result.side_effects.iter().any(|e| {
            matches!(
                e,
                semantic_cli::loop_engine::safety_analyzer::SideEffect::IOOperation {
                    operation_type:
                        semantic_cli::loop_engine::safety_analyzer::IOOperationType::FileSystem,
                    ..
                }
            )
        });
        let has_console = result.side_effects.iter().any(|e| {
            matches!(
                e,
                semantic_cli::loop_engine::safety_analyzer::SideEffect::IOOperation {
                    operation_type:
                        semantic_cli::loop_engine::safety_analyzer::IOOperationType::Console,
                    ..
                }
            )
        });
        let has_mutation = result.side_effects.iter().any(|e| {
            matches!(
                e,
                semantic_cli::loop_engine::safety_analyzer::SideEffect::ExternalMutation { .. }
            )
        });
        let has_call = result.side_effects.iter().any(|e| {
            matches!(
                e,
                semantic_cli::loop_engine::safety_analyzer::SideEffect::ExternalCall { .. }
            )
        });

        assert!(has_file_io);
        assert!(has_console);
        assert!(has_mutation);
        assert!(has_call);
    }

    #[test]
    fn test_known_safe_functions() {
        let mut analyzer = SafetyAnalyzer::new();
        let context = LoopAnalysisContext::new();

        // Test with known safe function
        let safe_function_body = "result = math_add(accumulator, i); accumulator = result";
        let result = analyzer
            .analyze_loop_safety(safe_function_body, &context)
            .unwrap();

        // Known safe functions should not cause unsafe classification
        assert_eq!(result.classification, SafetyClass::Safe);
        assert!(result.side_effects.is_empty());
    }

    #[test]
    fn test_safety_analysis_caching() {
        let mut analyzer = SafetyAnalyzer::new();
        let context = LoopAnalysisContext::new();

        let loop_body = "accumulator = accumulator + i";

        // First analysis - should be a cache miss
        let result1 = analyzer.analyze_loop_safety(loop_body, &context).unwrap();
        let cache_stats1 = analyzer.cache_stats();

        // Second analysis - should be a cache hit
        let result2 = analyzer.analyze_loop_safety(loop_body, &context).unwrap();
        let cache_stats2 = analyzer.cache_stats();

        // Results should be identical
        assert_eq!(result1.classification, result2.classification);
        assert_eq!(result1.cache_key, result2.cache_key);

        // Cache should have one entry and at least one hit
        assert_eq!(cache_stats1.entries, 1);
        assert_eq!(cache_stats2.entries, 1);
        assert!(cache_stats2.hit_count > cache_stats1.hit_count);
    }

    #[test]
    fn test_cache_clearing() {
        let mut analyzer = SafetyAnalyzer::new();
        let context = LoopAnalysisContext::new();

        let loop_body = "accumulator = accumulator + i";

        // Analyze to populate cache
        analyzer.analyze_loop_safety(loop_body, &context).unwrap();
        assert_eq!(analyzer.cache_stats().entries, 1);

        // Clear cache
        analyzer.clear_cache();
        assert_eq!(analyzer.cache_stats().entries, 0);
    }

    #[test]
    fn test_conservative_mode_behavior() {
        let mut analyzer = SafetyAnalyzer::new();
        let context = LoopAnalysisContext::new();

        // Test with unknown function - should be treated as unsafe in conservative mode
        let unknown_body =
            "result = completely_unknown_function(i); accumulator = accumulator + result";
        let result = analyzer
            .analyze_loop_safety(unknown_body, &context)
            .unwrap();

        // In conservative mode, unknown functions should be treated as unsafe
        assert_eq!(result.classification, SafetyClass::Unsafe);
        assert!(!result.side_effects.is_empty());
    }
}
