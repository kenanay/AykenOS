#!/usr/bin/env python3
"""Validate that convergence artifacts remain descriptive and non-elective."""

from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path
from typing import Any


REQUIRED_ARTIFACTS = (
    "parity_convergence_report.json",
    "parity_drift_attribution_report.json",
)

ALLOWED_DESCRIPTIVE_FIELDS = {
    "cluster_derivation",
    "conflict_summary",
    "determinism_conflict_surface_count",
    "determinism_violation_present",
    "edge_match_cluster_derivation",
    "edge_match_clusters",
    "global_status",
    "historical_authority_island_count",
    "historical_authority_islands",
    "insufficient_evidence_island_count",
    "insufficient_evidence_islands",
    "largest_outcome_cluster_size",
    "largest_surface_partition_size",
    "outcome_convergence_ratio",
    "outcome_partition_count",
    "outcome_partitions",
    "surface_consistency_ratio",
    "surface_partition_count",
    "surface_partitions",
}

ALLOWED_GLOBAL_STATUSES = {
    "N_PARITY_CONSISTENCY_SPLIT",
    "N_PARITY_CONVERGED",
    "N_PARITY_DETERMINISM_VIOLATION",
    "N_PARITY_HISTORICAL_ISLAND",
    "N_PARITY_INSUFFICIENT_EVIDENCE",
    "N_PARITY_MIXED",
}

ALLOWED_DERIVATION_VALUES = {
    "cluster_derivation": {"node_parity_outcome_dk_partitions"},
    "edge_match_cluster_derivation": {"pairwise_match_graph_connected_components"},
}

EXACT_FORBIDDEN_FIELDS = {
    "accepted_cluster",
    "accepted_partition",
    "admitted_cluster",
    "admitted_partition",
    "canonical_cluster",
    "canonical_partition",
    "cluster_acceptance",
    "cluster_commit",
    "cluster_finality",
    "cluster_policy_input",
    "cluster_priority",
    "cluster_replay_admission",
    "cluster_truth",
    "cluster_weight",
    "committed_cluster",
    "convergence_finality",
    "execution_route",
    "execution_weight",
    "largest_cluster_policy_input",
    "majority_cluster",
    "match_group_priority",
    "node_priority",
    "partition_commit",
    "partition_policy_input",
    "partition_replay_admission",
    "preferred_cluster",
    "preferred_partition",
    "recommended_cluster",
    "recommended_partition",
    "replay_admission_cluster",
    "routing_hint",
    "selected_cluster",
    "selected_partition",
    "strongest_cluster",
    "suppressed_partition",
    "verification_route",
    "verification_weight",
    "winning_cluster",
    "winning_partition",
}

PATTERN_RULES = (
    (
        "cluster_selection_pattern",
        re.compile(
            r"(selected|winning|chosen|preferred|recommended|committed|admitted|canonical).*(cluster|partition|match|group|convergence)"
            r"|(cluster|partition|match|group|convergence).*(selected|winning|chosen|preferred|recommended|committed|admitted|canonical)",
            re.IGNORECASE,
        ),
        "field encodes convergence election or selection semantics",
    ),
    (
        "policy_input_pattern",
        re.compile(
            r"(cluster|partition|match|group|convergence|ratio|size).*(policy|admission|replay|execution|route|routing|priority|weight|quarantine|suppress|mitigation)"
            r"|(policy|admission|replay|execution|route|routing|priority|weight|quarantine|suppress|mitigation).*(cluster|partition|match|group|convergence|ratio|size)",
            re.IGNORECASE,
        ),
        "field promotes descriptive convergence metrics into policy or routing input",
    ),
    (
        "finality_pattern",
        re.compile(
            r"(cluster|partition|convergence|global).*(truth|final|finality|accept|authority)"
            r"|(truth|final|finality|accept|authority).*(cluster|partition|convergence|global)",
            re.IGNORECASE,
        ),
        "field implies truth, authority, or finality from convergence state",
    ),
    (
        "island_collapse_pattern",
        re.compile(
            r"(historical|insufficient).*(collapse|collapsed|promote|promoted|selected|accepted|merged)"
            r"|(collapse|collapsed|promote|promoted|selected|accepted|merged).*(historical|insufficient)",
            re.IGNORECASE,
        ),
        "field silently collapses historical or insufficient-evidence islands into a selected cluster",
    ),
)

