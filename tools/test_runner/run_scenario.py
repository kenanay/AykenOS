#!/usr/bin/env python3
"""Run an external AykenOS test scenario and collect raw evidence."""

from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
from pathlib import Path


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Run a single AykenOS scenario.")
    parser.add_argument("--scenario", required=True, help="Scenario JSON path.")
    parser.add_argument("--evidence-dir", required=True, help="Scenario evidence directory.")
    parser.add_argument("--out", required=True, help="Output run report path.")
    return parser.parse_args()


def read_json(path: Path) -> dict:
    with path.open("r", encoding="utf-8") as handle:
        return json.load(handle)


def write_json(path: Path, payload: object) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("w", encoding="utf-8") as handle:
        json.dump(payload, handle, indent=2, sort_keys=True)
        handle.write("\n")


def main() -> int:
    args = parse_args()
    root = Path(__file__).resolve().parents[2]
    scenario_path = Path(args.scenario).resolve()
    evidence_dir = Path(args.evidence_dir).resolve()
    out_path = Path(args.out).resolve()

    scenario = read_json(scenario_path)
    runner = scenario.get("runner", {})
    if runner.get("kind") != "shell":
        raise SystemExit(f"unsupported runner kind: {runner.get('kind')}")

    command = [part.format(evidence_dir=str(evidence_dir)) for part in runner.get("command", [])]
    env = os.environ.copy()
    for key, value in runner.get("env", {}).items():
        env[key] = str(value).format(evidence_dir=str(evidence_dir))

    evidence_dir.mkdir(parents=True, exist_ok=True)
    stdout_path = evidence_dir / "scenario.stdout.log"
    stderr_path = evidence_dir / "scenario.stderr.log"

    with stdout_path.open("w", encoding="utf-8") as stdout_handle, stderr_path.open(
        "w", encoding="utf-8"
    ) as stderr_handle:
        proc = subprocess.run(
            command,
            cwd=root,
            env=env,
            stdout=stdout_handle,
            stderr=stderr_handle,
            check=False,
        )

    normalizer = scenario.get("normalizer", {})
    gate_report = evidence_dir / normalizer.get("report_relpath", "")
    report = {
        "schema_version": "1.0",
        "scenario_id": scenario.get("scenario_id"),
        "scenario_path": str(scenario_path),
        "domain": scenario.get("domain"),
        "surface": scenario.get("surface"),
        "goal": scenario.get("goal"),
        "command": command,
        "runner_env": {key: env.get(key, "") for key in runner.get("env", {})},
        "exit_code": proc.returncode,
        "verdict": "PASS" if proc.returncode == 0 else "FAIL",
        "gate_report": str(gate_report),
        "stdout_log": str(stdout_path),
        "stderr_log": str(stderr_path),
    }
    write_json(out_path, report)
    return 0 if proc.returncode == 0 else 3


if __name__ == "__main__":
    raise SystemExit(main())
