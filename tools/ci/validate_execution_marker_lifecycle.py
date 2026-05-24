#!/usr/bin/env python3
"""Validate marker-enabled single-slot kernel lifecycle evidence from QEMU."""

from __future__ import annotations

import argparse
import hashlib
import json
import re
from pathlib import Path

ARMED = "[[AYKEN_EXECUTION_MARKER_LIFECYCLE_ARMED]]"
FAIL = "[[AYKEN_EXECUTION_MARKER_LIFECYCLE_FAIL]]"
EVENT_RE = re.compile(r"^\[\[AYKEN_EXECUTION_MARKER_EVENT\]\] name=(?P<name>[A-Z_]+)$")
OK_RE = re.compile(
    r"^\[\[AYKEN_EXECUTION_MARKER_LIFECYCLE_OK\]\] "
    r"count=(?P<count>\d+) bitmap=(?P<bitmap>\d+) state=(?P<state>\d+)$"
)
HASH_RE = re.compile(
    r"^\[\[AYKEN_EXECUTION_MARKER_RESULT_HASH\]\] sha256=(?P<digest>[0-9a-f]{64})$"
)
EXPECTED_EVENTS = [
    "EXEC_START",
    "EXEC_OUTPUT_WRITTEN",
    "EXEC_COMPLETE_OK",
    "VERIFY_START",
    "VERIFY_PASS",
    "RESULT_OK",
    "WAIT_OK",
]


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Validate real-kernel marker lifecycle QEMU transcript."
    )
    parser.add_argument("--log", required=True, help="Authoritative debugcon log")
    parser.add_argument("--out", required=True, help="Output report JSON")
    parser.add_argument("--violations-out", required=True, help="Output violations text")
    parser.add_argument("--boot-audit-exit-code", required=True, type=int)
    parser.add_argument("--qemu-timeout", required=True, type=int)
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    log_path = Path(args.log)
    report_path = Path(args.out)
    violations_path = Path(args.violations_out)
    data = log_path.read_bytes() if log_path.is_file() else b""
    lines = data.decode("utf-8", errors="replace").splitlines()
    violations: list[str] = []

    armed_count = lines.count(ARMED)
    if armed_count != 1:
        violations.append(f"armed_marker_count:{armed_count}")
    if FAIL in lines:
        violations.append("kernel_lifecycle_failure_marker_present")

    observed_events = [
        match.group("name") for line in lines if (match := EVENT_RE.match(line))
    ]
    if observed_events != EXPECTED_EVENTS:
        violations.append(
            "event_order_mismatch:observed=" + ",".join(observed_events)
        )

    ok_rows = [match for line in lines if (match := OK_RE.match(line))]
    if len(ok_rows) != 1:
        violations.append(f"lifecycle_ok_marker_count:{len(ok_rows)}")
    else:
        ok_row = ok_rows[0]
        if (
            int(ok_row.group("count")) != len(EXPECTED_EVENTS)
            or int(ok_row.group("bitmap")) != 127
            or int(ok_row.group("state")) != 6
        ):
            violations.append("lifecycle_ok_metadata_invalid")

    hash_rows = [match.group("digest") for line in lines if (match := HASH_RE.match(line))]
    if len(hash_rows) != 1:
        violations.append(f"result_hash_marker_count:{len(hash_rows)}")

    if args.boot_audit_exit_code != 0:
        violations.append(f"boot_audit_exit_code:{args.boot_audit_exit_code}")

    verdict = "PASS" if not violations else "FAIL"
    report = {
        "gate": "execution-marker-lifecycle",
        "verdict": verdict,
        "guarantee_level": "real_kernel_qemu_single_slot_lifecycle",
        "scope": "validation_only_marker_enabled_execution_slot",
        "feature_flags": {
            "AYKEN_EXECUTION_MARKER_VALIDATION_ENABLE": 1,
            "AYKEN_EXECUTION_MARKER_LIFECYCLE_SELFTEST": 1,
            "production_default_off_required": True,
        },
        "does_prove": [
            "qemu_booted_marker_enabled_kernel",
            "single_slot_lifecycle_reached_result_mapped",
            "canonical_seven_marker_order_observed",
            "single_run_result_fingerprint_emitted",
        ],
        "does_not_prove": [
            "phase17_closure",
            "ring3_public_syscall_end_to_end_submission",
            "two_run_result_determinism",
            "race_isolation",
            "performance_acceptance",
        ],
        "expected_events": EXPECTED_EVENTS,
        "observed_events": observed_events,
        "armed_marker_count": armed_count,
        "result_fingerprint_sha256": hash_rows[0] if len(hash_rows) == 1 else "",
        "boot_audit_exit_code": args.boot_audit_exit_code,
        "qemu_timeout_seconds": args.qemu_timeout,
        "authoritative_log": str(log_path),
        "authoritative_log_sha256": hashlib.sha256(data).hexdigest(),
        "violations_count": len(violations),
        "violations": violations,
    }

    report_path.parent.mkdir(parents=True, exist_ok=True)
    report_path.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    violations_path.parent.mkdir(parents=True, exist_ok=True)
    violations_path.write_text("\n".join(violations) + ("\n" if violations else ""), encoding="utf-8")
    print(f"execution-marker-lifecycle: {verdict}")
    return 0 if verdict == "PASS" else 2


if __name__ == "__main__":
    raise SystemExit(main())
