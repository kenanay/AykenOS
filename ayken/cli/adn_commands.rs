// Constitutional Module: ADN CLI Commands
// Read/write for ADN only; no decision influence.

//! CLI helper functions for ADN entries.

use crate::architecture::decision_notes::{
    AdnConstraints, AdnRecord, AdnStore, AdnStoreBackend, DecisionType,
};

pub fn adn_add(
    store: &mut AdnStore,
    decision_type: DecisionType,
    decision_id: &str,
    author: &str,
    timestamp: &str,
    rationale: &str,
    references: Vec<String>,
) -> Result<(), String> {
    let record = AdnRecord {
        adn_id: String::new(),
        decision_type,
        decision_id: decision_id.to_string(),
        author: author.to_string(),
        timestamp: timestamp.to_string(),
        rationale: rationale.to_string(),
        references,
    };
    store.append(record)
}

pub fn adn_list(store: &AdnStore) -> Vec<String> {
    store
        .entries()
        .iter()
        .map(|e| format!("{} {}", e.record.decision_id, e.record.adn_id))
        .collect()
}

pub fn adn_show(store: &AdnStore, decision_id: &str) -> Vec<String> {
    store
        .find_by_decision_id(decision_id)
        .iter()
        .map(|e| e.record.rationale.clone())
        .collect()
}

pub fn adn_store_memory() -> AdnStore {
    AdnStore::new(AdnStoreBackend::Memory, AdnConstraints::default())
}
