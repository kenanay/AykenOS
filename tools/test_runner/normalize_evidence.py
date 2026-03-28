#!/usr/bin/env python3
"""Normalize AykenOS external test evidence into a stable validator input."""

from __future__ import annotations

import argparse
import json
from pathlib import Path


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Normalize scenario evidence.")
    parser.add_argument("--scenario", required=True, help="Scenario JSON path.")
    parser.add_argument("--run-report", required=True, help="run_scenario report path.")
    parser.add_argument("--out", required=True, help="Output normalized report path.")
    return parser.parse_args()


def read_json(path: Path) -> dict:
    with path.open("r", encoding="utf-8") as handle:
        return json.load(handle)


def write_json(path: Path, payload: object) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("w", encoding="utf-8") as handle:
        json.dump(payload, handle, indent=2, sort_keys=True)
        handle.write("\n")


def fmt_hex(value: int | None) -> str | None:
    if value is None:
        return None
    return f"0x{int(value):016X}"


def main() -> int:
    args = parse_args()
    scenario = read_json(Path(args.scenario))
    run_report = read_json(Path(args.run_report))

    gate_report_path = Path(run_report.get("gate_report", ""))
    if not gate_report_path.is_file():
        raise SystemExit(f"missing gate report: {gate_report_path}")

    gate_report = read_json(gate_report_path)
    runtime_rule = gate_report.get("runtime_rule_report", {})
    source_guard = gate_report.get("source_guard_report", {})
    witness = runtime_rule.get("selected_pre_dispatch_witness") or {}
    probe = runtime_rule.get("selected_post_cr3_probe") or {}
    user_marker = runtime_rule.get("selected_user_marker") or {}

    normalized = {
        "schema_version": "1.0",
        "scenario_id": scenario.get("scenario_id"),
        "domain": scenario.get("domain"),
        "surface": scenario.get("surface"),
        "goal": scenario.get("goal"),
        "gate": gate_report.get("gate"),
        "gate_verdict": gate_report.get("verdict"),
        "runtime_rule_verdict": runtime_rule.get("verdict"),
        "source_guard_verdict": source_guard.get("verdict"),
        "selected_root": fmt_hex(witness.get("root")),
        "selected_witness_qword": fmt_hex(witness.get("lo")),
        "selected_witness_line": witness.get("line"),
        "selected_probe_qword": fmt_hex(probe.get("q")),
        "selected_probe_rip": fmt_hex(probe.get("rip")),
        "selected_probe_cr3": fmt_hex(probe.get("cr3")),
        "selected_probe_line": probe.get("line"),
        "user_code_reached": user_marker is not None and bool(user_marker),
        "selected_user_marker_line": user_marker.get("line") if user_marker else None,
        "boot_audit_exit_code": gate_report.get("boot_audit_exit_code"),
        "authoritative_chain": [
            "P10_TEXT_FRAME_WITNESS",
            "P10_POST_CR3_TEXT_PROBE",
            "P10_RING3_USER_CODE",
        ],
        "observed_counts": runtime_rule.get("observed_counts", {}),
        "runtime_rule_violations": runtime_rule.get("violations", []),
        "source_guard_violations": source_guard.get("violations", []),
        "evidence_files": gate_report.get("evidence_files", {}),
    }
    write_json(Path(args.out), normalized)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
