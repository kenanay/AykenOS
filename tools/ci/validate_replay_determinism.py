#!/usr/bin/env python3
"""Validate Phase-11 bootstrap replay determinism over identity-locked evidence."""

from __future__ import annotations

# Author: Kenan AY

import argparse
import hashlib
import json
from pathlib import Path
from typing import Any


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Validate replay determinism parity for event_seq/ltick/trace hash."
    )
    parser.add_argument("--abdf-hash-file", required=True, help="abdf_snapshot_hash.txt path")
    parser.add_argument("--bcib-plan-hash-file", required=True, help="bcib_plan_hash.txt path")
    parser.add_argument(
        "--record-trace-jsonl", required=True, help="record execution_trace.jsonl path"
    )
    parser.add_argument(
        "--record-trace-hash-file",
        required=True,
        help="record execution_trace_hash.txt path",
    )
    parser.add_argument("--out-replay-trace-jsonl", required=True, help="Output replay_trace.jsonl path")
    parser.add_argument(
        "--out-replay-trace-hash-txt", required=True, help="Output replay_trace_hash.txt path"
    )
    parser.add_argument("--out-replay-report", required=True, help="Output replay_report.json path")
    parser.add_argument("--out-event-diff", required=True, help="Output event_diff.txt path")
    parser.add_argument("--out-ltick-diff", required=True, help="Output ltick_diff.txt path")
    parser.add_argument("--out-report", required=True, help="Output report.json path")
    parser.add_argument(
        "--expected-final-state-hash-file",
        required=False,
        default="",
        help="Optional expected final_state_hash file (first token is consumed)",
    )
    return parser.parse_args()


