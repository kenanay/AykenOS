#!/usr/bin/env python3
"""Validate validation-only IRQ-timeout versus delayed completion evidence."""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path

ARMED = "[[AYKEN_EXEC_TIMEOUT_RACE_ARMED]]"
ENTRY_GUARD_ARM = "P10_RING3_ENTRY_GUARD_ARM"
ENTRY_GUARD_DISARM = "P10_RING3_ENTRY_GUARD_DISARM"
SUBMIT_OK = "[[AYKEN_PUBLIC_EXEC_SUBMIT_OK]]"
DEADLINE_ARMED = "[[AYKEN_EXEC_RACE_DEADLINE_ARMED]]"
IRQ_TIMEOUT_OK = "[[AYKEN_EXEC_RACE_IRQ_TIMEOUT_OK]]"
WAIT_TIMEOUT_OK = "[[AYKEN_EXEC_RACE_WAIT_TIMEOUT_OK]]"
LATE_COMPLETE_REJECT_OK = "[[AYKEN_EXEC_RACE_LATE_COMPLETE_REJECT_OK]]"
USER_OK = "[[AYKEN_EXEC_RACE_USER_OBSERVED_OK]]"
ORDERED_MARKERS = (
    ARMED,
    ENTRY_GUARD_ARM,
    ENTRY_GUARD_DISARM,
    SUBMIT_OK,
    DEADLINE_ARMED,
    IRQ_TIMEOUT_OK,
    WAIT_TIMEOUT_OK,
    LATE_COMPLETE_REJECT_OK,
    USER_OK,
)
FORBIDDEN_MARKERS = (
    "[[AYKEN_EXEC_TIMEOUT_RACE_USER_FAIL]]",
    "[[AYKEN_PUBLIC_EXEC_WORKER_COMPLETE_OK]]",
    "[[AYKEN_PUBLIC_EXEC_WAIT_OK]]",
    "[[AYKEN_EXECUTION_MARKER_LIFECYCLE_OK]]",
    "[[AYKEN_EXECUTION_MARKER_RESULT_HASH]]",
    "PF!",
)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Validate real-QEMU IRQ timeout winning over late public completion."
    )
    parser.add_argument("--log", required=True)
    parser.add_argument("--out", required=True)
    parser.add_argument("--violations-out", required=True)
    parser.add_argument("--boot-audit-exit-code", required=True, type=int)
    parser.add_argument("--qemu-timeout", required=True, type=int)
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    log_path = Path(args.log)
    data = log_path.read_bytes() if log_path.is_file() else b""
    text = data.decode("utf-8", errors="replace")
    violations: list[str] = []
    positions: dict[str, int] = {}

    for marker in ORDERED_MARKERS:
        count = text.count(marker)
        if count != 1:
            violations.append(f"marker_count:{marker}:{count}")
        else:
            positions[marker] = text.index(marker)

    if len(positions) == len(ORDERED_MARKERS):
        marker_positions = [positions[marker] for marker in ORDERED_MARKERS]
        if marker_positions != sorted(marker_positions):
            violations.append("timeout_race_marker_order_invalid")

    observed_forbidden = [marker for marker in FORBIDDEN_MARKERS if marker in text]
    violations.extend(f"forbidden_marker_present:{marker}" for marker in observed_forbidden)

    if args.boot_audit_exit_code != 0:
        violations.append(f"boot_audit_exit_code:{args.boot_audit_exit_code}")

    verdict = "PASS" if not violations else "FAIL"
    intended_proof_scope = [
        "ring3_submitted_self_target_execution_through_public_1003",
        "validation_harness_armed_bounded_logical_deadline_after_running_delivery",
        "real_timer_irq_terminalized_running_slot_as_timeout",
        "running_ring3_poll_observed_timeout_terminal_state",
        "delayed_ring3_public_complete_execution_1011_was_rejected_after_timeout",
        "timeout_path_published_no_completed_result_witness",
    ]
    report = {
        "gate": "execution-timeout-race",
        "verdict": verdict,
        "guarantee_level": "validation_only_real_irq_timeout_wins_over_late_completion",
        "scope": "ring3_public_submit_delivered_running_deadline_poll_then_late_complete_reject",
        "feature_flags": {
            "AYKEN_EXECUTION_RACE_SELFTEST": 1,
            "AYKEN_BCIB_STUB_RESULT_ENABLE": 0,
            "AYKEN_EXECUTION_MARKER_VALIDATION_ENABLE": 1,
            "AYKEN_RING3_ENTRY_GUARD": 1,
            "AYKEN_RING3_MASK_IRQ0_FIRST_ENTRY": 0,
            "production_default_off_required": True,
        },
        "intended_proof_scope": intended_proof_scope,
        "does_prove": intended_proof_scope if verdict == "PASS" else [],
        "does_not_prove": [
            "exhaustive_scheduler_or_interrupt_race_matrix",
            "smp_race_safety",
            "general_bcib_interpreter_or_full_opcode_surface",
            "phase17_closure",
            "performance_acceptance",
            "production_selftest_enabled_behavior",
        ],
        "expected_ordered_markers": list(ORDERED_MARKERS),
        "forbidden_success_or_fault_markers": list(FORBIDDEN_MARKERS),
        "observed_marker_positions": positions,
        "observed_forbidden_markers": observed_forbidden,
        "boot_audit_exit_code": args.boot_audit_exit_code,
        "qemu_timeout_seconds": args.qemu_timeout,
        "authoritative_log": str(log_path),
        "authoritative_log_sha256": hashlib.sha256(data).hexdigest(),
        "violations_count": len(violations),
        "violations": violations,
    }

    out = Path(args.out)
    violations_out = Path(args.violations_out)
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    violations_out.parent.mkdir(parents=True, exist_ok=True)
    violations_out.write_text("\n".join(violations) + ("\n" if violations else ""), encoding="utf-8")
    print(f"execution-timeout-race: {verdict}")
    return 0 if verdict == "PASS" else 2


if __name__ == "__main__":
    raise SystemExit(main())
