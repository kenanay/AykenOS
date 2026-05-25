#!/usr/bin/env python3
"""Validate bounded Phase-17 performance acceptance over an existing perf report.

Author: Kenan AY
Attribution is tooling metadata only and has no runtime authority.
"""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path


REQUIRED_METRICS = (
    "boot_time_ms",
    "context_switch_latency_ms_proxy",
    "syscall_latency_ms_proxy",
)
REQUIRED_DEFAULT_OFF_FLAGS = (
    "-DAYKEN_BCIB_PUBLIC_E2E_SELFTEST=0",
    "-DAYKEN_BCIB_WORKER_COMPLETION_SELFTEST=0",
    "-DAYKEN_EXECUTION_RACE_SELFTEST=0",
    "-DAYKEN_EXECUTION_MARKER_VALIDATION_ENABLE=0",
    "-DAYKEN_EXECUTION_MARKER_LIFECYCLE_SELFTEST=0",
    "-DAYKEN_PHASE17_MARKER_INJECTION_TEST=0",
    "-DAYKEN_MARKER_INJECT_INVALID_ORDER=0",
    "-DAYKEN_EXECUTION_MARKER_NEGATIVE_EXPECT_REJECT=0",
)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Validate Phase-17 performance readiness without widening baseline authority."
    )
    parser.add_argument("--mode", choices=("locked-authority", "local-readiness"), required=True)
    parser.add_argument("--performance-report", required=True)
    parser.add_argument("--stability-report")
    parser.add_argument("--baseline-file", required=True)
    parser.add_argument("--build-log")
    parser.add_argument("--expected-authority", required=True)
    parser.add_argument("--out", required=True)
    parser.add_argument("--violations-out", required=True)
    parser.add_argument("--meta-out", required=True)
    return parser.parse_args()


def load_json(path: Path, label: str, violations: list[str]) -> dict:
    if not path.is_file():
        violations.append(f"missing_{label}:{path}")
        return {}
    try:
        payload = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, ValueError):
        violations.append(f"invalid_{label}:{path}")
        return {}
    if not isinstance(payload, dict):
        violations.append(f"invalid_{label}_shape:{path}")
        return {}
    return payload


def require_equal(
    violations: list[str], label: str, actual: object, expected: object
) -> None:
    if actual != expected:
        violations.append(f"{label}:expected={expected}:actual={actual}")


