#!/usr/bin/env python3
"""Classify Phase-17 local performance variance from existing evidence only.

Author: Kenan AY
Attribution is tooling metadata only and has no runtime or acceptance authority.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import statistics
from pathlib import Path
from typing import Any


GATE = "phase17-performance-variance-diagnostic"
EXPECTED_MEASUREMENT_CONTRACT = "deterministic_preempt_harness"


def read_json(path: Path) -> dict[str, Any]:
    payload = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(payload, dict):
        raise ValueError(f"invalid_json_object:{path}")
    return payload


def sha256_file(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def sha256_payload(payload: dict[str, Any]) -> str:
    normalized = json.dumps(payload, sort_keys=True, separators=(",", ":"))
    return hashlib.sha256(normalized.encode("utf-8")).hexdigest()


def require_source(
    performance: dict[str, Any],
    stability: dict[str, Any],
    label: str,
) -> list[str]:
    violations: list[str] = []
    if performance.get("gate") != "performance":
        violations.append(f"{label}:unexpected_performance_gate")
    if performance.get("verdict") != "PASS":
        violations.append(f"{label}:performance_median_gate_not_pass")
    if performance.get("measurement_contract") != EXPECTED_MEASUREMENT_CONTRACT:
        violations.append(f"{label}:unexpected_measurement_contract")
    if stability.get("gate") != "performance-stability":
        violations.append(f"{label}:unexpected_stability_gate")
    if stability.get("measurement_contract") != EXPECTED_MEASUREMENT_CONTRACT:
        violations.append(f"{label}:stability_measurement_contract_mismatch")
    if not isinstance(stability.get("metrics"), dict):
        violations.append(f"{label}:missing_stability_metrics")
    return violations


def metric_diagnostics(stability: dict[str, Any]) -> dict[str, Any]:
    output: dict[str, Any] = {}
    for metric_name, metric in sorted(stability.get("metrics", {}).items()):
        constraints = metric.get("constraints", {})
        breached_fail_constraints = []
        for constraint_name, constraint in sorted(constraints.items()):
            if constraint.get("severity") == "fail" and constraint.get("breached") is True:
                breached_fail_constraints.append(
                    {
                        "constraint": constraint_name,
                        "actual": constraint.get("actual"),
                        "max": constraint.get("max"),
                    }
                )
        candidates = metric.get("current_outlier_candidates", [])
        labels = sorted(
            {
                candidate.get("sample_label")
                for candidate in candidates
                if isinstance(candidate, dict) and candidate.get("sample_label")
            }
        )
        output[metric_name] = {
            "median": metric.get("current_median"),
            "range_percent_of_median": metric.get("current_range_percent_of_median"),
            "mad_percent_of_median": metric.get("current_mad_percent_of_median"),
            "outlier_candidate_labels": labels,
            "outlier_candidates": candidates,
            "breached_fail_constraints": breached_fail_constraints,
        }
    return output


def classify(stability: dict[str, Any], metrics: dict[str, Any]) -> tuple[str, list[str], list[str]]:
    breached_metrics = sorted(
        name for name, metric in metrics.items() if metric["breached_fail_constraints"]
    )
    label_sets = [
        set(metrics[name]["outlier_candidate_labels"])
        for name in breached_metrics
        if metrics[name]["outlier_candidate_labels"]
    ]
    shared_labels = sorted(set.intersection(*label_sets)) if label_sets and len(label_sets) == len(breached_metrics) else []

    if stability.get("verdict") == "FAIL" and len(breached_metrics) >= 2 and shared_labels:
        status = "synchronized_sample_outlier_observed"
    elif stability.get("verdict") == "FAIL":
        status = "stability_failure_without_shared_outlier"
    elif stability.get("verdict") == "PASS":
        status = "stable_reference_observed"
    else:
        status = "stability_risk_observed"
    return status, breached_metrics, shared_labels


def read_metric_file(path: Path) -> dict[str, str]:
    values: dict[str, str] = {}
    for line in path.read_text(encoding="utf-8").splitlines():
        if "=" not in line:
            continue
        key, value = line.split("=", 1)
        values[key.strip()] = value.strip()
    return values


def raw_runtime_diagnostics(
    performance: dict[str, Any],
    shared_labels: list[str],
) -> dict[str, Any]:
    sample_dirs = performance.get("sampling", {}).get("usable_sample_dirs", [])
    if not isinstance(sample_dirs, list):
        return {"available": False, "reason": "missing_usable_sample_dirs"}

    counter_keys = [
        "sw_count",
        "iret_count",
        "mark_sw_count",
        "mark_iret_count",
        "qemu_timeout_hit",
        "proof_done_seen",
    ]
    observation_keys = [
        "qemu_run_time_ms",
        "debug_bytes",
        "serial_bytes",
        "ab_len",
        "ab_alt_count",
    ]
    sample_rows: dict[str, dict[str, Any]] = {}
    for sample_dir in sample_dirs:
        metric_file = Path(str(sample_dir)) / "preempt.metrics.txt"
        if not metric_file.is_file():
            return {"available": False, "reason": f"missing_raw_metrics:{metric_file}"}
        raw = read_metric_file(metric_file)
        label = Path(str(sample_dir)).name
        row: dict[str, Any] = {}
        for key in counter_keys + observation_keys:
            raw_value = raw.get(key)
            try:
                row[key] = int(raw_value) if raw_value is not None else None
            except ValueError:
                row[key] = raw_value
        sample_rows[label] = row

    invariant_counters: dict[str, Any] = {}
    for key in counter_keys:
        values = {label: row[key] for label, row in sorted(sample_rows.items())}
        unique_values = sorted(set(values.values()), key=str)
        invariant_counters[key] = {
            "values_by_sample": values,
            "constant": len(unique_values) == 1,
            "constant_value": unique_values[0] if len(unique_values) == 1 else None,
        }

    elapsed_comparison: dict[str, Any] = {}
    for label in shared_labels:
        if label not in sample_rows:
            continue
        observed = sample_rows[label].get("qemu_run_time_ms")
        reference_values = [
            row.get("qemu_run_time_ms")
            for row_label, row in sample_rows.items()
            if row_label != label and isinstance(row.get("qemu_run_time_ms"), int)
        ]
        if not isinstance(observed, int) or not reference_values:
            continue
        reference_median = statistics.median(reference_values)
        elapsed_comparison[label] = {
            "qemu_run_time_ms": observed,
            "non_outlier_qemu_run_time_ms_median": reference_median,
            "percent_over_non_outlier_median": (
                (observed - reference_median) / reference_median * 100.0
            ),
        }

    count_invariance = all(item["constant"] for item in invariant_counters.values())
    elapsed_growth = any(
        comparison["percent_over_non_outlier_median"] > 0
        for comparison in elapsed_comparison.values()
    )
    refinement = (
        "observed_terminal_counts_constant_while_elapsed_runtime_increased"
        if count_invariance and elapsed_growth
        else "raw_metric_observation_recorded"
    )
    return {
        "available": True,
        "classification_refinement": refinement,
        "invariant_counters": invariant_counters,
        "elapsed_comparison_for_shared_outliers": elapsed_comparison,
        "observations_by_sample": {
            label: {key: row[key] for key in observation_keys}
            for label, row in sorted(sample_rows.items())
        },
    }


def write_text(path: Path, content: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(content, encoding="utf-8")


def emit_failure(
    reason: str,
    out: Path,
    violations_out: Path,
    meta_out: Path,
) -> int:
    payload = {
        "gate": GATE,
        "verdict": "FAIL",
        "authority_status": "diagnostic_integrity_failure",
        "closure_eligible_component": False,
        "violations": [reason],
        "violations_count": 1,
    }
    write_text(out, json.dumps(payload, indent=2, sort_keys=True) + "\n")
    write_text(violations_out, reason + "\n")
    write_text(meta_out, f"authority_status=diagnostic_integrity_failure\nreason={reason}\n")
    print(f"{GATE}: FAIL ({reason})")
    return 2


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--performance-report", required=True, type=Path)
    parser.add_argument("--stability-report", required=True, type=Path)
    parser.add_argument("--reference-performance-report", type=Path)
    parser.add_argument("--reference-stability-report", type=Path)
    parser.add_argument("--out", required=True, type=Path)
    parser.add_argument("--violations-out", required=True, type=Path)
    parser.add_argument("--observations-out", required=True, type=Path)
    parser.add_argument("--meta-out", required=True, type=Path)
    args = parser.parse_args()

    if bool(args.reference_performance_report) != bool(args.reference_stability_report):
        return emit_failure(
            "reference_reports_must_be_supplied_together",
            args.out,
            args.violations_out,
            args.meta_out,
        )

    required_paths = [args.performance_report, args.stability_report]
    if args.reference_performance_report:
        required_paths.extend([args.reference_performance_report, args.reference_stability_report])
    missing = [str(path) for path in required_paths if not path.is_file()]
    if missing:
        return emit_failure(
            f"missing_source_report:{','.join(missing)}",
            args.out,
            args.violations_out,
            args.meta_out,
        )

    try:
        performance = read_json(args.performance_report)
        stability = read_json(args.stability_report)
        reference_performance = (
            read_json(args.reference_performance_report)
            if args.reference_performance_report
            else None
        )
        reference_stability = (
            read_json(args.reference_stability_report)
            if args.reference_stability_report
            else None
        )
    except (OSError, ValueError, json.JSONDecodeError) as exc:
        return emit_failure(
            f"invalid_source_report:{exc}",
            args.out,
            args.violations_out,
            args.meta_out,
        )

    violations = require_source(performance, stability, "observed")
    if reference_performance is not None and reference_stability is not None:
        violations.extend(require_source(reference_performance, reference_stability, "reference"))
    if violations:
        return emit_failure(
            ";".join(violations),
            args.out,
            args.violations_out,
            args.meta_out,
        )

    metrics = metric_diagnostics(stability)
    classification, breached_metrics, shared_labels = classify(stability, metrics)
    raw_runtime = raw_runtime_diagnostics(performance, shared_labels)
    reference: dict[str, Any] | None = None
    comparison_status = "reference_not_supplied"
    if reference_stability is not None:
        reference_metrics = metric_diagnostics(reference_stability)
        reference_classification, reference_breached_metrics, reference_shared_labels = classify(
            reference_stability, reference_metrics
        )
        if reference_stability.get("verdict") == "PASS" and stability.get("verdict") == "FAIL":
            comparison_status = "repeat_run_divergence_observed"
        else:
            comparison_status = "comparison_recorded"
        reference = {
            "performance_report": str(args.reference_performance_report),
            "performance_report_sha256": sha256_file(args.reference_performance_report),
            "stability_report": str(args.reference_stability_report),
            "stability_report_sha256": sha256_file(args.reference_stability_report),
            "stability_verdict": reference_stability.get("verdict"),
            "classification": reference_classification,
            "breached_metrics": reference_breached_metrics,
            "shared_outlier_sample_labels": reference_shared_labels,
        }

    fingerprint_input = {
        "measurement_contract": performance.get("measurement_contract"),
        "stability_verdict": stability.get("verdict"),
        "classification": classification,
        "breached_metrics": breached_metrics,
        "shared_outlier_sample_labels": shared_labels,
        "metrics": metrics,
        "raw_runtime_diagnostics": raw_runtime,
        "comparison_status": comparison_status,
        "reference": reference,
    }
    acceptance_status = (
        "blocked_by_source_stability_failure"
        if stability.get("verdict") != "PASS"
        else "remote_locked_authority_still_required"
    )
    payload = {
        "gate": GATE,
        "verdict": "PASS",
        "authority_status": "diagnostic_only_upstream_stability_verdict_preserved",
        "acceptance_status": acceptance_status,
        "closure_eligible_component": False,
        "scope": "existing_local_timer_preemption_evidence_analysis_only",
        "measurement_contract": performance.get("measurement_contract"),
        "observed_source": {
            "performance_report": str(args.performance_report),
            "performance_report_sha256": sha256_file(args.performance_report),
            "stability_report": str(args.stability_report),
            "stability_report_sha256": sha256_file(args.stability_report),
            "performance_verdict": performance.get("verdict"),
            "stability_verdict": stability.get("verdict"),
        },
        "reference_source": reference,
        "comparison_status": comparison_status,
        "classification": classification,
        "breached_metrics": breached_metrics,
        "shared_outlier_sample_labels": shared_labels,
        "metric_diagnostics": metrics,
        "raw_runtime_diagnostics": raw_runtime,
        "variance_fingerprint_sha256": sha256_payload(fingerprint_input),
        "candidate_hypotheses": [
            {
                "hypothesis": "host_or_qemu_scheduling_jitter",
                "status": "requires_isolation_measurement",
            },
            {
                "hypothesis": "timer_or_irq_ordering_amplification",
                "status": "requires_isolation_measurement",
            },
            {
                "hypothesis": "cold_warm_or_cache_path_divergence",
                "status": "requires_isolation_measurement",
            },
        ],
        "does_prove": [
            "source_stability_verdict_is_preserved_without_acceptance_override",
            "variance_classification_is_derived_from_existing_local_evidence",
            "variance_fingerprint_is_reproducible_for_identical_input_reports",
        ]
        + (
            ["observed_terminal_counts_remained_constant_during_shared_elapsed_outlier"]
            if raw_runtime.get("classification_refinement")
            == "observed_terminal_counts_constant_while_elapsed_runtime_increased"
            else []
        ),
        "does_not_prove": [
            "kernel_root_cause",
            "scheduler_or_irq_nondeterminism",
            "production_performance_acceptance",
            "baseline_renewal_authority",
            "remote_locked_baseline_acceptance",
            "phase17_closure",
        ],
        "violations": [],
        "violations_count": 0,
    }
    observations = [
        f"classification={classification}",
        f"source_stability_verdict={stability.get('verdict')}",
        f"comparison_status={comparison_status}",
        f"breached_metrics={','.join(breached_metrics) if breached_metrics else 'none'}",
        f"shared_outlier_sample_labels={','.join(shared_labels) if shared_labels else 'none'}",
        f"raw_runtime_refinement={raw_runtime.get('classification_refinement', 'unavailable')}",
        f"acceptance_status={acceptance_status}",
        f"variance_fingerprint_sha256={payload['variance_fingerprint_sha256']}",
    ]
    write_text(args.out, json.dumps(payload, indent=2, sort_keys=True) + "\n")
    write_text(args.violations_out, "")
    write_text(args.observations_out, "\n".join(observations) + "\n")
    write_text(
        args.meta_out,
        "\n".join(
            [
                "authority_status=diagnostic_only_upstream_stability_verdict_preserved",
                f"acceptance_status={acceptance_status}",
                f"classification={classification}",
                f"variance_fingerprint_sha256={payload['variance_fingerprint_sha256']}",
                "closure_eligible_component=false",
            ]
        )
        + "\n",
    )
    print(f"{GATE}: PASS ({classification}; {acceptance_status})")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
