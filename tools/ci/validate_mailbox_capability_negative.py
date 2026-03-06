#!/usr/bin/env python3
"""Validate P11-01 mailbox capability contract with fail-closed negative cases."""

from __future__ import annotations

import argparse
import json
import re
from pathlib import Path
from typing import Any

PID_MAX = 1000


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Validate mailbox capability negative matrix and source contract."
    )
    parser.add_argument("--header", required=True, help="sched_mailbox_abi.h path")
    parser.add_argument("--source", required=True, help="sched_mailbox.c path")
    parser.add_argument("--out-report", required=True, help="Output report.json path")
    parser.add_argument(
        "--out-matrix", required=True, help="Output negative_matrix.json path"
    )
    return parser.parse_args()


def read_text(path: Path) -> str:
    return path.read_text(encoding="utf-8", errors="replace")


def parse_shift_define(text: str, name: str) -> int:
    pattern = re.compile(rf"#define\s+{re.escape(name)}\s+\(1u\s*<<\s*(\d+)\)")
    match = pattern.search(text)
    if not match:
        raise ValueError(f"missing_or_invalid_shift_define:{name}")
    return 1 << int(match.group(1))


def parse_u32_define(text: str, name: str) -> int:
    pattern = re.compile(rf"#define\s+{re.escape(name)}\s+([0-9]+)u")
    match = pattern.search(text)
    if not match:
        raise ValueError(f"missing_or_invalid_u32_define:{name}")
    return int(match.group(1))


def evaluate_case(
    flags: int,
    candidate_pid: int,
    budget_hint: int,
    *,
    flag_required: int,
    flag_sig_valid: int,
    flag_cap_present: int,
    flag_budget_ok: int,
    budget_max: int,
) -> str:
    # Backward-compatible default: no capability enforcement unless requested.
    if (flags & flag_required) == 0:
        if candidate_pid <= 0 or candidate_pid > PID_MAX:
            return "REJ_INVALID_PID"
        return "ACCEPT"

    if (flags & flag_sig_valid) == 0:
        return "REJ_BAD_SIG"

    if (flags & flag_cap_present) == 0:
        return "REJ_CAP_MISSING"

    if (flags & flag_budget_ok) == 0 or budget_hint > budget_max:
        return "REJ_BUDGET_EXCEEDED"

    if candidate_pid <= 0 or candidate_pid > PID_MAX:
        return "REJ_INVALID_PID"

    return "ACCEPT"


