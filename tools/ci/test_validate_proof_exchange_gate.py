#!/usr/bin/env python3
"""Black-box tests for gate_proof_exchange.sh."""

from __future__ import annotations

import json
import subprocess
import tempfile
import unittest
from pathlib import Path


class ProofExchangeGateTest(unittest.TestCase):
    def setUp(self) -> None:
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)
        self.repo_root = Path(__file__).resolve().parents[2]
        self.evidence_dir = self.root / "proof-exchange"
        self.script = self.repo_root / "scripts/ci/gate_proof_exchange.sh"

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
        contract = json.loads(
            (self.evidence_dir / "exchange_contract_report.json").read_text(encoding="utf-8")
        )
        matrix = json.loads(
            (self.evidence_dir / "transport_mutation_matrix.json").read_text(encoding="utf-8")
        )

        self.assertEqual(report.get("verdict"), "PASS")
        self.assertEqual(report.get("violations_count"), 0)
        self.assertEqual(contract.get("status"), "PASS")
        self.assertEqual(contract.get("exchange_mode"), "proof_bundle_transport_v1")
        self.assertTrue(contract.get("payload_overlay_receipt_separated") is True)
        self.assertEqual(len(matrix), 7)
        self.assertEqual(matrix[0].get("status"), "PASS")
        self.assertEqual(matrix[1].get("status"), "PASS")
        self.assertEqual(matrix[2].get("status"), "PASS")
        self.assertEqual(matrix[3].get("status"), "FAIL")
        self.assertTrue((self.evidence_dir / "exchange_message.json").is_file())
        self.assertTrue((self.evidence_dir / "violations.txt").is_file())
        self.assertEqual(
            (self.evidence_dir / "violations.txt").read_text(encoding="utf-8"), ""
        )


if __name__ == "__main__":
    unittest.main()
