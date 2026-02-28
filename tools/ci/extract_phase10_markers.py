#!/usr/bin/env python3
"""Extract Phase10-A2 ring3 execution markers from a combined QEMU log."""

from __future__ import annotations

import argparse
import bisect
import json
from pathlib import Path

MARKERS = [
    ("KERNEL_BEFORE_RING3", "[K][PHASE10] KERNEL_BEFORE_RING3"),
    ("AYKEN_RING3_PREP_OK", "[[AYKEN_RING3_PREP_OK]]"),
    ("P10_TSS_OK", "P10_TSS_OK"),
    ("P10_SCHED_DISPATCH", "P10_SCHED_DISPATCH"),
    ("P10_MAILBOX_DECISION", "P10_MAILBOX_DECISION"),
    ("P10_DECISION_APPLIED", "P10_DECISION_APPLIED"),
    ("P10_SCHED_FALLBACK", "P10_SCHED_FALLBACK"),
    ("P10_READY_HEAD_FALLBACK", "P10_READY_HEAD_FALLBACK"),
    ("P10_MAILBOX_OWNER_MISMATCH", "P10_MAILBOX_OWNER_MISMATCH"),
    ("P10_MAILBOX_OWNER_MISSING_FATAL", "P10_MAILBOX_OWNER_MISSING_FATAL"),
    ("P10_MAILBOX_OWNER_NOT_READY_FATAL", "P10_MAILBOX_OWNER_NOT_READY_FATAL"),
    ("P10_RFLAGS_IF_ON", "P10_RFLAGS_IF_ON"),
    ("P10_RFLAGS_IF_OFF", "P10_RFLAGS_IF_OFF"),
    ("P10_CR3_SWITCH", "P10_CR3_SWITCH"),
    ("P10_RING3_ATTEMPT", "P10_RING3_ATTEMPT"),
    ("P10_RING3_COMMIT", "P10_RING3_COMMIT"),
    ("P10_RING3_ENTER", "P10_RING3_ENTER"),
    ("AYKEN_SYSCALL_ENTER", "[[AYKEN_SYSCALL_ENTER]]"),
    ("P10_SYSCALL_ENTER", "P10_SYSCALL_ENTER"),
    ("AYKEN_SYSCALL_RETURN", "[[AYKEN_SYSCALL_RETURN]]"),
    ("P10_SYSCALL_RETURN", "P10_SYSCALL_RETURN"),
    ("P10_CAP_ENFORCED", "P10_CAP_ENFORCED"),
    ("P10_RING3_USER_CODE", "P10_RING3_USER_CODE"),
]


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Extract Phase10 runtime markers from a log stream."
    )
    parser.add_argument("--log", required=True, help="Input combined log path")
    parser.add_argument("--out", required=True, help="Output events.jsonl path")
    return parser.parse_args()


def collect_occurrences(haystack: str) -> list[dict]:
    newline_offsets = [idx for idx, ch in enumerate(haystack) if ch == "\n"]
    rows: list[dict] = []

    for name, token in MARKERS:
        start = 0
        while True:
            idx = haystack.find(token, start)
            if idx < 0:
                break

            line_no = bisect.bisect_right(newline_offsets, idx) + 1
            rows.append(
                {
                    "type": name,
                    "marker": token,
                    "offset": idx,
                    "line": line_no,
                }
            )
            start = idx + len(token)

    rows.sort(key=lambda row: row["offset"])
    return rows


def main() -> int:
    args = parse_args()
    log_path = Path(args.log)
    out_path = Path(args.out)

    if not log_path.is_file():
        raise SystemExit(f"missing input log: {log_path}")

    text = log_path.read_text(encoding="utf-8", errors="replace")
    events = collect_occurrences(text)

    out_path.parent.mkdir(parents=True, exist_ok=True)
    with out_path.open("w", encoding="utf-8") as fh:
        for event in events:
            fh.write(json.dumps(event, sort_keys=True) + "\n")

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
