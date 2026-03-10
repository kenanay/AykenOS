#!/usr/bin/env python3
"""Black-box tests for gate_proof_verifier_cli.sh."""

from __future__ import annotations

import json
import subprocess
import tempfile
import unittest
from pathlib import Path


class ProofVerifierCliGateTest(unittest.TestCase):
    def setUp(self) -> None:
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)
        self.repo_root = Path(__file__).resolve().parents[2]
        self.evidence_dir = self.root / "proof-verifier-cli"
        self.script = self.repo_root / "scripts/ci/gate_proof_verifier_cli.sh"

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
        smoke_report = json.loads(
            (self.evidence_dir / "cli_smoke_report.json").read_text(encoding="utf-8")
        )
        output_contract = json.loads(
            (self.evidence_dir / "cli_output_contract.json").read_text(encoding="utf-8")
        )

        self.assertEqual(report.get("verdict"), "PASS")
        self.assertEqual(report.get("violations_count"), 0)
        self.assertEqual(smoke_report.get("status"), "PASS")
        self.assertEqual(smoke_report.get("command_surface"), "verify bundle")
        self.assertEqual(output_contract.get("status"), "PASS")
        self.assertEqual(output_contract.get("verdict"), "TRUSTED")
        self.assertTrue(
            output_contract.get("required_fields_present", {}).get("bundle_id") is True
        )
        self.assertTrue(
            output_contract.get("matches_verifier_core", {}).get("policy_hash") is True
        )
        self.assertTrue((self.evidence_dir / "cli_human_stdout.txt").is_file())
        self.assertTrue((self.evidence_dir / "cli_json_output.json").is_file())
        self.assertTrue((self.evidence_dir / "violations.txt").is_file())
        self.assertEqual(
            (self.evidence_dir / "violations.txt").read_text(encoding="utf-8"), ""
        )


if __name__ == "__main__":
    unittest.main()
