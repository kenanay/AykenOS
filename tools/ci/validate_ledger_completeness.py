#!/usr/bin/env python3
"""Validate and materialize Phase-11 Decision Ledger v1 completeness evidence."""

from __future__ import annotations

# Author: Kenan AY

import argparse
import hashlib
import json
import re
import struct
from pathlib import Path
from typing import Any

TOKEN_CTX_SWITCH = "[[AYKEN_CTX_SWITCH]]"
TOKEN_MAILBOX_DECISION = "P10_MAILBOX_DECISION"
AY_EVT_CTX_SWITCH = 1
AYKEN_LEDGER_MAGIC = 0x3147444C  # "LDG1"
AYKEN_LEDGER_VERSION = 1
ENTRY_FLAGS_DEFAULT = 0
ENTRY_REASON_SCHEDULER_DECISION = 0x01
ENTRY_CPU_ID_DEFAULT = 0
ENTRY_DECISION_CAP_DEFAULT = 0

HEADER_STRUCT = struct.Struct("<4sHQQ42s")
ENTRY_STRUCT = struct.Struct("<IHHQQIIQQQQQQ32s32s32s")

MAILBOX_DECISION_PATTERN = re.compile(
    r"(?<![A-Za-z0-9_])P10_MAILBOX_DECISION\s+"
    r"id=(?P<id>\d+)\s+"
    r"pid=(?P<pid>\d+)\s+"
    r"valid=(?P<valid>[01])\s+"
    r"src=(?P<src>\d+)(?![A-Za-z0-9_])"
)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Validate P11-02 decision ledger completeness and emit ledger artifacts."
    )
    parser.add_argument("--events", required=True, help="ring3 events.jsonl path")
    parser.add_argument("--log", required=True, help="ring3 marker.log path")
    parser.add_argument("--out-report", required=True, help="Output report.json path")
    parser.add_argument(
        "--out-ledger-jsonl", required=True, help="Output decision_ledger.jsonl path"
    )
    parser.add_argument(
        "--out-ledger-bin", required=True, help="Output decision_ledger.bin path"
    )
    parser.add_argument(
        "--eti-events",
        default="",
        help="Optional ETI JSONL with event_seq/ltick rows for strict binding.",
    )
    parser.add_argument(
        "--require-eti-binding",
        choices=("0", "1"),
        default="0",
        help="Require ETI event_seq<->ltick binding (default: 0).",
    )
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


def load_eti_binding(path: Path) -> dict[int, int]:
    binding: dict[int, int] = {}
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
            if "event_seq" not in row or "ltick" not in row:
                raise RuntimeError(f"eti_missing_fields:{path}:line={line_no}")
            event_seq = int(row["event_seq"])
            ltick = int(row["ltick"])
            binding[event_seq] = ltick
    return binding


def parse_mailbox_decisions(log_text: str) -> list[dict[str, int]]:
    rows: list[dict[str, int]] = []
    for match in MAILBOX_DECISION_PATTERN.finditer(log_text):
        rows.append(
            {
                "decision_id": int(match.group("id")),
                "decision_pid": int(match.group("pid")),
                "decision_valid": int(match.group("valid")),
                "decision_src_pid": int(match.group("src")),
            }
        )
    return rows


def select_ctx_switch_events(events: list[dict[str, Any]]) -> list[dict[str, Any]]:
    rows: list[dict[str, Any]] = []
    for row in events:
        token = str(row.get("marker") or row.get("type") or "")
        if token == "AYKEN_CTX_SWITCH" or TOKEN_CTX_SWITCH in token:
            rows.append(row)
    return rows


def select_mailbox_events(events: list[dict[str, Any]]) -> list[dict[str, Any]]:
    rows: list[dict[str, Any]] = []
    for row in events:
        token = str(row.get("marker") or row.get("type") or "")
        if token == TOKEN_MAILBOX_DECISION:
            rows.append(row)
    return rows


def canonical_payload(decision: dict[str, int], origin_event: dict[str, Any]) -> bytes:
    payload = {
        "decision_id": int(decision["decision_id"]),
        "decision_pid": int(decision["decision_pid"]),
        "decision_src_pid": int(decision["decision_src_pid"]),
        "decision_valid": int(decision["decision_valid"]),
        "origin_event_line": int(origin_event.get("line", 0) or 0),
        "origin_event_offset": int(origin_event.get("offset", 0) or 0),
        "origin_event_type": str(origin_event.get("type", "")),
    }
    return json.dumps(payload, sort_keys=True, separators=(",", ":")).encode("utf-8")


