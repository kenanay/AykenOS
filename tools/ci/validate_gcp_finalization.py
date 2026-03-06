#!/usr/bin/env python3
"""Validate Phase-11 bootstrap GCP finalization contract from DLT evidence."""

from __future__ import annotations

# Author: Kenan AY

import argparse
import hashlib
import json
from pathlib import Path
from typing import Any


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Validate bootstrap GCP invariants and emit finalization evidence."
    )
    parser.add_argument("--dlt-trace-jsonl", required=True, help="ltick_trace.jsonl path")
    parser.add_argument("--out-gcp-snapshot", required=True, help="Output gcp_snapshot.json path")
    parser.add_argument("--out-gcp-record", required=True, help="Output gcp_record.json path")
    parser.add_argument(
        "--out-gcp-consistency-report",
        required=True,
        help="Output gcp_consistency_report.json path",
    )
    parser.add_argument("--out-report", required=True, help="Output report.json path")
    parser.add_argument(
        "--previous-gcp",
        required=False,
        default="",
        help="Optional previous gcp_snapshot.json path for monotonicity check",
    )
    return parser.parse_args()


def write_json(path: Path, payload: dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")


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
                    f"dlt_parse_error:{path}:line={line_no}:{type(exc).__name__}"
                ) from exc
            if not isinstance(row, dict):
                raise RuntimeError(f"dlt_type_error:{path}:line={line_no}")
            rows.append(row)
    return rows


def sha256_hex(payload: bytes) -> str:
    return hashlib.sha256(payload).hexdigest()


def canonical_dlt_row(row: dict[str, Any]) -> bytes:
    payload = {
        "event_seq": int(row["event_seq"]),
        "ltick": int(row["ltick"]),
        "source_event_seq": int(row.get("source_event_seq", row["event_seq"])),
        "source_ltick": int(row.get("source_ltick", row["ltick"])),
        "cpu_id": int(row.get("cpu_id", 0) or 0),
        "event_type": str(row.get("event_type", "")),
    }
    return json.dumps(payload, sort_keys=True, separators=(",", ":")).encode("utf-8")


def fail(
    report_path: Path,
    snapshot_path: Path,
    record_path: Path,
    consistency_path: Path,
    report: dict[str, Any],
) -> int:
    report["verdict"] = "FAIL"
    report["violations_count"] = len(report.get("violations", []))
    write_json(report_path, report)

    snapshot_payload = {
        "mode": "bootstrap",
        "status": "FAIL",
        "gcp_ltick": int(report.get("gcp_ltick", 0)),
        "gcp_event_seq": int(report.get("gcp_event_seq", 0)),
        "dlt_event_count": int(report.get("dlt_event_count", 0)),
        "dlt_trace_hash": str(report.get("dlt_trace_hash", "")),
        "dlt_prefix_hash": str(report.get("dlt_prefix_hash", "")),
        "violations": list(report.get("violations", [])),
        "violations_count": len(report.get("violations", [])),
    }
    write_json(snapshot_path, snapshot_payload)

    record_payload = {
        "status": "FAIL",
        "gcp_ltick": int(report.get("gcp_ltick", 0)),
        "gcp_event_seq": int(report.get("gcp_event_seq", 0)),
        "dlt_trace_hash": str(report.get("dlt_trace_hash", "")),
        "violations": list(report.get("violations", [])),
        "violations_count": len(report.get("violations", [])),
    }
    write_json(record_path, record_payload)

    consistency_payload = {
        "status": "FAIL",
        "mode": "bootstrap_gcp_finalization",
        "dlt_event_count": int(report.get("dlt_event_count", 0)),
        "gcp_ltick": int(report.get("gcp_ltick", 0)),
        "gcp_event_seq": int(report.get("gcp_event_seq", 0)),
        "prefix_immutable": bool(report.get("prefix_immutable", False)),
        "dlt_prefix_alignment": bool(report.get("dlt_prefix_alignment", False)),
        "violations": list(report.get("violations", [])),
        "violations_count": len(report.get("violations", [])),
    }
    write_json(consistency_path, consistency_payload)
    return 2


