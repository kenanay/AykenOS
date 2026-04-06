use crate::error::AppError;
use crate::models::Snapshot;

#[derive(Debug, Clone, PartialEq)]
pub enum CountField {
    PartitionCount,
    TotalNodes,
    TotalIncidents,
    AgreementCount,
    ConflictCount,
    IslandCount,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CompareOp {
    Gt,  // >
    Gte, // >=
    Lt,  // <
    Lte, // <=
    Eq,  // ==
}

#[derive(Debug, Clone, PartialEq)]
pub struct ThresholdCondition {
    pub field: CountField,
    pub op: CompareOp,
    pub value: usize,
}

/// Resolve a field name string to a CountField enum variant.
/// Returns None for unknown field names.
pub fn resolve_field(s: &str) -> Option<CountField> {
    match s {
        "partition_count" => Some(CountField::PartitionCount),
        "total_nodes" => Some(CountField::TotalNodes),
        "total_incidents" => Some(CountField::TotalIncidents),
        "agreement_count" => Some(CountField::AgreementCount),
        "conflict_count" => Some(CountField::ConflictCount),
        "island_count" => Some(CountField::IslandCount),
        _ => None,
    }
}

impl ThresholdCondition {
    /// Parse a condition string like "conflict_count > 0" or "total_incidents>=5".
    /// Trims whitespace around the operator. Resolves operators longest-match first
    /// (">=" before ">", "<=" before "<") to avoid misparse.
    pub fn parse(s: &str) -> Result<ThresholdCondition, AppError> {
        let trimmed = s.trim();

        // Resolve operator longest-match first to avoid ">=" being parsed as ">"
        let ops: &[(&str, CompareOp)] = &[
            (">=", CompareOp::Gte),
            ("<=", CompareOp::Lte),
            ("==", CompareOp::Eq),
            (">", CompareOp::Gt),
            ("<", CompareOp::Lt),
        ];

        let mut found: Option<(&str, &CompareOp, usize)> = None;
        for (op_str, op_variant) in ops {
            if let Some(pos) = trimmed.find(op_str) {
                found = Some((op_str, op_variant, pos));
                break;
            }
        }

        let (op_str, op_variant, op_pos) = match found {
            Some(f) => f,
            None => {
                return Err(AppError::Usage(format!(
                    "invalid condition: '{}'; expected: <field><op><value> where op is >, >=, <, <=, ==",
                    trimmed
                )));
            }
        };

        let field_part = trimmed[..op_pos].trim();
        let value_part = trimmed[op_pos + op_str.len()..].trim();

        let field = match resolve_field(field_part) {
            Some(f) => f,
            None => {
                return Err(AppError::Usage(format!(
                    "invalid condition: unknown field '{}'; valid fields: partition_count, total_nodes, total_incidents, agreement_count, conflict_count, island_count",
                    field_part
                )));
            }
        };

        let value: usize = value_part.parse().map_err(|_| {
            AppError::Usage(format!(
                "invalid condition: value '{}' is not a non-negative integer",
                value_part
            ))
        })?;

        Ok(ThresholdCondition {
            field,
            op: op_variant.clone(),
            value,
        })
    }

    /// Evaluate the condition against a snapshot.
    /// Returns true if the condition is violated (i.e. the condition holds).
    pub fn evaluate(&self, snapshot: &Snapshot) -> bool {
        let actual = match self.field {
            CountField::PartitionCount => snapshot.counts.partition_count,
            CountField::TotalNodes => snapshot.counts.total_nodes,
            CountField::TotalIncidents => snapshot.counts.total_incidents,
            CountField::AgreementCount => snapshot.counts.agreement_count,
            CountField::ConflictCount => snapshot.counts.conflict_count,
            CountField::IslandCount => snapshot.counts.island_count,
        };

        match self.op {
            CompareOp::Gt => actual > self.value,
            CompareOp::Gte => actual >= self.value,
            CompareOp::Lt => actual < self.value,
            CompareOp::Lte => actual <= self.value,
            CompareOp::Eq => actual == self.value,
        }
    }
}

fn field_name(field: &CountField) -> &'static str {
    match field {
        CountField::PartitionCount => "partition_count",
        CountField::TotalNodes => "total_nodes",
        CountField::TotalIncidents => "total_incidents",
        CountField::AgreementCount => "agreement_count",
        CountField::ConflictCount => "conflict_count",
        CountField::IslandCount => "island_count",
    }
}

fn op_str(op: &CompareOp) -> &'static str {
    match op {
        CompareOp::Gt => ">",
        CompareOp::Gte => ">=",
        CompareOp::Lt => "<",
        CompareOp::Lte => "<=",
        CompareOp::Eq => "==",
    }
}

