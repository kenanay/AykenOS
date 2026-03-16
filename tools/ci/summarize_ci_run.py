#!/usr/bin/env python3
"""Summarize a CI evidence run and reduce results to kill-switch categories."""

from __future__ import annotations

import argparse
import json
from pathlib import Path
from typing import Any


ACCEPTABLE_VERDICTS = {"PASS", "WARN"}
SKIP_REASON_FIELDS = ("skip_reason", "reason")

KILL_SWITCHES: tuple[dict[str, Any], ...] = (
    {
        "kill_switch_id": "observability-control-plane",
        "title": "observability -> control plane",
        "category": "architectural",
        "severity": "kill-switch",
        "invariant": "observability != scheduling",
        "risk_class": "topology-feedback-drift",
        "primary_gate": "observability-routing-separation",
        "supporting_gates": (
            "proofd-observability-boundary",
            "diagnostics-consumer-non-authoritative-contract",
            "diagnostics-callsite-correlation",
        ),
        "authoritative_failure_meaning": (
            "observability artifacts have started steering routing, scheduling, "
            "or execution behavior"
        ),
    },
    {
        "kill_switch_id": "authority-election",
        "title": "authority election",
        "category": "architectural",
        "severity": "kill-switch",
        "invariant": "truth is computed, not elected",
        "risk_class": "truth-election-drift",
        "primary_gate": "convergence-non-election-boundary",
        "supporting_gates": (
            "graph-non-authoritative-contract",
            "cross-node-parity",
        ),
        "authoritative_failure_meaning": (
            "distributed agreement shape is being treated as truth selection"
        ),
    },
    {
        "kill_switch_id": "verification-artifact-integrity",
        "title": "verification artifact integrity",
        "category": "architectural",
        "severity": "kill-switch",
        "invariant": "artifacts = canonical interface",
        "risk_class": "artifact-truth-drift",
        "primary_gate": "proof-verdict-binding",
        "supporting_gates": (
            "proof-receipt",
            "proofd-service",
        ),
        "authoritative_failure_meaning": (
            "verification truth is no longer artifact-bound"
        ),
    },
    {
        "kill_switch_id": "verifier-authority-drift",
        "title": "verifier authority drift",
        "category": "architectural",
        "severity": "kill-switch",
        "invariant": "valid receipt != trusted verifier",
        "risk_class": "authority-drift",
        "primary_gate": "verifier-authority-resolution",
        "supporting_gates": (
            "verifier-reputation-prohibition",
            "observability-routing-separation",
            "cross-node-parity",
        ),
        "authoritative_failure_meaning": (
            "valid receipt semantics are being confused with trusted verifier authority"
        ),
    },
)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Summarize a CI evidence run and emit kill-switch reduction artifacts."
    )
    parser.add_argument("--run-dir", required=True, help="Evidence run directory.")
    parser.add_argument(
        "--require-kill-switch-completeness",
        action="store_true",
        help="Fail when expected kill-switch gates are not discovered in the run.",
    )
    return parser.parse_args()


def load_json(path: Path, default: Any) -> tuple[Any, str | None]:
    if not path.exists():
        return default, None
    try:
        with path.open("r", encoding="utf-8", errors="replace") as fh:
            return json.load(fh), None
    except Exception as exc:  # pragma: no cover
        return default, f"{type(exc).__name__}: {exc}"


def load_text(path: Path, default: str = "") -> str:
    if not path.exists():
        return default
    return path.read_text(encoding="utf-8", errors="replace").strip()


def make_target_name(gate_name: str) -> str:
    return f"ci-gate-{gate_name}"


def classify_gate_acceptance(gate: dict[str, Any]) -> tuple[str, str | None]:
    verdict = str(gate.get("verdict", "UNKNOWN"))
    if verdict in ACCEPTABLE_VERDICTS:
        return "PASS", None
    if verdict == "SKIP":
        for field in SKIP_REASON_FIELDS:
            value = str(gate.get(field, "") or "").strip()
            if value:
                return "PASS", None
        return "FAIL", "skip_requires_reason"
    return "FAIL", None


def gate_status_entry(gates: dict[str, dict[str, Any]], gate_name: str) -> dict[str, str]:
    gate = gates.get(gate_name)
    if gate is None:
        return {
            "gate": gate_name,
            "make_target": make_target_name(gate_name),
            "status": "NOT_EXECUTED",
            "discovery_state": "NOT_DISCOVERED",
            "execution_state": "NOT_EXECUTED",
        }

    verdict = str(gate.get("verdict", "UNKNOWN"))
    status, summary_violation = classify_gate_acceptance(gate)
    entry = {
        "gate": gate_name,
        "make_target": make_target_name(gate_name),
        "status": status,
        "verdict": verdict,
        "discovery_state": "DISCOVERED",
        "execution_state": "EXECUTED",
    }
    for field in SKIP_REASON_FIELDS:
        value = str(gate.get(field, "") or "").strip()
        if value:
            entry[field] = value
    if summary_violation:
        entry["summary_violation"] = summary_violation
    return entry


