// Constitutional Module: CDE Health Monitor
// Analysis-only. Must not influence or alter decisions.

//! Self-health monitoring for CDE outputs (read-only, deterministic).

use std::collections::BTreeMap;

use crate::cde::decision_metrics::{DecisionDistribution, DecisionMetrics, DecisionOutcome, DecisionRecord};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum HealthFlagKind {
    LowEntropy,
    HighEntropy,
    OutcomeDominance,
    PhaseDrift,
    RuleInflation,
    OverWaivering,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HealthFlag {
    pub kind: HealthFlagKind,
    pub message: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct HealthSnapshot {
    pub phase: String,
    pub distribution: DecisionDistribution,
    pub entropy: f64,
    pub waiver_ratio: f64,
    pub active_rules: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub struct HealthReport {
    pub snapshot: HealthSnapshot,
    pub flags: Vec<HealthFlag>,
    pub messages: Vec<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PhasePolicy {
    pub max_fail_ratio: f64,
    pub max_waiver_ratio: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct HealthConfig {
    pub dominance_threshold: f64,
    pub entropy_low_threshold: f64,
    pub entropy_high_threshold: f64,
    pub waiver_ratio_threshold: f64,
    pub rule_inflation_threshold: usize,
    pub phase_policies: BTreeMap<String, PhasePolicy>,
}

impl HealthConfig {
    pub fn default() -> Self {
        let mut phase_policies = BTreeMap::new();
        phase_policies.insert(
            "Phase-1".to_string(),
            PhasePolicy {
                max_fail_ratio: 0.85,
                max_waiver_ratio: 0.20,
            },
        );
        phase_policies.insert(
            "Phase-2".to_string(),
            PhasePolicy {
                max_fail_ratio: 0.60,
                max_waiver_ratio: 0.15,
            },
        );
        phase_policies.insert(
            "Phase-3".to_string(),
            PhasePolicy {
                max_fail_ratio: 0.40,
                max_waiver_ratio: 0.10,
            },
        );
        Self {
            dominance_threshold: 0.80,
            entropy_low_threshold: 0.60,
            entropy_high_threshold: 1.90,
            waiver_ratio_threshold: 0.25,
            rule_inflation_threshold: 50,
            phase_policies,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct HealthInput {
    pub records: Vec<DecisionRecord>,
    pub declared_phase: String,
    pub active_rules: usize,
    pub previous_active_rules: Option<usize>,
}

pub struct HealthMonitor {
    config: HealthConfig,
}

impl HealthMonitor {
    pub fn new(config: HealthConfig) -> Self {
        Self { config }
    }

    pub fn evaluate(&self, input: HealthInput) -> HealthReport {
        let metrics = DecisionMetrics::from_records(&input.records);
        let snapshot = HealthSnapshot {
            phase: input.declared_phase.clone(),
            distribution: metrics.distribution.clone(),
            entropy: metrics.entropy,
            waiver_ratio: metrics.waiver_ratio,
            active_rules: input.active_rules,
        };

        let mut flags = Vec::new();
        let mut messages = Vec::new();

        // Outcome dominance
        let mut max_ratio = 0.0;
        let mut dominant = None;
        for outcome in [
            DecisionOutcome::Fail,
            DecisionOutcome::Allow,
            DecisionOutcome::Waiver,
            DecisionOutcome::Refactor,
        ] {
            let ratio = metrics.distribution.ratio(outcome);
            if ratio > max_ratio {
                max_ratio = ratio;
                dominant = Some(outcome);
            }
        }
        if max_ratio >= self.config.dominance_threshold {
            flags.push(HealthFlag {
                kind: HealthFlagKind::OutcomeDominance,
                message: format!(
                    "Outcome dominance detected: {:?} at {:.2}",
                    dominant, max_ratio
                ),
            });
        }

        // Entropy checks
        if metrics.entropy < self.config.entropy_low_threshold {
            flags.push(HealthFlag {
                kind: HealthFlagKind::LowEntropy,
                message: format!("Low decision entropy: {:.3}", metrics.entropy),
            });
        }
        if metrics.entropy > self.config.entropy_high_threshold {
            flags.push(HealthFlag {
                kind: HealthFlagKind::HighEntropy,
                message: format!("High decision entropy: {:.3}", metrics.entropy),
            });
        }

        // Phase alignment
        if let Some(policy) = self.config.phase_policies.get(&input.declared_phase) {
            let fail_ratio = metrics.distribution.ratio(DecisionOutcome::Fail);
            let waiver_ratio = metrics.waiver_ratio;
            if fail_ratio > policy.max_fail_ratio || waiver_ratio > policy.max_waiver_ratio {
                flags.push(HealthFlag {
                    kind: HealthFlagKind::PhaseDrift,
                    message: format!(
                        "Phase drift: fail {:.2} (max {:.2}), waiver {:.2} (max {:.2})",
                        fail_ratio, policy.max_fail_ratio, waiver_ratio, policy.max_waiver_ratio
                    ),
                });
            }
        } else {
            messages.push("Phase policy unknown; alignment check skipped".to_string());
        }

        // Rule inflation
        if let Some(previous) = input.previous_active_rules {
            let delta = input.active_rules.saturating_sub(previous);
            if delta >= self.config.rule_inflation_threshold {
                flags.push(HealthFlag {
                    kind: HealthFlagKind::RuleInflation,
                    message: format!(
                        "Rule inflation detected: +{} active rules",
                        delta
                    ),
                });
            }
        }

        // Over-waivering
        if metrics.waiver_ratio >= self.config.waiver_ratio_threshold {
            flags.push(HealthFlag {
                kind: HealthFlagKind::OverWaivering,
                message: format!("Waiver ratio high: {:.2}", metrics.waiver_ratio),
            });
        }

        HealthReport {
            snapshot,
            flags,
            messages,
        }
    }
}
