#!/usr/bin/env python3
"""Black-box tests for gate_proof_replay_admission_boundary.sh."""

from __future__ import annotations

import json
import subprocess
import tempfile
import unittest
from pathlib import Path


class ProofReplayAdmissionBoundaryGateTest(unittest.TestCase):
    def setUp(self) -> None:
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)
        self.repo_root = Path(__file__).resolve().parents[2]
        self.evidence_dir = self.root / "proof-replay-admission-boundary"
        self.script = (
            self.repo_root / "scripts/ci/gate_proof_replay_admission_boundary.sh"
        )

    def tearDown(self) -> None:
        self.tmp.cleanup()

    def test_gate_passes_and_exports_required_artifacts(self) -> None:
        proc = subprocess.run(
            ["bash", str(self.script), "--evidence-dir", str(self.evidence_dir)],
            cwd=self.repo_root,
            check=False,
        )
        self.assertEqual(proc.returncode, 0)

        report = json.loads((self.evidence_dir / "report.json").read_text(encoding="utf-8"))
        replay_report = json.loads(
            (self.evidence_dir / "replay_admission_report.json").read_text(
                encoding="utf-8"
            )
        )
        boundary = json.loads(
            (self.evidence_dir / "boundary_contract.json").read_text(encoding="utf-8")
        )

        self.assertEqual(report.get("verdict"), "PASS")
        self.assertEqual(report.get("violations_count"), 0)
        self.assertEqual(replay_report.get("status"), "PASS")
        self.assertEqual(replay_report.get("trusted_verdict"), "TRUSTED")
        self.assertTrue(replay_report.get("receipt_emitted") is True)
        self.assertTrue(replay_report.get("replay_admission_granted") is False)
        self.assertTrue(replay_report.get("separate_replay_contract_required") is True)
        self.assertTrue(replay_report.get("proof_chain_replay_evidence_present") is True)
        self.assertEqual(boundary.get("status"), "PASS")
        self.assertTrue(
            boundary.get("accepted_proof_requires_separate_replay_contract") is True
        )
        self.assertTrue(boundary.get("replay_report_bound_in_proof_chain") is True)
        self.assertTrue(
            boundary.get("proof_chain_replay_evidence_is_not_admission") is True
        )
        self.assertEqual(boundary.get("verdict_subject_forbidden_fields_present"), [])
        self.assertEqual(boundary.get("receipt_forbidden_fields_present"), [])


if __name__ == "__main__":
    unittest.main()