def evaluate_kill_switches(gates: dict[str, dict[str, Any]]) -> dict[str, Any]:
    kill_switches: list[dict[str, Any]] = []
    status_counts = {
        "PASS": 0,
        "FAIL": 0,
        "SUPPORT_ONLY": 0,
        "NOT_EVALUATED": 0,
    }

    for definition in KILL_SWITCHES:
        primary = gate_status_entry(gates, definition["primary_gate"])
        supporting = [
            gate_status_entry(gates, gate_name)
            for gate_name in definition["supporting_gates"]
        ]
        discovered = [
            entry for entry in [primary, *supporting] if entry["discovery_state"] == "DISCOVERED"
        ]
        failed = [entry for entry in [primary, *supporting] if entry["status"] == "FAIL"]
        primary_failed = primary["status"] == "FAIL"
        supporting_failed = [entry for entry in supporting if entry["status"] == "FAIL"]

        if primary_failed:
            status = "FAIL"
            failure_trigger = "PRIMARY_GATE"
        elif supporting_failed:
            status = "FAIL"
            failure_trigger = "SUPPORTING_GATE"
        elif primary["status"] == "PASS":
            status = "PASS"
            failure_trigger = "PRIMARY_GATE"
        elif discovered:
            status = "SUPPORT_ONLY"
            failure_trigger = "SUPPORTING_EVIDENCE_ONLY"
        else:
            status = "NOT_EVALUATED"
            failure_trigger = "NO_EXECUTED_GATES"

        status_counts[status] += 1
        kill_switches.append(
            {
                "kill_switch_id": definition["kill_switch_id"],
                "title": definition["title"],
                "category": definition["category"],
                "severity": definition["severity"],
                "invariant": definition["invariant"],
                "risk_class": definition["risk_class"],
                "status": status,
                "failure_trigger": failure_trigger,
                "primary_gate": primary,
                "supporting_gates": supporting,
                "authoritative_failure_meaning": definition["authoritative_failure_meaning"],
                "discovered_gate_count": len(discovered),
                "failed_gate_count": len(failed),
                "failed_gates": failed,
            }
        )

    if status_counts["FAIL"] > 0:
        overall_status = "FAIL"
    elif status_counts["PASS"] == len(KILL_SWITCHES):
        overall_status = "PASS"
    elif status_counts["PASS"] > 0 or status_counts["SUPPORT_ONLY"] > 0:
        overall_status = "PARTIAL"
    else:
        overall_status = "NOT_EVALUATED"

    return {
        "overall_status": overall_status,
        "status_counts": status_counts,
        "kill_switches": kill_switches,
    }


def evaluate_kill_switch_coverage(gates: dict[str, dict[str, Any]]) -> dict[str, Any]:
    expected_gates = sorted(
        {
            gate_name
            for definition in KILL_SWITCHES
            for gate_name in (definition["primary_gate"], *definition["supporting_gates"])
        }
    )
    discovered_gates = sorted(gate_name for gate_name in gates if gate_name in expected_gates)
    missing_gates = sorted(set(expected_gates) - set(discovered_gates))
    coverage_status = "COMPLETE" if not missing_gates else "INCOMPLETE"
    return {
        "coverage_status": coverage_status,
        "expected_gates": expected_gates,
        "expected_gate_count": len(expected_gates),
        "discovered_gates": discovered_gates,
        "discovered_gate_count": len(discovered_gates),
        "missing_gates": missing_gates,
    }