def pass_(
    report_path: Path,
    snapshot_path: Path,
    record_path: Path,
    consistency_path: Path,
    report: dict[str, Any],
    snapshot_payload: dict[str, Any],
    record_payload: dict[str, Any],
    consistency_payload: dict[str, Any],
) -> int:
    report["verdict"] = "PASS"
    report["violations"] = []
    report["violations_count"] = 0
    write_json(report_path, report)
    write_json(snapshot_path, snapshot_payload)
    write_json(record_path, record_payload)
    write_json(consistency_path, consistency_payload)
    return 0


def main() -> int:
    args = parse_args()

    dlt_trace_path = Path(args.dlt_trace_jsonl)
    snapshot_path = Path(args.out_gcp_snapshot)
    record_path = Path(args.out_gcp_record)
    consistency_path = Path(args.out_gcp_consistency_report)
    report_path = Path(args.out_report)
    previous_gcp_path = Path(args.previous_gcp) if str(args.previous_gcp).strip() else None

    report: dict[str, Any] = {
        "gate": "gcp-finalization",
        "mode": "bootstrap_gcp_from_dlt",
        "dlt_trace_jsonl": str(dlt_trace_path),
        "gcp_snapshot_json": str(snapshot_path),
        "gcp_record_json": str(record_path),
        "gcp_consistency_report_json": str(consistency_path),
        "violations": [],
    }

    if not dlt_trace_path.is_file():
        report["violations"].append(f"missing_dlt_trace_jsonl:{dlt_trace_path}")
        return fail(report_path, snapshot_path, record_path, consistency_path, report)

    try:
        dlt_rows = load_jsonl(dlt_trace_path)
    except RuntimeError as exc:
        report["violations"].append(str(exc))
        return fail(report_path, snapshot_path, record_path, consistency_path, report)

    if not dlt_rows:
        report["violations"].append("empty_dlt_trace")

    event_seq_values: list[int] = []
    ltick_values: list[int] = []
    canonical_blobs: list[bytes] = []

    for idx, row in enumerate(dlt_rows, start=1):
        required_fields = ("event_seq", "ltick")
        for field in required_fields:
            if row.get(field) in (None, ""):
                report["violations"].append(f"missing_dlt_field:{field}:entry={idx}")
        if any(row.get(field) in (None, "") for field in required_fields):
            continue

        try:
            event_seq = int(row["event_seq"])
            ltick = int(row["ltick"])
            canonical_blob = canonical_dlt_row(row)
        except Exception:
            report["violations"].append(f"invalid_dlt_row_fields:entry={idx}")
            continue

        event_seq_values.append(event_seq)
        ltick_values.append(ltick)
        canonical_blobs.append(canonical_blob)

    expected_range = list(range(1, len(event_seq_values) + 1))
    if event_seq_values != expected_range:
        report["violations"].append("dlt_event_seq_gap")
    if ltick_values != expected_range:
        report["violations"].append("dlt_ltick_gap")
    if event_seq_values != sorted(event_seq_values):
        report["violations"].append("dlt_event_seq_non_monotonic")
    if ltick_values != sorted(ltick_values):
        report["violations"].append("dlt_ltick_non_monotonic")
    if len(set(event_seq_values)) != len(event_seq_values):
        report["violations"].append("dlt_event_seq_duplicate")
    if len(set(ltick_values)) != len(ltick_values):
        report["violations"].append("dlt_ltick_duplicate")

    gcp_ltick = ltick_values[-1] if ltick_values else 0
    gcp_event_seq = event_seq_values[-1] if event_seq_values else 0
    dlt_event_count = len(event_seq_values)

    dlt_trace_hash = sha256_hex(b"".join(canonical_blobs)) if canonical_blobs else sha256_hex(b"")
    prefix_blobs = [
        canonical_blobs[idx]
        for idx, row in enumerate(dlt_rows)
        if idx < len(ltick_values) and int(ltick_values[idx]) <= gcp_ltick
    ]
    dlt_prefix_hash = sha256_hex(b"".join(prefix_blobs)) if prefix_blobs else sha256_hex(b"")

    prefix_immutable = gcp_ltick == max(ltick_values) if ltick_values else False
    dlt_prefix_alignment = gcp_ltick in set(ltick_values) if ltick_values else False
    if not prefix_immutable:
        report["violations"].append("gcp_prefix_not_immutable")
    if not dlt_prefix_alignment:
        report["violations"].append("gcp_ltick_not_in_dlt_trace")

    previous_gcp_ltick = None
    if previous_gcp_path is not None:
        if not previous_gcp_path.is_file():
            report["violations"].append(f"missing_previous_gcp:{previous_gcp_path}")
        else:
            try:
                previous_payload = json.loads(previous_gcp_path.read_text(encoding="utf-8"))
                previous_gcp_ltick = int(previous_payload.get("gcp_ltick", 0))
            except Exception:
                report["violations"].append(f"invalid_previous_gcp:{previous_gcp_path}")
            else:
                if gcp_ltick < previous_gcp_ltick:
                    report["violations"].append(
                        f"gcp_non_monotonic_previous:prev={previous_gcp_ltick}:current={gcp_ltick}"
                    )

    report["dlt_event_count"] = dlt_event_count
    report["gcp_ltick"] = gcp_ltick
    report["gcp_event_seq"] = gcp_event_seq
    report["dlt_trace_hash"] = dlt_trace_hash
    report["dlt_prefix_hash"] = dlt_prefix_hash
    report["prefix_immutable"] = prefix_immutable
    report["dlt_prefix_alignment"] = dlt_prefix_alignment
    report["previous_gcp_ltick"] = previous_gcp_ltick

    snapshot_payload = {
        "mode": "bootstrap",
        "status": "FAIL" if report["violations"] else "PASS",
        "gcp_ltick": gcp_ltick,
        "gcp_event_seq": gcp_event_seq,
        "dlt_event_count": dlt_event_count,
        "dlt_trace_hash": dlt_trace_hash,
        "dlt_prefix_hash": dlt_prefix_hash,
        "previous_gcp_ltick": previous_gcp_ltick,
        "prefix_immutable": prefix_immutable,
        "dlt_prefix_alignment": dlt_prefix_alignment,
        "violations": list(report["violations"]),
        "violations_count": len(report["violations"]),
    }
    record_payload = {
        "status": "FAIL" if report["violations"] else "PASS",
        "gcp_ltick": gcp_ltick,
        "gcp_event_seq": gcp_event_seq,
        "dlt_trace_hash": dlt_trace_hash,
        "dlt_event_count": dlt_event_count,
        "violations": list(report["violations"]),
        "violations_count": len(report["violations"]),
    }
    consistency_payload = {
        "status": "FAIL" if report["violations"] else "PASS",
        "mode": "bootstrap_gcp_finalization",
        "dlt_event_count": dlt_event_count,
        "gcp_ltick": gcp_ltick,
        "gcp_event_seq": gcp_event_seq,
        "prefix_immutable": prefix_immutable,
        "dlt_prefix_alignment": dlt_prefix_alignment,
        "previous_gcp_ltick": previous_gcp_ltick,
        "violations": list(report["violations"]),
        "violations_count": len(report["violations"]),
    }

    if report["violations"]:
        return fail(report_path, snapshot_path, record_path, consistency_path, report)
    return pass_(
        report_path,
        snapshot_path,
        record_path,
        consistency_path,
        report,
        snapshot_payload,
        record_payload,
        consistency_payload,
    )


if __name__ == "__main__":
    raise SystemExit(main())
