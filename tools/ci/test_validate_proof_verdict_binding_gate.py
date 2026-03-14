#!/usr/bin/env python3
"""Black-box tests for gate_proof_verdict_binding.sh."""

from __future__ import annotations

import json
import subprocess
import tempfile
import unittest
from pathlib import Path


class ProofVerdictBindingGateTest(unittest.TestCase):
    def setUp(self) -> None:
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)
        self.repo_root = Path(__file__).resolve().parents[2]
        self.evidence_dir = self.root / "proof-verdict-binding"
        self.script = self.repo_root / "scripts/ci/gate_proof_verdict_binding.sh"

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
        binding_report = json.loads(
            (self.evidence_dir / "verdict_binding_report.json").read_text(encoding="utf-8")
        )
        examples = json.loads(
            (self.evidence_dir / "verdict_subject_examples.json").read_text(encoding="utf-8")
        )

        self.assertEqual(report.get("verdict"), "PASS")
        self.assertEqual(report.get("violations_count"), 0)
        self.assertEqual(binding_report.get("status"), "PASS")
        self.assertEqual(binding_report.get("verification_verdict"), "TRUSTED")
        self.assertTrue(binding_report.get("same_subject_tuple"))
        self.assertTrue(binding_report.get("same_verdict"))
        self.assertTrue(binding_report.get("receipt_binding_equal"))
        self.assertEqual(len(examples.get("distributed_claim_weaker_tuples")), 3)
        self.assertTrue((self.evidence_dir / "violations.txt").is_file())
        self.assertEqual(
            (self.evidence_dir / "violations.txt").read_text(encoding="utf-8"), ""
        )


if __name__ == "__main__":
    unittest.main()
