#!/usr/bin/env python3
"""Tests for generate_phase12_official_closure_preflight.py."""

from __future__ import annotations

import hashlib
import json
import subprocess
import tempfile
import unittest
from pathlib import Path


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


def sha256_file(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def sha256_bytes(payload: bytes) -> str:
    return hashlib.sha256(payload).hexdigest()


def canonical_json_bytes(value: object) -> bytes:
    return (
        json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=False).encode("utf-8")
        + b"\n"
    )


def build_tree_root(entries: list[dict[str, object]]) -> str:
    material = bytearray()
    for entry in sorted(entries, key=lambda item: str(item["path"])):
        material.extend(str(entry["path"]).encode("utf-8"))
        material.append(0)
        material.extend(str(entry["sha256"]).encode("ascii"))
        material.append(0)
    return sha256_bytes(bytes(material))


def compute_manifest_self_hash(manifest: dict[str, object]) -> str:
    payload = json.loads(json.dumps(manifest))
    for field in MANIFEST_HASH_EXCLUDED_FIELDS:
        payload.pop(field, None)
    return sha256_bytes(canonical_json_bytes(payload))


class GeneratePhase12OfficialClosurePreflightTest(unittest.TestCase):
    def setUp(self) -> None:
        self.tmp = tempfile.TemporaryDirectory()
        self.repo_root = Path(self.tmp.name) / "repo"
        self.repo_root.mkdir(parents=True)
        self.script_repo_root = Path(__file__).resolve().parents[2]
        self.script = (
            self.script_repo_root / "tools/ci/generate_phase12_official_closure_preflight.py"
        )
        self.candidate_dir = self.repo_root / "reports/phase12_official_closure_candidate"
        self.output_dir = self.repo_root / "reports/phase12_official_closure_preflight"

        self._init_git_repo()
        self._build_candidate()
        self._commit_all("seed candidate")

    def tearDown(self) -> None:
        self.tmp.cleanup()

    def _git(self, *args: str) -> str:
        proc = subprocess.run(
            ["git", *args],
            cwd=self.repo_root,
            check=False,
            capture_output=True,
            text=True,
        )
        self.assertEqual(proc.returncode, 0, proc.stderr)
        return proc.stdout.strip()

    def _init_git_repo(self) -> None:
        self._git("init")
        self._git("config", "user.name", "Test User")
        self._git("config", "user.email", "test@example.com")
        current_phase = self.repo_root / "docs/roadmap/CURRENT_PHASE"
        current_phase.parent.mkdir(parents=True)
        current_phase.write_text("CURRENT_PHASE=10\n", encoding="utf-8")
        (self.repo_root / "README.md").write_text("temp repo\n", encoding="utf-8")
        self._commit_all("init")

    def _commit_all(self, message: str) -> None:
        self._git("add", ".")
        self._git("commit", "-m", message)

    def _build_candidate(self) -> None:
        run_dir = self.repo_root / "evidence/run-run-local-phase12c-closure-2026-03-11"
        (run_dir / "meta").mkdir(parents=True)
        (run_dir / "reports").mkdir(parents=True)
        (run_dir / "gates").mkdir(parents=True)
        self.candidate_dir.mkdir(parents=True)

        (run_dir / "meta/git.txt").write_text("placeholder-git-sha\n", encoding="utf-8")
        (run_dir / "meta/run.json").write_text(
            json.dumps(
                {
                    "run_id": "run-local-phase12c-closure-2026-03-11",
                    "time_utc": "2026-03-11T16:59:40Z",
                },
                indent=2,
                sort_keys=True,
            )
            + "\n",
            encoding="utf-8",
        )
        (run_dir / "reports/summary.json").write_text(
            json.dumps(
                {
                    "run_id": "run-local-phase12c-closure-2026-03-11",
                    "verdict": "PASS",
                    "gates": {
                        gate: {"verdict": "PASS", "violations_count": 0}
                        for gate in REQUIRED_GATES
                    },
                },
                indent=2,
                sort_keys=True,
            )
            + "\n",
            encoding="utf-8",
        )

        gate_entries: list[dict[str, object]] = []
        for gate in REQUIRED_GATES:
            gate_path = run_dir / "gates" / gate / "report.json"
            gate_path.parent.mkdir(parents=True)
            gate_path.write_text(
                json.dumps(
                    {"gate": gate, "verdict": "PASS", "violations_count": 0},
                    indent=2,
                    sort_keys=True,
                )
                + "\n",
                encoding="utf-8",
            )
            gate_entries.append(
                {
                    "path": gate_path.relative_to(self.repo_root).as_posix(),
                    "sha256": sha256_file(gate_path),
                    "size_bytes": gate_path.stat().st_size,
                    "gate": gate,
                    "verdict": "PASS",
                    "violations_count": 0,
                }
            )

        report_artifacts: list[dict[str, object]] = []
        for report_path in sorted((run_dir / "reports").glob("*")):
            report_artifacts.append(
                {
                    "path": report_path.relative_to(self.repo_root).as_posix(),
                    "sha256": sha256_file(report_path),
                    "size_bytes": report_path.stat().st_size,
                }
            )

        meta_artifacts: list[dict[str, object]] = []
        for meta_path in sorted((run_dir / "meta").glob("*")):
            meta_artifacts.append(
                {
                    "path": meta_path.relative_to(self.repo_root).as_posix(),
                    "sha256": sha256_file(meta_path),
                    "size_bytes": meta_path.stat().st_size,
                }
            )

        indexed_entries = [
            *[{k: entry[k] for k in ("path", "sha256", "size_bytes")} for entry in report_artifacts],
            *[{k: entry[k] for k in ("path", "sha256", "size_bytes")} for entry in gate_entries],
            *[{k: entry[k] for k in ("path", "sha256", "size_bytes")} for entry in meta_artifacts],
        ]
        evidence_root_hash = build_tree_root(indexed_entries)

        evidence_index = {
            "generated_at_utc": "2026-03-13T12:00:00Z",
            "index_version": 1,
            "run": {
                "run_id": "run-local-phase12c-closure-2026-03-11",
                "evidence_run_dir": run_dir.relative_to(self.repo_root).as_posix(),
                "git_sha": "placeholder-git-sha",
            },
            "evidence_root_algorithm": "sha256_path_digest_tree_v1",
            "evidence_root_hash": evidence_root_hash,
            "report_artifacts": report_artifacts,
            "gate_reports": gate_entries,
            "meta_artifacts": meta_artifacts,
        }

        evidence_index_path = self.candidate_dir / "evidence_index.json"
        evidence_index_path.write_text(
            json.dumps(evidence_index, indent=2, sort_keys=True) + "\n",
            encoding="utf-8",
        )

        manifest = {
            "boundary_invariants": [
                "proofd != authority_surface",
                "parity != consensus",
                "system computes truth; it does not choose truth",
            ],
            "closure_class": "official_closure_candidate",
            "closure_state": "LOCAL_CLOSURE_READY",
            "closure_attestation": {
                "attestation_state": "UNSIGNED",
                "reason": "attestor_key_material_not_provided",
            },
            "current_phase_pointer": "10",
            "evidence_index_path": evidence_index_path.relative_to(self.repo_root).as_posix(),
            "evidence_index_sha256": sha256_file(evidence_index_path),
            "evidence_root_algorithm": "sha256_path_digest_tree_v1",
            "evidence_root_hash": evidence_root_hash,
            "gate_policy": {
                "all_required_gates_passed": True,
                "required_gate_count": len(REQUIRED_GATES),
                "required_gates": list(REQUIRED_GATES),
            },
            "generated_at_utc": "2026-03-13T12:00:00Z",
            "manifest_hash_excluded_fields": list(MANIFEST_HASH_EXCLUDED_FIELDS),
            "manifest_digest_algorithm": "sha256",
            "manifest_sha256": "",
            "manifest_version": 1,
            "official_closure_prerequisites_remaining": [
                "mint_dedicated_closure_tag",
                "obtain_remote_official_confirmation",
                "execute_formal_phase_transition",
            ],
            "phase": "12",
            "recommended_tag": "phase12-official-closure",
            "run": {
                "evidence_run_dir": run_dir.relative_to(self.repo_root).as_posix(),
                "git_sha": "placeholder-git-sha",
                "reported_run_id": "run-local-phase12c-closure-2026-03-11",
                "run_dir_name": run_dir.name,
                "summary_path": (run_dir / "reports/summary.json").relative_to(self.repo_root).as_posix(),
                "time_utc": "2026-03-11T16:59:40Z",
            },
            "summary_note_path": (self.candidate_dir / "README.md").relative_to(self.repo_root).as_posix(),
        }
        manifest["manifest_sha256"] = compute_manifest_self_hash(manifest)

        manifest_path = self.candidate_dir / "closure_manifest.json"
        manifest_path.write_text(
            json.dumps(manifest, indent=2, sort_keys=True) + "\n", encoding="utf-8"
        )
        (self.candidate_dir / "closure_manifest.sha256").write_text(
            f"{sha256_file(manifest_path)}  {manifest_path.relative_to(self.repo_root).as_posix()}\n",
            encoding="utf-8",
        )
        (self.candidate_dir / "evidence_index.sha256").write_text(
            f"{sha256_file(evidence_index_path)}  {evidence_index_path.relative_to(self.repo_root).as_posix()}\n",
            encoding="utf-8",
        )
        (self.candidate_dir / "README.md").write_text(
            "# Phase-12 Official Closure Candidate\n", encoding="utf-8"
        )

    def _run_preflight(self, *extra_args: str) -> subprocess.CompletedProcess[str]:
        return subprocess.run(
            [
                "python3",
                str(self.script),
                "--repo-root",
                str(self.repo_root),
                "--candidate-dir",
                str(self.candidate_dir),
                "--output-dir",
                str(self.output_dir),
                *extra_args,
            ],
            cwd=self.script_repo_root,
            check=False,
            capture_output=True,
            text=True,
        )

    def test_reports_blockers_for_unsigned_candidate(self) -> None:
        proc = self._run_preflight()
        self.assertEqual(proc.returncode, 0, proc.stderr)

        report = json.loads(
            (self.output_dir / "preflight_report.json").read_text(encoding="utf-8")
        )
        blocker_codes = {item["code"] for item in report["blockers"]}

        self.assertEqual(report["local_execution_state"], "BLOCKED")
        self.assertEqual(report["official_closure_state"], "BLOCKED")
        self.assertIn("ATTESTATION_UNSIGNED", blocker_codes)

    def test_fail_on_blockers_returns_non_zero(self) -> None:
        proc = self._run_preflight("--fail-on-blockers")
        self.assertNotEqual(proc.returncode, 0)

    def test_reports_manifest_self_hash_mismatch(self) -> None:
        manifest_path = self.candidate_dir / "closure_manifest.json"
        manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
        manifest["manifest_sha256"] = "00" * 32
        manifest_path.write_text(
            json.dumps(manifest, indent=2, sort_keys=True) + "\n", encoding="utf-8"
        )
        self._commit_all("tamper manifest hash")

        proc = self._run_preflight()
        self.assertEqual(proc.returncode, 0, proc.stderr)

        report = json.loads(
            (self.output_dir / "preflight_report.json").read_text(encoding="utf-8")
        )
        blocker_codes = {item["code"] for item in report["blockers"]}
        self.assertIn("MANIFEST_SELF_HASH_MISMATCH", blocker_codes)


if __name__ == "__main__":
    unittest.main()