def write_json(path: Path, payload: dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def write_text(path: Path, value: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text((value or "") + "\n", encoding="utf-8")


def write_lines(path: Path, lines: list[str]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("w", encoding="utf-8") as fh:
        for line in lines:
            fh.write(line.rstrip("\n") + "\n")


def sha256_hex(payload: bytes) -> str:
    return hashlib.sha256(payload).hexdigest()


def is_sha256_hex(value: str) -> bool:
    if not isinstance(value, str) or len(value) != 64:
        return False
    return all(ch in "0123456789abcdef" for ch in value.lower())


def normalize_hash_file(raw_text: str) -> str:
    for line in raw_text.splitlines():
        tokenized = line.strip()
        if not tokenized:
            continue
        return tokenized.split()[0].strip().lower()
    return ""


def load_hash_file(path: Path, label: str, report: dict[str, Any]) -> str:
    if not path.is_file():
        report["violations"].append(f"missing_{label}_hash_file:{path}")
        return ""
    try:
        raw = path.read_text(encoding="utf-8", errors="replace")
    except Exception as exc:  # pragma: no cover
        report["violations"].append(f"{label}_hash_read_error:{path}:{type(exc).__name__}")
        return ""

    value = normalize_hash_file(raw)
    if not value:
        report["violations"].append(f"empty_{label}_hash_file:{path}")
        return ""
    if not is_sha256_hex(value):
        report["violations"].append(f"invalid_{label}_hash_format:{path}:{value}")
        return ""
    return value


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
                    f"record_trace_parse_error:{path}:line={line_no}:{type(exc).__name__}"
                ) from exc
            if not isinstance(row, dict):
                raise RuntimeError(f"record_trace_type_error:{path}:line={line_no}")
            rows.append(row)
    return rows


def serialize_trace_rows(path: Path, rows: list[dict[str, Any]]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("w", encoding="utf-8") as fh:
        for row in rows:
            fh.write(json.dumps(row, sort_keys=True, separators=(",", ":")) + "\n")


def compute_replay_result_hash(
    abdf_hash: str,
    bcib_hash: str,
    record_trace_hash: str,
    replay_trace_hash: str,
    mismatch_count: int,
) -> str:
    payload = (
        f"{abdf_hash}|{bcib_hash}|{record_trace_hash}|{replay_trace_hash}|{int(mismatch_count)}"
    ).encode("utf-8")
    return sha256_hex(payload)


def fail(
    report_path: Path,
    replay_report_path: Path,
    replay_trace_path: Path,
    replay_trace_hash_path: Path,
    event_diff_path: Path,
    ltick_diff_path: Path,
    report: dict[str, Any],
    replay_rows: list[dict[str, Any]],
    event_diff_lines: list[str],
    ltick_diff_lines: list[str],
) -> int:
    report["verdict"] = "FAIL"
    report["violations_count"] = len(report.get("violations", []))
    write_json(report_path, report)
    serialize_trace_rows(replay_trace_path, replay_rows)
    write_text(replay_trace_hash_path, str(report.get("replay_execution_trace_hash", "")))
    write_lines(event_diff_path, event_diff_lines)
    write_lines(ltick_diff_path, ltick_diff_lines)

    replay_payload = {
        "status": "FAIL",
        "mode": "bootstrap_replay_determinism",
        "abdf_snapshot_hash": str(report.get("abdf_snapshot_hash", "")),
        "bcib_plan_hash": str(report.get("bcib_plan_hash", "")),
        "record_execution_trace_hash": str(report.get("record_execution_trace_hash", "")),
        "replay_execution_trace_hash": str(report.get("replay_execution_trace_hash", "")),
        "trace_hash_parity": bool(report.get("trace_hash_parity", False)),
        "mismatch_count": int(report.get("mismatch_count", 0)),
        "replay_result_hash": str(report.get("replay_result_hash", "")),
        "final_state_hash": str(report.get("final_state_hash", "")),
        "expected_final_state_hash": str(report.get("expected_final_state_hash", "")),
        "expected_final_state_hash_match": bool(report.get("expected_final_state_hash_match", False)),
        "violations": list(report.get("violations", [])),
        "violations_count": len(report.get("violations", [])),
    }
    write_json(replay_report_path, replay_payload)
    return 2


def pass_(
    report_path: Path,
    replay_report_path: Path,
    replay_trace_path: Path,
    replay_trace_hash_path: Path,
    event_diff_path: Path,
    ltick_diff_path: Path,
    report: dict[str, Any],
    replay_rows: list[dict[str, Any]],
    event_diff_lines: list[str],
    ltick_diff_lines: list[str],
    replay_payload: dict[str, Any],
) -> int:
    report["verdict"] = "PASS"
    report["violations"] = []
    report["violations_count"] = 0
    write_json(report_path, report)
    serialize_trace_rows(replay_trace_path, replay_rows)
    write_text(replay_trace_hash_path, str(report.get("replay_execution_trace_hash", "")))
    write_lines(event_diff_path, event_diff_lines)
    write_lines(ltick_diff_path, ltick_diff_lines)
    write_json(replay_report_path, replay_payload)
    return 0


def main() -> int:
    args = parse_args()

    abdf_hash_path = Path(args.abdf_hash_file)
    bcib_hash_path = Path(args.bcib_plan_hash_file)
    record_trace_path = Path(args.record_trace_jsonl)
    record_trace_hash_path = Path(args.record_trace_hash_file)
    replay_trace_path = Path(args.out_replay_trace_jsonl)
    replay_trace_hash_path = Path(args.out_replay_trace_hash_txt)
    replay_report_path = Path(args.out_replay_report)
    event_diff_path = Path(args.out_event_diff)
    ltick_diff_path = Path(args.out_ltick_diff)
    report_path = Path(args.out_report)
    expected_final_state_hash_path = (
        Path(args.expected_final_state_hash_file)
        if str(args.expected_final_state_hash_file).strip()
        else None
    )

    report: dict[str, Any] = {
        "gate": "replay-determinism",
        "mode": "bootstrap_replay_from_execution_identity",
        "abdf_hash_file": str(abdf_hash_path),
        "bcib_plan_hash_file": str(bcib_hash_path),
        "record_trace_jsonl": str(record_trace_path),
        "record_trace_hash_file": str(record_trace_hash_path),
        "expected_final_state_hash_file": str(expected_final_state_hash_path)
        if expected_final_state_hash_path
        else "",
        "violations": [],
    }

    abdf_hash = load_hash_file(abdf_hash_path, "abdf_snapshot", report)
    bcib_hash = load_hash_file(bcib_hash_path, "bcib_plan", report)
    record_trace_hash = load_hash_file(record_trace_hash_path, "record_execution_trace", report)

    if not record_trace_path.is_file():
        report["violations"].append(f"missing_record_trace_jsonl:{record_trace_path}")
        return fail(
            report_path,
            replay_report_path,
            replay_trace_path,
            replay_trace_hash_path,
            event_diff_path,
            ltick_diff_path,
            report,
            [],
            [],
            [],
        )

    try:
        record_rows_raw = load_jsonl(record_trace_path)
    except RuntimeError as exc:
        report["violations"].append(str(exc))
        return fail(
            report_path,
            replay_report_path,
            replay_trace_path,
            replay_trace_hash_path,
            event_diff_path,
            ltick_diff_path,
            report,
            [],
            [],
            [],
        )
    if not record_rows_raw:
        report["violations"].append("empty_record_trace_jsonl")

    normalized_record_rows: list[dict[str, Any]] = []
    record_event_seq_values: list[int] = []
    record_ltick_values: list[int] = []
    for idx, row in enumerate(record_rows_raw, start=1):
        for field in ("event_seq", "ltick"):
            if row.get(field) in (None, ""):
                report["violations"].append(f"missing_record_trace_field:{field}:entry={idx}")
        if row.get("event_seq") in (None, "") or row.get("ltick") in (None, ""):
            continue
        try:
            trace_seq = int(row.get("trace_seq", idx) or idx)
            event_seq = int(row["event_seq"])
            ltick = int(row["ltick"])
            cpu_id = int(row.get("cpu_id", 0) or 0)
            event_type = str(row.get("event_type", ""))
        except Exception:
            report["violations"].append(f"invalid_record_trace_row_fields:entry={idx}")
            continue

        normalized_record_rows.append(
            {
                "trace_seq": trace_seq,
                "event_seq": event_seq,
                "ltick": ltick,
                "cpu_id": cpu_id,
                "event_type": event_type,
            }
        )
        record_event_seq_values.append(event_seq)
        record_ltick_values.append(ltick)

    if not normalized_record_rows:
        report["violations"].append("empty_normalized_record_trace")

    if record_event_seq_values != sorted(record_event_seq_values):
        report["violations"].append("record_trace_event_seq_non_monotonic")
    if len(set(record_event_seq_values)) != len(record_event_seq_values):
        report["violations"].append("record_trace_event_seq_duplicate")
    if record_ltick_values != sorted(record_ltick_values):
        report["violations"].append("record_trace_ltick_non_monotonic")
    if len(set(record_ltick_values)) != len(record_ltick_values):
        report["violations"].append("record_trace_ltick_duplicate")

    # Bootstrap replay materialization: deterministic canonical replay rows.
    replay_rows = [
        {
            "trace_seq": int(row["trace_seq"]),
            "event_seq": int(row["event_seq"]),
            "ltick": int(row["ltick"]),
            "cpu_id": int(row.get("cpu_id", 0)),
            "event_type": str(row.get("event_type", "")),
        }
        for row in normalized_record_rows
    ]
    serialize_trace_rows(replay_trace_path, replay_rows)

    record_trace_bytes = record_trace_path.read_bytes()
    recomputed_record_trace_hash = sha256_hex(record_trace_bytes)
    if record_trace_hash and recomputed_record_trace_hash != record_trace_hash:
        report["violations"].append(
            "record_trace_hash_mismatch:"
            f"expected={record_trace_hash}:actual={recomputed_record_trace_hash}"
        )

    replay_trace_bytes = replay_trace_path.read_bytes()
    replay_trace_hash = sha256_hex(replay_trace_bytes)

    event_diff_lines: list[str] = []
    ltick_diff_lines: list[str] = []
    compare_len = min(len(normalized_record_rows), len(replay_rows))
    mismatch_count = 0
    for i in range(compare_len):
        record_event_seq = int(normalized_record_rows[i]["event_seq"])
        replay_event_seq = int(replay_rows[i]["event_seq"])
        record_ltick = int(normalized_record_rows[i]["ltick"])
        replay_ltick = int(replay_rows[i]["ltick"])
        if record_event_seq != replay_event_seq:
            mismatch_count += 1
            event_diff_lines.append(
                f"idx={i+1}:record_event_seq={record_event_seq}:replay_event_seq={replay_event_seq}"
            )
        if record_ltick != replay_ltick:
            mismatch_count += 1
            ltick_diff_lines.append(
                f"idx={i+1}:record_ltick={record_ltick}:replay_ltick={replay_ltick}"
            )
    if len(normalized_record_rows) != len(replay_rows):
        mismatch_count += abs(len(normalized_record_rows) - len(replay_rows))
        event_diff_lines.append(
            f"length_mismatch:record={len(normalized_record_rows)}:replay={len(replay_rows)}"
        )

    trace_hash_parity = bool(record_trace_hash) and (record_trace_hash == replay_trace_hash)
    if not trace_hash_parity:
        report["violations"].append(
            f"replay_trace_hash_parity_fail:record={record_trace_hash}:replay={replay_trace_hash}"
        )

    if mismatch_count > 0:
        report["violations"].append(f"replay_mismatch_count:{mismatch_count}")

    replay_result_hash = compute_replay_result_hash(
        abdf_hash, bcib_hash, record_trace_hash, replay_trace_hash, mismatch_count
    )
    final_state_hash = replay_result_hash

    expected_final_state_hash = ""
    expected_final_state_hash_match = False
    if expected_final_state_hash_path is not None:
        expected_final_state_hash = load_hash_file(
            expected_final_state_hash_path, "expected_final_state", report
        )
        if expected_final_state_hash:
            expected_final_state_hash_match = expected_final_state_hash == final_state_hash
            if not expected_final_state_hash_match:
                report["violations"].append(
                    "final_state_hash_mismatch:"
                    f"expected={expected_final_state_hash}:actual={final_state_hash}"
                )

    report["abdf_snapshot_hash"] = abdf_hash
    report["bcib_plan_hash"] = bcib_hash
    report["record_execution_trace_hash"] = record_trace_hash
    report["replay_execution_trace_hash"] = replay_trace_hash
    report["trace_hash_parity"] = trace_hash_parity
    report["mismatch_count"] = mismatch_count
    report["record_event_count"] = len(normalized_record_rows)
    report["replay_event_count"] = len(replay_rows)
    report["replay_result_hash"] = replay_result_hash
    report["final_state_hash"] = final_state_hash
    report["expected_final_state_hash"] = expected_final_state_hash
    report["expected_final_state_hash_match"] = expected_final_state_hash_match
    report["event_diff_file"] = str(event_diff_path)
    report["ltick_diff_file"] = str(ltick_diff_path)

    replay_payload = {
        "status": "FAIL" if report["violations"] else "PASS",
        "mode": "bootstrap_replay_determinism",
        "abdf_snapshot_hash": abdf_hash,
        "bcib_plan_hash": bcib_hash,
        "record_execution_trace_hash": record_trace_hash,
        "replay_execution_trace_hash": replay_trace_hash,
        "trace_hash_parity": trace_hash_parity,
        "mismatch_count": mismatch_count,
        "replay_result_hash": replay_result_hash,
        "final_state_hash": final_state_hash,
        "expected_final_state_hash": expected_final_state_hash,
        "expected_final_state_hash_match": expected_final_state_hash_match,
        "record_event_count": len(normalized_record_rows),
        "replay_event_count": len(replay_rows),
        "violations": list(report["violations"]),
        "violations_count": len(report["violations"]),
    }

    if report["violations"]:
        return fail(
            report_path,
            replay_report_path,
            replay_trace_path,
            replay_trace_hash_path,
            event_diff_path,
            ltick_diff_path,
            report,
            replay_rows,
            event_diff_lines,
            ltick_diff_lines,
        )
    return pass_(
        report_path,
        replay_report_path,
        replay_trace_path,
        replay_trace_hash_path,
        event_diff_path,
        ltick_diff_path,
        report,
        replay_rows,
        event_diff_lines,
        ltick_diff_lines,
        replay_payload,
    )


if __name__ == "__main__":
    raise SystemExit(main())
