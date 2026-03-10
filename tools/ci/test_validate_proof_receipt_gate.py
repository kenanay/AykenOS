#!/usr/bin/env python3
"""Black-box tests for gate_proof_receipt.sh."""

from __future__ import annotations

import json
import subprocess
import tempfile
import unittest
from pathlib import Path


class ProofReceiptGateTest(unittest.TestCase):
    def setUp(self) -> None:
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)
        self.repo_root = Path(__file__).resolve().parents[2]
        self.evidence_dir = self.root / "proof-receipt"
        self.script = self.repo_root / "scripts/ci/gate_proof_receipt.sh"

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
            (self.evidence_dir / "receipt_schema_report.json").read_text(encoding="utf-8")
        )
        emit_report = json.loads(
            (self.evidence_dir / "receipt_emit_report.json").read_text(encoding="utf-8")
        )

        self.assertEqual(report.get("verdict"), "PASS")
        self.assertEqual(report.get("violations_count"), 0)
        self.assertEqual(schema_report.get("status"), "PASS")
        self.assertEqual(emit_report.get("status"), "PASS")
        self.assertEqual(emit_report.get("verification_verdict"), "TRUSTED")
        self.assertTrue((self.evidence_dir / "verification_receipt.json").is_file())
        self.assertTrue((self.evidence_dir / "violations.txt").is_file())
        self.assertEqual(
            (self.evidence_dir / "violations.txt").read_text(encoding="utf-8"), ""
        )


if __name__ == "__main__":
    unittest.main()
