#!/usr/bin/env python3
"""Validate Phase-17 PR-2 deterministic result and invalid-order evidence."""

from __future__ import annotations

import argparse
import hashlib
import json
import re
from pathlib import Path
from typing import Any


ARMED = "[[AYKEN_EXECUTION_MARKER_LIFECYCLE_ARMED]]"
FAIL = "[[AYKEN_EXECUTION_MARKER_LIFECYCLE_FAIL]]"
POS_EVENT_RE = re.compile(
    r"^\[\[AYKEN_EXECUTION_MARKER_EVENT\]\] name=(?P<name>[A-Z_]+)$"
)
NEG_EVENT_RE = re.compile(
    r"^\[\[AYKEN_EXECUTION_MARKER_NEGATIVE_EVENT\]\] name=(?P<name>[A-Z_]+)$"
)
HASH_RE = re.compile(
    r"^\[\[AYKEN_EXECUTION_MARKER_RESULT_HASH\]\] sha256=(?P<digest>[0-9a-f]{64})$"
)
OK_RE = re.compile(
    r"^\[\[AYKEN_EXECUTION_MARKER_LIFECYCLE_OK\]\] "
    r"count=(?P<count>\d+) bitmap=(?P<bitmap>\d+) state=(?P<state>\d+)$"
)
NEGATIVE_OK_RE = re.compile(
    r"^\[\[AYKEN_EXECUTION_MARKER_NEGATIVE_OK\]\] "
    r"reason=invalid_order state=2 hash_size=0 mapped=0 result_ok=0 wait_ok=0$"
)

EXPECTED_POSITIVE_EVENTS = [
    "EXEC_START",
    "EXEC_OUTPUT_WRITTEN",
    "EXEC_COMPLETE_OK",
    "VERIFY_START",
    "VERIFY_PASS",
    "RESULT_OK",
    "WAIT_OK",
]
EXPECTED_NEGATIVE_EVENTS = [
    "EXEC_START",
    "EXEC_COMPLETE_OK",
    "EXEC_OUTPUT_WRITTEN",
    "VERIFY_START",
    "VERIFY_PASS",
]


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Validate two-boot result fingerprint parity and invalid-order rejection."
    )
    parser.add_argument("--run-a-log", required=True)
    parser.add_argument("--run-b-log", required=True)
    parser.add_argument("--negative-log", required=True)
    parser.add_argument("--run-a-exit-code", required=True, type=int)
    parser.add_argument("--run-b-exit-code", required=True, type=int)
    parser.add_argument("--negative-exit-code", required=True, type=int)
    parser.add_argument("--qemu-timeout", required=True, type=int)
    parser.add_argument("--out", required=True)
    parser.add_argument("--violations-out", required=True)
    return parser.parse_args()


def read_lines(path: Path) -> tuple[bytes, list[str]]:
    data = path.read_bytes() if path.is_file() else b""
    return data, data.decode("utf-8", errors="replace").splitlines()


def validate_positive(
    label: str, path: Path, exit_code: int, violations: list[str]
) -> dict[str, Any]:
    data, lines = read_lines(path)
    events = [
        match.group("name") for line in lines if (match := POS_EVENT_RE.match(line))
    ]
    hashes = [match.group("digest") for line in lines if (match := HASH_RE.match(line))]
    ok_rows = [match for line in lines if (match := OK_RE.match(line))]

    if lines.count(ARMED) != 1:
        violations.append(f"{label}:armed_marker_count:{lines.count(ARMED)}")
    if FAIL in lines:
        violations.append(f"{label}:lifecycle_failure_marker_present")
    if exit_code != 0:
        violations.append(f"{label}:boot_audit_exit_code:{exit_code}")
    if events != EXPECTED_POSITIVE_EVENTS:
        violations.append(f"{label}:event_order_mismatch:" + ",".join(events))
    if len(hashes) != 1:
        violations.append(f"{label}:result_hash_marker_count:{len(hashes)}")
    if len(ok_rows) != 1:
        violations.append(f"{label}:lifecycle_ok_marker_count:{len(ok_rows)}")
    else:
        row = ok_rows[0]
        if (int(row.group("count")), int(row.group("bitmap")), int(row.group("state"))) != (
            7,
            127,
            6,
        ):
            violations.append(f"{label}:lifecycle_ok_metadata_invalid")

    return {
        "log": str(path),
        "log_sha256": hashlib.sha256(data).hexdigest(),
        "events": events,
        "result_fingerprint_sha256": hashes[0] if len(hashes) == 1 else "",
        "boot_audit_exit_code": exit_code,
    }


