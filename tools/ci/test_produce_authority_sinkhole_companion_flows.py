#!/usr/bin/env python3
"""Black-box tests for produce_authority_sinkhole_companion_flows.sh."""

from __future__ import annotations

import json
import subprocess
import tempfile
import unittest
from pathlib import Path


class AuthoritySinkholeCompanionFlowProducerTest(unittest.TestCase):
    def setUp(self) -> None:
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)
        self.repo_root = Path(__file__).resolve().parents[2]
        self.script = (
            self.repo_root
            / "scripts"
            / "ci"
            / "produce_authority_sinkhole_companion_flows.sh"
        )
        self.artifact_root = self.root / "artifacts"
        self.evidence_dir = self.root / "producer"
        self.artifact_root.mkdir(parents=True, exist_ok=True)

    def tearDown(self) -> None:
        self.tmp.cleanup()

    def test_producer_materializes_replay_and_trust_reports(self) -> None:
        self._write_source("replay_boundary")
        self._write_source("trust_reuse")

        proc = self._run_producer()
        self.assertEqual(proc.returncode, 0, proc.stderr)

        replay_report = json.loads(
            (self.artifact_root / "replay_boundary_flow_report.json").read_text(
                encoding="utf-8"
            )
        )
        trust_report = json.loads(
            (self.artifact_root / "trust_reuse_flow_report.json").read_text(
                encoding="utf-8"
            )
        )
        detail = json.loads(
            (
                self.evidence_dir
                / "authority_sinkhole_companion_flow_materialization_report.json"
            ).read_text(encoding="utf-8")
        )
        self.assertEqual(replay_report.get("status"), "PASS")
        self.assertEqual(trust_report.get("status"), "PASS")
        self.assertEqual(replay_report.get("flow_surface"), "replay_boundary")
        self.assertEqual(trust_report.get("flow_surface"), "trust_reuse")
        self.assertTrue(
            replay_report["events"][0]["event_id"].startswith("sha256:"),
            replay_report["events"][0]["event_id"],
        )
        self.assertEqual(detail.get("status"), "PASS")
        self.assertEqual(detail["metrics"].get("materialized_surface_count"), 2)

    def test_producer_fails_when_no_sources_exist(self) -> None:
        proc = self._run_producer()
        self.assertEqual(proc.returncode, 2)

        detail = json.loads(
            (
                self.evidence_dir
                / "authority_sinkhole_companion_flow_materialization_report.json"
            ).read_text(encoding="utf-8")
        )
        self.assertEqual(detail.get("status"), "FAIL")
        self.assertEqual(detail.get("load_failure_stage"), "source_discovery")

    def _run_producer(self) -> subprocess.CompletedProcess[str]:
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

    def _write_source(self, flow_surface: str) -> None:
        payload = {
            "source_version": 1,
            "flow_surface": flow_surface,
            "run_id": f"{flow_surface}-run",
            "window_model": "append_only_event_stream",
            "events": [
                {
                    "timestamp_unix_ns": 1_000_000_000,
                    "subject_bundle_id": "bundle-a",
                    "verification_context_id": "context-a",
                    "authority_chain_id": "chain-a",
                    "terminal": True,
                    "reused": True,
                    "verification_node_id": "node-a",
                    "verifier_id": "verifier-a",
                    "lineage_id": "lineage-a",
                    "execution_cluster_id": "cluster-a",
                },
                {
                    "timestamp_unix_ns": 2_000_000_000,
                    "subject_bundle_id": "bundle-b",
                    "verification_context_id": "context-a",
                    "authority_chain_id": "chain-b",
                    "terminal": True,
                    "reused": False,
                },
            ],
        }
        (self.artifact_root / f"{flow_surface}_flow_source.json").write_text(
            json.dumps(payload, indent=2, sort_keys=True) + "\n",
            encoding="utf-8",
        )


if __name__ == "__main__":
    unittest.main()
