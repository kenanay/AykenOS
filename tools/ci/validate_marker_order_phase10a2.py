#!/usr/bin/env python3
"""Validate strict Phase10-A2 marker order from extracted events."""

from __future__ import annotations

import argparse
import json
from pathlib import Path

EXPECTED_SEQUENCE = [
    "P10_TSS_OK",
    "KERNEL_BEFORE_RING3",
    "AYKEN_RING3_PREP_OK",
    "P10_SCHED_DISPATCH",
    "P10_RFLAGS_IF_ON",
    "P10_CR3_SWITCH",
    "P10_RING3_ENTER",
    "AYKEN_SYSCALL_ENTER",
    "P10_SYSCALL_ENTER",
    "AYKEN_SYSCALL_RETURN",
    "P10_SYSCALL_RETURN",
    "P10_CAP_ENFORCED",
    "P10_RING3_USER_CODE",
]

FORBIDDEN_AFTER_ENTER = [
    "GP!",
    "PF!",
    "DF!",
    "TRIPLE",
    "PANIC",
    "[[AYKEN_RING3_PREP_FAIL]]",
    "[[AYKEN_RING3_FAIL]]",
]

FATAL_ANYWHERE = [
    "TRIPLE",
    "PANIC",
    "P10_RFLAGS_IF_OFF",
]


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Validate strict marker order for Phase10-A2 roundtrip."
    )
    parser.add_argument("--events", required=True, help="Input events.jsonl")
    parser.add_argument("--log", required=True, help="Input combined log")
    parser.add_argument("--out", required=True, help="Output report.json")
    return parser.parse_args()


def load_events(path: Path) -> list[dict]:
    events: list[dict] = []
    with path.open("r", encoding="utf-8", errors="replace") as fh:
        for line_no, raw in enumerate(fh, start=1):
            row = raw.strip()
            if not row:
                continue
            try:
                event = json.loads(row)
            except Exception as exc:  # pragma: no cover - fail-closed path
                raise RuntimeError(
                    f"events_parse_error:{path}:line={line_no}:{type(exc).__name__}"
                ) from exc
            if not isinstance(event, dict):
                raise RuntimeError(f"events_type_error:{path}:line={line_no}")
            events.append(event)
    return events


def validate(events: list[dict], log_text: str) -> dict:
    violations: list[str] = []
    details: list[dict] = []

    if len(set(EXPECTED_SEQUENCE)) != len(EXPECTED_SEQUENCE):
        violations.append("validator_config_duplicate_expected_sequence_entries")

    by_type: dict[str, list[dict]] = {}
    for event in events:
        marker_type = str(event.get("type", ""))
        by_type.setdefault(marker_type, []).append(event)

    cursor = -1
    found_offsets: dict[str, int] = {}
    found_lines: dict[str, int] = {}

    for marker_type in EXPECTED_SEQUENCE:
        rows = by_type.get(marker_type, [])
        if len(rows) == 0:
            violations.append(f"missing_marker:{marker_type}")
            details.append(
                {
                    "marker": marker_type,
                    "offset": -1,
                    "line": 0,
                    "status": "missing",
                }
            )
            continue
        if len(rows) != 1:
            violations.append(f"extra_marker:{marker_type}:count={len(rows)}")

        chosen = None
        for row in rows:
            row_offset = int(row.get("offset", -1))
            if row_offset > cursor:
                chosen = row
                break

        if chosen is None:
            violations.append(f"out_of_order:{marker_type}")
            chosen = rows[0]

        offset = int(chosen.get("offset", -1))
        line = int(chosen.get("line", 0))
        cursor = max(cursor, offset)
        found_offsets[marker_type] = offset
        found_lines[marker_type] = line
        details.append(
            {
                "marker": marker_type,
                "offset": offset,
                "line": line,
                "status": "present",
            }
        )

    for token in FATAL_ANYWHERE:
        pos = log_text.find(token)
        if pos >= 0:
            line = log_text.count("\n", 0, pos) + 1
            violations.append(f"fatal_token_anywhere:{token}:line={line}")

    enter_offset = found_offsets.get("P10_RING3_ENTER")
    if enter_offset is not None and enter_offset >= 0:
        tail = log_text[enter_offset:]
        for token in FORBIDDEN_AFTER_ENTER:
            pos = tail.find(token)
            if pos >= 0:
                absolute = enter_offset + pos
                line = log_text.count("\n", 0, absolute) + 1
                violations.append(f"forbidden_token_after_enter:{token}:line={line}")

    verdict = "PASS" if len(violations) == 0 else "FAIL"
    return {
        "gate": "ring3-execution-phase10a2",
        "verdict": verdict,
        "violations_count": len(violations),
        "violations": violations,
        "expected_sequence": EXPECTED_SEQUENCE,
        "observed_sequence": details,
        "forbidden_after_enter": FORBIDDEN_AFTER_ENTER,
    }


def main() -> int:
    args = parse_args()
    events_path = Path(args.events)
    log_path = Path(args.log)
    out_path = Path(args.out)

    if not events_path.is_file():
        raise SystemExit(f"missing events file: {events_path}")
    if not log_path.is_file():
        raise SystemExit(f"missing log file: {log_path}")

    events = load_events(events_path)
    log_text = log_path.read_text(encoding="utf-8", errors="replace")
    report = validate(events, log_text)

    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n", encoding="utf-8")

    if report["verdict"] != "PASS":
        return 2
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
