#!/usr/bin/env python3
"""Generate a canonical Phase-13 official-closure candidate bundle."""

from __future__ import annotations

import argparse
import hashlib
import json
import subprocess
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

REQUIRED_GATES = (
    "observability-routing-separation",
    "convergence-non-election-boundary",
    "graph-non-authoritative-contract",
    "diagnostics-consumer-non-authoritative-contract",
    "diagnostics-callsite-correlation",
    "verifier-reputation-prohibition",
)

PREREQUISITES_REMAINING = (
    "mint_dedicated_closure_tag",
    "obtain_remote_official_confirmation",
    "execute_formal_phase_transition",
)

BOUNDARY_INVARIANTS = (
    "verified proof != replay admission",
    "replicated verification remains a Phase-13 bridge concern",
    "proofd = verification and diagnostics service",
    "service != authority",
    "parity != consensus",
    "system computes truth; it does not choose truth",
)

WORKSTREAMS_COMPLETED = (
    "service-expansion",
    "verifier-federation",
    "context-propagation",
    "trust-registry-propagation",
    "replicated-verification-boundary",
)

EVIDENCE_ROOT_ALGORITHM = "sha256_path_digest_tree_v1"
MANIFEST_DIGEST_ALGORITHM = "sha256"
MANIFEST_HASH_EXCLUDED_FIELDS = ("manifest_sha256", "closure_attestation")


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Generate Phase-13 official closure candidate artifacts."
    )
    parser.add_argument(
        "--run-dir",
        required=True,
        help="Evidence run directory containing gate reports.",
    )
    parser.add_argument(
        "--output-dir",
        default="reports/phase13_official_closure_candidate",
        help="Output directory for closure artifacts.",
    )
    parser.add_argument(
        "--current-phase-pointer",
        default="13",
        help="Current phase pointer value.",
    )
    return parser.parse_args()


