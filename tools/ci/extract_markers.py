#!/usr/bin/env python3
"""Extract canonical runtime markers from a combined QEMU log."""

from __future__ import annotations

import argparse
import json
import re
from pathlib import Path

# Prefix-tolerant parsing: marker payload may be wrapped by timestamps/log prefixes.
ACCEPT_RE = re.compile(
    r"\[\[AYKEN_SCHED_MB_ACCEPT\]\]\s+pid=([0-9]+)\s+epoch=([0-9]+)(?=\s|$)"
)
REJECT_RE = re.compile(
    r"\[\[AYKEN_SCHED_MB_REJECT\]\]\s+reason=([0-9]+)\s+epoch=([0-9]+)\s+pid=([0-9]+)(?=\s|$)"
)
RING3_KERNEL_RE = re.compile(r"\[\[AYKEN_RING3_OK\]\](?=\s|$)")
RING3_USER_RE = re.compile(r"(?<![A-Za-z0-9_])R3OK(?![A-Za-z0-9_])")


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Parse canonical scheduler/ring3 markers from log lines."
    )
    parser.add_argument("--log", required=True, help="Input combined log path")
    parser.add_argument("--out", required=True, help="Output events.jsonl path")
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    log_path = Path(args.log)
    out_path = Path(args.out)

    if not log_path.is_file():
        raise SystemExit(f"missing input log: {log_path}")

    out_path.parent.mkdir(parents=True, exist_ok=True)

    with log_path.open("r", encoding="utf-8", errors="replace") as src, out_path.open(
        "w", encoding="utf-8"
    ) as dst:
        for line_no, raw in enumerate(src, start=1):
            line = raw.rstrip("\r\n")

            m = ACCEPT_RE.search(line)
            if m:
                event = {
                    "type": "ACCEPT",
                    "pid": int(m.group(1)),
                    "epoch": int(m.group(2)),
                    "line": line_no,
                    "raw": line,
                }
                dst.write(json.dumps(event, sort_keys=True) + "\n")
                continue

            m = REJECT_RE.search(line)
            if m:
                event = {
                    "type": "REJECT",
                    "reason": int(m.group(1)),
                    "epoch": int(m.group(2)),
                    "pid": int(m.group(3)),
                    "line": line_no,
                    "raw": line,
                }
                dst.write(json.dumps(event, sort_keys=True) + "\n")
                continue

            if RING3_KERNEL_RE.search(line):
                event = {
                    "type": "AYKEN_RING3_OK",
                    "line": line_no,
                    "raw": line,
                }
                dst.write(json.dumps(event, sort_keys=True) + "\n")
                continue

            if RING3_USER_RE.search(line):
                event = {
                    "type": "R3OK_USER_TOKEN",
                    "line": line_no,
                    "raw": line,
                }
                dst.write(json.dumps(event, sort_keys=True) + "\n")

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
