#!/usr/bin/env python3
"""Run an AykenOS validator set against normalized scenario evidence."""

from __future__ import annotations

import argparse
import importlib.util
import json
import traceback
from pathlib import Path


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Run scenario validators.")
    parser.add_argument("--scenario", required=True, help="Scenario JSON path.")
    parser.add_argument("--normalized", required=True, help="Normalized report path.")
    parser.add_argument("--error-db", required=True, help="error_codes.json path.")
    parser.add_argument("--out", required=True, help="Output validator report path.")
    return parser.parse_args()


def read_json(path: Path) -> dict:
    with path.open("r", encoding="utf-8") as handle:
        return json.load(handle)


def write_json(path: Path, payload: object) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("w", encoding="utf-8") as handle:
        json.dump(payload, handle, indent=2, sort_keys=True)
        handle.write("\n")


def load_module(module_path: Path):
    spec = importlib.util.spec_from_file_location(module_path.stem, module_path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"unable to load validator: {module_path}")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def failure_record(validator: str, error_code: str, message: str, details: dict) -> dict:
    return {
        "validator": validator,
        "verdict": "FAIL",
        "error_code": error_code,
        "message": message,
        "details": details,
    }


def main() -> int:
    args = parse_args()
    scenario = read_json(Path(args.scenario))
    normalized = read_json(Path(args.normalized))
    error_db = read_json(Path(args.error_db))
    root = Path(__file__).resolve().parents[2]
    infra_error_code = "AYK-E904"

    results: list[dict] = []
    failed: list[dict] = []

    for rel_path in scenario.get("validators", []):
        module_path = (root / rel_path).resolve()
        validator_label = module_path.stem.replace("validate_", "", 1)
        try:
            module = load_module(module_path)
            validator_id = getattr(module, "VALIDATOR_ID", "") or validator_label
            error_code = getattr(module, "ERROR_CODE", "")
            description = getattr(module, "DESCRIPTION", "")
            if error_code not in error_db:
                raise RuntimeError(
                    f"undefined error code for validator {validator_id}: {error_code}"
                )
            outcome = module.validate(normalized)
            verdict = outcome.get("verdict", "FAIL")
            record = {
                "validator": validator_id,
                "description": description,
                "verdict": verdict,
                "message": outcome.get("message", ""),
                "details": outcome.get("details", {}),
            }
            if verdict != "PASS":
                record["error_code"] = error_code
        except Exception as exc:  # noqa: BLE001
            record = failure_record(
                validator_label,
                infra_error_code,
                "validator import or execution failed",
                {
                    "exception": repr(exc),
                    "traceback": traceback.format_exc(),
                    "validator_path": str(module_path),
                },
            )
            verdict = "FAIL"

        if verdict != "PASS":
            failed.append(
                {
                    "validator": record["validator"],
                    "error_code": record.get("error_code", infra_error_code),
                    "message": record.get("message", ""),
                }
            )
        results.append(record)

    payload = {
        "schema_version": "1.0",
        "scenario_id": scenario.get("scenario_id"),
        "verdict": "PASS" if not failed else "FAIL",
        "validators": results,
        "failed_validators": failed,
        "failed_count": len(failed),
    }
    write_json(Path(args.out), payload)
    return 0 if not failed else 2


if __name__ == "__main__":
    raise SystemExit(main())
