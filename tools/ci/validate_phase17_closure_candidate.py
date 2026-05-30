#!/usr/bin/env python3
"""Validate Phase-17 closure-candidate integrity without granting closure."""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
import sys


EXPECTED_STATE = "REMOTE_EVIDENCE_READY_REVIEW_PENDING"
REQUIRED_WORKFLOWS = {
    "ci-freeze",
    "Performance Gate",
    "ci-gate-execution-marker-lifecycle",
    "ci-gate-execution-marker-determinism",
    "ci-gate-execution-public-e2e",
    "ci-gate-execution-worker-completion",
    "ci-gate-execution-timeout-race",
    "ci-gate-phase17-performance-acceptance",
}


def fail(message: str) -> None:
    print(f"phase17-closure-candidate: FAIL ({message})", file=sys.stderr)
    raise SystemExit(1)


def read_json(path: Path) -> dict:
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as exc:
        fail(f"cannot read {path}: {exc}")
    return {}


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def validate_sidecar(candidate_dir: Path, filename: str, sidecar_name: str) -> None:
    subject = candidate_dir / filename
    sidecar = candidate_dir / sidecar_name
    if not subject.exists() or not sidecar.exists():
        fail(f"missing integrity artifact for {filename}")
    parts = sidecar.read_text(encoding="utf-8").strip().split()
    if len(parts) != 2 or parts[1] != filename:
        fail(f"invalid sidecar format for {filename}")
    if parts[0] != digest(subject):
        fail(f"digest mismatch for {filename}")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--candidate-dir",
        default="reports/phase17_official_closure_candidate",
        type=Path,
    )
    parser.add_argument("--expected-subject-sha")
    args = parser.parse_args()

    candidate_dir = args.candidate_dir
    manifest_path = candidate_dir / "closure_manifest.json"
    index_path = candidate_dir / "evidence_index.json"
    manifest = read_json(manifest_path)
    index = read_json(index_path)

    if manifest.get("closure_class") != "official_closure_candidate":
        fail("manifest is not candidate-class")
    if manifest.get("closure_state") != EXPECTED_STATE:
        fail("candidate state is not review-pending")
    if manifest.get("phase18_activation") != "BLOCKED_UNTIL_OFFICIAL_CLOSURE":
        fail("Phase-18 boundary is not fail-closed")

    subject_sha = manifest.get("candidate_subject", {}).get("commit_sha")
    if not subject_sha or subject_sha != index.get("evidence_subject", {}).get("head_sha"):
        fail("manifest/index subject SHA mismatch")
    if args.expected_subject_sha and subject_sha != args.expected_subject_sha:
        fail("unexpected candidate subject SHA")
    if manifest.get("evidence_index_sha256") != digest(index_path):
        fail("manifest evidence index digest mismatch")

    workflows = {}
    for run in index.get("runs", []):
        name = run.get("workflow")
        if name in workflows:
            fail(f"duplicate workflow evidence: {name}")
        workflows[name] = run

    if set(workflows) != REQUIRED_WORKFLOWS:
        missing = sorted(REQUIRED_WORKFLOWS - set(workflows))
        extra = sorted(set(workflows) - REQUIRED_WORKFLOWS)
        fail(f"workflow set mismatch; missing={missing} extra={extra}")
    for name, run in workflows.items():
        if run.get("result") != "PASS" or run.get("head_sha") != subject_sha:
            fail(f"non-PASS or wrong-SHA evidence: {name}")

    validate_sidecar(candidate_dir, "closure_manifest.json", "closure_manifest.sha256")
    validate_sidecar(candidate_dir, "evidence_index.json", "evidence_index.sha256")

    print(
        "phase17-closure-candidate: PASS "
        "(integrity only; official closure and tag remain pending)"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
