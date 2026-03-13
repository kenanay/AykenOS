#!/usr/bin/env python3
"""Validate Phase-11 DLT bootstrap monotonicity from ETI evidence."""

from __future__ import annotations

# Author: Kenan AY

import argparse
import json
from pathlib import Path
from typing import Any


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Validate DLT bootstrap monotonicity and emit ltick trace."
    )
    parser.add_argument("--eti-jsonl", required=True, help="eti_transcript.jsonl path")
    parser.add_argument("--out-ltick-trace", required=True, help="Output ltick_trace.jsonl path")
    parser.add_argument("--out-report", required=True, help="Output report.json path")
    return parser.parse_args()


def write_json(path: Path, payload: dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def write_jsonl(path: Path, rows: list[dict[str, Any]]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("w", encoding="utf-8") as fh:
        for row in rows:
            fh.write(json.dumps(row, sort_keys=True) + "\n")


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


def fail(report_path: Path, trace_path: Path, report: dict[str, Any]) -> int:
    report["verdict"] = "FAIL"
    report["violations_count"] = len(report.get("violations", []))
    write_jsonl(trace_path, [])
    write_json(report_path, report)
    return 2


def pass_(report_path: Path, trace_path: Path, report: dict[str, Any], trace_rows: list[dict[str, Any]]) -> int:
    report["verdict"] = "PASS"
    report["violations"] = []
    report["violations_count"] = 0
    write_jsonl(trace_path, trace_rows)
    write_json(report_path, report)
    return 0


def main() -> int:
    args = parse_args()

    eti_jsonl_path = Path(args.eti_jsonl)
    ltick_trace_path = Path(args.out_ltick_trace)
    report_path = Path(args.out_report)

    report: dict[str, Any] = {
        "gate": "dlt-monotonicity",
        "mode": "bootstrap_materialized_from_eti",
        "eti_jsonl": str(eti_jsonl_path),
        "ltick_trace_jsonl": str(ltick_trace_path),
        "violations": [],
    }

    if not eti_jsonl_path.is_file():
        report["violations"].append(f"missing_eti_jsonl:{eti_jsonl_path}")
        return fail(report_path, ltick_trace_path, report)

    try:
        eti_rows = load_jsonl(eti_jsonl_path)
    except RuntimeError as exc:
        report["violations"].append(str(exc))
        return fail(report_path, ltick_trace_path, report)

    if not eti_rows:
        report["violations"].append("empty_eti_stream")

    trace_rows: list[dict[str, Any]] = []
    source_event_seq_values: list[int] = []
    source_ltick_values: list[int] = []

    for idx, row in enumerate(eti_rows, start=1):
        if row.get("event_seq") in (None, ""):
            report["violations"].append(f"missing_source_event_seq:entry={idx}")
            continue
        if row.get("ltick") in (None, ""):
            report["violations"].append(f"missing_source_ltick:entry={idx}")
            continue

        try:
            source_event_seq = int(row["event_seq"])
            source_ltick = int(row["ltick"])
            cpu_id = int(row.get("cpu_id", 0) or 0)
        except Exception:
            report["violations"].append(f"invalid_source_ordering_fields:entry={idx}")
            continue

        source_event_seq_values.append(source_event_seq)
        source_ltick_values.append(source_ltick)

        generated_seq = idx
        trace_rows.append(
            {
                "event_seq": generated_seq,
                "ltick": generated_seq,
                "source_event_seq": source_event_seq,
                "source_ltick": source_ltick,
                "cpu_id": cpu_id,
                "event_type": str(row.get("event_type", "")),
            }
        )

    if source_event_seq_values != sorted(source_event_seq_values):
        report["violations"].append("source_event_seq_non_monotonic")
    if len(set(source_event_seq_values)) != len(source_event_seq_values):
        report["violations"].append("source_event_seq_duplicate")

    if source_ltick_values != sorted(source_ltick_values):
        report["violations"].append("source_ltick_non_monotonic")
    if len(set(source_ltick_values)) != len(source_ltick_values):
        report["violations"].append("source_ltick_duplicate")

    dlt_event_seq_values = [int(row["event_seq"]) for row in trace_rows]
    dlt_ltick_values = [int(row["ltick"]) for row in trace_rows]

    expected_range = list(range(1, len(trace_rows) + 1))
    if dlt_event_seq_values != expected_range:
        report["violations"].append("dlt_event_seq_gap")
    if dlt_ltick_values != expected_range:
        report["violations"].append("dlt_ltick_gap")
    if dlt_event_seq_values != sorted(dlt_event_seq_values):
        report["violations"].append("dlt_event_seq_non_monotonic")
    if dlt_ltick_values != sorted(dlt_ltick_values):
        report["violations"].append("dlt_ltick_non_monotonic")
    if len(set(dlt_event_seq_values)) != len(dlt_event_seq_values):
        report["violations"].append("dlt_event_seq_duplicate")
    if len(set(dlt_ltick_values)) != len(dlt_ltick_values):
        report["violations"].append("dlt_ltick_duplicate")

    report["eti_event_count"] = len(eti_rows)
    report["dlt_trace_count"] = len(trace_rows)
    report["first_generated_ltick"] = dlt_ltick_values[0] if dlt_ltick_values else 0
    report["last_generated_ltick"] = dlt_ltick_values[-1] if dlt_ltick_values else 0
    report["source_event_seq_first"] = source_event_seq_values[0] if source_event_seq_values else 0
    report["source_event_seq_last"] = source_event_seq_values[-1] if source_event_seq_values else 0
    report["source_ltick_first"] = source_ltick_values[0] if source_ltick_values else 0
    report["source_ltick_last"] = source_ltick_values[-1] if source_ltick_values else 0

    if report["violations"]:
        report["verdict"] = "FAIL"
        report["violations_count"] = len(report["violations"])
        write_jsonl(ltick_trace_path, trace_rows)
        write_json(report_path, report)
        return 2

    return pass_(report_path, ltick_trace_path, report, trace_rows)


if __name__ == "__main__":
    raise SystemExit(main())
