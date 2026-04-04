#!/usr/bin/env python3
"""Validate that descriptive diagnostics stay out of execution-bearing consumers."""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path


DEFAULT_SCAN_ROOTS = (
    "ayken-core/crates",
    "userspace",
)

DEFAULT_ALLOWED_PATHS = {
    "ayken-core/crates/proof-verifier/examples/phase12_gate_harness.rs",
    "ayken-core/crates/proof-verifier/src/authority/authority_drift_topology.rs",
    "ayken-core/crates/proof-verifier/src/authority/drift_attribution.rs",
    # Producer modules: these files generate dominant_authority_chain_id as an
    # observability output field — they do not consume it as an execution input.
    # Gate intent is to block consumer-side use; producer-side is allowed.
    "ayken-core/crates/proof-verifier/src/authority_sinkhole_absorption.rs",
    "ayken-core/crates/proof-verifier/src/diversity_floor.rs",
    # Canonical public diagnostics contract registry: these references define
    # boundary-visible artifact identities and query/method policy, but do not
    # consume diagnostics as execution input.
    "userspace/proofd/src/api_contract.rs",
    "userspace/proofd/src/lib.rs",
    "userspace/proofd/examples/proofd_gate_harness.rs",
}

PROTECTED_DIAGNOSTIC_FIELDS = (
    "cluster_derivation",
    "dominant_authority_chain_id",
    "edge_match_cluster_derivation",
    "global_status",
    "historical_authority_island_count",
    "historical_authority_islands",
    "insufficient_evidence_island_count",
    "insufficient_evidence_islands",
    "largest_outcome_cluster_size",
    "outcome_convergence_ratio",
)

PROTECTED_DIAGNOSTIC_ARTIFACTS = (
    "parity_authority_drift_topology.json",
    "parity_convergence_report.json",
    "parity_drift_attribution_report.json",
)

VIOLATION_MATRIX = (
    {
        "case_id": "P13-CONS-01",
        "rule": "diagnostics fields must not be imported into non-observability runtime code",
    },
    {
        "case_id": "P13-CONS-02",
        "rule": "convergence and topology artifacts must not become execution or routing inputs",
    },
    {
        "case_id": "P13-CONS-03",
        "rule": "diagnostic global status must not become admission, policy, or priority input",
    },
    {
        "case_id": "P13-CONS-04",
        "rule": "historical or insufficient-evidence island diagnostics must not drive suppression or trust promotion",
    },
)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Validate that descriptive diagnostics stay out of non-authoritative consumers."
    )
    parser.add_argument("--source-root", required=True, help="Repository or fixture root to scan.")
    parser.add_argument(
        "--scan-root",
        action="append",
        dest="scan_roots",
        help="Relative source root to scan recursively for Rust sources. Defaults to ayken-core/crates and userspace.",
    )
    parser.add_argument(
        "--allow-path",
        action="append",
        dest="allow_paths",
        help="Relative Rust source path allowed to reference protected diagnostics. May be passed multiple times.",
    )
    parser.add_argument("--out-report", required=True, help="Output gate report.json path.")
    parser.add_argument("--out-detail-report", required=True, help="Output detailed report path.")
    parser.add_argument("--violations-out", required=True, help="Output violations.txt path.")
    return parser.parse_args()


def write_json(path: Path, payload: object) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("w", encoding="utf-8") as handle:
        json.dump(payload, handle, indent=2, sort_keys=True)
        handle.write("\n")


def write_violations(path: Path, violations: list[str]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text("\n".join(violations) + ("\n" if violations else ""), encoding="utf-8")


def is_comment_only(line: str) -> bool:
    stripped = line.strip()
    return (
        not stripped
        or stripped.startswith("//")
        or stripped.startswith("/*")
        or stripped.startswith("*")
        or stripped.startswith("#")
    )


def main() -> int:
    args = parse_args()
    source_root = Path(args.source_root).resolve()
    scan_roots = tuple(args.scan_roots or DEFAULT_SCAN_ROOTS)
    allow_paths = set(args.allow_paths or DEFAULT_ALLOWED_PATHS)
    out_report = Path(args.out_report).resolve()
    out_detail_report = Path(args.out_detail_report).resolve()
    violations_out = Path(args.violations_out).resolve()

    violations: list[str] = []
    checked_files: list[str] = []
    field_hits: list[dict[str, str | int]] = []
    artifact_hits: list[dict[str, str | int]] = []

    for relative_root in scan_roots:
        scan_root = source_root / relative_root
        if not scan_root.is_dir():
            violations.append(f"missing_scan_root:{relative_root}")
            continue
        for path in sorted(scan_root.rglob("*.rs")):
            relative_path = path.relative_to(source_root).as_posix()
            checked_files.append(relative_path)
            if relative_path in allow_paths:
                continue
            lines = path.read_text(encoding="utf-8").splitlines()
            for index, line in enumerate(lines, start=1):
                if is_comment_only(line):
                    continue
                for token in PROTECTED_DIAGNOSTIC_FIELDS:
                    if token in line:
                        field_hits.append(
                            {
                                "file": relative_path,
                                "line": index,
                                "token": token,
                                "snippet": line.strip(),
                            }
                        )
                for artifact in PROTECTED_DIAGNOSTIC_ARTIFACTS:
                    if artifact in line:
                        artifact_hits.append(
                            {
                                "file": relative_path,
                                "line": index,
                                "token": artifact,
                                "snippet": line.strip(),
                            }
                        )

    for hit in field_hits:
        violations.append(
            "forbidden_diagnostics_consumer_field:"
            f"{hit['file']}:{hit['line']}:{hit['token']}"
        )
    for hit in artifact_hits:
        violations.append(
            "forbidden_diagnostics_consumer_artifact:"
            f"{hit['file']}:{hit['line']}:{hit['token']}"
        )

    detail_report = {
        "status": "PASS" if not violations else "FAIL",
        "mode": "phase13_diagnostics_consumer_non_authoritative_contract_gate",
        "source_root": source_root.as_posix(),
        "scan_roots": list(scan_roots),
        "checked_file_count": len(checked_files),
        "checked_files": checked_files,
        "allowed_path_count": len(allow_paths),
        "allowed_paths": sorted(allow_paths),
        "protected_diagnostic_fields": list(PROTECTED_DIAGNOSTIC_FIELDS),
        "protected_diagnostic_artifacts": list(PROTECTED_DIAGNOSTIC_ARTIFACTS),
        "violation_matrix": list(VIOLATION_MATRIX),
        "field_hit_count": len(field_hits),
        "field_hits": field_hits,
        "artifact_hit_count": len(artifact_hits),
        "artifact_hits": artifact_hits,
        "violations": violations,
        "violations_count": len(violations),
    }

    gate_report = {
        "gate": "diagnostics-consumer-non-authoritative-contract",
        "mode": "phase13_diagnostics_consumer_non_authoritative_contract_gate",
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
