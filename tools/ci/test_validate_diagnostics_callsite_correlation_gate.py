#!/usr/bin/env python3
"""Black-box tests for gate_diagnostics_callsite_correlation.sh."""

from __future__ import annotations

import json
import subprocess
import tempfile
import unittest
from pathlib import Path


class DiagnosticsCallsiteCorrelationGateTest(unittest.TestCase):
    def setUp(self) -> None:
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)
        self.repo_root = Path(__file__).resolve().parents[2]
        self.script = (
            self.repo_root / "scripts" / "ci" / "gate_diagnostics_callsite_correlation.sh"
        )
        self.evidence_dir = self.root / "diagnostics-callsite-correlation"

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
            (self.evidence_dir / "diagnostics_callsite_correlation_report.json").read_text(
                encoding="utf-8"
            )
        )

        self.assertEqual(report.get("verdict"), "PASS")
        self.assertEqual(detail.get("status"), "PASS")
        self.assertEqual(detail.get("correlation_hits"), [])

    def test_gate_fails_when_protected_field_flows_directly_to_sink(self) -> None:
        source_root = self.root / "fixture-root"
        relative_path = Path("approved") / "flow.rs"
        full_path = source_root / relative_path
        full_path.parent.mkdir(parents=True, exist_ok=True)
        full_path.write_text(
            """
fn consume(payload: &serde_json::Value) {
    apply_policy(payload["global_status"].as_str());
}
""".strip()
            + "\n",
            encoding="utf-8",
        )

        proc = self._run_fixture(source_root, relative_path)
        self.assertEqual(proc.returncode, 2)

        detail = json.loads(
            (self.evidence_dir / "diagnostics_callsite_correlation_report.json").read_text(
                encoding="utf-8"
            )
        )
        hits = detail.get("correlation_hits", [])
        self.assertTrue(any("global_status" in hit.get("source_tokens", []) for hit in hits))

    def test_gate_fails_when_aliased_status_reaches_replay_sink(self) -> None:
        source_root = self.root / "fixture-root"
        relative_path = Path("approved") / "flow.rs"
        full_path = source_root / relative_path
        full_path.parent.mkdir(parents=True, exist_ok=True)
        full_path.write_text(
            """
fn consume(payload: &serde_json::Value) {
    let status_class = payload["global_status"].as_str();
    let replay_basis = status_class;
    replay_admission(replay_basis);
}
""".strip()
            + "\n",
            encoding="utf-8",
        )

        proc = self._run_fixture(source_root, relative_path)
        self.assertEqual(proc.returncode, 2)

        detail = json.loads(
            (self.evidence_dir / "diagnostics_callsite_correlation_report.json").read_text(
                encoding="utf-8"
            )
        )
        hits = detail.get("correlation_hits", [])
        self.assertTrue(
            any("replay_basis" in hit.get("tainted_aliases", []) for hit in hits),
            hits,
        )

    def test_gate_fails_when_artifact_alias_reaches_override_sink(self) -> None:
        source_root = self.root / "fixture-root"
        relative_path = Path("approved") / "flow.rs"
        full_path = source_root / relative_path
        full_path.parent.mkdir(parents=True, exist_ok=True)
        full_path.write_text(
            """
fn consume() {
    let artifact_name = "parity_convergence_report.json";
    let selected_artifact = artifact_name;
    execution_override(selected_artifact);
}
""".strip()
            + "\n",
            encoding="utf-8",
        )

        proc = self._run_fixture(source_root, relative_path)
        self.assertEqual(proc.returncode, 2)

        detail = json.loads(
            (self.evidence_dir / "diagnostics_callsite_correlation_report.json").read_text(
                encoding="utf-8"
            )
        )
        hits = detail.get("correlation_hits", [])
        self.assertTrue(
            any("selected_artifact" in hit.get("tainted_aliases", []) for hit in hits),
            hits,
        )

    def _run_fixture(self, source_root: Path, relative_path: Path) -> subprocess.CompletedProcess[str]:
        return subprocess.run(
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
            capture_output=True,
            text=True,
        )


if __name__ == "__main__":
    unittest.main()
