#!/usr/bin/env python3
"""Black-box tests for gate_proof_verifier_core.sh."""

from __future__ import annotations

import json
import subprocess
import tempfile
import unittest
from pathlib import Path


class ProofVerifierCoreGateTest(unittest.TestCase):
    def setUp(self) -> None:
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)
        self.repo_root = Path(__file__).resolve().parents[2]
        self.evidence_dir = self.root / "proof-verifier-core"
        self.script = self.repo_root / "scripts/ci/gate_proof_verifier_core.sh"

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
        core_report = json.loads(
            (self.evidence_dir / "verifier_core_report.json").read_text(encoding="utf-8")
        )
        determinism_matrix = json.loads(
            (self.evidence_dir / "determinism_matrix.json").read_text(encoding="utf-8")
        )

        self.assertEqual(report.get("verdict"), "PASS")
        self.assertEqual(report.get("violations_count"), 0)
        self.assertEqual(core_report.get("status"), "PASS")
        self.assertEqual(core_report.get("api_entrypoint"), "verify_bundle")
        self.assertEqual(core_report.get("scenario_count"), 5)
        self.assertEqual(core_report.get("deterministic_case_count"), 5)
        self.assertEqual(len(determinism_matrix), 5)
        self.assertTrue(
            all(row.get("deterministic") is True for row in determinism_matrix)
        )
        self.assertEqual(determinism_matrix[0].get("expected_verdict"), "TRUSTED")
        self.assertTrue((self.evidence_dir / "violations.txt").is_file())
        self.assertEqual(
            (self.evidence_dir / "violations.txt").read_text(encoding="utf-8"), ""
        )


if __name__ == "__main__":
    unittest.main()
