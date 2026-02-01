// Constitutional Module: Regression Tracking
// This module MUST NOT mutate code or auto-apply fixes.
// It detects reintroduced violations deterministically.

//! Regression detection for previously fixed violations.

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RegressionRecord {
    pub violation_id: String,
    pub first_fixed_commit: String,
    pub reintroduced_commit: String,
}

pub struct RegressionDetector;

impl RegressionDetector {
    pub fn detect(&self, regressions: &[RegressionRecord]) -> Option<String> {
        if let Some(record) = regressions.first() {
            return Some(format!(
                "CI FAILED: Regression detected for {} (fixed at {}, reintroduced at {})",
                record.violation_id, record.first_fixed_commit, record.reintroduced_commit
            ));
        }
        None
    }
}
