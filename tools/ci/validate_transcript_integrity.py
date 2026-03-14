#!/usr/bin/env python3
"""Validate Phase-11 ETI transcript integrity."""

from __future__ import annotations

# Author: Kenan AY

import argparse
import hashlib
import json
import struct
from pathlib import Path
from typing import Any

AYKEN_ETI_FILE_MAGIC = 0x31544945  # "ETI1"
AYKEN_ETI_VERSION = 1
HEADER_STRUCT = struct.Struct("<4sHQQ42s")
ENTRY_STRUCT = struct.Struct("<IHHQQIIIQQ32s")

REQUIRED_FIELDS = (
    "event_seq",
    "ltick",
    "cpu_id",
    "ctx_id",
    "event_type",
    "event_type_value",
    "source_line",
    "source_offset",
    "source_marker",
    "entry_hash",
)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Validate ETI transcript integrity.")
    parser.add_argument("--eti-jsonl", required=True, help="eti_transcript.jsonl path")
    parser.add_argument("--eti-bin", required=True, help="eti_transcript.bin path")
    parser.add_argument("--out-report", required=True, help="Output report.json path")
    return parser.parse_args()


def sha256_bytes(payload: bytes) -> bytes:
    return hashlib.sha256(payload).digest()


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
                    f"transcript_parse_error:{path}:line={line_no}:{type(exc).__name__}"
                ) from exc
            if not isinstance(row, dict):
                raise RuntimeError(f"transcript_type_error:{path}:line={line_no}")
            rows.append(row)
    return rows


def canonical_payload(row: dict[str, Any]) -> bytes:
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


def fail(report_path: Path, report: dict[str, Any]) -> int:
    report["verdict"] = "FAIL"
    report["violations_count"] = len(report.get("violations", []))
    write_json(report_path, report)
    return 2


def pass_(report_path: Path, report: dict[str, Any]) -> int:
    report["verdict"] = "PASS"
    report["violations"] = []
    report["violations_count"] = 0
    write_json(report_path, report)
    return 0


