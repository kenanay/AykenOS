// Constitutional Module: Refactor Effectiveness Analytics
// Read-only analytics for outcome distribution.

//! Deterministic analytics for refactor outcome effectiveness.

use std::collections::BTreeMap;

use crate::arh::refactor_outcome::{Effectiveness, RefactorOutcomeRecord};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EffectivenessDistribution {
    pub counts: BTreeMap<Effectiveness, usize>,
    pub total: usize,
}

impl EffectivenessDistribution {
    pub fn ratio(&self, effectiveness: Effectiveness) -> f64 {
        if self.total == 0 {
            0.0
        } else {
            let count = *self.counts.get(&effectiveness).unwrap_or(&0);
            count as f64 / self.total as f64
        }
    }
}

pub fn distribution(records: &[RefactorOutcomeRecord]) -> EffectivenessDistribution {
    let mut counts: BTreeMap<Effectiveness, usize> = BTreeMap::new();
    for record in records {
        *counts.entry(record.effectiveness).or_insert(0) += 1;
    }
    let total = records.len();
    EffectivenessDistribution { counts, total }
}

pub fn effectiveness_by_module(records: &[RefactorOutcomeRecord]) -> BTreeMap<String, EffectivenessDistribution> {
    let mut map: BTreeMap<String, Vec<RefactorOutcomeRecord>> = BTreeMap::new();
    for record in records {
        for delta in &record.module_deltas {
            map.entry(delta.module_id.clone())
                .or_insert_with(Vec::new)
                .push(record.clone());
        }
    }

    let mut result: BTreeMap<String, EffectivenessDistribution> = BTreeMap::new();
    for (module_id, module_records) in map {
        result.insert(module_id, distribution(&module_records));
    }
    result
}
