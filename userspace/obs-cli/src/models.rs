use std::collections::BTreeMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Counts {
    pub partition_count: usize,
    pub total_nodes: usize,
    pub total_incidents: usize,
    pub agreement_count: usize,
    pub conflict_count: usize,
    pub island_count: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SnapshotFlags {
    pub produces_truth: bool,
    pub produces_decision: bool,
    pub produces_ranking: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Snapshot {
    pub summary_origin: String,
    pub authority_classification: String,
    pub display_mode: String,
    pub counts: Counts,
    pub flags: SnapshotFlags,
    pub incident_groups: BTreeMap<String, usize>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CountsDiff {
    // NOTE: counts are expected to be within i64 bounds (safe for current domain)
    pub partition_count: i64,
    pub total_nodes: i64,
    pub total_incidents: i64,
    pub agreement_count: i64,
    pub conflict_count: i64,
    pub island_count: i64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum IncidentGroupDelta {
    Added(usize),
    Removed(usize),
    Changed { baseline: usize, current: usize, delta: i64 },
    Unchanged(usize),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Diff {
    pub counts: CountsDiff,
    pub incident_groups: BTreeMap<String, IncidentGroupDelta>,
}
