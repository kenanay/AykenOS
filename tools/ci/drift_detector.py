#!/usr/bin/env python3
"""Gate-6 drift telemetry detector (Tier-3, non-constitutional)."""

from __future__ import annotations

import hashlib
import json
from datetime import datetime, timezone
from pathlib import Path
from statistics import mean, pstdev


def _is_number(value) -> bool:
    return isinstance(value, (int, float))


def _stable_sha256(payload: dict) -> str:
    canonical = json.dumps(payload, sort_keys=True, separators=(",", ":"))
    return hashlib.sha256(canonical.encode("utf-8")).hexdigest()


def _normalize_context(raw_context: dict) -> dict:
    normalized: dict[str, str] = {}
    required = (
        "kernel_profile",
        "ai_policy_hash",
        "workload_id",
        "marker_schema_version",
        "run_class",
    )
    for key in required:
        value = raw_context.get(key, "")
        text = str(value).strip()
        if key in ("kernel_profile", "run_class"):
            text = text.lower()
        normalized[key] = text

    # Keep optional context fields deterministic for future extensions.
    for key in sorted(raw_context.keys()):
        if key in normalized:
            continue
        value = raw_context.get(key, "")
        normalized[str(key)] = str(value).strip()
    return normalized


def _load_json(path: Path) -> dict:
    if not path.is_file():
        return {}
    try:
        data = json.loads(path.read_text(encoding="utf-8", errors="replace"))
    except Exception:
        return {}
    return data if isinstance(data, dict) else {}


def _metadata_missing(context: dict) -> list[str]:
    missing = []
    for key in (
        "kernel_profile",
        "ai_policy_hash",
        "workload_id",
        "marker_schema_version",
        "run_class",
    ):
        value = str(context.get(key, "")).strip().lower()
        if value in ("", "unknown", "unset", "n/a", "na"):
            missing.append(key)
    return missing


def _read_history(history_file: Path) -> list[dict]:
    if not history_file.is_file():
        return []
    rows = []
    with history_file.open("r", encoding="utf-8", errors="replace") as fh:
        for raw in fh:
            line = raw.strip()
            if not line:
                continue
            try:
                row = json.loads(line)
            except Exception:
                continue
            if isinstance(row, dict):
                rows.append(row)
    return rows


def _append_history(history_file: Path, row: dict) -> None:
    history_file.parent.mkdir(parents=True, exist_ok=True)
    with history_file.open("a", encoding="utf-8") as fh:
        fh.write(json.dumps(row, sort_keys=True) + "\n")


def _history_metric_values(rows: list[dict], metric: str, last_n: int) -> list[float]:
    values = []
    for row in rows:
        metrics = row.get("metrics")
        if not isinstance(metrics, dict):
            continue
        value = metrics.get(metric)
        if _is_number(value):
            values.append(float(value))
    if last_n > 0:
        values = values[-last_n:]
    return values


def _detector_verdict(abs_z: float, warn_abs_z: float, severe_abs_z: float, severe_verdict: str) -> str:
    if abs_z < warn_abs_z:
        return "INFO"
    if abs_z < severe_abs_z:
        return "WARN"
    return severe_verdict


def _score_rank(verdict: str) -> int:
    return {"INFO": 0, "WARN": 1, "FAIL": 2}.get(verdict, 0)


def _consecutive(history_rows: list[dict], current: str, target: str) -> int:
    if current != target:
        return 0
    count = 1
    for row in reversed(history_rows):
        drift_row = row.get("drift")
        if not isinstance(drift_row, dict):
            break
        prev = str(drift_row.get("verdict", "INFO"))
        if prev != target:
            break
        count += 1
    return count


