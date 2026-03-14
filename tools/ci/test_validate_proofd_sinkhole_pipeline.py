#!/usr/bin/env python3
"""End-to-end black-box test for proofd Stage-2 sinkhole pipeline."""

from __future__ import annotations

import json
import subprocess
import tempfile
import unittest
from pathlib import Path


class ProofdSinkholePipelineTest(unittest.TestCase):
    def setUp(self) -> None:
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)
        self.repo_root = Path(__file__).resolve().parents[2]
        self.proofd_script = self.repo_root / "scripts" / "ci" / "gate_proofd_service.sh"
        self.producer_script = (
            self.repo_root
            / "scripts"
            / "ci"
            / "produce_authority_sinkhole_companion_flows.sh"
        )
        self.sinkhole_script = (
            self.repo_root
            / "scripts"
            / "ci"
            / "gate_authority_sinkhole_absorption.sh"
        )
        self.proofd_evidence = self.root / "proofd-service"
        self.producer_evidence = self.root / "producer"
        self.sinkhole_evidence = self.root / "sinkhole"

    def tearDown(self) -> None:
        self.tmp.cleanup()

    def test_proofd_outputs_drive_stage2_sinkhole_pipeline(self) -> None:
        proofd = subprocess.run(
            [
                "bash",
                str(self.proofd_script),
                "--evidence-dir",
                str(self.proofd_evidence),
            ],
            cwd=self.repo_root,
            check=False,
            capture_output=True,
            text=True,
        )
        self.assertEqual(proofd.returncode, 0, proofd.stderr)

        self._write_lenient_sinkhole_policy()

        producer = subprocess.run(
            [
                "bash",
                str(self.producer_script),
                "--evidence-dir",
                str(self.producer_evidence),
                "--artifact-root",
                str(self.proofd_evidence),
                "--replay-output",
                str(self.proofd_evidence / "replay_boundary_flow_report.json"),
                "--trust-output",
                str(self.proofd_evidence / "trust_reuse_flow_report.json"),
            ],
            cwd=self.repo_root,
            check=False,
            capture_output=True,
            text=True,
        )
        self.assertEqual(producer.returncode, 0, producer.stderr)

        sinkhole = subprocess.run(
            [
                "bash",
                str(self.sinkhole_script),
                "--evidence-dir",
                str(self.sinkhole_evidence),
                "--artifact-root",
                str(self.proofd_evidence),
                "--ledger",
                str(self.proofd_evidence / "verification_diversity_ledger.json"),
                "--policy",
                str(self.proofd_evidence / "authority_sinkhole_policy.json"),
                "--replay-flow",
                str(self.proofd_evidence / "replay_boundary_flow_report.json"),
                "--trust-reuse-flow",
                str(self.proofd_evidence / "trust_reuse_flow_report.json"),
            ],
            cwd=self.repo_root,
            check=False,
            capture_output=True,
            text=True,
        )
        self.assertEqual(sinkhole.returncode, 0, sinkhole.stderr)

        proofd_service = json.loads(
            (self.proofd_evidence / "proofd_service_report.json").read_text(
                encoding="utf-8"
            )
        )
        producer_report = json.loads(
            (
                self.producer_evidence
                / "authority_sinkhole_companion_flow_materialization_report.json"
            ).read_text(encoding="utf-8")
        )
        sinkhole_report = json.loads(
            (self.sinkhole_evidence / "authority_sinkhole_absorption_report.json").read_text(
                encoding="utf-8"
            )
        )
        cross_surface = json.loads(
            (
                self.sinkhole_evidence / "cross_surface_basin_alignment_report.json"
            ).read_text(encoding="utf-8")
        )

        self.assertEqual(
            proofd_service.get("trust_reuse_runtime_surface_origin"),
            "runtime_bundle_trust_reuse",
        )
        self.assertEqual(
            proofd_service.get("trust_reuse_flow_source_origin"),
            "runtime_bundle_trust_reuse",
        )
        self.assertEqual(producer_report.get("status"), "PASS")
        self.assertEqual(
            producer_report.get("metrics", {}).get("materialized_surface_count"), 2
        )
        self.assertEqual(sinkhole_report.get("status"), "PASS")
        self.assertEqual(
            sinkhole_report.get("replay_boundary_flow_path"),
            str(self.proofd_evidence / "replay_boundary_flow_report.json"),
        )
        self.assertEqual(
            sinkhole_report.get("trust_reuse_flow_path"),
            str(self.proofd_evidence / "trust_reuse_flow_report.json"),
        )
        self.assertEqual(cross_surface.get("status"), "PASS")
        self.assertIsNotNone(
            cross_surface.get("metrics", {}).get("cross_surface_basin_alignment_ratio")
        )
        self.assertTrue(
            (self.proofd_evidence / "trust_reuse_runtime_surface.json").is_file()
        )
        self.assertTrue(
            (self.proofd_evidence / "replay_boundary_flow_report.json").is_file()
        )
        self.assertTrue(
            (self.proofd_evidence / "trust_reuse_flow_report.json").is_file()
        )

    def _write_lenient_sinkhole_policy(self) -> None:
        payload = {
            "policy_version": 1,
            "window_runs": 64,
            "window_seconds": 86400,
            "min_repeated_subject_groups": 0,
            "max_authority_basin_share": 1.10,
            "max_authority_basin_reuse_ratio": 1.10,
            "max_authority_basin_repeat_capture_rate": 1.10,
            "min_alternate_path_decay_ratio": 0.0,
            "series_window_runs": 4,
            "series_window_count": 2,
            "max_basin_dominance_slope": 1.0,
            "max_replay_boundary_basin_capture_ratio": 1.10,
            "max_replay_boundary_repeat_capture_rate": 1.10,
            "max_trust_reuse_basin_capture_ratio": 1.10,
            "max_trust_reuse_repeat_capture_rate": 1.10,
            "max_cross_surface_basin_alignment_ratio": 1.10,
        }
        (self.proofd_evidence / "authority_sinkhole_policy.json").write_text(
            json.dumps(payload, indent=2, sort_keys=True) + "\n",
            encoding="utf-8",
        )


if __name__ == "__main__":
    unittest.main()
