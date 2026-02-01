// Constitutional Module: Config Effectiveness Gate
// CI must fail on dead controls (no silent placebo).

//! CI gate for dead control detection.

use crate::diagnostic::dead_control_messages::format_finding;
use crate::steering::dead_control_detector::{DeadControlReport};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConfigGateLevel {
    Pass,
    Fail,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConfigGateResult {
    pub level: ConfigGateLevel,
    pub messages: Vec<String>,
}

pub fn evaluate_report(report: &DeadControlReport) -> ConfigGateResult {
    if report.findings.is_empty() {
        return ConfigGateResult {
            level: ConfigGateLevel::Pass,
            messages: Vec::new(),
        };
    }

    let messages = report
        .findings
        .iter()
        .map(format_finding)
        .collect::<Vec<_>>();

    ConfigGateResult {
        level: ConfigGateLevel::Fail,
        messages,
    }
}
