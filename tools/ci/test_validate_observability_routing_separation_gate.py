#!/usr/bin/env python3
"""Black-box tests for gate_observability_routing_separation.sh."""

from __future__ import annotations

import json
import subprocess
import tempfile
import unittest
from pathlib import Path


class ObservabilityRoutingSeparationGateTest(unittest.TestCase):
    def setUp(self) -> None:
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)
        self.repo_root = Path(__file__).resolve().parents[2]
        self.script = (
            self.repo_root / "scripts" / "ci" / "gate_observability_routing_separation.sh"
        )
        self.evidence_dir = self.root / "observability-routing-separation"

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
            (self.evidence_dir / "observability_routing_separation_report.json").read_text(
                encoding="utf-8"
            )
        )
        matrix = json.loads(
            (self.evidence_dir / "observability_routing_negative_matrix.json").read_text(
                encoding="utf-8"
            )
        )

        self.assertEqual(report.get("verdict"), "PASS")
        self.assertEqual(detail.get("status"), "PASS")
        self.assertEqual(detail.get("correlation_hits"), [])
        self.assertEqual(len(matrix.get("violation_matrix", [])), 5)

    def test_gate_fails_when_routing_function_reads_observability_field(self) -> None:
        source_root = self.root / "fixture-root"
        relative_path = Path("approved") / "routing.rs"
        full_path = source_root / relative_path
        full_path.parent.mkdir(parents=True, exist_ok=True)
        full_path.write_text(
            """
fn route_verification(payload: &serde_json::Value) {
    let selected = payload["dominant_authority_chain_id"].as_str();
}
""".strip()
            + "\n",
            encoding="utf-8",
        )

        proc = self._run_fixture(source_root, relative_path)
        self.assertEqual(proc.returncode, 2)

        detail = json.loads(
            (self.evidence_dir / "observability_routing_separation_report.json").read_text(
                encoding="utf-8"
            )
        )
        hits = detail.get("correlation_hits", [])
        self.assertTrue(any(hit.get("rule") == "routing_blindness" for hit in hits), hits)

    def test_gate_fails_when_observability_alias_reaches_routing_sink(self) -> None:
        source_root = self.root / "fixture-root"
        relative_path = Path("approved") / "routing.rs"
        full_path = source_root / relative_path
        full_path.parent.mkdir(parents=True, exist_ok=True)
        full_path.write_text(
            """
fn choose_verifier(payload: &serde_json::Value) {
    let route_basis = payload["outcome_convergence_ratio"].as_f64();
    verification_route(route_basis);
}
""".strip()
            + "\n",
            encoding="utf-8",
        )

        proc = self._run_fixture(source_root, relative_path)
        self.assertEqual(proc.returncode, 2)

        detail = json.loads(
            (self.evidence_dir / "observability_routing_separation_report.json").read_text(
                encoding="utf-8"
            )
        )
        hits = detail.get("correlation_hits", [])
        self.assertTrue(
            any("route_basis" in hit.get("tainted_aliases", []) for hit in hits),
            hits,
        )

    def test_gate_fails_when_scheduling_optimizes_for_agreement_likelihood(self) -> None:
        source_root = self.root / "fixture-root"
        relative_path = Path("approved") / "routing.rs"
        full_path = source_root / relative_path
        full_path.parent.mkdir(parents=True, exist_ok=True)
        full_path.write_text(
            """
fn schedule_verification() {
    let agreement_ratio = 0.92;
    schedule_next_verifier(agreement_ratio);
}
""".strip()
            + "\n",
            encoding="utf-8",
        )

        proc = self._run_fixture(source_root, relative_path)
        self.assertEqual(proc.returncode, 2)

        detail = json.loads(
            (self.evidence_dir / "observability_routing_separation_report.json").read_text(
                encoding="utf-8"
            )
        )
        hits = detail.get("correlation_hits", [])
        self.assertTrue(any(hit.get("rule") == "agreement_bias" for hit in hits), hits)

    def test_gate_fails_when_routing_file_imports_observability_module(self) -> None:
        source_root = self.root / "fixture-root"
        relative_path = Path("approved") / "routing.rs"
        full_path = source_root / relative_path
        full_path.parent.mkdir(parents=True, exist_ok=True)
        full_path.write_text(
            """
use crate::authority::authority_drift_topology::build_authority_drift_topology;

fn select_verifier() {
    let _ = 1;
}
""".strip()
            + "\n",
            encoding="utf-8",
        )

        proc = self._run_fixture(source_root, relative_path)
        self.assertEqual(proc.returncode, 2)

        detail = json.loads(
            (self.evidence_dir / "observability_routing_separation_report.json").read_text(
                encoding="utf-8"
            )
        )
        hits = detail.get("correlation_hits", [])
        self.assertTrue(
            any(hit.get("rule") == "observability_module_import" for hit in hits),
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
