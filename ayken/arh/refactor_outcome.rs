// Constitutional Module: Refactor Outcome
// Analysis-only. Must not mutate code or influence decisions directly.

//! Deterministic refactor outcome evaluation (audit-grade, rule-based).

use std::collections::BTreeMap;
use std::fs::OpenOptions;
use std::io::Write;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AhsSnapshot {
    pub score: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MarsModuleScore {
    pub module_id: String,
    pub score: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RefactorScope {
    pub refactor_id: String,
    pub module_ids: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RefactorOutcomeInput {
    pub scope: RefactorScope,
    pub ahs_before: AhsSnapshot,
    pub ahs_after: AhsSnapshot,
    pub mars_before: Vec<MarsModuleScore>,
    pub mars_after: Vec<MarsModuleScore>,
    pub timestamp: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Effectiveness {
    Positive,
    Neutral,
    Negative,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModuleDeltaKind {
    Observed,
    MissingAfter,
    NewAfter,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ModuleDelta {
    pub module_id: String,
    pub before: f64,
    pub after: f64,
    pub delta: f64,
    pub kind: ModuleDeltaKind,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RefactorOutcomeRecord {
    pub refactor_id: String,
    pub timestamp: String,
    pub ahs_before: f64,
    pub ahs_after: f64,
    pub ahs_delta: f64,
    pub module_deltas: Vec<ModuleDelta>,
    pub effectiveness: Effectiveness,
    pub applied_rules: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OutcomeRules {
    pub ahs_positive_threshold: f64,
    pub ahs_negative_threshold: f64,
    pub allow_module_regression: bool,
}

impl OutcomeRules {
    pub fn default() -> Self {
        Self {
            ahs_positive_threshold: 0.05,
            ahs_negative_threshold: -0.05,
            allow_module_regression: false,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct RefactorOutcomeEngine {
    pub rules: OutcomeRules,
}

impl RefactorOutcomeEngine {
    pub fn new(rules: OutcomeRules) -> Self {
        Self { rules }
    }

    pub fn evaluate(&self, input: RefactorOutcomeInput) -> RefactorOutcomeRecord {
        let ahs_delta = input.ahs_after.score - input.ahs_before.score;
        let module_deltas = compute_module_deltas(
            &input.scope.module_ids,
            &input.mars_before,
            &input.mars_after,
        );
        let has_regression = module_deltas.iter().any(|d| {
            d.kind == ModuleDeltaKind::MissingAfter || d.delta < 0.0
        });

        let effectiveness = if ahs_delta >= self.rules.ahs_positive_threshold
            && (self.rules.allow_module_regression || !has_regression)
        {
            Effectiveness::Positive
        } else if ahs_delta <= self.rules.ahs_negative_threshold
            || (!self.rules.allow_module_regression && has_regression)
        {
            Effectiveness::Negative
        } else {
            Effectiveness::Neutral
        };

        let mut applied_rules = Vec::new();
        applied_rules.push(format!(
            "AHS thresholds: [{:.3}, {:.3}]",
            self.rules.ahs_negative_threshold, self.rules.ahs_positive_threshold
        ));
        applied_rules.push(format!(
            "Module regression allowed: {}",
            self.rules.allow_module_regression
        ));
        applied_rules.push("Module scope: refactor scope only".to_string());

        RefactorOutcomeRecord {
            refactor_id: input.scope.refactor_id,
            timestamp: input.timestamp,
            ahs_before: input.ahs_before.score,
            ahs_after: input.ahs_after.score,
            ahs_delta,
            module_deltas,
            effectiveness,
            applied_rules,
        }
    }
}

pub fn compute_module_deltas(
    scope_modules: &[String],
    before: &[MarsModuleScore],
    after: &[MarsModuleScore],
) -> Vec<ModuleDelta> {
    let mut scope_set: BTreeMap<String, ()> = BTreeMap::new();
    for module_id in scope_modules {
        scope_set.insert(module_id.clone(), ());
    }

    let mut after_map: BTreeMap<String, f64> = BTreeMap::new();
    for entry in after {
        after_map.insert(entry.module_id.clone(), entry.score);
    }

    let mut deltas = Vec::new();
    for entry in before {
        if !scope_set.contains_key(&entry.module_id) {
            continue;
        }
        if let Some(after_score) = after_map.get(&entry.module_id) {
            deltas.push(ModuleDelta {
                module_id: entry.module_id.clone(),
                before: entry.score,
                after: *after_score,
                delta: *after_score - entry.score,
                kind: ModuleDeltaKind::Observed,
            });
        } else {
            deltas.push(ModuleDelta {
                module_id: entry.module_id.clone(),
                before: entry.score,
                after: entry.score,
                delta: 0.0,
                kind: ModuleDeltaKind::MissingAfter,
            });
        }
    }
    for entry in after {
        if !scope_set.contains_key(&entry.module_id) {
            continue;
        }
        if !before.iter().any(|b| b.module_id == entry.module_id) {
            deltas.push(ModuleDelta {
                module_id: entry.module_id.clone(),
                before: entry.score,
                after: entry.score,
                delta: 0.0,
                kind: ModuleDeltaKind::NewAfter,
            });
        }
    }
    deltas
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OutcomeEntry {
    pub record: RefactorOutcomeRecord,
    pub prev_hash: String,
    pub hash: String,
}

#[derive(Clone, Debug, PartialEq)]
pub enum OutcomeStoreBackend {
    Memory,
    AppendOnlyFile { path: String },
}

#[derive(Clone, Debug, PartialEq)]
pub struct OutcomeStore {
    entries: Vec<OutcomeEntry>,
    backend: OutcomeStoreBackend,
}

impl OutcomeStore {
    pub fn new(backend: OutcomeStoreBackend) -> Self {
        Self {
            entries: Vec::new(),
            backend,
        }
    }

    pub fn append(&mut self, record: RefactorOutcomeRecord) -> Result<(), String> {
        let prev_hash = self
            .entries
            .last()
            .map(|e| e.hash.clone())
            .unwrap_or_else(|| "GENESIS".to_string());
        let hash = hash_record(&record, &prev_hash)?;
        let entry = OutcomeEntry {
            record,
            prev_hash,
            hash,
        };
        self.persist(&entry)?;
        self.entries.push(entry);
        Ok(())
    }

    pub fn entries(&self) -> &[OutcomeEntry] {
        &self.entries
    }

    fn persist(&self, entry: &OutcomeEntry) -> Result<(), String> {
        match &self.backend {
            OutcomeStoreBackend::Memory => Ok(()),
            OutcomeStoreBackend::AppendOnlyFile { path } => {
                let mut file = OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(path)
                    .map_err(|e| format!("outcome store open failed: {}", e))?;
                let line = serde_json::to_string(entry)
                    .map_err(|e| format!("outcome store serialize failed: {}", e))?;
                file.write_all(line.as_bytes())
                    .and_then(|_| file.write_all(b"\n"))
                    .map_err(|e| format!("outcome store write failed: {}", e))?;
                Ok(())
            }
        }
    }
}

fn hash_record(record: &RefactorOutcomeRecord, prev_hash: &str) -> Result<String, String> {
    let payload = serde_json::to_string(&(record, prev_hash))
        .map_err(|e| format!("outcome hash serialize failed: {}", e))?;
    let mut hasher = Sha256::new();
    hasher.update(payload.as_bytes());
    Ok(hex::encode(hasher.finalize()))
}
