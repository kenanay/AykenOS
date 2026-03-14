#!/usr/bin/env python3
"""Black-box tests for gate_proofd_observability_boundary.sh."""

from __future__ import annotations

import json
import subprocess
import tempfile
import unittest
from pathlib import Path


class ProofdObservabilityBoundaryGateTest(unittest.TestCase):
    RUN_ID = "run-proofd-local-r1"

    def setUp(self) -> None:
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)
        self.repo_root = Path(__file__).resolve().parents[2]
        self.script = (
            self.repo_root / "scripts" / "ci" / "gate_proofd_observability_boundary.sh"
        )
        self.artifact_root = self.root / "artifact-root"
        self.evidence_dir = self.root / "proofd-observability-boundary"

    def tearDown(self) -> None:
        self.tmp.cleanup()

    def test_gate_passes_for_read_only_non_authoritative_surface(self) -> None:
        self._write_fixture()
        proc = self._run_gate()
        self.assertEqual(proc.returncode, 0)

        report = json.loads((self.evidence_dir / "report.json").read_text(encoding="utf-8"))
        boundary = json.loads(
            (self.evidence_dir / "proofd_observability_boundary_report.json").read_text(
                encoding="utf-8"
            )
        )
        matrix = json.loads(
            (self.evidence_dir / "proofd_observability_negative_matrix.json").read_text(
                encoding="utf-8"
            )
        )

        self.assertEqual(report.get("verdict"), "PASS")
        self.assertEqual(report.get("violations_count"), 0)
        self.assertEqual(boundary.get("status"), "PASS")
        self.assertTrue(boundary.get("artifact_backed_ok") is True)
        self.assertTrue(boundary.get("read_only_namespace_ok") is True)
        self.assertTrue(boundary.get("unsupported_query_fail_closed_ok") is True)
        self.assertTrue(boundary.get("allowed_incident_filter_ok") is True)
        self.assertTrue(boundary.get("payload_non_authoritative_ok") is True)
        self.assertTrue(boundary.get("payload_control_plane_free_ok") is True)
        self.assertGreaterEqual(boundary.get("endpoint_count", 0), 17)
        self.assertEqual(boundary.get("payload_field_hits"), [])
        self.assertEqual(matrix.get("status"), "PASS")
        self.assertEqual(matrix.get("case_count"), 6)
        case_ids = {case.get("case_id") for case in matrix.get("cases", [])}
        self.assertEqual(
            case_ids,
            {"P13-NEG-01", "P13-NEG-02", "P13-NEG-03", "P13-NEG-04", "P13-NEG-13", "P13-NEG-14"},
        )
        self.assertTrue((self.evidence_dir / "violations.txt").is_file())
        self.assertEqual(
            (self.evidence_dir / "violations.txt").read_text(encoding="utf-8"),
            "",
        )

    def test_gate_fails_when_payload_exposes_truth_selection_field(self) -> None:
        self._write_fixture(graph_extra={"selected_truth": "TRUSTED"})
        proc = self._run_gate()
        self.assertEqual(proc.returncode, 2)

        report = json.loads((self.evidence_dir / "report.json").read_text(encoding="utf-8"))
        boundary = json.loads(
            (self.evidence_dir / "proofd_observability_boundary_report.json").read_text(
                encoding="utf-8"
            )
        )
        matrix = json.loads(
            (self.evidence_dir / "proofd_observability_negative_matrix.json").read_text(
                encoding="utf-8"
            )
        )

        self.assertEqual(report.get("verdict"), "FAIL")
        self.assertEqual(boundary.get("status"), "FAIL")
        self.assertTrue(boundary.get("payload_non_authoritative_ok") is False)
        self.assertIn(
            "forbidden_truth_or_authority_field_exposed",
            boundary.get("violations", []),
        )
        neg13 = next(
            case for case in matrix.get("cases", []) if case.get("case_id") == "P13-NEG-13"
        )
        hits = neg13.get("forbidden_field_hits", [])
        self.assertTrue(any(hit.get("field") == "selected_truth" for hit in hits))

    def test_gate_fails_when_payload_exposes_actionable_control_signal(self) -> None:
        self._write_fixture(graph_extra={"recommended_action": "suppress_node"})
        proc = self._run_gate()
        self.assertEqual(proc.returncode, 2)

        boundary = json.loads(
            (self.evidence_dir / "proofd_observability_boundary_report.json").read_text(
                encoding="utf-8"
            )
        )
        matrix = json.loads(
            (self.evidence_dir / "proofd_observability_negative_matrix.json").read_text(
                encoding="utf-8"
            )
        )

        self.assertTrue(boundary.get("payload_control_plane_free_ok") is False)
        self.assertIn(
            "forbidden_control_plane_field_exposed",
            boundary.get("violations", []),
        )
        neg14 = next(
            case for case in matrix.get("cases", []) if case.get("case_id") == "P13-NEG-14"
        )
        hits = neg14.get("forbidden_field_hits", [])
        self.assertTrue(any(hit.get("field") == "recommended_action" for hit in hits))

    def _run_gate(self) -> subprocess.CompletedProcess[bytes]:
        return subprocess.run(
            [
                "bash",
                str(self.script),
                "--evidence-dir",
                str(self.evidence_dir),
                "--artifact-root",
                str(self.artifact_root),
                "--run-id",
                self.RUN_ID,
            ],
            cwd=self.repo_root,
            check=False,
        )

    def _write_fixture(self, graph_extra: dict | None = None) -> None:
        graph_extra = graph_extra or {}
        self.artifact_root.mkdir(parents=True, exist_ok=True)
        run_dir = self.artifact_root / self.RUN_ID
        run_dir.mkdir(parents=True, exist_ok=True)

        artifacts = {
            "parity_report.json": {
                "status": "PASS",
                "node_count": 3,
                "surface_partition_count": 1,
            },
            "parity_determinism_incidents.json": {
                "node_count": 3,
                "surface_partition_count": 1,
                "determinism_incident_count": 1,
                "severity_counts": {"pure_determinism_failure": 1},
                "incidents": [
                    {
                        "incident_id": "sha256:a",
                        "surface_key": "surface-a",
                        "severity": "pure_determinism_failure",
                        "nodes": ["node-a", "node-b"],
                    }
                ],
            },
            "parity_drift_attribution_report.json": {
                "status": "PASS",
                "node_count": 3,
                "partitions": [],
            },
            "parity_convergence_report.json": {
                "status": "PASS",
                "node_count": 3,
                "surface_partition_count": 2,
                "clusters": [{"cluster_id": "cluster-a", "node_count": 2}],
            },
            "failure_matrix.json": {
                "status": "PASS",
                "rows": [],
            },
            "parity_authority_drift_topology.json": {
                "status": "PASS",
                "topology": {
                    "node_count": 3,
                    "authority_cluster_count": 2,
                    "dominant_authority_chain_id": "chain-a",
                },
            },
            "parity_authority_suppression_report.json": {
                "status": "PASS",
                "suppression": {
                    "suppressed_drift_count": 1,
                    "rule_counts": {"historical_shadow": 1},
                },
            },
            "parity_incident_graph.json": {
                "status": "PASS",
                "graph": {"node_count": 3, "edge_count": 2, "incident_count": 1},
                **graph_extra,
            },
        }

        for name, payload in artifacts.items():
            encoded = json.dumps(payload, indent=2, sort_keys=True)
            (self.artifact_root / name).write_text(encoded, encoding="utf-8")
            (run_dir / name).write_text(encoded, encoding="utf-8")


if __name__ == "__main__":
    unittest.main()
