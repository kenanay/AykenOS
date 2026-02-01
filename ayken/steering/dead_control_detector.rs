// Constitutional Module: Dead-Control Detector
// Analysis-only. Must fail-loud on placebo configuration.

//! Detect configuration controls with no observable effect.

use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum UsageEvidence {
    Read,
    ConditionalBranch,
    ThresholdCompare,
    DecisionModifier,
    DiagnosticEmission,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum EffectKind {
    Analysis,
    Enforcement,
    DecisionOutcome,
    Diagnostic,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ControlDefinition {
    pub path: String,
    pub source: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ControlUsage {
    pub path: String,
    pub evidence: UsageEvidence,
    pub effect: Option<EffectKind>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DeadControlReason {
    NeverRead,
    ReadButUnused,
    NoObservableEffect,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeadControlFinding {
    pub path: String,
    pub reason: DeadControlReason,
    pub evidence: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeadControlReport {
    pub findings: Vec<DeadControlFinding>,
}

pub fn detect_dead_controls(
    definitions: &[ControlDefinition],
    usages: &[ControlUsage],
) -> DeadControlReport {
    let mut usage_map: BTreeMap<String, Vec<&ControlUsage>> = BTreeMap::new();
    for usage in usages {
        usage_map.entry(usage.path.clone()).or_default().push(usage);
    }

    let mut findings = Vec::new();
    for def in definitions {
        match usage_map.get(&def.path) {
            None => findings.push(DeadControlFinding {
                path: def.path.clone(),
                reason: DeadControlReason::NeverRead,
                evidence: "Defined but never read".to_string(),
            }),
            Some(usages) => {
                let mut effects: BTreeSet<EffectKind> = BTreeSet::new();
                let mut evidence: BTreeSet<UsageEvidence> = BTreeSet::new();
                for usage in usages {
                    evidence.insert(usage.evidence.clone());
                    if let Some(effect) = usage.effect {
                        effects.insert(effect);
                    }
                }

                if effects.is_empty() {
                    let reason = if evidence.contains(&UsageEvidence::Read) {
                        DeadControlReason::ReadButUnused
                    } else {
                        DeadControlReason::NoObservableEffect
                    };
                    findings.push(DeadControlFinding {
                        path: def.path.clone(),
                        reason,
                        evidence: format!(
                            "Evidence={:?}, Effects=none",
                            evidence.iter().collect::<Vec<_>>()
                        ),
                    });
                }
            }
        }
    }

    DeadControlReport { findings }
}