def write_json(path: Path, payload: dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def main() -> int:
    args = parse_args()
    header_path = Path(args.header)
    source_path = Path(args.source)
    report_path = Path(args.out_report)
    matrix_path = Path(args.out_matrix)

    report: dict[str, Any] = {
        "gate": "mailbox-capability-negative",
        "header": str(header_path),
        "source": str(source_path),
        "violations": [],
    }

    if not header_path.is_file():
        report["violations"].append(f"missing_header:{header_path}")
    if not source_path.is_file():
        report["violations"].append(f"missing_source:{source_path}")
    if report["violations"]:
        report["verdict"] = "FAIL"
        report["violations_count"] = len(report["violations"])
        write_json(report_path, report)
        write_json(matrix_path, {"cases": []})
        return 2

    header_text = read_text(header_path)
    source_text = read_text(source_path)

    required_header_symbols = (
        "AYKEN_SCHED_REJECT_BAD_SIG",
        "AYKEN_SCHED_REJECT_CAP_MISSING",
        "AYKEN_SCHED_REJECT_BUDGET_EXCEEDED",
        "AYKEN_SCHED_REJECT_INVALID_PID",
        "REJ_BAD_SIG",
        "REJ_CAP_MISSING",
        "REJ_BUDGET_EXCEEDED",
        "REJ_INVALID_PID",
        "AYKEN_SCHED_MB_FLAG_CAP_CHECK_REQUIRED",
        "AYKEN_SCHED_MB_FLAG_SIG_VALID",
        "AYKEN_SCHED_MB_FLAG_CAP_PRESENT",
        "AYKEN_SCHED_MB_FLAG_BUDGET_OK",
        "AYKEN_SCHED_MB_CAP_BUDGET_MAX",
    )
    for symbol in required_header_symbols:
        symbol_pattern = re.compile(
            rf"(?<![A-Za-z0-9_]){re.escape(symbol)}(?![A-Za-z0-9_])"
        )
        if not symbol_pattern.search(header_text):
            report["violations"].append(f"missing_header_symbol:{symbol}")

    required_source_snippets = (
        "sched_mailbox_validate_capability_envelope",
        "reject_reason = REJ_BAD_SIG;",
        "reject_reason = REJ_CAP_MISSING;",
        "reject_reason = REJ_BUDGET_EXCEEDED;",
        "reject_reason = REJ_INVALID_PID;",
        "sched_mailbox_validate_capability_envelope(mb, &reject_reason)",
    )
    for snippet in required_source_snippets:
        if snippet not in source_text:
            report["violations"].append(f"missing_source_snippet:{snippet}")

    try:
        flag_required = parse_shift_define(
            header_text, "AYKEN_SCHED_MB_FLAG_CAP_CHECK_REQUIRED"
        )
        flag_sig_valid = parse_shift_define(header_text, "AYKEN_SCHED_MB_FLAG_SIG_VALID")
        flag_cap_present = parse_shift_define(
            header_text, "AYKEN_SCHED_MB_FLAG_CAP_PRESENT"
        )
        flag_budget_ok = parse_shift_define(header_text, "AYKEN_SCHED_MB_FLAG_BUDGET_OK")
        budget_max = parse_u32_define(header_text, "AYKEN_SCHED_MB_CAP_BUDGET_MAX")
    except ValueError as exc:
        report["violations"].append(str(exc))
        report["verdict"] = "FAIL"
        report["violations_count"] = len(report["violations"])
        write_json(report_path, report)
        write_json(matrix_path, {"cases": []})
        return 2

    all_valid_flags = flag_required | flag_sig_valid | flag_cap_present | flag_budget_ok

    cases = [
        {
            "id": "bad_signature",
            "flags": flag_required | flag_cap_present | flag_budget_ok,
            "candidate_pid": 42,
            "budget_hint": 8,
            "expected": "REJ_BAD_SIG",
        },
        {
            "id": "capability_missing",
            "flags": flag_required | flag_sig_valid | flag_budget_ok,
            "candidate_pid": 42,
            "budget_hint": 8,
            "expected": "REJ_CAP_MISSING",
        },
        {
            "id": "budget_exceeded_by_flag",
            "flags": flag_required | flag_sig_valid | flag_cap_present,
            "candidate_pid": 42,
            "budget_hint": 8,
            "expected": "REJ_BUDGET_EXCEEDED",
        },
        {
            "id": "budget_exceeded_by_value",
            "flags": all_valid_flags,
            "candidate_pid": 42,
            "budget_hint": budget_max + 1,
            "expected": "REJ_BUDGET_EXCEEDED",
        },
        {
            "id": "invalid_pid_zero",
            "flags": all_valid_flags,
            "candidate_pid": 0,
            "budget_hint": 8,
            "expected": "REJ_INVALID_PID",
        },
        {
            "id": "invalid_pid_out_of_range",
            "flags": all_valid_flags,
            "candidate_pid": PID_MAX + 1,
            "budget_hint": 8,
            "expected": "REJ_INVALID_PID",
        },
    ]

    failed_case_count = 0
    for case in cases:
        actual = evaluate_case(
            flags=case["flags"],
            candidate_pid=case["candidate_pid"],
            budget_hint=case["budget_hint"],
            flag_required=flag_required,
            flag_sig_valid=flag_sig_valid,
            flag_cap_present=flag_cap_present,
            flag_budget_ok=flag_budget_ok,
            budget_max=budget_max,
        )
        case["actual"] = actual
        case["pass"] = actual == case["expected"]
        if not case["pass"]:
            failed_case_count += 1
            report["violations"].append(
                f"matrix_case_failed:{case['id']}:expected={case['expected']}:actual={actual}"
            )

    matrix_payload: dict[str, Any] = {
        "gate": "mailbox-capability-negative",
        "constants": {
            "flag_required": flag_required,
            "flag_sig_valid": flag_sig_valid,
            "flag_cap_present": flag_cap_present,
            "flag_budget_ok": flag_budget_ok,
            "budget_max": budget_max,
            "pid_max": PID_MAX,
        },
        "cases": cases,
    }
    write_json(matrix_path, matrix_payload)

    report["matrix_cases"] = len(cases)
    report["matrix_failed_cases"] = failed_case_count
    if report["violations"]:
        report["verdict"] = "FAIL"
        report["violations_count"] = len(report["violations"])
        write_json(report_path, report)
        return 2

    report["verdict"] = "PASS"
    report["violations"] = []
    report["violations_count"] = 0
    write_json(report_path, report)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
