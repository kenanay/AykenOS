#!/usr/bin/env python3
"""Validate local readiness for Phase-12 official closure execution."""

from __future__ import annotations

import argparse
import hashlib
import json
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path
from typing import Any


REQUIRED_GATES = (
    "proof-producer-schema",
    "proof-signature-envelope",
    "proof-bundle-v2-schema",
    "proof-bundle-v2-compat",
    "proof-signature-verify",
    "proof-registry-resolution",
    "proof-key-rotation",
    "proof-verifier-core",
    "proof-trust-policy",
    "proof-verdict-binding",
    "proof-verifier-cli",
    "proof-receipt",
    "proof-audit-ledger",
    "proof-exchange",
    "verifier-authority-resolution",
    "cross-node-parity",
    "proofd-service",
    "proof-multisig-quorum",
    "proof-replay-admission-boundary",
    "proof-replicated-verification-boundary",
)

MANIFEST_HASH_EXCLUDED_FIELDS = ("manifest_sha256", "closure_attestation")
EVIDENCE_ROOT_ALGORITHM = "sha256_path_digest_tree_v1"


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Generate a Phase-12 official closure execution preflight report."
    )
    parser.add_argument(
        "--candidate-dir",
        required=True,
        help="Directory containing the Phase-12 official closure candidate artifacts.",
    )
    parser.add_argument(
        "--output-dir",
        required=True,
        help="Directory to write the preflight report artifacts.",
    )
    parser.add_argument(
        "--repo-root",
        help="Repository root. Defaults to the AykenOS workspace containing this script.",
    )
    parser.add_argument(
        "--expected-current-phase",
        default="10",
        help="Expected CURRENT_PHASE pointer before formal transition.",
    )
    parser.add_argument(
        "--expected-tag",
        default="phase12-official-closure",
        help="Expected dedicated closure tag name.",
    )
    parser.add_argument(
        "--attestor-public-key",
        help="Optional Ed25519 public key used to verify detached closure attestation.",
    )
    parser.add_argument(
        "--remote-ci-workflow",
        default="ci-freeze",
        help="Remote workflow expected to confirm the closure on the same SHA.",
    )
    parser.add_argument(
        "--remote-ci-run-id",
        help="Optional remote CI run identifier if confirmation has already been recorded.",
    )
    parser.add_argument(
        "--fail-on-blockers",
        action="store_true",
        help="Exit non-zero when local execution blockers are detected.",
    )
    return parser.parse_args()


def load_json(path: Path) -> Any:
    with path.open("r", encoding="utf-8") as handle:
        return json.load(handle)


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        while True:
            chunk = handle.read(65536)
            if not chunk:
                break
            digest.update(chunk)
    return digest.hexdigest()


def sha256_bytes(payload: bytes) -> str:
    return hashlib.sha256(payload).hexdigest()


def canonical_json_bytes(value: Any) -> bytes:
    return (
        json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=False).encode("utf-8")
        + b"\n"
    )


def compute_manifest_self_hash(manifest: dict[str, Any]) -> str:
    payload = json.loads(json.dumps(manifest))
    for field in MANIFEST_HASH_EXCLUDED_FIELDS:
        payload.pop(field, None)
    return sha256_bytes(canonical_json_bytes(payload))


def build_tree_root(entries: list[dict[str, Any]]) -> str:
    material = bytearray()
    for entry in sorted(entries, key=lambda item: item["path"]):
        material.extend(entry["path"].encode("utf-8"))
        material.append(0)
        material.extend(entry["sha256"].encode("ascii"))
        material.append(0)
    return sha256_bytes(bytes(material))


def repo_relative(path: Path, repo_root: Path) -> str:
    resolved = path.resolve()
    try:
        return resolved.relative_to(repo_root.resolve()).as_posix()
    except ValueError:
        return resolved.as_posix()


def git_stdout(repo_root: Path, *args: str) -> str:
    proc = subprocess.run(
        ["git", *args],
        cwd=repo_root,
        check=False,
        capture_output=True,
        text=True,
    )
    if proc.returncode != 0:
        raise RuntimeError(proc.stderr.strip() or proc.stdout.strip() or "git command failed")
    return proc.stdout.strip()


def add_blocker(blockers: list[dict[str, str]], code: str, message: str) -> None:
    blockers.append({"code": code, "message": message})