def encode_ledger_binary(entries: list[dict[str, Any]]) -> bytes:
    entry_blobs: list[bytes] = []
    for row in entries:
        entry_blob = ENTRY_STRUCT.pack(
            AYKEN_LEDGER_MAGIC,
            AYKEN_LEDGER_VERSION,
            int(row["flags"]),
            int(row["event_seq"]),
            int(row["ltick"]),
            int(row["cpu_id"]),
            int(row["event_type_value"]),
            int(row["prev_ctx"]),
            int(row["next_ctx"]),
            int(row["decision_cap"]),
            int(row["reason_code"]),
            int(row["aux0"]),
            int(row["aux1"]),
            bytes.fromhex(row["payload_hash"]),
            bytes.fromhex(row["prev_hash"]),
            bytes.fromhex(row["entry_hash"]),
        )
        entry_blobs.append(entry_blob)

    total_size = HEADER_STRUCT.size + sum(len(blob) for blob in entry_blobs)
    header = HEADER_STRUCT.pack(
        b"LDG1",
        AYKEN_LEDGER_VERSION,
        len(entry_blobs),
        total_size,
        bytes(42),
    )
    return header + b"".join(entry_blobs)


def fail(path: Path, report: dict[str, Any]) -> int:
    report["verdict"] = "FAIL"
    report["violations_count"] = len(report.get("violations", []))
    write_json(path, report)
    return 2


def pass_(path: Path, report: dict[str, Any]) -> int:
    report["verdict"] = "PASS"
    report["violations_count"] = 0
    report["violations"] = []
    write_json(path, report)
    return 0


