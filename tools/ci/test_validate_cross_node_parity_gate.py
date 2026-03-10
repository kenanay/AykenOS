#!/usr/bin/env python3
"""Black-box tests for gate_cross_node_parity.sh."""

from __future__ import annotations

import json
import subprocess
import tempfile
import unittest
from pathlib import Path


class CrossNodeParityGateTest(unittest.TestCase):
    def setUp(self) -> None:
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)
        self.repo_root = Path(__file__).resolve().parents[2]
        self.evidence_dir = self.root / "cross-node-parity"
        self.script = self.repo_root / "scripts/ci/gate_cross_node_parity.sh"

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
        parity_report = json.loads(
            (self.evidence_dir / "parity_report.json").read_text(encoding="utf-8")
        )
        consistency_report = json.loads(
            (self.evidence_dir / "parity_consistency_report.json").read_text(
                encoding="utf-8"
            )
        )
        determinism_report = json.loads(
            (self.evidence_dir / "parity_determinism_report.json").read_text(
                encoding="utf-8"
            )
        )
        determinism_incidents = json.loads(
            (self.evidence_dir / "parity_determinism_incidents.json").read_text(
                encoding="utf-8"
            )
        )
        incident_graph = json.loads(
            (self.evidence_dir / "parity_incident_graph.json").read_text(
                encoding="utf-8"
            )
        )
        authority_topology = json.loads(
            (self.evidence_dir / "parity_authority_drift_topology.json").read_text(
                encoding="utf-8"
            )
        )
        authority_suppression = json.loads(
            (self.evidence_dir / "parity_authority_suppression_report.json").read_text(
                encoding="utf-8"
            )
        )
        convergence_report = json.loads(
            (self.evidence_dir / "parity_convergence_report.json").read_text(
                encoding="utf-8"
            )
        )
        drift_report = json.loads(
            (self.evidence_dir / "parity_drift_attribution_report.json").read_text(
                encoding="utf-8"
            )
        )
        failure_matrix = json.loads(
            (self.evidence_dir / "failure_matrix.json").read_text(encoding="utf-8")
        )

        def find_partition(node_id: str) -> dict:
            for partition in drift_report.get("partition_reports", []):
                if node_id in partition.get("node_ids", []):
                    return partition
            self.fail(f"missing drift partition for {node_id}")

        def find_island(island_type: str) -> dict:
            for island in drift_report.get(f"{island_type}_islands", []):
                return island
            self.fail(f"missing {island_type} island")

        self.assertEqual(report.get("verdict"), "PASS")
        self.assertEqual(report.get("violations_count"), 0)
        self.assertEqual(parity_report.get("status"), "PASS")
        self.assertEqual(consistency_report.get("status"), "PASS")
        self.assertEqual(determinism_report.get("status"), "PASS")
        self.assertEqual(determinism_incidents.get("status"), "PASS")
        self.assertEqual(incident_graph.get("status"), "PASS")
        self.assertEqual(authority_topology.get("status"), "PASS")
        self.assertEqual(authority_suppression.get("status"), "PASS")
        self.assertEqual(convergence_report.get("status"), "PASS")
        self.assertEqual(drift_report.get("status"), "PASS")
        self.assertEqual(parity_report.get("row_count"), 10)
        self.assertEqual(consistency_report.get("row_count"), 9)
        self.assertEqual(determinism_report.get("row_count"), 1)
        self.assertEqual(convergence_report.get("node_count"), 13)
        self.assertEqual(convergence_report.get("edge_count"), 10)
        self.assertEqual(convergence_report.get("unique_subject_count"), 2)
        self.assertEqual(convergence_report.get("unique_context_count"), 4)
        self.assertEqual(convergence_report.get("unique_authority_count"), 4)
        self.assertEqual(convergence_report.get("unique_outcome_count"), 9)
        self.assertEqual(convergence_report.get("surface_partition_count"), 8)
        self.assertEqual(convergence_report.get("outcome_partition_count"), 9)
        self.assertEqual(convergence_report.get("largest_surface_partition_size"), 5)
        self.assertEqual(convergence_report.get("largest_outcome_cluster_size"), 4)
        self.assertEqual(convergence_report.get("historical_only_node_count"), 2)
        self.assertEqual(convergence_report.get("insufficient_evidence_node_count"), 1)
        self.assertTrue(convergence_report.get("determinism_violation_present") is True)
        self.assertEqual(convergence_report.get("determinism_conflict_surface_count"), 1)
        self.assertEqual(
            convergence_report.get("global_status"),
            "N_PARITY_DETERMINISM_VIOLATION",
        )
        self.assertEqual(
            convergence_report.get("cluster_derivation"),
            "node_parity_outcome_dk_partitions",
        )
        self.assertEqual(drift_report.get("node_count"), 13)
        self.assertEqual(drift_report.get("surface_partition_count"), 8)
        self.assertEqual(drift_report.get("outcome_partition_count"), 9)
        self.assertEqual(drift_report.get("historical_authority_island_count"), 1)
        self.assertEqual(drift_report.get("insufficient_evidence_island_count"), 1)
        self.assertEqual(
            drift_report.get("primary_cause_counts", {}).get("verdict_drift"), 1
        )
        self.assertEqual(
            drift_report.get("primary_cause_counts", {}).get("subject_drift"), 1
        )
        self.assertEqual(
            drift_report.get("primary_cause_counts", {}).get("context_drift"), 2
        )
        self.assertEqual(
            drift_report.get("primary_cause_counts", {}).get("authority_scope_drift"), 1
        )
        self.assertEqual(
            drift_report.get("primary_cause_counts", {}).get("authority_chain_drift"), 1
        )
        self.assertEqual(
            drift_report.get("primary_cause_counts", {}).get("authority_historical_only"),
            1,
        )
        self.assertEqual(
            drift_report.get("primary_cause_counts", {}).get("insufficient_evidence"), 1
        )
        historical_island = find_island("historical_authority")
        self.assertEqual(historical_island.get("island_type"), "authority_historical_only")
        self.assertEqual(historical_island.get("node_count"), 2)
        self.assertEqual(
            historical_island.get("node_ids"),
            ["node-d-historical", "node-e-historical"],
        )
        insufficient_island = find_island("insufficient_evidence")
        self.assertEqual(insufficient_island.get("island_type"), "insufficient_evidence")
        self.assertEqual(insufficient_island.get("node_count"), 1)
        self.assertEqual(
            insufficient_island.get("node_ids"),
            ["node-f-insufficient"],
        )
        self.assertEqual(
            convergence_report.get("edge_match_cluster_derivation"),
            "pairwise_match_graph_connected_components",
        )
        self.assertEqual(
            convergence_report.get("conflict_summary", {}).get(
                "determinism_violation_edges"
            ),
            1,
        )
        self.assertEqual(
            convergence_report.get("conflict_summary", {}).get("subject_mismatch_edges"),
            1,
        )
        self.assertEqual(
            convergence_report.get("conflict_summary", {}).get("context_mismatch_edges"),
            2,
        )
        self.assertEqual(
            convergence_report.get("conflict_summary", {}).get("verifier_mismatch_edges"),
            2,
        )
        self.assertEqual(
            convergence_report.get("conflict_summary", {}).get(
                "determinism_conflict_surface_count"
            ),
            1,
        )
        self.assertEqual(parity_report.get("authority_chain_id_mismatch_rows"), 2)
        self.assertEqual(
            parity_report.get("effective_authority_scope_mismatch_rows"), 1
        )
        self.assertEqual(
            consistency_report.get("status_counts", {}).get("PARITY_VERIFIER_MISMATCH"), 2
        )
        self.assertTrue(determinism_report.get("determinism_violation_present") is True)
        self.assertEqual(determinism_report.get("determinism_violation_count"), 1)
        self.assertEqual(determinism_report.get("conflict_surface_count"), 1)
        self.assertTrue(determinism_report.get("false_determinism_guard_active") is True)
        self.assertEqual(
            determinism_report.get("severity_counts", {}).get("pure_determinism_failure"), 1
        )
        self.assertEqual(determinism_report.get("suppressed_incident_count"), 0)
        self.assertEqual(
            determinism_report.get("determinism_incidents_path"),
            "parity_determinism_incidents.json",
        )
        self.assertEqual(determinism_incidents.get("node_count"), 13)
        self.assertEqual(determinism_incidents.get("surface_partition_count"), 8)
        self.assertEqual(determinism_incidents.get("determinism_incident_count"), 1)
        self.assertTrue(determinism_incidents.get("false_determinism_guard_active") is True)
        self.assertEqual(
            determinism_incidents.get("severity_counts", {}).get("pure_determinism_failure"),
            1,
        )
        self.assertEqual(determinism_incidents.get("suppressed_incident_count"), 0)
        self.assertEqual(determinism_incidents.get("suppressed_incidents"), [])
        self.assertEqual(
            determinism_incidents.get("incidents", [{}])[0].get("drift_class"),
            "determinism_failure",
        )
        self.assertEqual(
            determinism_incidents.get("incidents", [{}])[0].get("severity"),
            "pure_determinism_failure",
        )
        self.assertTrue(
            determinism_incidents.get("incidents", [{}])[0]
            .get("incident_id", "")
            .startswith("sha256:")
        )
        self.assertEqual(
            determinism_incidents.get("incidents", [{}])[0].get("node_count"), 5
        )
        self.assertEqual(
            determinism_incidents.get("incidents", [{}])[0].get("outcome_partition_count"),
            2,
        )
        self.assertTrue(
            determinism_incidents.get("incidents", [{}])[0].get("subject_equal") is True
        )
        self.assertTrue(
            determinism_incidents.get("incidents", [{}])[0].get("context_equal") is True
        )
        self.assertTrue(
            determinism_incidents.get("incidents", [{}])[0].get("authority_equal") is True
        )
        self.assertIn(
            "node-g-verdict-drift",
            determinism_incidents.get("incidents", [{}])[0].get("nodes", []),
        )
        incident_verdicts = {
            verdict
            for partition in determinism_incidents.get("incidents", [{}])[0].get(
                "outcome_partitions", []
            )
            for verdict in partition.get("verdicts", [])
        }
        self.assertIn("REJECTED_BY_POLICY", incident_verdicts)
        self.assertIn("TRUSTED", incident_verdicts)
        self.assertEqual(
            determinism_report.get("conflict_pairs", [{}])[0].get("scenario"),
            "p14-18-verdict-mismatch-guard",
        )
        self.assertTrue(
            determinism_report.get("conflict_pairs", [{}])[0].get("same_subject") is True
        )
        self.assertTrue(
            determinism_report.get("conflict_pairs", [{}])[0].get("same_context") is True
        )
        self.assertTrue(
            determinism_report.get("conflict_pairs", [{}])[0].get("same_authority") is True
        )
        self.assertEqual(
            parity_report.get("consistency_report_path"), "parity_consistency_report.json"
        )
        self.assertEqual(
            parity_report.get("determinism_report_path"), "parity_determinism_report.json"
        )
        self.assertEqual(
            parity_report.get("determinism_incidents_path"),
            "parity_determinism_incidents.json",
        )
        self.assertEqual(
            parity_report.get("convergence_report_path"), "parity_convergence_report.json"
        )
        self.assertEqual(
            parity_report.get("drift_attribution_report_path"),
            "parity_drift_attribution_report.json",
        )
        self.assertEqual(
            parity_report.get("incident_graph_path"),
            "parity_incident_graph.json",
        )
        self.assertEqual(
            parity_report.get("authority_drift_topology_path"),
            "parity_authority_drift_topology.json",
        )
        self.assertEqual(
            parity_report.get("authority_suppression_report_path"),
            "parity_authority_suppression_report.json",
        )
        self.assertEqual(incident_graph.get("graph", {}).get("node_count"), 13)
        self.assertEqual(incident_graph.get("graph", {}).get("incident_count"), 1)
        self.assertEqual(incident_graph.get("graph", {}).get("edge_count"), 10)
        self.assertEqual(
            incident_graph.get("graph", {}).get("incidents", [{}])[0].get("severity"),
            "pure_determinism_failure",
        )
        self.assertIn(
            "node-g-verdict-drift",
            incident_graph.get("graph", {}).get("incidents", [{}])[0].get("nodes", []),
        )
        self.assertEqual(authority_topology.get("topology", {}).get("node_count"), 13)
        self.assertEqual(
            authority_topology.get("topology", {}).get("authority_cluster_count"), 4
        )
        self.assertTrue(
            authority_topology.get("topology", {})
            .get("dominant_authority_chain_id", "")
            .startswith("sha256:")
        )
        self.assertEqual(
            authority_topology.get("topology", {}).get("drifted_node_count"), 2
        )
        self.assertEqual(
            authority_topology.get("topology", {}).get("historical_only_node_count"), 2
        )
        self.assertEqual(
            authority_topology.get("topology", {}).get("unresolved_node_count"), 0
        )
        self.assertEqual(
            authority_topology.get("topology", {}).get("clusters", [{}])[0].get("kind"),
            "current",
        )
        self.assertEqual(
            authority_topology.get("topology", {}).get("clusters", [{}])[0].get("node_count"),
            9,
        )
        self.assertEqual(
            authority_topology.get("topology", {}).get("clusters", [{}])[1].get("kind"),
            "historical_only",
        )
        topology_node_ids = {
            node_id
            for cluster in authority_topology.get("topology", {}).get("clusters", [])
            for node_id in cluster.get("node_ids", [])
        }
        self.assertIn("node-c-alt-root", topology_node_ids)
        self.assertIn("node-scope-scope-drift", topology_node_ids)
        self.assertTrue(
            authority_suppression.get("suppression", {}).get("suppression_guard_active")
            is True
        )
        self.assertEqual(
            authority_suppression.get("suppression", {}).get("suppressed_drift_count"), 0
        )
        self.assertEqual(
            authority_suppression.get("suppression", {})
            .get("rule_counts", {})
            .get("historical_shadow"),
            None,
        )
        self.assertEqual(
            authority_suppression.get("suppression", {})
            .get("suppressed_drifts"),
            [],
        )
        self.assertEqual(
            convergence_report.get("surface_partitions", [{}])[0].get("size"), 5
        )
        self.assertEqual(
            convergence_report.get("outcome_partitions", [{}])[0].get("size"), 4
        )
        self.assertEqual(
            convergence_report.get("node_outcomes", [{}])[0].get("node_id"),
            "node-a-current",
        )
        self.assertEqual(
            find_partition("node-g-verdict-drift").get("primary_cause"),
            "verdict_drift",
        )
        self.assertTrue(find_partition("node-g-verdict-drift").get("verdict_split") is True)
        self.assertEqual(
            find_partition("node-j-subject-drift").get("primary_cause"),
            "subject_drift",
        )
        self.assertEqual(
            find_partition("node-k-contract-drift").get("primary_cause"),
            "context_drift",
        )
        self.assertEqual(
            find_partition("node-scope-scope-drift").get("primary_cause"),
            "authority_scope_drift",
        )
        self.assertEqual(
            find_partition("node-d-historical").get("primary_cause"),
            "authority_historical_only",
        )
        self.assertIn(
            "authority_chain_drift",
            find_partition("node-d-historical").get("secondary_causes", []),
        )
        self.assertEqual(
            find_partition("node-f-insufficient").get("primary_cause"),
            "insufficient_evidence",
        )
        self.assertIn(
            "context_drift",
            find_partition("node-f-insufficient").get("secondary_causes", []),
        )
        self.assertEqual(len(failure_matrix), 10)
        self.assertEqual(parity_report.get("status_counts", {}).get("PARITY_MATCH"), 2)
        self.assertEqual(
            parity_report.get("status_counts", {}).get("PARITY_SUBJECT_MISMATCH"), 1
        )
        self.assertEqual(
            parity_report.get("status_counts", {}).get("PARITY_CONTEXT_MISMATCH"), 2
        )
        self.assertEqual(
            parity_report.get("status_counts", {}).get("PARITY_VERIFIER_MISMATCH"), 2
        )
        self.assertEqual(
            parity_report.get("status_counts", {}).get("PARITY_HISTORICAL_ONLY"), 1
        )
        self.assertEqual(
            parity_report.get("status_counts", {}).get("PARITY_INSUFFICIENT_EVIDENCE"), 1
        )
        self.assertEqual(
            parity_report.get("status_counts", {}).get("PARITY_VERDICT_MISMATCH"), 1
        )
        self.assertEqual(
            failure_matrix[0].get("scenario"), "p14-01-baseline-identical-nodes"
        )
        self.assertEqual(failure_matrix[0].get("parity_status"), "PARITY_MATCH")
        self.assertEqual(
            failure_matrix[1].get("scenario"), "p14-05-overlay-hash-drift-same-bundle"
        )
        self.assertEqual(failure_matrix[1].get("parity_status"), "PARITY_SUBJECT_MISMATCH")
        self.assertEqual(failure_matrix[1].get("subject_drift_surface"), "trust_overlay_hash")
        self.assertEqual(
            failure_matrix[2].get("scenario"), "p14-10-verification-context-id-drift"
        )
        self.assertEqual(failure_matrix[2].get("parity_status"), "PARITY_CONTEXT_MISMATCH")
        self.assertEqual(
            failure_matrix[3].get("scenario"), "p14-12-verifier-contract-version-drift"
        )
        self.assertEqual(
            failure_matrix[3].get("parity_status"), "PARITY_CONTEXT_MISMATCH"
        )
        self.assertEqual(
            failure_matrix[3].get("context_drift_surface"), "verifier_contract_version"
        )
        self.assertEqual(
            failure_matrix[4].get("scenario"), "p14-13-different-trusted-root-set"
        )
        self.assertEqual(failure_matrix[4].get("parity_status"), "PARITY_VERIFIER_MISMATCH")
        self.assertEqual(failure_matrix[4].get("authority_chain_id_equal"), False)
        self.assertEqual(
            failure_matrix[5].get("scenario"), "p14-15-authority-scope-drift"
        )
        self.assertEqual(failure_matrix[5].get("parity_status"), "PARITY_VERIFIER_MISMATCH")
        self.assertEqual(
            failure_matrix[5].get("authority_drift_surface"), "effective_authority_scope"
        )
        self.assertEqual(
            failure_matrix[5].get("effective_authority_scope_equal"), False
        )
        self.assertEqual(
            failure_matrix[6].get("scenario"), "p14-16-historical-only-authority"
        )
        self.assertEqual(failure_matrix[6].get("parity_status"), "PARITY_HISTORICAL_ONLY")
        self.assertEqual(
            failure_matrix[7].get("scenario"), "p14-19-insufficient-evidence"
        )
        self.assertEqual(
            failure_matrix[7].get("parity_status"), "PARITY_INSUFFICIENT_EVIDENCE"
        )
        self.assertEqual(
            failure_matrix[8].get("scenario"), "p14-18-verdict-mismatch-guard"
        )
        self.assertEqual(failure_matrix[8].get("parity_status"), "PARITY_VERDICT_MISMATCH")
        self.assertTrue(failure_matrix[8].get("determinism_guard") is True)
        self.assertEqual(
            failure_matrix[9].get("scenario"), "p14-20-receipt-absent-parity-artifact"
        )
        self.assertEqual(failure_matrix[9].get("parity_status"), "PARITY_MATCH")
        self.assertTrue(failure_matrix[9].get("receipt_present") is False)
        self.assertEqual(
            failure_matrix[9].get("parity_artifact_form"), "local_verification_outcome"
        )
        self.assertEqual(
            parity_report.get("receipt_absent_artifact_form"), "local_verification_outcome"
        )
        self.assertTrue((self.evidence_dir / "scenario_reports").is_dir())
        self.assertTrue((self.evidence_dir / "parity_consistency_report.json").is_file())
        self.assertTrue((self.evidence_dir / "parity_determinism_report.json").is_file())
        self.assertTrue((self.evidence_dir / "parity_determinism_incidents.json").is_file())
        self.assertTrue((self.evidence_dir / "parity_incident_graph.json").is_file())
        self.assertTrue((self.evidence_dir / "parity_authority_drift_topology.json").is_file())
        self.assertTrue((self.evidence_dir / "parity_authority_suppression_report.json").is_file())
        self.assertTrue((self.evidence_dir / "parity_convergence_report.json").is_file())
        self.assertTrue((self.evidence_dir / "parity_drift_attribution_report.json").is_file())
        self.assertTrue((self.evidence_dir / "violations.txt").is_file())
        self.assertEqual(
            (self.evidence_dir / "violations.txt").read_text(encoding="utf-8"), ""
        )


if __name__ == "__main__":
    unittest.main()
