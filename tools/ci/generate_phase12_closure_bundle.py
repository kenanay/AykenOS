#!/usr/bin/env python3
"""Generate a canonical Phase-12 official-closure candidate bundle."""

from __future__ import annotations

import argparse
import hashlib
import json
import subprocess
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

PREREQUISITES_REMAINING = (
    "mint_dedicated_closure_tag",
    "obtain_remote_official_confirmation",
    "execute_formal_phase_transition",
)

BOUNDARY_INVARIANTS = (
    "proofd != authority_surface",
    "parity != consensus",
    "system computes truth; it does not choose truth",
)

EVIDENCE_ROOT_ALGORITHM = "sha256_path_digest_tree_v1"
MANIFEST_DIGEST_ALGORITHM = "sha256"
MANIFEST_HASH_EXCLUDED_FIELDS = ("manifest_sha256", "closure_attestation")


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Generate Phase-12 official closure candidate artifacts."
    )
    parser.add_argument(
        "--run-dir",
        required=True,
        help="Evidence run directory for the local Phase-12C closure pass.",
    )
    parser.add_argument(
        "--output-dir",
        required=True,
        help="Output directory for closure manifest, evidence index, and summary note.",
    )
    parser.add_argument(
        "--recommended-tag",
        default="phase12-official-closure",
        help="Recommended dedicated closure tag name.",
    )
    parser.add_argument(
        "--attestor-node-id",
        help="Optional detached attestor node identifier for closure manifest attestation.",
    )
    parser.add_argument(
        "--attestor-key-id",
        help="Optional detached attestor key identifier for closure manifest attestation.",
    )
    parser.add_argument(
        "--attestor-private-key",
        help="Optional base64 Ed25519 private key for closure manifest attestation.",
    )
    parser.add_argument(
        "--attested-at-utc",
        help="Optional attestation timestamp. Defaults to bundle generation time.",
    )
    return parser.parse_args()


def load_json(path: Path) -> Any:
    with path.open("r", encoding="utf-8") as handle:
        return json.load(handle)


def load_text(path: Path) -> str:
    return path.read_text(encoding="utf-8", errors="replace").strip()


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


def repo_relative(path: Path, repo_root: Path) -> str:
    resolved = path.resolve()
    try:
        return resolved.relative_to(repo_root.resolve()).as_posix()
    except ValueError:
        return resolved.as_posix()


def build_file_entry(path: Path, repo_root: Path, extra: dict[str, Any] | None = None) -> dict[str, Any]:
    entry = {
        "path": repo_relative(path, repo_root),
        "sha256": sha256_file(path),
        "size_bytes": path.stat().st_size,
    }
    if extra:
        entry.update(extra)
    return entry


def build_tree_root(entries: list[dict[str, Any]]) -> str:
    material = bytearray()
    for entry in sorted(entries, key=lambda item: item["path"]):
        material.extend(entry["path"].encode("utf-8"))
        material.append(0)
        material.extend(entry["sha256"].encode("ascii"))
        material.append(0)
    return sha256_bytes(bytes(material))


def compute_manifest_self_hash(manifest: dict[str, Any]) -> str:
    payload = json.loads(json.dumps(manifest))
    for field in MANIFEST_HASH_EXCLUDED_FIELDS:
        payload.pop(field, None)
    return sha256_bytes(canonical_json_bytes(payload))


def read_current_phase(repo_root: Path) -> str:
    phase_file = repo_root / "docs" / "roadmap" / "CURRENT_PHASE"
    if not phase_file.is_file():
        return "UNKNOWN"
    raw = load_text(phase_file)
    if "=" not in raw:
        return raw
    return raw.split("=", 1)[1]


def validate_summary(summary: dict[str, Any], run_dir: Path) -> None:
    if summary.get("verdict") != "PASS":
        raise SystemExit(
            f"Phase-12 closure bundle requires PASS summary, got {summary.get('verdict')!r} "
            f"for {run_dir}"
        )

    gates = summary.get("gates", {})
    missing = [gate for gate in REQUIRED_GATES if gate not in gates]
    if missing:
        raise SystemExit(
            "Phase-12 closure bundle is missing required gates: " + ", ".join(missing)
        )

    failing = [
        gate
        for gate in REQUIRED_GATES
        if str(gates[gate].get("verdict")) != "PASS"
    ]
    if failing:
        raise SystemExit(
            "Phase-12 closure bundle requires all required gates to PASS; failing gates: "
            + ", ".join(failing)
        )


