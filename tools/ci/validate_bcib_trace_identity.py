#!/usr/bin/env python3
"""Validate Phase-11 BCIB plan + execution trace identity."""

from __future__ import annotations

# Author: Kenan AY

import argparse
import hashlib
import json
from pathlib import Path
from typing import Any


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Validate BCIB plan hash and ETI-derived execution trace hash identity."
    )
    parser.add_argument("--bcib-plan-bin", required=True, help="BCIB plan binary path")
    parser.add_argument("--eti-jsonl", required=True, help="ETI transcript jsonl path")
    parser.add_argument("--out-plan-hash-txt", required=True, help="Output bcib_plan_hash.txt path")
    parser.add_argument(
        "--out-execution-trace-jsonl", required=True, help="Output execution_trace.jsonl path"
    )
    parser.add_argument(
        "--out-execution-trace-hash-txt",
        required=True,
        help="Output execution_trace_hash.txt path",
    )
    parser.add_argument("--out-trace-verify-json", required=True, help="Output trace_verify.json path")
    parser.add_argument("--out-report", required=True, help="Output report.json path")
    parser.add_argument(
        "--expected-plan-hash-file",
        required=False,
        default="",
        help="Optional expected BCIB hash file (first token is consumed)",
    )
    parser.add_argument(
        "--expected-trace-hash-file",
        required=False,
        default="",
        help="Optional expected trace hash file (first token is consumed)",
    )
    return parser.parse_args()


