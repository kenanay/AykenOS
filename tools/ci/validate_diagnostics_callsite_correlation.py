#!/usr/bin/env python3
"""Validate that descriptive diagnostics do not flow into decision sinks."""

from __future__ import annotations

import argparse
import json
import re
import sys
from dataclasses import dataclass
from pathlib import Path


DEFAULT_SOURCE_PATHS = (
    "ayken-core/crates/proof-verifier/examples/phase12_gate_harness.rs",
    "ayken-core/crates/proof-verifier/src/authority/authority_drift_topology.rs",
    "ayken-core/crates/proof-verifier/src/authority/drift_attribution.rs",
    "userspace/proofd/src/lib.rs",
    "userspace/proofd/examples/proofd_gate_harness.rs",
)

PROTECTED_SOURCE_TOKENS = (
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
    "parity_authority_drift_topology.json",
    "parity_convergence_report.json",
    "parity_drift_attribution_report.json",
)

SINK_RULES = (
    (
        "policy_sink",
        re.compile(r"\b(evaluate_policy|validate_policy|quorum_satisfied|apply_policy)\s*\("),
        "diagnostics sources must not flow into policy-evaluation call sites",
    ),
    (
        "verification_sink",
        re.compile(r"\b(verify_bundle|run_core_verification)\s*\("),
        "diagnostics sources must not flow into verification execution call sites",
    ),
    (
        "replay_sink",
        re.compile(r"\b(replay_admission|execution_admission|admission_contract)\b"),
        "diagnostics sources must not flow into replay or execution admission sinks",
    ),
    (
        "routing_sink",
        re.compile(r"\b(routing_hint|verification_route|route_verification)\b"),
        "diagnostics sources must not flow into routing sinks",
    ),
    (
        "priority_sink",
        re.compile(r"\b(node_priority|verification_weight|execution_override)\b"),
        "diagnostics sources must not flow into priority or override sinks",
    ),
    (
        "control_sink",
        re.compile(r"\b(recommended_action|accept_authority|promote)\b"),
        "diagnostics sources must not flow into control or promotion sinks",
    ),
)

LET_ASSIGNMENT_RE = re.compile(
    r"\blet\s+(?:mut\s+)?([A-Za-z_][A-Za-z0-9_]*)\b(?:\s*:\s*[^=]+)?\s*="
)
PLAIN_ASSIGNMENT_RE = re.compile(r"\b([A-Za-z_][A-Za-z0-9_]*)\s*=")
FN_RE = re.compile(r"^\s*(?:pub\s+)?(?:async\s+)?fn\s+([A-Za-z_][A-Za-z0-9_]*)\b")

VIOLATION_MATRIX = (
    {
        "case_id": "P13-CORR-01",
        "rule": "descriptive diagnostics must not flow directly into policy or verification sinks",
    },
    {
        "case_id": "P13-CORR-02",
        "rule": "aliasing or renaming descriptive diagnostics must not hide replay or routing consumption",
    },
    {
        "case_id": "P13-CORR-03",
        "rule": "diagnostics artifact imports must not become priority or override signals",
    },
)


