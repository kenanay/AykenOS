// Constitutional Module: Incremental Analysis
// Incremental logic may only invalidate; it must never approximate.

use std::collections::BTreeMap;

use crate::arh::performance::PerfProfile;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IncrementalState {
    pub file_hash: String,
    pub rule_id: String,
    pub latency_profile: PerfProfile,
    pub config_fingerprint: String,
    pub arh_output_hash: String,
}

#[derive(Default)]
pub struct IncrementalIndex {
    by_file: BTreeMap<String, BTreeMap<String, IncrementalState>>,
    output_changes: BTreeMap<String, usize>,
}

impl IncrementalIndex {
    pub fn new() -> Self {
        Self { by_file: BTreeMap::new(), output_changes: BTreeMap::new() }
    }

    pub fn should_recompute(
        &self,
        file_path: &str,
        file_hash: &str,
        violation_id: &str,
        rule_id: &str,
        config_fingerprint: &str,
        latency_profile: PerfProfile,
    ) -> bool {
        match self.by_file.get(file_path).and_then(|m| m.get(violation_id)) {
            Some(state) => {
                state.file_hash != file_hash
                    || state.rule_id != rule_id
                    || state.config_fingerprint != config_fingerprint
                    || state.latency_profile != latency_profile
            }
            None => true,
        }
    }

    pub fn update(
        &mut self,
        file_path: &str,
        file_hash: &str,
        violation_id: &str,
        rule_id: &str,
        config_fingerprint: &str,
        latency_profile: PerfProfile,
        arh_output_hash: &str,
    ) {
        let entry = IncrementalState {
            file_hash: file_hash.to_string(),
            rule_id: rule_id.to_string(),
            latency_profile,
            config_fingerprint: config_fingerprint.to_string(),
            arh_output_hash: arh_output_hash.to_string(),
        };
        if let Some(existing) = self
            .by_file
            .get(file_path)
            .and_then(|m| m.get(violation_id))
        {
            if existing.arh_output_hash != entry.arh_output_hash {
                *self.output_changes.entry(file_path.to_string()).or_insert(0) += 1;
            }
        }
        self.by_file
            .entry(file_path.to_string())
            .or_insert_with(BTreeMap::new)
            .insert(violation_id.to_string(), entry);
    }

    pub fn output_change_count(&self, file_path: &str) -> usize {
        *self.output_changes.get(file_path).unwrap_or(&0)
    }
}
