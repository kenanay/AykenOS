#!/usr/bin/env python3
"""Validate validation-only Ring3 BCIB fixture completion evidence."""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path

ARMED = "[[AYKEN_BCIB_WORKER_COMPLETION_ARMED]]"
ENTRY_GUARD_ARM = "P10_RING3_ENTRY_GUARD_ARM"
ENTRY_GUARD_DISARM = "P10_RING3_ENTRY_GUARD_DISARM"
SUBMIT_OK = "[[AYKEN_PUBLIC_EXEC_SUBMIT_OK]]"
COMPLETE_OK = "[[AYKEN_PUBLIC_EXEC_WORKER_COMPLETE_OK]]"
WAIT_OK = "[[AYKEN_PUBLIC_EXEC_WAIT_OK]] count=7 bitmap=127 state=6"
USER_OK = "[[AYKEN_BCIB_WORKER_USER_OBSERVED_OK]]"
FAIL_MARKERS = (
    "[[AYKEN_PUBLIC_EXEC_WAIT_FAIL]]",
    "[[AYKEN_BCIB_WORKER_COMPLETION_USER_FAIL]]",
    "[[AYKEN_EXECUTION_MARKER_LIFECYCLE_FAIL]]",
)
ORDERED_MARKERS = (
    ARMED,
    ENTRY_GUARD_ARM,
    ENTRY_GUARD_DISARM,
    SUBMIT_OK,
    COMPLETE_OK,
    WAIT_OK,
    USER_OK,
)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Validate Ring3 literal-fixture complete_execution QEMU transcript."
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
        ordered_positions = [positions[marker] for marker in ORDERED_MARKERS]
        if ordered_positions != sorted(ordered_positions):
            violations.append("worker_completion_marker_order_invalid")

    observed_failures = [marker for marker in FAIL_MARKERS if marker in text]
    violations.extend(f"failure_marker_present:{marker}" for marker in observed_failures)

    if args.boot_audit_exit_code != 0:
        violations.append(f"boot_audit_exit_code:{args.boot_audit_exit_code}")

    verdict = "PASS" if not violations else "FAIL"
    intended_proof_scope = [
        "ring3_read_delivered_bcib_literal_fixture_from_inbox_payload_surface",
        "ring3_wrote_validated_output_window_for_fixture_result",
        "ring3_invoked_public_complete_execution_1011_with_stub_disabled",
        "ring3_invoked_public_wait_result_1004_and_read_frozen_worker_result",
        "public_worker_path_reached_canonical_seven_marker_result_mapped_boundary",
    ]
    report = {
        "gate": "execution-worker-completion",
        "verdict": verdict,
        "guarantee_level": "validation_only_ring3_literal_fixture_public_completion",
        "scope": "ring3_public_submit_complete_wait_with_stub_disabled",
        "fixture": {
            "operation": "literal_result_u64",
            "payload_hex": "42434942171101008877665544332211",
            "expected_result_u64": "0x1122334455667788",
        },
        "feature_flags": {
            "AYKEN_BCIB_WORKER_COMPLETION_SELFTEST": 1,
            "AYKEN_BCIB_STUB_RESULT_ENABLE": 0,
            "AYKEN_EXECUTION_MARKER_VALIDATION_ENABLE": 1,
            "AYKEN_RING3_ENTRY_GUARD": 1,
            "AYKEN_RING3_MASK_IRQ0_FIRST_ENTRY": 0,
            "production_default_off_required": True,
        },
        "intended_proof_scope": intended_proof_scope,
        "does_prove": intended_proof_scope if verdict == "PASS" else [],
        "does_not_prove": [
            "general_bcib_interpreter_or_full_opcode_surface",
            "phase17_closure",
            "scheduler_or_interrupt_race_isolation",
            "performance_acceptance",
            "production_selftest_enabled_behavior",
        ],
        "expected_ordered_markers": list(ORDERED_MARKERS),
        "observed_marker_positions": positions,
        "observed_failure_markers": observed_failures,
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
    print(f"execution-worker-completion: {verdict}")
    return 0 if verdict == "PASS" else 2


if __name__ == "__main__":
    raise SystemExit(main())
