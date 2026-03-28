#!/usr/bin/env python3
"""Validate AykenOS external test error-code usage against the registry."""

from __future__ import annotations

import argparse
import ast
import json
import re
from pathlib import Path


ERROR_CODE_RE = re.compile(r"AYK-E\d{3}")
VALIDATOR_ID_RE = re.compile(r"^AYK_(KRN|SYS|PRJ|INT)_L([0-3])_[A-Z0-9]+_[A-Z0-9]+(_[A-Z0-9]+)*$")


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Validate AykenOS error code usage.")
    parser.add_argument("--root", default=".", help="Repository root.")
    parser.add_argument(
        "--error-db",
        default="docs/governance/error_codes.json",
        help="Error code registry path.",
    )
    parser.add_argument("--out", default="", help="Optional output report.json path.")
    return parser.parse_args()


def write_json(path: Path, payload: object) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("w", encoding="utf-8") as handle:
        json.dump(payload, handle, indent=2, sort_keys=True)
        handle.write("\n")


def validate(root: Path, error_db_path: Path) -> dict:
    violations: list[str] = []
    observed: list[str] = []
    unused: list[str] = []

    if not error_db_path.is_file():
        return {
            "gate": "error-codes",
            "verdict": "FAIL",
            "violations": [f"missing_error_db:{error_db_path}"],
            "violations_count": 1,
            "observed": [],
            "unused_codes": [],
        }

    error_db = json.loads(error_db_path.read_text(encoding="utf-8"))
    db_codes = set(error_db.keys())
    used_codes: set[str] = set()
    validator_error_map: dict[str, str] = {}
    code_owner_map: dict[str, str] = {}

    scan_roots = [root / "tests", root / "tools/test_runner"]
    for scan_root in scan_roots:
        if not scan_root.exists():
            continue
        for path in sorted(scan_root.rglob("*")):
            if not path.is_file():
                continue
            if path.suffix not in {".py", ".json"}:
                continue
            text = path.read_text(encoding="utf-8", errors="ignore")
            for match in ERROR_CODE_RE.findall(text):
                used_codes.add(match)
                observed.append(f"{path}:{match}")

    validators_root = root / "tests"
    for path in sorted(validators_root.rglob("*.py")):
        if "validators" not in path.parts:
            continue
        tree = ast.parse(path.read_text(encoding="utf-8"), filename=str(path))
        validator_id = ""
        error_code = ""
        for node in tree.body:
            if isinstance(node, ast.Assign):
                for target in node.targets:
                    if isinstance(target, ast.Name) and target.id == "VALIDATOR_ID":
                        if isinstance(node.value, ast.Constant) and isinstance(node.value.value, str):
                            validator_id = node.value.value
                    if isinstance(target, ast.Name) and target.id == "ERROR_CODE":
                        if isinstance(node.value, ast.Constant) and isinstance(node.value.value, str):
                            error_code = node.value.value
        if validator_id:
            validator_error_map[validator_id] = error_code
            match = VALIDATOR_ID_RE.fullmatch(validator_id)
            if not match:
                violations.append(f"invalid_validator_id_for_error_check:{validator_id}")
            elif error_code in error_db:
                expected_layer = f"L{match.group(2)}"
                observed_layer = error_db[error_code].get("layer", "")
                if observed_layer != expected_layer:
                    violations.append(
                        f"validator_error_layer_mismatch:{validator_id}:{error_code}:expected={expected_layer}:actual={observed_layer}"
                    )
            if error_code in code_owner_map and code_owner_map[error_code] != validator_id:
                violations.append(
                    f"duplicate_validator_error_code:{error_code}:{code_owner_map[error_code]}:{validator_id}"
                )
            elif error_code:
                code_owner_map[error_code] = validator_id

    for code in sorted(used_codes):
        if code not in db_codes:
            violations.append(f"undefined_error_code:{code}")

    unused = sorted(db_codes - used_codes)
    report = {
        "gate": "error-codes",
        "verdict": "PASS" if not violations else "FAIL",
        "violations": violations,
        "violations_count": len(violations),
        "observed": observed,
        "validator_error_map": validator_error_map,
        "unused_codes": unused,
    }
    return report


def main() -> int:
    args = parse_args()
    root = Path(args.root).resolve()
    report = validate(root, (root / args.error_db).resolve())
    if args.out:
        write_json(Path(args.out), report)
    else:
        print(json.dumps(report, indent=2, sort_keys=True))
    return 0 if report["violations_count"] == 0 else 2


if __name__ == "__main__":
    raise SystemExit(main())
