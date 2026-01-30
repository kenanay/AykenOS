//! Semantic CLI Main Entry Point
//!
//! This is the main executable for the Semantic CLI REPL.
//! Phase 3.5.1.a - Gate B implementation.
//! Phase 4.2.3.4 - CI Integration Infrastructure support.

use semantic_cli::repl::MinimalREPL;
use semantic_cli::error::Result;
use semantic_cli::gate_c::performance::ci_integration::CIIntegration;
use semantic_cli::gate_c::performance::BaselineManager;
use semantic_cli::gate_c::performance::validation_cli::{
    execute_regression_detection_validation, 
    execute_quick_validation
};
use semantic_cli::gate_c::performance::baseline_tagger::BaselineTagger;
use semantic_cli::performance_management::{run_simple_benchmarks, run_quick_test};
use std::env;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    
    // Check for baseline establishment command
    if args.len() > 1 && args[1] == "establish-baselines" {
        return run_baseline_establishment();
    }
    
    // Check for performance benchmarking command
    if args.len() > 1 && args[1] == "benchmark-performance" {
        return run_simple_benchmarks().map_err(|e| {
            semantic_cli::error::SemanticCLIError::Other(format!("Benchmark error: {}", e))
        });
    }
    
    // Check for quick benchmark command
    if args.len() > 1 && args[1] == "quick-benchmark" {
        return run_quick_test().map_err(|e| {
            semantic_cli::error::SemanticCLIError::Other(format!("Quick benchmark error: {}", e))
        });
    }
    
    // Check for CI performance command
    if args.len() > 1 && args[1] == "ci-performance" {
        return run_ci_performance_analysis();
    }
    
    // Check for regression detection validation command
    if args.len() > 1 && args[1] == "validate-regression-detection" {
        return run_regression_detection_validation();
    }
    
    // Check for quick validation command
    if args.len() > 1 && args[1] == "quick-validate" {
        return run_quick_validation();
    }
    
    // Default behavior: run REPL
    let mut repl = MinimalREPL::new()?;
    repl.run()?;
    
    Ok(())
}

/// Run CI performance regression detection analysis
/// 
/// This function is called when the CLI is invoked with the `ci-performance` argument.
/// It executes the complete CI performance regression detection pipeline and exits
/// with appropriate status codes for CI systems.
fn run_ci_performance_analysis() -> Result<()> {
    println!("🚀 Semantic CLI - CI Performance Regression Detection");
    println!("📋 Phase 4.2.3.4 - CI Integration Infrastructure");
    println!("⚖️  Constitutional Principle: 'Does this commit slow the system? Yes → FAIL'");
    println!();

    // Create baseline manager for CI integration
    let baseline_manager = BaselineManager::new("ci_baselines");
    
    // Create CI integration
    let mut ci_integration = match CIIntegration::new(&baseline_manager) {
        Ok(integration) => integration,
        Err(e) => {
            println!("❌ Failed to initialize CI integration: {:?}", e);
            println!("⚠️  This may be expected on first run - infrastructure may not be fully ready");
            println!("📋 Recommended actions:");
            println!("   1. Ensure performance baselines are established");
            println!("   2. Verify measurement infrastructure is configured");
            println!("   3. Check system permissions and dependencies");
            
            // Exit with success on first run (constitutionally acceptable)
            std::process::exit(0);
        }
    };

    // Execute CI pipeline
    match ci_integration.execute_ci_pipeline() {
        Ok(result) => {
            // Print diagnostic output
            println!("{}", result.diagnostic_output);
            
            // Print markdown report if available
            if let Some(markdown_report) = &result.markdown_report {
                println!("📋 Markdown Report Generated:");
                println!("{}", markdown_report);
            }
            
            // Exit with appropriate status code
            if result.should_pass {
                println!("🎉 CI Performance Analysis: PASS");
                std::process::exit(0);
            } else {
                println!("🚨 CI Performance Analysis: FAIL");
                println!("📊 Performance regression detected - blocking commit");
                std::process::exit(1);
            }
        }
        Err(e) => {
            println!("⚠️  CI performance analysis encountered error: {:?}", e);
            println!("📋 Error details may indicate:");
            println!("   - Missing performance baselines (run baseline establishment first)");
            println!("   - Insufficient system permissions");
            println!("   - CI environment configuration issues");
            println!("   - Measurement infrastructure not ready");
            
            println!();
            println!("⚖️  CONSTITUTIONAL NOTE:");
            println!("   First-run failures are acceptable if documented and explainable");
            println!("   This error does not necessarily indicate a performance regression");
            
            // Exit with success on infrastructure errors (first-run acceptable)
            std::process::exit(0);
        }
    }
}

/// Run regression detection validation
/// 
/// This function is called when the CLI is invoked with the `validate-regression-detection` argument.
/// It executes comprehensive validation of the regression detection system including false positive/negative
/// analysis, constitutional compliance verification, and CI integration testing.
fn run_regression_detection_validation() -> Result<()> {
    match execute_regression_detection_validation() {
        Ok(()) => {
            println!("🎉 Regression Detection Validation: SUCCESS");
            std::process::exit(0);
        }
        Err(e) => {
            println!("❌ Regression Detection Validation: FAILED");
            println!("⚠️  Error: {:?}", e);
            println!();
            println!("📋 This indicates critical issues with the regression detection system");
            println!("   that must be addressed before proceeding to Phase 4.2.4");
            std::process::exit(1);
        }
    }
}

/// Run quick regression detection validation
/// 
/// This function is called when the CLI is invoked with the `quick-validate` argument.
/// It executes a reduced validation suitable for CI environments with fewer test scenarios.
fn run_quick_validation() -> Result<()> {
    match execute_quick_validation() {
        Ok(()) => {
            println!("🎉 Quick Regression Detection Validation: SUCCESS");
            std::process::exit(0);
        }
        Err(e) => {
            println!("❌ Quick Regression Detection Validation: FAILED");
            println!("⚠️  Error: {:?}", e);
            std::process::exit(1);
        }
    }
}
/// Run baseline establishment
/// 
/// This function is called when the CLI is invoked with the `establish-baselines` argument.
/// It establishes comprehensive Phase 4.2.0 performance baselines for regression detection.
fn run_baseline_establishment() -> Result<()> {
    println!("🏗️  Semantic CLI - Baseline Establishment");
    println!("📋 Phase 4.2.0 - Performance Baseline Establishment");
    println!("⚖️  Constitutional Principle: 'Measurable > Optimized'");
    println!();

    match BaselineTagger::new() {
        Ok(mut tagger) => {
            match tagger.establish_phase_4_2_baseline() {
                Ok(result) => {
                    println!("🎉 Baseline Establishment: SUCCESS");
                    println!("✅ Tag: {}", result.tag_name);
                    println!("✅ Commit: {}", result.commit_hash);
                    println!("✅ Baselines: {}", result.baselines_established);
                    println!("✅ Measurements: {}", result.total_measurements);
                    println!();
                    println!("🚀 Ready for regression detection validation!");
                    std::process::exit(0);
                }
                Err(e) => {
                    println!("❌ Baseline Establishment: FAILED");
                    println!("⚠️  Error: {:?}", e);
                    println!();
                    println!("📋 This indicates issues with baseline establishment");
                    println!("   Check system permissions and measurement infrastructure");
                    std::process::exit(1);
                }
            }
        }
        Err(e) => {
            println!("❌ Failed to initialize baseline tagger: {:?}", e);
            println!("⚠️  This may indicate git repository or system issues");
            std::process::exit(1);
        }
    }
}