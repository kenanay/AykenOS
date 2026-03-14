#!/usr/bin/env python3
"""Black-box tests for gate_verification_determinism_contract.sh."""

from __future__ import annotations

import json
import subprocess
import tempfile
import unittest
from pathlib import Path


class VerificationDeterminismContractGateTest(unittest.TestCase):
    def setUp(self) -> None:
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)
        self.repo_root = Path(__file__).resolve().parents[2]
        self.script = (
            self.repo_root / "scripts" / "ci" / "gate_verification_determinism_contract.sh"
        )
        self.evidence_dir = self.root / "verification-determinism-contract"

    def tearDown(self) -> None:
        self.tmp.cleanup()

    def test_gate_passes_on_current_repo_contract(self) -> None:
        proc = subprocess.run(
            [
                "bash",
                str(self.script),
                "--evidence-dir",
                str(self.evidence_dir),
            ],
            cwd=self.repo_root,
            check=False,
        )
        self.assertEqual(proc.returncode, 0)

        report = json.loads((self.evidence_dir / "report.json").read_text(encoding="utf-8"))
        detail = json.loads(
            (self.evidence_dir / "verification_determinism_contract_report.json").read_text(
                encoding="utf-8"
            )
        )

        self.assertEqual(report.get("verdict"), "PASS")
        self.assertEqual(detail.get("status"), "PASS")
        self.assertEqual(detail.get("pattern_hits"), [])

    def test_gate_fails_on_time_dependency_in_critical_source(self) -> None:
        source_root = self.root / "fixture-root"
        relative_path = Path("critical") / "verifier.rs"
        full_path = source_root / relative_path
        full_path.parent.mkdir(parents=True, exist_ok=True)
        full_path.write_text(
            "use std::time::SystemTime;\n\npub fn verify() { let _ = SystemTime::now(); }\n",
            encoding="utf-8",
        )

        proc = subprocess.run(
            [
                "bash",
                str(self.script),
                "--evidence-dir",
                str(self.evidence_dir),
                "--source-root",
                str(source_root),
                "--source-path",
                relative_path.as_posix(),
            ],
            cwd=self.repo_root,
            check=False,
        )
        self.assertEqual(proc.returncode, 2)

        detail = json.loads(
            (self.evidence_dir / "verification_determinism_contract_report.json").read_text(
                encoding="utf-8"
            )
        )
        hits = detail.get("pattern_hits", [])
        self.assertTrue(any(hit.get("rule") == "time_dependency" for hit in hits))


if __name__ == "__main__":
    unittest.main()
