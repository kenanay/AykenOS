//! B-MODE (Specification Mode) for D4 Constitutional Framework
//!
//! This module contains pure B-MODE implementations that focus on specification,
//! analysis, and reporting without any enforcement or execution semantics.
//!
//! ## B-MODE PRINCIPLES
//!
//! 1. **Pure Specification**: All operations return SpecificationReport, never Result<()> for spec violations
//! 2. **Immutable Analysis**: All trait methods use &self, no &mut self operations
//! 3. **No Side Effects**: No state mutations, caches, or persistent changes
//! 4. **Analysis Only**: Specify requirements and analyze compliance, never enforce
//!
//! ## ARCHITECTURAL SEPARATION
//!
//! B-MODE is completely separated from runtime enforcement:
//! - bmode/ contains only specification and analysis
//! - runtime/ contains enforcement and execution (when needed)
//! - No imports from runtime/ to bmode/ (one-way dependency)
//!
//! ## KEY DIFFERENCES FROM RUNTIME MODE
//!
//! - `add_rule()` → `specify_rule_addition()`
//! - `Result<()>` → `SpecificationReport`
//! - `&mut self` → `&self`
//! - `SystemAction` → `RecommendedSystemAction`
//! - `panic!` → violation in report
//! - `UUID::new()` → `RuleId::from_content()`

pub mod constitutional;
pub mod determinism;
pub mod integration;
pub mod templates;
pub mod validation_location;
pub mod types;
pub mod reports;
pub mod register_invariants;
pub mod contracts;
pub mod failure_matrix;
pub mod semantic_spec_registry;

#[cfg(test)]
mod tests;

// Re-export key B-MODE interfaces
pub use constitutional::{ConstitutionalRuleAnalyzer, DefaultConstitutionalRuleAnalyzer};
pub use determinism::{DeterminismAnalyzer, DefaultDeterminismAnalyzer};
pub use integration::{ConstitutionalIntegrationAnalyzer};
pub use templates::{TemplateSpecAnalyzer, DefaultTemplateSpecAnalyzer};
pub use validation_location::{ValidationLocationAnalyzer, DefaultValidationLocationAnalyzer};
// Sadece gerekli re-export'ları tut
pub use reports::{BModeSpecificationReport, analyze_bmode_compliance};
pub use register_invariants::{RegisterInvariantsAnalyzer, DefaultRegisterInvariantsAnalyzer};
pub use contracts::{ContractSpecAnalyzer, DefaultContractSpecAnalyzer};
pub use failure_matrix::{FailureMatrixAnalyzer, DefaultFailureMatrixAnalyzer};
pub use semantic_spec_registry::{SemanticSpecRegistry, DefaultSemanticSpecRegistry};

// Re-export key B-MODE types
pub use types::{RuleType, EnforcementLevel, OperationType, JITOperation};
pub use constitutional::{
    ConstitutionalRuleSpec, RuleSpec, SystemSpec,
    RecommendedResponse, RecommendedAction, AuthorityHierarchySpec
};
pub use determinism::{
    AllocationInputs, StateChange, ConstitutionalAction, SystemState,
    FailureScenario, RecommendedSystemAction, AuditLogSpec
};
pub use templates::{
    TemplateType, TemplateSpecification, TemplateField, TemplateAvailabilityReport,
    TemplateCatalog
};
// Sadece var olan type'ları re-export et
pub use validation_location::{
    ValidationLocation, ValidationPhase, LocationContext, LocationAnalysisReport
};
pub use register_invariants::{
    RegisterInvariantType, RegisterInvariantSpec, RegisterInvariantRequirementsReport
};