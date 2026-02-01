// Constitutional Module: AHTS → ARH Mapping Contract
// This module MUST NOT mutate code or apply fixes.
// It defines a deterministic, lossless mapping contract only.

//! CONSTITUTIONAL GUARANTEE
//! ------------------------
//! - AHTS produces analysis-only QuickFix hints
//! - ARH MUST NOT escalate these into automated actions
//! - This mapping is the sole legal bridge between systems
//! - Any deviation is a constitutional violation and MUST fail CI
//!
//! Canonical mapping between AHTS QuickFix outputs and ARH RefactorHint contract.

use crate::arh::assisted_fix_engine::ImpactScope;
use crate::arh::fix_mapping::HintType;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TrustBoundary {
    Userspace,
    Kernel,
    Tooling,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AutomationLevel {
    Assisted,
}

#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum QuickFixImpactScope {
    ProcessOnly,
    ApiPotential,
    Architecture,
    CrossModule,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AhtsQuickFix {
    pub id: String,
    pub impact_scope: QuickFixImpactScope,
    pub constitutional_justification: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ArhRefactorHintContract {
    pub hint_type: HintType,
    pub automation_level: AutomationLevel,
    pub trust_boundary: TrustBoundary,
    pub impact_scope: ImpactScope,
    pub constitutional_justification: String,
}

/// Map a QuickFix impact scope into the ARH impact scope without semantic drift.
/// ⚠️ Constitutional invariant:
/// Any new QuickFixImpactScope variant MUST be explicitly mapped.
/// Silent defaults are forbidden.
pub fn map_impact_scope(scope: QuickFixImpactScope) -> ImpactScope {
    match scope {
        QuickFixImpactScope::ProcessOnly => ImpactScope::ProcessOnly,
        QuickFixImpactScope::ApiPotential => ImpactScope::ApiPotential,
        QuickFixImpactScope::Architecture => ImpactScope::Architecture,
        QuickFixImpactScope::CrossModule => ImpactScope::CrossModule,
    }
}

/// Canonical mapping from AHTS QuickFix to ARH RefactorHint contract.
/// This mapping is total, deterministic, and advisory-only.
pub fn map_quickfix_to_refactor_hint(ahts: &AhtsQuickFix) -> ArhRefactorHintContract {
    ArhRefactorHintContract {
        hint_type: HintType::AssistedFix,
        automation_level: AutomationLevel::Assisted,
        // Constitutional invariant: QuickFix hints are always Userspace-bound advisory artifacts.
        trust_boundary: TrustBoundary::Userspace,
        impact_scope: map_impact_scope(ahts.impact_scope),
        constitutional_justification: ahts.constitutional_justification.clone(),
    }
}