def verify_evidence_index(
    evidence_index: dict[str, Any],
    repo_root: Path,
) -> tuple[list[dict[str, str]], list[dict[str, Any]], list[str], list[str]]:
    blockers: list[dict[str, str]] = []
    verified_entries: list[dict[str, Any]] = []
    missing_gates: list[str] = []
    failing_gates: list[str] = []

    for section in ("report_artifacts", "gate_reports", "meta_artifacts"):
        for entry in evidence_index.get(section, []):
            path = repo_root / entry["path"]
            if not path.is_file():
                add_blocker(
                    blockers,
                    "EVIDENCE_ENTRY_MISSING",
                    f"indexed artifact is missing: {entry['path']}",
                )
                continue

            actual_sha256 = sha256_file(path)
            if actual_sha256 != entry.get("sha256"):
                add_blocker(
                    blockers,
                    "EVIDENCE_ENTRY_DIGEST_MISMATCH",
                    f"indexed artifact digest mismatch: {entry['path']}",
                )
                continue

            actual_size = path.stat().st_size
            if actual_size != entry.get("size_bytes"):
                add_blocker(
                    blockers,
                    "EVIDENCE_ENTRY_SIZE_MISMATCH",
                    f"indexed artifact size mismatch: {entry['path']}",
                )
                continue

            verified_entries.append(
                {
                    "path": entry["path"],
                    "sha256": actual_sha256,
                    "size_bytes": actual_size,
                }
            )

    gate_entries = {entry.get("gate"): entry for entry in evidence_index.get("gate_reports", [])}
    for gate in REQUIRED_GATES:
        entry = gate_entries.get(gate)
        if entry is None:
            missing_gates.append(gate)
            continue
        if str(entry.get("verdict")) != "PASS":
            failing_gates.append(gate)

    if missing_gates:
        add_blocker(
            blockers,
            "REQUIRED_GATES_MISSING",
            "required gate reports missing from evidence index: " + ", ".join(missing_gates),
        )
    if failing_gates:
        add_blocker(
            blockers,
            "REQUIRED_GATES_NOT_PASS",
            "required gate verdicts are not PASS: " + ", ".join(failing_gates),
        )

    return blockers, verified_entries, missing_gates, failing_gates


def verify_attestation(
    manifest: dict[str, Any],
    repo_root: Path,
    attestor_public_key: str | None,
) -> tuple[list[dict[str, str]], dict[str, Any]]:
    blockers: list[dict[str, str]] = []
    attestation = manifest.get("closure_attestation", {})
    state = attestation.get("attestation_state")
    result = {
        "attestation_state": state,
        "attestation_verified": False,
    }

    if state != "SIGNED":
        add_blocker(
            blockers,
            "ATTESTATION_UNSIGNED",
            "closure candidate is not signed with real attestor material",
        )
        return blockers, result

    payload_path_value = attestation.get("payload_path")
    attestation_path_value = attestation.get("attestation_path")
    if not payload_path_value or not attestation_path_value:
        add_blocker(
            blockers,
            "ATTESTATION_PATHS_MISSING",
            "signed closure candidate is missing detached attestation paths",
        )
        return blockers, result

    payload_path = repo_root / payload_path_value
    attestation_path = repo_root / attestation_path_value
    if not payload_path.is_file() or not attestation_path.is_file():
        add_blocker(
            blockers,
            "ATTESTATION_FILES_MISSING",
            "detached attestation payload or signature file is missing",
        )
        return blockers, result

    if not attestor_public_key:
        add_blocker(
            blockers,
            "ATTESTATION_PUBLIC_KEY_MISSING",
            "detached attestation exists but no public key was provided for verification",
        )
        return blockers, result

    cmd = [
        "cargo",
        "run",
        "--quiet",
        "--manifest-path",
        str(repo_root / "ayken-core" / "Cargo.toml"),
        "-p",
        "proof-verifier",
        "--bin",
        "closure-attest",
        "--",
        "verify-json",
        "--payload",
        str(payload_path),
        "--attestation",
        str(attestation_path),
        "--public-key",
        attestor_public_key,
    ]
    proc = subprocess.run(
        cmd,
        cwd=repo_root,
        check=False,
        capture_output=True,
        text=True,
    )
    if proc.returncode != 0:
        add_blocker(
            blockers,
            "ATTESTATION_VERIFICATION_FAILED",
            proc.stderr.strip() or proc.stdout.strip() or "detached attestation verification failed",
        )
        return blockers, result

    result["attestation_verified"] = True
    result["payload_path"] = payload_path_value
    result["attestation_path"] = attestation_path_value
    return blockers, result