def evaluate_drift(
    *,
    metrics: dict,
    phase: str,
    profile: str,
    context: dict,
    run_id: str,
    history_root: Path,
    drift_profile_path: Path,
) -> dict:
    context = _normalize_context(context)
    context_key = _stable_sha256(context)
    result = {
        "enabled": False,
        "status": "not_active",
        "schema_version": "1.0.0",
        "profile": profile,
        "phase": phase,
        "context": context,
        "context_key": context_key,
        "window": {"last_n": 0},
        "metrics": metrics,
        "detectors": [],
        "persistence": {
            "consecutive_warn": 0,
            "consecutive_fail": 0,
            "threshold": 1,
        },
        "verdict": "INFO",
        "notes": "Drift detector not active.",
    }

    profile_doc = _load_json(drift_profile_path)
    if not profile_doc:
        result["status"] = "profile_not_configured"
        result["notes"] = "Drift profile config missing; telemetry disabled."
        return result

    if str(profile_doc.get("profile", profile)) != profile:
        result["status"] = "profile_mismatch"
        result["notes"] = "Drift profile mismatch; telemetry disabled."
        return result

    phases = profile_doc.get("phases")
    if not isinstance(phases, dict):
        result["status"] = "invalid_profile"
        result["notes"] = "Invalid drift profile schema; telemetry disabled."
        return result

    phase_cfg = phases.get(phase)
    if not isinstance(phase_cfg, dict):
        result["status"] = "phase_not_configured"
        result["notes"] = "Drift phase config missing; telemetry disabled."
        return result

    if not bool(phase_cfg.get("enabled", False)):
        result["status"] = "disabled"
        result["notes"] = "Drift detector explicitly disabled for phase/profile."
        return result

    missing = _metadata_missing(context)
    if missing:
        result["status"] = "metadata_incomplete"
        result["notes"] = (
            "Drift telemetry disabled: missing context metadata (" + ",".join(missing) + ")."
        )
        return result

    result["enabled"] = True
    status = str(phase_cfg.get("status", "telemetry_only")).strip().lower()
    if status not in ("telemetry_only", "observe", "enforce"):
        status = "telemetry_only"
    result["status"] = status
    if status in ("telemetry_only", "observe"):
        result["notes"] = "Non-blocking drift telemetry."
    else:
        result["notes"] = "Drift status is enforce-capable; gate policy decides blocking."

    window_cfg = phase_cfg.get("window")
    last_n = 20
    if isinstance(window_cfg, dict):
        last_n_raw = window_cfg.get("last_n", 20)
        if isinstance(last_n_raw, (int, float)) and int(last_n_raw) > 0:
            last_n = int(last_n_raw)
    result["window"] = {"last_n": last_n}

    persistence_cfg = phase_cfg.get("persistence")
    persistence_threshold = 5
    if isinstance(persistence_cfg, dict):
        th = persistence_cfg.get("threshold", 5)
        if isinstance(th, (int, float)) and int(th) > 0:
            persistence_threshold = int(th)
    result["persistence"]["threshold"] = persistence_threshold

    history_file = history_root / profile / f"{context_key}.jsonl"
    history_rows = _read_history(history_file)

    detectors_cfg = phase_cfg.get("detectors")
    if not isinstance(detectors_cfg, list):
        detectors_cfg = []

    drift_verdict = "INFO"
    for idx, det in enumerate(detectors_cfg):
        if not isinstance(det, dict):
            continue
        metric = det.get("metric")
        if not isinstance(metric, str) or not metric:
            continue
        name = str(det.get("name") or f"zscore_detector_{idx}")

        detector_row = {
            "name": name,
            "metric": metric,
            "baseline_mean": None,
            "baseline_std": None,
            "current": None,
            "score": None,
            "verdict": "INFO",
            "notes": "",
        }

        current = metrics.get(metric)
        if not _is_number(current):
            detector_row["notes"] = "metric_not_numeric"
            result["detectors"].append(detector_row)
            continue
        current = float(current)
        detector_row["current"] = current

        values = _history_metric_values(history_rows, metric, last_n)
        min_samples = int(det.get("min_samples", 3))
        if len(values) < max(min_samples, 1):
            detector_row["notes"] = "insufficient_history"
            result["detectors"].append(detector_row)
            continue

        mu = mean(values)
        sigma = pstdev(values)
        detector_row["baseline_mean"] = mu
        detector_row["baseline_std"] = sigma

        warn_abs_z = float(det.get("warn_abs_z", 1.5))
        severe_abs_z = float(det.get("severe_abs_z", 3.0))
        severe_verdict = str(det.get("severe_verdict", "WARN")).upper()
        if severe_verdict not in ("WARN", "FAIL"):
            severe_verdict = "WARN"

        if sigma == 0:
            if current == mu:
                score = 0.0
                verdict = "INFO"
            else:
                score = None
                verdict = severe_verdict
                detector_row["notes"] = "zero_std_nonzero_delta"
        else:
            score = (current - mu) / sigma
            verdict = _detector_verdict(abs(score), warn_abs_z, severe_abs_z, severe_verdict)

        detector_row["score"] = score
        detector_row["verdict"] = verdict
        result["detectors"].append(detector_row)
        if _score_rank(verdict) > _score_rank(drift_verdict):
            drift_verdict = verdict

    result["verdict"] = drift_verdict
    result["persistence"]["consecutive_warn"] = _consecutive(history_rows, drift_verdict, "WARN")
    result["persistence"]["consecutive_fail"] = _consecutive(history_rows, drift_verdict, "FAIL")

    history_row = {
        "time_utc": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "run_id": run_id,
        "phase": phase,
        "profile": profile,
        "context_key": context_key,
        "metrics": {
            key: float(val) if _is_number(val) else val
            for key, val in metrics.items()
        },
        "drift": {
            "verdict": drift_verdict,
            "detectors": [
                {
                    "name": row.get("name"),
                    "metric": row.get("metric"),
                    "verdict": row.get("verdict"),
                    "score": row.get("score"),
                }
                for row in result["detectors"]
            ],
        },
    }
    _append_history(history_file, history_row)
    return result
