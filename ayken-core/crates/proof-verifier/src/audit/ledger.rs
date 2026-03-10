use crate::audit::schema::build_audit_event;
use crate::audit::verify::verify_audit_ledger;
use crate::canonical::jcs::canonicalize_json;
use crate::errors::VerifierRuntimeError;
use crate::types::{
    VerdictSubject, VerificationAuditEvent, VerificationReceipt, VerificationVerdict,
};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;

pub fn append_verification_audit_event(
    ledger_path: &Path,
    subject: &VerdictSubject,
    verdict: VerificationVerdict,
    receipt: &VerificationReceipt,
) -> Result<VerificationAuditEvent, VerifierRuntimeError> {
    if receipt.verifier_signature_algorithm.is_none() || receipt.verifier_signature.is_none() {
        return Err(VerifierRuntimeError::config(
            "audit append requires a signed verification receipt",
        ));
    }

    let _lock = AuditLedgerLock::acquire(ledger_path)?;

    if ledger_path.exists() {
        let findings = verify_audit_ledger(ledger_path)?;
        if findings
            .iter()
            .any(|finding| matches!(finding.severity, crate::types::FindingSeverity::Error))
        {
            return Err(VerifierRuntimeError::config(
                "existing audit ledger failed integrity verification before append",
            ));
        }
    }

    let existing_events = load_audit_events(ledger_path)?;
    let previous_event_hash = existing_events.last().map(|event| event.event_id.clone());
    let event = build_audit_event(subject, verdict, receipt, previous_event_hash)?;
    append_event(ledger_path, &event)?;
    Ok(event)
}

pub fn append_event(
    ledger_path: &Path,
    event: &VerificationAuditEvent,
) -> Result<(), VerifierRuntimeError> {
    if let Some(parent) = ledger_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| VerifierRuntimeError::io("create audit ledger directory", error))?;
    }

    let bytes = canonicalize_json(event)?;
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(ledger_path)
        .map_err(|error| VerifierRuntimeError::io("open audit ledger", error))?;
    file.write_all(&bytes)
        .map_err(|error| VerifierRuntimeError::io("append audit event", error))?;
    file.sync_data()
        .map_err(|error| VerifierRuntimeError::io("sync audit ledger", error))
}

pub fn load_audit_events(
    ledger_path: &Path,
) -> Result<Vec<VerificationAuditEvent>, VerifierRuntimeError> {
    if !ledger_path.exists() {
        return Ok(Vec::new());
    }

    let raw = fs::read_to_string(ledger_path)
        .map_err(|error| VerifierRuntimeError::io("read audit ledger", error))?;
    let mut events = Vec::new();
    for line in raw.lines().filter(|line| !line.trim().is_empty()) {
        let event = serde_json::from_str(line)
            .map_err(|error| VerifierRuntimeError::json("parse audit ledger event", error))?;
        events.push(event);
    }
    Ok(events)
}

struct AuditLedgerLock {
    path: PathBuf,
}

impl AuditLedgerLock {
    fn acquire(ledger_path: &Path) -> Result<Self, VerifierRuntimeError> {
        let path = lock_path_for(ledger_path)?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|error| {
                VerifierRuntimeError::io("create audit ledger lock directory", error)
            })?;
        }
        for _ in 0..200 {
            match OpenOptions::new().write(true).create_new(true).open(&path) {
                Ok(_) => return Ok(Self { path }),
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                    thread::sleep(Duration::from_millis(10));
                }
                Err(error) => {
                    return Err(VerifierRuntimeError::io("acquire audit ledger lock", error));
                }
            }
        }

        Err(VerifierRuntimeError::config(
            "timed out acquiring audit ledger append lock",
        ))
    }
}

impl Drop for AuditLedgerLock {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

fn lock_path_for(ledger_path: &Path) -> Result<PathBuf, VerifierRuntimeError> {
    let file_name = ledger_path.file_name().ok_or_else(|| {
        VerifierRuntimeError::config("audit ledger path must include a file name")
    })?;
    let mut lock_name = file_name.to_os_string();
    lock_name.push(".lock");
    Ok(ledger_path.with_file_name(lock_name))
}
