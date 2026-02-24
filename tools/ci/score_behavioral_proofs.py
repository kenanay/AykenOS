#!/usr/bin/env python3
"""Score phase-driven behavioral proofs from marker events."""

from __future__ import annotations

import argparse
import json
from datetime import datetime, timezone
from pathlib import Path

from drift_detector import evaluate_drift


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Score behavioral proofs from marker events.jsonl."
    )
    parser.add_argument("--events", required=True, help="Input events.jsonl path")
    parser.add_argument("--suite", required=True, help="Behavioral suite json path")
    parser.add_argument("--out", required=True, help="Output report.json path")
    parser.add_argument(
        "--violations-out",
        required=True,
        help="Output violations.txt path",
    )
    parser.add_argument(
        "--seed-violations",
        default="",
        help="Optional path with pre-scoring violation lines",
    )
    parser.add_argument("--phase", default="", help="Phase key (default: suite.phase)")
    parser.add_argument("--run-id", default="", help="Run id for report metadata")
    parser.add_argument(
        "--kernel-profile",
        default="validation",
        help="Kernel profile for metadata",
    )
    parser.add_argument(
        "--strict-mode",
        default="",
        help="Override strict mode: 1 or 0",
    )
    parser.add_argument(
        "--envelope-file",
        default="",
        help="Optional profile-scoped envelope config file",
    )
    parser.add_argument(
        "--drift-profile-file",
        default="",
        help="Optional drift profile config file",
    )
    parser.add_argument(
        "--history-root",
        default="evidence/history",
        help="History root for drift telemetry window storage",
    )
    parser.add_argument(
        "--ai-policy-hash",
        default="unknown",
        help="AI policy hash (ABDF/BCIB context field)",
    )
    parser.add_argument(
        "--workload-id",
        default="default",
        help="Workload identifier (ABDF/BCIB context field)",
    )
    parser.add_argument(
        "--run-class",
        default="ci",
        help="Run class for context key: ci/local/lab/perf",
    )
    parser.add_argument(
        "--marker-schema-version",
        default="",
        help="Marker schema version override (default: constitution/runtime_markers.json)",
    )
    return parser.parse_args()


def load_json(path: Path) -> dict:
    try:
        data = json.loads(path.read_text(encoding="utf-8", errors="replace"))
    except Exception as exc:
        raise RuntimeError(f"json_parse_failed:{path}:{type(exc).__name__}") from exc
    if not isinstance(data, dict):
        raise RuntimeError(f"json_type_invalid:{path}:root_must_be_object")
    return data


def load_events(path: Path) -> list[dict]:
    if not path.is_file():
        raise RuntimeError(f"missing_file:{path}")
    events: list[dict] = []
    with path.open("r", encoding="utf-8", errors="replace") as fh:
        for line_no, raw in enumerate(fh, start=1):
            line = raw.strip()
            if not line:
                continue
            try:
                row = json.loads(line)
            except Exception as exc:
                raise RuntimeError(
                    f"event_parse_failed:{path}:line={line_no}:{type(exc).__name__}"
                ) from exc
            if not isinstance(row, dict):
                raise RuntimeError(
                    f"event_type_invalid:{path}:line={line_no}:row_must_be_object"
                )
            events.append(row)
    return events


def load_seed_violations(path: Path) -> list[str]:
    if not path:
        return []
    if not path.exists() or not path.is_file():
        return []
    out = []
    for raw in path.read_text(encoding="utf-8", errors="replace").splitlines():
        row = raw.strip()
        if row:
            out.append(row)
    return out


def build_signals(events: list[dict]) -> dict:
    signals = {
        "accept_count": 0,
        "reject_count": 0,
        "ring3_ok_count": 0,
        "r3ok_user_count": 0,
        "epochs_non_zero": [],
    }
    for row in events:
        kind = str(row.get("type", ""))
        if kind == "ACCEPT":
            signals["accept_count"] += 1
        elif kind == "REJECT":
            signals["reject_count"] += 1
        elif kind == "AYKEN_RING3_OK":
            signals["ring3_ok_count"] += 1
        elif kind == "R3OK_USER_TOKEN":
            signals["r3ok_user_count"] += 1

        if kind in ("ACCEPT", "REJECT"):
            epoch = row.get("epoch")
            if isinstance(epoch, int) and epoch > 0:
                signals["epochs_non_zero"].append(epoch)
    return signals


