use crate::models::Snapshot;

/// Words that must never appear in formatter output.
/// Checked in tests to enforce the epistemic boundary.
pub const FORBIDDEN: &[&str] = &[
    "best",
    "worst",
    "recommended",
    "optimal",
    "trust score",
    "ranking",
    "decision",
    "recommendation",
];

pub fn format_snapshot(snapshot: &Snapshot) -> String {
    let mut out = String::new();

    // Header block
    out.push_str(&format!(
        "authority: {}\n",
        snapshot.authority_classification
    ));
    out.push_str(&format!(
        "produces_truth: {}\n",
        snapshot.flags.produces_truth
    ));
    out.push_str(&format!(
        "produces_decision: {}\n",
        snapshot.flags.produces_decision
    ));
    out.push_str(&format!(
        "produces_ranking: {}\n",
        snapshot.flags.produces_ranking
    ));

    out.push('\n');

    // Counts block
    out.push_str("counts:\n");
    out.push_str(&format!(
        "  partition_count:   {}\n",
        snapshot.counts.partition_count
    ));
    out.push_str(&format!(
        "  total_nodes:       {}\n",
        snapshot.counts.total_nodes
    ));
    out.push_str(&format!(
        "  total_incidents:   {}\n",
        snapshot.counts.total_incidents
    ));
    out.push_str(&format!(
        "  agreement_count:   {}\n",
        snapshot.counts.agreement_count
    ));
    out.push_str(&format!(
        "  conflict_count:    {}\n",
        snapshot.counts.conflict_count
    ));
    out.push_str(&format!(
        "  island_count:      {}\n",
        snapshot.counts.island_count
    ));

    out.push('\n');

    // Incident groups block
    out.push_str("incident_groups:\n");
    if snapshot.incident_groups.is_empty() {
        out.push_str("  no incidents recorded\n");
    } else {
        // BTreeMap guarantees lexicographic order — no sort needed
        for (key, count) in &snapshot.incident_groups {
            out.push_str(&format!("  {}: {}\n", key, count));
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Counts, SnapshotFlags};
    use std::collections::BTreeMap;

    fn make_snapshot(incident_groups: BTreeMap<String, usize>) -> Snapshot {
        Snapshot {
            summary_origin: "test".to_string(),
            authority_classification: "non_authoritative".to_string(),
            display_mode: "machine_structured".to_string(),
            counts: Counts {
                partition_count: 1,
                total_nodes: 2,
                total_incidents: 3,
                agreement_count: 4,
                conflict_count: 5,
                island_count: 6,
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
    fn empty_incident_groups_emits_no_incidents_recorded() {
        let snap = make_snapshot(BTreeMap::new());
        let out = format_snapshot(&snap);
        assert!(
            out.contains("  no incidents recorded"),
            "expected '  no incidents recorded' in output, got:\n{}",
            out
        );
    }

    #[test]
    fn non_empty_incident_groups_appear_in_lexicographic_order() {
        let mut groups = BTreeMap::new();
        groups.insert("zebra".to_string(), 1);
        groups.insert("alpha".to_string(), 2);
        groups.insert("mango".to_string(), 3);

        let snap = make_snapshot(groups);
        let out = format_snapshot(&snap);

        let alpha_pos = out.find("alpha").expect("alpha not found");
        let mango_pos = out.find("mango").expect("mango not found");
        let zebra_pos = out.find("zebra").expect("zebra not found");

        assert!(
            alpha_pos < mango_pos && mango_pos < zebra_pos,
            "incident groups not in lexicographic order"
        );
    }

    #[test]
    fn output_contains_all_six_count_labels() {
        let snap = make_snapshot(BTreeMap::new());
        let out = format_snapshot(&snap);
        for label in &[
            "partition_count",
            "total_nodes",
            "total_incidents",
            "agreement_count",
            "conflict_count",
            "island_count",
        ] {
            assert!(out.contains(label), "missing count label: {}", label);
        }
    }

    #[test]
    fn output_contains_authority_non_authoritative() {
        let snap = make_snapshot(BTreeMap::new());
        let out = format_snapshot(&snap);
        assert!(
            out.contains("authority: non_authoritative"),
            "missing 'authority: non_authoritative'"
        );
    }

    #[test]
    fn output_contains_flag_labels_as_false() {
        let snap = make_snapshot(BTreeMap::new());
        let out = format_snapshot(&snap);
        assert!(out.contains("produces_truth: false"), "missing produces_truth: false");
        assert!(out.contains("produces_decision: false"), "missing produces_decision: false");
        assert!(out.contains("produces_ranking: false"), "missing produces_ranking: false");
    }

    #[test]
    fn output_does_not_contain_forbidden_words() {
        let snap = make_snapshot(BTreeMap::new());
        let out = format_snapshot(&snap).to_lowercase();
        for word in FORBIDDEN {
            // Check that the forbidden word does not appear as a standalone token.
            // We look for the word surrounded by non-alphanumeric/non-underscore
            // boundaries so that e.g. "ranking" inside "produces_ranking" is not
            // a false positive.
            let pattern = format!(r"(?:^|[^a-z0-9_]){}(?:[^a-z0-9_]|$)", word);
            let re = regex_lite_check(&out, &pattern);
            assert!(
                !re,
                "forbidden word '{}' found as standalone token in output:\n{}",
                word,
                out
            );
        }
    }

    /// Simple manual boundary check without pulling in a regex crate.
    fn regex_lite_check(haystack: &str, pattern: &str) -> bool {
        // Extract the word from the pattern (between the two (?:...) groups)
        // Pattern format: r"(?:^|[^a-z0-9_])<word>(?:[^a-z0-9_]|$)"
        let prefix = "(?:^|[^a-z0-9_])";
        let suffix = "(?:[^a-z0-9_]|$)";
        let word = pattern
            .strip_prefix(prefix)
            .and_then(|s| s.strip_suffix(suffix))
            .unwrap_or("");
        if word.is_empty() {
            return false;
        }
        let mut start = 0;
        while let Some(pos) = haystack[start..].find(word) {
            let abs = start + pos;
            let before_ok = abs == 0
                || !haystack.as_bytes()[abs - 1].is_ascii_alphanumeric()
                    && haystack.as_bytes()[abs - 1] != b'_';
            let end = abs + word.len();
            let after_ok = end >= haystack.len()
                || !haystack.as_bytes()[end].is_ascii_alphanumeric()
                    && haystack.as_bytes()[end] != b'_';
            if before_ok && after_ok {
                return true;
            }
            start = abs + 1;
        }
        false
    }

    #[test]
    fn same_snapshot_formatted_twice_is_byte_identical() {
        let mut groups = BTreeMap::new();
        groups.insert("err_timeout".to_string(), 7);
        groups.insert("err_auth".to_string(), 3);

        let snap = make_snapshot(groups);
        let a = format_snapshot(&snap);
        let b = format_snapshot(&snap);
        assert_eq!(a, b, "format_snapshot is not deterministic");
    }
}
