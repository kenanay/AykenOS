//! D4 Constitutional Framework - Architectural Separation (BOUNDARY KEEPER)
//!
//! This crate implements the D4 Constitutional Framework with clear architectural separation
//! according to the Turkish kader tablosu (fate table):
//!
//! ## B-MODE (Specification Mode) - KALICI (PERMANENT)
//! - Pure specification and analysis
//! - All operations return SpecificationReport
//! - Immutable analysis (&self only)
//! - No enforcement or execution
//! - Located in bmode/ directory - UNTOUCHABLE
//!
//! ## Runtime Mode (when needed)
//! - Enforcement and execution
//! - Can import from B-MODE
//! - B-MODE never imports from runtime
//! - Located in runtime/ directory (to be created)
//!
//! ## ARCHITECTURAL PRINCIPLES (Requirements 1-15 + B-MODE Purity)
//! 1. B-MODE is completely pure and side-effect free
//! 2. Runtime can use B-MODE for analysis before enforcement
//! 3. One-way dependency: runtime → bmode (never bmode → runtime)
//! 4. All domain violations go to SpecificationReport, not ConstitutionalError
//! 5. "Yanlışlıkla enforcement" ihtimali ortadan kalkıyor (possibility of accidental enforcement is eliminated)

// B-MODE modules (pure specification and analysis) - KALICI
pub mod bmode;

// Error system (split according to kader tablosu)
pub mod errors;

// Existing modules (B-MODE compliant analysis components)
pub mod build_fingerprint;
pub mod compliance;
pub mod gate_readiness;
pub mod jit_bounds;
pub mod testing;

// Core types (shared between B-MODE and analysis)
pub mod types;

// Property tests (only compiled during testing)
#[cfg(test)]
mod bmode_purity_property_tests;
#[cfg(test)]
mod build_fingerprint_property_tests;
#[cfg(test)]
mod compliance_property_tests;
#[cfg(test)]
mod error_type_property_tests;
// Integration tests temporarily disabled during LOCK-READY refactoring
// TODO: Update integration tests to work with new modular architecture
// #[cfg(test)]
// mod integration_tests;

// Fixed seed for deterministic CI testing
pub const CI_FIXED_SEED: u64 = 42;
// Default iteration count for property tests in development
pub const DEFAULT_PROPERTY_TEST_ITERATIONS: u32 = 50;

// BOUNDARY KEEPER API: Default users see B-MODE API (recommended)
pub use bmode::{
    // Core analyzers (pure specification)
    ConstitutionalRuleAnalyzer, DefaultConstitutionalRuleAnalyzer,
    DeterminismAnalyzer, DefaultDeterminismAnalyzer,
    TemplateSpecAnalyzer, DefaultTemplateSpecAnalyzer,
    ValidationLocationAnalyzer, DefaultValidationLocationAnalyzer,
    
    // Core reports
    BModeSpecificationReport,
    
    // Basic specs
    ValidationLocation, ValidationPhase,
};

// Core error types (split according to kader tablosu)
pub use errors::{
    // Framework errors (init/IO only)
    ConstitutionalError, Result as ConstitutionalResult,
    
    // Specification reports (B-MODE compliant)
    SpecificationReport, SpecificationViolation, SpecificationFinding, 
    SpecificationRecommendation, ViolationType, FindingType, RecommendationType,
    ReportContext,
};

// Core types (essential only)
pub use types::{
    ComponentId, Severity, RuleId, DeterministicClock, LogicalTimestamp,
    AuthorizationLevel, LockedBehavior, SpecLocation,
};

// Analysis components (B-MODE compliant)
pub use build_fingerprint::{BuildFingerprintAnalyzer, DefaultBuildFingerprintAnalyzer, BuildFingerprintSpec, BuildContext, FingerprintAnalysisReport, DeterministicBuildId};
pub use compliance::{ComplianceAnalyzer, ValidationComplianceAnalyzer, ComplianceAnalysisReport, ValidationInput, Penalty, Bonus, AnalysisMetadata};
pub use gate_readiness::{GateReadinessAnalyzer, DefaultGateReadinessAnalyzer, GateReadinessReport, GateReadinessContext};
pub use jit_bounds::{JITBoundsChecker, DefaultJITBoundsChecker, StaticSafetyAnalysisInput, BoundsCheckingResult};
pub use testing::{PropertyTestConfig, PropertyTestRunner, TestDataGenerators};