def collect_repo_state(
    repo_root: Path,
    manifest: dict[str, Any],
    expected_current_phase: str,
    expected_tag: str,
) -> tuple[list[dict[str, str]], dict[str, Any]]:
    blockers: list[dict[str, str]] = []
    current_phase_file = repo_root / "docs" / "roadmap" / "CURRENT_PHASE"
    current_phase_raw = current_phase_file.read_text(encoding="utf-8").strip()
    current_phase_value = current_phase_raw.split("=", 1)[1] if "=" in current_phase_raw else current_phase_raw

    head_commit = git_stdout(repo_root, "rev-parse", "HEAD")
    status_lines = [
        line for line in git_stdout(repo_root, "status", "--short", "--untracked-files=all").splitlines() if line
    ]
    worktree_clean = not status_lines
    manifest_git_sha = str(manifest.get("run", {}).get("git_sha", ""))
    head_matches_manifest_git_sha = head_commit == manifest_git_sha

    if current_phase_value != expected_current_phase:
        add_blocker(
            blockers,
            "CURRENT_PHASE_MISMATCH",
            f"CURRENT_PHASE={current_phase_value} but expected {expected_current_phase} before transition",
        )
    if current_phase_value != str(manifest.get("current_phase_pointer", "")):
        add_blocker(
            blockers,
            "MANIFEST_PHASE_POINTER_MISMATCH",
            "manifest current_phase_pointer does not match docs/roadmap/CURRENT_PHASE",
        )
    if not worktree_clean:
        add_blocker(
            blockers,
            "WORKTREE_DIRTY",
            f"git worktree has {len(status_lines)} dirty entries; official closure requires clean git state",
        )
    if not head_matches_manifest_git_sha:
        add_blocker(
            blockers,
            "HEAD_SHA_MISMATCH",
            f"HEAD {head_commit} does not match closure evidence SHA {manifest_git_sha}",
        )

    tag_target = None
    tag_exists = False
    tag_points_to_head = False
    proc = subprocess.run(
        ["git", "rev-parse", "-q", "--verify", f"refs/tags/{expected_tag}^{{}}"],
        cwd=repo_root,
        check=False,
        capture_output=True,
        text=True,
    )
    if proc.returncode == 0:
        tag_exists = True
        tag_target = proc.stdout.strip()
        tag_points_to_head = tag_target == head_commit
        if not tag_points_to_head:
            add_blocker(
                blockers,
                "CLOSURE_TAG_CONFLICT",
                f"tag {expected_tag} already exists but points to {tag_target} instead of HEAD {head_commit}",
            )

    state = {
        "current_phase": current_phase_value,
        "expected_current_phase": expected_current_phase,
        "head_commit": head_commit,
        "manifest_git_sha": manifest_git_sha,
        "head_matches_manifest_git_sha": head_matches_manifest_git_sha,
        "worktree_clean": worktree_clean,
        "dirty_entries": status_lines[:50],
        "dirty_entry_count": len(status_lines),
        "expected_tag": expected_tag,
        "tag_exists": tag_exists,
        "tag_target": tag_target,
        "tag_points_to_head": tag_points_to_head,
    }
    return blockers, state


def write_json(path: Path, payload: Any) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("w", encoding="utf-8") as handle:
        json.dump(payload, handle, indent=2, sort_keys=True)
        handle.write("\n")