def collect_gate_reports(run_dir: Path, repo_root: Path, summary: dict[str, Any]) -> list[dict[str, Any]]:
    gate_entries: list[dict[str, Any]] = []
    for gate_name in REQUIRED_GATES:
        report_path = run_dir / "gates" / gate_name / "report.json"
        if not report_path.is_file():
            raise SystemExit(f"Missing gate report: {report_path}")
        gate_summary = summary["gates"][gate_name]
        gate_entries.append(
            build_file_entry(
                report_path,
                repo_root,
                {
                    "gate": gate_name,
                    "verdict": gate_summary.get("verdict"),
                    "violations_count": gate_summary.get("violations_count", 0),
                },
            )
        )
    return gate_entries


def collect_report_artifacts(run_dir: Path, repo_root: Path) -> list[dict[str, Any]]:
    reports_dir = run_dir / "reports"
    report_entries: list[dict[str, Any]] = []
    for path in sorted(reports_dir.glob("*")):
        if path.is_file():
            report_entries.append(build_file_entry(path, repo_root))
    return report_entries


def collect_meta_artifacts(run_dir: Path, repo_root: Path) -> list[dict[str, Any]]:
    meta_dir = run_dir / "meta"
    meta_entries: list[dict[str, Any]] = []
    for path in sorted(meta_dir.glob("*")):
        if path.is_file():
            meta_entries.append(build_file_entry(path, repo_root))
    return meta_entries


def write_json(path: Path, payload: Any) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("w", encoding="utf-8") as handle:
        json.dump(payload, handle, indent=2, sort_keys=True)
        handle.write("\n")


def write_digest_file(path: Path, digest_value: str, relative_target: str) -> None:
    path.write_text(f"{digest_value}  {relative_target}\n", encoding="utf-8")


def validate_attestation_args(args: argparse.Namespace) -> bool:
    fields = (
        args.attestor_node_id,
        args.attestor_key_id,
        args.attestor_private_key,
    )
    provided = [field is not None for field in fields]
    if any(provided) and not all(provided):
        raise SystemExit(
            "closure manifest attestation requires --attestor-node-id, "
            "--attestor-key-id, and --attestor-private-key together"
        )
    return all(provided)


def maybe_generate_attestation(
    args: argparse.Namespace,
    repo_root: Path,
    output_dir: Path,
    manifest_path: Path,
    evidence_root_hash: str,
    manifest: dict[str, Any],
) -> dict[str, Any]:
    if not validate_attestation_args(args):
        return {
            "attestation_state": "UNSIGNED",
            "reason": "attestor_key_material_not_provided",
        }

    attested_at = args.attested_at_utc or manifest["generated_at_utc"]
    payload = {
        "attestation_version": 1,
        "artifact_kind": "phase12_closure_manifest",
        "phase": manifest["phase"],
        "closure_state": manifest["closure_state"],
        "current_phase_pointer": manifest["current_phase_pointer"],
        "recommended_tag": manifest["recommended_tag"],
        "manifest_path": repo_relative(manifest_path, repo_root),
        "manifest_sha256": manifest["manifest_sha256"],
        "evidence_root_hash": evidence_root_hash,
        "evidence_root_algorithm": EVIDENCE_ROOT_ALGORITHM,
        "evidence_run_dir": manifest["run"]["evidence_run_dir"],
        "attestor_node_id": args.attestor_node_id,
        "attestor_key_id": args.attestor_key_id,
        "attested_at_utc": attested_at,
    }

    payload_path = output_dir / "closure_manifest.attestation.payload.json"
    attestation_path = output_dir / "closure_manifest.attestation.json"
    write_json(payload_path, payload)

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
        "sign-json",
        "--payload",
        str(payload_path),
        "--output",
        str(attestation_path),
        "--attestor-node-id",
        str(args.attestor_node_id),
        "--attestor-key-id",
        str(args.attestor_key_id),
        "--private-key",
        str(args.attestor_private_key),
        "--attested-at-utc",
        attested_at,
    ]
    subprocess.run(cmd, check=True, cwd=repo_root)

    write_digest_file(
        output_dir / "closure_manifest.attestation.payload.sha256",
        sha256_file(payload_path),
        repo_relative(payload_path, repo_root),
    )
    write_digest_file(
        output_dir / "closure_manifest.attestation.sha256",
        sha256_file(attestation_path),
        repo_relative(attestation_path, repo_root),
    )
    return {
        "attestation_state": "SIGNED",
        "payload_path": repo_relative(payload_path, repo_root),
        "attestation_path": repo_relative(attestation_path, repo_root),
        "attestor_node_id": args.attestor_node_id,
        "attestor_key_id": args.attestor_key_id,
    }