def as_number(raw) -> float | None:
    if isinstance(raw, (int, float)):
        return float(raw)
    return None


def build_metrics(signals: dict) -> dict:
    accept = float(signals["accept_count"])
    reject = float(signals["reject_count"])
    total = accept + reject
    ring3_total = float(signals["r3ok_user_count"] + signals["ring3_ok_count"])

    accept_rate = 0.0
    reject_rate = 0.0
    if total > 0.0:
        accept_rate = accept / total
        reject_rate = reject / total

    return {
        "accept_count": accept,
        "reject_count": reject,
        "total_sched_events": total,
        "accept_rate": accept_rate,
        "reject_rate": reject_rate,
        "ring3_marker_count": ring3_total,
        "ring3_kernel_count": float(signals["ring3_ok_count"]),
        "ring3_user_count": float(signals["r3ok_user_count"]),
        "epochs_non_zero_count": float(len(signals["epochs_non_zero"])),
    }


def threshold_from_rule(rule: dict) -> tuple[float | None, float | None]:
    low = as_number(rule.get("min"))
    high = as_number(rule.get("max"))

    baseline = as_number(rule.get("baseline"))
    if baseline is not None:
        delta_abs = as_number(rule.get("delta_abs"))
        if delta_abs is not None:
            low = baseline - delta_abs if low is None else max(low, baseline - delta_abs)
            high = baseline + delta_abs if high is None else min(high, baseline + delta_abs)

        min_delta_abs = as_number(rule.get("min_delta_abs"))
        if min_delta_abs is not None:
            edge = baseline - min_delta_abs
            low = edge if low is None else max(low, edge)

        max_delta_abs = as_number(rule.get("max_delta_abs"))
        if max_delta_abs is not None:
            edge = baseline + max_delta_abs
            high = edge if high is None else min(high, edge)

        delta_ratio = as_number(rule.get("delta_ratio"))
        if delta_ratio is not None:
            lo_edge = baseline * (1.0 - delta_ratio)
            hi_edge = baseline * (1.0 + delta_ratio)
            low = lo_edge if low is None else max(low, lo_edge)
            high = hi_edge if high is None else min(high, hi_edge)

        min_relative_delta = as_number(rule.get("min_relative_delta"))
        if min_relative_delta is not None:
            edge = baseline * (1.0 - min_relative_delta)
            low = edge if low is None else max(low, edge)

        max_relative_delta = as_number(rule.get("max_relative_delta"))
        if max_relative_delta is not None:
            edge = baseline * (1.0 + max_relative_delta)
            high = edge if high is None else min(high, edge)

    return low, high