@dataclass
class FunctionBlock:
    name: str
    start_line: int
    lines: list[tuple[int, str]]


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Validate that descriptive diagnostics do not flow into decision call sites."
    )
    parser.add_argument("--source-root", required=True, help="Repository or fixture root to scan.")
    parser.add_argument(
        "--source-path",
        action="append",
        dest="source_paths",
        help="Relative source path to scan. May be passed multiple times. Defaults to approved diagnostics producer/passthrough files.",
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


def parse_functions(text: str) -> list[FunctionBlock]:
    functions: list[FunctionBlock] = []
    current_name: str | None = None
    current_start = 0
    current_lines: list[tuple[int, str]] = []
    brace_depth = 0
    saw_open_brace = False

    for line_number, line in enumerate(text.splitlines(), start=1):
        if current_name is None:
            match = FN_RE.match(line)
            if match is None:
                continue
            current_name = match.group(1)
            current_start = line_number
            current_lines = [(line_number, line)]
            brace_depth = line.count("{") - line.count("}")
            saw_open_brace = "{" in line
            if saw_open_brace and brace_depth == 0:
                functions.append(
                    FunctionBlock(name=current_name, start_line=current_start, lines=current_lines)
                )
                current_name = None
            continue

        current_lines.append((line_number, line))
        brace_depth += line.count("{") - line.count("}")
        saw_open_brace = saw_open_brace or "{" in line
        if saw_open_brace and brace_depth == 0:
            functions.append(
                FunctionBlock(name=current_name, start_line=current_start, lines=current_lines)
            )
            current_name = None
            current_lines = []
            brace_depth = 0
            saw_open_brace = False

    return functions


def line_source_tokens(line: str) -> list[str]:
    return [token for token in PROTECTED_SOURCE_TOKENS if token in line]


def extract_assigned_name(line: str) -> str | None:
    match = LET_ASSIGNMENT_RE.search(line)
    if match is not None:
        return match.group(1)
    match = PLAIN_ASSIGNMENT_RE.search(line)
    if match is not None:
        return match.group(1)
    return None


def sink_hits_for_line(line: str) -> list[tuple[str, str]]:
    hits: list[tuple[str, str]] = []
    for rule_name, pattern, message in SINK_RULES:
        if pattern.search(line):
            hits.append((rule_name, message))
    return hits


def has_word(line: str, token: str) -> bool:
    return re.search(rf"\b{re.escape(token)}\b", line) is not None


def analyze_function(
    relative_path: str,
    block: FunctionBlock,
) -> list[dict[str, str | int | list[str]]]:
    tainted_names: set[str] = set()
    findings: list[dict[str, str | int | list[str]]] = []

    for line_number, line in block.lines:
        if is_comment_only(line):
            continue

        source_tokens = line_source_tokens(line)
        assigned_name = extract_assigned_name(line)
        if source_tokens and assigned_name is not None:
            tainted_names.add(assigned_name)

        if assigned_name is not None and not source_tokens:
            for tainted_name in sorted(tainted_names):
                if has_word(line, tainted_name):
                    tainted_names.add(assigned_name)
                    break

        sink_hits = sink_hits_for_line(line)
        if not sink_hits:
            continue

        if source_tokens:
            for rule_name, message in sink_hits:
                findings.append(
                    {
                        "file": relative_path,
                        "function": block.name,
                        "line": line_number,
                        "rule": rule_name,
                        "message": message,
                        "source_tokens": source_tokens,
                        "tainted_aliases": [],
                        "snippet": line.strip(),
                    }
                )
            continue

        used_aliases = [
            alias for alias in sorted(tainted_names) if has_word(line, alias)
        ]
        if used_aliases:
            for rule_name, message in sink_hits:
                findings.append(
                    {
                        "file": relative_path,
                        "function": block.name,
                        "line": line_number,
                        "rule": rule_name,
                        "message": message,
                        "source_tokens": [],
                        "tainted_aliases": used_aliases,
                        "snippet": line.strip(),
                    }
                )

    return findings


def main() -> int:
    args = parse_args()
    source_root = Path(args.source_root).resolve()
    source_paths = tuple(args.source_paths or DEFAULT_SOURCE_PATHS)
    out_report = Path(args.out_report).resolve()
    out_detail_report = Path(args.out_detail_report).resolve()
    violations_out = Path(args.violations_out).resolve()

    violations: list[str] = []
    checked_files: list[str] = []
    correlation_hits: list[dict[str, str | int | list[str]]] = []

    for relative_path in source_paths:
        path = source_root / relative_path
        if not path.is_file():
            violations.append(f"missing_required_source:{relative_path}")
            continue
        checked_files.append(relative_path)
        functions = parse_functions(path.read_text(encoding="utf-8"))
        for block in functions:
            correlation_hits.extend(analyze_function(relative_path, block))

    for hit in correlation_hits:
        alias_fragment = ",".join(hit["tainted_aliases"]) if hit["tainted_aliases"] else "-"
        source_fragment = ",".join(hit["source_tokens"]) if hit["source_tokens"] else "-"
        violations.append(
            "forbidden_diagnostics_callsite_correlation:"
            f"{hit['file']}:{hit['function']}:{hit['line']}:{hit['rule']}:{source_fragment}:{alias_fragment}"
        )

    detail_report = {
        "status": "PASS" if not violations else "FAIL",
        "mode": "phase13_diagnostics_callsite_correlation_gate",
        "source_root": source_root.as_posix(),
        "required_source_count": len(source_paths),
        "checked_file_count": len(checked_files),
        "checked_files": checked_files,
        "protected_source_tokens": list(PROTECTED_SOURCE_TOKENS),
        "sink_rules": [
            {"rule": rule_name, "description": message}
            for rule_name, _pattern, message in SINK_RULES
        ],
        "violation_matrix": list(VIOLATION_MATRIX),
        "correlation_hit_count": len(correlation_hits),
        "correlation_hits": correlation_hits,
        "violations": violations,
        "violations_count": len(violations),
    }

    gate_report = {
        "gate": "diagnostics-callsite-correlation",
        "mode": "phase13_diagnostics_callsite_correlation_gate",
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