def main() -> int:
    args = parse_args()

    events_path = Path(args.events)
    log_path = Path(args.log)
    report_path = Path(args.out_report)
    ledger_jsonl_path = Path(args.out_ledger_jsonl)
    ledger_bin_path = Path(args.out_ledger_bin)
    eti_events_path = Path(args.eti_events) if args.eti_events else None
    require_eti_binding = args.require_eti_binding == "1"

    report: dict[str, Any] = {
        "gate": "ledger-completeness",
        "events": str(events_path),
        "marker_log": str(log_path),
        "ledger_jsonl": str(ledger_jsonl_path),
        "ledger_bin": str(ledger_bin_path),
        "require_eti_binding": int(require_eti_binding),
        "violations": [],
    }

    if not events_path.is_file():
        report["violations"].append(f"missing_events:{events_path}")
    if not log_path.is_file():
        report["violations"].append(f"missing_marker_log:{log_path}")
    if require_eti_binding and (not eti_events_path or not eti_events_path.is_file()):
        report["violations"].append(f"missing_eti_events:{eti_events_path}")

    if report["violations"]:
        write_jsonl(ledger_jsonl_path, [])
        ledger_bin_path.parent.mkdir(parents=True, exist_ok=True)
        ledger_bin_path.write_bytes(encode_ledger_binary([]))
        return fail(report_path, report)

    try:
        events = load_events(events_path)
    except RuntimeError as exc:
        report["violations"].append(str(exc))
        write_jsonl(ledger_jsonl_path, [])
        ledger_bin_path.parent.mkdir(parents=True, exist_ok=True)
        ledger_bin_path.write_bytes(encode_ledger_binary([]))
        return fail(report_path, report)

    log_text = log_path.read_text(encoding="utf-8", errors="replace")
    decisions = parse_mailbox_decisions(log_text)
    ctx_switch_events = select_ctx_switch_events(events)
    mailbox_events = select_mailbox_events(events)
    ctx_switch_marker_count = log_text.count(TOKEN_CTX_SWITCH)

    origin_events = ctx_switch_events
    origin_mode = "ctx_switch_event"
    if not origin_events:
        origin_events = mailbox_events
        origin_mode = "mailbox_event_fallback"
    report["origin_mode"] = origin_mode

    report["context_switch_count"] = ctx_switch_marker_count
    report["context_switch_event_count"] = len(ctx_switch_events)
    report["schedule_decision_count"] = len(decisions)

    if ctx_switch_marker_count == 0:
        report["violations"].append("missing_ctx_switch_markers")
    if len(decisions) == 0:
        report["violations"].append("missing_schedule_decisions")
    if ctx_switch_marker_count != len(decisions):
        report["violations"].append(
            f"switch_decision_count_mismatch:switch={ctx_switch_marker_count}:decision={len(decisions)}"
        )
    if len(origin_events) == 0:
        report["violations"].append("missing_origin_events_for_binding")
    if len(origin_events) < len(decisions):
        report["violations"].append(
            f"insufficient_origin_events:origin={len(origin_events)}:decision={len(decisions)}"
        )

    seen_decision_ids: set[int] = set()
    last_decision_id = 0
    for row in decisions:
        decision_id = int(row["decision_id"])
        if decision_id in seen_decision_ids:
            report["violations"].append(f"duplicate_decision_id:{decision_id}")
        seen_decision_ids.add(decision_id)
        if decision_id <= last_decision_id:
            report["violations"].append(
                f"decision_id_non_monotonic:prev={last_decision_id}:curr={decision_id}"
            )
        last_decision_id = decision_id
        if row["decision_valid"] != 1:
            report["violations"].append(f"decision_not_valid:{decision_id}")

    eti_binding: dict[int, int] = {}
    if require_eti_binding:
        try:
            assert eti_events_path is not None
            eti_binding = load_eti_binding(eti_events_path)
        except RuntimeError as exc:
            report["violations"].append(str(exc))

    ledger_rows: list[dict[str, Any]] = []
    prev_hash_raw = bytes(32)

    for idx, (decision, origin_event) in enumerate(
        zip(decisions, origin_events), start=1
    ):
        event_seq = int(origin_event["__event_seq"])
        if require_eti_binding:
            ltick = eti_binding.get(event_seq)
            if ltick is None:
                report["violations"].append(f"missing_eti_binding_for_event_seq:{event_seq}")
                ltick = event_seq
        else:
            ltick = event_seq

        payload_raw = canonical_payload(decision, origin_event)
        payload_hash_raw = sha256_bytes(payload_raw)
        entry_hash_raw = sha256_bytes(prev_hash_raw + payload_hash_raw)

        prev_ctx = 0 if idx == 1 else int(ledger_rows[idx - 2]["next_ctx"])
        next_ctx = int(decision["decision_pid"])

        row = {
            "magic": AYKEN_LEDGER_MAGIC,
            "version": AYKEN_LEDGER_VERSION,
            "flags": ENTRY_FLAGS_DEFAULT,
            "event_seq": event_seq,
            "ltick": int(ltick),
            "cpu_id": ENTRY_CPU_ID_DEFAULT,
            "event_type": "AY_EVT_CTX_SWITCH",
            "event_type_value": AY_EVT_CTX_SWITCH,
            "prev_ctx": prev_ctx,
            "next_ctx": next_ctx,
            "decision_cap": ENTRY_DECISION_CAP_DEFAULT,
            "reason_code": ENTRY_REASON_SCHEDULER_DECISION,
            "aux0": int(decision["decision_id"]),
            "aux1": int(decision["decision_src_pid"]),
            "decision_valid": int(decision["decision_valid"]),
            "payload_hash": payload_hash_raw.hex(),
            "prev_hash": prev_hash_raw.hex(),
            "entry_hash": entry_hash_raw.hex(),
            "origin_marker": str(origin_event.get("marker", "")),
            "origin_event_type": str(origin_event.get("type", "")),
            "origin_line": int(origin_event.get("line", 0) or 0),
            "origin_offset": int(origin_event.get("offset", 0) or 0),
        }

        required_fields = (
            "event_seq",
            "ltick",
            "cpu_id",
            "event_type",
            "prev_ctx",
            "next_ctx",
            "decision_cap",
            "reason_code",
            "payload_hash",
            "prev_hash",
            "entry_hash",
        )
        for key in required_fields:
            if row.get(key) in (None, ""):
                report["violations"].append(f"missing_required_field:{key}:entry={idx}")

        ledger_rows.append(row)
        prev_hash_raw = entry_hash_raw

    event_seq_values = [int(row["event_seq"]) for row in ledger_rows]
    if event_seq_values != sorted(event_seq_values):
        report["violations"].append("event_seq_non_monotonic")
    if len(set(event_seq_values)) != len(event_seq_values):
        report["violations"].append("event_seq_duplicate")

    ltick_values = [int(row["ltick"]) for row in ledger_rows]
    if ltick_values != sorted(ltick_values):
        report["violations"].append("ltick_non_monotonic")

    if require_eti_binding:
        for row in ledger_rows:
            expected_ltick = eti_binding.get(int(row["event_seq"]))
            if expected_ltick is None:
                report["violations"].append(
                    f"eti_binding_missing:event_seq={row['event_seq']}"
                )
                continue
            if int(row["ltick"]) != int(expected_ltick):
                report["violations"].append(
                    f"eti_binding_mismatch:event_seq={row['event_seq']}:ledger_ltick={row['ltick']}:eti_ltick={expected_ltick}"
                )

    ledger_root_input = b"".join(bytes.fromhex(row["entry_hash"]) for row in ledger_rows)
    ledger_root_hash = sha256_bytes(ledger_root_input).hex() if ledger_rows else bytes(32).hex()

    report["entries_count"] = len(ledger_rows)
    report["event_seq_unique_count"] = len(set(event_seq_values))
    report["ledger_root_hash"] = ledger_root_hash
    report["ltick_mode"] = "eti_binding" if require_eti_binding else "compat_event_seq"

    write_jsonl(ledger_jsonl_path, ledger_rows)
    ledger_bin_path.parent.mkdir(parents=True, exist_ok=True)
    ledger_bin_path.write_bytes(encode_ledger_binary(ledger_rows))

    if report["violations"]:
        return fail(report_path, report)
    return pass_(report_path, report)


if __name__ == "__main__":
    raise SystemExit(main())