def evaluate_envelope(
    envelope_path: Path,
    kernel_profile: str,
    phase: str,
    metrics: dict,
) -> dict:
    result = {
        "enabled": False,
        "status": "not_configured",
        "file": str(envelope_path),
        "profile": kernel_profile,
        "phase": phase,
        "mode": "warn",
        "warn_escalates_in_strict": False,
        "verdict": "SKIP",
        "metrics": metrics,
        "rules": [],
        "violations": [],
        "warnings": [],
    }

    if not envelope_path.exists():
        return result

    envelope = load_json(envelope_path)
    if str(envelope.get("profile", kernel_profile)) != kernel_profile:
        result["violations"].append(
            f"envelope_profile_mismatch:expected={kernel_profile}:actual={envelope.get('profile')}"
        )
        result["status"] = "profile_mismatch"
        result["verdict"] = "FAIL"
        return result

    phases = envelope.get("phases")
    if not isinstance(phases, dict):
        result["violations"].append("envelope_phases_missing")
        result["status"] = "invalid_schema"
        result["verdict"] = "FAIL"
        return result

    phase_cfg = phases.get(phase)
    if not isinstance(phase_cfg, dict):
        result["status"] = "phase_not_configured"
        return result

    enabled = bool(phase_cfg.get("enabled", True))
    result["enabled"] = enabled
    if not enabled:
        result["status"] = "disabled"
        return result

    mode = str(phase_cfg.get("mode", "warn")).lower()
    if mode not in ("warn", "fail"):
        result["violations"].append(f"envelope_mode_invalid:{mode}")
        result["status"] = "invalid_mode"
        result["verdict"] = "FAIL"
        return result
    result["mode"] = mode

    warn_escalates = bool(phase_cfg.get("warn_escalates_in_strict", False))
    result["warn_escalates_in_strict"] = warn_escalates
    result["status"] = "active"

    min_samples_raw = phase_cfg.get("min_samples", 0)
    min_samples = 0
    if isinstance(min_samples_raw, int) and min_samples_raw >= 0:
        min_samples = min_samples_raw
    elif isinstance(min_samples_raw, float) and min_samples_raw >= 0:
        min_samples = int(min_samples_raw)
    else:
        result["violations"].append("envelope_min_samples_invalid")
        result["verdict"] = "FAIL"
        return result

    total_sched_events = metrics.get("total_sched_events", 0.0)
    if total_sched_events < float(min_samples):
        result["warnings"].append(
            f"envelope_insufficient_samples:actual={int(total_sched_events)}:required={min_samples}"
        )

    rules = phase_cfg.get("rules")
    if not isinstance(rules, list):
        result["violations"].append("envelope_rules_missing")
        result["verdict"] = "FAIL"
        return result

    has_fail = False
    has_warn = bool(result["warnings"])

    for idx, rule in enumerate(rules):
        if not isinstance(rule, dict):
            result["violations"].append(f"envelope_rule_invalid:index={idx}")
            has_fail = True
            continue

        metric = rule.get("metric")
        if not isinstance(metric, str) or not metric:
            result["violations"].append(f"envelope_rule_metric_invalid:index={idx}")
            has_fail = True
            continue

        metric_value = metrics.get(metric)
        if not isinstance(metric_value, (int, float)):
            result["violations"].append(f"envelope_rule_metric_unknown:{metric}")
            has_fail = True
            continue

        severity = str(rule.get("severity", mode)).lower()
        if severity not in ("warn", "fail"):
            result["violations"].append(
                f"envelope_rule_severity_invalid:{metric}:{severity}"
            )
            has_fail = True
            continue

        baseline = as_number(rule.get("baseline"))
        has_explicit_threshold = (
            as_number(rule.get("min")) is not None
            or as_number(rule.get("max")) is not None
        )
        has_delta_threshold = (
            baseline is not None
            or as_number(rule.get("delta_abs")) is not None
            or as_number(rule.get("min_delta_abs")) is not None
            or as_number(rule.get("max_delta_abs")) is not None
            or as_number(rule.get("delta_ratio")) is not None
            or as_number(rule.get("min_relative_delta")) is not None
            or as_number(rule.get("max_relative_delta")) is not None
        )
        if has_explicit_threshold and has_delta_threshold:
            result["violations"].append(
                f"envelope_rule_threshold_ambiguous:{metric}:use_explicit_or_baseline_delta"
            )
            has_fail = True
            continue

        low, high = threshold_from_rule(rule)
        if low is None and high is None:
            result["violations"].append(f"envelope_rule_threshold_missing:{metric}")
            has_fail = True
            continue

        pass_rule = True
        if low is not None and metric_value < low:
            pass_rule = False
        if high is not None and metric_value > high:
            pass_rule = False

        rule_name = str(rule.get("name") or f"rule_{idx}")
        rule_row = {
            "name": rule_name,
            "metric": metric,
            "value": metric_value,
            "min": low,
            "max": high,
            "severity": severity,
            "verdict": "PASS",
        }

        if not pass_rule:
            msg = (
                f"envelope_rule_failed:{rule_name}:metric={metric}:"
                f"value={metric_value}:min={low}:max={high}"
            )
            if severity == "fail":
                rule_row["verdict"] = "FAIL"
                result["violations"].append(msg)
                has_fail = True
            else:
                rule_row["verdict"] = "WARN"
                result["warnings"].append(msg)
                has_warn = True
        result["rules"].append(rule_row)

    if has_fail:
        result["verdict"] = "FAIL"
    elif has_warn:
        result["verdict"] = "WARN"
    else:
        result["verdict"] = "PASS"
    return result


