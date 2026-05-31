#!/usr/bin/env python3
"""Validate Phase-17 closure-candidate integrity without granting closure."""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
import sys


REVIEW_PENDING_STATE = "REMOTE_EVIDENCE_READY_REVIEW_PENDING"
DECISION_READY_STATE = "OFFICIAL_CLOSURE_DECISION_READY_TAG_PENDING"
CONFIRMED_STATE = "OFFICIAL_CLOSURE_CONFIRMED"
ALLOWED_STATES = {REVIEW_PENDING_STATE, DECISION_READY_STATE, CONFIRMED_STATE}
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


def validate_index_artifact(
    closure_index: dict, artifact_key: str, expected_filename: str, expected_digest: str
) -> None:
    artifact = closure_index.get("artifacts", {}).get(artifact_key, {})
    if not artifact:
        fail(f"missing closure-index artifact entry: {artifact_key}")
    if Path(artifact.get("path", "")).name != expected_filename:
        fail(f"unexpected closure-index artifact path: {artifact_key}")
    if artifact.get("sha256") != expected_digest:
        fail(f"closure-index artifact digest mismatch: {artifact_key}")


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

    closure_class = manifest.get("closure_class")
    state = manifest.get("closure_state")
    if state not in ALLOWED_STATES:
        fail("candidate state is not supported")
    if state == CONFIRMED_STATE:
        if closure_class != "official_closure":
            fail("confirmed manifest is not official-closure class")
        if manifest.get("phase18_activation") != "BLOCKED_UNTIL_SEPARATE_TRANSITION_DECISION":
            fail("Phase-18 boundary is not transition-blocked")
    elif closure_class != "official_closure_candidate":
        fail("manifest is not candidate-class")
    elif manifest.get("phase18_activation") != "BLOCKED_UNTIL_OFFICIAL_CLOSURE":
        fail("Phase-18 boundary is not fail-closed")

    subject_sha = manifest.get("candidate_subject", {}).get("commit_sha")
    if not subject_sha or subject_sha != index.get("evidence_subject", {}).get("head_sha"):
        fail("manifest/index subject SHA mismatch")
    if args.expected_subject_sha and subject_sha != args.expected_subject_sha:
        fail("unexpected candidate subject SHA")
    if manifest.get("evidence_index_sha256") != digest(index_path):
        fail("manifest evidence index digest mismatch")
    if index.get("authority_status") and index.get("authority_status") != state:
        fail("manifest/evidence authority state mismatch")

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

    if state in {DECISION_READY_STATE, CONFIRMED_STATE}:
        decision_path = candidate_dir / "closure_decision_record.json"
        closure_index_path = candidate_dir / "closure_index.json"
        decision = read_json(decision_path)
        closure_index = read_json(closure_index_path)
        tag = decision.get("official_tag", {})
        expected_decision_state = (
            "TAG_MINTED_AND_VERIFIED"
            if state == CONFIRMED_STATE
            else "APPROVED_FOR_EXACT_SHA_TAG_MINTING"
        )
        if decision.get("decision_state") != expected_decision_state:
            fail("decision record state does not match closure state")
        if tag.get("subject_sha") != subject_sha:
            fail("decision tag subject does not match candidate subject")
        if closure_index.get("closure_state") != state:
            fail("closure index state does not match manifest")
        if closure_index.get("tag_subject_sha") != subject_sha:
            fail("closure index tag subject does not match candidate subject")
        tag_verification = closure_index.get("tag_verification", {})
        expected_tag_state = (
            "VERIFIED_REMOTE_TAG_TARGET"
            if state == CONFIRMED_STATE
            else "PENDING_REMOTE_TAG_MINT"
        )
        if tag_verification.get("state") != expected_tag_state:
            fail("closure index tag verification state mismatch")
        if tag_verification.get("required_tag_target") != subject_sha:
            fail("closure index tag verification target mismatch")
        if state == CONFIRMED_STATE:
            if tag_verification.get("verified_target_sha") != subject_sha:
                fail("verified tag target mismatch")
            if tag.get("verified_target_sha") != subject_sha:
                fail("decision verified tag target mismatch")
        if closure_index.get("phase18_activation") != "NOT_ACTIVATED_BY_THIS_CLOSURE_INDEX":
            fail("closure index incorrectly activates Phase-18")
        validate_index_artifact(
            closure_index,
            "closure_manifest",
            "closure_manifest.json",
            digest(manifest_path),
        )
        validate_index_artifact(
            closure_index,
            "evidence_index",
            "evidence_index.json",
            digest(index_path),
        )
        validate_index_artifact(
            closure_index,
            "closure_decision_record",
            "closure_decision_record.json",
            digest(decision_path),
        )

    print(
        "phase17-closure-candidate: PASS "
        "(integrity only; Phase-18 activation remains separate)"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