def write_summary_note(path: Path, report: dict[str, Any], repo_root: Path) -> None:
    lines = [
        "# Phase-12 Official Closure Preflight",
        "",
        f"- Generated at: `{report['generated_at_utc']}`",
        f"- Local execution state: `{report['local_execution_state']}`",
        f"- Official closure state: `{report['official_closure_state']}`",
        f"- Candidate manifest: `{report['candidate']['manifest_path']}`",
        f"- Candidate evidence index: `{report['candidate']['evidence_index_path']}`",
        f"- Head commit: `{report['repo_state']['head_commit']}`",
        f"- Candidate evidence SHA: `{report['repo_state']['manifest_git_sha']}`",
        f"- Worktree clean: `{report['repo_state']['worktree_clean']}`",
        f"- Closure tag exists: `{report['repo_state']['tag_exists']}`",
        f"- Remote workflow: `{report['governance']['remote_ci_workflow']}`",
        f"- Remote run id: `{report['governance']['remote_ci_run_id'] or 'PENDING'}`",
        "",
        "## Blockers",
        "",
    ]
    if report["blockers"]:
        for blocker in report["blockers"]:
            lines.append(f"- `{blocker['code']}`: {blocker['message']}")
    else:
        lines.append("- `none`")

    lines.extend(
        [
            "",
            "## Next Actions",
            "",
        ]
    )
    for action in report["next_actions"]:
        lines.append(f"- `{action}`")

    lines.extend(
        [
            "",
            "## Boundary Invariants",
            "",
        ]
    )
    for invariant in report["candidate"]["boundary_invariants"]:
        lines.append(f"- `{invariant}`")

    path.write_text("\n".join(lines) + "\n", encoding="utf-8")


