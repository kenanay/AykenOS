// Constitutional Module: System Health Reporting
// Read-only summary of governance subsystem health.

//! System health snapshot for CLI display (deterministic, no recalculation).

use std::collections::BTreeMap;

use crate::analytics::refactor_effectiveness::EffectivenessDistribution;
use crate::arh::refactor_outcome::Effectiveness;
use crate::cde::decision_metrics::DecisionOutcome;
use crate::cde::health_monitor::HealthFlagKind;
use crate::steering::dead_control_detector::DeadControlReport;

#[derive(Clone, Debug, PartialEq)]
pub struct CdeHealthSummary {
    pub phase: String,
    pub entropy: f64,
    pub outcome_distribution: BTreeMap<DecisionOutcome, usize>,
    pub phase_alignment: String,
    pub flags: Vec<HealthFlagKind>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ArhActivitySummary {
    pub recent_refactors: usize,
    pub effectiveness_distribution: EffectivenessDistribution,
    pub confidence_adjustment_count: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WaiverRefactorSummary {
    pub open_waivers: usize,
    pub oldest_waiver_days: Option<u32>,
    pub aging_refactors: usize,
    pub repeated_waivers: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SystemRiskSummary {
    pub dead_controls: usize,
    pub over_waivering: bool,
    pub rule_inflation: bool,
    pub governance_flags: Vec<HealthFlagKind>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SystemHealthReport {
    pub cde: Option<CdeHealthSummary>,
    pub arh: Option<ArhActivitySummary>,
    pub waivers: Option<WaiverRefactorSummary>,
    pub risks: SystemRiskSummary,
}

impl SystemHealthReport {
    pub fn empty() -> Self {
        Self {
            cde: None,
            arh: None,
            waivers: None,
            risks: SystemRiskSummary {
                dead_controls: 0,
                over_waivering: false,
                rule_inflation: false,
                governance_flags: Vec::new(),
            },
        }
    }
}

pub fn risks_from_dead_controls(report: &DeadControlReport) -> SystemRiskSummary {
    SystemRiskSummary {
        dead_controls: report.findings.len(),
        over_waivering: false,
        rule_inflation: false,
        governance_flags: Vec::new(),
    }
}

pub fn empty_effectiveness_distribution() -> EffectivenessDistribution {
    let mut counts: BTreeMap<Effectiveness, usize> = BTreeMap::new();
    counts.insert(Effectiveness::Positive, 0);
    counts.insert(Effectiveness::Neutral, 0);
    counts.insert(Effectiveness::Negative, 0);
    EffectivenessDistribution { counts, total: 0 }
}
