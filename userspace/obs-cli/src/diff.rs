use std::collections::BTreeMap;

use crate::models::{Diff, CountsDiff, IncidentGroupDelta, Snapshot};
use crate::formatter::FORBIDDEN;

/// Compute a field-by-field diff between two snapshots.
/// All arithmetic is signed integer subtraction: current - baseline.
/// No floats; no unwrap/expect.
pub fn compute_diff(baseline: &Snapshot, current: &Snapshot) -> Diff {
    let counts = CountsDiff {
        partition_count: current.counts.partition_count as i64
            - baseline.counts.partition_count as i64,
        total_nodes: current.counts.total_nodes as i64
            - baseline.counts.total_nodes as i64,
        total_incidents: current.counts.total_incidents as i64
            - baseline.counts.total_incidents as i64,
        agreement_count: current.counts.agreement_count as i64
            - baseline.counts.agreement_count as i64,
        conflict_count: current.counts.conflict_count as i64
            - baseline.counts.conflict_count as i64,
        island_count: current.counts.island_count as i64
            - baseline.counts.island_count as i64,
    };

    // BTreeMap guarantees lexicographic key order in the output.
    let mut incident_groups: BTreeMap<String, IncidentGroupDelta> = BTreeMap::new();

    // Keys present in current
    for (key, &cur_count) in &current.incident_groups {
        match baseline.incident_groups.get(key) {
            Some(&base_count) => {
                if cur_count == base_count {
                    incident_groups.insert(key.clone(), IncidentGroupDelta::Unchanged(cur_count));
                } else {
                    let delta = cur_count as i64 - base_count as i64;
                    incident_groups.insert(
                        key.clone(),
                        IncidentGroupDelta::Changed {
                            baseline: base_count,
                            current: cur_count,
                            delta,
                        },
                    );
                }
            }
            None => {
                incident_groups.insert(key.clone(), IncidentGroupDelta::Added(cur_count));
            }
        }
    }

    // Keys only in baseline (removed)
    for (key, &base_count) in &baseline.incident_groups {
        if !current.incident_groups.contains_key(key) {
            incident_groups.insert(key.clone(), IncidentGroupDelta::Removed(base_count));
        }
    }

    Diff { counts, incident_groups }
}

/// Format a delta value: "+n" for positive, "-n" for negative, "0" for zero.
fn fmt_delta(d: i64) -> String {
    if d > 0 {
        format!("+{}", d)
    } else if d < 0 {
        format!("{}", d) // i64 Display already includes the '-'
    } else {
        "0".to_string()
    }
}

