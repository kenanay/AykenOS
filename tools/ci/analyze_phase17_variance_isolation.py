#!/usr/bin/env python3
"""Analyze bounded Phase-17 performance variance isolation measurements.

Author: Kenan AY
Attribution is tooling metadata only and has no runtime or acceptance authority.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import re
import statistics
from pathlib import Path
from typing import Any


GATE = "phase17-performance-variance-isolation"
SIGNIFICANT_PEAK_PERCENT = 3.0
DOMINANT_STAGE_RATIO = 0.60
STAGES = (
    "phase_boot_start_to_core_ready_ticks",
    "phase_core_ready_to_first_sched_activity_ticks",
    "phase_first_sched_activity_to_first_user_entry_ticks",
    "phase_first_user_entry_to_first_syscall_gate_entry_ticks",
    "phase_first_syscall_gate_entry_to_first_syscall_gate_return_ticks",
)
TERMINAL_COUNTERS = (
    "sw_count",
    "iret_count",
    "mark_sw_count",
    "mark_iret_count",
    "proof_done_seen",
    "qemu_timeout_hit",
)
CONTRACT_KEYS = (
    "contract_user_minimal_mode",
    "contract_bootstrap_policy",
    "contract_mb_selftest",
    "contract_deterministic_exit",
    "contract_ring3_entry_guard",
    "observed_user_minimal_mode",
    "observed_bootstrap_policy",
    "observed_mb_selftest",
    "observed_deterministic_exit",
    "observed_ring3_entry_guard",
)


def read_kv(path: Path) -> dict[str, str]:
    values: dict[str, str] = {}
    for line in path.read_text(encoding="utf-8").splitlines():
        if "=" not in line:
            continue
        key, value = line.split("=", 1)
        values[key.strip()] = value.strip()
    return values


def int_value(values: dict[str, str], key: str, path: Path) -> int:
    raw = values.get(key)
    if raw is None:
        raise ValueError(f"missing_metric:{path}:{key}")
    try:
        return int(raw)
    except ValueError as exc:
        raise ValueError(f"invalid_metric:{path}:{key}:{raw}") from exc


def median(values: list[int]) -> float:
    return float(statistics.median(values))


def file_sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def payload_sha256(payload: Any) -> str:
    normalized = json.dumps(payload, separators=(",", ":"), sort_keys=True)
    return hashlib.sha256(normalized.encode("utf-8")).hexdigest()


def metric_paths(group_path: Path) -> list[Path]:
    paths = sorted((group_path / "runs").glob("sample-*.metrics.txt"))
    if paths:
        return paths
    return sorted(group_path.glob("sample-*/preempt.metrics.txt"))


def load_group(label: str, group_path: Path) -> dict[str, Any]:
    paths = metric_paths(group_path)
    if len(paths) < 3:
        raise ValueError(f"insufficient_samples:{label}:expected_at_least=3:actual={len(paths)}")

    samples: dict[str, dict[str, int]] = {}
    contracts_by_sample: dict[str, dict[str, str]] = {}
    input_hashes: dict[str, str] = {}
    required = ("qemu_run_time_ms",) + STAGES + TERMINAL_COUNTERS
    for path in paths:
        sample_label = (
            path.stem.split(".")[0]
            if path.parent.name == "runs"
            else path.parent.name
        )
        values = read_kv(path)
        samples[sample_label] = {key: int_value(values, key, path) for key in required}
        contracts_by_sample[sample_label] = {
            key: values.get(key, "<missing>") for key in CONTRACT_KEYS
        }
        input_hashes[sample_label] = file_sha256(path)

    contract_signatures = {
        json.dumps(contract, separators=(",", ":"), sort_keys=True)
        for contract in contracts_by_sample.values()
    }
    if len(contract_signatures) != 1:
        raise ValueError(f"intra_group_contract_drift:{label}")
    contract_signature = contracts_by_sample[sorted(contracts_by_sample)[0]]
    expected_contract = {
        "contract_user_minimal_mode": "syscall-v2-runtime",
        "contract_bootstrap_policy": "1",
        "contract_mb_selftest": "0",
        "contract_deterministic_exit": "1",
        "contract_ring3_entry_guard": "1",
        "observed_user_minimal_mode": "syscall-v2-runtime",
        "observed_bootstrap_policy": "1",
        "observed_mb_selftest": "0",
        "observed_deterministic_exit": "1",
        "observed_ring3_entry_guard": "1",
    }
    if contract_signature != expected_contract:
        raise ValueError(f"unexpected_runtime_contract:{label}:{contract_signature}")

    qemu_values = {key: value["qemu_run_time_ms"] for key, value in samples.items()}
    peak_label = max(qemu_values, key=qemu_values.get)
    peer_labels = sorted(key for key in samples if key != peak_label)
    peer_elapsed_median = median([qemu_values[key] for key in peer_labels])
    peak_elapsed = qemu_values[peak_label]
    peak_over_peer_percent = (
        ((peak_elapsed - peer_elapsed_median) / peer_elapsed_median) * 100.0
        if peer_elapsed_median
        else 0.0
    )

    stage_analysis: dict[str, Any] = {}
    positive_deltas: dict[str, float] = {}
    for stage in STAGES:
        peer_stage_median = median([samples[key][stage] for key in peer_labels])
        peak_stage = samples[peak_label][stage]
        delta = peak_stage - peer_stage_median
        if delta > 0:
            positive_deltas[stage] = delta
        stage_analysis[stage] = {
            "peak_value": peak_stage,
            "peer_median": peer_stage_median,
            "delta": delta,
            "percent_over_peer_median": (
                (delta / peer_stage_median) * 100.0 if peer_stage_median else None
            ),
        }

    total_positive_delta = sum(positive_deltas.values())
    dominant_stage = max(positive_deltas, key=positive_deltas.get) if positive_deltas else None
    dominant_ratio = (
        positive_deltas[dominant_stage] / total_positive_delta
        if dominant_stage and total_positive_delta
        else 0.0
    )

    counter_invariants: dict[str, Any] = {}
    for counter in TERMINAL_COUNTERS:
        values = {key: samples[key][counter] for key in sorted(samples)}
        distinct = sorted(set(values.values()))
        counter_invariants[counter] = {
            "values_by_sample": values,
            "constant": len(distinct) == 1,
            "constant_value": distinct[0] if len(distinct) == 1 else None,
        }
    terminal_counts_constant = all(
        item["constant"] for item in counter_invariants.values()
    )

    significant_peak = peak_over_peer_percent > SIGNIFICANT_PEAK_PERCENT
    if (
        significant_peak
        and terminal_counts_constant
        and dominant_stage == "phase_boot_start_to_core_ready_ticks"
        and dominant_ratio >= DOMINANT_STAGE_RATIO
    ):
        classification = "pre_scheduler_stage_dominant_elapsed_outlier"
    elif significant_peak and terminal_counts_constant:
        classification = "post_core_or_distributed_elapsed_outlier"
    elif significant_peak:
        classification = "elapsed_outlier_with_terminal_count_drift"
    else:
        classification = "no_significant_elapsed_outlier_reproduced"

    return {
        "label": label,
        "source_dir": str(group_path),
        "input_metric_sha256": input_hashes,
        "sample_count": len(samples),
        "samples": samples,
        "runtime_contract": contract_signature,
        "peak_sample_label": peak_label,
        "peak_qemu_run_time_ms": peak_elapsed,
        "peer_qemu_run_time_ms_median": peer_elapsed_median,
        "peak_over_peer_percent": peak_over_peer_percent,
        "significant_peak_threshold_percent": SIGNIFICANT_PEAK_PERCENT,
        "significant_peak": significant_peak,
        "stage_analysis": stage_analysis,
        "dominant_positive_delta_stage": dominant_stage,
        "dominant_positive_delta_ratio": dominant_ratio,
        "terminal_counter_invariants": counter_invariants,
        "terminal_counts_constant": terminal_counts_constant,
        "classification": classification,
    }


def parse_group(value: str) -> tuple[str, Path]:
    if "=" not in value:
        raise ValueError(f"invalid_group_argument:{value}")
    label, path = value.split("=", 1)
    if not re.match(r"^[A-Za-z0-9_.-]+$", label):
        raise ValueError(f"invalid_group_label:{label}")
    return label, Path(path)


def write(path: Path, text: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text, encoding="utf-8")


def failure(reason: str, args: argparse.Namespace) -> int:
    payload = {
        "gate": GATE,
        "verdict": "FAIL",
        "authority_status": "diagnostic_integrity_failure",
        "closure_eligible_component": False,
        "violations": [reason],
        "violations_count": 1,
    }
    write(args.out, json.dumps(payload, indent=2, sort_keys=True) + "\n")
    write(args.violations_out, reason + "\n")
    write(args.observations_out, "")
    write(args.meta_out, f"authority_status=diagnostic_integrity_failure\nreason={reason}\n")
    print(f"{GATE}: FAIL ({reason})")
    return 2


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--group", action="append", required=True)
    parser.add_argument("--out", type=Path, required=True)
    parser.add_argument("--violations-out", type=Path, required=True)
    parser.add_argument("--observations-out", type=Path, required=True)
    parser.add_argument("--meta-out", type=Path, required=True)
    args = parser.parse_args()

    try:
        group_args = [parse_group(value) for value in args.group]
        labels = [label for label, _ in group_args]
        if len(set(labels)) != len(labels):
            raise ValueError("duplicate_group_label")
        groups = {label: load_group(label, path) for label, path in group_args}
    except (OSError, ValueError) as exc:
        return failure(str(exc), args)

    campaign_groups = [
        groups[label] for label in ("image-reuse", "rebuild-per-run") if label in groups
    ]
    if len(campaign_groups) != 2:
        return failure("missing_required_campaign_groups:image-reuse,rebuild-per-run", args)
    if reuse_contract := groups["image-reuse"].get("runtime_contract"):
        if reuse_contract != groups["rebuild-per-run"].get("runtime_contract"):
            return failure("cross_group_runtime_contract_mismatch", args)
    for counter in TERMINAL_COUNTERS:
        reuse_value = groups["image-reuse"]["terminal_counter_invariants"][counter]
        rebuild_value = groups["rebuild-per-run"]["terminal_counter_invariants"][counter]
        if (
            not reuse_value["constant"]
            or not rebuild_value["constant"]
            or reuse_value["constant_value"] != rebuild_value["constant_value"]
        ):
            return failure(f"cross_group_terminal_counter_mismatch:{counter}", args)

    pre_scheduler_groups = sorted(
        group["label"]
        for group in campaign_groups
        if group["classification"] == "pre_scheduler_stage_dominant_elapsed_outlier"
    )
    post_core_groups = sorted(
        group["label"]
        for group in campaign_groups
        if group["classification"] == "post_core_or_distributed_elapsed_outlier"
    )
    if pre_scheduler_groups:
        source_localization = "pre_scheduler_or_host_qemu_timing_candidate"
    elif post_core_groups:
        source_localization = "post_core_timer_or_scheduler_candidate"
    else:
        source_localization = "prior_outlier_not_reproduced_in_bounded_campaign"

    reuse = groups["image-reuse"]
    rebuild = groups["rebuild-per-run"]
    if rebuild["significant_peak"] and not reuse["significant_peak"]:
        cold_warm_comparison = "rebuild_sensitive_candidate"
    elif reuse["significant_peak"] and not rebuild["significant_peak"]:
        cold_warm_comparison = "image_reuse_or_host_scheduling_candidate"
    elif reuse["significant_peak"] and rebuild["significant_peak"]:
        cold_warm_comparison = "variance_persists_across_image_modes"
    else:
        cold_warm_comparison = "no_campaign_outlier_reproduced"

    fingerprint_input = {
        "groups": groups,
        "source_localization": source_localization,
        "cold_warm_comparison": cold_warm_comparison,
    }
    payload = {
        "gate": GATE,
        "verdict": "PASS",
        "authority_status": "diagnostic_only_no_acceptance_authority",
        "acceptance_status": "pr4_remote_locked_authority_still_required",
        "closure_eligible_component": False,
        "scope": "local_existing_harness_stage_localization_only",
        "collection_contract": {
            "runtime_behavior_change": False,
            "baseline_mutation": False,
            "threshold_mutation": False,
            "conditions": ["image-reuse", "rebuild-per-run"],
            "minimum_samples_per_condition": 3,
            "runtime_contract_parity_required": True,
            "terminal_counter_parity_required": True,
        },
        "groups": groups,
        "source_localization": source_localization,
        "cold_warm_comparison": cold_warm_comparison,
        "pre_scheduler_dominant_groups": pre_scheduler_groups,
        "post_core_dominant_groups": post_core_groups,
        "isolation_fingerprint_sha256": payload_sha256(fingerprint_input),
        "does_prove": [
            "bounded_measurement_groups_were_classified_from_existing_phase_markers",
            "diagnostic_result_does_not_override_pr4_readiness_or_remote_acceptance",
        ]
        + (
            ["measured_elapsed_outlier_is_dominated_by_pre_scheduler_stage_in_at_least_one_condition"]
            if pre_scheduler_groups
            else []
        ),
        "does_not_prove": [
            "host_scheduler_root_cause",
            "qemu_root_cause",
            "timer_or_irq_root_cause_elimination",
            "cold_warm_causality",
            "production_performance_acceptance",
            "baseline_renewal_authority",
            "remote_locked_baseline_acceptance",
            "phase17_closure",
        ],
        "violations": [],
        "violations_count": 0,
    }
    observations = [
        f"source_localization={source_localization}",
        f"cold_warm_comparison={cold_warm_comparison}",
        f"pre_scheduler_dominant_groups={','.join(pre_scheduler_groups) if pre_scheduler_groups else 'none'}",
        f"isolation_fingerprint_sha256={payload['isolation_fingerprint_sha256']}",
    ]
    for label in ("image-reuse", "rebuild-per-run"):
        group = groups[label]
        observations.extend(
            [
                f"{label}.classification={group['classification']}",
                f"{label}.peak_sample={group['peak_sample_label']}",
                f"{label}.peak_over_peer_percent={group['peak_over_peer_percent']:.6f}",
                f"{label}.dominant_stage={group['dominant_positive_delta_stage']}",
                f"{label}.terminal_counts_constant={str(group['terminal_counts_constant']).lower()}",
            ]
        )
    write(args.out, json.dumps(payload, indent=2, sort_keys=True) + "\n")
    write(args.violations_out, "")
    write(args.observations_out, "\n".join(observations) + "\n")
    write(
        args.meta_out,
        "\n".join(
            [
                "authority_status=diagnostic_only_no_acceptance_authority",
                "acceptance_status=pr4_remote_locked_authority_still_required",
                f"source_localization={source_localization}",
                f"cold_warm_comparison={cold_warm_comparison}",
                f"isolation_fingerprint_sha256={payload['isolation_fingerprint_sha256']}",
                "closure_eligible_component=false",
            ]
        )
        + "\n",
    )
    print(f"{GATE}: PASS ({source_localization}; {cold_warm_comparison})")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
