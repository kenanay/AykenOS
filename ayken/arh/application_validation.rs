// Constitutional Module: ApplicationValidation
// Validation must be deterministic and fail-closed.

//! Pre-apply and post-apply validation guardrails.

use crate::arh::fix_application_engine::FixPlan;
use std::collections::HashSet;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ValidationResult {
    Ok,
    Failed,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ValidationReport {
    pub result: ValidationResult,
    pub message: String,
}

pub trait PostApplyVerifier {
    fn verify(&self, module_id: &str) -> Result<(), String>;
}

pub struct NoopVerifier;

impl PostApplyVerifier for NoopVerifier {
    fn verify(&self, _module_id: &str) -> Result<(), String> {
        Ok(())
    }
}

pub struct ApplicationValidation {
    verifier: Box<dyn PostApplyVerifier>,
}

impl ApplicationValidation {
    pub fn new() -> Self {
        Self {
            verifier: Box::new(NoopVerifier),
        }
    }

    pub fn with_verifier(verifier: Box<dyn PostApplyVerifier>) -> Self {
        Self { verifier }
    }

    pub fn pre_apply(&self, plan: &FixPlan) -> ValidationReport {
        if plan.module_id.is_empty() {
            return ValidationReport {
                result: ValidationResult::Failed,
                message: "Missing module_id".to_string(),
            };
        }
        let mut seen = HashSet::new();
        for step in &plan.plans {
            if step.module_id != plan.module_id {
                return ValidationReport {
                    result: ValidationResult::Failed,
                    message: "Cross-module plan is forbidden".to_string(),
                };
            }
            if step.file.trim().is_empty() || step.range.trim().is_empty() {
                return ValidationReport {
                    result: ValidationResult::Failed,
                    message: "Plan step missing file or range".to_string(),
                };
            }
            let key = format!("{}::{}", step.file, step.range);
            if !seen.insert(key) {
                return ValidationReport {
                    result: ValidationResult::Failed,
                    message: "Conflicting transformation ranges detected".to_string(),
                };
            }
        }
        ValidationReport {
            result: ValidationResult::Ok,
            message: "Pre-apply validation passed".to_string(),
        }
    }

    pub fn post_apply(&self, plan: &FixPlan) -> ValidationReport {
        match self.verifier.verify(&plan.module_id) {
            Ok(()) => ValidationReport {
                result: ValidationResult::Ok,
                message: "Post-apply verification passed".to_string(),
            },
            Err(err) => ValidationReport {
                result: ValidationResult::Failed,
                message: err,
            },
        }
    }
}