def _to_non_negative_int(raw, default: int) -> int:
    if isinstance(raw, bool):
        return default
    if isinstance(raw, (int, float)):
        as_int = int(raw)
        if as_int >= 0:
            return as_int
    return default


def _parse_phase_int(phase: str) -> int | None:
    try:
        return int(str(phase).strip())
    except Exception:
        return None


def evaluate_drift_blocking_policy(
    *,
    suite_defaults: dict,
    phase: str,
    kernel_profile: str,
    drift: dict,
) -> dict:
    raw = suite_defaults.get("drift_blocking_policy", {})
    if not isinstance(raw, dict):
        raw = {}

    enabled = bool(raw.get("enabled", False))
    phase_min = _to_non_negative_int(raw.get("phase_min", 9), 9)

    profiles_raw = raw.get("profiles", ["validation"])
    profiles: list[str] = []
    if isinstance(profiles_raw, list):
        for row in profiles_raw:
            if isinstance(row, str) and row.strip():
                profiles.append(row.strip())
    if not profiles:
        profiles = ["validation"]

    require_status = str(raw.get("require_status", "enforce")).strip().lower()
    if require_status not in ("", "enforce", "observe", "telemetry_only"):
        require_status = "enforce"

    persistence = drift.get("persistence", {})
    if not isinstance(persistence, dict):
        persistence = {}
    default_threshold = _to_non_negative_int(persistence.get("threshold", 5), 5)
    warn_threshold = _to_non_negative_int(
        raw.get("warn_threshold", default_threshold), default_threshold
    )
    fail_threshold = _to_non_negative_int(
        raw.get("fail_threshold", default_threshold), default_threshold
    )
    if warn_threshold <= 0:
        warn_threshold = default_threshold if default_threshold > 0 else 1
    if fail_threshold <= 0:
        fail_threshold = default_threshold if default_threshold > 0 else 1

    phase_int = _parse_phase_int(phase)
    phase_guard_non_blocking = phase_int in (7, 8)
    eligible_phase = phase_int is not None and phase_int >= phase_min
    eligible_profile = kernel_profile in profiles

    drift_status = str(drift.get("status", "not_active")).lower()
    drift_verdict = str(drift.get("verdict", "INFO")).upper()
    consecutive_warn = _to_non_negative_int(persistence.get("consecutive_warn", 0), 0)
    consecutive_fail = _to_non_negative_int(persistence.get("consecutive_fail", 0), 0)

    result = {
        "enabled": enabled,
        "phase_min": phase_min,
        "profiles": profiles,
        "require_status": require_status,
        "warn_threshold": warn_threshold,
        "fail_threshold": fail_threshold,
        "phase_guard_non_blocking": phase_guard_non_blocking,
        "eligible_phase": eligible_phase,
        "eligible_profile": eligible_profile,
        "drift_status": drift_status,
        "drift_verdict": drift_verdict,
        "consecutive_warn": consecutive_warn,
        "consecutive_fail": consecutive_fail,
        "blocking_triggered": False,
        "reason": "policy_disabled",
    }

    if phase_guard_non_blocking:
        result["reason"] = "phase7_8_non_blocking_guard"
        return result
    if not enabled:
        return result
    if not eligible_phase:
        result["reason"] = "phase_below_min"
        return result
    if not eligible_profile:
        result["reason"] = "profile_not_enforced"
        return result
    if require_status and drift_status != require_status:
        result["reason"] = f"status_not_{require_status}"
        return result

    if drift_verdict == "FAIL" and consecutive_fail >= fail_threshold:
        result["blocking_triggered"] = True
        result["reason"] = "persistent_fail_threshold_reached"
        return result
    if drift_verdict == "WARN" and consecutive_warn >= warn_threshold:
        result["blocking_triggered"] = True
        result["reason"] = "persistent_warn_threshold_reached"
        return result

    result["reason"] = "threshold_not_reached"
    return result


