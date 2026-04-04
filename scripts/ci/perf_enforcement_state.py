#!/usr/bin/env python3
import argparse
import json
import os
from pathlib import Path


METRIC_FLAGS = {
    "entry_latency_ticks": "PERF_ENFORCE_ENTRY",
    "syscall_gate_return_latency_ticks": "PERF_ENFORCE_RETURN",
    "syscall_latency_ticks_pure": "PERF_ENFORCE_PURE",
}


def load_json(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def load_optional_json(path: Path) -> dict | None:
    if not path.exists():
        return None
    return load_json(path)


def env_flag(name: str, default: str = "0") -> bool:
    return os.environ.get(name, default) == "1"


def env_int(name: str, default: int) -> int:
    raw = os.environ.get(name, str(default))
    return int(raw)


def env_float(name: str, default: float) -> float:
    raw = os.environ.get(name, str(default))
    return float(raw)


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Evaluate split-metric enforcement state in default-off/shadow mode."
    )
    parser.add_argument("--results-json", required=True)
    parser.add_argument("--env-json", required=True)
    parser.add_argument("--policy-json", required=True)
    parser.add_argument("--policy-verification-json", required=True)
    parser.add_argument("--state-path", required=True)
    parser.add_argument("--output-json", required=True)
    args = parser.parse_args()

    results_path = Path(args.results_json)
    env_path = Path(args.env_json)
    policy_path = Path(args.policy_json)
    policy_verification_path = Path(args.policy_verification_json)
    state_path = Path(args.state_path)
    output_path = Path(args.output_json)

    results = load_json(results_path)
    env_payload = load_json(env_path)
    previous_state = load_optional_json(state_path) or {}
    policy_payload = load_optional_json(policy_path)
    policy_verification = load_optional_json(policy_verification_path) or {}

    shadow_requested = env_flag("PERF_SPLIT_METRICS_SHADOW", "0")
    enforcement_requested = env_flag("PERF_SPLIT_METRICS_ENFORCEMENT", "0")
    variance_guard = env_float("PERF_VARIANCE_GUARD", 0.10)
    consecutive_limit = env_int("PERF_CONSECUTIVE_VIOLATION_LIMIT", 2)
    multi_metric_limit = env_int("PERF_GLOBAL_DISABLE_METRIC_VIOLATION_LIMIT", 2)

    authority = env_payload.get("baseline_authority")
    env_hash = env_payload.get("env_hash")

    policy_present = policy_payload is not None
    policy_authority = None
    if policy_payload is not None:
        policy_authority = policy_payload.get("source", {}).get("authority")
    policy_trusted = bool(policy_verification.get("trusted", False))
    policy_trust_reason = str(policy_verification.get("reason", "policy_missing"))

    global_disabled_reason = ""
    if previous_state.get("authority") and previous_state.get("authority") != authority:
        global_disabled_reason = "authority_changed"
    elif previous_state.get("env_hash") and previous_state.get("env_hash") != env_hash:
        global_disabled_reason = "env_hash_changed"
    elif policy_present and not policy_trusted:
        global_disabled_reason = policy_trust_reason
    elif policy_present and policy_authority and policy_authority != authority:
        global_disabled_reason = "policy_authority_mismatch"

    evaluate_requested = shadow_requested or enforcement_requested

    shadow_violation_count = 0
    blocking_violation_count = 0
    multi_metric_triggered = False
    metrics_out: dict[str, dict] = {}

    for metric_name, flag_name in METRIC_FLAGS.items():
        metric_result = results.get(metric_name, {})
        metric_policy = (
            (policy_payload or {}).get("recommendations", {}).get(metric_name)
            if policy_payload is not None
            else None
        )
        previous_metric = (previous_state.get("metrics") or {}).get(metric_name, {})

        available = bool(metric_result.get("available", False))
        value = float(metric_result.get("ticks", 0.0))
        requested = env_flag(flag_name, "0")
        policy_status = None
        threshold = None
        variance_ratio = None
        notes: list[str] = []

        if metric_policy is not None:
            policy_status = metric_policy.get("status")
            threshold = metric_policy.get("recommended_threshold_ticks")
            variance_ratio = metric_policy.get("variance_ratio")

        status = "inactive"
        if evaluate_requested:
            if not available:
                status = "metric_unavailable"
                notes.append("split metric unavailable")
            elif not policy_present:
                status = "policy_missing"
                notes.append("threshold policy JSON missing")
            elif not isinstance(metric_policy, dict):
                status = "policy_metric_missing"
                notes.append("metric missing from threshold policy")
            elif policy_status != "ready":
                status = f"policy_{policy_status or 'missing'}"
                notes.append(f"policy status is {policy_status or 'missing'}")
            elif variance_ratio is not None and float(variance_ratio) > variance_guard:
                status = "variance_guard_blocked"
                notes.append(
                    f"variance_ratio {float(variance_ratio):.6f} exceeds guard {variance_guard:.2f}"
                )
            elif threshold is not None and value > float(threshold):
                status = "violation"
                notes.append(
                    f"value {value:.1f} exceeds threshold {float(threshold):.1f}"
                )
            else:
                status = "ok"

        consecutive_violations = 0
        if status == "violation":
            consecutive_violations = int(previous_metric.get("consecutive_violations", 0)) + 1
        elif status == "ok":
            consecutive_violations = 0

        rollback_active = bool(previous_metric.get("rollback_active", False))
        rollback_reason = str(previous_metric.get("rollback_reason", ""))
        rollback_triggered = False

        if not enforcement_requested or not requested:
            rollback_active = False
            rollback_reason = ""

        enforcement_enabled = (
            enforcement_requested
            and requested
            and not bool(global_disabled_reason)
            and not rollback_active
        )

        if enforcement_enabled and status == "violation":
            shadow_violation_count += 1
            blocking_violation_count += 1
            if consecutive_violations >= consecutive_limit:
                rollback_active = True
                rollback_reason = "consecutive_violations"
                rollback_triggered = True
                enforcement_enabled = False
                notes.append(
                    f"metric rollback triggered after {consecutive_violations} consecutive violations"
                )
        elif status == "violation":
            shadow_violation_count += 1

        metrics_out[metric_name] = {
            "value_ticks": value,
            "available": available,
            "policy_present": policy_present,
            "policy_status": policy_status,
            "threshold_ticks": threshold,
            "variance_ratio": variance_ratio,
            "requested": requested,
            "enforcement_enabled": enforcement_enabled,
            "status": status,
            "consecutive_violations": consecutive_violations,
            "rollback_active": rollback_active,
            "rollback_reason": rollback_reason,
            "rollback_triggered": rollback_triggered,
            "notes": notes,
        }

    if enforcement_requested and not global_disabled_reason:
        enabled_violation_count = sum(
            1
            for metric_payload in metrics_out.values()
            if metric_payload.get("status") == "violation"
            and metric_payload.get("enforcement_enabled")
        )
        if enabled_violation_count >= multi_metric_limit:
            global_disabled_reason = "multi_metric_violation"
            multi_metric_triggered = True
            for metric_payload in metrics_out.values():
                if metric_payload.get("enforcement_enabled"):
                    metric_payload["enforcement_enabled"] = False
                    metric_payload["notes"].append(
                        "global disable triggered by multi-metric violation limit"
                    )

    global_enforcement_enabled = enforcement_requested and not bool(global_disabled_reason)

    output = {
        "schema_version": 1,
        "gate": "performance-split-enforcement",
        "mode": (
            "enforce"
            if enforcement_requested
            else "shadow"
            if shadow_requested
            else "disabled"
        ),
        "policy_path": str(policy_path),
        "policy_present": policy_present,
        "policy_authority": policy_authority,
        "policy_trusted": policy_trusted,
        "policy_trust_reason": policy_trust_reason,
        "policy_verification_path": str(policy_verification_path),
        "policy_verification": policy_verification,
        "current_authority": authority,
        "current_env_hash": env_hash,
        "global": {
            "shadow_requested": shadow_requested,
            "enforcement_requested": enforcement_requested,
            "enforcement_enabled": global_enforcement_enabled,
            "disabled_reason": global_disabled_reason,
            "variance_guard": variance_guard,
            "consecutive_violation_limit": consecutive_limit,
            "multi_metric_violation_limit": multi_metric_limit,
            "shadow_violation_count": shadow_violation_count,
            "blocking_violation_count": blocking_violation_count,
            "multi_metric_triggered": multi_metric_triggered,
        },
        "metrics": metrics_out,
        "note": "Default-off scaffold only. This surface is non-authoritative until a later activation PR explicitly consumes it.",
    }

    state_out = {
        "schema_version": 1,
        "authority": authority,
        "env_hash": env_hash,
        "global": output["global"],
        "metrics": {
            metric_name: {
                "consecutive_violations": metric_payload["consecutive_violations"],
                "last_status": metric_payload["status"],
                "rollback_active": metric_payload["rollback_active"],
                "rollback_reason": metric_payload["rollback_reason"],
                "last_threshold_ticks": metric_payload["threshold_ticks"],
                "last_value_ticks": metric_payload["value_ticks"],
                "last_variance_ratio": metric_payload["variance_ratio"],
            }
            for metric_name, metric_payload in metrics_out.items()
        },
    }

    state_path.parent.mkdir(parents=True, exist_ok=True)
    output_path.parent.mkdir(parents=True, exist_ok=True)
    state_path.write_text(json.dumps(state_out, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    output_path.write_text(json.dumps(output, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
