#!/usr/bin/env python3
"""Validate Phase-11 ETI <-> DLT bootstrap source identity binding."""

from __future__ import annotations

# Author: Kenan AY

import argparse
import json
from pathlib import Path
from typing import Any


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Validate strict source event_seq/ltick ETI<->DLT binding."
    )
    parser.add_argument("--eti-jsonl", required=True, help="eti_transcript.jsonl path")
    parser.add_argument("--ltick-trace-jsonl", required=True, help="ltick_trace.jsonl path")
    parser.add_argument("--out-binding-report", required=True, help="Output binding_report.json path")
    parser.add_argument("--out-report", required=True, help="Output report.json path")
    return parser.parse_args()


def write_json(path: Path, payload: dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def load_jsonl(path: Path, name: str) -> list[dict[str, Any]]:
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
                    f"{name}_parse_error:{path}:line={line_no}:{type(exc).__name__}"
                ) from exc
            if not isinstance(row, dict):
                raise RuntimeError(f"{name}_type_error:{path}:line={line_no}")
            rows.append(row)
    return rows


def fail(report_path: Path, binding_path: Path, report: dict[str, Any]) -> int:
    report["verdict"] = "FAIL"
    report["violations_count"] = len(report.get("violations", []))
    write_json(report_path, report)
    binding_payload = {
        "status": "FAIL",
        "mode": "strict_source_event_seq_ltick_binding",
        "eti_entries": int(report.get("eti_entries", 0)),
        "dlt_entries": int(report.get("dlt_entries", 0)),
        "matched_entries": int(report.get("matched_entries", 0)),
        "missing_bindings": int(report.get("missing_bindings", 0)),
        "source_ltick_mismatch_count": int(report.get("source_ltick_mismatch_count", 0)),
        "orphan_dlt_entries": int(report.get("orphan_dlt_entries", 0)),
        "violations": list(report.get("violations", [])),
        "violations_count": len(report.get("violations", [])),
    }
    write_json(binding_path, binding_payload)
    return 2


def pass_(report_path: Path, binding_path: Path, report: dict[str, Any], binding_payload: dict[str, Any]) -> int:
    report["verdict"] = "PASS"
    report["violations"] = []
    report["violations_count"] = 0
    write_json(report_path, report)
    write_json(binding_path, binding_payload)
    return 0


