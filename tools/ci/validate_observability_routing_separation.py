#!/usr/bin/env python3
"""Validate that observability artifacts do not influence verification routing."""

from __future__ import annotations

import argparse
import json
import re
import sys
from dataclasses import dataclass
from pathlib import Path


DEFAULT_SCAN_ROOTS = (
    "ayken-core/crates/proof-verifier",
    "userspace/proofd",
)

PROTECTED_OBSERVABILITY_TOKENS = (
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
    "parity_authority_suppression_report.json",
    "parity_convergence_report.json",
    "parity_drift_attribution_report.json",
    "suppressed_drift_count",
    "suppression_guard_active",
)

PROTECTED_OBSERVABILITY_MODULE_TOKENS = (
    "authority_drift_topology",
    "determinism_incident",
    "drift_attribution",
    "incident_graph",
)

ROUTING_CONTEXT_FUNCTION_RE = re.compile(
    r"^\s*(?:pub\s+)?(?:async\s+)?fn\s+"
    r"([A-Za-z_][A-Za-z0-9_]*(?:route|routing|schedule|scheduling|select_verifier|"
    r"choose_verifier|prefer_verifier|verifier_order|preferred_node)[A-Za-z0-9_]*)\b"
)

ROUTING_SINK_RE = re.compile(
    r"\b(route_verification|verification_route|routing_hint|schedule_verification|"
    r"schedule_next_verifier|select_verifier|choose_verifier|prefer_verifier|"
    r"set_preferred_node|set_verifier_order|set_verification_weight)\b"
)

FORBIDDEN_HEURISTIC_PATTERNS = (
    (
        "agreement_bias",
        re.compile(r"\b(agreement_ratio|agreement_likelihood|likely_agreement)\b"),
        "routing or scheduling must not optimize for agreement likelihood",
    ),
    (
        "dominance_bias",
        re.compile(
            r"\b(dominant_cluster|dominant_authority|dominant_authority_chain_id|"
            r"largest_outcome_cluster_size|outcome_convergence_ratio)\b"
        ),
        "routing or scheduling must not optimize around dominant topology or convergence signals",
    ),
    (
        "reliability_bias",
        re.compile(r"\b(reliability_score|stability_score|lowest_divergence|preferred_cluster)\b"),
        "routing or scheduling must not optimize around reliability or stability heuristics derived from observability",
    ),
)

LET_ASSIGNMENT_RE = re.compile(
    r"\blet\s+(?:mut\s+)?([A-Za-z_][A-Za-z0-9_]*)\b(?:\s*:\s*[^=]+)?\s*="
)
PLAIN_ASSIGNMENT_RE = re.compile(r"\b([A-Za-z_][A-Za-z0-9_]*)\s*=")
FN_RE = re.compile(r"^\s*(?:pub\s+)?(?:async\s+)?fn\s+([A-Za-z_][A-Za-z0-9_]*)\b")

VIOLATION_MATRIX = (
    {
        "case_id": "P13-FEED-01",
        "rule": "descriptive observability fields must not become verifier ordering, preferred-node, or first-hop routing input",
    },
    {
        "case_id": "P13-FEED-02",
        "rule": "topology or convergence observability must not bias verification diversity or routing order",
    },
    {
        "case_id": "P13-FEED-03",
        "rule": "suppression or island diagnostics must not become runtime scheduling or orchestration control",
    },
    {
        "case_id": "P13-FEED-04",
        "rule": "verification scheduling must optimize for diversity, not agreement likelihood or dominant-cluster recurrence",
    },
    {
        "case_id": "P13-FEED-05",
        "rule": "routing or scheduling code must not import observability modules directly",
    },
)


@dataclass
class FunctionBlock:
    name: str
    start_line: int
    lines: list[tuple[int, str]]


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Validate that observability artifacts do not influence verification routing or scheduling."
    )
    parser.add_argument("--source-root", required=True, help="Repository or fixture root to scan.")
    parser.add_argument(
        "--scan-root",
        action="append",
        dest="scan_roots",
        help="Relative source root to scan recursively for Rust sources. Defaults to proof-verifier and proofd trees.",
    )
    parser.add_argument(
        "--source-path",
        action="append",
        dest="source_paths",
        help="Relative Rust source path to scan directly. If omitted, scan roots are used.",
    )
    parser.add_argument("--out-report", required=True, help="Output gate report.json path.")
    parser.add_argument("--out-detail-report", required=True, help="Output detailed report path.")
    parser.add_argument("--out-negative-matrix", required=True, help="Output negative matrix report path.")
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


