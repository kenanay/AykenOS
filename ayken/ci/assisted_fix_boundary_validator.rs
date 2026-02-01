// Constitutional Module: AssistedFix Boundary Validator
// This module MUST NOT mutate code or apply fixes.
// It enforces mapping contract invariants and fails closed.

//! CI validator for AHTS → ARH AssistedFix mapping contract.

use crate::arh::ahts_mapping::{
    map_impact_scope, AhtsQuickFix, ArhRefactorHintContract, AutomationLevel, TrustBoundary,
};
use crate::arh::fix_mapping::HintType;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConstitutionalViolation {
    HintTypeBreach,
    AutomationEscalation,
    TrustBoundaryBreach,
    SemanticDrift,
    JustificationMutation,
}

impl ConstitutionalViolation {
    pub fn message(&self) -> &'static str {
        match self {
            ConstitutionalViolation::HintTypeBreach => {
                "AHTS QuickFix must map to HintType::AssistedFix"
            }
            ConstitutionalViolation::AutomationEscalation => {
                "AutomationLevel must remain Assisted"
            }
            ConstitutionalViolation::TrustBoundaryBreach => {
                "TrustBoundary must remain Userspace"
            }
            ConstitutionalViolation::SemanticDrift => {
                "ImpactScope mapping drift detected"
            }
            ConstitutionalViolation::JustificationMutation => {
                "Constitutional justification must be preserved verbatim"
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BoundaryViolation {
    pub quickfix_id: String,
    pub message: String,
}

pub struct AssistedFixBoundaryValidator;

impl AssistedFixBoundaryValidator {
    /// Validate a single mapping contract. Returns Ok if compliant, Err with message otherwise.
    pub fn validate_mapping(
        quickfix: &AhtsQuickFix,
        contract: &ArhRefactorHintContract,
    ) -> Result<(), ConstitutionalViolation> {
        if contract.hint_type != HintType::AssistedFix {
            return Err(ConstitutionalViolation::HintTypeBreach);
        }
        #[allow(unreachable_patterns)]
        match contract.automation_level {
            AutomationLevel::Assisted => {}
            _ => return Err(ConstitutionalViolation::AutomationEscalation),
        }
        if contract.trust_boundary != TrustBoundary::Userspace {
            return Err(ConstitutionalViolation::TrustBoundaryBreach);
        }
        if contract.impact_scope != map_impact_scope(quickfix.impact_scope) {
            return Err(ConstitutionalViolation::SemanticDrift);
        }
        if contract.constitutional_justification != quickfix.constitutional_justification {
            return Err(ConstitutionalViolation::JustificationMutation);
        }
        Ok(())
    }

    /// Validate a batch of mappings; returns all violations (fail-closed if any).
    pub fn validate_all(
        mappings: &[(AhtsQuickFix, ArhRefactorHintContract)],
    ) -> Result<(), Vec<BoundaryViolation>> {
        let mut violations = Vec::new();
        for (quickfix, contract) in mappings {
            if let Err(message) = Self::validate_mapping(quickfix, contract) {
                violations.push(BoundaryViolation {
                    quickfix_id: quickfix.id.clone(),
                    message: message.message().to_string(),
                });
            }
        }
        if violations.is_empty() {
            Ok(())
        } else {
            Err(violations)
        }
    }
}
