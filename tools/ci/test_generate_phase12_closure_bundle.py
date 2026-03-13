#!/usr/bin/env python3
"""Tests for generate_phase12_closure_bundle.py."""

from __future__ import annotations

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


class GeneratePhase12ClosureBundleTest(unittest.TestCase):
    def setUp(self) -> None:
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)
        self.repo_root = Path(__file__).resolve().parents[2]
        self.script = self.repo_root / "tools/ci/generate_phase12_closure_bundle.py"
        self.run_dir = self.root / "evidence" / "run-run-local-phase12c-closure-2026-03-11"
        self.output_dir = self.root / "reports" / "phase12_official_closure_candidate"
        self._build_run_dir()

    def tearDown(self) -> None:
        self.tmp.cleanup()

    def _build_run_dir(self) -> None:
        (self.run_dir / "meta").mkdir(parents=True)
        (self.run_dir / "reports").mkdir(parents=True)
        (self.run_dir / "gates").mkdir(parents=True)

        (self.run_dir / "meta" / "git.txt").write_text(
            "0123456789abcdef0123456789abcdef01234567\n", encoding="utf-8"
        )
        (self.run_dir / "meta" / "run.json").write_text(
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

        summary = {
            "run_id": "run-local-phase12c-closure-2026-03-11",
            "time_utc": "2026-03-11T16:59:40Z",
            "git_sha": "0123456789abcdef0123456789abcdef01234567",
            "verdict": "PASS",
            "freeze_status": "pending_runtime_verification",
            "gates": {
                gate: {"verdict": "PASS", "violations_count": 0}
                for gate in REQUIRED_GATES
            },
        }
        (self.run_dir / "reports" / "summary.json").write_text(
            json.dumps(summary, indent=2, sort_keys=True) + "\n", encoding="utf-8"
        )
        (self.run_dir / "reports" / "proofd-service.json").write_text(
            json.dumps(
                {"gate": "proofd-service", "verdict": "PASS", "violations_count": 0},
                indent=2,
                sort_keys=True,
            )
            + "\n",
            encoding="utf-8",
        )

        for gate in REQUIRED_GATES:
            gate_dir = self.run_dir / "gates" / gate
            gate_dir.mkdir(parents=True)
            (gate_dir / "report.json").write_text(
                json.dumps(
                    {"gate": gate, "verdict": "PASS", "violations_count": 0},
                    indent=2,
                    sort_keys=True,
                )
                + "\n",
                encoding="utf-8",
            )

    def test_generates_manifest_and_evidence_index(self) -> None:
        proc = subprocess.run(
            [
                "python3",
                str(self.script),
                "--run-dir",
                str(self.run_dir),
                "--output-dir",
                str(self.output_dir),
            ],
            cwd=self.repo_root,
            check=False,
        )
        self.assertEqual(proc.returncode, 0)

        manifest = json.loads(
            (self.output_dir / "closure_manifest.json").read_text(encoding="utf-8")
        )
        evidence_index = json.loads(
            (self.output_dir / "evidence_index.json").read_text(encoding="utf-8")
        )
        summary_note = (self.output_dir / "README.md").read_text(encoding="utf-8")

        self.assertEqual(manifest["phase"], "12")
        self.assertEqual(manifest["closure_state"], "LOCAL_CLOSURE_READY")
        self.assertEqual(manifest["recommended_tag"], "phase12-official-closure")
        self.assertEqual(manifest["gate_policy"]["required_gate_count"], 20)
        self.assertEqual(manifest["closure_attestation"]["attestation_state"], "UNSIGNED")
        self.assertTrue(manifest["manifest_sha256"])
        self.assertTrue(manifest["evidence_root_hash"])
        self.assertEqual(len(evidence_index["gate_reports"]), 20)
        self.assertIn("Phase-12 Official Closure Candidate", summary_note)

    def test_fails_when_required_gate_is_missing(self) -> None:
        (self.run_dir / "gates" / "proofd-service" / "report.json").unlink()

        proc = subprocess.run(
            [
                "python3",
                str(self.script),
                "--run-dir",
                str(self.run_dir),
                "--output-dir",
                str(self.output_dir),
            ],
            cwd=self.repo_root,
            check=False,
            capture_output=True,
            text=True,
        )

        self.assertNotEqual(proc.returncode, 0)
        self.assertIn("Missing gate report", proc.stderr)

    def test_fails_when_required_gate_verdict_is_not_pass(self) -> None:
        failing_summary = json.loads(
            (self.run_dir / "reports" / "summary.json").read_text(encoding="utf-8")
        )
        failing_summary["gates"]["proofd-service"]["verdict"] = "FAIL"
        (self.run_dir / "reports" / "summary.json").write_text(
            json.dumps(failing_summary, indent=2, sort_keys=True) + "\n",
            encoding="utf-8",
        )

        proc = subprocess.run(
            [
                "python3",
                str(self.script),
                "--run-dir",
                str(self.run_dir),
                "--output-dir",
                str(self.output_dir),
            ],
            cwd=self.repo_root,
            check=False,
            capture_output=True,
            text=True,
        )

        self.assertNotEqual(proc.returncode, 0)
        self.assertIn("failing gates", proc.stderr)


if __name__ == "__main__":
    unittest.main()
