#!/usr/bin/env python3
"""Validate Phase-11 DEOL sequence invariants from ledger evidence."""

from __future__ import annotations

# Author: Kenan AY

import argparse
import json
from pathlib import Path
from typing import Any


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Validate DEOL bootstrap sequence and emit ordering evidence."
    )
    parser.add_argument("--ledger-jsonl", required=True, help="decision_ledger.jsonl path")
    parser.add_argument("--out-event-seq", required=True, help="Output event_seq.jsonl path")
    parser.add_argument(
        "--out-sequence-report", required=True, help="Output sequence_report.json path"
    )
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


def load_ledger(path: Path) -> list[dict[str, Any]]:
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
                    f"ledger_parse_error:{path}:line={line_no}:{type(exc).__name__}"
                ) from exc
            if not isinstance(row, dict):
                raise RuntimeError(f"ledger_type_error:{path}:line={line_no}")
            rows.append(row)
    return rows


def summarize_sequence(seq: list[int]) -> tuple[int, int, int]:
    duplicates = 0
    gaps = 0
    seen: set[int] = set()
    expected = seq[0] if seq else 1

    for current in seq:
        if current in seen:
            duplicates += 1
        seen.add(current)
        if current != expected:
            gaps += 1
            expected = current + 1
        else:
            expected += 1

    return duplicates, gaps, len(seen)


def main() -> int:
    args = parse_args()

    ledger_jsonl_path = Path(args.ledger_jsonl)
    out_event_seq_path = Path(args.out_event_seq)
    out_sequence_report_path = Path(args.out_sequence_report)
    out_report_path = Path(args.out_report)

    report: dict[str, Any] = {
        "gate": "deol-sequence",
        "ledger_jsonl": str(ledger_jsonl_path),
        "violations": [],
    }

    if not ledger_jsonl_path.is_file():
        report["violations"].append(f"missing_ledger_jsonl:{ledger_jsonl_path}")
        sequence_report = {
            "status": "FAIL",
            "mode": "bootstrap_materialized_from_ledger",
            "total_events": 0,
            "first_seq": 0,
            "last_seq": 0,
            "duplicates": 0,
            "gaps": 0,
            "violations": list(report["violations"]),
        }
        write_jsonl(out_event_seq_path, [])
        write_json(out_sequence_report_path, sequence_report)
        report["verdict"] = "FAIL"
        report["violations_count"] = len(report["violations"])
        write_json(out_report_path, report)
        return 2

    try:
        ledger_rows = load_ledger(ledger_jsonl_path)
    except RuntimeError as exc:
        report["violations"].append(str(exc))
        sequence_report = {
            "status": "FAIL",
            "mode": "bootstrap_materialized_from_ledger",
            "total_events": 0,
            "first_seq": 0,
            "last_seq": 0,
            "duplicates": 0,
            "gaps": 0,
            "violations": list(report["violations"]),
        }
        write_jsonl(out_event_seq_path, [])
        write_json(out_sequence_report_path, sequence_report)
        report["verdict"] = "FAIL"
        report["violations_count"] = len(report["violations"])
        write_json(out_report_path, report)
        return 2

    if not ledger_rows:
        report["violations"].append("empty_ledger")

    event_seq_rows: list[dict[str, Any]] = []
    source_event_seq_values: list[int] = []
    source_ltick_values: list[int] = []

    for idx, row in enumerate(ledger_rows, start=1):
        if "event_seq" not in row:
            report["violations"].append(f"missing_event_seq:entry={idx}")
            continue
        if "ltick" not in row:
            report["violations"].append(f"missing_ltick:entry={idx}")
            continue

        try:
            source_event_seq = int(row.get("event_seq"))
            source_ltick = int(row.get("ltick"))
        except Exception:
            report["violations"].append(f"invalid_ordering_fields:entry={idx}")
            continue

        source_event_seq_values.append(source_event_seq)
        source_ltick_values.append(source_ltick)

        event_seq_rows.append(
            {
                "event_seq": idx,
                "ltick": idx,
                "source_event_seq": source_event_seq,
                "source_ltick": source_ltick,
                "event_type": str(row.get("event_type", "")),
            }
        )

    deol_seq_values = [int(row["event_seq"]) for row in event_seq_rows]
    deol_ltick_values = [int(row["ltick"]) for row in event_seq_rows]

    deol_duplicates, deol_gaps, deol_unique = summarize_sequence(deol_seq_values)
    if deol_duplicates:
        report["violations"].append(f"deol_event_seq_duplicate:count={deol_duplicates}")
    if deol_gaps:
        report["violations"].append(f"deol_event_seq_gap:count={deol_gaps}")

    deol_ltick_duplicates, deol_ltick_gaps, _ = summarize_sequence(deol_ltick_values)
    if deol_ltick_duplicates:
        report["violations"].append(
            f"deol_ltick_duplicate:count={deol_ltick_duplicates}"
        )
    if deol_ltick_gaps:
        report["violations"].append(f"deol_ltick_gap:count={deol_ltick_gaps}")

    if source_event_seq_values != sorted(source_event_seq_values):
        report["violations"].append("source_event_seq_non_monotonic")
    if len(set(source_event_seq_values)) != len(source_event_seq_values):
        report["violations"].append("source_event_seq_duplicate")

    if source_ltick_values != sorted(source_ltick_values):
        report["violations"].append("source_ltick_non_monotonic")
    if len(set(source_ltick_values)) != len(source_ltick_values):
        report["violations"].append("source_ltick_duplicate")

    write_jsonl(out_event_seq_path, event_seq_rows)

    sequence_report = {
        "status": "FAIL" if report["violations"] else "PASS",
        "mode": "bootstrap_materialized_from_ledger",
        "total_events": len(event_seq_rows),
        "first_seq": deol_seq_values[0] if deol_seq_values else 0,
        "last_seq": deol_seq_values[-1] if deol_seq_values else 0,
        "duplicates": deol_duplicates,
        "gaps": deol_gaps,
        "first_ltick": deol_ltick_values[0] if deol_ltick_values else 0,
        "last_ltick": deol_ltick_values[-1] if deol_ltick_values else 0,
        "source_event_seq_first": source_event_seq_values[0] if source_event_seq_values else 0,
        "source_event_seq_last": source_event_seq_values[-1] if source_event_seq_values else 0,
        "source_ltick_first": source_ltick_values[0] if source_ltick_values else 0,
        "source_ltick_last": source_ltick_values[-1] if source_ltick_values else 0,
        "unique_count": deol_unique,
        "violations": list(report["violations"]),
        "violations_count": len(report["violations"]),
    }

    write_json(out_sequence_report_path, sequence_report)

    report["verdict"] = "FAIL" if report["violations"] else "PASS"
    report["violations_count"] = len(report["violations"])
    report["total_events"] = len(event_seq_rows)
    report["first_seq"] = sequence_report["first_seq"]
    report["last_seq"] = sequence_report["last_seq"]
    report["first_ltick"] = sequence_report["first_ltick"]
    report["last_ltick"] = sequence_report["last_ltick"]
    write_json(out_report_path, report)

    return 2 if report["violations"] else 0


if __name__ == "__main__":
    raise SystemExit(main())