def main() -> int:
    args = parse_args()
    report_path = Path(args.performance_report)
    stability_path = Path(args.stability_report) if args.stability_report else None
    baseline_path = Path(args.baseline_file)
    build_log_path = Path(args.build_log) if args.build_log else None
    violations: list[str] = []

    source = load_json(report_path, "performance_report", violations)
    baseline = load_json(baseline_path, "baseline_file", violations)
    source_meta = source.get("meta", {}) if isinstance(source.get("meta"), dict) else {}
    source_env = source.get("env", {}) if isinstance(source.get("env"), dict) else {}
    baseline_env = baseline.get("env", {}) if isinstance(baseline.get("env"), dict) else {}
    baseline_policy = (
        baseline.get("policy", {}) if isinstance(baseline.get("policy"), dict) else {}
    )
    marker_contract = (
        baseline_policy.get("marker_contract", {})
        if isinstance(baseline_policy.get("marker_contract"), dict)
        else {}
    )

    require_equal(violations, "source_gate", source.get("gate"), "performance")
    require_equal(violations, "source_verdict", source.get("verdict"), "PASS")
    require_equal(violations, "source_violations_count", source.get("violations_count"), 0)
    require_equal(
        violations,
        "measurement_contract",
        source.get("measurement_contract"),
        "deterministic_preempt_harness",
    )
    require_equal(
        violations,
        "baseline_measurement_contract",
        marker_contract.get("measurement_contract"),
        "deterministic_preempt_harness",
    )
    require_equal(
        violations,
        "baseline_measured_user_mode",
        marker_contract.get("preempt_user_minimal_mode"),
        "syscall-v2-runtime",
    )

    metric_container = source.get("results") if args.mode == "locked-authority" else source.get("metrics")
    if not isinstance(metric_container, dict):
        violations.append("missing_source_metrics")
    else:
        for metric in REQUIRED_METRICS:
            if metric not in metric_container:
                violations.append(f"missing_source_metric:{metric}")

    if args.mode == "locked-authority":
        require_equal(violations, "baseline_mode", source_meta.get("baseline_mode"), "constitutional")
        require_equal(violations, "env_mismatch_policy", source_meta.get("env_mismatch_policy"), "fail")
        require_equal(violations, "regression_policy", source_meta.get("regression_policy"), "fail")
        require_equal(
            violations,
            "source_authority",
            source_env.get("baseline_authority"),
            args.expected_authority,
        )
        require_equal(
            violations,
            "baseline_authority",
            baseline_policy.get("baseline_authority"),
            args.expected_authority,
        )
        require_equal(
            violations,
            "source_env_hash",
            source_env.get("env_hash"),
            baseline_env.get("env_hash"),
        )
        require_equal(
            violations,
            "source_ci_image_digest",
            source_env.get("ci_image_digest"),
            baseline_env.get("ci_image_digest"),
        )
        require_equal(violations, "baseline_diff", source.get("baseline_diff"), [])
        if build_log_path is None or not build_log_path.is_file():
            violations.append("missing_locked_authority_build_log")
        else:
            build_log = build_log_path.read_text(encoding="utf-8", errors="replace")
            for flag in REQUIRED_DEFAULT_OFF_FLAGS:
                if flag not in build_log:
                    violations.append(f"default_off_flag_not_observed:{flag}")
        authority_status = "locked_authority_pass" if not violations else "locked_authority_fail"
        does_prove = [
            "constitutional_locked_baseline_performance_report_passed",
            "deterministic_preempt_harness_measured_timer_preemption_hot_path",
            "phase17_validation_feature_flags_default_off_in_measured_build",
        ]
        closure_eligible_component = not violations
    else:
        current = source.get("current", {}) if isinstance(source.get("current"), dict) else {}
        require_equal(
            violations,
            "local_source_authority",
            current.get("baseline_authority"),
            args.expected_authority,
        )
        if not args.expected_authority.startswith("local-dev-"):
            violations.append("local_readiness_requires_local_authority")
        if stability_path is None:
            violations.append("missing_local_stability_report_argument")
        else:
            stability = load_json(stability_path, "stability_report", violations)
            require_equal(
                violations,
                "stability_gate",
                stability.get("gate"),
                "performance-stability",
            )
            require_equal(violations, "stability_verdict", stability.get("verdict"), "PASS")
            require_equal(
                violations,
                "stability_violations_count",
                stability.get("violations_count"),
                0,
            )
        authority_status = (
            "local_diagnostic_pass_remote_locked_acceptance_pending"
            if not violations
            else "local_diagnostic_fail"
        )
        does_prove = ["local_development_baseline_diagnostic_passed"]
        closure_eligible_component = False

    verdict = "PASS" if not violations else "FAIL"
    measured_scope = (
        "existing_locked_baseline_timer_preemption_hot_path_only"
        if args.mode == "locked-authority"
        else "existing_local_baseline_timer_preemption_hot_path_diagnostic_only"
    )
    source_bytes = report_path.read_bytes() if report_path.is_file() else b""
    stability_bytes = (
        stability_path.read_bytes()
        if stability_path is not None and stability_path.is_file()
        else b""
    )
    baseline_bytes = baseline_path.read_bytes() if baseline_path.is_file() else b""
    report = {
        "gate": "phase17-performance-acceptance",
        "verdict": verdict,
        "mode": args.mode,
        "authority_status": authority_status,
        "closure_eligible_component": closure_eligible_component,
        "scope": measured_scope,
        "expected_authority": args.expected_authority,
        "source_performance_report": str(report_path),
        "source_performance_report_sha256": hashlib.sha256(source_bytes).hexdigest(),
        "source_stability_report": str(stability_path) if stability_path else None,
        "source_stability_report_sha256": (
            hashlib.sha256(stability_bytes).hexdigest() if stability_path else None
        ),
        "baseline_file": str(baseline_path),
        "baseline_file_sha256": hashlib.sha256(baseline_bytes).hexdigest(),
        "build_log": str(build_log_path) if build_log_path else None,
        "does_prove": does_prove if verdict == "PASS" else [],
        "does_not_prove": [
            "validation_only_worker_completion_latency_acceptance",
            "validation_only_timeout_race_latency_acceptance",
            "general_bcib_interpreter_performance",
            "exhaustive_race_or_smp_performance",
            "phase17_closure_without_same_sha_remote_runtime_evidence",
        ],
        "violations_count": len(violations),
        "violations": violations,
    }

    out_path = Path(args.out)
    violations_path = Path(args.violations_out)
    meta_path = Path(args.meta_out)
    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    violations_path.write_text(
        "\n".join(violations) + ("\n" if violations else ""), encoding="utf-8"
    )
    meta_path.write_text(
        f"gate=phase17-performance-acceptance\n"
        f"mode={args.mode}\n"
        f"authority_status={authority_status}\n"
        f"closure_eligible_component={str(closure_eligible_component).lower()}\n"
        f"scope={measured_scope}\n"
        "does_not_prove=validation_only_worker_completion_latency,validation_only_timeout_race_latency,phase17_closure\n",
        encoding="utf-8",
    )
    print(f"phase17-performance-acceptance: {verdict} ({authority_status})")
    return 0 if verdict == "PASS" else 2


if __name__ == "__main__":
    raise SystemExit(main())
