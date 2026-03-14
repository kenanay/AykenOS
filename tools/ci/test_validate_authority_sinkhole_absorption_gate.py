#!/usr/bin/env python3
"""Black-box tests for gate_authority_sinkhole_absorption.sh."""

from __future__ import annotations

import json
import subprocess
import tempfile
import unittest
from pathlib import Path


class AuthoritySinkholeAbsorptionGateTest(unittest.TestCase):
    def setUp(self) -> None:
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)
        self.repo_root = Path(__file__).resolve().parents[2]
        self.script = (
            self.repo_root / "scripts" / "ci" / "gate_authority_sinkhole_absorption.sh"
        )
        self.producer_script = (
            self.repo_root
            / "scripts"
            / "ci"
            / "produce_authority_sinkhole_companion_flows.sh"
        )
        self.artifact_root = self.root / "artifacts"
        self.evidence_dir = self.root / "gate"
        self.artifact_root.mkdir(parents=True, exist_ok=True)
        self._write_policy()

    def tearDown(self) -> None:
        self.tmp.cleanup()

    def test_gate_passes_for_non_absorbing_authority_flow(self) -> None:
        entries = [
            self._entry(1, "bundle-a", "verifier-a", "node-a", "chain-a", "lineage-a"),
            self._entry(2, "bundle-b", "verifier-b", "node-b", "chain-b", "lineage-b"),
            self._entry(3, "bundle-c", "verifier-c", "node-c", "chain-c", "lineage-c"),
            self._entry(4, "bundle-d", "verifier-d", "node-d", "chain-a", "lineage-d"),
            self._entry(5, "bundle-a", "verifier-e", "node-e", "chain-b", "lineage-e"),
            self._entry(6, "bundle-b", "verifier-f", "node-f", "chain-a", "lineage-f"),
            self._entry(7, "bundle-e", "verifier-g", "node-g", "chain-c", "lineage-g"),
            self._entry(8, "bundle-f", "verifier-h", "node-h", "chain-b", "lineage-h"),
            self._entry(9, "bundle-g", "verifier-i", "node-i", "chain-a", "lineage-i"),
            self._entry(10, "bundle-h", "verifier-j", "node-j", "chain-b", "lineage-j"),
            self._entry(11, "bundle-c", "verifier-k", "node-k", "chain-c", "lineage-k"),
            self._entry(12, "bundle-d", "verifier-l", "node-l", "chain-a", "lineage-l"),
        ]
        self._write_ledger(entries)

        proc = self._run_gate()
        self.assertEqual(proc.returncode, 0, proc.stderr)

        report = json.loads((self.evidence_dir / "report.json").read_text(encoding="utf-8"))
        detail = json.loads(
            (self.evidence_dir / "authority_sinkhole_absorption_report.json").read_text(
                encoding="utf-8"
            )
        )
        basin = json.loads(
            (self.evidence_dir / "basin_absorption_report.json").read_text(encoding="utf-8")
        )
        series = json.loads(
            (self.evidence_dir / "basin_window_series.json").read_text(encoding="utf-8")
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
            str(self.artifact_root / "authority_sinkhole_policy.json"),
        )
        self.assertEqual(basin.get("status"), "PASS")
        self.assertEqual(series.get("status"), "PASS")
        self.assertEqual(len(series.get("windows", [])), 3)
        cross_surface = json.loads(
            (self.evidence_dir / "cross_surface_basin_alignment_report.json").read_text(
                encoding="utf-8"
            )
        )
        self.assertEqual(cross_surface.get("status"), "NOT_EVALUATED")

    def test_gate_fails_for_basin_absorption_and_positive_slope(self) -> None:
        entries = [
            self._entry(1, "bundle-a", "verifier-a", "node-a", "chain-a", "lineage-a"),
            self._entry(2, "bundle-b", "verifier-b", "node-b", "chain-a", "lineage-b"),
            self._entry(3, "bundle-c", "verifier-c", "node-c", "chain-b", "lineage-c"),
            self._entry(4, "bundle-d", "verifier-d", "node-d", "chain-c", "lineage-d"),
            self._entry(5, "bundle-a", "verifier-e", "node-e", "chain-a", "lineage-e"),
            self._entry(6, "bundle-b", "verifier-f", "node-f", "chain-a", "lineage-f"),
            self._entry(7, "bundle-c", "verifier-g", "node-g", "chain-a", "lineage-g"),
            self._entry(8, "bundle-e", "verifier-h", "node-h", "chain-b", "lineage-h"),
            self._entry(9, "bundle-a", "verifier-i", "node-i", "chain-a", "lineage-i"),
            self._entry(10, "bundle-b", "verifier-j", "node-j", "chain-a", "lineage-j"),
            self._entry(11, "bundle-c", "verifier-k", "node-k", "chain-a", "lineage-k"),
            self._entry(12, "bundle-d", "verifier-l", "node-l", "chain-a", "lineage-l"),
        ]
        self._write_ledger(entries)

        proc = self._run_gate()
        self.assertEqual(proc.returncode, 2)

        detail = json.loads(
            (self.evidence_dir / "authority_sinkhole_absorption_report.json").read_text(
                encoding="utf-8"
            )
        )
        basin = json.loads(
            (self.evidence_dir / "basin_absorption_report.json").read_text(encoding="utf-8")
        )
        self.assertEqual(detail.get("status"), "FAIL")
        self.assertEqual(basin.get("status"), "FAIL")

        violations = (self.evidence_dir / "violations.txt").read_text(encoding="utf-8")
        self.assertIn("authority_sinkhole_violation:authority_basin_share:chain-a", violations)
        self.assertIn(
            "authority_sinkhole_violation:authority_basin_reuse_ratio:chain-a", violations
        )
        self.assertIn(
            "authority_sinkhole_violation:basin_dominance_slope:chain-a", violations
        )

    def test_gate_fails_for_cross_surface_basin_alignment_when_stage2_thresholds_are_set(self) -> None:
        self._write_policy(
            {
                "max_cross_surface_basin_alignment_ratio": 0.80,
                "max_replay_boundary_basin_capture_ratio": 1.10,
                "max_trust_reuse_basin_capture_ratio": 1.10,
            }
        )
        entries = [
            self._entry(1, "bundle-a", "verifier-a", "node-a", "chain-a", "lineage-a"),
            self._entry(2, "bundle-b", "verifier-b", "node-b", "chain-b", "lineage-b"),
            self._entry(3, "bundle-c", "verifier-c", "node-c", "chain-c", "lineage-c"),
            self._entry(4, "bundle-d", "verifier-d", "node-d", "chain-a", "lineage-d"),
            self._entry(5, "bundle-a", "verifier-e", "node-e", "chain-a", "lineage-e"),
            self._entry(6, "bundle-b", "verifier-f", "node-f", "chain-a", "lineage-f"),
            self._entry(7, "bundle-c", "verifier-g", "node-g", "chain-b", "lineage-g"),
            self._entry(8, "bundle-d", "verifier-h", "node-h", "chain-a", "lineage-h"),
        ]
        self._write_ledger(entries)
        self._write_companion_source(
            "replay_boundary",
            [
                self._companion_event(11, "bundle-a", "chain-a"),
                self._companion_event(12, "bundle-b", "chain-a"),
                self._companion_event(13, "bundle-d", "chain-a"),
            ],
        )
        self._write_companion_source(
            "trust_reuse",
            [
                self._companion_event(21, "bundle-a", "chain-a"),
                self._companion_event(22, "bundle-b", "chain-a"),
                self._companion_event(23, "bundle-d", "chain-a"),
            ],
        )
        producer = self._run_companion_producer()
        self.assertEqual(producer.returncode, 0, producer.stderr)

        proc = self._run_gate()
        self.assertEqual(proc.returncode, 2, proc.stderr)

        cross_surface = json.loads(
            (self.evidence_dir / "cross_surface_basin_alignment_report.json").read_text(
                encoding="utf-8"
            )
        )
        self.assertEqual(cross_surface.get("status"), "FAIL")
        self.assertAlmostEqual(
            cross_surface["metrics"].get("cross_surface_basin_alignment_ratio"),
            1.0,
        )
        violations = (self.evidence_dir / "violations.txt").read_text(encoding="utf-8")
        self.assertIn(
            "authority_sinkhole_violation:cross_surface_basin_alignment_ratio:chain-a",
            violations,
        )

    def test_gate_fails_closed_when_ledger_is_missing(self) -> None:
        proc = self._run_gate()
        self.assertEqual(proc.returncode, 2)

        detail = json.loads(
            (self.evidence_dir / "authority_sinkhole_absorption_report.json").read_text(
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

    def _run_companion_producer(self) -> subprocess.CompletedProcess[str]:
        return subprocess.run(
            [
                "bash",
                str(self.producer_script),
                "--evidence-dir",
                str(self.root / "producer"),
                "--artifact-root",
                str(self.artifact_root),
            ],
            cwd=self.repo_root,
            check=False,
            capture_output=True,
            text=True,
        )

    def _write_policy(self, updates: dict[str, object] | None = None) -> None:
        payload = {
            "policy_version": 1,
            "window_runs": 12,
            "window_seconds": 7200,
            "min_repeated_subject_groups": 2,
            "max_authority_basin_share": 0.70,
            "max_authority_basin_reuse_ratio": 0.75,
            "max_authority_basin_repeat_capture_rate": 0.60,
            "min_alternate_path_decay_ratio": 0.25,
            "series_window_runs": 4,
            "series_window_count": 3,
            "max_basin_dominance_slope": 0.10,
        }
        if updates:
            payload.update(updates)
        (self.artifact_root / "authority_sinkhole_policy.json").write_text(
            json.dumps(payload, indent=2, sort_keys=True) + "\n",
            encoding="utf-8",
        )

    def _write_ledger(self, entries: list[dict[str, object]]) -> None:
        payload = {"entries": entries}
        (self.artifact_root / "verification_diversity_ledger.json").write_text(
            json.dumps(payload, indent=2, sort_keys=True) + "\n",
            encoding="utf-8",
        )

    def _write_companion_source(
        self, flow_surface: str, events: list[dict[str, object]]
    ) -> None:
        payload = {
            "source_version": 1,
            "flow_surface": flow_surface,
            "run_id": f"{flow_surface}-run",
            "window_model": "append_only_event_stream",
            "events": events,
        }
        (self.artifact_root / f"{flow_surface}_flow_source.json").write_text(
            json.dumps(payload, indent=2, sort_keys=True) + "\n",
            encoding="utf-8",
        )

    def _companion_event(
        self, timestamp: int, subject_bundle_id: str, authority_chain_id: str
    ) -> dict[str, object]:
        return {
            "timestamp_unix_ns": timestamp * 1_000_000_000,
            "subject_bundle_id": subject_bundle_id,
            "verification_context_id": "context-a",
            "authority_chain_id": authority_chain_id,
            "terminal": True,
            "reused": True,
            "verification_node_id": "node-stage2",
            "verifier_id": "verifier-stage2",
            "lineage_id": "lineage-stage2",
            "execution_cluster_id": "cluster-stage2",
        }

    def _entry(
        self,
        timestamp: int,
        subject_bundle_id: str,
        verifier_id: str,
        node_id: str,
        authority_chain_id: str,
        lineage_id: str,
    ) -> dict[str, object]:
        return {
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
            "verdict": "PASS",
            "receipt_hash": f"receipt-{timestamp}",
        }


if __name__ == "__main__":
    unittest.main()
