#!/usr/bin/env python3
"""Phase-17.5 spec validation gate.

This gate keeps historical local Kiro specs from blocking the bootstrap PR while
enforcing fail-closed validation for new post-cutoff specs.
"""

from __future__ import annotations

import argparse
import os
import re
import subprocess
import sys
from dataclasses import dataclass
from datetime import datetime, timezone
from pathlib import Path


DEFAULT_CUTOFF = "2026-05-02 19:00:00"


@dataclass
class Metadata:
    created_at: datetime
    validation_level: int
    phase: str


def parse_dt(value: str) -> datetime:
    raw = value.strip().strip('"').strip("'")
    if raw.endswith("Z"):
        raw = f"{raw[:-1]}+00:00"
    if re.fullmatch(r"\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}", raw):
        return datetime.strptime(raw, "%Y-%m-%d %H:%M:%S").replace(tzinfo=timezone.utc)
    parsed = datetime.fromisoformat(raw)
    if parsed.tzinfo is None:
        parsed = parsed.replace(tzinfo=timezone.utc)
    return parsed.astimezone(timezone.utc)


def read_metadata(path: Path) -> Metadata:
    fields: dict[str, str] = {}
    for line in path.read_text(encoding="utf-8").splitlines():
        match = re.match(r"^\s*(created_at|validation_level|phase):\s*(.+?)\s*(?:#.*)?$", line)
        if match:
            fields[match.group(1)] = match.group(2).strip().strip('"').strip("'")

    missing = [key for key in ("created_at", "validation_level", "phase") if key not in fields]
    if missing:
        raise ValueError(f"metadata missing required field(s): {', '.join(missing)}")

    return Metadata(
        created_at=parse_dt(fields["created_at"]),
        validation_level=int(fields["validation_level"]),
        phase=fields["phase"],
    )


def read_legacy_inventory(path: Path) -> set[str]:
    if not path.exists():
        return set()
    names: set[str] = set()
    for line in path.read_text(encoding="utf-8").splitlines():
        stripped = line.strip()
        if stripped and not stripped.startswith("#"):
            names.add(stripped)
    return names


def first_commit_date(path: Path) -> str:
    result = subprocess.run(
        ["git", "log", "--follow", "--format=%cI", "--reverse", "--", str(path)],
        text=True,
        capture_output=True,
        check=False,
    )
    if result.returncode != 0:
        return ""
    return next((line.strip() for line in result.stdout.splitlines() if line.strip()), "")


def run_validation(spec_dir: Path) -> bool:
    env = os.environ.copy()
    env["SPEC_DIR"] = f"{spec_dir}/"
    result = subprocess.run(["make", "ci-gate-spec-validation"], env=env, check=False)
    return result.returncode == 0


