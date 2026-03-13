#!/usr/bin/env python3
"""Black-box tests for produce_verification_diversity_ledger.sh."""

from __future__ import annotations

import json
import subprocess
import tempfile
import unittest
from pathlib import Path


class VerificationDiversityLedgerProducerTest(unittest.TestCase):
    def setUp(self) -> None:
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)
        self.repo_root = Path(__file__).resolve().parents[2]
        self.script = (
            self.repo_root / "scripts" / "ci" / "produce_verification_diversity_ledger.sh"
        )
        self.artifact_root = self.root / "artifacts"
        self.evidence_dir = self.root / "producer"
        self.artifact_root.mkdir(parents=True, exist_ok=True)
        self._write_binding()
        self._write_audit_ledger()

    def tearDown(self) -> None:
        self.tmp.cleanup()

    def test_producer_appends_canonical_vdl_entries(self) -> None:
        proc = self._run_producer()
        self.assertEqual(proc.returncode, 0, proc.stderr)

        artifact_ledger = json.loads(
            (self.artifact_root / "verification_diversity_ledger.json").read_text(
                encoding="utf-8"
            )
        )
        report = json.loads((self.evidence_dir / "report.json").read_text(encoding="utf-8"))
        detail = json.loads(
            (self.evidence_dir / "verification_diversity_ledger_append_report.json").read_text(
                encoding="utf-8"
            )
        )
        self.assertEqual(report.get("verdict"), "PASS")
        self.assertEqual(detail.get("status"), "PASS")
        self.assertEqual(detail["metrics"].get("appended_entry_count"), 2)
        self.assertEqual(len(artifact_ledger.get("entries", [])), 2)
        self.assertEqual(
            artifact_ledger["entries"][0].get("verification_context_id"), "policy-hash-a"
        )
        self.assertEqual(artifact_ledger["entries"][0].get("verdict"), "PASS")

    def test_producer_skips_duplicate_entries_on_repeat_run(self) -> None:
        first = self._run_producer()
        self.assertEqual(first.returncode, 0, first.stderr)

        second = self._run_producer()
        self.assertEqual(second.returncode, 0, second.stderr)

        detail = json.loads(
            (self.evidence_dir / "verification_diversity_ledger_append_report.json").read_text(
                encoding="utf-8"
            )
        )
        self.assertEqual(detail["metrics"].get("appended_entry_count"), 0)
        self.assertEqual(detail["metrics"].get("duplicate_skipped_count"), 2)
        artifact_ledger = json.loads(
            (self.artifact_root / "verification_diversity_ledger.json").read_text(
                encoding="utf-8"
            )
        )
        self.assertEqual(len(artifact_ledger.get("entries", [])), 2)

    def test_producer_fails_when_binding_is_missing(self) -> None:
        (self.artifact_root / "verification_diversity_ledger_binding.json").unlink()

        proc = self._run_producer()
        self.assertEqual(proc.returncode, 2)

        detail = json.loads(
            (self.evidence_dir / "verification_diversity_ledger_append_report.json").read_text(
                encoding="utf-8"
            )
        )
        self.assertEqual(detail.get("status"), "FAIL")
        self.assertEqual(detail.get("load_failure_stage"), "binding_manifest_load")

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

    def _write_binding(self) -> None:
        payload = {
            "binding_version": 1,
            "run_id": "run-20260314-vdl",
            "verification_context_id_source": "policy_hash",
            "node_bindings": [
                {
                    "verification_node_id": "node-a",
                    "verifier_key_id": "key-a",
                    "verifier_id": "verifier-a",
                    "authority_chain_id": "chain-a",
                    "lineage_id": "lineage-a",
                    "execution_cluster_id": "cluster-a",
                }
            ],
        }
        (self.artifact_root / "verification_diversity_ledger_binding.json").write_text(
            json.dumps(payload, indent=2, sort_keys=True) + "\n",
            encoding="utf-8",
        )

    def _write_audit_ledger(self) -> None:
        events = [
            {
                "event_version": 1,
                "event_type": "verification",
                "event_id": "sha256:event-1",
                "event_time_utc": "2026-03-14T12:00:00Z",
                "verifier_node_id": "node-a",
                "verifier_key_id": "key-a",
                "bundle_id": "bundle-a",
                "trust_overlay_hash": "overlay-a",
                "policy_hash": "policy-hash-a",
                "registry_snapshot_hash": "registry-a",
                "verdict": "Trusted",
                "receipt_hash": "a" * 64,
                "previous_event_hash": None,
            },
            {
                "event_version": 1,
                "event_type": "verification",
                "event_id": "sha256:event-2",
                "event_time_utc": "2026-03-14T12:05:00Z",
                "verifier_node_id": "node-a",
                "verifier_key_id": "key-a",
                "bundle_id": "bundle-b",
                "trust_overlay_hash": "overlay-b",
                "policy_hash": "policy-hash-a",
                "registry_snapshot_hash": "registry-a",
                "verdict": "RejectedByPolicy",
                "receipt_hash": "b" * 64,
                "previous_event_hash": "sha256:event-1",
            },
        ]
        raw = "\n".join(json.dumps(event, sort_keys=True) for event in events) + "\n"
        (self.artifact_root / "verification_audit_ledger.jsonl").write_text(
            raw,
            encoding="utf-8",
        )


if __name__ == "__main__":
    unittest.main()