/// Evaluate all conditions against a snapshot.
/// Returns Ok(()) if none are violated, Err(AppError::Threshold(violations)) otherwise.
pub fn evaluate_all(
    conditions: &[ThresholdCondition],
    snapshot: &Snapshot,
) -> Result<(), AppError> {
    let violations: Vec<String> = conditions
        .iter()
        .filter(|c| c.evaluate(snapshot))
        .map(|c| {
            let actual = match c.field {
                CountField::PartitionCount => snapshot.counts.partition_count,
                CountField::TotalNodes => snapshot.counts.total_nodes,
                CountField::TotalIncidents => snapshot.counts.total_incidents,
                CountField::AgreementCount => snapshot.counts.agreement_count,
                CountField::ConflictCount => snapshot.counts.conflict_count,
                CountField::IslandCount => snapshot.counts.island_count,
            };
            format!(
                "{}{}{} (observed: {})",
                field_name(&c.field),
                op_str(&c.op),
                c.value,
                actual
            )
        })
        .collect();

    if violations.is_empty() {
        Ok(())
    } else {
        Err(AppError::Threshold(violations))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Counts, Snapshot, SnapshotFlags};
    use std::collections::BTreeMap;

    fn make_snapshot(counts: Counts) -> Snapshot {
        Snapshot {
            summary_origin: "test".to_string(),
            authority_classification: "non_authoritative".to_string(),
            display_mode: "machine_structured".to_string(),
            counts,
            flags: SnapshotFlags {
                produces_truth: false,
                produces_decision: false,
                produces_ranking: false,
            },
            incident_groups: BTreeMap::new(),
        }
    }

    fn default_counts() -> Counts {
        Counts {
            partition_count: 1,
            total_nodes: 10,
            total_incidents: 3,
            agreement_count: 2,
            conflict_count: 1,
            island_count: 0,
        }
    }

    #[test]
    fn parse_conflict_count_gt_0() {
        let cond = ThresholdCondition::parse("conflict_count>0").unwrap();
        assert_eq!(cond.field, CountField::ConflictCount);
        assert_eq!(cond.op, CompareOp::Gt);
        assert_eq!(cond.value, 0);
    }

    #[test]
    fn parse_total_incidents_gte_5() {
        let cond = ThresholdCondition::parse("total_incidents>=5").unwrap();
        assert_eq!(cond.field, CountField::TotalIncidents);
        assert_eq!(cond.op, CompareOp::Gte);
        assert_eq!(cond.value, 5);
    }

    #[test]
    fn parse_with_spaces_same_as_without() {
        let with_spaces = ThresholdCondition::parse("conflict_count > 0").unwrap();
        let without_spaces = ThresholdCondition::parse("conflict_count>0").unwrap();
        assert_eq!(with_spaces, without_spaces);
    }

    #[test]
    fn parse_unknown_field_returns_usage_error() {
        let result = ThresholdCondition::parse("unknown_field>0");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.exit_code(), 1);
        assert!(matches!(err, AppError::Usage(_)));
    }

    #[test]
    fn parse_double_gt_operator_returns_usage_error() {
        // "conflict_count>>0" — after splitting on ">", field="conflict_count", value=">0"
        // ">0" fails to parse as usize → Usage error
        let result = ThresholdCondition::parse("conflict_count>>0");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.exit_code(), 1);
        assert!(matches!(err, AppError::Usage(_)));
    }

    #[test]
    fn parse_negative_value_returns_usage_error() {
        // "-1" cannot parse as usize
        let result = ThresholdCondition::parse("conflict_count>=-1");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.exit_code(), 1);
        assert!(matches!(err, AppError::Usage(_)));
    }

    #[test]
    fn evaluate_all_no_violations_returns_ok() {
        let snapshot = make_snapshot(default_counts());
        // conflict_count is 1, condition: conflict_count > 5 → not violated
        let conditions = vec![ThresholdCondition {
            field: CountField::ConflictCount,
            op: CompareOp::Gt,
            value: 5,
        }];
        assert!(evaluate_all(&conditions, &snapshot).is_ok());
    }

    #[test]
    fn evaluate_all_one_violation_returns_err() {
        let snapshot = make_snapshot(default_counts());
        // conflict_count is 1, condition: conflict_count > 0 → violated
        let conditions = vec![ThresholdCondition {
            field: CountField::ConflictCount,
            op: CompareOp::Gt,
            value: 0,
        }];
        let result = evaluate_all(&conditions, &snapshot);
        assert!(result.is_err());
        match result.unwrap_err() {
            AppError::Threshold(violations) => {
                assert_eq!(violations.len(), 1);
                assert!(violations[0].contains("conflict_count>0"));
                assert!(violations[0].contains("observed: 1"));
            }
            other => panic!("expected Threshold, got {:?}", other),
        }
    }

    #[test]
    fn evaluate_all_multiple_violations_all_reported() {
        let snapshot = make_snapshot(default_counts());
        // conflict_count=1 > 0 → violated
        // total_incidents=3 >= 1 → violated
        // island_count=0 > 10 → not violated
        let conditions = vec![
            ThresholdCondition {
                field: CountField::ConflictCount,
                op: CompareOp::Gt,
                value: 0,
            },
            ThresholdCondition {
                field: CountField::TotalIncidents,
                op: CompareOp::Gte,
                value: 1,
            },
            ThresholdCondition {
                field: CountField::IslandCount,
                op: CompareOp::Gt,
                value: 10,
            },
        ];
        let result = evaluate_all(&conditions, &snapshot);
        assert!(result.is_err());
        match result.unwrap_err() {
            AppError::Threshold(violations) => {
                assert_eq!(violations.len(), 2);
            }
            other => panic!("expected Threshold, got {:?}", other),
        }
    }

    #[test]
    fn evaluate_all_empty_conditions_returns_ok() {
        let snapshot = make_snapshot(default_counts());
        assert!(evaluate_all(&[], &snapshot).is_ok());
    }
}