def extract_assigned_name(line: str) -> str | None:
    match = LET_ASSIGNMENT_RE.search(line)
    if match is not None:
        return match.group(1)
    match = PLAIN_ASSIGNMENT_RE.search(line)
    if match is not None:
        return match.group(1)
    return None


def has_word(line: str, token: str) -> bool:
    return re.search(rf"\b{re.escape(token)}\b", line) is not None


def source_tokens_for_line(line: str) -> list[str]:
    return [token for token in PROTECTED_OBSERVABILITY_TOKENS if token in line]


def observability_modules_for_line(line: str) -> list[str]:
    if not re.match(r"^\s*use\b", line):
        return []
    return [token for token in PROTECTED_OBSERVABILITY_MODULE_TOKENS if token in line]


def heuristic_hits_for_line(line: str) -> list[tuple[str, str]]:
    hits: list[tuple[str, str]] = []
    for rule_name, pattern, message in FORBIDDEN_HEURISTIC_PATTERNS:
        if pattern.search(line):
            hits.append((rule_name, message))
    return hits


def is_routing_context(block: FunctionBlock) -> bool:
    header = block.lines[0][1]
    if ROUTING_CONTEXT_FUNCTION_RE.match(header):
        return True
    return any(ROUTING_SINK_RE.search(line) for _, line in block.lines if not is_comment_only(line))


def analyze_function(relative_path: str, block: FunctionBlock) -> list[dict[str, object]]:
    findings: list[dict[str, object]] = []
    tainted_names: set[str] = set()
    routing_context = is_routing_context(block)

    for line_number, line in block.lines:
        if is_comment_only(line):
            continue

        source_tokens = source_tokens_for_line(line)
        assigned_name = extract_assigned_name(line)
        if source_tokens and assigned_name is not None:
            tainted_names.add(assigned_name)

        if assigned_name is not None and not source_tokens:
            for tainted_name in sorted(tainted_names):
                if has_word(line, tainted_name):
                    tainted_names.add(assigned_name)
                    break

        used_aliases = [alias for alias in sorted(tainted_names) if has_word(line, alias)]

        if routing_context and (source_tokens or used_aliases):
            findings.append(
                {
                    "file": relative_path,
                    "function": block.name,
                    "line": line_number,
                    "rule": "routing_blindness",
                    "message": "routing or scheduling surfaces must remain observability blind",
                    "source_tokens": source_tokens,
                    "tainted_aliases": used_aliases,
                    "snippet": line.strip(),
                }
            )

        if routing_context:
            for rule_name, message in heuristic_hits_for_line(line):
                findings.append(
                    {
                        "file": relative_path,
                        "function": block.name,
                        "line": line_number,
                        "rule": rule_name,
                        "message": message,
                        "source_tokens": source_tokens,
                        "tainted_aliases": used_aliases,
                        "snippet": line.strip(),
                    }
                )

    deduped: list[dict[str, object]] = []
    seen: set[tuple[object, ...]] = set()
    for finding in findings:
        key = (
            finding["file"],
            finding["function"],
            finding["line"],
            finding["rule"],
            finding["snippet"],
        )
        if key in seen:
            continue
        seen.add(key)
        deduped.append(finding)
    return deduped


def file_import_findings(
    relative_path: str,
    lines: list[str],
    routing_functions: list[FunctionBlock],
) -> list[dict[str, object]]:
    if not routing_functions:
        return []

    findings: list[dict[str, object]] = []
    for line_number, line in enumerate(lines, start=1):
        if is_comment_only(line):
            continue
        modules = observability_modules_for_line(line)
        if not modules:
            continue
        findings.append(
            {
                "file": relative_path,
                "function": "<file_import>",
                "line": line_number,
                "rule": "observability_module_import",
                "message": "routing or scheduling code must not import observability modules directly",
                "source_tokens": modules,
                "tainted_aliases": [],
                "snippet": line.strip(),
            }
        )
    return findings


