// Constitutional Module: Parallel Processing
// Parallelism must not change output ordering or results.
// Kernel paths must remain serial.

use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorkItem {
    pub key: String,
    pub is_kernel: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorkResult {
    pub key: String,
    pub output: String,
}

pub struct ParallelProcessor;

const DUP_KEY: &str = "duplicate work key detected";

impl ParallelProcessor {
    pub fn process_serial<F>(&self, items: &[WorkItem], mut f: F) -> Vec<WorkResult>
    where
        F: FnMut(&WorkItem) -> WorkResult,
    {
        items.iter().map(|i| f(i)).collect()
    }

    /// Deterministic processing: results are ordered by key.
    /// Requirement: f must be pure with respect to the item (no dependence on call order).
    pub fn process_deterministic<F>(&self, items: &[WorkItem], mut f: F) -> Result<Vec<WorkResult>, String>
    where
        F: FnMut(&WorkItem) -> WorkResult,
    {
        let mut map: BTreeMap<String, WorkResult> = BTreeMap::new();
        for item in items {
            let result = f(item);
            match map.entry(result.key.clone()) {
                std::collections::btree_map::Entry::Vacant(entry) => {
                    entry.insert(result);
                }
                std::collections::btree_map::Entry::Occupied(_) => {
                    return Err(DUP_KEY.to_string());
                }
            }
        }
        Ok(map.into_values().collect())
    }

    /// Kernel items are processed serially; final ordering is deterministic by key.
    /// Requirement: f must be pure with respect to the item (no dependence on call order).
    pub fn process_with_kernel_guard<F>(&self, items: &[WorkItem], mut f: F) -> Result<Vec<WorkResult>, String>
    where
        F: FnMut(&WorkItem) -> WorkResult,
    {
        let mut map: BTreeMap<String, WorkResult> = BTreeMap::new();

        for item in items.iter().filter(|i| i.is_kernel) {
            let result = f(item);
            match map.entry(result.key.clone()) {
                std::collections::btree_map::Entry::Vacant(entry) => {
                    entry.insert(result);
                }
                std::collections::btree_map::Entry::Occupied(_) => {
                    return Err(DUP_KEY.to_string());
                }
            }
        }

        for item in items.iter().filter(|i| !i.is_kernel) {
            let result = f(item);
            match map.entry(result.key.clone()) {
                std::collections::btree_map::Entry::Vacant(entry) => {
                    entry.insert(result);
                }
                std::collections::btree_map::Entry::Occupied(_) => {
                    return Err(DUP_KEY.to_string());
                }
            }
        }

        Ok(map.into_values().collect())
    }
}
