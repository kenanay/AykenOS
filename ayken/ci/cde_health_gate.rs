// Constitutional Module: CDE Health Gate
// Read-only CI integration for health reports.

//! CI health gate (non-blocking by default).

use crate::cde::health_monitor::{HealthFlag, HealthFlagKind, HealthReport};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HealthGateLevel {
    Pass,
    Warn,
    Fail,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HealthGateResult {
    pub level: HealthGateLevel,
    pub messages: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HealthGateConfig {
    pub warn_on_flags: bool,
    pub fail_on_critical: bool,
    pub critical_flags: Vec<HealthFlagKind>,
}

impl HealthGateConfig {
    pub fn default() -> Self {
        Self {
            warn_on_flags: true,
            fail_on_critical: false,
            critical_flags: vec![HealthFlagKind::PhaseDrift, HealthFlagKind::RuleInflation],
        }
    }
}

pub fn evaluate_health(report: &HealthReport, config: &HealthGateConfig) -> HealthGateResult {
    let mut messages = Vec::new();
    for flag in &report.flags {
        messages.push(flag.message.clone());
    }

    if config.fail_on_critical {
        let has_critical = report
            .flags
            .iter()
            .any(|f| config.critical_flags.contains(&f.kind));
        if has_critical {
            return HealthGateResult {
                level: HealthGateLevel::Fail,
                messages,
            };
        }
    }

    if config.warn_on_flags && !report.flags.is_empty() {
        return HealthGateResult {
            level: HealthGateLevel::Warn,
            messages,
        };
    }

    HealthGateResult {
        level: HealthGateLevel::Pass,
        messages,
    }
}

pub fn summarize_flags(flags: &[HealthFlag]) -> Vec<String> {
    flags.iter().map(|f| f.message.clone()).collect()
}
