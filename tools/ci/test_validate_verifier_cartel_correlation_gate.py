#!/usr/bin/env python3
"""Black-box tests for gate_verifier_cartel_correlation.sh."""

from __future__ import annotations

import json
import subprocess
import tempfile
import unittest
from pathlib import Path


class VerifierCartelCorrelationGateTest(unittest.TestCase):
    def setUp(self) -> None:
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)
        self.repo_root = Path(__file__).resolve().parents[2]
        self.script = (
            self.repo_root / "scripts" / "ci" / "gate_verifier_cartel_correlation.sh"
        )
        self.artifact_root = self.root / "artifacts"
        self.evidence_dir = self.root / "gate"
        self.artifact_root.mkdir(parents=True, exist_ok=True)
        self._write_policy()

    def tearDown(self) -> None:
        self.tmp.cleanup()

    def test_gate_passes_for_independent_verifier_window(self) -> None:
        entries = []
        patterns = {
            "verifier-a": ("lineage-a", "chain-a", "cluster-a", ["PASS", "FAIL", "PASS", "FAIL"]),
            "verifier-b": ("lineage-b", "chain-b", "cluster-b", ["FAIL", "PASS", "PASS", "FAIL"]),
            "verifier-c": ("lineage-c", "chain-c", "cluster-c", ["PASS", "PASS", "FAIL", "FAIL"]),
        }
        ts = 1
        for bundle_idx in range(4):
            bundle_id = f"bundle-{bundle_idx}"
            for verifier_id, (lineage, chain, cluster, verdicts) in patterns.items():
                entries.append(
                    self._entry(
                        ts,
                        bundle_id,
                        verifier_id,
                        f"node-{verifier_id}",
                        chain,
                        lineage,
                        verdicts[bundle_idx],
                        cluster,
                    )
                )
                ts += 1
        self._write_ledger(entries)

        proc = self._run_gate()
        self.assertEqual(proc.returncode, 0, proc.stderr)

        report = json.loads((self.evidence_dir / "report.json").read_text(encoding="utf-8"))
        detail = json.loads(
            (self.evidence_dir / "verifier_cartel_correlation_report.json").read_text(
                encoding="utf-8"
            )
        )
        metrics = json.loads(
            (self.evidence_dir / "cartel_correlation_metrics.json").read_text(
                encoding="utf-8"
            )
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
            str(self.artifact_root / "cartel_correlation_policy.json"),
        )
        self.assertEqual(metrics.get("status"), "PASS")
        self.assertIn("window_counts", metrics)
        self.assertEqual(metrics["metrics"].get("unique_verifier_count"), 3)

    def test_gate_fails_for_same_lineage_high_correlation(self) -> None:
        entries = []
        ts = 1
        shared_verdicts = ["PASS", "FAIL", "PASS", "FAIL"]
        for bundle_idx, verdict in enumerate(shared_verdicts):
            bundle_id = f"bundle-{bundle_idx}"
            entries.append(
                self._entry(
                    ts,
                    bundle_id,
                    "verifier-a",
                    "node-a",
                    "chain-a",
                    "lineage-x",
                    verdict,
                    "cluster-a",
                )
            )
            ts += 1
            entries.append(
                self._entry(
                    ts,
                    bundle_id,
                    "verifier-b",
                    "node-b",
                    "chain-b",
                    "lineage-x",
                    verdict,
                    "cluster-b",
                )
            )
            ts += 1
            entries.append(
                self._entry(
                    ts,
                    bundle_id,
                    "verifier-c",
                    "node-c",
                    "chain-c",
                    "lineage-c",
                    "PASS" if bundle_idx % 2 == 0 else "FAIL",
                    "cluster-c",
                )
            )
            ts += 1
        self._write_ledger(entries)

        proc = self._run_gate()
        self.assertEqual(proc.returncode, 2)

        detail = json.loads(
            (self.evidence_dir / "verifier_cartel_correlation_report.json").read_text(
                encoding="utf-8"
            )
        )
        self.assertEqual(detail.get("status"), "FAIL")
        violations = (self.evidence_dir / "violations.txt").read_text(encoding="utf-8")
        self.assertIn("cartel_correlation_violation:lineage:lineage-x", violations)

    def test_gate_fails_for_execution_cluster_overlap(self) -> None:
        entries = []
        patterns = {
            "verifier-a": ("lineage-a", "chain-a", "cluster-z", ["PASS", "FAIL", "FAIL", "PASS"]),
            "verifier-b": ("lineage-b", "chain-b", "cluster-z", ["FAIL", "PASS", "FAIL", "PASS"]),
            "verifier-c": ("lineage-c", "chain-c", "cluster-z", ["PASS", "PASS", "FAIL", "FAIL"]),
            "verifier-d": ("lineage-d", "chain-d", "cluster-y", ["FAIL", "FAIL", "PASS", "PASS"]),
        }
        ts = 1
        for bundle_idx in range(4):
            bundle_id = f"bundle-{bundle_idx}"
            for verifier_id, (lineage, chain, cluster, verdicts) in patterns.items():
                entries.append(
                    self._entry(
                        ts,
                        bundle_id,
                        verifier_id,
                        f"node-{verifier_id}",
                        chain,
                        lineage,
                        verdicts[bundle_idx],
                        cluster,
                    )
                )
                ts += 1
        self._write_ledger(entries)

        proc = self._run_gate()
        self.assertEqual(proc.returncode, 2)

        cluster_report = json.loads(
            (self.evidence_dir / "cluster_overlap_report.json").read_text(encoding="utf-8")
        )
        self.assertEqual(cluster_report.get("status"), "FAIL")
        violations = (self.evidence_dir / "violations.txt").read_text(encoding="utf-8")
        self.assertIn("cartel_correlation_violation:execution_cluster_overlap:cluster-z", violations)

    def test_gate_fails_closed_when_ledger_is_missing(self) -> None:
        proc = self._run_gate()
        self.assertEqual(proc.returncode, 2)

        detail = json.loads(
            (self.evidence_dir / "verifier_cartel_correlation_report.json").read_text(
                encoding="utf-8"
            )
        )
        self.assertEqual(detail.get("status"), "FAIL")
        self.assertEqual(detail.get("load_failure_stage"), "ledger_load")
        self.assertEqual(
            detail.get("ledger_path"),
            str(self.artifact_root / "verification_diversity_ledger.json"),
        )

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
            "min_shared_events": 3,
            "pairwise_correlation_threshold": 0.95,
            "lineage_conditioned_correlation_threshold": 0.95,
            "authority_chain_conditioned_correlation_threshold": 0.95,
            "max_execution_cluster_overlap_ratio": 0.60,
            "stability_window_runs": 3,
            "stability_window_count": 3,
            "stability_min_high_windows": 2,
            "stability_correlation_threshold": 0.95,
        }
        (self.artifact_root / "cartel_correlation_policy.json").write_text(
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
        subject_bundle_id: str,
        verifier_id: str,
        node_id: str,
        authority_chain_id: str,
        lineage_id: str,
        verdict: str,
        execution_cluster_id: str | None = None,
    ) -> dict[str, object]:
        entry = {
            "ledger_version": 1,
            "entry_id": f"entry-{timestamp}-{verifier_id}",
            "run_id": f"run-{timestamp}",
            "timestamp_unix_ns": timestamp * 1_000_000_000,
            "subject_bundle_id": subject_bundle_id,
            "verification_context_id": "context-a",
            "verification_node_id": node_id,
            "verifier_id": verifier_id,
            "authority_chain_id": authority_chain_id,
            "lineage_id": lineage_id,
            "verdict": verdict,
            "receipt_hash": f"receipt-{timestamp}",
        }
        if execution_cluster_id is not None:
            entry["execution_cluster_id"] = execution_cluster_id
        return entry


if __name__ == "__main__":
    unittest.main()
