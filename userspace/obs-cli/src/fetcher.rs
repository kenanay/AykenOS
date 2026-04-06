use crate::error::AppError;
use std::io::Read;
use std::path::Path;

/// Fetch the machine_structured snapshot from proofd via HTTP GET.
/// Returns raw response body bytes on HTTP 200.
/// Non-200 responses → AppError::Http(status, body).
/// Connection failures / timeouts → AppError::Io.
pub fn fetch_from_proofd(addr: &str, timeout_ms: u64) -> Result<Vec<u8>, AppError> {
    let url = format!("{}/diagnostics/summary?display_mode=machine_structured", addr);
    let response = ureq::get(&url)
        .timeout(std::time::Duration::from_millis(timeout_ms))
        .call()
        .map_err(|e| AppError::Io(format!("connection failed: {}", e)))?;
    let status = response.status();
    if status != 200 {
        let body = response
            .into_string()
            .unwrap_or_else(|_| "<unreadable body>".to_string());
        return Err(AppError::Http(status, body));
    }
    let mut bytes = Vec::new();
    response
        .into_reader()
        .read_to_end(&mut bytes)
        .map_err(|e| AppError::Io(format!("failed to read response body: {}", e)))?;
    Ok(bytes)
}

/// Read a snapshot file from disk. Returns raw bytes.
/// File read failures → AppError::Io.
pub fn read_snapshot_file(path: &Path) -> Result<Vec<u8>, AppError> {
    std::fs::read(path)
        .map_err(|e| AppError::Io(format!("failed to read {}: {}", path.display(), e)))
}

/// Write snapshot bytes to a file.
/// Write failures → AppError::Io.
pub fn write_snapshot_file(path: &Path, data: &[u8]) -> Result<(), AppError> {
    std::fs::write(path, data)
        .map_err(|e| AppError::Io(format!("failed to write {}: {}", path.display(), e)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn read_nonexistent_file_returns_io_error() {
        let path = PathBuf::from("/tmp/obs-cli-test-nonexistent-file-xyz-12345.json");
        let result = read_snapshot_file(&path);
        assert!(result.is_err());
        match result.unwrap_err() {
            AppError::Io(_) => {}
            other => panic!("expected AppError::Io, got {:?}", other),
        }
    }

    #[test]
    fn write_then_read_round_trip() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("snap.bin");
        let data = b"hello obs-cli round-trip test";
        write_snapshot_file(&path, data).expect("write should succeed");
        let loaded = read_snapshot_file(&path).expect("read should succeed");
        assert_eq!(loaded, data);
    }

    #[test]
    fn write_to_unwritable_path_returns_io_error() {
        // /dev/null/impossible is a path that cannot be written to on any POSIX system
        let path = PathBuf::from("/dev/null/impossible");
        let result = write_snapshot_file(&path, b"data");
        assert!(result.is_err());
        match result.unwrap_err() {
            AppError::Io(_) => {}
            other => panic!("expected AppError::Io, got {:?}", other),
        }
    }

    // Feature: obs-cli-consumer, Property 12: snapshot save/load round-trip
    // Validates: Requirements 5.1, 5.3
    #[cfg(test)]
    mod prop_tests {
        use super::*;
        use crate::models::{Counts, Snapshot, SnapshotFlags};
        use crate::printer::to_canonical_json;
        use proptest::prelude::*;
        use std::collections::BTreeMap;

        fn arb_counts() -> impl Strategy<Value = Counts> {
            (
                0usize..1000,
                0usize..1000,
                0usize..1000,
                0usize..1000,
                0usize..1000,
                0usize..1000,
            )
                .prop_map(
                    |(partition_count, total_nodes, total_incidents, agreement_count, conflict_count, island_count)| {
                        Counts {
                            partition_count,
                            total_nodes,
                            total_incidents,
                            agreement_count,
                            conflict_count,
                            island_count,
                        }
                    },
                )
        }

        fn arb_incident_groups() -> impl Strategy<Value = BTreeMap<String, usize>> {
            // Keys must be non-empty and not parse as non-negative integers
            prop::collection::btree_map(
                "[a-z][a-z_]{1,15}",
                0usize..1000,
                0..5,
            )
        }

        fn arb_snapshot() -> impl Strategy<Value = Snapshot> {
            (arb_counts(), arb_incident_groups()).prop_map(|(counts, incident_groups)| Snapshot {
                summary_origin: "derived".to_string(),
                authority_classification: "non_authoritative".to_string(),
                display_mode: "machine_structured".to_string(),
                counts,
                flags: SnapshotFlags {
                    produces_truth: false,
                    produces_decision: false,
                    produces_ranking: false,
                },
                incident_groups,
            })
        }

        proptest! {
            // Feature: obs-cli-consumer, Property 12: snapshot save/load round-trip
            #[test]
            fn prop_snapshot_save_load_round_trip(snapshot in arb_snapshot()) {
                let dir = tempfile::tempdir().expect("tempdir");
                let path = dir.path().join("snap.json");
                let bytes = to_canonical_json(&snapshot).expect("serialization failed");
                write_snapshot_file(&path, &bytes).expect("write failed");
                let loaded_bytes = read_snapshot_file(&path).expect("read failed");
                // serde round-trip (canonical JSON → Snapshot struct)
                let loaded: Snapshot = serde_json::from_slice(&loaded_bytes)
                    .expect("deserialization failed");
                prop_assert_eq!(snapshot, loaded);
            }
        }
    }
}
