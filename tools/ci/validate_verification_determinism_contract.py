#!/usr/bin/env python3
"""Validate that verifier-critical modules stay environment-independent."""

from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path


DEFAULT_SOURCE_PATHS = (
    "ayken-core/crates/proof-verifier/src/lib.rs",
    "ayken-core/crates/proof-verifier/src/errors.rs",
    "ayken-core/crates/proof-verifier/src/types.rs",
    "ayken-core/crates/proof-verifier/src/canonical/digest.rs",
    "ayken-core/crates/proof-verifier/src/canonical/jcs.rs",
    "ayken-core/crates/proof-verifier/src/canonical/tree_hash.rs",
    "ayken-core/crates/proof-verifier/src/policy/policy_engine.rs",
    "ayken-core/crates/proof-verifier/src/policy/quorum.rs",
    "ayken-core/crates/proof-verifier/src/policy/schema.rs",
    "ayken-core/crates/proof-verifier/src/registry/resolver.rs",
    "ayken-core/crates/proof-verifier/src/registry/snapshot.rs",
    "ayken-core/crates/proof-verifier/src/authority/parity.rs",
    "ayken-core/crates/proof-verifier/src/authority/determinism_incident.rs",
    "ayken-core/crates/proof-verifier/src/authority/drift_attribution.rs",
    "ayken-core/crates/proof-verifier/src/authority/incident_graph.rs",
    "ayken-core/crates/proof-verifier/src/authority/authority_drift_topology.rs",
    "ayken-core/crates/proof-verifier/src/authority/resolution.rs",
    "ayken-core/crates/proof-verifier/src/authority/snapshot.rs",
    "ayken-core/crates/proof-verifier/src/verdict/verdict_engine.rs",
    "ayken-core/crates/proof-verifier/src/verdict/subject.rs",
    "ayken-core/crates/proof-verifier/src/overlay/overlay_validator.rs",
    "ayken-core/crates/proof-verifier/src/portable_core/identity.rs",
    "ayken-core/crates/proof-verifier/src/receipt/schema.rs",
    "ayken-core/crates/proof-verifier/src/receipt/verify.rs",
    "ayken-core/crates/proof-verifier/src/crypto/ed25519.rs",
)

PATTERN_RULES = (
    (
        "time_dependency",
        re.compile(r"\b(SystemTime|Instant|UNIX_EPOCH)\b|std::time", re.IGNORECASE),
        "verification-critical code must not depend on wall-clock or process time",
    ),
    (
        "randomness_dependency",
        re.compile(r"\brand::|thread_rng|getrandom|random\s*\(", re.IGNORECASE),
        "verification-critical code must not depend on randomness",
    ),
    (
        "ambient_environment_dependency",
        re.compile(
            r"std::env|env::var|env::vars|env::var_os|current_dir|temp_dir|home_dir",
            re.IGNORECASE,
        ),
        "verification-critical code must not depend on ambient environment state",
    ),
    (
        "network_dependency",
        re.compile(
            r"\b(TcpListener|TcpStream|UdpSocket)\b|reqwest|hyper|tokio::net|ureq",
            re.IGNORECASE,
        ),
        "verification-critical code must not depend on network-visible context",
    ),
    (
        "filesystem_dependency",
        re.compile(
            r"\bstd::fs\b|use\s+std::fs|fs::read|fs::write|read_dir|OpenOptions|File::open|canonicalize\(",
            re.IGNORECASE,
        ),
        "verification-critical code must not perform filesystem I/O",
    ),
)

VIOLATION_MATRIX = (
    {
        "case_id": "P13-DET-01",
        "rule": "verification-critical code must not depend on time",
    },
    {
        "case_id": "P13-DET-02",
        "rule": "verification-critical code must not depend on randomness",
    },
    {
        "case_id": "P13-DET-03",
        "rule": "verification-critical code must not depend on ambient environment state",
    },
    {
        "case_id": "P13-DET-04",
        "rule": "verification-critical code must not depend on network-visible context",
    },
    {
        "case_id": "P13-DET-05",
        "rule": "verification-critical code must not perform filesystem I/O",
    },
)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Validate verifier-critical modules against the verification determinism contract."
    )
    parser.add_argument("--source-root", required=True, help="Repository or fixture root to scan.")
    parser.add_argument(
        "--source-path",
        action="append",
        dest="source_paths",
        help="Relative source path to scan. May be passed multiple times. Defaults to the curated verifier-critical list.",
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
    path.write_text(
        "\n".join(violations) + ("\n" if violations else ""),
        encoding="utf-8",
    )


def main() -> int:
    args = parse_args()
    source_root = Path(args.source_root).resolve()
    source_paths = tuple(args.source_paths or DEFAULT_SOURCE_PATHS)
    out_report = Path(args.out_report).resolve()
    out_detail_report = Path(args.out_detail_report).resolve()
    violations_out = Path(args.violations_out).resolve()

    violations: list[str] = []
    pattern_hits: list[dict[str, str | int]] = []
    checked_files: list[str] = []

    for relative_path in source_paths:
        path = source_root / relative_path
        if not path.is_file():
            violations.append(f"missing_required_source:{relative_path}")
            continue
        checked_files.append(relative_path)
        lines = path.read_text(encoding="utf-8").splitlines()
        for index, line in enumerate(lines, start=1):
            for rule_name, pattern, message in PATTERN_RULES:
                if pattern.search(line):
                    pattern_hits.append(
                        {
                            "file": relative_path,
                            "line": index,
                            "rule": rule_name,
                            "message": message,
                            "snippet": line.strip(),
                        }
                    )

    for hit in pattern_hits:
        violations.append(
            "forbidden_environment_dependency:"
            f"{hit['file']}:{hit['line']}:{hit['rule']}"
        )

    detail_report = {
        "status": "PASS" if not violations else "FAIL",
        "mode": "phase13_verification_determinism_contract_gate",
        "source_root": source_root.as_posix(),
        "required_source_count": len(source_paths),
        "checked_file_count": len(checked_files),
        "checked_files": checked_files,
        "pattern_rules": [
            {
                "rule": rule_name,
                "description": message,
            }
            for rule_name, _pattern, message in PATTERN_RULES
        ],
        "violation_matrix": list(VIOLATION_MATRIX),
        "pattern_hit_count": len(pattern_hits),
        "pattern_hits": pattern_hits,
        "violations": violations,
        "violations_count": len(violations),
    }

    gate_report = {
        "gate": "verification-determinism-contract",
        "mode": "phase13_verification_determinism_contract_gate",
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