/// Format a Diff as a human-readable table.
/// Columns: field, baseline, current, delta.
/// Incident groups section sorted lexicographically (guaranteed by BTreeMap).
/// No forbidden vocabulary; deterministic output.
pub fn format_diff(diff: &Diff) -> String {
    let mut out = String::new();

    out.push_str("diff:\n");
    out.push_str(&format!(
        "  {:<22} {:<12} {:<12} {}\n",
        "field", "baseline", "current", "delta"
    ));

    // CountsDiff only stores the delta (not the original baseline/current values).
    // We emit the delta in the delta column and "—" for baseline/current (not available).
    out.push_str(&format!(
        "  {:<22} {:<12} {:<12} {}\n",
        "partition_count", "—", "—", fmt_delta(diff.counts.partition_count)
    ));
    out.push_str(&format!(
        "  {:<22} {:<12} {:<12} {}\n",
        "total_nodes", "—", "—", fmt_delta(diff.counts.total_nodes)
    ));
    out.push_str(&format!(
        "  {:<22} {:<12} {:<12} {}\n",
        "total_incidents", "—", "—", fmt_delta(diff.counts.total_incidents)
    ));
    out.push_str(&format!(
        "  {:<22} {:<12} {:<12} {}\n",
        "agreement_count", "—", "—", fmt_delta(diff.counts.agreement_count)
    ));
    out.push_str(&format!(
        "  {:<22} {:<12} {:<12} {}\n",
        "conflict_count", "—", "—", fmt_delta(diff.counts.conflict_count)
    ));
    out.push_str(&format!(
        "  {:<22} {:<12} {:<12} {}\n",
        "island_count", "—", "—", fmt_delta(diff.counts.island_count)
    ));

    out.push('\n');
    out.push_str("incident_groups diff:\n");

    if diff.incident_groups.is_empty() {
        out.push_str("  no changes\n");
    } else {
        // BTreeMap guarantees lexicographic order — no sort needed
        for (key, delta) in &diff.incident_groups {
            let line = match delta {
                IncidentGroupDelta::Added(count) => {
                    format!("  {}: added ({})\n", key, count)
                }
                IncidentGroupDelta::Removed(count) => {
                    format!("  {}: removed ({})\n", key, count)
                }
                IncidentGroupDelta::Changed { baseline, current, delta: d } => {
                    format!(
                        "  {}: {} -> {} ({})\n",
                        key,
                        baseline,
                        current,
                        fmt_delta(*d)
                    )
                }
                IncidentGroupDelta::Unchanged(count) => {
                    format!("  {}: {} (unchanged)\n", key, count)
                }
            };
            out.push_str(&line);
        }
    }

    // Runtime forbidden semantics guard
    for &word in FORBIDDEN {
        if crate::formatter::contains_forbidden_word(&out, word) {
            return format!("[invalid output: forbidden semantics detected: {}]", word);
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Counts, SnapshotFlags};
    use std::collections::BTreeMap;

    fn make_snapshot(
        partition_count: usize,
        total_nodes: usize,
        total_incidents: usize,
        agreement_count: usize,
        conflict_count: usize,
        island_count: usize,
        incident_groups: BTreeMap<String, usize>,
    ) -> Snapshot {
        Snapshot {
            summary_origin: "derived".to_string(),
            authority_classification: "non_authoritative".to_string(),
            display_mode: "machine_structured".to_string(),
            counts: Counts {
                partition_count,
                total_nodes,
                total_incidents,
                agreement_count,
                conflict_count,
                island_count,
            },
            flags: SnapshotFlags {
                produces_truth: false,
                produces_decision: false,
                produces_ranking: false,
            },
            incident_groups,
        }
    }

    fn base_snapshot() -> Snapshot {
        make_snapshot(1, 2, 3, 4, 5, 6, BTreeMap::new())
    }

    // --- compute_diff tests ---

    #[test]
    fn identical_snapshots_all_deltas_zero() {
        let snap = base_snapshot();
        let diff = compute_diff(&snap, &snap);
        assert_eq!(diff.counts.partition_count, 0);
        assert_eq!(diff.counts.total_nodes, 0);
        assert_eq!(diff.counts.total_incidents, 0);
        assert_eq!(diff.counts.agreement_count, 0);
        assert_eq!(diff.counts.conflict_count, 0);
        assert_eq!(diff.counts.island_count, 0);
    }

    #[test]
    fn identical_snapshots_incident_groups_all_unchanged() {
        let mut groups = BTreeMap::new();
        groups.insert("err_timeout".to_string(), 3);
        groups.insert("err_auth".to_string(), 7);
        let snap = make_snapshot(1, 2, 3, 4, 5, 6, groups);
        let diff = compute_diff(&snap, &snap);
        for (_, delta) in &diff.incident_groups {
            assert!(
                matches!(delta, IncidentGroupDelta::Unchanged(_)),
                "expected Unchanged, got {:?}",
                delta
            );
        }
    }

    #[test]
    fn differing_counts_positive_deltas() {
        let baseline = make_snapshot(1, 2, 3, 4, 5, 6, BTreeMap::new());
        let current = make_snapshot(3, 5, 10, 8, 9, 11, BTreeMap::new());
        let diff = compute_diff(&baseline, &current);
        assert_eq!(diff.counts.partition_count, 2);
        assert_eq!(diff.counts.total_nodes, 3);
        assert_eq!(diff.counts.total_incidents, 7);
        assert_eq!(diff.counts.agreement_count, 4);
        assert_eq!(diff.counts.conflict_count, 4);
        assert_eq!(diff.counts.island_count, 5);
    }

    #[test]
    fn differing_counts_negative_deltas() {
        let baseline = make_snapshot(10, 20, 30, 40, 50, 60, BTreeMap::new());
        let current = make_snapshot(1, 2, 3, 4, 5, 6, BTreeMap::new());
        let diff = compute_diff(&baseline, &current);
        assert_eq!(diff.counts.partition_count, -9);
        assert_eq!(diff.counts.total_nodes, -18);
        assert_eq!(diff.counts.total_incidents, -27);
        assert_eq!(diff.counts.agreement_count, -36);
        assert_eq!(diff.counts.conflict_count, -45);
        assert_eq!(diff.counts.island_count, -54);
    }

    #[test]
    fn added_incident_group_key() {
        let baseline = make_snapshot(1, 2, 3, 4, 5, 6, BTreeMap::new());
        let mut cur_groups = BTreeMap::new();
        cur_groups.insert("new_error".to_string(), 5);
        let current = make_snapshot(1, 2, 3, 4, 5, 6, cur_groups);
        let diff = compute_diff(&baseline, &current);
        assert_eq!(
            diff.incident_groups.get("new_error"),
            Some(&IncidentGroupDelta::Added(5))
        );
    }

    #[test]
    fn removed_incident_group_key() {
        let mut base_groups = BTreeMap::new();
        base_groups.insert("old_error".to_string(), 3);
        let baseline = make_snapshot(1, 2, 3, 4, 5, 6, base_groups);
        let current = make_snapshot(1, 2, 3, 4, 5, 6, BTreeMap::new());
        let diff = compute_diff(&baseline, &current);
        assert_eq!(
            diff.incident_groups.get("old_error"),
            Some(&IncidentGroupDelta::Removed(3))
        );
    }

    #[test]
    fn changed_incident_group_count_correct_delta() {
        let mut base_groups = BTreeMap::new();
        base_groups.insert("err_x".to_string(), 4);
        let baseline = make_snapshot(1, 2, 3, 4, 5, 6, base_groups);
        let mut cur_groups = BTreeMap::new();
        cur_groups.insert("err_x".to_string(), 9);
        let current = make_snapshot(1, 2, 3, 4, 5, 6, cur_groups);
        let diff = compute_diff(&baseline, &current);
        assert_eq!(
            diff.incident_groups.get("err_x"),
            Some(&IncidentGroupDelta::Changed {
                baseline: 4,
                current: 9,
                delta: 5,
            })
        );
    }

    // --- format_diff tests ---

    #[test]
    fn format_diff_positive_delta_sign() {
        let baseline = make_snapshot(1, 1, 1, 1, 1, 1, BTreeMap::new());
        let current = make_snapshot(5, 1, 1, 1, 1, 1, BTreeMap::new());
        let diff = compute_diff(&baseline, &current);
        let out = format_diff(&diff);
        assert!(
            out.contains("+4"),
            "expected '+4' in output for positive delta, got:\n{}",
            out
        );
    }

    #[test]
    fn format_diff_negative_delta_sign() {
        let baseline = make_snapshot(10, 1, 1, 1, 1, 1, BTreeMap::new());
        let current = make_snapshot(3, 1, 1, 1, 1, 1, BTreeMap::new());
        let diff = compute_diff(&baseline, &current);
        let out = format_diff(&diff);
        assert!(
            out.contains("-7"),
            "expected '-7' in output for negative delta, got:\n{}",
            out
        );
    }

    #[test]
    fn format_diff_zero_delta_no_sign() {
        let snap = base_snapshot();
        let diff = compute_diff(&snap, &snap);
        let out = format_diff(&diff);
        // All deltas are zero — should appear as "0" not "+0" or "-0"
        assert!(
            out.contains(" 0\n") || out.contains(" 0 "),
            "expected '0' (no sign) for zero delta, got:\n{}",
            out
        );
        assert!(!out.contains("+0"), "unexpected '+0' in output:\n{}", out);
        assert!(!out.contains("-0"), "unexpected '-0' in output:\n{}", out);
    }

    #[test]
    fn format_diff_same_diff_twice_byte_identical() {
        let mut base_groups = BTreeMap::new();
        base_groups.insert("err_a".to_string(), 2);
        let baseline = make_snapshot(1, 2, 3, 4, 5, 6, base_groups);
        let mut cur_groups = BTreeMap::new();
        cur_groups.insert("err_a".to_string(), 5);
        cur_groups.insert("err_b".to_string(), 1);
        let current = make_snapshot(3, 4, 5, 6, 7, 8, cur_groups);
        let diff = compute_diff(&baseline, &current);
        let a = format_diff(&diff);
        let b = format_diff(&diff);
        assert_eq!(a, b, "format_diff is not deterministic");
    }
}
