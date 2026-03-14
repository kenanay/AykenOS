#!/usr/bin/env python3
"""Black-box tests for gate_verifier_reputation_prohibition.sh."""

from __future__ import annotations

import json
import subprocess
import tempfile
import unittest
from pathlib import Path


REQUIRED_ARTIFACTS = (
    "parity_report.json",
    "parity_determinism_incidents.json",
    "parity_drift_attribution_report.json",
    "parity_convergence_report.json",
    "parity_authority_drift_topology.json",
    "parity_authority_suppression_report.json",
    "parity_incident_graph.json",
)


class VerifierReputationProhibitionGateTest(unittest.TestCase):
    def setUp(self) -> None:
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)
        self.repo_root = Path(__file__).resolve().parents[2]
        self.script = self.repo_root / "scripts/ci/gate_verifier_reputation_prohibition.sh"
        self.evidence_dir = self.root / "gate"
        self.artifact_root = self.root / "artifacts"
        self.artifact_root.mkdir(parents=True)
        self._write_safe_artifacts()

    def tearDown(self) -> None:
        self.tmp.cleanup()

    def _write_json(self, name: str, payload: object) -> None:
        (self.artifact_root / name).write_text(
            json.dumps(payload, indent=2, sort_keys=True) + "\n",
            encoding="utf-8",
        )

    def _write_safe_artifacts(self) -> None:
        self._write_json(
            "parity_report.json",
            {
                "status": "PASS",
                "node_count": 3,
                "consistency_report_path": "parity_consistency_report.json",
            },
        )
        self._write_json(
            "parity_determinism_incidents.json",
            {
                "determinism_incident_count": 1,
                "severity_counts": {"authority_drift": 1},
                "incidents": [
                    {
                        "incident_id": "sha256:a",
                        "severity": "authority_drift",
                        "nodes": ["node-a", "node-b"],
                    }
                ],
            },
        )
        self._write_json(
            "parity_drift_attribution_report.json",
            {
                "status": "PASS",
                "partitions": [{"kind": "authority_drift", "node_count": 1}],
            },
        )
        self._write_json(
            "parity_convergence_report.json",
            {
                "status": "PASS",
                "surface_partition_count": 2,
                "outcome_partition_count": 2,
            },
        )
        self._write_json(
            "parity_authority_drift_topology.json",
            {
                "status": "PASS",
                "topology": {
                    "node_count": 3,
                    "authority_cluster_count": 2,
                    "dominant_authority_chain_id": "chain-a",
                },
            },
        )
        self._write_json(
            "parity_authority_suppression_report.json",
            {
                "status": "PASS",
                "suppressed_drift_count": 1,
                "rule_counts": {"historical_shadow": 1},
            },
        )
        self._write_json(
            "parity_incident_graph.json",
            {
                "status": "PASS",
                "graph": {"node_count": 3, "edge_count": 2, "incident_count": 1},
            },
        )

    def _run_gate(self) -> subprocess.CompletedProcess[str]:
        return subprocess.run(
            [
                "bash",
                str(self.script),
                "--evidence-dir",
                str(self.evidence_dir),
                "--artifact-root",
                str(self.artifact_root),
            ],
            cwd=self.repo_root,
            check=False,
            capture_output=True,
            text=True,
        )

    def test_gate_passes_for_descriptive_only_artifacts(self) -> None:
        proc = self._run_gate()
        self.assertEqual(proc.returncode, 0, proc.stderr)

        report = json.loads((self.evidence_dir / "report.json").read_text(encoding="utf-8"))
        detail = json.loads(
            (self.evidence_dir / "reputation_prohibition_report.json").read_text(
                encoding="utf-8"
            )
        )

        self.assertEqual(report.get("verdict"), "PASS")
        self.assertEqual(report.get("violations_count"), 0)
        self.assertEqual(detail.get("status"), "PASS")
        self.assertEqual(detail.get("checked_artifact_count"), len(REQUIRED_ARTIFACTS))
        self.assertEqual(detail.get("forbidden_field_count"), 0)
        self.assertTrue((self.evidence_dir / "violations.txt").is_file())
        self.assertEqual(
            (self.evidence_dir / "violations.txt").read_text(encoding="utf-8"),
            "",
        )

    def test_gate_fails_on_exact_forbidden_field(self) -> None:
        self._write_json(
            "parity_incident_graph.json",
            {
                "status": "PASS",
                "graph": {"node_count": 3, "verifier_score": 0.97},
            },
        )

        proc = self._run_gate()
        self.assertEqual(proc.returncode, 2)

        report = json.loads((self.evidence_dir / "report.json").read_text(encoding="utf-8"))
        detail = json.loads(
            (self.evidence_dir / "reputation_prohibition_report.json").read_text(
                encoding="utf-8"
            )
        )
        self.assertEqual(report.get("verdict"), "FAIL")
        self.assertEqual(detail.get("status"), "FAIL")
        self.assertGreaterEqual(detail.get("forbidden_field_count", 0), 1)
        violations = (self.evidence_dir / "violations.txt").read_text(encoding="utf-8")
        self.assertIn("forbidden_reputation_field:parity_incident_graph.json", violations)
        self.assertIn("verifier_score", violations)

    def test_gate_fails_on_pattern_based_reputation_field(self) -> None:
        self._write_json(
            "parity_convergence_report.json",
            {
                "status": "PASS",
                "analytics": {"node_reliability_score": 12},
            },
        )

        proc = self._run_gate()
        self.assertEqual(proc.returncode, 2)

        detail = json.loads(
            (self.evidence_dir / "reputation_prohibition_report.json").read_text(
                encoding="utf-8"
            )
        )
        hit_fields = {hit.get("field") for hit in detail.get("forbidden_field_hits", [])}
        self.assertIn("node_reliability_score", hit_fields)


if __name__ == "__main__":
    unittest.main()
