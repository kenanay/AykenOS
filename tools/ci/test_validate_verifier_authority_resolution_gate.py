#!/usr/bin/env python3
"""Black-box tests for gate_verifier_authority_resolution.sh."""

from __future__ import annotations

import json
import subprocess
import tempfile
import unittest
from pathlib import Path


class VerifierAuthorityResolutionGateTest(unittest.TestCase):
    def setUp(self) -> None:
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)
        self.repo_root = Path(__file__).resolve().parents[2]
        self.evidence_dir = self.root / "verifier-authority-resolution"
        self.script = self.repo_root / "scripts/ci/gate_verifier_authority_resolution.sh"

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
        resolution_report = json.loads(
            (self.evidence_dir / "authority_resolution_report.json").read_text(encoding="utf-8")
        )
        receipt_authority_report = json.loads(
            (self.evidence_dir / "receipt_authority_report.json").read_text(encoding="utf-8")
        )
        chain_report = json.loads(
            (self.evidence_dir / "authority_chain_report.json").read_text(encoding="utf-8")
        )

        self.assertEqual(report.get("verdict"), "PASS")
        self.assertEqual(report.get("violations_count"), 0)
        self.assertEqual(
            resolution_report.get("result_class"), "AUTHORITY_RESOLVED_DELEGATED"
        )
        self.assertEqual(receipt_authority_report.get("status"), "PASS")
        self.assertEqual(
            receipt_authority_report.get("result_class"), "AUTHORITY_RESOLVED_DELEGATED"
        )
        self.assertEqual(receipt_authority_report.get("authority_chain_id_equal"), True)
        self.assertEqual(chain_report.get("status"), "PASS")
        self.assertTrue(
            resolution_report.get("authority_chain_id", "").startswith("sha256:")
        )
        self.assertTrue((self.evidence_dir / "violations.txt").is_file())
        self.assertEqual(
            (self.evidence_dir / "violations.txt").read_text(encoding="utf-8"), ""
        )


if __name__ == "__main__":
    unittest.main()