def iter_source_files(source_root: Path, scan_roots: tuple[str, ...], source_paths: list[str] | None) -> list[tuple[str, Path]]:
    files: list[tuple[str, Path]] = []
    seen: set[str] = set()

    if source_paths:
        for relative_path in source_paths:
            path = (source_root / relative_path).resolve()
            rel = Path(relative_path).as_posix()
            if rel in seen:
                continue
            seen.add(rel)
            files.append((rel, path))
        return files

    for relative_root in scan_roots:
        scan_root = source_root / relative_root
        if not scan_root.is_dir():
            continue
        for path in sorted(scan_root.rglob("*.rs")):
            relative_path = path.relative_to(source_root).as_posix()
            if relative_path in seen:
                continue
            seen.add(relative_path)
            files.append((relative_path, path))
    return files


def main() -> int:
    args = parse_args()
    source_root = Path(args.source_root).resolve()
    scan_roots = tuple(args.scan_roots or DEFAULT_SCAN_ROOTS)
    source_paths = list(args.source_paths or [])
    out_report = Path(args.out_report).resolve()
    out_detail_report = Path(args.out_detail_report).resolve()
    out_negative_matrix = Path(args.out_negative_matrix).resolve()
    violations_out = Path(args.violations_out).resolve()

    checked_files: list[str] = []
    missing_paths: list[str] = []
    routing_functions: list[dict[str, object]] = []
    correlation_hits: list[dict[str, object]] = []

    for relative_path, path in iter_source_files(source_root, scan_roots, source_paths):
        checked_files.append(relative_path)
        if not path.is_file():
            missing_paths.append(relative_path)
            continue
        text = path.read_text(encoding="utf-8")
        lines = text.splitlines()
        parsed_functions = parse_functions(text)
        routing_blocks: list[FunctionBlock] = []
        for block in parsed_functions:
            if not is_routing_context(block):
                continue
            routing_blocks.append(block)
            routing_functions.append(
                {
                    "file": relative_path,
                    "function": block.name,
                    "start_line": block.start_line,
                }
            )
            correlation_hits.extend(analyze_function(relative_path, block))
        correlation_hits.extend(file_import_findings(relative_path, lines, routing_blocks))

    violations = [f"missing_source_path:{path}" for path in missing_paths]
    for hit in correlation_hits:
        violations.append(
            "observability_routing_separation_violation:"
            f"{hit['file']}:{hit['line']}:{hit['rule']}"
        )

    detail_report = {
        "status": "PASS" if not violations else "FAIL",
        "mode": "phase13_observability_routing_separation_gate",
        "source_root": source_root.as_posix(),
        "scan_roots": list(scan_roots),
        "source_paths": source_paths,
        "checked_file_count": len(checked_files),
        "checked_files": checked_files,
        "protected_observability_tokens": list(PROTECTED_OBSERVABILITY_TOKENS),
        "protected_observability_modules": list(PROTECTED_OBSERVABILITY_MODULE_TOKENS),
        "routing_context_function_pattern": ROUTING_CONTEXT_FUNCTION_RE.pattern,
        "routing_sink_pattern": ROUTING_SINK_RE.pattern,
        "forbidden_heuristics": [name for name, _, _ in FORBIDDEN_HEURISTIC_PATTERNS],
        "routing_function_count": len(routing_functions),
        "routing_functions": routing_functions,
        "correlation_hit_count": len(correlation_hits),
        "correlation_hits": correlation_hits,
        "missing_source_paths": missing_paths,
        "violations": violations,
        "violations_count": len(violations),
    }

    negative_matrix = {
        "mode": "phase13_observability_routing_separation_gate",
        "violation_matrix": list(VIOLATION_MATRIX),
        "evaluated_routing_function_count": len(routing_functions),
        "correlation_hit_count": len(correlation_hits),
        "violations_count": len(violations),
    }

    gate_report = {
        "gate": "observability-routing-separation",
        "mode": "phase13_observability_routing_separation_gate",
        "verdict": "PASS" if not violations else "FAIL",
        "detail_report_path": out_detail_report.name,
        "negative_matrix_path": out_negative_matrix.name,
        "violations": violations,
        "violations_count": len(violations),
    }

    write_json(out_detail_report, detail_report)
    write_json(out_negative_matrix, negative_matrix)
    write_json(out_report, gate_report)
    write_violations(violations_out, violations)
    return 0 if not violations else 2


if __name__ == "__main__":
    sys.exit(main())