VIOLATION_MATRIX = (
    {
        "case_id": "P13-NEG-07",
        "rule": "largest cluster and partition metadata must remain descriptive only",
    },
    {
        "case_id": "P13-NEG-08",
        "rule": "convergence must not imply admission, execution, or truth finality",
    },
    {
        "case_id": "P13-NEG-09",
        "rule": "convergence artifacts must not resolve a winning verdict or cluster",
    },
    {
        "case_id": "P13-NEG-10",
        "rule": "historical and insufficient-evidence islands must remain explicit diagnostics",
    },
)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Validate convergence artifacts against the non-election boundary."
    )
    parser.add_argument("--artifact-root", required=True, help="Directory containing convergence artifacts.")
    parser.add_argument("--out-report", required=True, help="Output gate report.json path.")
    parser.add_argument("--out-detail-report", required=True, help="Output detailed report path.")
    parser.add_argument("--violations-out", required=True, help="Output violations.txt path.")
    return parser.parse_args()


def load_json(path: Path) -> Any:
    with path.open("r", encoding="utf-8") as handle:
        return json.load(handle)


def write_json(path: Path, payload: Any) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("w", encoding="utf-8") as handle:
        json.dump(payload, handle, indent=2, sort_keys=True)
        handle.write("\n")


def write_violations(path: Path, violations: list[str]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text("\n".join(violations) + ("\n" if violations else ""), encoding="utf-8")


def classify_key(key: str) -> tuple[str, str] | None:
    lowered = key.lower()
    if lowered in ALLOWED_DESCRIPTIVE_FIELDS:
        return None
    if lowered in EXACT_FORBIDDEN_FIELDS:
        return ("exact_forbidden_field", "field matches a prohibited convergence-election key")
    for rule_name, pattern, message in PATTERN_RULES:
        if pattern.search(key):
            return (rule_name, message)
    return None


def scan_value(
    artifact_name: str,
    value: Any,
    path: str,
    hits: list[dict[str, str]],
) -> None:
    if isinstance(value, dict):
        for key, child in value.items():
            key_path = f"{path}.{key}" if path else key
            classification = classify_key(key)
            if classification is not None:
                rule, message = classification
                hits.append(
                    {
                        "artifact": artifact_name,
                        "path": key_path,
                        "field": key,
                        "rule": rule,
                        "message": message,
                    }
                )
            scan_value(artifact_name, child, key_path, hits)
    elif isinstance(value, list):
        for index, child in enumerate(value):
            child_path = f"{path}[{index}]" if path else f"[{index}]"
            scan_value(artifact_name, child, child_path, hits)


def validate_semantic_contracts(
    convergence_payload: Any,
) -> tuple[list[str], list[dict[str, str]]]:
    violations: list[str] = []
    checks: list[dict[str, str]] = []
    if not isinstance(convergence_payload, dict):
        violations.append("invalid_payload:parity_convergence_report.json:not_an_object")
        return violations, checks

    global_status = convergence_payload.get("global_status")
    if global_status is None:
        violations.append("missing_required_field:parity_convergence_report.json:global_status")
        checks.append(
            {
                "field": "global_status",
                "status": "FAIL",
                "rule": "convergence report must expose diagnostic global_status",
            }
        )
    elif global_status not in ALLOWED_GLOBAL_STATUSES:
        violations.append(
            "invalid_global_status:"
            f"parity_convergence_report.json:global_status:{global_status}"
        )
        checks.append(
            {
                "field": "global_status",
                "observed": str(global_status),
                "status": "FAIL",
                "rule": "global_status must remain within the descriptive parity status enum",
            }
        )
    else:
        checks.append(
            {
                "field": "global_status",
                "observed": str(global_status),
                "status": "PASS",
                "rule": "global_status must remain within the descriptive parity status enum",
            }
        )

    for field_name, allowed_values in ALLOWED_DERIVATION_VALUES.items():
        value = convergence_payload.get(field_name)
        if value is None:
            violations.append(
                f"missing_required_field:parity_convergence_report.json:{field_name}"
            )
            checks.append(
                {
                    "field": field_name,
                    "status": "FAIL",
                    "rule": "derivation metadata must remain explicit and descriptive",
                }
            )
            continue
        if value not in allowed_values:
            violations.append(
                "invalid_derivation_value:"
                f"parity_convergence_report.json:{field_name}:{value}"
            )
            checks.append(
                {
                    "field": field_name,
                    "observed": str(value),
                    "status": "FAIL",
                    "rule": "derivation metadata must not drift into selection or voting algorithms",
                }
            )
            continue
        checks.append(
            {
                "field": field_name,
                "observed": str(value),
                "status": "PASS",
                "rule": "derivation metadata must not drift into selection or voting algorithms",
            }
        )

    return violations, checks


def main() -> int:
    args = parse_args()
    artifact_root = Path(args.artifact_root).resolve()
    out_report = Path(args.out_report).resolve()
    out_detail_report = Path(args.out_detail_report).resolve()
    violations_out = Path(args.violations_out).resolve()

    violations: list[str] = []
    forbidden_hits: list[dict[str, str]] = []
    checked_artifacts: list[str] = []
    convergence_payload: Any = None

    for artifact_name in REQUIRED_ARTIFACTS:
        path = artifact_root / artifact_name
        if not path.is_file():
            violations.append(f"missing_required_artifact:{artifact_name}")
            continue
        checked_artifacts.append(artifact_name)
        try:
            payload = load_json(path)
        except json.JSONDecodeError:
            violations.append(f"invalid_json:{artifact_name}")
            continue
        if artifact_name == "parity_convergence_report.json":
            convergence_payload = payload
        scan_value(artifact_name, payload, "", forbidden_hits)

    semantic_violations, semantic_checks = validate_semantic_contracts(convergence_payload)
    violations.extend(semantic_violations)

    for hit in forbidden_hits:
        violations.append(
            "forbidden_convergence_election_field:"
            f"{hit['artifact']}:{hit['path']}:{hit['field']}:{hit['rule']}"
        )

    detail_report = {
        "status": "PASS" if not violations else "FAIL",
        "mode": "phase13_convergence_non_election_boundary_gate",
        "artifact_root": artifact_root.as_posix(),
        "required_artifact_count": len(REQUIRED_ARTIFACTS),
        "checked_artifact_count": len(checked_artifacts),
        "checked_artifacts": checked_artifacts,
        "allowed_descriptive_fields": sorted(ALLOWED_DESCRIPTIVE_FIELDS),
        "allowed_global_statuses": sorted(ALLOWED_GLOBAL_STATUSES),
        "allowed_derivation_values": {
            key: sorted(values) for key, values in ALLOWED_DERIVATION_VALUES.items()
        },
        "exact_forbidden_fields": sorted(EXACT_FORBIDDEN_FIELDS),
        "pattern_rules": [
            {"rule": rule_name, "description": message}
            for rule_name, _pattern, message in PATTERN_RULES
        ],
        "violation_matrix": list(VIOLATION_MATRIX),
        "semantic_contract_checks": semantic_checks,
        "forbidden_field_count": len(forbidden_hits),
        "forbidden_field_hits": forbidden_hits,
        "violations": violations,
        "violations_count": len(violations),
    }

    gate_report = {
        "gate": "convergence-non-election-boundary",
        "mode": "phase13_convergence_non_election_boundary_gate",
        "verdict": "PASS" if not violations else "FAIL",
        "detail_report_path": out_detail_report.name,
        "violations": violations,
        "violations_count": len(violations),
    }

    write_json(out_detail_report, detail_report)
    write_json(out_report, gate_report)
    write_violations(violations_out, violations)
    return 0 if not violations else 2


if __name__ == "__main__":
    sys.exit(main())
