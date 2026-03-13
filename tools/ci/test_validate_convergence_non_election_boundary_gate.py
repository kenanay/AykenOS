#!/usr/bin/env python3
"""Black-box tests for gate_convergence_non_election_boundary.sh."""

from __future__ import annotations

import json
import subprocess
import tempfile
import unittest
from pathlib import Path


class ConvergenceNonElectionBoundaryGateTest(unittest.TestCase):
    def setUp(self) -> None:
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)
        self.repo_root = Path(__file__).resolve().parents[2]
        self.script = (
            self.repo_root / "scripts" / "ci" / "gate_convergence_non_election_boundary.sh"
        )
        self.artifact_root = self.root / "artifact-root"
        self.evidence_dir = self.root / "convergence-non-election-boundary"
        self.artifact_root.mkdir(parents=True, exist_ok=True)

    def tearDown(self) -> None:
        self.tmp.cleanup()

    def test_gate_passes_for_descriptive_convergence_artifacts(self) -> None:
        self._write_fixture()
        proc = self._run_gate()
        self.assertEqual(proc.returncode, 0, proc.stderr)

        report = json.loads((self.evidence_dir / "report.json").read_text(encoding="utf-8"))
        detail = json.loads(
            (self.evidence_dir / "convergence_non_election_report.json").read_text(
                encoding="utf-8"
            )
        )

        self.assertEqual(report.get("verdict"), "PASS")
        self.assertEqual(detail.get("status"), "PASS")
        self.assertEqual(detail.get("forbidden_field_hits"), [])
        self.assertTrue(
            any(
                check.get("field") == "global_status" and check.get("status") == "PASS"
                for check in detail.get("semantic_contract_checks", [])
            )
        )

    def test_gate_fails_when_convergence_artifact_selects_cluster(self) -> None:
        self._write_fixture(extra_convergence={"winning_cluster": "cluster_1"})
        proc = self._run_gate()
        self.assertEqual(proc.returncode, 2)

        detail = json.loads(
            (self.evidence_dir / "convergence_non_election_report.json").read_text(
                encoding="utf-8"
            )
        )
        hits = detail.get("forbidden_field_hits", [])
        self.assertTrue(any(hit.get("field") == "winning_cluster" for hit in hits))

    def test_gate_fails_when_global_status_drifts_into_finality(self) -> None:
        self._write_fixture(global_status="N_PARITY_FINAL_ACCEPTED")
        proc = self._run_gate()
        self.assertEqual(proc.returncode, 2)

        violations = (self.evidence_dir / "violations.txt").read_text(encoding="utf-8")
        self.assertIn("invalid_global_status:parity_convergence_report.json", violations)
        self.assertIn("N_PARITY_FINAL_ACCEPTED", violations)

    def test_gate_fails_when_derivation_drifts_into_majority_selection(self) -> None:
        self._write_fixture(cluster_derivation="majority_vote_cluster_selection")
        proc = self._run_gate()
        self.assertEqual(proc.returncode, 2)

        violations = (self.evidence_dir / "violations.txt").read_text(encoding="utf-8")
        self.assertIn("invalid_derivation_value:parity_convergence_report.json", violations)
        self.assertIn("majority_vote_cluster_selection", violations)

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

    def _write_fixture(
        self,
        *,
        extra_convergence: dict | None = None,
        global_status: str = "N_PARITY_CONSISTENCY_SPLIT",
        cluster_derivation: str = "node_parity_outcome_dk_partitions",
    ) -> None:
        extra_convergence = extra_convergence or {}
        convergence_payload = {
            "status": "PASS",
            "surface": "n-node-convergence",
            "cluster_derivation": cluster_derivation,
            "edge_match_cluster_derivation": "pairwise_match_graph_connected_components",
            "node_count": 3,
            "edge_count": 2,
            "surface_partition_count": 2,
            "outcome_partition_count": 2,
            "largest_surface_partition_size": 2,
            "largest_outcome_cluster_size": 2,
            "surface_consistency_ratio": 0.66,
            "outcome_convergence_ratio": 0.66,
            "determinism_violation_present": False,
            "determinism_conflict_surface_count": 0,
            "global_status": global_status,
            "surface_partitions": [{"partition_id": "partition_1", "size": 2}],
            "outcome_partitions": [{"partition_id": "partition_1", "size": 2}],
            "edge_match_clusters": [{"cluster_id": "cluster_1", "size": 2}],
            "node_outcomes": [{"node_id": "node-a"}],
            **extra_convergence,
        }
        drift_payload = {
            "status": "PASS",
            "node_count": 3,
            "surface_partition_count": 2,
            "outcome_partition_count": 2,
            "historical_authority_island_count": 1,
            "insufficient_evidence_island_count": 1,
            "primary_cause_counts": {
                "authority_historical_only": 1,
                "insufficient_evidence": 1,
            },
            "historical_authority_islands": [
                {
                    "island_id": "historical_authority",
                    "island_type": "authority_historical_only",
                    "node_count": 1,
                    "node_ids": ["node-historical"],
                }
            ],
            "insufficient_evidence_islands": [
                {
                    "island_id": "insufficient_evidence",
                    "island_type": "insufficient_evidence",
                    "node_count": 1,
                    "node_ids": ["node-insufficient"],
                }
            ],
            "partition_reports": [
                {
                    "partition_id": "partition_1",
                    "node_ids": ["node-a", "node-b"],
                    "primary_cause": "context_drift",
                    "secondary_causes": [],
                    "verdict_split": False,
                }
            ],
        }

        (self.artifact_root / "parity_convergence_report.json").write_text(
            json.dumps(convergence_payload, indent=2, sort_keys=True) + "\n",
            encoding="utf-8",
        )
        (self.artifact_root / "parity_drift_attribution_report.json").write_text(
            json.dumps(drift_payload, indent=2, sort_keys=True) + "\n",
            encoding="utf-8",
        )


if __name__ == "__main__":
    unittest.main()
