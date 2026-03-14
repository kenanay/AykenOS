#!/usr/bin/env python3
"""Validate and materialize Phase-11 ETI bootstrap evidence."""

from __future__ import annotations

# Author: Kenan AY

import argparse
import hashlib
import json
import struct
from pathlib import Path
from typing import Any

AYKEN_ETI_FILE_MAGIC = 0x31544945  # "ETI1"
AYKEN_ETI_ENTRY_MAGIC = AYKEN_ETI_FILE_MAGIC
AYKEN_ETI_VERSION = 1
ENTRY_FLAGS_DEFAULT = 0
ENTRY_CPU_ID_DEFAULT = 0
ENTRY_CTX_ID_DEFAULT = 0

HEADER_STRUCT = struct.Struct("<4sHQQ42s")
ENTRY_STRUCT = struct.Struct("<IHHQQIIIQQ32s")

EVENT_MAP: list[tuple[str, str, int, int, int]] = [
    ("AYKEN_CTX_SWITCH", "AY_EVT_CTX_SWITCH", 1, 1, 1),
    ("P10_MAILBOX_DECISION", "AY_EVT_MAILBOX_ACCEPT", 20, 1, 0),
    ("AYKEN_SYSCALL_ENTER", "AY_EVT_SYSCALL_ENTER", 10, 0, 1),
    ("AYKEN_SYSCALL_RETURN", "AY_EVT_SYSCALL_EXIT", 11, 0, 1),
    ("AYKEN_SYSCALL_EXIT", "AY_EVT_SYSCALL_EXIT", 11, 0, 1),
    ("AYKEN_IRQ_ENTER", "AY_EVT_IRQ_ENTER", 12, 0, 1),
    ("AYKEN_IRQ_EXIT", "AY_EVT_IRQ_EXIT", 13, 0, 1),
    ("AYKEN_TRAP_ENTER", "AY_EVT_TRAP_ENTER", 14, 0, 1),
    ("AYKEN_TRAP_EXIT", "AY_EVT_TRAP_EXIT", 15, 0, 1),
    ("AYKEN_SCHED_MB_ACCEPT", "AY_EVT_MAILBOX_ACCEPT", 20, 1, 0),
    ("AYKEN_SCHED_MB_REJECT", "AY_EVT_MAILBOX_REJECT", 21, 1, 0),
]

REQUIRED_EVENT_TYPES = (
    "AY_EVT_SYSCALL_ENTER",
    "AY_EVT_SYSCALL_EXIT",
)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Validate ETI bootstrap sequence and emit transcript evidence."
    )
    parser.add_argument("--events", required=True, help="ring3 events.jsonl path")
    parser.add_argument("--out-eti-jsonl", required=True, help="Output eti_transcript.jsonl path")
    parser.add_argument("--out-eti-bin", required=True, help="Output eti_transcript.bin path")
    parser.add_argument(
        "--out-chain-verify", required=True, help="Output eti_chain_verify.json path"
    )
    parser.add_argument("--out-report", required=True, help="Output report.json path")
    return parser.parse_args()


def sha256_bytes(payload: bytes) -> bytes:
    return hashlib.sha256(payload).digest()


