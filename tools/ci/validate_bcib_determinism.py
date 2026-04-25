#!/usr/bin/env python3
"""Validate two-run BCIB kernel determinism over result artifacts."""

from __future__ import annotations

# Author: Kenan AY

import argparse
import hashlib
import json
import re
import shutil
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any


REQUIRED_MARKERS = (
    "submit_bind",
    "queue_create",
    "dequeue_hit",
    "pickup",
    "result_va",
    "wait_ok",
    "result_ok",
)

FALLBACK_PATTERNS = (
    re.compile(r"reason=fallback[a-z_]*", re.IGNORECASE),
    re.compile(r"\[\[AYKEN_PERF_MB_PATH\]\]\s+name=fallback", re.IGNORECASE),
    re.compile(r"\[\[AYKEN_PERF_MB_PHASE\]\]\s+name=[^\n]*fallback", re.IGNORECASE),
)

BOUNDARY_PATTERNS = (
    re.compile(r"boundary_violation", re.IGNORECASE),
    re.compile(r"\[BOUNDARY_[^\]]*\]", re.IGNORECASE),
    re.compile(r"immediate_termination", re.IGNORECASE),
    re.compile(r"\[TERMINAT[^\]]*\]", re.IGNORECASE),
)

PF_PATTERNS = (
    re.compile(r"PF!"),
    re.compile(r"\[PF\]"),
    re.compile(r"page fault", re.IGNORECASE),
)

# shared/abi/execution_output_abi.h -> ayken_execution_output_v1_t
RESULT_HEADER_SIZE = 48


@dataclass
class RunEvidence:
    label: str
    run_dir: Path
    summary_path: Path
    summary: dict[str, Any] = field(default_factory=dict)
    trace_path: Path | None = None
    trace_window_lines: list[str] = field(default_factory=list)
    trace_window_start: int | None = None
    trace_window_end: int | None = None
    trace_window_sha256: str = ""
    result_path: Path | None = None
    result_sha256: str = ""
    result_artifact_size: int = 0
    hash_artifact_path: Path | None = None
    hash_artifact_sha256: str = ""
    fixture_sha256: str = ""
    bcib_sha256: str = ""
    canonical_plan_fingerprint: str = ""
    canonical_binding_fingerprint: str = ""
    result_size: int = -1
    result_fingerprint: str = ""
    expected_sidecar_digest: str = ""
    hash_header_digest: str = ""
    pf_count: int = -1
    boundary_hits: list[str] = field(default_factory=list)
    fallback_hits: list[str] = field(default_factory=list)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Validate same BCIB -> same kernel result determinism over two runs."
    )
    parser.add_argument("--run-a-dir", required=True, help="Directory containing run-1 artifacts")
    parser.add_argument("--run-b-dir", required=True, help="Directory containing run-2 artifacts")
    parser.add_argument("--out-run-a-json", required=True, help="Output normalized run-1 summary path")
    parser.add_argument("--out-run-b-json", required=True, help="Output normalized run-2 summary path")
    parser.add_argument("--out-trace-run-a", required=True, help="Output trace window for run-1")
    parser.add_argument("--out-trace-run-b", required=True, help="Output trace window for run-2")
    parser.add_argument("--out-result-bin", required=True, help="Output canonical result.bin path")
    parser.add_argument("--out-result-sha256", required=True, help="Output canonical result.sha256 path")
    parser.add_argument(
        "--out-result-metadata", required=True, help="Output canonical result_metadata.json path"
    )
    parser.add_argument(
        "--out-comparison-log", required=True, help="Output result_sha256_comparison.log path"
    )
    parser.add_argument(
        "--out-determinism-evidence",
        required=True,
        help="Output bcib_kernel_determinism_evidence.json path",
    )
    parser.add_argument("--out-report", required=True, help="Output report.json path")
    return parser.parse_args()