def sha256_file(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as fh:
        for chunk in iter(lambda: fh.read(65536), b""):
            h.update(chunk)
    return h.hexdigest()


def sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def git_sha() -> str:
    try:
        result = subprocess.run(
            ["git", "rev-parse", "HEAD"],
            capture_output=True,
            text=True,
            check=True,
        )
        return result.stdout.strip()
    except Exception:
        return "UNKNOWN"


def collect_gate_reports(run_dir: Path) -> dict[str, Any]:
    gates_dir = run_dir / "gates"
    gate_reports: list[dict[str, Any]] = []
    gates: dict[str, str] = {}

    if not gates_dir.is_dir():
        return {"gate_reports": gate_reports, "gates": gates}

    for report_path in sorted(gates_dir.glob("*/report.json")):
        try:
            report = json.loads(report_path.read_text(encoding="utf-8"))
        except Exception:
            continue
        gate_name = report.get("gate") or report_path.parent.name
        verdict = report.get("verdict", "UNKNOWN")
        gates[gate_name] = verdict
        gate_reports.append(
            {
                "gate": gate_name,
                "path": str(report_path),
                "sha256": sha256_file(report_path),
                "size_bytes": report_path.stat().st_size,
                "verdict": verdict,
                "violations_count": report.get("violations_count", 0),
            }
        )

    return {"gate_reports": gate_reports, "gates": gates}


def build_evidence_root_hash(gate_reports: list[dict[str, Any]]) -> str:
    """Compute a deterministic hash over all gate report paths and their hashes."""
    h = hashlib.sha256()
    for entry in sorted(gate_reports, key=lambda e: e["path"]):
        h.update(f"{entry['path']}:{entry['sha256']}\n".encode())
    return h.hexdigest()


def write_json(path: Path, payload: Any) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("w", encoding="utf-8") as fh:
        json.dump(payload, fh, indent=2, sort_keys=True)
        fh.write("\n")


def main() -> int:
    args = parse_args()
    run_dir = Path(args.run_dir).resolve()
    output_dir = Path(args.output_dir)
    output_dir.mkdir(parents=True, exist_ok=True)

    now_utc = datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")
    head_sha = git_sha()

    # Collect gate reports
    collected = collect_gate_reports(run_dir)
    gate_reports = collected["gate_reports"]
    gates = collected["gates"]

    # Check required gates
    all_required_passed = all(
        gates.get(gate) in ("PASS", "WARN") for gate in REQUIRED_GATES
    )

    # Build evidence root hash
    evidence_root_hash = build_evidence_root_hash(gate_reports)

    # Build evidence index
    evidence_index = {
        "evidence_root_algorithm": EVIDENCE_ROOT_ALGORITHM,
        "evidence_root_hash": evidence_root_hash,
        "gate_reports": gate_reports,
        "generated_at_utc": now_utc,
        "index_version": 1,
        "run": {
            "evidence_run_dir": str(run_dir),
            "git_sha": head_sha,
            "run_id": run_dir.name,
        },
    }

    evidence_index_path = output_dir / "evidence_index.json"
    write_json(evidence_index_path, evidence_index)
    evidence_index_sha = sha256_file(evidence_index_path)
    (output_dir / "evidence_index.sha256").write_text(
        f"{evidence_index_sha}  evidence_index.json\n"
    )

    # Build closure manifest
    manifest: dict[str, Any] = {
        "boundary_invariants": list(BOUNDARY_INVARIANTS),
        "closure_attestation": {
            "attestation_state": "UNSIGNED",
            "reason": "attestor_key_material_not_provided",
        },
        "closure_class": "official_closure_candidate",
        "closure_state": "LOCAL_CLOSURE_READY",
        "current_phase_pointer": args.current_phase_pointer,
        "evidence_index_path": str(evidence_index_path),
        "evidence_index_sha256": evidence_index_sha,
        "evidence_root_algorithm": EVIDENCE_ROOT_ALGORITHM,
        "evidence_root_hash": evidence_root_hash,
        "gate_policy": {
            "all_required_gates_passed": all_required_passed,
            "required_gate_count": len(REQUIRED_GATES),
            "required_gates": list(REQUIRED_GATES),
        },
        "generated_at_utc": now_utc,
        "manifest_digest_algorithm": MANIFEST_DIGEST_ALGORITHM,
        "manifest_hash_excluded_fields": list(MANIFEST_HASH_EXCLUDED_FIELDS),
        "official_closure_prerequisites_remaining": list(PREREQUISITES_REMAINING),
        "phase": "13",
        "recommended_tag": "phase13-official-closure",
        "run": {
            "evidence_run_dir": str(run_dir),
            "git_sha": head_sha,
            "run_id": run_dir.name,
        },
        "summary_note_path": str(output_dir / "README.md"),
        "workstreams_completed": list(WORKSTREAMS_COMPLETED),
    }

    # Compute manifest hash (excluding excluded fields)
    manifest_for_hash = {
        k: v for k, v in manifest.items() if k not in MANIFEST_HASH_EXCLUDED_FIELDS
    }
    manifest_sha = sha256_bytes(
        json.dumps(manifest_for_hash, sort_keys=True).encode()
    )
    manifest["manifest_sha256"] = manifest_sha

    manifest_path = output_dir / "closure_manifest.json"
    write_json(manifest_path, manifest)
    (output_dir / "closure_manifest.sha256").write_text(
        f"{manifest_sha}  closure_manifest.json\n"
    )

    # Write README
    readme = f"""# Phase-13 Official Closure Candidate

- Generated at: `{now_utc}`
- Closure state: `LOCAL_CLOSURE_READY`
- Current phase pointer: `{args.current_phase_pointer}`
- Recommended dedicated tag: `phase13-official-closure`
- Evidence run: `{run_dir.name}`
- Evidence directory: `{run_dir}`
- Evidence git SHA: `{head_sha}`
- Manifest digest: `{manifest_sha}`
- Evidence root hash: `{evidence_root_hash}`
- Attestation state: `UNSIGNED`

## Required Gates

`{", ".join(REQUIRED_GATES)}`

## Workstreams Completed

`{", ".join(WORKSTREAMS_COMPLETED)}`

## Generated Artifacts

- Closure manifest: `{output_dir}/closure_manifest.json`
- Closure manifest digest: `{output_dir}/closure_manifest.sha256`
- Evidence index: `{output_dir}/evidence_index.json`
- Evidence index digest: `{output_dir}/evidence_index.sha256`
- Indexed gate reports: `{len(gate_reports)}`

## Remaining Governance Steps

- `mint_dedicated_closure_tag`
- `obtain_remote_official_confirmation`
- `execute_formal_phase_transition`

## Boundary Invariants

{chr(10).join(f"- `{inv}`" for inv in BOUNDARY_INVARIANTS)}
"""
    (output_dir / "README.md").write_text(readme, encoding="utf-8")

    print(f"closure_state: LOCAL_CLOSURE_READY")
    print(f"all_required_gates_passed: {all_required_passed}")
    print(f"manifest: {manifest_path}")
    print(f"evidence_index: {evidence_index_path}")
    print(f"manifest_sha256: {manifest_sha}")
    print(f"evidence_root_hash: {evidence_root_hash}")

    return 0 if all_required_passed else 2


if __name__ == "__main__":
    raise SystemExit(main())
