#!/usr/bin/env python3
"""Validate that observability artifacts do not encode verifier reputation semantics."""

from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path
from typing import Any


REQUIRED_ARTIFACTS = (
    "parity_report.json",
    "parity_determinism_incidents.json",
    "parity_drift_attribution_report.json",
    "parity_convergence_report.json",
    "parity_authority_drift_topology.json",
    "parity_authority_suppression_report.json",
    "parity_incident_graph.json",
)

EXACT_FORBIDDEN_FIELDS = {
    "agreement_ratio",
    "authority_alignment_score",
    "convergence_leadership_score",
    "correctness_rate",
    "dominant_verifier_frequency",
    "historical_correctness_index",
    "node_success_ratio",
    "node_trust_score",
    "reliability_index",
    "trust_score",
    "verifier_reputation",
    "verifier_score",
    "weighted_authority",
}

PATTERN_RULES = (
    (
        "reputation_pattern",
        re.compile(r"reputation", re.IGNORECASE),
        "field encodes verifier reputation semantics",
    ),
    (
        "reliability_pattern",
        re.compile(r"reliability", re.IGNORECASE),
        "field encodes verifier reliability semantics",
    ),
    (
        "correctness_pattern",
        re.compile(r"(verifier|node|historical).*(correctness|accuracy)|(correctness|accuracy).*(verifier|node|historical)", re.IGNORECASE),
        "field encodes historical verifier correctness semantics",
    ),
    (
        "weighted_authority_pattern",
        re.compile(r"weighted.*authority|authority.*weighted", re.IGNORECASE),
        "field encodes weighted authority semantics",
    ),
    (
        "score_pattern",
        re.compile(r"(verifier|trust|authority|correctness|dominant|convergence|node).*(score|rank|rating)|(score|rank|rating).*(verifier|trust|authority|correctness|dominant|convergence|node)", re.IGNORECASE),
        "field encodes ranking or scoring semantics",
    ),
    (
        "leaderboard_pattern",
        re.compile(r"leaderboard|ranking", re.IGNORECASE),
        "field encodes leaderboard semantics",
    ),
    (
        "frequency_pattern",
        re.compile(r"(dominant|verifier|cluster).*(frequency)|(frequency).*(dominant|verifier|cluster)", re.IGNORECASE),
        "field encodes historical frequency-based ranking semantics",
    ),
)

VIOLATION_MATRIX = (
    {
        "case_id": "P13-NEG-15",
        "rule": "observability payloads must not expose verifier reputation or scoring outputs",
    },
    {
        "case_id": "P13-NEG-16",
        "rule": "verification history must not be transformed into implicit authority ranking",
    },
)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Validate observability artifacts against the verifier reputation prohibition contract."
    )
    parser.add_argument("--artifact-root", required=True, help="Directory containing diagnostics artifacts.")
    parser.add_argument("--out-report", required=True, help="Output gate report.json path.")
    parser.add_argument(
        "--out-detail-report",
        required=True,
        help="Output detailed reputation_prohibition_report.json path.",
    )
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
    if lowered in EXACT_FORBIDDEN_FIELDS:
        return ("exact_forbidden_field", "field matches a prohibited reputation or authority-scoring key")
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
            "forbidden_reputation_field:"
            f"{hit['artifact']}:{hit['path']}:{hit['field']}:{hit['rule']}"
        )

    detail_report = {
        "status": "PASS" if not violations else "FAIL",
        "mode": "phase13_verifier_reputation_prohibition_gate",
        "artifact_root": artifact_root.as_posix(),
        "required_artifact_count": len(REQUIRED_ARTIFACTS),
        "checked_artifact_count": len(checked_artifacts),
        "checked_artifacts": checked_artifacts,
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
        "gate": "verifier-reputation-prohibition",
        "mode": "phase13_verifier_reputation_prohibition_gate",
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