def main() -> int:
    args = parse_args()
    script_repo_root = Path(__file__).resolve().parents[2]
    repo_root = Path(args.repo_root).resolve() if args.repo_root else script_repo_root
    candidate_dir = Path(args.candidate_dir).resolve()
    output_dir = Path(args.output_dir).resolve()

    manifest_path = candidate_dir / "closure_manifest.json"
    index_path = candidate_dir / "evidence_index.json"
    if not manifest_path.is_file():
        raise SystemExit(f"missing closure manifest: {manifest_path}")
    if not index_path.is_file():
        raise SystemExit(f"missing evidence index: {index_path}")

    manifest = load_json(manifest_path)
    evidence_index = load_json(index_path)
    blockers: list[dict[str, str]] = []

    if manifest.get("closure_class") != "official_closure_candidate":
        add_blocker(
            blockers,
            "CLOSURE_CLASS_MISMATCH",
            f"unexpected closure_class: {manifest.get('closure_class')!r}",
        )
    if manifest.get("closure_state") != "LOCAL_CLOSURE_READY":
        add_blocker(
            blockers,
            "CLOSURE_STATE_MISMATCH",
            f"unexpected closure_state: {manifest.get('closure_state')!r}",
        )
    if str(manifest.get("phase")) != "12":
        add_blocker(
            blockers,
            "PHASE_MISMATCH",
            f"unexpected manifest phase: {manifest.get('phase')!r}",
        )
    if manifest.get("recommended_tag") != args.expected_tag:
        add_blocker(
            blockers,
            "RECOMMENDED_TAG_MISMATCH",
            f"manifest recommends {manifest.get('recommended_tag')!r}, expected {args.expected_tag!r}",
        )
    if manifest.get("evidence_index_path") != repo_relative(index_path, repo_root):
        add_blocker(
            blockers,
            "EVIDENCE_INDEX_PATH_MISMATCH",
            "manifest evidence_index_path does not match candidate directory layout",
        )

    evidence_index_sha256 = sha256_file(index_path)
    if evidence_index_sha256 != manifest.get("evidence_index_sha256"):
        add_blocker(
            blockers,
            "EVIDENCE_INDEX_SHA256_MISMATCH",
            "manifest evidence_index_sha256 does not match the evidence index content",
        )

    manifest_self_hash = compute_manifest_self_hash(manifest)
    if manifest_self_hash != manifest.get("manifest_sha256"):
        add_blocker(
            blockers,
            "MANIFEST_SELF_HASH_MISMATCH",
            "manifest_sha256 does not match the canonical semantic manifest hash",
        )

    evidence_blockers, verified_entries, missing_gates, failing_gates = verify_evidence_index(
        evidence_index,
        repo_root,
    )
    blockers.extend(evidence_blockers)

    evidence_root_hash = build_tree_root(verified_entries)
    if evidence_index.get("evidence_root_algorithm") != EVIDENCE_ROOT_ALGORITHM:
        add_blocker(
            blockers,
            "EVIDENCE_ROOT_ALGORITHM_MISMATCH",
            f"unexpected evidence_root_algorithm: {evidence_index.get('evidence_root_algorithm')!r}",
        )
    if evidence_root_hash != evidence_index.get("evidence_root_hash"):
        add_blocker(
            blockers,
            "EVIDENCE_ROOT_HASH_INDEX_MISMATCH",
            "evidence_index evidence_root_hash does not match the verified evidence tree",
        )
    if evidence_root_hash != manifest.get("evidence_root_hash"):
        add_blocker(
            blockers,
            "EVIDENCE_ROOT_HASH_MANIFEST_MISMATCH",
            "manifest evidence_root_hash does not match the verified evidence tree",
        )

    attestation_blockers, attestation_state = verify_attestation(
        manifest,
        repo_root,
        args.attestor_public_key,
    )
    blockers.extend(attestation_blockers)

    repo_blockers, repo_state = collect_repo_state(
        repo_root,
        manifest,
        args.expected_current_phase,
        args.expected_tag,
    )
    blockers.extend(repo_blockers)

    if blockers:
        local_execution_state = "BLOCKED"
        official_closure_state = "BLOCKED"
    else:
        tag_exists = repo_state["tag_exists"]
        remote_run_id = args.remote_ci_run_id
        if not tag_exists:
            local_execution_state = "READY_FOR_TAG"
            official_closure_state = "PENDING_CLOSURE_TAG"
        elif not remote_run_id:
            local_execution_state = "READY_FOR_REMOTE_CONFIRMATION"
            official_closure_state = "PENDING_REMOTE_CONFIRMATION"
        else:
            local_execution_state = "READY_FOR_FORMAL_PHASE_TRANSITION"
            official_closure_state = "PENDING_FORMAL_PHASE_TRANSITION"

    next_actions: list[str] = []
    blocker_codes = {blocker["code"] for blocker in blockers}
    if "ATTESTATION_UNSIGNED" in blocker_codes or "ATTESTATION_PUBLIC_KEY_MISSING" in blocker_codes:
        next_actions.append("regenerate_closure_candidate_with_real_attestor_material")
    if "WORKTREE_DIRTY" in blocker_codes:
        next_actions.append("clean_git_worktree_before_official_closure")
    if "HEAD_SHA_MISMATCH" in blocker_codes:
        next_actions.append("regenerate_candidate_on_current_head_or_rewind_to_evidence_sha")
    if not repo_state["tag_exists"] and not blockers:
        next_actions.append("create_dedicated_closure_tag")
    if repo_state["tag_exists"] and not args.remote_ci_run_id and not blockers:
        next_actions.append("obtain_remote_ci_freeze_confirmation_on_tagged_sha")
    if args.remote_ci_run_id and not blockers:
        next_actions.append("execute_formal_phase_transition_workflow")
    if not next_actions:
        next_actions.append("resolve_local_blockers_before_governance_follow_through")

    generated_at = datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")
    report = {
        "generated_at_utc": generated_at,
        "preflight_version": 1,
        "candidate": {
            "manifest_path": repo_relative(manifest_path, repo_root),
            "evidence_index_path": repo_relative(index_path, repo_root),
            "manifest_sha256_verified": manifest_self_hash == manifest.get("manifest_sha256"),
            "evidence_index_sha256_verified": evidence_index_sha256 == manifest.get("evidence_index_sha256"),
            "evidence_root_hash_verified": (
                evidence_root_hash == evidence_index.get("evidence_root_hash")
                and evidence_root_hash == manifest.get("evidence_root_hash")
            ),
            "required_gate_count": len(REQUIRED_GATES),
            "missing_gates": missing_gates,
            "failing_gates": failing_gates,
            "boundary_invariants": manifest.get("boundary_invariants", []),
            "attestation": attestation_state,
        },
        "repo_state": repo_state,
        "governance": {
            "remote_ci_workflow": args.remote_ci_workflow,
            "remote_ci_run_id": args.remote_ci_run_id,
            "phase_transition_required": True,
        },
        "local_execution_state": local_execution_state,
        "official_closure_state": official_closure_state,
        "blockers": blockers,
        "next_actions": next_actions,
    }

    output_dir.mkdir(parents=True, exist_ok=True)
    report_path = output_dir / "preflight_report.json"
    readme_path = output_dir / "README.md"
    write_json(report_path, report)
    write_summary_note(readme_path, report, repo_root)

    if args.fail_on_blockers and blockers:
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
