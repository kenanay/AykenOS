#!/usr/bin/env python3
"""Validate that graph/topology artifacts remain non-authoritative and non-inferential."""

from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path
from typing import Any


REQUIRED_ARTIFACTS = (
    "parity_convergence_report.json",
    "parity_authority_drift_topology.json",
    "parity_incident_graph.json",
    "parity_consistency_report.json",
)

ALLOWED_DESCRIPTIVE_FIELDS = {
    "dominant_authority_chain_id",
    "dominant_authority_cluster_key",
    "surface_consistency_ratio",
    "outcome_convergence_ratio",
    "largest_surface_partition_size",
    "largest_outcome_cluster_size",
}

EXACT_FORBIDDEN_FIELDS = {
    "accepted_authority",
    "authority_winner",
    "canonical_truth",
    "chosen_cluster",
    "cluster_consensus_strength",
    "cluster_truth",
    "consensus_strength",
    "majority_accept",
    "majority_verdict",
    "recommended_verdict",
    "selected_authority",
    "selected_truth",
    "statistical_truth",
    "truth_estimate",
    "truth_signal",
    "winning_cluster",
    "winning_verdict",
}

PATTERN_RULES = (
    (
        "consensus_truth_pattern",
        re.compile(
            r"(consensus|majority).*(truth|verdict|accept|winner|cluster)|(truth|verdict|accept|winner|cluster).*(consensus|majority)",
            re.IGNORECASE,
        ),
        "field encodes consensus- or majority-derived truth semantics",
    ),
    (
        "truth_inference_pattern",
        re.compile(
            r"(truth|verdict).*(estimate|inference|prediction|selection)|(estimate|inference|prediction|selection).*(truth|verdict)",
            re.IGNORECASE,
        ),
        "field encodes truth inference semantics",
    ),
    (
        "winner_selection_pattern",
        re.compile(
            r"(selected|winning|chosen|recommended).*(cluster|authority|verdict|truth)|(cluster|authority|verdict|truth).*(selected|winning|chosen|recommended)",
            re.IGNORECASE,
        ),
        "field encodes winner selection or recommendation semantics",
    ),
)

VIOLATION_MATRIX = (
    {
        "case_id": "P13-NEG-05",
        "rule": "majority verdict must not be promoted to canonical truth",
    },
    {
        "case_id": "P13-NEG-06",
        "rule": "dominant cluster metadata must remain descriptive only",
    },
    {
        "case_id": "P13-NEG-08",
        "rule": "convergence must not imply admission, execution, or truth finality",
    },
    {
        "case_id": "P13-NEG-09",
        "rule": "graph and convergence artifacts must not resolve a winning verdict",
    },
)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Validate graph/topology artifacts against the non-authoritative contract."
    )
    parser.add_argument("--artifact-root", required=True, help="Directory containing diagnostics artifacts.")
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
    path.write_text(
        "\n".join(violations) + ("\n" if violations else ""),
        encoding="utf-8",
    )


def classify_key(key: str) -> tuple[str, str] | None:
    lowered = key.lower()
    if lowered in ALLOWED_DESCRIPTIVE_FIELDS:
        return None
    if lowered in EXACT_FORBIDDEN_FIELDS:
        return ("exact_forbidden_field", "field matches a prohibited truth-inference key")
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


def main() -> int:
    args = parse_args()
    artifact_root = Path(args.artifact_root).resolve()
    out_report = Path(args.out_report).resolve()
    out_detail_report = Path(args.out_detail_report).resolve()
    violations_out = Path(args.violations_out).resolve()

    violations: list[str] = []
    forbidden_hits: list[dict[str, str]] = []
    checked_artifacts: list[str] = []

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
        scan_value(artifact_name, payload, "", forbidden_hits)

    for hit in forbidden_hits:
        violations.append(
            "forbidden_truth_inference_field:"
            f"{hit['artifact']}:{hit['path']}:{hit['field']}:{hit['rule']}"
        )

    detail_report = {
        "status": "PASS" if not violations else "FAIL",
        "mode": "phase13_graph_non_authoritative_contract_gate",
        "artifact_root": artifact_root.as_posix(),
        "required_artifact_count": len(REQUIRED_ARTIFACTS),
        "checked_artifact_count": len(checked_artifacts),
        "checked_artifacts": checked_artifacts,
        "allowed_descriptive_fields": sorted(ALLOWED_DESCRIPTIVE_FIELDS),
        "exact_forbidden_fields": sorted(EXACT_FORBIDDEN_FIELDS),
        "pattern_rules": [
            {
                "rule": rule_name,
                "description": message,
            }
            for rule_name, _pattern, message in PATTERN_RULES
        ],
        "violation_matrix": list(VIOLATION_MATRIX),
        "forbidden_field_count": len(forbidden_hits),
        "forbidden_field_hits": forbidden_hits,
        "violations": violations,
        "violations_count": len(violations),
    }

    gate_report = {
        "gate": "graph-non-authoritative-contract",
        "mode": "phase13_graph_non_authoritative_contract_gate",
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
