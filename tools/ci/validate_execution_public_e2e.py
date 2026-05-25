#!/usr/bin/env python3
"""Validate validation-only public Ring3 submit/wait execution evidence."""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path

ARMED = "[[AYKEN_PUBLIC_EXEC_E2E_ARMED]]"
ENTRY_GUARD_ARM = "P10_RING3_ENTRY_GUARD_ARM"
ENTRY_GUARD_DISARM = "P10_RING3_ENTRY_GUARD_DISARM"
SUBMIT_OK = "[[AYKEN_PUBLIC_EXEC_SUBMIT_OK]]"
OUTPUT_WRITTEN = "[EXEC_OUTPUT_WRITTEN]"
COMPLETE_OK = "[EXEC_COMPLETE_OK]"
WAIT_OK = "[[AYKEN_PUBLIC_EXEC_WAIT_OK]] count=7 bitmap=127 state=6"
USER_OK = "[[AYKEN_SYSCALL_V2_OK]]"
FAIL_MARKERS = (
    "[[AYKEN_PUBLIC_EXEC_WAIT_FAIL]]",
    "[[AYKEN_PUBLIC_EXEC_E2E_USER_FAIL]]",
    "[[AYKEN_EXECUTION_MARKER_LIFECYCLE_FAIL]]",
)
ORDERED_MARKERS = (
    ARMED,
    ENTRY_GUARD_ARM,
    ENTRY_GUARD_DISARM,
    SUBMIT_OK,
    OUTPUT_WRITTEN,
    COMPLETE_OK,
    WAIT_OK,
    USER_OK,
)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Validate public Ring3 submit/wait QEMU transcript."
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
    text = data.decode("utf-8", errors="replace")
    violations: list[str] = []
    marker_positions: dict[str, int] = {}

    for marker in ORDERED_MARKERS:
        count = text.count(marker)
        if count != 1:
            violations.append(f"marker_count:{marker}:{count}")
        else:
            marker_positions[marker] = text.index(marker)

    if len(marker_positions) == len(ORDERED_MARKERS):
        positions = [marker_positions[marker] for marker in ORDERED_MARKERS]
        if positions != sorted(positions):
            violations.append("public_e2e_marker_order_invalid")

    observed_failures = [marker for marker in FAIL_MARKERS if marker in text]
    for marker in observed_failures:
        violations.append(f"failure_marker_present:{marker}")

    if args.boot_audit_exit_code != 0:
        violations.append(f"boot_audit_exit_code:{args.boot_audit_exit_code}")

    verdict = "PASS" if not violations else "FAIL"
    intended_proof_scope = [
        "ring3_first_syscall_progress_was_guarded_until_gate_entry",
        "ring3_invoked_public_submit_execution_1003",
        "scheduler_picked_up_submitted_slot_in_qemu",
        "ring3_invoked_public_wait_result_1004_and_read_frozen_published_result",
        "public_path_reached_canonical_seven_marker_result_mapped_boundary",
        "ring3_verified_mapped_stub_payload_before_canonical_debug_witness",
    ]
    report = {
        "gate": "execution-public-e2e",
        "verdict": verdict,
        "guarantee_level": "validation_only_public_ring3_submit_wait_result_publication",
        "scope": "ring3_public_syscall_boundary_with_deterministic_stub_completion",
        "feature_flags": {
            "AYKEN_BCIB_PUBLIC_E2E_SELFTEST": 1,
            "AYKEN_BCIB_STUB_RESULT_ENABLE": 1,
            "AYKEN_EXECUTION_MARKER_VALIDATION_ENABLE": 1,
            "AYKEN_RING3_ENTRY_GUARD": 1,
            "AYKEN_RING3_MASK_IRQ0_FIRST_ENTRY": 0,
            "production_default_off_required": True,
        },
        "intended_proof_scope": intended_proof_scope,
        "does_prove": intended_proof_scope if verdict == "PASS" else [],
        "does_not_prove": [
            "real_bcib_interpreter_or_worker_completion",
            "phase17_closure",
            "scheduler_or_interrupt_race_isolation",
            "performance_acceptance",
            "production_stub_enabled_behavior",
        ],
        "expected_ordered_markers": list(ORDERED_MARKERS),
        "observed_marker_positions": marker_positions,
        "observed_failure_markers": observed_failures,
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
    print(f"execution-public-e2e: {verdict}")
    return 0 if verdict == "PASS" else 2


if __name__ == "__main__":
    raise SystemExit(main())