def validate_negative(path: Path, exit_code: int, violations: list[str]) -> dict[str, Any]:
    data, lines = read_lines(path)
    events = [
        match.group("name") for line in lines if (match := NEG_EVENT_RE.match(line))
    ]
    positive_events = [
        match.group("name") for line in lines if (match := POS_EVENT_RE.match(line))
    ]
    hash_rows = [match.group("digest") for line in lines if (match := HASH_RE.match(line))]
    negative_ok_count = sum(1 for line in lines if NEGATIVE_OK_RE.match(line))

    if lines.count(ARMED) != 1:
        violations.append(f"negative:armed_marker_count:{lines.count(ARMED)}")
    if FAIL in lines:
        violations.append("negative:lifecycle_failure_marker_present")
    if exit_code != 0:
        violations.append(f"negative:boot_audit_exit_code:{exit_code}")
    if events != EXPECTED_NEGATIVE_EVENTS:
        violations.append("negative:event_order_mismatch:" + ",".join(events))
    if positive_events:
        violations.append("negative:published_positive_events:" + ",".join(positive_events))
    if hash_rows:
        violations.append("negative:result_hash_published")
    if any(OK_RE.match(line) for line in lines):
        violations.append("negative:lifecycle_ok_published")
    if negative_ok_count != 1:
        violations.append(f"negative:acceptance_marker_count:{negative_ok_count}")

    return {
        "log": str(path),
        "log_sha256": hashlib.sha256(data).hexdigest(),
        "observed_rejected_prefix": events,
        "boot_audit_exit_code": exit_code,
        "rejection_marker_count": negative_ok_count,
        "hash_published": bool(hash_rows),
    }


def main() -> int:
    args = parse_args()
    violations: list[str] = []
    run_a = validate_positive("run_a", Path(args.run_a_log), args.run_a_exit_code, violations)
    run_b = validate_positive("run_b", Path(args.run_b_log), args.run_b_exit_code, violations)
    negative = validate_negative(Path(args.negative_log), args.negative_exit_code, violations)
    fingerprints_match = (
        bool(run_a["result_fingerprint_sha256"])
        and run_a["result_fingerprint_sha256"] == run_b["result_fingerprint_sha256"]
    )
    if not fingerprints_match:
        violations.append("positive_runs:result_fingerprint_mismatch")

    report = {
        "gate": "execution-marker-determinism",
        "verdict": "PASS" if not violations else "FAIL",
        "guarantee_level": "real_kernel_qemu_result_repeat_and_invalid_order_rejection",
        "scope": "validation_only_marker_enabled_execution_slot",
        "feature_flags": {
            "positive": {
                "AYKEN_EXECUTION_MARKER_VALIDATION_ENABLE": 1,
                "AYKEN_EXECUTION_MARKER_LIFECYCLE_SELFTEST": 1,
            },
            "negative": {
                "AYKEN_EXECUTION_MARKER_VALIDATION_ENABLE": 1,
                "AYKEN_EXECUTION_MARKER_LIFECYCLE_SELFTEST": 1,
                "AYKEN_PHASE17_MARKER_INJECTION_TEST": 1,
                "AYKEN_MARKER_INJECT_INVALID_ORDER": 1,
                "AYKEN_EXECUTION_MARKER_NEGATIVE_EXPECT_REJECT": 1,
            },
            "production_default_off_required": True,
        },
        "does_prove": [
            "two_qemu_boots_same_validation_input_same_kernel_result_fingerprint",
            "invalid_marker_order_rejected_before_hash_or_result_mapping_publication",
        ],
        "does_not_prove": [
            "phase17_closure",
            "ring3_public_syscall_end_to_end_submission",
            "scheduler_or_interrupt_race_isolation",
            "performance_acceptance",
            "resource_rollback_after_rejected_verification",
        ],
        "expected_positive_events": EXPECTED_POSITIVE_EVENTS,
        "expected_negative_events": EXPECTED_NEGATIVE_EVENTS,
        "run_a": run_a,
        "run_b": run_b,
        "negative": negative,
        "fingerprints_match": fingerprints_match,
        "qemu_timeout_seconds": args.qemu_timeout,
        "violations_count": len(violations),
        "violations": violations,
    }

    out = Path(args.out)
    violations_out = Path(args.violations_out)
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    violations_out.parent.mkdir(parents=True, exist_ok=True)
    violations_out.write_text(
        "\n".join(violations) + ("\n" if violations else ""), encoding="utf-8"
    )
    print(f"execution-marker-determinism: {report['verdict']}")
    return 0 if not violations else 2


if __name__ == "__main__":
    raise SystemExit(main())