def resolve_marker_schema_version(
    *,
    suite_path: Path,
    marker_schema_override: str,
    suite_version: str,
) -> str:
    if marker_schema_override:
        return marker_schema_override
    runtime_registry = suite_path.parent.parent / "runtime_markers.json"
    if runtime_registry.is_file():
        try:
            doc = json.loads(runtime_registry.read_text(encoding="utf-8", errors="replace"))
            schema = doc.get("schema_version")
            if isinstance(schema, str) and schema:
                return schema
        except Exception:
            pass
    return suite_version


def proof_sched_bridge_smoke(signals: dict) -> tuple[dict, list[str], list[str]]:
    violations = []
    if signals["accept_count"] < 1:
        violations.append("sched_bridge_smoke_accept_missing")
    if signals["reject_count"] < 1:
        violations.append("sched_bridge_smoke_reject_missing")
    result = {
        "name": "sched_bridge_smoke",
        "verdict": "PASS" if not violations else "FAIL",
        "signals": {
            "accept": signals["accept_count"],
            "reject": signals["reject_count"],
        },
        "notes": "At least one ACCEPT and one REJECT must be observed.",
    }
    return result, violations, []


def proof_ring3_presence_smoke(signals: dict) -> tuple[dict, list[str], list[str]]:
    observed = signals["r3ok_user_count"] + signals["ring3_ok_count"]
    violations = []
    if observed < 1:
        violations.append("ring3_presence_smoke_missing")
    result = {
        "name": "ring3_presence_smoke",
        "verdict": "PASS" if not violations else "FAIL",
        "signals": {
            "r3ok_user": signals["r3ok_user_count"],
            "ring3_kernel": signals["ring3_ok_count"],
        },
        "notes": "At least one Ring3 execution marker must be observed.",
    }
    return result, violations, []


def proof_epoch_progression_non_decreasing(
    signals: dict,
) -> tuple[dict, list[str], list[str]]:
    epochs = signals["epochs_non_zero"]
    warnings = []
    violations = []
    if not epochs:
        warnings.append("epoch_progression_empty")
        result = {
            "name": "epoch_progression_non_decreasing",
            "verdict": "WARN",
            "signals": {
                "checked_epochs": 0,
                "violations": 0,
            },
            "notes": "No non-zero epochs observed.",
        }
        return result, violations, warnings

    prev = epochs[0]
    count_violations = 0
    for epoch in epochs[1:]:
        if epoch < prev:
            count_violations += 1
        prev = epoch

    if count_violations > 0:
        violations.append(
            f"epoch_progression_non_decreasing_failed:violations={count_violations}"
        )

    result = {
        "name": "epoch_progression_non_decreasing",
        "verdict": "PASS" if count_violations == 0 else "FAIL",
        "signals": {
            "checked_epochs": len(epochs),
            "violations": count_violations,
        },
        "notes": "Non-zero epochs must be non-decreasing.",
    }
    return result, violations, warnings


def proof_policy_mechanism_separation_signal(
    signals: dict,
) -> tuple[dict, list[str], list[str]]:
    violations = []
    if signals["r3ok_user_count"] < 1:
        violations.append("policy_mechanism_separation_missing_ring3_signal")
    if signals["accept_count"] < 1 and signals["reject_count"] < 1:
        violations.append("policy_mechanism_separation_missing_sched_signal")
    result = {
        "name": "policy_mechanism_separation_signal",
        "verdict": "PASS" if not violations else "FAIL",
        "signals": {
            "r3ok_user": signals["r3ok_user_count"],
            "accept": signals["accept_count"],
            "reject": signals["reject_count"],
        },
        "notes": "Ring3 presence and scheduler reaction must both be visible.",
    }
    return result, violations, []


