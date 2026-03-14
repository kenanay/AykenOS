#!/usr/bin/env python3
"""Black-box tests for gate_proof_replicated_verification_boundary.sh."""

from __future__ import annotations

import json
import subprocess
import tempfile
import unittest
from pathlib import Path


class ProofReplicatedVerificationBoundaryGateTest(unittest.TestCase):
    def setUp(self) -> None:
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)
        self.repo_root = Path(__file__).resolve().parents[2]
        self.evidence_dir = self.root / "proof-replicated-verification-boundary"
        self.script = (
            self.repo_root / "scripts/ci/gate_proof_replicated_verification_boundary.sh"
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
        bridge = json.loads(
            (self.evidence_dir / "phase13_bridge_report.json").read_text(
                encoding="utf-8"
            )
        )
        note = (self.evidence_dir / "research_boundary_note.md").read_text(
            encoding="utf-8"
        )

        self.assertEqual(report.get("verdict"), "PASS")
        self.assertEqual(report.get("violations_count"), 0)
        self.assertEqual(bridge.get("status"), "PASS")
        self.assertTrue(bridge.get("phase13_map_present") is True)
        self.assertTrue(
            bridge.get("replicated_verification_outside_phase12_core") is True
        )
        self.assertIn("verified proof != replay admission", note)
        self.assertIn("Phase-12 preserves a hard boundary", note)
        self.assertEqual(bridge.get("proofd_disallowed_routes_present"), [])


if __name__ == "__main__":
    unittest.main()