def build_summary(run_dir: Path) -> dict[str, Any]:
    run_meta, run_meta_err = load_json(run_dir / "meta" / "run.json", {})
    git_sha = load_text(run_dir / "meta" / "git.txt", "UNKNOWN")
    gates_dir = run_dir / "gates"
    parse_errors: list[dict[str, str]] = []
    gates: dict[str, dict[str, Any]] = {}

    if run_meta_err:
        parse_errors.append({"path": str(run_dir / "meta" / "run.json"), "error": run_meta_err})

    for report_path in sorted(gates_dir.glob("*/report.json")):
        report, report_err = load_json(report_path, {})
        gate_name = str((report or {}).get("gate") or report_path.parent.name)
        verdict = str((report or {}).get("verdict", "UNKNOWN"))

        if report_err:
            gates[gate_name] = {
                "verdict": "FAIL",
                "report_path": str(report_path),
                "parse_error": report_err,
            }
            parse_errors.append({"path": str(report_path), "error": report_err})
            continue

        gate_entry: dict[str, Any] = {"verdict": verdict}
        if "violations_count" in report:
            try:
                gate_entry["violations_count"] = int(report.get("violations_count", 0))
            except (TypeError, ValueError):
                gate_entry["violations_count"] = 0
        for field in SKIP_REASON_FIELDS:
            value = str(report.get(field, "") or "").strip()
            if value:
                gate_entry[field] = value
        gates[gate_name] = gate_entry

    overall_verdict = "PASS" if gates else "FAIL"
    for gate in gates.values():
        status, _ = classify_gate_acceptance(gate)
        if status != "PASS":
            overall_verdict = "FAIL"
            break
    if parse_errors:
        overall_verdict = "FAIL"

    runtime_gate = gates.get("syscall-v2-runtime")
    runtime_verdict = str((runtime_gate or {}).get("verdict", "MISSING"))
    if runtime_gate is None:
        freeze_status = "pending_runtime_verification"
        kernel_runtime_verified = None
    elif runtime_verdict == "PASS":
        freeze_status = "kernel_runtime_verified"
        kernel_runtime_verified = True
    else:
        freeze_status = "kernel_runtime_unverified"
        kernel_runtime_verified = False

    return {
        "run_id": run_meta.get("run_id", run_dir.name),
        "time_utc": run_meta.get("time_utc", ""),
        "git_sha": git_sha,
        "verdict": overall_verdict,
        "freeze_status": freeze_status,
        "kernel_runtime_verified": kernel_runtime_verified,
        "gates_discovered": len(gates),
        "parse_errors_count": len(parse_errors),
        "parse_errors": parse_errors,
        "gates": gates,
    }


def write_json(path: Path, payload: dict[str, Any]) -> None:
    with path.open("w", encoding="utf-8") as fh:
        json.dump(payload, fh, indent=2, sort_keys=True)
        fh.write("\n")


def write_kill_switch_text(path: Path, kill_switch_summary: dict[str, Any]) -> None:
    lines: list[str] = []
    coverage = kill_switch_summary["coverage"]
    lines.append(f'coverage: {coverage["coverage_status"]}')
    lines.append(
        "expected_gates: "
        f'{coverage["discovered_gate_count"]}/{coverage["expected_gate_count"]} discovered'
    )
    if coverage["missing_gates"]:
        missing = ", ".join(make_target_name(gate) for gate in coverage["missing_gates"])
        lines.append(f"missing: {missing}")
    lines.append("")
    for item in kill_switch_summary["kill_switches"]:
        if item["status"] == "NOT_EVALUATED":
            continue
        lines.append(f'{item["status"]}: {item["title"]}')
        lines.append(f'trigger: {item["failure_trigger"]}')
        primary = item["primary_gate"]
        lines.append(f'primary: {primary["make_target"]} ({primary["status"]})')
        support_bits = [
            f'{entry["make_target"]} ({entry["status"]})'
            for entry in item["supporting_gates"]
        ]
        if support_bits:
            lines.append("support: " + ", ".join(support_bits))
        lines.append(f'meaning: {item["authoritative_failure_meaning"]}')
        lines.append("")
    path.write_text("\n".join(lines).rstrip() + ("\n" if lines else ""), encoding="utf-8")


def main() -> int:
    args = parse_args()
    run_dir = Path(args.run_dir).resolve()
    reports_dir = run_dir / "reports"
    reports_dir.mkdir(parents=True, exist_ok=True)

    summary = build_summary(run_dir)
    summary_path = reports_dir / "summary.json"
    write_json(summary_path, summary)

    kill_switch_summary = evaluate_kill_switches(summary["gates"])
    coverage = evaluate_kill_switch_coverage(summary["gates"])
    kill_switch_payload = {
        "run_id": summary["run_id"],
        "time_utc": summary["time_utc"],
        "git_sha": summary["git_sha"],
        "summary_path": str(summary_path),
        "completeness_required": bool(args.require_kill_switch_completeness),
        "overall_status": kill_switch_summary["overall_status"],
        "status_counts": kill_switch_summary["status_counts"],
        "coverage": coverage,
        "kill_switches": kill_switch_summary["kill_switches"],
    }
    write_json(reports_dir / "kill_switch_summary.json", kill_switch_payload)
    write_kill_switch_text(reports_dir / "kill_switch_summary.txt", kill_switch_payload)

    if summary["verdict"] != "PASS":
        return 2
    if args.require_kill_switch_completeness and coverage["missing_gates"]:
        return 2
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
