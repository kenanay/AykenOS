#!/usr/bin/env python3
"""Black-box tests for gate_diagnostics_consumer_non_authoritative_contract.sh."""

from __future__ import annotations

import json
import subprocess
import tempfile
import unittest
from pathlib import Path


class DiagnosticsConsumerNonAuthoritativeContractGateTest(unittest.TestCase):
    def setUp(self) -> None:
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)
        self.repo_root = Path(__file__).resolve().parents[2]
        self.script = (
            self.repo_root
            / "scripts"
            / "ci"
            / "gate_diagnostics_consumer_non_authoritative_contract.sh"
        )
        self.evidence_dir = self.root / "diagnostics-consumer-non-authoritative-contract"

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
            capture_output=True,
            text=True,
        )
        self.assertEqual(proc.returncode, 0, proc.stderr)

        report = json.loads((self.evidence_dir / "report.json").read_text(encoding="utf-8"))
        detail = json.loads(
            (self.evidence_dir / "diagnostics_consumer_contract_report.json").read_text(
                encoding="utf-8"
            )
        )

        self.assertEqual(report.get("verdict"), "PASS")
        self.assertEqual(detail.get("status"), "PASS")
        self.assertEqual(detail.get("field_hits"), [])
        self.assertEqual(detail.get("artifact_hits"), [])

    def test_gate_fails_when_unapproved_runtime_file_reads_descriptive_field(self) -> None:
        source_root = self.root / "fixture-root"
        runtime_file = source_root / "runtime" / "consumer.rs"
        runtime_file.parent.mkdir(parents=True, exist_ok=True)
        runtime_file.write_text(
            'pub fn route() { let _ = payload["global_status"].as_str(); }\n',
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
                "--scan-root",
                "runtime",
            ],
            cwd=self.repo_root,
            check=False,
            capture_output=True,
            text=True,
        )
        self.assertEqual(proc.returncode, 2)

        detail = json.loads(
            (self.evidence_dir / "diagnostics_consumer_contract_report.json").read_text(
                encoding="utf-8"
            )
        )
        hits = detail.get("field_hits", [])
        self.assertTrue(any(hit.get("token") == "global_status" for hit in hits))

    def test_gate_fails_when_unapproved_runtime_file_reads_diagnostics_artifact(self) -> None:
        source_root = self.root / "fixture-root"
        runtime_file = source_root / "runtime" / "consumer.rs"
        runtime_file.parent.mkdir(parents=True, exist_ok=True)
        runtime_file.write_text(
            'pub const REPORT: &str = "parity_convergence_report.json";\n',
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
                "--scan-root",
                "runtime",
            ],
            cwd=self.repo_root,
            check=False,
            capture_output=True,
            text=True,
        )
        self.assertEqual(proc.returncode, 2)

        detail = json.loads(
            (self.evidence_dir / "diagnostics_consumer_contract_report.json").read_text(
                encoding="utf-8"
            )
        )
        hits = detail.get("artifact_hits", [])
        self.assertTrue(
            any(hit.get("token") == "parity_convergence_report.json" for hit in hits)
        )

    def test_gate_allows_explicit_passthrough_when_path_is_allowlisted(self) -> None:
        source_root = self.root / "fixture-root"
        allowed_file = source_root / "observability" / "passthrough.rs"
        allowed_file.parent.mkdir(parents=True, exist_ok=True)
        allowed_file.write_text(
            'pub fn serve() { let _ = payload["dominant_authority_chain_id"].as_str(); }\n',
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
                "--scan-root",
                "observability",
                "--allow-path",
                "observability/passthrough.rs",
            ],
            cwd=self.repo_root,
            check=False,
            capture_output=True,
            text=True,
        )
        self.assertEqual(proc.returncode, 0, proc.stderr)


if __name__ == "__main__":
    unittest.main()