PROOF_MAP = {
    "sched_bridge_smoke": proof_sched_bridge_smoke,
    "ring3_presence_smoke": proof_ring3_presence_smoke,
    "epoch_progression_non_decreasing": proof_epoch_progression_non_decreasing,
    "policy_mechanism_separation_signal": proof_policy_mechanism_separation_signal,
}


def main() -> int:
    args = parse_args()

    events_path = Path(args.events)
    suite_path = Path(args.suite)
    report_path = Path(args.out)
    violations_path = Path(args.violations_out)
    seed_path = Path(args.seed_violations) if args.seed_violations else Path()

    suite = load_json(suite_path)
    events = load_events(events_path)
    seed_violations = load_seed_violations(seed_path)

    phase = str(args.phase or suite.get("phase") or "").strip()
    if not phase:
        raise RuntimeError("suite_phase_missing")

    phases = suite.get("phases")
    if not isinstance(phases, dict) or phase not in phases:
        raise RuntimeError(f"suite_phase_unknown:{phase}")

    phase_cfg = phases.get(phase)
    if not isinstance(phase_cfg, dict):
        raise RuntimeError(f"suite_phase_invalid:{phase}")

    enabled = phase_cfg.get("enabled_proofs")
    if not isinstance(enabled, list):
        raise RuntimeError(f"suite_enabled_proofs_missing:{phase}")

    suite_defaults = suite.get("defaults", {}) if isinstance(suite.get("defaults"), dict) else {}
    strict_mode_cfg = suite_defaults.get("strict_mode", True)
    strict_mode = bool(strict_mode_cfg)
    phase_strict_mode = phase_cfg.get("strict_mode")
    if isinstance(phase_strict_mode, bool):
        strict_mode = phase_strict_mode
    if args.strict_mode:
        if args.strict_mode not in ("0", "1"):
            raise RuntimeError("strict_mode_invalid")
        strict_mode = args.strict_mode == "1"

    proof_modes = phase_cfg.get("proof_modes", {})
    if proof_modes is None:
        proof_modes = {}
    if not isinstance(proof_modes, dict):
        raise RuntimeError(f"suite_proof_modes_invalid:{phase}")

    signals = build_signals(events)
    metrics = build_metrics(signals)
    proofs = []
    proof_modes_effective: dict[str, str] = {}
    violations = list(seed_violations)
    warnings: list[str] = []

    for proof_name_raw in enabled:
        proof_name = str(proof_name_raw)
        mode = str(proof_modes.get(proof_name, "fail")).lower()
        if mode not in ("fail", "warn"):
            raise RuntimeError(f"suite_proof_mode_invalid:{phase}:{proof_name}:{mode}")
        proof_modes_effective[proof_name] = mode

        fn = PROOF_MAP.get(proof_name)
        if fn is None:
            violations.append(f"unknown_proof:{proof_name}")
            proofs.append(
                {
                    "name": proof_name,
                    "mode": mode,
                    "verdict": "FAIL",
                    "signals": {},
                    "notes": "Proof name is not implemented in scorer.",
                }
            )
            continue
        result, pf_violations, pf_warnings = fn(signals)
        result["mode"] = mode
        if mode == "warn" and result.get("verdict") == "FAIL":
            result["verdict"] = "WARN"
            pf_warnings = list(pf_warnings) + list(pf_violations)
            pf_violations = []
        proofs.append(result)
        violations.extend(pf_violations)
        warnings.extend(pf_warnings)

    envelope_path = (
        Path(args.envelope_file)
        if args.envelope_file
        else suite_path.parent / "envelopes" / f"{args.kernel_profile}.json"
    )
    envelope = evaluate_envelope(
        envelope_path=envelope_path,
        kernel_profile=args.kernel_profile,
        phase=phase,
        metrics=metrics,
    )
    violations.extend(envelope.get("violations", []))
    warnings.extend(envelope.get("warnings", []))

    suite_version = str(suite.get("suite_version", "unknown"))
    marker_schema_version = resolve_marker_schema_version(
        suite_path=suite_path,
        marker_schema_override=args.marker_schema_version,
        suite_version=suite_version,
    )
    drift_context = {
        "kernel_profile": args.kernel_profile,
        "ai_policy_hash": args.ai_policy_hash,
        "workload_id": args.workload_id,
        "marker_schema_version": marker_schema_version,
        "run_class": args.run_class,
    }
    drift_profile_path = (
        Path(args.drift_profile_file)
        if args.drift_profile_file
        else suite_path.parent / "drift_profiles" / f"{args.kernel_profile}.json"
    )
    drift = evaluate_drift(
        metrics=metrics,
        phase=phase,
        profile=args.kernel_profile,
        context=drift_context,
        run_id=args.run_id,
        history_root=Path(args.history_root),
        drift_profile_path=drift_profile_path,
    )
    drift_policy = evaluate_drift_blocking_policy(
        suite_defaults=suite_defaults,
        phase=phase,
        kernel_profile=args.kernel_profile,
        drift=drift,
    )
    if bool(drift_policy.get("blocking_triggered", False)):
        violations.append(
            "drift_persistent_blocking_triggered:"
            f"reason={drift_policy.get('reason')}:"
            f"status={drift_policy.get('drift_status')}:"
            f"verdict={drift_policy.get('drift_verdict')}:"
            f"consecutive_warn={drift_policy.get('consecutive_warn')}:"
            f"consecutive_fail={drift_policy.get('consecutive_fail')}"
        )

    proof_has_warn = any(str(p.get("verdict")) == "WARN" for p in proofs)
    envelope_verdict = str(envelope.get("verdict", "SKIP"))
    envelope_has_warn = envelope_verdict == "WARN"

    verdict = "PASS"
    if violations:
        verdict = "FAIL"
    elif proof_has_warn or envelope_has_warn or warnings:
        verdict = "WARN"

    if strict_mode and verdict == "WARN":
        if proof_has_warn:
            verdict = "FAIL"
            violations.append("strict_mode_proof_warn_escalation")
        elif envelope_has_warn and bool(envelope.get("warn_escalates_in_strict", False)):
            verdict = "FAIL"
            violations.append("strict_mode_envelope_warn_escalation")

    # Normalize output ordering for deterministic diffs.
    violations = sorted(set(v for v in violations if v))
    warnings = sorted(set(w for w in warnings if w))
    envelope["violations"] = sorted(
        set(v for v in envelope.get("violations", []) if v)
    )
    envelope["warnings"] = sorted(set(w for w in envelope.get("warnings", []) if w))

    violations_path.parent.mkdir(parents=True, exist_ok=True)
    violations_path.write_text(
        "\n".join(violations) + ("\n" if violations else ""),
        encoding="utf-8",
    )

    report = {
        "gate": "behavioral-suite",
        "tier": "tier-3-behavioral",
        "run_id": args.run_id,
        "kernel_profile": args.kernel_profile,
        "suite_version": suite_version,
        "phase": phase,
        "strict_mode": strict_mode,
        "proof_modes": proof_modes_effective,
        "envelope": envelope,
        "time_utc": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "event_count": len(events),
        "signals": {
            "accept_count": signals["accept_count"],
            "reject_count": signals["reject_count"],
            "ring3_ok_count": signals["ring3_ok_count"],
            "r3ok_user_count": signals["r3ok_user_count"],
            "epochs_non_zero_count": len(signals["epochs_non_zero"]),
        },
        "metrics": metrics,
        "drift": drift,
        "drift_policy": drift_policy,
        "verdict": verdict,
        "violations_count": len(violations),
        "warnings_count": len(warnings),
        "proofs": proofs,
        "violations": violations,
        "warnings": warnings,
    }

    report_path.parent.mkdir(parents=True, exist_ok=True)
    report_path.write_text(
        json.dumps(report, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )

    return 2 if verdict == "FAIL" else 0


if __name__ == "__main__":
    raise SystemExit(main())
