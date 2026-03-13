#!/usr/bin/env python3
"""Validate Phase-11 Decision Ledger v1 hash-chain integrity."""

from __future__ import annotations

# Author: Kenan AY

import argparse
import copy
import hashlib
import json
import struct
from pathlib import Path
from typing import Any


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Validate ledger hash-chain continuity and tamper fail-closed behavior."
    )
    parser.add_argument("--ledger-jsonl", required=True, help="decision_ledger.jsonl path")
    parser.add_argument("--out-chain-verify", required=True, help="Output chain_verify.json")
    parser.add_argument("--out-tamper-test", required=True, help="Output tamper_test.json")
    parser.add_argument("--out-report", required=True, help="Output report.json")
    return parser.parse_args()


def write_json(path: Path, payload: dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def sha256_bytes(data: bytes) -> bytes:
    return hashlib.sha256(data).digest()


def load_ledger_rows(path: Path) -> list[dict[str, Any]]:
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


def normalize_payload_from_row(row: dict[str, Any]) -> bytes:
    payload = {
        "decision_id": int(row.get("aux0", 0)),
        "decision_pid": int(row.get("next_ctx", 0)),
        "decision_src_pid": int(row.get("aux1", 0)),
        "decision_valid": int(row.get("decision_valid", 0)),
        "origin_event_line": int(row.get("origin_line", 0)),
        "origin_event_offset": int(row.get("origin_offset", 0)),
        "origin_event_type": str(row.get("origin_event_type") or row.get("origin_marker") or ""),
    }
    return json.dumps(payload, sort_keys=True, separators=(",", ":")).encode("utf-8")


def validate_chain(rows: list[dict[str, Any]]) -> tuple[dict[str, Any], list[str]]:
    violations: list[str] = []

    required_fields = (
        "event_seq",
        "ltick",
        "payload_hash",
        "prev_hash",
        "entry_hash",
        "aux0",
        "aux1",
        "next_ctx",
    )

    if not rows:
        violations.append("empty_ledger")
        return {"entries_count": 0}, violations

    event_seq_values: list[int] = []
    ltick_values: list[int] = []
    entry_hashes_raw: list[bytes] = []

    prev_entry_hash_raw = bytes(32)

    for idx, row in enumerate(rows, start=1):
        for key in required_fields:
            if key not in row:
                violations.append(f"missing_required_field:{key}:entry={idx}")

        try:
            event_seq = int(row.get("event_seq", -1))
            ltick = int(row.get("ltick", -1))
        except Exception:
            violations.append(f"invalid_numeric_field:event_seq_or_ltick:entry={idx}")
            continue

        event_seq_values.append(event_seq)
        ltick_values.append(ltick)

        try:
            payload_hash_raw = bytes.fromhex(str(row.get("payload_hash", "")))
            prev_hash_raw = bytes.fromhex(str(row.get("prev_hash", "")))
            entry_hash_raw = bytes.fromhex(str(row.get("entry_hash", "")))
        except Exception:
            violations.append(f"invalid_hash_encoding:entry={idx}")
            continue

        if len(payload_hash_raw) != 32:
            violations.append(f"invalid_payload_hash_length:entry={idx}")
        if len(prev_hash_raw) != 32:
            violations.append(f"invalid_prev_hash_length:entry={idx}")
        if len(entry_hash_raw) != 32:
            violations.append(f"invalid_entry_hash_length:entry={idx}")

        expected_prev_hash_raw = bytes(32) if idx == 1 else prev_entry_hash_raw
        if prev_hash_raw != expected_prev_hash_raw:
            violations.append(f"prev_hash_continuity_mismatch:entry={idx}")

        normalized_payload = normalize_payload_from_row(row)
        recomputed_payload_hash_raw = sha256_bytes(normalized_payload)
        if payload_hash_raw != recomputed_payload_hash_raw:
            violations.append(f"payload_hash_mismatch:entry={idx}")

        recomputed_entry_hash_raw = sha256_bytes(prev_hash_raw + payload_hash_raw)
        if entry_hash_raw != recomputed_entry_hash_raw:
            violations.append(f"entry_hash_mismatch:entry={idx}")

        prev_entry_hash_raw = entry_hash_raw
        entry_hashes_raw.append(entry_hash_raw)

    if event_seq_values != sorted(event_seq_values):
        violations.append("event_seq_non_monotonic")
    if len(set(event_seq_values)) != len(event_seq_values):
        violations.append("event_seq_duplicate")

    if ltick_values != sorted(ltick_values):
        violations.append("ltick_non_monotonic")
    if len(set(ltick_values)) != len(ltick_values):
        violations.append("ltick_duplicate")

    event_seq_chain_input = b"".join(struct.pack("<Q", seq) for seq in event_seq_values)
    ltick_chain_input = b"".join(struct.pack("<Q", tick) for tick in ltick_values)
    ledger_root_input = b"".join(entry_hashes_raw)

    chain = {
        "entries_count": len(rows),
        "chain_head": entry_hashes_raw[-1].hex() if entry_hashes_raw else bytes(32).hex(),
        "ledger_root_hash": sha256_bytes(ledger_root_input).hex() if entry_hashes_raw else bytes(32).hex(),
        "event_seq_chain_hash": sha256_bytes(event_seq_chain_input).hex(),
        "ltick_chain_hash": sha256_bytes(ltick_chain_input).hex(),
        "event_seq_min": min(event_seq_values) if event_seq_values else 0,
        "event_seq_max": max(event_seq_values) if event_seq_values else 0,
        "ltick_min": min(ltick_values) if ltick_values else 0,
        "ltick_max": max(ltick_values) if ltick_values else 0,
    }
    return chain, violations


def run_tamper_test(rows: list[dict[str, Any]]) -> dict[str, Any]:
    if not rows:
        return {
            "tamper_applied": 0,
            "tamper_target": "none",
            "expected_verdict": "FAIL",
            "actual_verdict": "FAIL",
            "detected": 1,
            "violations": ["empty_ledger"],
        }

    tampered = copy.deepcopy(rows)
    original_payload_hash = str(tampered[0].get("payload_hash", ""))

    try:
        payload_hash_raw = bytearray(bytes.fromhex(original_payload_hash))
    except Exception:
        return {
            "tamper_applied": 0,
            "tamper_target": "payload_hash",
            "expected_verdict": "FAIL",
            "actual_verdict": "FAIL",
            "detected": 1,
            "violations": ["invalid_payload_hash_encoding_before_tamper"],
        }

    if not payload_hash_raw:
        return {
            "tamper_applied": 0,
            "tamper_target": "payload_hash",
            "expected_verdict": "FAIL",
            "actual_verdict": "FAIL",
            "detected": 1,
            "violations": ["empty_payload_hash_before_tamper"],
        }

    payload_hash_raw[0] ^= 0x01
    tampered[0]["payload_hash"] = payload_hash_raw.hex()

    _, tamper_violations = validate_chain(tampered)
    detected = 1 if tamper_violations else 0

    return {
        "tamper_applied": 1,
        "tamper_target": "entry[1].payload_hash.bit0",
        "expected_verdict": "FAIL",
        "actual_verdict": "FAIL" if tamper_violations else "PASS",
        "detected": detected,
        "violations": tamper_violations,
    }


def main() -> int:
    args = parse_args()

    ledger_jsonl_path = Path(args.ledger_jsonl)
    out_chain_verify_path = Path(args.out_chain_verify)
    out_tamper_test_path = Path(args.out_tamper_test)
    out_report_path = Path(args.out_report)

    report: dict[str, Any] = {
        "gate": "ledger-integrity",
        "ledger_jsonl": str(ledger_jsonl_path),
        "violations": [],
    }

    if not ledger_jsonl_path.is_file():
        report["violations"].append(f"missing_ledger_jsonl:{ledger_jsonl_path}")
        chain_verify = {
            "verdict": "FAIL",
            "entries_count": 0,
            "violations": list(report["violations"]),
        }
        tamper_test = {
            "tamper_applied": 0,
            "tamper_target": "none",
            "expected_verdict": "FAIL",
            "actual_verdict": "FAIL",
            "detected": 1,
            "violations": ["skipped_due_to_missing_ledger"],
        }
        write_json(out_chain_verify_path, chain_verify)
        write_json(out_tamper_test_path, tamper_test)
        report["verdict"] = "FAIL"
        report["violations_count"] = len(report["violations"])
        report["chain_verify_path"] = str(out_chain_verify_path)
        report["tamper_test_path"] = str(out_tamper_test_path)
        write_json(out_report_path, report)
        return 2

    try:
        rows = load_ledger_rows(ledger_jsonl_path)
    except RuntimeError as exc:
        report["violations"].append(str(exc))
        chain_verify = {
            "verdict": "FAIL",
            "entries_count": 0,
            "violations": list(report["violations"]),
        }
        tamper_test = {
            "tamper_applied": 0,
            "tamper_target": "none",
            "expected_verdict": "FAIL",
            "actual_verdict": "FAIL",
            "detected": 1,
            "violations": ["skipped_due_to_parse_error"],
        }
        write_json(out_chain_verify_path, chain_verify)
        write_json(out_tamper_test_path, tamper_test)
        report["verdict"] = "FAIL"
        report["violations_count"] = len(report["violations"])
        report["chain_verify_path"] = str(out_chain_verify_path)
        report["tamper_test_path"] = str(out_tamper_test_path)
        write_json(out_report_path, report)
        return 2

    chain, chain_violations = validate_chain(rows)
    chain_verify = dict(chain)
    chain_verify["verdict"] = "FAIL" if chain_violations else "PASS"
    chain_verify["violations"] = chain_violations
    chain_verify["violations_count"] = len(chain_violations)

    tamper_test = run_tamper_test(rows)

    write_json(out_chain_verify_path, chain_verify)
    write_json(out_tamper_test_path, tamper_test)

    report["chain_verify_path"] = str(out_chain_verify_path)
    report["tamper_test_path"] = str(out_tamper_test_path)
    report["entries_count"] = int(chain.get("entries_count", 0))
    report["chain_head"] = str(chain.get("chain_head", ""))
    report["ledger_root_hash"] = str(chain.get("ledger_root_hash", ""))
    report["event_seq_chain_hash"] = str(chain.get("event_seq_chain_hash", ""))
    report["ltick_chain_hash"] = str(chain.get("ltick_chain_hash", ""))

    if chain_violations:
        report["violations"].extend(chain_violations)

    if int(tamper_test.get("detected", 0)) != 1:
        report["violations"].append("tamper_detection_failed")

    report["verdict"] = "FAIL" if report["violations"] else "PASS"
    report["violations_count"] = len(report["violations"])
    write_json(out_report_path, report)

    return 2 if report["violations"] else 0


if __name__ == "__main__":
    raise SystemExit(main())
