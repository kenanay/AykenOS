#!/usr/bin/env python3
"""Validate Phase-11 ledger <-> ETI strict binding."""

from __future__ import annotations

# Author: Kenan AY

import argparse
import json
from pathlib import Path
from typing import Any

DECISION_EVENT_TYPES = {
    "AY_EVT_CTX_SWITCH",
    "AY_EVT_MAILBOX_ACCEPT",
    "AY_EVT_MAILBOX_REJECT",
    "AY_EVT_POLICY_SWAP",
}


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Validate strict event_seq/ltick ledger<->ETI binding.")
    parser.add_argument("--ledger-jsonl", required=True, help="decision_ledger.jsonl path")
    parser.add_argument("--eti-jsonl", required=True, help="eti_transcript.jsonl path")
    parser.add_argument(
        "--out-binding-report", required=True, help="Output binding_report.json path"
    )
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
                raise RuntimeError(f"{name}_parse_error:{path}:line={line_no}:{type(exc).__name__}") from exc
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
        "mode": "strict_event_seq_ltick_binding",
        "ledger_entries": int(report.get("ledger_entries", 0)),
        "decision_entries": int(report.get("decision_entries", 0)),
        "matched_entries": int(report.get("matched_entries", 0)),
        "missing_bindings": int(report.get("missing_bindings", 0)),
        "ltick_mismatch_count": int(report.get("ltick_mismatch_count", 0)),
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

    ledger_jsonl_path = Path(args.ledger_jsonl)
    eti_jsonl_path = Path(args.eti_jsonl)
    binding_report_path = Path(args.out_binding_report)
    report_path = Path(args.out_report)

    report: dict[str, Any] = {
        "gate": "ledger-eti-binding",
        "ledger_jsonl": str(ledger_jsonl_path),
        "eti_jsonl": str(eti_jsonl_path),
        "binding_mode": "strict_event_seq_ltick",
        "violations": [],
    }

    if not ledger_jsonl_path.is_file():
        report["violations"].append(f"missing_ledger_jsonl:{ledger_jsonl_path}")
    if not eti_jsonl_path.is_file():
        report["violations"].append(f"missing_eti_jsonl:{eti_jsonl_path}")
    if report["violations"]:
        return fail(report_path, binding_report_path, report)

    try:
        ledger_rows = load_jsonl(ledger_jsonl_path, "ledger")
    except RuntimeError as exc:
        report["violations"].append(str(exc))
        return fail(report_path, binding_report_path, report)

    try:
        eti_rows = load_jsonl(eti_jsonl_path, "eti")
    except RuntimeError as exc:
        report["violations"].append(str(exc))
        return fail(report_path, binding_report_path, report)

    report["ledger_entries"] = len(ledger_rows)
    report["eti_entries"] = len(eti_rows)

    if not ledger_rows:
        report["violations"].append("empty_ledger")
    if not eti_rows:
        report["violations"].append("empty_eti")

    eti_by_seq: dict[int, dict[str, Any]] = {}
    for idx, row in enumerate(eti_rows, start=1):
        if "event_seq" not in row or "ltick" not in row:
            report["violations"].append(f"missing_eti_ordering_fields:entry={idx}")
            continue
        try:
            event_seq = int(row["event_seq"])
            ltick = int(row["ltick"])
        except Exception:
            report["violations"].append(f"invalid_eti_ordering_fields:entry={idx}")
            continue
        if event_seq in eti_by_seq:
            report["violations"].append(f"duplicate_eti_event_seq:{event_seq}")
        eti_by_seq[event_seq] = dict(row)
        eti_by_seq[event_seq]["ltick"] = ltick

    decision_rows: list[dict[str, Any]] = []
    for idx, row in enumerate(ledger_rows, start=1):
        event_type = str(row.get("event_type", ""))
        if event_type in DECISION_EVENT_TYPES:
            row_copy = dict(row)
            row_copy["__idx"] = idx
            decision_rows.append(row_copy)

    report["decision_entries"] = len(decision_rows)
    if len(decision_rows) == 0:
        report["violations"].append("empty_decision_event_stream")

    missing_bindings = 0
    ltick_mismatch_count = 0
    event_type_mismatch_count = 0
    matched_entries = 0

    for row in decision_rows:
        idx = int(row["__idx"])
        if "event_seq" not in row or "ltick" not in row:
            report["violations"].append(f"missing_ledger_ordering_fields:entry={idx}")
            continue
        try:
            event_seq = int(row["event_seq"])
            ltick = int(row["ltick"])
        except Exception:
            report["violations"].append(f"invalid_ledger_ordering_fields:entry={idx}")
            continue

        eti_row = eti_by_seq.get(event_seq)
        if eti_row is None:
            missing_bindings += 1
            report["violations"].append(f"missing_eti_binding:event_seq={event_seq}")
            continue

        if int(eti_row.get("ltick", -1)) != ltick:
            ltick_mismatch_count += 1
            report["violations"].append(
                f"ltick_binding_mismatch:event_seq={event_seq}:ledger_ltick={ltick}:eti_ltick={eti_row.get('ltick')}"
            )

        ledger_event_type = str(row.get("event_type", ""))
        eti_event_type = str(eti_row.get("event_type", ""))
        if ledger_event_type == "AY_EVT_CTX_SWITCH":
            allowed_event_types = {"AY_EVT_CTX_SWITCH"}
            if str(row.get("origin_marker", "")) == "P10_MAILBOX_DECISION" or str(
                row.get("origin_event_type", "")
            ) == "P10_MAILBOX_DECISION":
                # #35 bootstrap can bind ctx-switch ledger rows to mailbox-origin ETI.
                allowed_event_types.add("AY_EVT_MAILBOX_ACCEPT")
            if eti_event_type not in allowed_event_types:
                event_type_mismatch_count += 1
                report["violations"].append(
                    f"event_type_mismatch:event_seq={event_seq}:ledger={ledger_event_type}:eti={eti_event_type}:allowed={sorted(allowed_event_types)}"
                )
        elif ledger_event_type != eti_event_type:
            event_type_mismatch_count += 1
            report["violations"].append(
                f"event_type_mismatch:event_seq={event_seq}:ledger={ledger_event_type}:eti={eti_event_type}"
            )

        matched_entries += 1

    report["matched_entries"] = matched_entries
    report["missing_bindings"] = missing_bindings
    report["ltick_mismatch_count"] = ltick_mismatch_count
    report["event_type_mismatch_count"] = event_type_mismatch_count

    binding_payload = {
        "status": "FAIL" if report["violations"] else "PASS",
        "mode": "strict_event_seq_ltick_binding",
        "ledger_entries": len(ledger_rows),
        "decision_entries": len(decision_rows),
        "eti_entries": len(eti_rows),
        "matched_entries": matched_entries,
        "missing_bindings": missing_bindings,
        "ltick_mismatch_count": ltick_mismatch_count,
        "event_type_mismatch_count": event_type_mismatch_count,
        "violations": list(report["violations"]),
        "violations_count": len(report["violations"]),
    }

    if report["violations"]:
        return fail(report_path, binding_report_path, report)
    return pass_(report_path, binding_report_path, report, binding_payload)


if __name__ == "__main__":
    raise SystemExit(main())