def write_json(path: Path, payload: dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def write_jsonl(path: Path, rows: list[dict[str, Any]]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("w", encoding="utf-8") as fh:
        for row in rows:
            fh.write(json.dumps(row, sort_keys=True) + "\n")


def load_events(path: Path) -> list[dict[str, Any]]:
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
                    f"events_parse_error:{path}:line={line_no}:{type(exc).__name__}"
                ) from exc
            if not isinstance(row, dict):
                raise RuntimeError(f"events_type_error:{path}:line={line_no}")
            row = dict(row)
            row["__event_seq"] = len(rows) + 1
            rows.append(row)
    return rows


def classify_event(token: str) -> tuple[str, int, int, int] | None:
    for marker_token, event_type, event_type_value, is_decision, is_execution in EVENT_MAP:
        if marker_token in token:
            return event_type, event_type_value, is_decision, is_execution
    return None


def canonical_eti_payload(row: dict[str, Any]) -> bytes:
    payload = {
        "event_seq": int(row["event_seq"]),
        "ltick": int(row["ltick"]),
        "cpu_id": int(row["cpu_id"]),
        "ctx_id": int(row["ctx_id"]),
        "event_type": str(row["event_type"]),
        "event_type_value": int(row["event_type_value"]),
        "source_line": int(row["source_line"]),
        "source_offset": int(row["source_offset"]),
        "source_marker": str(row["source_marker"]),
    }
    return json.dumps(payload, sort_keys=True, separators=(",", ":")).encode("utf-8")


def encode_eti_binary(entries: list[dict[str, Any]]) -> bytes:
    entry_blobs: list[bytes] = []
    for row in entries:
        entry_blob = ENTRY_STRUCT.pack(
            AYKEN_ETI_ENTRY_MAGIC,
            AYKEN_ETI_VERSION,
            int(row["flags"]),
            int(row["event_seq"]),
            int(row["ltick"]),
            int(row["cpu_id"]),
            int(row["ctx_id"]),
            int(row["event_type_value"]),
            int(row["source_line"]),
            int(row["source_offset"]),
            bytes.fromhex(row["entry_hash"]),
        )
        entry_blobs.append(entry_blob)

    total_size = HEADER_STRUCT.size + sum(len(blob) for blob in entry_blobs)
    header = HEADER_STRUCT.pack(
        b"ETI1",
        AYKEN_ETI_VERSION,
        len(entry_blobs),
        total_size,
        bytes(42),
    )
    return header + b"".join(entry_blobs)


def fail(report_path: Path, chain_verify_path: Path, report: dict[str, Any]) -> int:
    report["verdict"] = "FAIL"
    report["violations_count"] = len(report.get("violations", []))
    write_json(report_path, report)
    chain_payload = {
        "status": "FAIL",
        "mode": "bootstrap_materialized_from_phase10a2",
        "event_count": int(report.get("eti_event_count", 0)),
        "event_seq_chain_hash": bytes(32).hex(),
        "ltick_chain_hash": bytes(32).hex(),
        "eti_chain_hash": bytes(32).hex(),
        "violations": list(report.get("violations", [])),
        "violations_count": len(report.get("violations", [])),
    }
    write_json(chain_verify_path, chain_payload)
    return 2


def pass_(report_path: Path, chain_verify_path: Path, report: dict[str, Any], chain_payload: dict[str, Any]) -> int:
    report["verdict"] = "PASS"
    report["violations"] = []
    report["violations_count"] = 0
    write_json(report_path, report)
    write_json(chain_verify_path, chain_payload)
    return 0


def main() -> int:
    args = parse_args()

    events_path = Path(args.events)
    eti_jsonl_path = Path(args.out_eti_jsonl)
    eti_bin_path = Path(args.out_eti_bin)
    chain_verify_path = Path(args.out_chain_verify)
    report_path = Path(args.out_report)

    report: dict[str, Any] = {
        "gate": "eti-sequence",
        "events": str(events_path),
        "eti_transcript_jsonl": str(eti_jsonl_path),
        "eti_transcript_bin": str(eti_bin_path),
        "eti_chain_verify": str(chain_verify_path),
        "ltick_mode": "compat_event_seq",
        "violations": [],
    }

    if not events_path.is_file():
        report["violations"].append(f"missing_events:{events_path}")
        write_jsonl(eti_jsonl_path, [])
        eti_bin_path.parent.mkdir(parents=True, exist_ok=True)
        eti_bin_path.write_bytes(encode_eti_binary([]))
        return fail(report_path, chain_verify_path, report)

    try:
        events = load_events(events_path)
    except RuntimeError as exc:
        report["violations"].append(str(exc))
        write_jsonl(eti_jsonl_path, [])
        eti_bin_path.parent.mkdir(parents=True, exist_ok=True)
        eti_bin_path.write_bytes(encode_eti_binary([]))
        return fail(report_path, chain_verify_path, report)

    eti_rows: list[dict[str, Any]] = []
    for row in events:
        token = f"{row.get('type', '')} {row.get('marker', '')}"
        classified = classify_event(token)
        if classified is None:
            continue

        event_type, event_type_value, is_decision, is_execution = classified
        event_seq = int(row["__event_seq"])
        ltick = event_seq

        eti_row: dict[str, Any] = {
            "magic": AYKEN_ETI_ENTRY_MAGIC,
            "version": AYKEN_ETI_VERSION,
            "flags": ENTRY_FLAGS_DEFAULT,
            "event_seq": event_seq,
            "ltick": ltick,
            "cpu_id": ENTRY_CPU_ID_DEFAULT,
            "ctx_id": ENTRY_CTX_ID_DEFAULT,
            "event_type": event_type,
            "event_type_value": event_type_value,
            "is_decision_event": is_decision,
            "is_execution_event": is_execution,
            "source_line": int(row.get("line", 0) or 0),
            "source_offset": int(row.get("offset", 0) or 0),
            "source_marker": str(row.get("marker", "")),
            "source_type": str(row.get("type", "")),
        }

        payload_raw = canonical_eti_payload(eti_row)
        eti_row["entry_hash"] = sha256_bytes(payload_raw).hex()

        eti_rows.append(eti_row)

    if not eti_rows:
        report["violations"].append("empty_eti_stream")

    event_type_counts: dict[str, int] = {}
    for row in eti_rows:
        event_type = str(row["event_type"])
        event_type_counts[event_type] = event_type_counts.get(event_type, 0) + 1

    for required_event_type in REQUIRED_EVENT_TYPES:
        if event_type_counts.get(required_event_type, 0) == 0:
            report["violations"].append(f"missing_required_event_type:{required_event_type}")

    if sum(int(row["is_decision_event"]) for row in eti_rows) == 0:
        report["violations"].append("missing_decision_class_event")

    event_seq_values = [int(row["event_seq"]) for row in eti_rows]
    if event_seq_values != sorted(event_seq_values):
        report["violations"].append("event_seq_non_monotonic")
    if len(set(event_seq_values)) != len(event_seq_values):
        report["violations"].append("event_seq_duplicate")

    ltick_values = [int(row["ltick"]) for row in eti_rows]
    if ltick_values != sorted(ltick_values):
        report["violations"].append("ltick_non_monotonic")
    if len(set(ltick_values)) != len(ltick_values):
        report["violations"].append("ltick_duplicate")

    for idx, row in enumerate(eti_rows, start=1):
        recomputed = sha256_bytes(canonical_eti_payload(row)).hex()
        if str(row.get("entry_hash", "")) != recomputed:
            report["violations"].append(f"entry_hash_mismatch:entry={idx}")

    event_seq_chain_input = b"".join(struct.pack("<Q", seq) for seq in event_seq_values)
    ltick_chain_input = b"".join(struct.pack("<Q", tick) for tick in ltick_values)
    entry_hash_chain_input = b"".join(bytes.fromhex(str(row["entry_hash"])) for row in eti_rows)

    event_seq_chain_hash = (
        sha256_bytes(event_seq_chain_input).hex() if event_seq_values else bytes(32).hex()
    )
    ltick_chain_hash = sha256_bytes(ltick_chain_input).hex() if ltick_values else bytes(32).hex()
    eti_chain_hash = (
        sha256_bytes(entry_hash_chain_input).hex() if eti_rows else bytes(32).hex()
    )

    write_jsonl(eti_jsonl_path, eti_rows)
    eti_bin_path.parent.mkdir(parents=True, exist_ok=True)
    eti_bin_path.write_bytes(encode_eti_binary(eti_rows))

    report["eti_event_count"] = len(eti_rows)
    report["decision_event_count"] = sum(int(row["is_decision_event"]) for row in eti_rows)
    report["execution_event_count"] = sum(int(row["is_execution_event"]) for row in eti_rows)
    report["event_type_counts"] = event_type_counts
    report["event_seq_chain_hash"] = event_seq_chain_hash
    report["ltick_chain_hash"] = ltick_chain_hash
    report["eti_chain_hash"] = eti_chain_hash

    chain_payload = {
        "status": "FAIL" if report["violations"] else "PASS",
        "mode": "bootstrap_materialized_from_phase10a2",
        "event_count": len(eti_rows),
        "event_seq_chain_hash": event_seq_chain_hash,
        "ltick_chain_hash": ltick_chain_hash,
        "eti_chain_hash": eti_chain_hash,
        "violations": list(report["violations"]),
        "violations_count": len(report["violations"]),
    }

    if report["violations"]:
        return fail(report_path, chain_verify_path, report)
    return pass_(report_path, chain_verify_path, report, chain_payload)


if __name__ == "__main__":
    raise SystemExit(main())
