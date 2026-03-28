#!/usr/bin/env python3
"""Validate AykenOS external test naming contract."""

from __future__ import annotations

import argparse
import ast
import json
import re
from pathlib import Path


TEST_ID_RE = re.compile(r"^AYK_(KRN|SYS|PRJ|INT)_L[0-3]_[A-Z0-9]+_[A-Z0-9]+(_[A-Z0-9]+)*$")
SCENARIO_ID_RE = re.compile(r"^AYK_SCN_(KRN|SYS|PRJ|INT)_[A-Z0-9]+_[A-Z0-9]+(_[A-Z0-9]+)*$")


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Validate AykenOS external test naming.")
    parser.add_argument("--root", default=".", help="Repository root.")
    parser.add_argument("--out", default="", help="Optional output report.json path.")
    return parser.parse_args()


def write_json(path: Path, payload: object) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("w", encoding="utf-8") as handle:
        json.dump(payload, handle, indent=2, sort_keys=True)
        handle.write("\n")


def validate(root: Path) -> dict:
    violations: list[str] = []
    observed: list[str] = []

    tests_root = root / "tests"
    if not tests_root.exists():
        return {
            "gate": "test-naming",
            "verdict": "PASS",
            "violations": [],
            "violations_count": 0,
            "observed": [],
        }

    for scenario_path in sorted(tests_root.rglob("*.json")):
        if "scenarios" not in scenario_path.parts:
            continue
        payload = json.loads(scenario_path.read_text(encoding="utf-8"))
        scenario_id = payload.get("scenario_id", "")
        observed.append(scenario_id)
        if not scenario_path.stem.startswith("AYK_SCN_"):
            violations.append(f"{scenario_path}:invalid_scenario_filename:{scenario_path.stem}")
        if scenario_path.stem != scenario_id:
            violations.append(f"{scenario_path}:scenario_id_filename_mismatch:{scenario_id}")
        if not SCENARIO_ID_RE.fullmatch(scenario_id):
            violations.append(f"{scenario_path}:invalid_scenario_id:{scenario_id}")

    for validator_path in sorted(tests_root.rglob("*.py")):
        if "validators" not in validator_path.parts:
            continue
        tree = ast.parse(validator_path.read_text(encoding="utf-8"), filename=str(validator_path))
        validator_id = ""
        for node in tree.body:
            if isinstance(node, ast.Assign):
                for target in node.targets:
                    if isinstance(target, ast.Name) and target.id == "VALIDATOR_ID":
                        if isinstance(node.value, ast.Constant) and isinstance(node.value.value, str):
                            validator_id = node.value.value
        observed.append(validator_id)
        if not validator_path.stem.startswith("validate_AYK_"):
            violations.append(f"{validator_path}:invalid_validator_filename:{validator_path.stem}")
        expected_stem = validator_path.stem.replace("validate_", "", 1)
        if not validator_id:
            violations.append(f"{validator_path}:missing_validator_id")
        if expected_stem != validator_id:
            violations.append(f"{validator_path}:validator_id_filename_mismatch:{validator_id}")
        if not TEST_ID_RE.fullmatch(validator_id):
            violations.append(f"{validator_path}:invalid_validator_id:{validator_id}")

    report = {
        "gate": "test-naming",
        "verdict": "PASS" if not violations else "FAIL",
        "violations": violations,
        "violations_count": len(violations),
        "observed": observed,
    }
    return report


def main() -> int:
    args = parse_args()
    report = validate(Path(args.root).resolve())
    if args.out:
        write_json(Path(args.out), report)
    else:
        print(json.dumps(report, indent=2, sort_keys=True))
    return 0 if report["violations_count"] == 0 else 2


if __name__ == "__main__":
    raise SystemExit(main())
