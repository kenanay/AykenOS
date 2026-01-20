//! Error System for D4 Constitutional Framework
//!
//! This module is split according to the Turkish kader tablosu (fate table):
//! - framework_error.rs: Framework initialization, IO, and system-level errors
//! - specification_reports.rs: Specification violations and analysis reports
//!
//! ## ARCHITECTURAL PRINCIPLE
//! Property/determinism/prerequisite violations are NEVER ConstitutionalError.
//! They go to SpecificationReport.violations[] instead.

pub mod framework_error;
pub mod specification_reports;

// Re-export framework errors
pub use framework_error::{ConstitutionalError, Result, ErrorContext};

// Re-export specification reports (B-MODE compliant)
pub use specification_reports::{
    SpecificationReport, SpecificationViolation, SpecificationFinding, SpecificationRecommendation,
    ViolationType, FindingType, RecommendationType, Priority,
    ContractViolationReport, ContractFinding, ContractViolation, ContractViolationType,
    ContractRecommendation, RecommendationPriority,
    GateDecisionReport, GateReadinessAnalysis, GatePhase, ReadinessStatus,
    TransitionFinding, TransitionFindingType, PermissionAnalysis, Permission,
    PermissionType, AuthorityLevel, PermissionGap, FingerprintAnalysis,
    FingerprintValidity, FingerprintVariation, IntegrityStatus,
    ConstitutionalViolation, PropertyTestFailureInfo, AnalysisSummary,
    ReportContext,
};