def write_json(path: Path, payload: dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def write_text(path: Path, value: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text((value or "") + "\n", encoding="utf-8")


def sha256_hex(payload: bytes) -> str:
    return hashlib.sha256(payload).hexdigest()


def is_sha256_hex(value: str) -> bool:
    if not isinstance(value, str) or len(value) != 64:
        return False
    return all(ch in "0123456789abcdef" for ch in value.lower())


def normalize_expected_hash(raw_text: str) -> str:
    for line in raw_text.splitlines():
        tokenized = line.strip()
        if not tokenized:
            continue
        return tokenized.split()[0].strip().lower()
    return ""


def load_jsonl(path: Path) -> list[dict[str, Any]]:
    rows: list[dict[str, Any]] = []
    with path.open("r", encoding="utf-8", errors="replace") as fh:
        for line_no, raw in enumerate(fh, start=1):
            line = raw.strip()
            if not line:
                continue
            try:
                row = json.loads(line)
            except Exception as exc:  # pragma: no cover
                raise RuntimeError(
                    f"eti_parse_error:{path}:line={line_no}:{type(exc).__name__}"
                ) from exc
            if not isinstance(row, dict):
                raise RuntimeError(f"eti_type_error:{path}:line={line_no}")
            rows.append(row)
    return rows


def serialize_trace_rows(path: Path, rows: list[dict[str, Any]]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("w", encoding="utf-8") as fh:
        for row in rows:
            fh.write(json.dumps(row, sort_keys=True, separators=(",", ":")) + "\n")


def fail(
    report_path: Path,
    plan_hash_path: Path,
    trace_path: Path,
    trace_hash_path: Path,
    trace_verify_path: Path,
    report: dict[str, Any],
    trace_rows: list[dict[str, Any]],
) -> int:
    report["verdict"] = "FAIL"
    report["violations_count"] = len(report.get("violations", []))
    write_json(report_path, report)
    write_text(plan_hash_path, str(report.get("bcib_plan_hash", "")))
    serialize_trace_rows(trace_path, trace_rows)
    write_text(trace_hash_path, str(report.get("execution_trace_hash", "")))

    trace_verify_payload = {
        "status": "FAIL",
        "mode": "bootstrap_bcib_trace_identity",
        "trace_entry_count": int(report.get("trace_entry_count", 0)),
        "bcib_plan_hash": str(report.get("bcib_plan_hash", "")),
        "execution_trace_hash": str(report.get("execution_trace_hash", "")),
        "plan_hash_recomputed_match": bool(report.get("plan_hash_recomputed_match", False)),
        "trace_hash_recomputed_match": bool(report.get("trace_hash_recomputed_match", False)),
        "expected_plan_hash": str(report.get("expected_plan_hash", "")),
        "expected_plan_hash_match": bool(report.get("expected_plan_hash_match", False)),
        "expected_trace_hash": str(report.get("expected_trace_hash", "")),
        "expected_trace_hash_match": bool(report.get("expected_trace_hash_match", False)),
        "violations": list(report.get("violations", [])),
        "violations_count": len(report.get("violations", [])),
    }
    write_json(trace_verify_path, trace_verify_payload)
    return 2


def pass_(
    report_path: Path,
    plan_hash_path: Path,
    trace_path: Path,
    trace_hash_path: Path,
    trace_verify_path: Path,
    report: dict[str, Any],
    trace_rows: list[dict[str, Any]],
    trace_verify_payload: dict[str, Any],
) -> int:
    report["verdict"] = "PASS"
    report["violations"] = []
    report["violations_count"] = 0
    write_json(report_path, report)
    write_text(plan_hash_path, str(report.get("bcib_plan_hash", "")))
    serialize_trace_rows(trace_path, trace_rows)
    write_text(trace_hash_path, str(report.get("execution_trace_hash", "")))
    write_json(trace_verify_path, trace_verify_payload)
    return 0


def load_expected_hash(path: Path, label: str, report: dict[str, Any]) -> str:
    if not path.is_file():
        report["violations"].append(f"missing_expected_{label}_hash_file:{path}")
        return ""
    try:
        raw = path.read_text(encoding="utf-8", errors="replace")
    except Exception as exc:  # pragma: no cover
        report["violations"].append(
            f"expected_{label}_hash_read_error:{path}:{type(exc).__name__}"
        )
        return ""

    normalized = normalize_expected_hash(raw)
    if not normalized:
        report["violations"].append(f"empty_expected_{label}_hash_file:{path}")
        return ""
    if not is_sha256_hex(normalized):
        report["violations"].append(
            f"invalid_expected_{label}_hash_format:{path}:{normalized}"
        )
        return ""
    return normalized


def main() -> int:
    args = parse_args()

    bcib_plan_path = Path(args.bcib_plan_bin)
    eti_jsonl_path = Path(args.eti_jsonl)
    plan_hash_path = Path(args.out_plan_hash_txt)
    trace_path = Path(args.out_execution_trace_jsonl)
    trace_hash_path = Path(args.out_execution_trace_hash_txt)
    trace_verify_path = Path(args.out_trace_verify_json)
    report_path = Path(args.out_report)
    expected_plan_hash_path = (
        Path(args.expected_plan_hash_file) if str(args.expected_plan_hash_file).strip() else None
    )
    expected_trace_hash_path = (
        Path(args.expected_trace_hash_file) if str(args.expected_trace_hash_file).strip() else None
    )

    report: dict[str, Any] = {
        "gate": "bcib-trace-identity",
        "mode": "bootstrap_execution_identity",
        "bcib_plan_bin": str(bcib_plan_path),
        "eti_jsonl": str(eti_jsonl_path),
        "expected_plan_hash_file": str(expected_plan_hash_path) if expected_plan_hash_path else "",
        "expected_trace_hash_file": str(expected_trace_hash_path) if expected_trace_hash_path else "",
        "violations": [],
    }

    if not bcib_plan_path.is_file():
        report["violations"].append(f"missing_bcib_plan_bin:{bcib_plan_path}")
        return fail(
            report_path, plan_hash_path, trace_path, trace_hash_path, trace_verify_path, report, []
        )
    if not eti_jsonl_path.is_file():
        report["violations"].append(f"missing_eti_jsonl:{eti_jsonl_path}")
        return fail(
            report_path, plan_hash_path, trace_path, trace_hash_path, trace_verify_path, report, []
        )

    try:
        bcib_plan_bytes = bcib_plan_path.read_bytes()
    except Exception as exc:  # pragma: no cover
        report["violations"].append(
            f"bcib_plan_read_error:{bcib_plan_path}:{type(exc).__name__}"
        )
        return fail(
            report_path, plan_hash_path, trace_path, trace_hash_path, trace_verify_path, report, []
        )
    if len(bcib_plan_bytes) == 0:
        report["violations"].append("empty_bcib_plan_bin")
        return fail(
            report_path, plan_hash_path, trace_path, trace_hash_path, trace_verify_path, report, []
        )

    try:
        eti_rows = load_jsonl(eti_jsonl_path)
    except RuntimeError as exc:
        report["violations"].append(str(exc))
        return fail(
            report_path, plan_hash_path, trace_path, trace_hash_path, trace_verify_path, report, []
        )
    if not eti_rows:
        report["violations"].append("empty_eti_jsonl")
        return fail(
            report_path, plan_hash_path, trace_path, trace_hash_path, trace_verify_path, report, []
        )

    trace_rows: list[dict[str, Any]] = []
    event_seq_values: list[int] = []
    ltick_values: list[int] = []
    for idx, row in enumerate(eti_rows, start=1):
        for field in ("event_seq", "ltick", "event_type"):
            if row.get(field) in (None, ""):
                report["violations"].append(f"missing_eti_field:{field}:entry={idx}")
        if row.get("event_seq") in (None, "") or row.get("ltick") in (None, ""):
            continue

        try:
            event_seq = int(row["event_seq"])
            ltick = int(row["ltick"])
            cpu_id = int(row.get("cpu_id", 0) or 0)
            event_type = str(row.get("event_type", ""))
        except Exception:
            report["violations"].append(f"invalid_eti_row_fields:entry={idx}")
            continue

        event_seq_values.append(event_seq)
        ltick_values.append(ltick)
        trace_rows.append(
            {
                "trace_seq": len(trace_rows) + 1,
                "event_seq": event_seq,
                "ltick": ltick,
                "cpu_id": cpu_id,
                "event_type": event_type,
            }
        )

    if not trace_rows:
        report["violations"].append("empty_execution_trace")

    if event_seq_values != sorted(event_seq_values):
        report["violations"].append("execution_trace_event_seq_non_monotonic")
    if len(set(event_seq_values)) != len(event_seq_values):
        report["violations"].append("execution_trace_event_seq_duplicate")
    if ltick_values != sorted(ltick_values):
        report["violations"].append("execution_trace_ltick_non_monotonic")
    if len(set(ltick_values)) != len(ltick_values):
        report["violations"].append("execution_trace_ltick_duplicate")

    plan_hash = sha256_hex(bcib_plan_bytes)
    plan_recomputed_hash = ""
    try:
        plan_recomputed_hash = sha256_hex(bcib_plan_path.read_bytes())
    except Exception as exc:  # pragma: no cover
        report["violations"].append(
            f"bcib_plan_reread_error:{bcib_plan_path}:{type(exc).__name__}"
        )
    plan_hash_recomputed_match = bool(plan_recomputed_hash) and plan_hash == plan_recomputed_hash
    if not plan_hash_recomputed_match:
        report["violations"].append("bcib_plan_hash_recompute_mismatch")

    serialize_trace_rows(trace_path, trace_rows)
    trace_bytes = trace_path.read_bytes() if trace_path.is_file() else b""
    execution_trace_hash = sha256_hex(trace_bytes) if trace_bytes else ""
    trace_recomputed_hash = ""
    try:
        trace_recomputed_hash = sha256_hex(trace_path.read_bytes())
    except Exception as exc:  # pragma: no cover
        report["violations"].append(f"execution_trace_reread_error:{trace_path}:{type(exc).__name__}")
    trace_hash_recomputed_match = (
        bool(trace_recomputed_hash)
        and bool(execution_trace_hash)
        and trace_recomputed_hash == execution_trace_hash
    )
    if not trace_hash_recomputed_match:
        report["violations"].append("execution_trace_hash_recompute_mismatch")

    expected_plan_hash = ""
    expected_plan_hash_match = False
    if expected_plan_hash_path is not None:
        expected_plan_hash = load_expected_hash(expected_plan_hash_path, "plan", report)
        if expected_plan_hash:
            expected_plan_hash_match = expected_plan_hash == plan_hash
            if not expected_plan_hash_match:
                report["violations"].append(
                    f"bcib_plan_hash_mismatch:expected={expected_plan_hash}:actual={plan_hash}"
                )

    expected_trace_hash = ""
    expected_trace_hash_match = False
    if expected_trace_hash_path is not None:
        expected_trace_hash = load_expected_hash(expected_trace_hash_path, "trace", report)
        if expected_trace_hash:
            expected_trace_hash_match = expected_trace_hash == execution_trace_hash
            if not expected_trace_hash_match:
                report["violations"].append(
                    "execution_trace_hash_mismatch:"
                    f"expected={expected_trace_hash}:actual={execution_trace_hash}"
                )

    report["bcib_plan_size_bytes"] = len(bcib_plan_bytes)
    report["trace_entry_count"] = len(trace_rows)
    report["bcib_plan_hash"] = plan_hash
    report["execution_trace_hash"] = execution_trace_hash
    report["plan_hash_recomputed_match"] = plan_hash_recomputed_match
    report["trace_hash_recomputed_match"] = trace_hash_recomputed_match
    report["expected_plan_hash"] = expected_plan_hash
    report["expected_plan_hash_match"] = expected_plan_hash_match
    report["expected_trace_hash"] = expected_trace_hash
    report["expected_trace_hash_match"] = expected_trace_hash_match

    trace_verify_payload = {
        "status": "FAIL" if report["violations"] else "PASS",
        "mode": "bootstrap_bcib_trace_identity",
        "trace_entry_count": len(trace_rows),
        "bcib_plan_hash": plan_hash,
        "execution_trace_hash": execution_trace_hash,
        "plan_hash_recomputed_match": plan_hash_recomputed_match,
        "trace_hash_recomputed_match": trace_hash_recomputed_match,
        "expected_plan_hash": expected_plan_hash,
        "expected_plan_hash_match": expected_plan_hash_match,
        "expected_trace_hash": expected_trace_hash,
        "expected_trace_hash_match": expected_trace_hash_match,
        "violations": list(report["violations"]),
        "violations_count": len(report["violations"]),
    }

    if report["violations"]:
        return fail(
            report_path,
            plan_hash_path,
            trace_path,
            trace_hash_path,
            trace_verify_path,
            report,
            trace_rows,
        )
    return pass_(
        report_path,
        plan_hash_path,
        trace_path,
        trace_hash_path,
        trace_verify_path,
        report,
        trace_rows,
        trace_verify_payload,
    )


if __name__ == "__main__":
    raise SystemExit(main())
