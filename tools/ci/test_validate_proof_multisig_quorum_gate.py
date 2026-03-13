#!/usr/bin/env python3
"""Black-box tests for gate_proof_multisig_quorum.sh."""

from __future__ import annotations

import json
import subprocess
import tempfile
import unittest
from pathlib import Path


class ProofMultisigQuorumGateTest(unittest.TestCase):
    def setUp(self) -> None:
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)
        self.repo_root = Path(__file__).resolve().parents[2]
        self.evidence_dir = self.root / "proof-multisig-quorum"
        self.script = self.repo_root / "scripts/ci/gate_proof_multisig_quorum.sh"

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
        quorum_matrix = json.loads(
            (self.evidence_dir / "quorum_matrix.json").read_text(encoding="utf-8")
        )
        evaluator = json.loads(
            (self.evidence_dir / "quorum_evaluator_report.json").read_text(
                encoding="utf-8"
            )
        )

        def find_row(name: str) -> dict:
            for row in quorum_matrix:
                if row.get("scenario") == name:
                    return row
            self.fail(f"missing scenario {name}")

        duplicate_row = find_row("two_of_two_duplicate_key_entries_rejected")
        distinct_row = find_row("two_of_two_distinct_keys_trusted")
        revoked_row = find_row("two_of_two_revoked_secondary_key_invalid")

        self.assertEqual(report.get("verdict"), "PASS")
        self.assertEqual(report.get("violations_count"), 0)
        self.assertEqual(len(quorum_matrix), 7)
        self.assertEqual(evaluator.get("status"), "PASS")
        self.assertEqual(evaluator.get("scenario_count"), 7)
        self.assertEqual(evaluator.get("trusted_scenarios"), 2)
        self.assertEqual(evaluator.get("rejected_scenarios"), 3)
        self.assertEqual(evaluator.get("invalid_scenarios"), 2)
        self.assertTrue(evaluator.get("explicit_quorum_policy_active") is True)
        self.assertTrue(evaluator.get("distinct_key_quorum_enforced") is True)
        self.assertTrue(evaluator.get("duplicate_key_entries_fail_closed") is True)

        self.assertEqual(distinct_row.get("actual_verdict"), "TRUSTED")
        self.assertEqual(distinct_row.get("unique_trusted_key_count"), 2)
        self.assertEqual(duplicate_row.get("actual_verdict"), "REJECTED_BY_POLICY")
        self.assertEqual(duplicate_row.get("unique_trusted_key_count"), 1)
        self.assertEqual(revoked_row.get("actual_verdict"), "INVALID")
        self.assertIn("PV0403", revoked_row.get("error_codes", []))

        self.assertTrue((self.evidence_dir / "violations.txt").is_file())
        self.assertEqual(
            (self.evidence_dir / "violations.txt").read_text(encoding="utf-8"), ""
        )


if __name__ == "__main__":
    unittest.main()
