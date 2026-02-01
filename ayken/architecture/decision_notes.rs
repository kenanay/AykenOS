// Constitutional Module: Architectural Decision Notes (ADN)
// Records intent only; never authority. Append-only, audit-grade.

//! Human rationale records for Waiver / Refactor / AssistedFix decisions.

use std::fs::OpenOptions;
use std::io::Write;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum DecisionType {
    Waiver,
    Refactor,
    AssistedFix,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AdnRecord {
    pub adn_id: String,
    pub decision_type: DecisionType,
    pub decision_id: String,
    pub author: String,
    pub timestamp: String,
    pub rationale: String,
    pub references: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AdnEntry {
    pub record: AdnRecord,
    pub prev_hash: String,
    pub hash: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AdnStoreBackend {
    Memory,
    AppendOnlyFile { path: String },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AdnConstraints {
    pub max_rationale_chars: usize,
    pub max_reference_count: usize,
}

impl AdnConstraints {
    pub fn default() -> Self {
        Self {
            max_rationale_chars: 1200,
            max_reference_count: 8,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct AdnStore {
    entries: Vec<AdnEntry>,
    backend: AdnStoreBackend,
    constraints: AdnConstraints,
}

impl AdnStore {
    pub fn new(backend: AdnStoreBackend, constraints: AdnConstraints) -> Self {
        Self {
            entries: Vec::new(),
            backend,
            constraints,
        }
    }

    pub fn append(&mut self, mut record: AdnRecord) -> Result<(), String> {
        self.validate(&record)?;
        record.adn_id = compute_adn_id(&record)?;

        let prev_hash = self
            .entries
            .last()
            .map(|e| e.hash.clone())
            .unwrap_or_else(|| "GENESIS".to_string());
        let hash = hash_entry(&record, &prev_hash)?;
        let entry = AdnEntry {
            record,
            prev_hash,
            hash,
        };
        self.persist(&entry)?;
        self.entries.push(entry);
        Ok(())
    }

    pub fn entries(&self) -> &[AdnEntry] {
        &self.entries
    }

    pub fn find_by_decision_id(&self, decision_id: &str) -> Vec<&AdnEntry> {
        self.entries
            .iter()
            .filter(|e| e.record.decision_id == decision_id)
            .collect()
    }

    fn validate(&self, record: &AdnRecord) -> Result<(), String> {
        if record.rationale.trim().is_empty() {
            return Err("ADN rationale cannot be empty".to_string());
        }
        if record.rationale.chars().count() > self.constraints.max_rationale_chars {
            return Err("ADN rationale exceeds length limit".to_string());
        }
        if record.references.len() > self.constraints.max_reference_count {
            return Err("ADN reference count exceeds limit".to_string());
        }
        Ok(())
    }

    fn persist(&self, entry: &AdnEntry) -> Result<(), String> {
        match &self.backend {
            AdnStoreBackend::Memory => Ok(()),
            AdnStoreBackend::AppendOnlyFile { path } => {
                let mut file = OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(path)
                    .map_err(|e| format!("ADN store open failed: {}", e))?;
                let line = serde_json::to_string(entry)
                    .map_err(|e| format!("ADN serialize failed: {}", e))?;
                file.write_all(line.as_bytes())
                    .and_then(|_| file.write_all(b"\n"))
                    .map_err(|e| format!("ADN write failed: {}", e))?;
                Ok(())
            }
        }
    }
}

fn compute_adn_id(record: &AdnRecord) -> Result<String, String> {
    let payload = serde_json::to_string(&(
        record.decision_type,
        &record.decision_id,
        &record.author,
        &record.timestamp,
        &record.rationale,
        &record.references,
    ))
    .map_err(|e| format!("ADN id serialize failed: {}", e))?;
    let mut hasher = Sha256::new();
    hasher.update(payload.as_bytes());
    Ok(hex::encode(hasher.finalize()))
}

fn hash_entry(record: &AdnRecord, prev_hash: &str) -> Result<String, String> {
    let payload = serde_json::to_string(&(record, prev_hash))
        .map_err(|e| format!("ADN hash serialize failed: {}", e))?;
    let mut hasher = Sha256::new();
    hasher.update(payload.as_bytes());
    Ok(hex::encode(hasher.finalize()))
}