def write_summary_note(
    path: Path,
    manifest: dict[str, Any],
    evidence_index: dict[str, Any],
    repo_root: Path,
) -> None:
    run = manifest["run"]
    gate_names = ", ".join(REQUIRED_GATES)
    lines = [
        "# Phase-12 Official Closure Candidate",
        "",
        f"- Generated at: `{manifest['generated_at_utc']}`",
        f"- Closure state: `{manifest['closure_state']}`",
        f"- Current phase pointer: `{manifest['current_phase_pointer']}`",
        f"- Recommended dedicated tag: `{manifest['recommended_tag']}`",
        f"- Evidence run: `{run['reported_run_id']}`",
        f"- Evidence directory: `{run['evidence_run_dir']}`",
        f"- Evidence git SHA: `{run['git_sha']}`",
        f"- Manifest digest: `{manifest['manifest_sha256']}`",
        f"- Evidence root hash: `{manifest['evidence_root_hash']}`",
        f"- Attestation state: `{manifest['closure_attestation']['attestation_state']}`",
        "",
        "## Required Gates",
        "",
        f"`{gate_names}`",
        "",
        "## Generated Artifacts",
        "",
        f"- Closure manifest: `{repo_relative(path.parent / 'closure_manifest.json', repo_root)}`",
        f"- Closure manifest digest: `{repo_relative(path.parent / 'closure_manifest.sha256', repo_root)}`",
        f"- Evidence index: `{repo_relative(path.parent / 'evidence_index.json', repo_root)}`",
        f"- Evidence index digest: `{repo_relative(path.parent / 'evidence_index.sha256', repo_root)}`",
        f"- Indexed report artifacts: `{len(evidence_index['report_artifacts'])}`",
        f"- Indexed gate reports: `{len(evidence_index['gate_reports'])}`",
        "",
        "## Remaining Governance Steps",
        "",
    ]
    for item in manifest["official_closure_prerequisites_remaining"]:
        lines.append(f"- `{item}`")
    lines.extend(
        [
            "",
            "## Boundary Invariants",
            "",
        ]
    )
    for invariant in manifest["boundary_invariants"]:
        lines.append(f"- `{invariant}`")
    if manifest["closure_attestation"]["attestation_state"] == "SIGNED":
        lines.extend(
            [
                "",
                "## Attestation Artifacts",
                "",
                f"- `{manifest['closure_attestation']['payload_path']}`",
                f"- `{manifest['closure_attestation']['attestation_path']}`",
            ]
        )
    path.write_text("\n".join(lines) + "\n", encoding="utf-8")


