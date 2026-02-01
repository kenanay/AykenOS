// Constitutional Module: CI Fix Enforcement
// This module MUST NOT mutate code or auto-apply fixes.
// Outputs are advisory-only with deterministic PASS/WARN/FAIL decisions.

//! CI enforcement for ARH fixes (advisory-only).

use crate::arh::arh_engine::ArhOutput;
use crate::arh::fix_validation::{classify_fix, FixSeverity};
use crate::arh::regression_tracking::{RegressionDetector, RegressionRecord};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CIEnforcementLevel {
    Pass,
    Warn,
    Fail,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CIEnforcementResult {
    pub level: CIEnforcementLevel,
    pub messages: Vec<String>,
    pub exit_code: i32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CIAcknowledgmentRegistry {
    pub waivers: Vec<CIWaiver>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CIWaiver {
    pub violation_id: String,
    pub reason: String,
    pub approved_by: String,
    pub timestamp: String,
}

pub struct CIFixEnforcement {
    regression: RegressionDetector,
}

impl CIFixEnforcement {
    pub fn new(regression: RegressionDetector) -> Self {
        Self { regression }
    }

    /// Enforce CI rules deterministically without mutating code.
    pub fn enforce(
        &self,
        outputs: &[(String, ArhOutput, bool)],
        acknowledgments: &CIAcknowledgmentRegistry,
        regressions: &[RegressionRecord],
    ) -> CIEnforcementResult {
        let mut level = CIEnforcementLevel::Pass;
        let mut messages = Vec::new();

        // Regression failures are highest priority.
        if let Some(regression_msg) = self.regression.detect(regressions) {
            messages.push(regression_msg);
            return CIEnforcementResult {
                level: CIEnforcementLevel::Fail,
                messages,
                exit_code: 2,
            };
        }

        for (rule_id, output, is_kernel) in outputs {
            for hint in &output.ranked_hints {
                let validation = classify_fix(rule_id, hint, *is_kernel);

                // Kernel never auto-fails on fix availability; emit escalation only.
                if *is_kernel && validation.enforceable {
                    messages.push(format!(
                        "KERNEL NOTICE: fix available for {} but kernel enforcement is disabled. {}",
                        rule_id, validation.reason
                    ));
                    continue;
                }

                if validation.enforceable && !acknowledged(&output.violation_id, acknowledgments) {
                    match validation.severity {
                        FixSeverity::SecurityCritical => {
                            level = CIEnforcementLevel::Fail;
                            messages.push(format!(
                                "CI FAILED: {}\n{}",
                                rule_id, validation.suggested_command
                            ));
                        }
                        FixSeverity::PerformanceRisk => {
                            if level != CIEnforcementLevel::Fail {
                                level = CIEnforcementLevel::Warn;
                            }
                            messages.push(format!(
                                "CI WARNING: {}\n{}",
                                rule_id, validation.suggested_command
                            ));
                        }
                        FixSeverity::Correctness => {
                            level = CIEnforcementLevel::Fail;
                            messages.push(format!(
                                "CI FAILED: {}\n{}",
                                rule_id, validation.suggested_command
                            ));
                        }
                    }
                }
            }
        }

        let exit_code = match level {
            CIEnforcementLevel::Pass => 0,
            CIEnforcementLevel::Warn => 0,
            CIEnforcementLevel::Fail => 1,
        };

        CIEnforcementResult {
            level,
            messages,
            exit_code,
        }
    }
}

fn acknowledged(violation_id: &str, registry: &CIAcknowledgmentRegistry) -> bool {
    registry.waivers.iter().any(|w| w.violation_id == violation_id)
}
