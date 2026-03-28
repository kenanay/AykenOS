#!/usr/bin/env python3
"""AykenOS external test pipeline: scenario -> normalize -> validators -> verdict."""

from __future__ import annotations

import argparse
import json
import subprocess
import sys
from pathlib import Path


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Run AykenOS external test pipeline.")
    parser.add_argument("--scenario", required=True, help="Scenario JSON path.")
    parser.add_argument("--evidence-dir", required=True, help="Pipeline evidence directory.")
    parser.add_argument("--out", required=True, help="Output pipeline report path.")
    return parser.parse_args()


def read_json(path: Path) -> dict:
    with path.open("r", encoding="utf-8") as handle:
        return json.load(handle)


def write_json(path: Path, payload: object) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("w", encoding="utf-8") as handle:
        json.dump(payload, handle, indent=2, sort_keys=True)
        handle.write("\n")


def run_step(script: Path, *args: str) -> int:
    proc = subprocess.run([sys.executable, str(script), *args], check=False)
    return proc.returncode


def safe_read_json(path: Path) -> dict | None:
    try:
        return read_json(path)
    except Exception:  # noqa: BLE001
        return None


def main() -> int:
    args = parse_args()
    root = Path(__file__).resolve().parents[2]
    evidence_dir = Path(args.evidence_dir).resolve()
    evidence_dir.mkdir(parents=True, exist_ok=True)

    scenario_path = Path(args.scenario).resolve()
    run_report = evidence_dir / "run_report.json"
    normalized_report = evidence_dir / "normalized_report.json"
    validator_report = evidence_dir / "validator_report.json"
    pipeline_report = Path(args.out).resolve()

    run_scenario = Path(__file__).with_name("run_scenario.py")
    normalize = Path(__file__).with_name("normalize_evidence.py")
    run_validators = Path(__file__).with_name("run_validator_set.py")
    error_db = root / "docs/governance/error_codes.json"

    run_rc = run_step(
        run_scenario,
        "--scenario",
        str(scenario_path),
        "--evidence-dir",
        str(evidence_dir),
        "--out",
        str(run_report),
    )
    if run_rc != 0:
        payload = {
            "schema_version": "1.0",
            "gate": "kernel-test-pipeline",
            "scenario_id": read_json(scenario_path).get("scenario_id"),
            "verdict": "FAIL",
            "pipeline_verdict": "FAIL",
            "error_code": "AYK-E901",
            "message": "scenario execution failed",
            "run_report": str(run_report),
            "violations": ["AYK-E901"],
            "violations_count": 1,
        }
        write_json(pipeline_report, payload)
        return 3

    normalize_rc = run_step(
        normalize,
        "--scenario",
        str(scenario_path),
        "--run-report",
        str(run_report),
        "--out",
        str(normalized_report),
    )
    if normalize_rc != 0:
        payload = {
            "schema_version": "1.0",
            "gate": "kernel-test-pipeline",
            "scenario_id": read_json(scenario_path).get("scenario_id"),
            "verdict": "FAIL",
            "pipeline_verdict": "FAIL",
            "error_code": "AYK-E903",
            "message": "normalized evidence is missing or malformed",
            "run_report": str(run_report),
            "violations": ["AYK-E903"],
            "violations_count": 1,
        }
        write_json(pipeline_report, payload)
        return 3

    validator_rc = run_step(
        run_validators,
        "--scenario",
        str(scenario_path),
        "--normalized",
        str(normalized_report),
        "--error-db",
        str(error_db),
        "--out",
        str(validator_report),
    )

    validator_payload = safe_read_json(validator_report)
    if validator_payload is None:
        payload = {
            "schema_version": "1.0",
            "gate": "kernel-test-pipeline",
            "scenario_id": read_json(scenario_path).get("scenario_id"),
            "verdict": "FAIL",
            "pipeline_verdict": "FAIL",
            "error_code": "AYK-E903",
            "message": "validator report is missing or malformed",
            "violations": ["AYK-E903"],
            "violations_count": 1,
            "run_report": str(run_report),
            "normalized_report": str(normalized_report),
            "validator_report": str(validator_report),
        }
        write_json(pipeline_report, payload)
        return 3
    payload = {
        "schema_version": "1.0",
        "gate": "kernel-test-pipeline",
        "scenario_id": validator_payload.get("scenario_id"),
        "verdict": validator_payload.get("verdict"),
        "pipeline_verdict": validator_payload.get("verdict"),
        "failed_validators": validator_payload.get("failed_validators", []),
        "violations": [
            item.get("error_code", "UNKNOWN") for item in validator_payload.get("failed_validators", [])
        ],
        "violations_count": len(validator_payload.get("failed_validators", [])),
        "run_report": str(run_report),
        "normalized_report": str(normalized_report),
        "validator_report": str(validator_report),
    }
    write_json(pipeline_report, payload)
    return 0 if validator_rc == 0 else 2


if __name__ == "__main__":
    raise SystemExit(main())