def main() -> int:
    args = parse_args()
    repo_root = Path(__file__).resolve().parents[2]
    run_dir = Path(args.run_dir).resolve()
    output_dir = Path(args.output_dir).resolve()

    summary_path = run_dir / "reports" / "summary.json"
    if not summary_path.is_file():
        raise SystemExit(f"Missing summary report: {summary_path}")

    summary = load_json(summary_path)
    validate_summary(summary, run_dir)

    run_meta_path = run_dir / "meta" / "run.json"
    git_txt_path = run_dir / "meta" / "git.txt"
    if not run_meta_path.is_file():
        raise SystemExit(f"Missing run metadata: {run_meta_path}")
    if not git_txt_path.is_file():
        raise SystemExit(f"Missing git metadata: {git_txt_path}")

    run_meta = load_json(run_meta_path)
    git_sha = load_text(git_txt_path)
    current_phase_pointer = read_current_phase(repo_root)
    generated_at = datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")

    gate_reports = collect_gate_reports(run_dir, repo_root, summary)
    report_artifacts = collect_report_artifacts(run_dir, repo_root)
    meta_artifacts = collect_meta_artifacts(run_dir, repo_root)
    indexed_artifacts = sorted(
        [*report_artifacts, *gate_reports, *meta_artifacts], key=lambda item: item["path"]
    )
    evidence_root_hash = build_tree_root(indexed_artifacts)

    output_dir.mkdir(parents=True, exist_ok=True)
    manifest_path = output_dir / "closure_manifest.json"
    index_path = output_dir / "evidence_index.json"
    summary_note_path = output_dir / "README.md"

    evidence_index = {
        "generated_at_utc": generated_at,
        "index_version": 1,
        "run": {
            "run_id": summary.get("run_id"),
            "evidence_run_dir": repo_relative(run_dir, repo_root),
            "git_sha": git_sha,
        },
        "evidence_root_algorithm": EVIDENCE_ROOT_ALGORITHM,
        "evidence_root_hash": evidence_root_hash,
        "report_artifacts": report_artifacts,
        "gate_reports": gate_reports,
        "meta_artifacts": meta_artifacts,
    }

    manifest = {
        "boundary_invariants": list(BOUNDARY_INVARIANTS),
        "closure_class": "official_closure_candidate",
        "closure_state": "LOCAL_CLOSURE_READY",
        "closure_attestation": {
            "attestation_state": "UNSIGNED",
            "reason": "pending_generation",
        },
        "current_phase_pointer": current_phase_pointer,
        "evidence_index_path": repo_relative(index_path, repo_root),
        "evidence_index_sha256": "",
        "evidence_root_algorithm": EVIDENCE_ROOT_ALGORITHM,
        "evidence_root_hash": evidence_root_hash,
        "gate_policy": {
            "all_required_gates_passed": True,
            "required_gate_count": len(REQUIRED_GATES),
            "required_gates": list(REQUIRED_GATES),
        },
        "generated_at_utc": generated_at,
        "manifest_hash_excluded_fields": list(MANIFEST_HASH_EXCLUDED_FIELDS),
        "manifest_digest_algorithm": MANIFEST_DIGEST_ALGORITHM,
        "manifest_sha256": "",
        "manifest_version": 1,
        "official_closure_prerequisites_remaining": list(PREREQUISITES_REMAINING),
        "phase": "12",
        "recommended_tag": args.recommended_tag,
        "run": {
            "evidence_run_dir": repo_relative(run_dir, repo_root),
            "git_sha": git_sha,
            "reported_run_id": summary.get("run_id", run_meta.get("run_id", "")),
            "run_dir_name": run_dir.name,
            "summary_path": repo_relative(summary_path, repo_root),
            "time_utc": summary.get("time_utc", run_meta.get("time_utc", "")),
        },
        "summary_note_path": repo_relative(summary_note_path, repo_root),
    }

    write_json(index_path, evidence_index)
    evidence_index_sha256 = sha256_file(index_path)
    write_digest_file(
        output_dir / "evidence_index.sha256",
        evidence_index_sha256,
        repo_relative(index_path, repo_root),
    )

    manifest["evidence_index_sha256"] = evidence_index_sha256
    manifest["manifest_sha256"] = compute_manifest_self_hash(manifest)
    manifest["closure_attestation"] = maybe_generate_attestation(
        args,
        repo_root,
        output_dir,
        manifest_path,
        evidence_root_hash,
        manifest,
    )
    write_json(manifest_path, manifest)
    write_digest_file(
        output_dir / "closure_manifest.sha256",
        sha256_file(manifest_path),
        repo_relative(manifest_path, repo_root),
    )
    write_summary_note(summary_note_path, manifest, evidence_index, repo_root)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
