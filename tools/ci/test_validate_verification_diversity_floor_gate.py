#!/usr/bin/env python3
"""Black-box tests for gate_verification_diversity_floor.sh."""

from __future__ import annotations

import json
import subprocess
import tempfile
import unittest
from pathlib import Path


class VerificationDiversityFloorGateTest(unittest.TestCase):
    def setUp(self) -> None:
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)
        self.repo_root = Path(__file__).resolve().parents[2]
        self.script = self.repo_root / "scripts" / "ci" / "gate_verification_diversity_floor.sh"
        self.artifact_root = self.root / "artifacts"
        self.evidence_dir = self.root / "gate"
        self.artifact_root.mkdir(parents=True, exist_ok=True)
        self._write_policy()

    def tearDown(self) -> None:
        self.tmp.cleanup()

    def test_gate_passes_for_behaviorally_diverse_window(self) -> None:
        self._write_ledger(
            [
                self._entry(1, "verifier-a", "node-a", "chain-a", "lineage-a", "cluster-1"),
                self._entry(2, "verifier-b", "node-b", "chain-b", "lineage-b", "cluster-1"),
                self._entry(3, "verifier-c", "node-c", "chain-c", "lineage-c", "cluster-2"),
                self._entry(4, "verifier-d", "node-d", "chain-d", "lineage-d", "cluster-2"),
                self._entry(5, "verifier-a", "node-a", "chain-a", "lineage-a", "cluster-1"),
                self._entry(6, "verifier-b", "node-b", "chain-b", "lineage-b", "cluster-1"),
            ]
        )

        proc = self._run_gate()
        self.assertEqual(proc.returncode, 0, proc.stderr)

        report = json.loads((self.evidence_dir / "report.json").read_text(encoding="utf-8"))
        detail = json.loads(
            (self.evidence_dir / "verification_diversity_floor_report.json").read_text(
                encoding="utf-8"
            )
        )
        metrics = json.loads(
            (self.evidence_dir / "diversity_metrics.json").read_text(encoding="utf-8")
        )
        dominance = json.loads(
            (self.evidence_dir / "dominance_analysis.json").read_text(encoding="utf-8")
        )
        clusters = json.loads(
            (self.evidence_dir / "cluster_distribution.json").read_text(encoding="utf-8")
        )

        self.assertEqual(report.get("verdict"), "PASS")
        self.assertEqual(detail.get("status"), "PASS")
        self.assertEqual(detail.get("violations_count"), 0)
        self.assertEqual(
            detail.get("ledger_path"),
            str(self.artifact_root / "verification_diversity_ledger.json"),
        )
        self.assertEqual(
            detail.get("policy_path"),
            str(self.artifact_root / "diversity_policy.json"),
        )
        self.assertGreaterEqual(metrics.get("unique_verifier_count", 0), 4)
        self.assertLess(metrics.get("dominance_ratio", 1.0), 0.4)
        self.assertIn("lineage_dominance_ratio", metrics)
        self.assertIn("authority_chain_dominance_ratio", metrics)
        self.assertIn("execution_cluster_dominance_ratio", metrics)
        self.assertIn("dominant_execution_cluster_id", dominance)
        self.assertEqual(clusters.get("unique_execution_cluster_count"), 2)

    def test_gate_fails_when_verifier_dominance_exceeds_policy(self) -> None:
        self._write_ledger(
            [
                self._entry(1, "verifier-a", "node-a", "chain-a", "lineage-a"),
                self._entry(2, "verifier-a", "node-a", "chain-a", "lineage-a"),
                self._entry(3, "verifier-a", "node-a", "chain-a", "lineage-a"),
                self._entry(4, "verifier-a", "node-a", "chain-a", "lineage-a"),
                self._entry(5, "verifier-a", "node-a", "chain-a", "lineage-a"),
                self._entry(6, "verifier-b", "node-b", "chain-b", "lineage-b"),
                self._entry(7, "verifier-c", "node-c", "chain-c", "lineage-c"),
            ]
        )

        proc = self._run_gate()
        self.assertEqual(proc.returncode, 2)

        violations = (self.evidence_dir / "violations.txt").read_text(encoding="utf-8")
        self.assertIn("diversity_floor_violation:dominance_ratio", violations)

        detail = json.loads(
            (self.evidence_dir / "verification_diversity_floor_report.json").read_text(
                encoding="utf-8"
            )
        )
        self.assertEqual(detail.get("status"), "FAIL")

    def test_gate_reports_empty_window_after_run_limit(self) -> None:
        self._write_ledger(
            [
                self._entry(1, "verifier-a", "node-a", "chain-a", "lineage-a"),
                self._entry(2, "verifier-b", "node-b", "chain-b", "lineage-b"),
            ]
        )

        proc = self._run_gate("--window-runs", "0")
        self.assertEqual(proc.returncode, 2)

        detail = json.loads(
            (self.evidence_dir / "verification_diversity_floor_report.json").read_text(
                encoding="utf-8"
            )
        )
        self.assertEqual(detail.get("empty_reason"), "empty_window_after_run_limit")
        violations = (self.evidence_dir / "violations.txt").read_text(encoding="utf-8")
        self.assertIn("diversity_floor_violation:empty_window_after_run_limit", violations)

    def test_gate_fails_closed_when_ledger_artifact_is_missing(self) -> None:
        proc = self._run_gate()
        self.assertEqual(proc.returncode, 2)

        report = json.loads((self.evidence_dir / "report.json").read_text(encoding="utf-8"))
        self.assertEqual(report.get("verdict"), "FAIL")
        detail = json.loads(
            (self.evidence_dir / "verification_diversity_floor_report.json").read_text(
                encoding="utf-8"
            )
        )
        self.assertEqual(detail.get("load_failure_stage"), "ledger_load")
        self.assertEqual(
            detail.get("ledger_path"),
            str(self.artifact_root / "verification_diversity_ledger.json"),
        )

        violations = (self.evidence_dir / "violations.txt").read_text(encoding="utf-8")
        self.assertIn("missing_or_invalid_ledger", violations)

    def _run_gate(self, *extra_args: str) -> subprocess.CompletedProcess[str]:
        return subprocess.run(
            [
                "bash",
                str(self.script),
                "--evidence-dir",
                str(self.evidence_dir),
                "--artifact-root",
                str(self.artifact_root),
                *extra_args,
            ],
            cwd=self.repo_root,
            check=False,
            capture_output=True,
            text=True,
        )

    def _write_policy(self) -> None:
        payload = {
            "policy_version": 1,
            "window_runs": 20,
            "window_seconds": 3600,
            "min_unique_verifiers": 3,
            "min_unique_verification_nodes": 3,
            "min_unique_authority_chains": 3,
            "min_unique_lineages": 3,
            "max_dominance_ratio": 0.40,
            "min_lineage_entropy": 1.2,
        }
        (self.artifact_root / "diversity_policy.json").write_text(
            json.dumps(payload, indent=2, sort_keys=True) + "\n",
            encoding="utf-8",
        )

    def _write_ledger(self, entries: list[dict[str, object]]) -> None:
        payload = {"entries": entries}
        (self.artifact_root / "verification_diversity_ledger.json").write_text(
            json.dumps(payload, indent=2, sort_keys=True) + "\n",
            encoding="utf-8",
        )

    def _entry(
        self,
        timestamp: int,
        verifier_id: str,
        node_id: str,
        authority_chain_id: str,
        lineage_id: str,
        execution_cluster_id: str | None = None,
    ) -> dict[str, object]:
        entry = {
            "ledger_version": 1,
            "entry_id": f"entry-{timestamp}-{verifier_id}",
            "run_id": f"run-{timestamp}",
            "timestamp_unix_ns": timestamp * 1_000_000_000,
            "subject_bundle_id": "bundle-a",
            "verification_context_id": "context-a",
            "verification_node_id": node_id,
            "verifier_id": verifier_id,
            "authority_chain_id": authority_chain_id,
            "lineage_id": lineage_id,
            "verdict": "PASS",
            "receipt_hash": f"receipt-{timestamp}",
        }
        if execution_cluster_id is not None:
            entry["execution_cluster_id"] = execution_cluster_id
        return entry


if __name__ == "__main__":
    unittest.main()
