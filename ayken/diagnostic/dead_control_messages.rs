// Constitutional Module: Dead-Control Diagnostics
// Messages must be explicit and fail-loud.

//! Canonical messages for dead control findings.

use crate::steering::dead_control_detector::{DeadControlFinding, DeadControlReason};

pub fn format_finding(finding: &DeadControlFinding) -> String {
    match finding.reason {
        DeadControlReason::NeverRead => format!(
            "Configuration field '{}' is defined but has no observable effect (never read).",
            finding.path
        ),
        DeadControlReason::ReadButUnused => format!(
            "Configuration field '{}' is read but has no observable effect (no impact on decisions).",
            finding.path
        ),
        DeadControlReason::NoObservableEffect => format!(
            "Configuration field '{}' has no observable effect on analysis or enforcement.",
            finding.path
        ),
    }
}
