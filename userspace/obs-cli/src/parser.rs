use crate::error::AppError;
use crate::models::Snapshot;

const REQUIRED_FIELDS: &[&str] = &[
    "summary_origin",
    "authority_classification",
    "display_mode",
    "epistemic_boundary",
    "counts",
    "flags",
    "incident_groups",
];

const COUNT_FIELDS: &[&str] = &[
    "partition_count",
    "total_nodes",
    "total_incidents",
    "agreement_count",
    "conflict_count",
    "island_count",
];

const FLAG_FIELDS: &[&str] = &["produces_truth", "produces_decision", "produces_ranking"];

pub fn parse_snapshot(raw: &[u8]) -> Result<Snapshot, AppError> {
    // Step 1: Parse raw bytes as serde_json::Value
    let value: serde_json::Value = serde_json::from_slice(raw)
        .map_err(|e| AppError::Parse(e.to_string()))?;

    // Step 2: Check all required top-level fields are present
    let obj = value
        .as_object()
        .ok_or_else(|| AppError::Parse("expected a JSON object".into()))?;

    for &field in REQUIRED_FIELDS {
        if !obj.contains_key(field) {
            return Err(AppError::Parse(format!("missing required field: {}", field)));
        }
    }

    // Step 3: Float rejection — counts fields
    let counts_obj = value["counts"]
        .as_object()
        .ok_or_else(|| AppError::Parse("counts must be a JSON object".into()))?;

    // Step 3a: counts completeness check
    for &field in COUNT_FIELDS {
        if !counts_obj.contains_key(field) {
            return Err(AppError::Parse(format!(
                "missing required counts field: {}",
                field
            )));
        }
    }

    // Step 3b: float rejection in counts
    for &field in COUNT_FIELDS {
        let v = &counts_obj[field];
        let n = v.as_number().ok_or_else(|| {
            AppError::Schema(format!("counts field must be a number: {}", field))
        })?;
        if !n.is_i64() && !n.is_u64() {
            return Err(AppError::Schema(format!(
                "float value not permitted in field: {}",
                field
            )));
        }
    }

    // Step 3c: Float rejection — incident_groups values (must be numbers, not floats)
    let groups_obj = value["incident_groups"]
        .as_object()
        .ok_or_else(|| AppError::Parse("incident_groups must be a JSON object".into()))?;

    for (key, v) in groups_obj {
        let n = v.as_number().ok_or_else(|| {
            AppError::Schema(format!(
                "incident_groups value must be a number: {}",
                key
            ))
        })?;
        if !n.is_i64() && !n.is_u64() {
            return Err(AppError::Schema(format!(
                "float value not permitted in field: {}",
                key
            )));
        }
    }

    // Step 4: Assert authority_classification == "non_authoritative"
    let auth = value["authority_classification"]
        .as_str()
        .ok_or_else(|| AppError::Parse("authority_classification must be a string".into()))?;
    if auth != "non_authoritative" {
        return Err(AppError::Schema(
            "authority_classification must be non_authoritative".into(),
        ));
    }

    // Step 4b: Assert summary_origin == "derived"
    let origin = value["summary_origin"]
        .as_str()
        .ok_or_else(|| AppError::Parse("summary_origin must be a string".into()))?;
    if origin != "derived" {
        return Err(AppError::Schema("summary_origin must be derived".into()));
    }

    // Step 5: Assert display_mode == "machine_structured"
    let display = value["display_mode"]
        .as_str()
        .ok_or_else(|| AppError::Parse("display_mode must be a string".into()))?;
    if display != "machine_structured" {
        return Err(AppError::Schema(
            "display_mode must be machine_structured".into(),
        ));
    }

    // Step 6: Assert all three flags are false
    let flags_obj = value["flags"]
        .as_object()
        .ok_or_else(|| AppError::Parse("flags must be a JSON object".into()))?;

    for &flag in FLAG_FIELDS {
        let flag_val = flags_obj
            .get(flag)
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        if flag_val {
            return Err(AppError::Schema(format!("flag {} must be false", flag)));
        }
    }

    // Step 6b: Validate epistemic_boundary object and its flags
    let eb_obj = value["epistemic_boundary"]
        .as_object()
        .ok_or_else(|| AppError::Schema("epistemic_boundary must be a JSON object".into()))?;

    for &flag in FLAG_FIELDS {
        let v = eb_obj
            .get(flag)
            .and_then(|v| v.as_bool())
            .ok_or_else(|| {
                AppError::Schema(format!("epistemic_boundary missing field: {}", flag))
            })?;
        if v {
            return Err(AppError::Schema(format!(
                "epistemic_boundary {} must be false",
                flag
            )));
        }
    }

    // Step 7: Validate incident_groups keys
    for key in groups_obj.keys() {
        if key.is_empty() {
            return Err(AppError::Schema(
                "incident_groups key must not be empty".into(),
            ));
        }
        if key.parse::<usize>().is_ok() {
            return Err(AppError::Schema(format!(
                "incident_groups key must not be numeric: {}",
                key
            )));
        }
    }

    // Step 8: Deserialize the validated Value into Snapshot
    let snapshot: Snapshot = serde_json::from_value(value)
        .map_err(|e| AppError::Parse(e.to_string()))?;

    // Step 9: Return Ok(snapshot)
    Ok(snapshot)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_json() -> &'static [u8] {
        br#"{
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
                "beta": 1
            }
        }"#
    }

    #[test]
    fn test_valid_body_parses_correctly() {
        let result = parse_snapshot(valid_json());
        assert!(result.is_ok(), "expected Ok, got {:?}", result);
        let snap = result.unwrap();
        assert_eq!(snap.authority_classification, "non_authoritative");
        assert_eq!(snap.counts.partition_count, 3);
        assert_eq!(snap.counts.total_nodes, 10);
        assert_eq!(snap.counts.total_incidents, 2);
        assert_eq!(snap.counts.agreement_count, 8);
        assert_eq!(snap.counts.conflict_count, 1);
        assert_eq!(snap.counts.island_count, 0);
        assert!(!snap.flags.produces_truth);
        assert!(!snap.flags.produces_decision);
        assert!(!snap.flags.produces_ranking);
        assert_eq!(snap.incident_groups["alpha"], 1);
        assert_eq!(snap.incident_groups["beta"], 1);
    }

    #[test]
    fn test_float_in_counts_returns_schema_error() {
        let raw = br#"{
            "summary_origin": "proofd",
            "authority_classification": "non_authoritative",
            "display_mode": "machine_structured",
            "epistemic_boundary": {"produces_truth": false, "produces_decision": false, "produces_ranking": false},
            "counts": {
                "partition_count": 3,
                "total_nodes": 10,
                "total_incidents": 2,
                "agreement_count": 8,
                "conflict_count": 1.5,
                "island_count": 0
            },
            "flags": {
                "produces_truth": false,
                "produces_decision": false,
                "produces_ranking": false
            },
            "incident_groups": {}
        }"#;
        let result = parse_snapshot(raw);
        assert!(matches!(result, Err(AppError::Schema(_))), "expected Schema error, got {:?}", result);
    }

    #[test]
    fn test_authority_classification_authoritative_returns_schema_error() {
        let raw = br#"{
            "summary_origin": "proofd",
            "authority_classification": "authoritative",
            "display_mode": "machine_structured",
            "epistemic_boundary": {"produces_truth": false, "produces_decision": false, "produces_ranking": false},
            "counts": {
                "partition_count": 0,
                "total_nodes": 0,
                "total_incidents": 0,
                "agreement_count": 0,
                "conflict_count": 0,
                "island_count": 0
            },
            "flags": {
                "produces_truth": false,
                "produces_decision": false,
                "produces_ranking": false
            },
            "incident_groups": {}
        }"#;
        let result = parse_snapshot(raw);
        assert!(matches!(result, Err(AppError::Schema(_))), "expected Schema error, got {:?}", result);
    }

    #[test]
    fn test_display_mode_human_readable_returns_schema_error() {
        let raw = br#"{
            "summary_origin": "proofd",
            "authority_classification": "non_authoritative",
            "display_mode": "human_readable",
            "epistemic_boundary": {"produces_truth": false, "produces_decision": false, "produces_ranking": false},
            "counts": {
                "partition_count": 0,
                "total_nodes": 0,
                "total_incidents": 0,
                "agreement_count": 0,
                "conflict_count": 0,
                "island_count": 0
            },
            "flags": {
                "produces_truth": false,
                "produces_decision": false,
                "produces_ranking": false
            },
            "incident_groups": {}
        }"#;
        let result = parse_snapshot(raw);
        assert!(matches!(result, Err(AppError::Schema(_))), "expected Schema error, got {:?}", result);
    }

    #[test]
    fn test_produces_truth_true_returns_schema_error() {
        let raw = br#"{
            "summary_origin": "proofd",
            "authority_classification": "non_authoritative",
            "display_mode": "machine_structured",
            "epistemic_boundary": {"produces_truth": false, "produces_decision": false, "produces_ranking": false},
            "counts": {
                "partition_count": 0,
                "total_nodes": 0,
                "total_incidents": 0,
                "agreement_count": 0,
                "conflict_count": 0,
                "island_count": 0
            },
            "flags": {
                "produces_truth": true,
                "produces_decision": false,
                "produces_ranking": false
            },
            "incident_groups": {}
        }"#;
        let result = parse_snapshot(raw);
        assert!(matches!(result, Err(AppError::Schema(_))), "expected Schema error, got {:?}", result);
    }

    #[test]
    fn test_missing_conflict_count_returns_parse_error_with_field_name() {
        let raw = br#"{
            "summary_origin": "proofd",
            "authority_classification": "non_authoritative",
            "display_mode": "machine_structured",
            "epistemic_boundary": {"produces_truth": false, "produces_decision": false, "produces_ranking": false},
            "counts": {
                "partition_count": 0,
                "total_nodes": 0,
                "total_incidents": 0,
                "agreement_count": 0,
                "island_count": 0
            },
            "flags": {
                "produces_truth": false,
                "produces_decision": false,
                "produces_ranking": false
            },
            "incident_groups": {}
        }"#;
        // conflict_count is missing from counts — serde will catch this at step 8
        // but we also need to check the step-8 deserialization catches it
        let result = parse_snapshot(raw);
        assert!(matches!(result, Err(AppError::Parse(_))), "expected Parse error, got {:?}", result);
    }

    #[test]
    fn test_invalid_json_bytes_returns_parse_error() {
        let raw = b"not valid json {{{";
        let result = parse_snapshot(raw);
        assert!(matches!(result, Err(AppError::Parse(_))), "expected Parse error, got {:?}", result);
    }

    #[test]
    fn test_empty_string_key_in_incident_groups_returns_schema_error() {
        let raw = br#"{
            "summary_origin": "proofd",
            "authority_classification": "non_authoritative",
            "display_mode": "machine_structured",
            "epistemic_boundary": {"produces_truth": false, "produces_decision": false, "produces_ranking": false},
            "counts": {
                "partition_count": 0,
                "total_nodes": 0,
                "total_incidents": 0,
                "agreement_count": 0,
                "conflict_count": 0,
                "island_count": 0
            },
            "flags": {
                "produces_truth": false,
                "produces_decision": false,
                "produces_ranking": false
            },
            "incident_groups": {
                "": 1
            }
        }"#;
        let result = parse_snapshot(raw);
        assert!(matches!(result, Err(AppError::Schema(_))), "expected Schema error, got {:?}", result);
    }

    #[test]
    fn test_numeric_string_key_in_incident_groups_returns_schema_error() {
        let raw = br#"{
            "summary_origin": "derived",
            "authority_classification": "non_authoritative",
            "display_mode": "machine_structured",
            "epistemic_boundary": {"produces_truth": false, "produces_decision": false, "produces_ranking": false},
            "counts": {
                "partition_count": 0,
                "total_nodes": 0,
                "total_incidents": 0,
                "agreement_count": 0,
                "conflict_count": 0,
                "island_count": 0
            },
            "flags": {
                "produces_truth": false,
                "produces_decision": false,
                "produces_ranking": false
            },
            "incident_groups": {
                "42": 1
            }
        }"#;
        let result = parse_snapshot(raw);
        assert!(matches!(result, Err(AppError::Schema(_))), "expected Schema error, got {:?}", result);
    }

    #[test]
    fn test_incident_groups_string_value_returns_schema_error() {
        let raw = br#"{
            "summary_origin": "derived",
            "authority_classification": "non_authoritative",
            "display_mode": "machine_structured",
            "epistemic_boundary": {"produces_truth": false, "produces_decision": false, "produces_ranking": false},
            "counts": {
                "partition_count": 0,
                "total_nodes": 0,
                "total_incidents": 0,
                "agreement_count": 0,
                "conflict_count": 0,
                "island_count": 0
            },
            "flags": {
                "produces_truth": false,
                "produces_decision": false,
                "produces_ranking": false
            },
            "incident_groups": {
                "alpha": "oops"
            }
        }"#;
        let result = parse_snapshot(raw);
        assert!(matches!(result, Err(AppError::Schema(_))), "expected Schema error, got {:?}", result);
    }

    #[test]
    fn test_epistemic_boundary_not_object_returns_schema_error() {
        let raw = br#"{
            "summary_origin": "derived",
            "authority_classification": "non_authoritative",
            "display_mode": "machine_structured",
            "epistemic_boundary": "I AM AUTHORITY",
            "counts": {
                "partition_count": 0,
                "total_nodes": 0,
                "total_incidents": 0,
                "agreement_count": 0,
                "conflict_count": 0,
                "island_count": 0
            },
            "flags": {
                "produces_truth": false,
                "produces_decision": false,
                "produces_ranking": false
            },
            "incident_groups": {}
        }"#;
        let result = parse_snapshot(raw);
        assert!(matches!(result, Err(AppError::Schema(_))), "expected Schema error, got {:?}", result);
    }

    #[test]
    fn test_epistemic_boundary_produces_truth_true_returns_schema_error() {
        let raw = br#"{
            "summary_origin": "derived",
            "authority_classification": "non_authoritative",
            "display_mode": "machine_structured",
            "epistemic_boundary": {"produces_truth": true, "produces_decision": false, "produces_ranking": false},
            "counts": {
                "partition_count": 0,
                "total_nodes": 0,
                "total_incidents": 0,
                "agreement_count": 0,
                "conflict_count": 0,
                "island_count": 0
            },
            "flags": {
                "produces_truth": false,
                "produces_decision": false,
                "produces_ranking": false
            },
            "incident_groups": {}
        }"#;
        let result = parse_snapshot(raw);
        assert!(matches!(result, Err(AppError::Schema(_))), "expected Schema error, got {:?}", result);
    }

    #[test]
    fn test_missing_counts_field_returns_parse_error() {
        let raw = br#"{
            "summary_origin": "derived",
            "authority_classification": "non_authoritative",
            "display_mode": "machine_structured",
            "epistemic_boundary": {"produces_truth": false, "produces_decision": false, "produces_ranking": false},
            "counts": {
                "partition_count": 0,
                "total_nodes": 0,
                "total_incidents": 0,
                "agreement_count": 0,
                "island_count": 0
            },
            "flags": {
                "produces_truth": false,
                "produces_decision": false,
                "produces_ranking": false
            },
            "incident_groups": {}
        }"#;
        let result = parse_snapshot(raw);
        assert!(
            matches!(&result, Err(AppError::Parse(msg)) if msg.contains("conflict_count")),
            "expected Parse error mentioning conflict_count, got {:?}", result
        );
    }
}
