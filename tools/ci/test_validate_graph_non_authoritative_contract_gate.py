#!/usr/bin/env python3
"""Black-box tests for gate_graph_non_authoritative_contract.sh."""

from __future__ import annotations

import json
import subprocess
import tempfile
import unittest
from pathlib import Path


class GraphNonAuthoritativeContractGateTest(unittest.TestCase):
    def setUp(self) -> None:
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)
        self.repo_root = Path(__file__).resolve().parents[2]
        self.script = (
            self.repo_root / "scripts" / "ci" / "gate_graph_non_authoritative_contract.sh"
        )
        self.artifact_root = self.root / "artifact-root"
        self.evidence_dir = self.root / "graph-non-authoritative-contract"

    def tearDown(self) -> None:
        self.tmp.cleanup()

    def test_gate_passes_for_descriptive_graph_fields(self) -> None:
        self._write_fixture()
        proc = self._run_gate()
        self.assertEqual(proc.returncode, 0)

        report = json.loads((self.evidence_dir / "report.json").read_text(encoding="utf-8"))
        detail = json.loads(
            (self.evidence_dir / "graph_non_authoritative_report.json").read_text(
                encoding="utf-8"
            )
        )

        self.assertEqual(report.get("verdict"), "PASS")
        self.assertEqual(detail.get("status"), "PASS")
        self.assertEqual(detail.get("forbidden_field_hits"), [])
        self.assertIn(
            "dominant_authority_chain_id",
            detail.get("allowed_descriptive_fields", []),
        )

    def test_gate_fails_when_graph_encodes_truth_inference(self) -> None:
        self._write_fixture(extra_convergence={"consensus_strength": 0.91})
        proc = self._run_gate()
        self.assertEqual(proc.returncode, 2)

        report = json.loads((self.evidence_dir / "report.json").read_text(encoding="utf-8"))
        detail = json.loads(
            (self.evidence_dir / "graph_non_authoritative_report.json").read_text(
                encoding="utf-8"
            )
        )

        self.assertEqual(report.get("verdict"), "FAIL")
        self.assertEqual(detail.get("status"), "FAIL")
        hits = detail.get("forbidden_field_hits", [])
        self.assertTrue(any(hit.get("field") == "consensus_strength" for hit in hits))

    def _run_gate(self) -> subprocess.CompletedProcess[bytes]:
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
        )

    def _write_fixture(self, extra_convergence: dict | None = None) -> None:
        extra_convergence = extra_convergence or {}
        self.artifact_root.mkdir(parents=True, exist_ok=True)

        artifacts = {
            "parity_convergence_report.json": {
                "status": "PASS",
                "surface_partition_count": 2,
                "largest_surface_partition_size": 2,
                "largest_outcome_cluster_size": 2,
                "surface_consistency_ratio": 0.66,
                "outcome_convergence_ratio": 0.66,
                "edge_match_clusters": [{"cluster_id": "cluster-1", "size": 2}],
                **extra_convergence,
            },
            "parity_authority_drift_topology.json": {
                "status": "PASS",
                "topology": {
                    "authority_cluster_count": 2,
                    "dominant_authority_chain_id": "chain-a",
                    "dominant_authority_cluster_key": "cluster-a",
                    "clusters": [
                        {
                            "authority_cluster_key": "cluster-a",
                            "authority_chain_id": "chain-a",
                            "node_count": 2,
                        }
                    ],
                },
            },
            "parity_incident_graph.json": {
                "status": "PASS",
                "graph": {
                    "node_count": 3,
                    "edge_count": 2,
                    "incident_count": 1,
                    "nodes": [{"id": "node-a", "surface_key": "s1", "outcome_key": "o1"}],
                    "edges": [{"from": "node-a", "to": "node-b", "edge_type": "incident"}],
                },
            },
            "parity_consistency_report.json": {
                "status": "PASS",
                "row_count": 3,
                "status_counts": {"PARITY_MATCH": 2, "PARITY_VERDICT_MISMATCH": 1},
            },
        }

        for name, payload in artifacts.items():
            (self.artifact_root / name).write_text(
                json.dumps(payload, indent=2, sort_keys=True),
                encoding="utf-8",
            )


if __name__ == "__main__":
    unittest.main()
