#!/usr/bin/env python3
"""Black-box tests for gate_proof_trust_policy.sh."""

from __future__ import annotations

import json
import subprocess
import tempfile
import unittest
from pathlib import Path


class ProofTrustPolicyGateTest(unittest.TestCase):
    def setUp(self) -> None:
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)
        self.repo_root = Path(__file__).resolve().parents[2]
        self.evidence_dir = self.root / "proof-trust-policy"
        self.script = self.repo_root / "scripts/ci/gate_proof_trust_policy.sh"

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
        schema_report = json.loads(
            (self.evidence_dir / "policy_schema_report.json").read_text(encoding="utf-8")
        )
        hash_report = json.loads(
            (self.evidence_dir / "policy_hash_report.json").read_text(encoding="utf-8")
        )

        self.assertEqual(report.get("verdict"), "PASS")
        self.assertEqual(report.get("violations_count"), 0)
        self.assertEqual(schema_report.get("status"), "PASS")
        self.assertTrue(schema_report.get("external_to_bundle"))
        self.assertTrue(hash_report.get("baseline_hash_stable"))
        self.assertTrue(hash_report.get("policy_hash_changes_under_mutation"))
        verdict_rows = hash_report.get("verdict_rows")
        self.assertEqual(len(verdict_rows), 4)
        self.assertEqual(verdict_rows[0].get("actual_verdict"), "TRUSTED")
        self.assertEqual(verdict_rows[1].get("actual_verdict"), "REJECTED_BY_POLICY")
        self.assertEqual(verdict_rows[2].get("actual_verdict"), "UNTRUSTED")
        self.assertEqual(verdict_rows[3].get("actual_verdict"), "INVALID")
        self.assertIn("PV0504", verdict_rows[3].get("error_codes"))
        self.assertTrue((self.evidence_dir / "violations.txt").is_file())
        self.assertEqual(
            (self.evidence_dir / "violations.txt").read_text(encoding="utf-8"), ""
        )


if __name__ == "__main__":
    unittest.main()
