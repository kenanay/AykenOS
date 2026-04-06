use crate::error::AppError;
use crate::models::Snapshot;

/// Serialize a Snapshot to canonical JSON bytes.
/// Uses serde_json::to_vec — BTreeMap in Snapshot guarantees lexicographic key order.
/// This is the single source of truth for --json output and --save-snapshot files.
///
/// Defense-in-depth: rejects snapshots that violate the epistemic boundary,
/// even if parser and formatter already enforce this.
pub fn to_canonical_json(snapshot: &Snapshot) -> Result<Vec<u8>, AppError> {
    if snapshot.flags.produces_truth
        || snapshot.flags.produces_decision
        || snapshot.flags.produces_ranking
    {
        return Err(AppError::Schema(
            "epistemic violation: snapshot flags must all be false".into(),
        ));
    }
    if snapshot.authority_classification != "non_authoritative" {
        return Err(AppError::Schema(
            "epistemic violation: authority_classification must be non_authoritative".into(),
        ));
    }
    serde_json::to_vec(snapshot)
        .map_err(|e| AppError::Io(format!("failed to serialize snapshot: {}", e)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Counts, Snapshot, SnapshotFlags};
    use crate::parser::parse_snapshot;
    use std::collections::BTreeMap;

    fn make_snapshot() -> Snapshot {
        let mut incident_groups = BTreeMap::new();
        incident_groups.insert("zebra".to_string(), 3);
        incident_groups.insert("alpha".to_string(), 1);
        incident_groups.insert("mango".to_string(), 2);

        Snapshot {
            summary_origin: "derived".to_string(),
            authority_classification: "non_authoritative".to_string(),
            display_mode: "machine_structured".to_string(),
            counts: Counts {
                partition_count: 3,
                total_nodes: 10,
                total_incidents: 2,
                agreement_count: 8,
                conflict_count: 1,
                island_count: 0,
            },
            flags: SnapshotFlags {
                produces_truth: false,
                produces_decision: false,
                produces_ranking: false,
            },
            incident_groups,
        }
    }

    #[test]
    fn test_valid_snapshot_serializes_without_error() {
        let snap = make_snapshot();
        let result = to_canonical_json(&snap);
        assert!(result.is_ok(), "expected Ok, got {:?}", result);
        let bytes = result.unwrap();
        assert!(!bytes.is_empty());
    }

    #[test]
    fn test_round_trip_produces_equal_snapshot() {
        // The parser requires `epistemic_boundary` as a top-level field, which is
        // not part of the Snapshot struct (it's validated but not stored).
        // Round-trip: parse raw JSON → serialize → parse again → assert equality.
        let raw = br#"{
            "summary_origin": "derived",
            "authority_classification": "non_authoritative",
            "display_mode": "machine_structured",
            "epistemic_boundary": {
                "produces_truth": false,
                "produces_decision": false,
                "produces_ranking": false
            },
            "counts": {
                "partition_count": 3,
                "total_nodes": 10,
                "total_incidents": 2,
                "agreement_count": 8,
                "conflict_count": 1,
                "island_count": 0
            },
            "flags": {
                "produces_truth": false,
                "produces_decision": false,
                "produces_ranking": false
            },
            "incident_groups": {
                "alpha": 1,
                "beta": 2
            }
        }"#;
        let first = parse_snapshot(raw).expect("first parse failed");
        let bytes = to_canonical_json(&first).expect("serialization failed");
        // The serialized form won't have epistemic_boundary, so we can't re-parse
        // through parse_snapshot. Instead verify the serialized snapshot re-parses
        // via serde directly (the struct-level round-trip).
        let restored: Snapshot = serde_json::from_slice(&bytes).expect("serde round-trip failed");
        assert_eq!(first, restored);
    }

    #[test]
    fn test_serialization_is_deterministic() {
        let snap = make_snapshot();
        let bytes1 = to_canonical_json(&snap).expect("first serialization failed");
        let bytes2 = to_canonical_json(&snap).expect("second serialization failed");
        assert_eq!(bytes1, bytes2, "serialization must be byte-identical across calls");
    }

    #[test]
    fn test_incident_groups_keys_in_lexicographic_order() {
        let snap = make_snapshot();
        let bytes = to_canonical_json(&snap).expect("serialization failed");
        let json_str = std::str::from_utf8(&bytes).expect("invalid utf8");

        // Find positions of the three keys in the JSON output
        let pos_alpha = json_str.find("\"alpha\"").expect("alpha not found");
        let pos_mango = json_str.find("\"mango\"").expect("mango not found");
        let pos_zebra = json_str.find("\"zebra\"").expect("zebra not found");

        assert!(
            pos_alpha < pos_mango && pos_mango < pos_zebra,
            "incident_groups keys must appear in lexicographic order: alpha < mango < zebra, \
             got positions alpha={}, mango={}, zebra={}",
            pos_alpha, pos_mango, pos_zebra
        );
    }
}
