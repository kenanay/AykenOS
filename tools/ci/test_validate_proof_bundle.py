#!/usr/bin/env python3
"""Black-box tests for validate_proof_bundle.py."""

from __future__ import annotations

# Author: Kenan AY

import hashlib
import json
import subprocess
import tempfile
import unittest
from pathlib import Path


class ProofBundleValidatorTest(unittest.TestCase):
    def setUp(self) -> None:
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)

        self.abdf_dir = self.root / "abdf"
        self.execution_dir = self.root / "execution"
        self.replay_dir = self.root / "replay"
        self.kpl_dir = self.root / "kpl"
        self.ledger_dir = self.root / "ledger"
        self.eti_dir = self.root / "eti"
        self.meta_dir = self.root / "meta"
        self.bundle_root = self.root / "proof_bundle"
        self.verify_dir = self.root / "verify"

        self.kernel_image_bin = self.root / "kernel.elf"
        self.summary_json = self.root / "summary.json"
        self.meta_run_json = self.meta_dir / "run.json"

        self.bundle_verify_json = self.verify_dir / "bundle_verify.json"
        self.report_json = self.verify_dir / "report.json"

        self.validator = Path(__file__).with_name("validate_proof_bundle.py")

    def tearDown(self) -> None:
        self.tmp.cleanup()

    def _sha256_hex(self, payload: bytes) -> str:
        return hashlib.sha256(payload).hexdigest()

    def _canonical_json(self, payload: dict) -> bytes:
        return json.dumps(payload, sort_keys=True, separators=(",", ":")).encode("utf-8")

    def _bundle_id(self, manifest: dict, checksums: dict) -> str:
        base = dict(manifest)
        base.pop("bundle_id", None)
        return self._sha256_hex(self._canonical_json(base) + self._canonical_json(checksums))

    def _proof_hash(self, manifest: dict) -> str:
        base = dict(manifest)
        base.pop("proof_hash", None)
        return self._sha256_hex(self._canonical_json(base))

    def _write_text(self, path: Path, value: str) -> None:
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(value, encoding="utf-8")

    def _write_json(self, path: Path, payload: dict) -> None:
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(json.dumps(payload, sort_keys=True) + "\n", encoding="utf-8")

    def _write_base_inputs(self) -> None:
        self.abdf_dir.mkdir(parents=True, exist_ok=True)
        self.execution_dir.mkdir(parents=True, exist_ok=True)
        self.replay_dir.mkdir(parents=True, exist_ok=True)
        self.kpl_dir.mkdir(parents=True, exist_ok=True)
        self.ledger_dir.mkdir(parents=True, exist_ok=True)
        self.eti_dir.mkdir(parents=True, exist_ok=True)
        self.meta_dir.mkdir(parents=True, exist_ok=True)

        execution_trace_bytes = (
            b'{"cpu_id":0,"event_seq":1,"event_type":"AY_EVT_SYSCALL_ENTER","ltick":1}\n'
            b'{"cpu_id":0,"event_seq":2,"event_type":"AY_EVT_SYSCALL_EXIT","ltick":2}\n'
        )
        replay_trace_bytes = execution_trace_bytes
        ledger_bytes = b'{"event_seq":1,"ltick":1}\n{"event_seq":2,"ltick":2}\n'
        eti_bytes = (
            b'{"cpu_id":0,"event_seq":1,"event_type":"AY_EVT_SYSCALL_ENTER","ltick":1}\n'
            b'{"cpu_id":0,"event_seq":2,"event_type":"AY_EVT_SYSCALL_EXIT","ltick":2}\n'
        )
        kernel_bytes = b"KERNEL"
        config_bytes = b'{"run_id":"bundle-test"}\n'

        execution_trace_hash = self._sha256_hex(execution_trace_bytes)
        replay_trace_hash = self._sha256_hex(replay_trace_bytes)
        ledger_root_hash = self._sha256_hex(ledger_bytes)
        transcript_root_hash = self._sha256_hex(eti_bytes)
        kernel_image_hash = self._sha256_hex(kernel_bytes)
        config_hash = self._sha256_hex(config_bytes)

        self._write_text(self.abdf_dir / "abdf_snapshot_hash.txt", ("a" * 64) + "\n")
        self._write_text(self.execution_dir / "bcib_plan_hash.txt", ("b" * 64) + "\n")
        self._write_text(
            self.execution_dir / "execution_trace_hash.txt", execution_trace_hash + "\n"
        )
        (self.execution_dir / "execution_trace.jsonl").write_bytes(execution_trace_bytes)
        self._write_text(self.replay_dir / "replay_trace_hash.txt", replay_trace_hash + "\n")
        (self.replay_dir / "replay_trace.jsonl").write_bytes(replay_trace_bytes)
        self._write_json(
            self.replay_dir / "replay_report.json",
            {
                "status": "PASS",
                "replay_execution_trace_hash": replay_trace_hash,
                "replay_result_hash": "d" * 64,
                "final_state_hash": "e" * 64,
                "replay_event_count": 2,
                "violations_count": 0,
            },
        )

        (self.ledger_dir / "decision_ledger.jsonl").write_bytes(ledger_bytes)
        (self.eti_dir / "eti_transcript.jsonl").write_bytes(eti_bytes)
        self.kernel_image_bin.write_bytes(kernel_bytes)
        self.meta_run_json.write_bytes(config_bytes)
        self._write_json(self.summary_json, {"gate": "summary", "verdict": "PASS"})

        proof_manifest = {
            "manifest_version": 1,
            "mode": "bootstrap_kpl_proof_manifest",
            "signature_mode": "bootstrap-none",
            "signer_sig": "",
            "hash_algorithm": "sha256",
            "kernel_image_hash": kernel_image_hash,
            "config_hash": config_hash,
            "ledger_root_hash": ledger_root_hash,
            "transcript_root_hash": transcript_root_hash,
            "abdf_snapshot_hash": "a" * 64,
            "bcib_plan_hash": "b" * 64,
            "execution_trace_hash": execution_trace_hash,
            "replay_result_hash": "d" * 64,
            "final_state_hash": "e" * 64,
            "event_count": 2,
            "violation_count": 0,
        }
        proof_manifest["proof_hash"] = self._proof_hash(proof_manifest)

        self._write_json(self.kpl_dir / "proof_manifest.json", proof_manifest)
        self._write_json(
            self.kpl_dir / "proof_verify.json",
            {"status": "PASS", "proof_hash": proof_manifest["proof_hash"]},
        )
        self._write_json(self.kpl_dir / "report.json", {"gate": "kpl-proof", "verdict": "PASS"})

    def _run_generate(self) -> int:
        cmd = [
            "python3",
            str(self.validator),
            "generate",
            "--bundle-root",
            str(self.bundle_root),
            "--abdf-evidence",
            str(self.abdf_dir),
            "--execution-evidence",
            str(self.execution_dir),
            "--replay-evidence",
            str(self.replay_dir),
            "--kpl-evidence",
            str(self.kpl_dir),
            "--ledger-evidence",
            str(self.ledger_dir),
            "--eti-evidence",
            str(self.eti_dir),
            "--kernel-image-bin",
            str(self.kernel_image_bin),
            "--summary-json",
            str(self.summary_json),
            "--meta-run-json",
            str(self.meta_run_json),
        ]
        proc = subprocess.run(cmd, check=False)
        return proc.returncode

    def _run_verify(self) -> tuple[int, dict, dict]:
        cmd = [
            "python3",
            str(self.validator),
            "verify",
            "--bundle-root",
            str(self.bundle_root),
            "--out-bundle-verify-json",
            str(self.bundle_verify_json),
            "--out-report",
            str(self.report_json),
        ]
        proc = subprocess.run(cmd, check=False)
        report = json.loads(self.report_json.read_text(encoding="utf-8"))
        verify = json.loads(self.bundle_verify_json.read_text(encoding="utf-8"))
        return proc.returncode, report, verify

    def test_pass_generate_and_verify_bundle(self) -> None:
        self._write_base_inputs()
        self.assertEqual(self._run_generate(), 0)

        rc, report, verify = self._run_verify()
        self.assertEqual(rc, 0)
        self.assertEqual(report.get("verdict"), "PASS")
        self.assertEqual(verify.get("status"), "PASS")
        self.assertTrue(verify.get("portability_parity"))
        self.assertTrue((self.bundle_root / "manifest.json").is_file())
        self.assertTrue((self.bundle_root / "checksums.json").is_file())

    def test_fail_on_missing_required_artifact(self) -> None:
        self._write_base_inputs()
        self.assertEqual(self._run_generate(), 0)
        (self.bundle_root / "reports/proof_manifest.json").unlink()

        rc, report, verify = self._run_verify()
        self.assertEqual(rc, 2)
        self.assertEqual(report.get("verdict"), "FAIL")
        self.assertEqual(verify.get("status"), "FAIL")
        self.assertIn(
            "missing_bundle_required_file:reports/proof_manifest.json",
            report.get("violations", []),
        )

    def test_fail_on_checksum_mismatch(self) -> None:
        self._write_base_inputs()
        self.assertEqual(self._run_generate(), 0)
        self._write_text(
            self.bundle_root / "traces/replay_trace.jsonl",
            '{"cpu_id":0,"event_seq":9,"event_type":"AY_EVT_TAMPER","ltick":9}\n',
        )

        rc, report, _ = self._run_verify()
        self.assertEqual(rc, 2)
        self.assertEqual(report.get("verdict"), "FAIL")
        self.assertTrue(
            any(
                v.startswith("bundle_checksum_mismatch:traces/replay_trace.jsonl:")
                for v in report.get("violations", [])
            )
        )

    def test_fail_on_source_proof_hash_binding_mismatch(self) -> None:
        self._write_base_inputs()
        self.assertEqual(self._run_generate(), 0)

        manifest_path = self.bundle_root / "manifest.json"
        checksums_path = self.bundle_root / "checksums.json"
        manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
        checksums = json.loads(checksums_path.read_text(encoding="utf-8"))
        manifest["source_proof_hash"] = "f" * 64
        manifest["bundle_id"] = self._bundle_id(manifest, checksums)
        manifest_path.write_text(json.dumps(manifest, sort_keys=True) + "\n", encoding="utf-8")

        rc, report, _ = self._run_verify()
        self.assertEqual(rc, 2)
        self.assertTrue(
            any(
                v.startswith("bundle_source_proof_hash_mismatch:")
                for v in report.get("violations", [])
            )
        )


if __name__ == "__main__":
    unittest.main()