def main() -> int:
    args = parse_args()

    eti_jsonl_path = Path(args.eti_jsonl)
    eti_bin_path = Path(args.eti_bin)
    report_path = Path(args.out_report)

    report: dict[str, Any] = {
        "gate": "transcript-integrity",
        "eti_jsonl": str(eti_jsonl_path),
        "eti_bin": str(eti_bin_path),
        "violations": [],
    }

    if not eti_jsonl_path.is_file():
        report["violations"].append(f"missing_eti_jsonl:{eti_jsonl_path}")
    if not eti_bin_path.is_file():
        report["violations"].append(f"missing_eti_bin:{eti_bin_path}")
    if report["violations"]:
        return fail(report_path, report)

    try:
        rows = load_jsonl(eti_jsonl_path)
    except RuntimeError as exc:
        report["violations"].append(str(exc))
        return fail(report_path, report)

    if not rows:
        report["violations"].append("empty_transcript")

    event_seq_values: list[int] = []
    ltick_values: list[int] = []

    for idx, row in enumerate(rows, start=1):
        for key in REQUIRED_FIELDS:
            if row.get(key) in (None, ""):
                report["violations"].append(f"missing_required_field:{key}:entry={idx}")

        try:
            event_seq = int(row["event_seq"])
            ltick = int(row["ltick"])
            int(row["cpu_id"])
            int(row["ctx_id"])
            int(row["event_type_value"])
            int(row["source_line"])
            int(row["source_offset"])
        except Exception:
            report["violations"].append(f"invalid_numeric_field:entry={idx}")
            continue

        event_seq_values.append(event_seq)
        ltick_values.append(ltick)

        recomputed = sha256_bytes(canonical_payload(row)).hex()
        if str(row.get("entry_hash", "")) != recomputed:
            report["violations"].append(f"entry_hash_mismatch:entry={idx}")

    if event_seq_values != sorted(event_seq_values):
        report["violations"].append("event_seq_non_monotonic")
    if len(set(event_seq_values)) != len(event_seq_values):
        report["violations"].append("event_seq_duplicate")

    if ltick_values != sorted(ltick_values):
        report["violations"].append("ltick_non_monotonic")
    if len(set(ltick_values)) != len(ltick_values):
        report["violations"].append("ltick_duplicate")

    blob = eti_bin_path.read_bytes()
    if len(blob) < HEADER_STRUCT.size:
        report["violations"].append("eti_bin_too_small")
    else:
        header = blob[: HEADER_STRUCT.size]
        magic_bytes, version, entry_count, total_size, _reserved = HEADER_STRUCT.unpack(header)
        if magic_bytes != b"ETI1":
            report["violations"].append("invalid_eti_bin_magic")
        if int(version) != AYKEN_ETI_VERSION:
            report["violations"].append(
                f"invalid_eti_bin_version:expected={AYKEN_ETI_VERSION}:actual={version}"
            )
        if int(total_size) != len(blob):
            report["violations"].append(
                f"invalid_eti_bin_total_size:expected={len(blob)}:actual={total_size}"
            )

        expected_size = HEADER_STRUCT.size + int(entry_count) * ENTRY_STRUCT.size
        if expected_size != len(blob):
            report["violations"].append(
                f"invalid_eti_bin_entry_layout:expected={expected_size}:actual={len(blob)}"
            )
        if int(entry_count) != len(rows):
            report["violations"].append(
                f"eti_bin_entry_count_mismatch:bin={entry_count}:jsonl={len(rows)}"
            )

        offset = HEADER_STRUCT.size
        for idx in range(int(entry_count)):
            if offset + ENTRY_STRUCT.size > len(blob):
                report["violations"].append(f"eti_bin_truncated:entry={idx + 1}")
                break
            entry_blob = blob[offset : offset + ENTRY_STRUCT.size]
            offset += ENTRY_STRUCT.size

            (
                entry_magic,
                entry_version,
                _flags,
                entry_seq,
                entry_ltick,
                _cpu_id,
                _ctx_id,
                entry_event_type_value,
                _source_line,
                _source_offset,
                entry_hash_raw,
            ) = ENTRY_STRUCT.unpack(entry_blob)

            if entry_magic != AYKEN_ETI_FILE_MAGIC:
                report["violations"].append(f"eti_bin_entry_magic_mismatch:entry={idx + 1}")
            if entry_version != AYKEN_ETI_VERSION:
                report["violations"].append(f"eti_bin_entry_version_mismatch:entry={idx + 1}")

            if idx < len(rows):
                row = rows[idx]
                if int(row.get("event_seq", -1)) != int(entry_seq):
                    report["violations"].append(
                        f"eti_bin_event_seq_mismatch:entry={idx + 1}:bin={entry_seq}:jsonl={row.get('event_seq')}"
                    )
                if int(row.get("ltick", -1)) != int(entry_ltick):
                    report["violations"].append(
                        f"eti_bin_ltick_mismatch:entry={idx + 1}:bin={entry_ltick}:jsonl={row.get('ltick')}"
                    )
                if int(row.get("event_type_value", -1)) != int(entry_event_type_value):
                    report["violations"].append(
                        f"eti_bin_event_type_value_mismatch:entry={idx + 1}:bin={entry_event_type_value}:jsonl={row.get('event_type_value')}"
                    )
                if str(row.get("entry_hash", "")) != entry_hash_raw.hex():
                    report["violations"].append(f"eti_bin_entry_hash_mismatch:entry={idx + 1}")

    event_seq_chain_input = b"".join(struct.pack("<Q", seq) for seq in event_seq_values)
    ltick_chain_input = b"".join(struct.pack("<Q", tick) for tick in ltick_values)
    entry_hash_chain_input = b"".join(bytes.fromhex(str(row.get("entry_hash", bytes(32).hex()))) for row in rows)

    report["event_count"] = len(rows)
    report["event_seq_chain_hash"] = (
        sha256_bytes(event_seq_chain_input).hex() if event_seq_values else bytes(32).hex()
    )
    report["ltick_chain_hash"] = (
        sha256_bytes(ltick_chain_input).hex() if ltick_values else bytes(32).hex()
    )
    report["eti_chain_hash"] = (
        sha256_bytes(entry_hash_chain_input).hex() if rows else bytes(32).hex()
    )

    if report["violations"]:
        return fail(report_path, report)
    return pass_(report_path, report)


if __name__ == "__main__":
    raise SystemExit(main())
