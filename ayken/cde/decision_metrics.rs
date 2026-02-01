// Constitutional Module: CDE Decision Metrics
// Analysis-only. Must not influence or alter decisions.

//! Deterministic decision metrics for CDE health analysis.

use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum DecisionOutcome {
    Fail,
    Allow,
    Waiver,
    Refactor,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DecisionRecord {
    pub timestamp: String,
    pub phase: String,
    pub rule_id: String,
    pub outcome: DecisionOutcome,
    pub waived: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DecisionDistribution {
    pub counts: BTreeMap<DecisionOutcome, usize>,
    pub total: usize,
}

impl DecisionDistribution {
    pub fn ratio(&self, outcome: DecisionOutcome) -> f64 {
        if self.total == 0 {
            0.0
        } else {
            let count = *self.counts.get(&outcome).unwrap_or(&0);
            count as f64 / self.total as f64
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct DecisionMetrics {
    pub distribution: DecisionDistribution,
    pub entropy: f64,
    pub waiver_ratio: f64,
}

pub fn compute_distribution(records: &[DecisionRecord]) -> DecisionDistribution {
    let mut counts: BTreeMap<DecisionOutcome, usize> = BTreeMap::new();
    for record in records {
        *counts.entry(record.outcome).or_insert(0) += 1;
    }
    let total = records.len();
    DecisionDistribution { counts, total }
}

pub fn compute_entropy(distribution: &DecisionDistribution) -> f64 {
    if distribution.total == 0 {
        return 0.0;
    }
    let mut entropy = 0.0;
    for count in distribution.counts.values() {
        if *count == 0 {
            continue;
        }
        let p = *count as f64 / distribution.total as f64;
        entropy -= p * p.log2();
    }
    entropy
}

pub fn compute_waiver_ratio(records: &[DecisionRecord]) -> f64 {
    if records.is_empty() {
        return 0.0;
    }
    let waivers = records.iter().filter(|r| r.waived).count();
    waivers as f64 / records.len() as f64
}

impl DecisionMetrics {
    pub fn from_records(records: &[DecisionRecord]) -> Self {
        let distribution = compute_distribution(records);
        let entropy = compute_entropy(&distribution);
        let waiver_ratio = compute_waiver_ratio(records);
        Self {
            distribution,
            entropy,
            waiver_ratio,
        }
    }
}