def fail(message: str, details: list[str] | None = None) -> None:
    print(message)
    for detail in details or []:
        print(f"   {detail}")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--spec-root", default=".kiro/specs")
    parser.add_argument("--cutoff", default=DEFAULT_CUTOFF)
    parser.add_argument("--legacy-inventory", default=".kiro/specs/.phase_17_5_legacy_specs")
    args = parser.parse_args()

    spec_root = Path(args.spec_root)
    cutoff = parse_dt(args.cutoff)
    legacy_names = read_legacy_inventory(Path(args.legacy_inventory))

    print("== SPEC VALIDATION GATE (TIER 4.5: PROVABLE INTEGRITY) ==")
    print("Checking for specs requiring validation...")
    print(f"Phase-17.5 cutoff: {cutoff.isoformat()}")
    print("")

    if not spec_root.exists():
        print("✅ No specs directory found - validation not applicable")
        return 0

    spec_count = 0
    validated_count = 0
    legacy_count = 0
    violations = 0

    for spec_dir in sorted(path for path in spec_root.iterdir() if path.is_dir() and not path.name.startswith(".")):
        spec_count += 1
        spec_name = spec_dir.name
        metadata_path = spec_dir / ".metadata.yml"
        validator_path = spec_dir / "ci_gate_spec_validation.sh"
        original_path = spec_dir / "ORIGINAL_BASELINE.md"
        is_legacy = spec_name in legacy_names

        print("")
        print(f">> Checking spec: {spec_name}")

        if not metadata_path.exists():
            if is_legacy:
                print("⚠️  LEGACY SPEC: No metadata found (allowlisted pre-Phase-17.5)")
                print(f"   Spec: {spec_name}")
                print("   Status: Allowed (legacy inventory)")
                legacy_count += 1
                continue

            first_commit = first_commit_date(spec_dir)
            fail(
                "❌ METADATA REQUIRED (post-cutoff spec)",
                [
                    f"Spec: {spec_name}",
                    f"Git first commit: {first_commit or 'unknown'}",
                    "Policy: Post-cutoff specs MUST have .metadata.yml",
                    "Note: Git history is not trusted for new specs without metadata",
                ],
            )
            violations += 1
            continue

        try:
            metadata = read_metadata(metadata_path)
        except Exception as exc:
            fail("❌ METADATA INVALID", [f"Spec: {spec_name}", str(exc)])
            violations += 1
            continue

        print("   Metadata: PRESENT")
        print(f"   Created: {metadata.created_at.isoformat()}")
        print(f"   Validation Level: {metadata.validation_level}")

        if metadata.created_at <= cutoff:
            if not is_legacy:
                fail(
                    "❌ METADATA BACKDATING DETECTED (Tier 4)",
                    [
                        f"Spec: {spec_name}",
                        f"Metadata created_at: {metadata.created_at.isoformat()}",
                        "Policy: New specs cannot claim pre-Phase-17.5 legacy status",
                        "Required: add the spec to the post-cutoff validation path instead",
                    ],
                )
                violations += 1
                continue

            print("   Status: LEGACY (metadata-based, allowlisted)")
            if not original_path.exists():
                print("⚠️  LEGACY SPEC: No ORIGINAL baseline (pre-Phase-17.5)")
                print("   Status: Allowed (legacy exception)")
                legacy_count += 1
                continue

        else:
            print("   Status: POST-CUTOFF (metadata-based)")
            if metadata.validation_level < 3:
                fail(
                    "❌ VALIDATION LEVEL TOO LOW",
                    [
                        f"Spec: {spec_name}",
                        f"Validation Level: {metadata.validation_level}",
                        "Policy: Post-cutoff specs MUST have validation_level >= 3",
                    ],
                )
                violations += 1
                continue

            if not original_path.exists():
                fail(
                    "❌ LEGACY FREEZE VIOLATION (metadata-based)",
                    [
                        f"Spec: {spec_name}",
                        f"Created: {metadata.created_at.isoformat()} (post-cutoff)",
                        "Policy: Post-cutoff specs MUST have ORIGINAL baseline",
                        f"Required: {original_path}",
                    ],
                )
                violations += 1
                continue

        if not validator_path.exists():
            fail(
                "❌ VALIDATION INFRASTRUCTURE REQUIRED",
                [
                    f"Spec: {spec_name}",
                    f"Required: {validator_path}",
                    "Policy: Validated specs MUST have ci_gate_spec_validation.sh",
                ],
            )
            violations += 1
            continue

        print(f"Running validation for: {spec_name}")
        if run_validation(spec_dir):
            print(f"✅ Validation PASS: {spec_name}")
            validated_count += 1
        else:
            fail("❌ Validation FAIL", [f"Spec: {spec_name}", "Policy: Validation failure blocks merge"])
            violations += 1

    print("")
    print("== SPEC VALIDATION SUMMARY (TIER 4.5) ==")
    print(f"Total specs: {spec_count}")
    print(f"Validated: {validated_count}")
    print(f"Legacy (pre-Phase-17.5): {legacy_count}")
    print(f"Violations: {violations}")
    print("⚠️  VALIDATOR TRUST: NOT VERIFIED (Phase-18 pending)")
    print("")

    if violations:
        print("❌ FAIL: Spec validation violations detected")
        return 1

    if validated_count:
        print("✅ All applicable spec validations passed (Tier 4.5: provable integrity)")
    else:
        print("⚠️  All specs are legacy (no post-cutoff validation performed)")
        print("   Note: Future specs MUST use metadata + Level 3 validation")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