def main() -> int:
    args = parse_args()

    eti_jsonl_path = Path(args.eti_jsonl)
    ltick_trace_path = Path(args.ltick_trace_jsonl)
    binding_report_path = Path(args.out_binding_report)
    report_path = Path(args.out_report)

    report: dict[str, Any] = {
        "gate": "eti-dlt-binding",
        "binding_mode": "strict_source_event_seq_ltick",
        "eti_jsonl": str(eti_jsonl_path),
        "ltick_trace_jsonl": str(ltick_trace_path),
        "violations": [],
    }

    if not eti_jsonl_path.is_file():
        report["violations"].append(f"missing_eti_jsonl:{eti_jsonl_path}")
    if not ltick_trace_path.is_file():
        report["violations"].append(f"missing_ltick_trace_jsonl:{ltick_trace_path}")
    if report["violations"]:
        return fail(report_path, binding_report_path, report)

    try:
        eti_rows = load_jsonl(eti_jsonl_path, "eti")
    except RuntimeError as exc:
        report["violations"].append(str(exc))
        return fail(report_path, binding_report_path, report)

    try:
        dlt_rows = load_jsonl(ltick_trace_path, "dlt")
    except RuntimeError as exc:
        report["violations"].append(str(exc))
        return fail(report_path, binding_report_path, report)

    report["eti_entries"] = len(eti_rows)
    report["dlt_entries"] = len(dlt_rows)

    if not eti_rows:
        report["violations"].append("empty_eti_stream")
    if not dlt_rows:
        report["violations"].append("empty_dlt_stream")

    eti_by_event_seq: dict[int, int] = {}
    for idx, row in enumerate(eti_rows, start=1):
        if row.get("event_seq") in (None, "") or row.get("ltick") in (None, ""):
            report["violations"].append(f"missing_eti_ordering_fields:entry={idx}")
            continue
        try:
            event_seq = int(row["event_seq"])
            ltick = int(row["ltick"])
        except Exception:
            report["violations"].append(f"invalid_eti_ordering_fields:entry={idx}")
            continue
        if event_seq in eti_by_event_seq:
            report["violations"].append(f"duplicate_eti_event_seq:{event_seq}")
        eti_by_event_seq[event_seq] = ltick

    dlt_by_source_event_seq: dict[int, dict[str, int]] = {}
    dlt_generated_lticks: list[int] = []
    dlt_generated_event_seqs: list[int] = []
    for idx, row in enumerate(dlt_rows, start=1):
        required_fields = ("event_seq", "ltick", "source_event_seq", "source_ltick")
        for field in required_fields:
            if row.get(field) in (None, ""):
                report["violations"].append(f"missing_dlt_field:{field}:entry={idx}")
        if any(row.get(field) in (None, "") for field in required_fields):
            continue
        try:
            dlt_event_seq = int(row["event_seq"])
            dlt_ltick = int(row["ltick"])
            source_event_seq = int(row["source_event_seq"])
            source_ltick = int(row["source_ltick"])
        except Exception:
            report["violations"].append(f"invalid_dlt_ordering_fields:entry={idx}")
            continue

        dlt_generated_event_seqs.append(dlt_event_seq)
        dlt_generated_lticks.append(dlt_ltick)

        if source_event_seq in dlt_by_source_event_seq:
            report["violations"].append(f"duplicate_dlt_source_event_seq:{source_event_seq}")
        dlt_by_source_event_seq[source_event_seq] = {
            "dlt_ltick": dlt_ltick,
            "source_ltick": source_ltick,
        }

    expected_range = list(range(1, len(dlt_generated_lticks) + 1))
    if dlt_generated_event_seqs != expected_range:
        report["violations"].append("dlt_event_seq_gap")
    if dlt_generated_lticks != expected_range:
        report["violations"].append("dlt_ltick_gap")
    if dlt_generated_event_seqs != sorted(dlt_generated_event_seqs):
        report["violations"].append("dlt_event_seq_non_monotonic")
    if dlt_generated_lticks != sorted(dlt_generated_lticks):
        report["violations"].append("dlt_ltick_non_monotonic")
    if len(set(dlt_generated_event_seqs)) != len(dlt_generated_event_seqs):
        report["violations"].append("dlt_event_seq_duplicate")
    if len(set(dlt_generated_lticks)) != len(dlt_generated_lticks):
        report["violations"].append("dlt_ltick_duplicate")

    missing_bindings = 0
    source_ltick_mismatch_count = 0
    matched_entries = 0

    for event_seq, eti_ltick in eti_by_event_seq.items():
        dlt_row = dlt_by_source_event_seq.get(event_seq)
        if dlt_row is None:
            missing_bindings += 1
            report["violations"].append(f"missing_dlt_binding:event_seq={event_seq}")
            continue
        if int(dlt_row["source_ltick"]) != int(eti_ltick):
            source_ltick_mismatch_count += 1
            report["violations"].append(
                f"source_ltick_mismatch:event_seq={event_seq}:eti_ltick={eti_ltick}:dlt_source_ltick={dlt_row['source_ltick']}"
            )
        matched_entries += 1

    orphan_dlt_entries = 0
    for source_event_seq in dlt_by_source_event_seq:
        if source_event_seq not in eti_by_event_seq:
            orphan_dlt_entries += 1
            report["violations"].append(f"orphan_dlt_source_event_seq:{source_event_seq}")

    report["matched_entries"] = matched_entries
    report["missing_bindings"] = missing_bindings
    report["source_ltick_mismatch_count"] = source_ltick_mismatch_count
    report["orphan_dlt_entries"] = orphan_dlt_entries

    binding_payload = {
        "status": "FAIL" if report["violations"] else "PASS",
        "mode": "strict_source_event_seq_ltick_binding",
        "eti_entries": len(eti_rows),
        "dlt_entries": len(dlt_rows),
        "matched_entries": matched_entries,
        "missing_bindings": missing_bindings,
        "source_ltick_mismatch_count": source_ltick_mismatch_count,
        "orphan_dlt_entries": orphan_dlt_entries,
        "violations": list(report["violations"]),
        "violations_count": len(report["violations"]),
    }

    if report["violations"]:
        return fail(report_path, binding_report_path, report)
    return pass_(report_path, binding_report_path, report, binding_payload)


if __name__ == "__main__":
    raise SystemExit(main())