def write_json(path: Path, payload: dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def write_text(path: Path, value: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(value, encoding="utf-8")


def sha256_hex_bytes(payload: bytes) -> str:
    return hashlib.sha256(payload).hexdigest()


def sha256_hex_file(path: Path) -> str:
    return sha256_hex_bytes(path.read_bytes())


def is_sha256_hex(value: str) -> bool:
    if not isinstance(value, str) or len(value) != 64:
        return False
    return all(ch in "0123456789abcdef" for ch in value.lower())


def normalize_sha256(value: Any) -> str:
    if not isinstance(value, str):
        return ""
    token = value.strip().lower()
    return token if is_sha256_hex(token) else ""


def load_json(path: Path) -> dict[str, Any]:
    payload = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(payload, dict):
        raise RuntimeError(f"json_not_object:{path}")
    return payload


def resolve_summary_path(
    run_dir: Path,
    raw_value: Any,
    default_name: str,
) -> Path:
    if isinstance(raw_value, str) and raw_value.strip():
        raw_path = Path(raw_value.strip())
        if raw_path.is_absolute():
            return raw_path
        for candidate in (run_dir / raw_path, Path.cwd() / raw_path, run_dir / raw_path.name):
            if candidate.exists():
                return candidate
        return run_dir / raw_path
    return run_dir / default_name


def coerce_non_negative_int(value: Any) -> int:
    if isinstance(value, bool):
        return int(value)
    if isinstance(value, int):
        return value if value >= 0 else -1
    if isinstance(value, str) and value.isdigit():
        return int(value)
    return -1


def marker_line(summary: dict[str, Any], marker_name: str) -> int:
    markers = summary.get("markers")
    if not isinstance(markers, dict):
        return -1
    marker = markers.get(marker_name)
    if not isinstance(marker, dict):
        return -1
    return coerce_non_negative_int(marker.get("line"))


def find_trace_hits(
    lines: list[str],
    start_line: int,
    patterns: tuple[re.Pattern[str], ...],
) -> list[str]:
    hits: list[str] = []
    for offset, line in enumerate(lines, start=start_line):
        for pattern in patterns:
            if pattern.search(line):
                hits.append(f"line={offset}:{line.rstrip()}")
                break
    return hits


def load_run(run_dir: Path, label: str, report: dict[str, Any]) -> RunEvidence:
    summary_path = run_dir / "run_summary.json"
    run = RunEvidence(label=label, run_dir=run_dir, summary_path=summary_path)

    if not summary_path.is_file():
        report["violations"].append(f"missing_run_summary:{label}:{summary_path}")
        return run

    try:
        summary = load_json(summary_path)
    except Exception as exc:
        report["violations"].append(f"run_summary_read_error:{label}:{type(exc).__name__}")
        return run

    run.summary = summary
    if str(summary.get("result", "")).upper() != "PASS":
        report["violations"].append(f"run_not_pass:{label}:{summary.get('result')}")
    failure_code = summary.get("failure_code")
    if failure_code not in (None, ""):
        report["violations"].append(f"run_failure_code_present:{label}:{failure_code}")

    run.fixture_sha256 = normalize_sha256(summary.get("fixture_sha256"))
    if not run.fixture_sha256:
        report["violations"].append(f"missing_fixture_sha256:{label}")

    fixture_metadata = summary.get("fixture_metadata")
    if not isinstance(fixture_metadata, dict):
        fixture_metadata = {}
        report["violations"].append(f"missing_fixture_metadata:{label}")
    run.bcib_sha256 = normalize_sha256(fixture_metadata.get("bcib_sha256"))
    if not run.bcib_sha256:
        report["violations"].append(f"missing_bcib_sha256:{label}")
    run.canonical_plan_fingerprint = normalize_sha256(
        fixture_metadata.get("canonical_plan_fingerprint")
    )
    if not run.canonical_plan_fingerprint:
        report["violations"].append(f"missing_canonical_plan_fingerprint:{label}")
    run.canonical_binding_fingerprint = normalize_sha256(
        fixture_metadata.get("canonical_binding_fingerprint")
    )
    if not run.canonical_binding_fingerprint:
        report["violations"].append(f"missing_canonical_binding_fingerprint:{label}")

    marker_lines: list[int] = []
    for marker_name in REQUIRED_MARKERS:
        line = marker_line(summary, marker_name)
        if line <= 0:
            report["violations"].append(f"missing_marker:{label}:{marker_name}")
            continue
        marker_lines.append(line)
        if marker_name == "submit_bind":
            run.trace_window_start = line
        if marker_name == "result_ok":
            run.trace_window_end = line
    if marker_lines and marker_lines != sorted(marker_lines):
        report["violations"].append(f"marker_order_invalid:{label}")

    run.trace_path = resolve_summary_path(run_dir, summary.get("trace_file"), "debugcon.trace")
    if not run.trace_path.is_file():
        report["violations"].append(f"missing_trace_file:{label}:{run.trace_path}")
    else:
        trace_lines = run.trace_path.read_text(encoding="utf-8", errors="replace").splitlines(True)
        if run.trace_window_start is None or run.trace_window_end is None:
            report["violations"].append(f"missing_trace_window_bounds:{label}")
        elif (
            run.trace_window_start < 1
            or run.trace_window_end < run.trace_window_start
            or run.trace_window_end > len(trace_lines)
        ):
            report["violations"].append(
                f"trace_window_out_of_bounds:{label}:{run.trace_window_start}:{run.trace_window_end}:{len(trace_lines)}"
            )
        else:
            run.trace_window_lines = trace_lines[
                run.trace_window_start - 1 : run.trace_window_end
            ]
            run.trace_window_sha256 = sha256_hex_bytes("".join(run.trace_window_lines).encode("utf-8"))
            run.fallback_hits = find_trace_hits(
                run.trace_window_lines,
                run.trace_window_start,
                FALLBACK_PATTERNS,
            )
            run.boundary_hits = find_trace_hits(
                run.trace_window_lines,
                run.trace_window_start,
                BOUNDARY_PATTERNS,
            )
            if run.fallback_hits:
                report["violations"].append(
                    f"fallback_path_observed:{label}:count={len(run.fallback_hits)}"
                )
            if run.boundary_hits:
                report["violations"].append(
                    f"boundary_violation_observed:{label}:count={len(run.boundary_hits)}"
                )

    marker_counts = summary.get("marker_counts")
    pf_count = -1
    if isinstance(marker_counts, dict):
        pf_count = coerce_non_negative_int(marker_counts.get("pf"))
    if pf_count < 0 and run.trace_window_lines and run.trace_window_start is not None:
        pf_count = len(find_trace_hits(run.trace_window_lines, run.trace_window_start, PF_PATTERNS))
    run.pf_count = pf_count
    if run.pf_count < 0:
        report["violations"].append(f"missing_pf_evidence:{label}")
    elif run.pf_count != 0:
        report["violations"].append(f"pf_observed:{label}:count={run.pf_count}")

    run.result_path = resolve_summary_path(run_dir, summary.get("result_artifact"), "result.bin")
    if not run.result_path.is_file():
        report["violations"].append(f"missing_result_artifact:{label}:{run.result_path}")
    else:
        run.result_sha256 = sha256_hex_file(run.result_path)
        run.result_artifact_size = run.result_path.stat().st_size

    run.hash_artifact_path = resolve_summary_path(
        run_dir, summary.get("hash_artifact"), "result_hash.bin"
    )
    if not run.hash_artifact_path.is_file():
        report["violations"].append(f"missing_hash_artifact:{label}:{run.hash_artifact_path}")
    else:
        run.hash_artifact_sha256 = sha256_hex_file(run.hash_artifact_path)

    result_header = summary.get("result_header")
    if not isinstance(result_header, dict):
        result_header = {}
        report["violations"].append(f"missing_result_header:{label}")
    run.result_size = coerce_non_negative_int(result_header.get("bytes_written"))
    if run.result_size < 0:
        report["violations"].append(f"missing_result_size:{label}")
    elif run.result_size == 0:
        report["violations"].append(f"invalid_result_size:{label}:0")
    if run.result_size == 0 and run.result_artifact_size == RESULT_HEADER_SIZE:
        report["violations"].append(
            f"header_only_result:{label}:artifact_size={run.result_artifact_size}"
        )

    run.result_fingerprint = normalize_sha256(summary.get("kernel_result_fingerprint"))
    if not run.result_fingerprint:
        report["violations"].append(f"missing_kernel_result_fingerprint:{label}")

    kernel_result_sha = normalize_sha256(summary.get("kernel_result_sha256"))
    if not kernel_result_sha:
        report["violations"].append(f"missing_kernel_result_sha256:{label}")
    elif run.result_sha256 and kernel_result_sha != run.result_sha256:
        report["violations"].append(f"kernel_result_sha256_mismatch:{label}")

    hash_header = summary.get("hash_header")
    if not isinstance(hash_header, dict):
        hash_header = {}
        report["violations"].append(f"missing_hash_header:{label}")
    run.hash_header_digest = normalize_sha256(hash_header.get("digest_hex"))
    if not run.hash_header_digest:
        report["violations"].append(f"missing_hash_header_digest:{label}")
    elif run.result_fingerprint and run.hash_header_digest != run.result_fingerprint:
        report["violations"].append(f"hash_header_digest_mismatch:{label}")

    run.expected_sidecar_digest = normalize_sha256(summary.get("expected_sidecar_digest"))
    if not run.expected_sidecar_digest:
        report["violations"].append(f"missing_expected_sidecar_digest:{label}")
    elif run.result_fingerprint and run.expected_sidecar_digest != run.result_fingerprint:
        report["violations"].append(f"expected_sidecar_digest_mismatch:{label}")

    if summary.get("hash_sidecar_valid") is not True:
        report["violations"].append(f"hash_sidecar_invalid:{label}")

    return run


def compare_runs(run_a: RunEvidence, run_b: RunEvidence, report: dict[str, Any]) -> None:
    if run_a.fixture_sha256 and run_b.fixture_sha256 and run_a.fixture_sha256 != run_b.fixture_sha256:
        report["violations"].append("fixture_sha256_mismatch")
    if run_a.bcib_sha256 and run_b.bcib_sha256 and run_a.bcib_sha256 != run_b.bcib_sha256:
        report["violations"].append("bcib_sha256_mismatch")
    if (
        run_a.fixture_sha256
        and run_a.bcib_sha256
        and run_a.fixture_sha256 != run_a.bcib_sha256
    ):
        report["violations"].append("fixture_vs_bcib_sha256_mismatch:run_a")
    if (
        run_b.fixture_sha256
        and run_b.bcib_sha256
        and run_b.fixture_sha256 != run_b.bcib_sha256
    ):
        report["violations"].append("fixture_vs_bcib_sha256_mismatch:run_b")

    if (
        run_a.canonical_plan_fingerprint
        and run_b.canonical_plan_fingerprint
        and run_a.canonical_plan_fingerprint != run_b.canonical_plan_fingerprint
    ):
        report["violations"].append("canonical_plan_fingerprint_mismatch")
    if (
        run_a.canonical_binding_fingerprint
        and run_b.canonical_binding_fingerprint
        and run_a.canonical_binding_fingerprint != run_b.canonical_binding_fingerprint
    ):
        report["violations"].append("canonical_binding_fingerprint_mismatch")

    if run_a.result_sha256 and run_b.result_sha256 and run_a.result_sha256 != run_b.result_sha256:
        report["violations"].append("result_sha256_mismatch")
    if run_a.hash_artifact_sha256 and run_b.hash_artifact_sha256 and (
        run_a.hash_artifact_sha256 != run_b.hash_artifact_sha256
    ):
        report["violations"].append("result_hash_artifact_sha256_mismatch")
    if (
        run_a.result_fingerprint
        and run_b.result_fingerprint
        and run_a.result_fingerprint != run_b.result_fingerprint
    ):
        report["violations"].append("kernel_result_fingerprint_mismatch")
    if run_a.result_size >= 0 and run_b.result_size >= 0 and run_a.result_size != run_b.result_size:
        report["violations"].append("result_size_mismatch")


def write_outputs(
    run_a: RunEvidence,
    run_b: RunEvidence,
    out_run_a_json: Path,
    out_run_b_json: Path,
    out_trace_run_a: Path,
    out_trace_run_b: Path,
    out_result_bin: Path,
    out_result_sha256: Path,
    out_result_metadata: Path,
    out_comparison_log: Path,
    out_determinism_evidence: Path,
    out_report: Path,
    report: dict[str, Any],
) -> None:
    write_json(out_run_a_json, run_a.summary if run_a.summary else {"status": "MISSING"})
    write_json(out_run_b_json, run_b.summary if run_b.summary else {"status": "MISSING"})
    write_text(out_trace_run_a, "".join(run_a.trace_window_lines))
    write_text(out_trace_run_b, "".join(run_b.trace_window_lines))

    canonical_result_source = run_a.result_path if run_a.result_path and run_a.result_path.is_file() else None
    if canonical_result_source is None and run_b.result_path and run_b.result_path.is_file():
        canonical_result_source = run_b.result_path
    out_result_bin.parent.mkdir(parents=True, exist_ok=True)
    if canonical_result_source is not None:
        shutil.copyfile(canonical_result_source, out_result_bin)
    else:
        out_result_bin.write_bytes(b"")

    canonical_result_sha256 = run_a.result_sha256 or run_b.result_sha256
    write_text(out_result_sha256, canonical_result_sha256 + ("\n" if canonical_result_sha256 else ""))

    fallback_path = 1 if run_a.fallback_hits or run_b.fallback_hits else 0
    boundary_violation = 1 if run_a.boundary_hits or run_b.boundary_hits else 0
    pf_value = max(run_a.pf_count, run_b.pf_count, 0)
    result_size_value = run_a.result_size if run_a.result_size >= 0 else run_b.result_size
    metadata = {
        "status": report.get("verdict", "FAIL"),
        "closure_verdict": report.get("closure_verdict", "DETERMINISM_FAIL"),
        "bcib_sha256": run_a.bcib_sha256 or run_b.bcib_sha256,
        "result_sha256": canonical_result_sha256,
        "result_size": result_size_value if result_size_value >= 0 else 0,
        "result_artifact_size": run_a.result_artifact_size or run_b.result_artifact_size,
        "payload_non_empty": int(result_size_value > 0),
        "header_only_result": int(
            result_size_value == 0
            and (run_a.result_artifact_size or run_b.result_artifact_size) == RESULT_HEADER_SIZE
        ),
        "result_fingerprint": run_a.result_fingerprint or run_b.result_fingerprint,
        "hash_artifact_sha256": run_a.hash_artifact_sha256 or run_b.hash_artifact_sha256,
        "pf": pf_value,
        "boundary_violation": boundary_violation,
        "fallback_path": fallback_path,
        "run_count": 2,
    }
    write_json(out_result_metadata, metadata)

    comparison_lines = [
        f"run_a_dir={run_a.run_dir}",
        f"run_b_dir={run_b.run_dir}",
        f"run_a_fixture_sha256={run_a.fixture_sha256}",
        f"run_b_fixture_sha256={run_b.fixture_sha256}",
        f"fixture_match={int(bool(run_a.fixture_sha256 and run_a.fixture_sha256 == run_b.fixture_sha256))}",
        f"run_a_bcib_sha256={run_a.bcib_sha256}",
        f"run_b_bcib_sha256={run_b.bcib_sha256}",
        f"bcib_match={int(bool(run_a.bcib_sha256 and run_a.bcib_sha256 == run_b.bcib_sha256))}",
        f"run_a_result_sha256={run_a.result_sha256}",
        f"run_b_result_sha256={run_b.result_sha256}",
        f"result_sha256_match={int(bool(run_a.result_sha256 and run_a.result_sha256 == run_b.result_sha256))}",
        f"run_a_result_fingerprint={run_a.result_fingerprint}",
        f"run_b_result_fingerprint={run_b.result_fingerprint}",
        f"result_fingerprint_match={int(bool(run_a.result_fingerprint and run_a.result_fingerprint == run_b.result_fingerprint))}",
        f"run_a_result_size={run_a.result_size}",
        f"run_b_result_size={run_b.result_size}",
        f"result_size_match={int(run_a.result_size >= 0 and run_a.result_size == run_b.result_size)}",
        f"run_a_payload_non_empty={int(run_a.result_size > 0)}",
        f"run_b_payload_non_empty={int(run_b.result_size > 0)}",
        f"run_a_header_only_result={int(run_a.result_size == 0 and run_a.result_artifact_size == RESULT_HEADER_SIZE)}",
        f"run_b_header_only_result={int(run_b.result_size == 0 and run_b.result_artifact_size == RESULT_HEADER_SIZE)}",
        f"run_a_hash_artifact_sha256={run_a.hash_artifact_sha256}",
        f"run_b_hash_artifact_sha256={run_b.hash_artifact_sha256}",
        f"hash_artifact_match={int(bool(run_a.hash_artifact_sha256 and run_a.hash_artifact_sha256 == run_b.hash_artifact_sha256))}",
        f"run_a_trace_window_sha256={run_a.trace_window_sha256}",
        f"run_b_trace_window_sha256={run_b.trace_window_sha256}",
        f"trace_window_match={int(bool(run_a.trace_window_sha256 and run_a.trace_window_sha256 == run_b.trace_window_sha256))}",
        f"run_a_pf={run_a.pf_count}",
        f"run_b_pf={run_b.pf_count}",
        f"run_a_boundary_violation={int(bool(run_a.boundary_hits))}",
        f"run_b_boundary_violation={int(bool(run_b.boundary_hits))}",
        f"run_a_fallback_path={int(bool(run_a.fallback_hits))}",
        f"run_b_fallback_path={int(bool(run_b.fallback_hits))}",
        f"closure_verdict={report.get('closure_verdict', 'DETERMINISM_FAIL')}",
    ]
    write_text(out_comparison_log, "\n".join(comparison_lines) + "\n")

    evidence = {
        "status": report.get("verdict", "FAIL"),
        "closure_verdict": report.get("closure_verdict", "DETERMINISM_FAIL"),
        "gate": "bcib-determinism",
        "mode": "kernel_result_two_run_parity",
        "fixture_sha256": run_a.fixture_sha256 or run_b.fixture_sha256,
        "bcib_sha256": run_a.bcib_sha256 or run_b.bcib_sha256,
        "canonical_plan_fingerprint": run_a.canonical_plan_fingerprint
        or run_b.canonical_plan_fingerprint,
        "canonical_binding_fingerprint": run_a.canonical_binding_fingerprint
        or run_b.canonical_binding_fingerprint,
        "result_sha256": {
            "run_a": run_a.result_sha256,
            "run_b": run_b.result_sha256,
            "match": bool(run_a.result_sha256 and run_a.result_sha256 == run_b.result_sha256),
        },
        "result_fingerprint": {
            "run_a": run_a.result_fingerprint,
            "run_b": run_b.result_fingerprint,
            "match": bool(
                run_a.result_fingerprint and run_a.result_fingerprint == run_b.result_fingerprint
            ),
        },
        "result_size": {
            "run_a": run_a.result_size,
            "run_b": run_b.result_size,
            "match": run_a.result_size >= 0 and run_a.result_size == run_b.result_size,
        },
        "payload_non_empty": {
            "run_a": int(run_a.result_size > 0),
            "run_b": int(run_b.result_size > 0),
            "match": (run_a.result_size > 0) == (run_b.result_size > 0),
        },
        "header_only_result": {
            "run_a": int(
                run_a.result_size == 0 and run_a.result_artifact_size == RESULT_HEADER_SIZE
            ),
            "run_b": int(
                run_b.result_size == 0 and run_b.result_artifact_size == RESULT_HEADER_SIZE
            ),
        },
        "hash_artifact_sha256": {
            "run_a": run_a.hash_artifact_sha256,
            "run_b": run_b.hash_artifact_sha256,
            "match": bool(
                run_a.hash_artifact_sha256
                and run_a.hash_artifact_sha256 == run_b.hash_artifact_sha256
            ),
        },
        "trace_window_sha256": {
            "run_a": run_a.trace_window_sha256,
            "run_b": run_b.trace_window_sha256,
            "match": bool(
                run_a.trace_window_sha256
                and run_a.trace_window_sha256 == run_b.trace_window_sha256
            ),
        },
        "pf": {"run_a": run_a.pf_count, "run_b": run_b.pf_count},
        "boundary_violation": {
            "run_a": int(bool(run_a.boundary_hits)),
            "run_b": int(bool(run_b.boundary_hits)),
        },
        "fallback_path": {
            "run_a": int(bool(run_a.fallback_hits)),
            "run_b": int(bool(run_b.fallback_hits)),
        },
        "fallback_hits": {"run_a": run_a.fallback_hits, "run_b": run_b.fallback_hits},
        "boundary_hits": {"run_a": run_a.boundary_hits, "run_b": run_b.boundary_hits},
        "violations": list(report.get("violations", [])),
        "violations_count": len(report.get("violations", [])),
    }
    write_json(out_determinism_evidence, evidence)
    write_json(out_report, report)


def main() -> int:
    args = parse_args()

    run_a_dir = Path(args.run_a_dir)
    run_b_dir = Path(args.run_b_dir)
    out_run_a_json = Path(args.out_run_a_json)
    out_run_b_json = Path(args.out_run_b_json)
    out_trace_run_a = Path(args.out_trace_run_a)
    out_trace_run_b = Path(args.out_trace_run_b)
    out_result_bin = Path(args.out_result_bin)
    out_result_sha256 = Path(args.out_result_sha256)
    out_result_metadata = Path(args.out_result_metadata)
    out_comparison_log = Path(args.out_comparison_log)
    out_determinism_evidence = Path(args.out_determinism_evidence)
    out_report = Path(args.out_report)

    report: dict[str, Any] = {
        "gate": "bcib-determinism",
        "mode": "kernel_result_two_run_parity",
        "run_a_dir": str(run_a_dir),
        "run_b_dir": str(run_b_dir),
        "violations": [],
    }

    run_a = load_run(run_a_dir, "run_a", report)
    run_b = load_run(run_b_dir, "run_b", report)
    compare_runs(run_a, run_b, report)

    if report["violations"]:
        report["verdict"] = "FAIL"
        report["closure_verdict"] = "DETERMINISM_FAIL"
    else:
        report["verdict"] = "PASS"
        report["closure_verdict"] = "DETERMINISM_PASS"
    report["violations_count"] = len(report["violations"])

    write_outputs(
        run_a,
        run_b,
        out_run_a_json,
        out_run_b_json,
        out_trace_run_a,
        out_trace_run_b,
        out_result_bin,
        out_result_sha256,
        out_result_metadata,
        out_comparison_log,
        out_determinism_evidence,
        out_report,
        report,
    )
    return 0 if not report["violations"] else 2


if __name__ == "__main__":
    raise SystemExit(main())
