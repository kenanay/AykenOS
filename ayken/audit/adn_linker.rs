// Constitutional Module: ADN Linker
// Read-only linking between decisions and ADN entries.

//! Deterministic linker for decision ↔ ADN references.

use crate::architecture::decision_notes::{AdnEntry, DecisionType};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AdnLink {
    pub decision_type: DecisionType,
    pub decision_id: String,
    pub adn_id: String,
    pub timestamp: String,
}

pub fn link_entries(entries: &[AdnEntry]) -> Vec<AdnLink> {
    let mut links = Vec::new();
    for entry in entries {
        links.push(AdnLink {
            decision_type: entry.record.decision_type,
            decision_id: entry.record.decision_id.clone(),
            adn_id: entry.record.adn_id.clone(),
            timestamp: entry.record.timestamp.clone(),
        });
    }
    links
}

pub fn links_for_decision(entries: &[AdnEntry], decision_id: &str) -> Vec<AdnLink> {
    link_entries(entries)
        .into_iter()
        .filter(|l| l.decision_id == decision_id)
        .collect()
}
