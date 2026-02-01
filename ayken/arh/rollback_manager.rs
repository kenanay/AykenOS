// Constitutional Module: RollbackManager
// Rollback must restore the workspace to a byte-identical pre-apply state.
// Supports nested rollback scopes and idempotent restore.

//! Rollback controller for fix application.

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RollbackScope {
    pub module_id: String,
    pub snapshot_id: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SnapshotEntry {
    pub path: String,
    pub content: String,
}

pub trait WorkspaceIO {
    fn read_file(&self, path: &str) -> Result<String, String>;
    fn write_file(&self, path: &str, content: &str) -> Result<(), String>;
    fn reset_security_state(&self) -> Result<(), String>;
}

pub struct InMemoryWorkspaceIO {
    pub files: RefCell<HashMap<String, String>>,
}

impl InMemoryWorkspaceIO {
    pub fn new() -> Self {
        Self { files: RefCell::new(HashMap::new()) }
    }
}

impl WorkspaceIO for InMemoryWorkspaceIO {
    fn read_file(&self, path: &str) -> Result<String, String> {
        self.files
            .borrow()
            .get(path)
            .cloned()
            .ok_or_else(|| "File not found".to_string())
    }

    fn write_file(&self, path: &str, content: &str) -> Result<(), String> {
        self.files.borrow_mut().insert(path.to_string(), content.to_string());
        Ok(())
    }

    fn reset_security_state(&self) -> Result<(), String> {
        Ok(())
    }
}

pub struct RollbackManager {
    io: Box<dyn WorkspaceIO>,
    snapshots: RefCell<HashMap<String, Vec<SnapshotEntry>>>,
    counter: RefCell<u64>,
}

impl RollbackManager {
    pub fn new(io: Box<dyn WorkspaceIO>) -> Self {
        Self {
            io,
            snapshots: RefCell::new(HashMap::new()),
            counter: RefCell::new(0),
        }
    }

    pub fn begin_scope(&self, module_id: &str, files: &[String]) -> Result<RollbackScope, String> {
        let mut counter = self.counter.borrow_mut();
        let snapshot_id = format!("snapshot:{}:{}", module_id, *counter);
        *counter += 1;
        let mut entries = Vec::new();
        let mut seen: HashSet<String> = HashSet::new();
        for path in files {
            if !seen.insert(path.clone()) {
                continue;
            }
            let content = self.io.read_file(path)?;
            entries.push(SnapshotEntry { path: path.clone(), content });
        }
        // Store snapshot for rollback
        self.snapshots.borrow_mut().insert(snapshot_id.clone(), entries);
        Ok(RollbackScope {
            module_id: module_id.to_string(),
            snapshot_id,
        })
    }

    pub fn rollback(&self, scope: RollbackScope) -> Result<(), String> {
        let entries = match self.snapshots.borrow().get(&scope.snapshot_id) {
            Some(entries) => entries.clone(),
            None => return Err("snapshot not found".to_string()),
        };
        for entry in &entries {
            self.io.write_file(&entry.path, &entry.content)?;
        }
        self.io.reset_security_state()?;
        for entry in &entries {
            let restored = self.io.read_file(&entry.path)?;
            if restored != entry.content {
                return Err("restore verification failed".to_string());
            }
        }
        self.snapshots.borrow_mut().remove(&scope.snapshot_id);
        Ok(())
    }

    pub fn commit(&self, snapshot_id: &str) -> Result<(), String> {
        if self.snapshots.borrow_mut().remove(snapshot_id).is_none() {
            return Err("snapshot not found".to_string());
        }
        Ok(())
    }
}
