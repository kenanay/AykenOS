// Constitutional Module: Hint Caching
// Cache must be deterministic, content-addressed, and tightening-only.
// No time-based eviction; no heuristic shortcuts.

use std::collections::BTreeMap;

use crate::arh::performance::PerfProfile;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum CacheScope {
    PatternMatch,
    ContextAnalysis,
    SemanticAssessment,
    FullArhOutput,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct CacheKey {
    pub scope: CacheScope,
    pub rule_id: String,
    pub violation_id: String,
    pub file_hash: String,
    pub config_fingerprint: String,
    pub latency_profile: PerfProfile,
}

#[derive(Default)]
pub struct HintCache<T: Clone> {
    entries: BTreeMap<CacheKey, T>,
}

impl<T: Clone> HintCache<T> {
    pub fn new() -> Self {
        Self { entries: BTreeMap::new() }
    }

    pub fn get(&self, key: &CacheKey) -> Option<T> {
        self.entries.get(key).cloned()
    }

    pub fn insert(&mut self, key: CacheKey, value: T) {
        self.entries.insert(key, value);
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }
}
