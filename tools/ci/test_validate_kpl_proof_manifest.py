#!/usr/bin/env python3
"""Black-box tests for validate_kpl_proof_manifest.py."""

from __future__ import annotations

# Author: Kenan AY

import hashlib
import json
import subprocess
import tempfile
import unittest
from pathlib import Path


class KplProofManifestValidatorTest(unittest.TestCase):
    def setUp(self) -> None:
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)

        self.abdf_hash_file = self.root / "abdf_snapshot_hash.txt"
        self.bcib_hash_file = self.root / "bcib_plan_hash.txt"
        self.execution_trace_hash_file = self.root / "execution_trace_hash.txt"
        self.replay_report_json = self.root / "replay_report.json"
        self.ledger_jsonl = self.root / "decision_ledger.jsonl"
        self.eti_jsonl = self.root / "eti_transcript.jsonl"
        self.kernel_image_bin = self.root / "kernel.elf"
        self.config_json = self.root / "run.json"

        self.input_manifest_json = self.root / "input_proof_manifest.json"
        self.expected_proof_hash_file = self.root / "expected_proof_hash.txt"
        self.expected_final_state_hash_file = self.root / "expected_final_state_hash.txt"

        self.out_manifest = self.root / "proof_manifest.json"
        self.out_verify = self.root / "proof_verify.json"
        self.out_report = self.root / "report.json"

        self.validator = Path(__file__).with_name("validate_kpl_proof_manifest.py")

    def tearDown(self) -> None:
        self.tmp.cleanup()

    def _write_hash(self, path: Path, value: str) -> None:
        path.write_text(value + "\n", encoding="utf-8")

    def _manifest_hash(self, payload: dict) -> str:
        base = dict(payload)
        base.pop("proof_hash", None)
        blob = json.dumps(base, sort_keys=True, separators=(",", ":")).encode("utf-8")
        return hashlib.sha256(blob).hexdigest()

    def _write_base_inputs(self) -> None:
        self._write_hash(self.abdf_hash_file, "a" * 64)
        self._write_hash(self.bcib_hash_file, "b" * 64)
        self._write_hash(self.execution_trace_hash_file, "c" * 64)
        self.replay_report_json.write_text(
            json.dumps(
                {
                    "status": "PASS",
                    "replay_result_hash": "d" * 64,
                    "final_state_hash": "e" * 64,
                    "replay_event_count": 2,
                    "violations_count": 0,
                },
                sort_keys=True,
            )
            + "\n",
            encoding="utf-8",
        )
        self.ledger_jsonl.write_text(
            '{"event_seq":1,"ltick":1}\n{"event_seq":2,"ltick":2}\n', encoding="utf-8"
        )
        self.eti_jsonl.write_text(
            '{"event_seq":1,"ltick":1,"event_type":"AY_EVT_SYSCALL_ENTER"}\n',
            encoding="utf-8",
        )
        self.kernel_image_bin.write_bytes(b"KERNEL")
        self.config_json.write_text('{"run_id":"local-test"}\n', encoding="utf-8")

    def _run(
        self,
        input_manifest: Path | None = None,
        expected_proof_hash: Path | None = None,
        expected_final_state_hash: Path | None = None,
    ) -> tuple[int, dict, dict, dict]:
        cmd = [
            "python3",
            str(self.validator),
            "--abdf-hash-file",
            str(self.abdf_hash_file),
            "--bcib-plan-hash-file",
            str(self.bcib_hash_file),
            "--execution-trace-hash-file",
            str(self.execution_trace_hash_file),
            "--replay-report-json",
            str(self.replay_report_json),
            "--ledger-jsonl",
            str(self.ledger_jsonl),
            "--eti-jsonl",
            str(self.eti_jsonl),
            "--kernel-image-bin",
            str(self.kernel_image_bin),
            "--config-json",
            str(self.config_json),
            "--out-proof-manifest-json",
            str(self.out_manifest),
            "--out-proof-verify-json",
            str(self.out_verify),
            "--out-report",
            str(self.out_report),
        ]
        if input_manifest is not None:
            cmd.extend(["--in-proof-manifest-json", str(input_manifest)])
        if expected_proof_hash is not None:
            cmd.extend(["--expected-proof-hash-file", str(expected_proof_hash)])
        if expected_final_state_hash is not None:
            cmd.extend(
                ["--expected-final-state-hash-file", str(expected_final_state_hash)]
            )

        proc = subprocess.run(cmd, check=False)
        report = json.loads(self.out_report.read_text(encoding="utf-8"))
        verify = json.loads(self.out_verify.read_text(encoding="utf-8"))
        manifest = json.loads(self.out_manifest.read_text(encoding="utf-8"))
        return proc.returncode, report, verify, manifest

    def test_pass_generated_manifest(self) -> None:
        self._write_base_inputs()
        rc, report, verify, manifest = self._run()
        self.assertEqual(rc, 0)
        self.assertEqual(report.get("verdict"), "PASS")
        self.assertEqual(verify.get("status"), "PASS")
        self.assertTrue(verify.get("proof_hash_match"))
        self.assertEqual(manifest.get("proof_hash"), self._manifest_hash(manifest))

    def test_pass_with_expected_hash_files(self) -> None:
        self._write_base_inputs()
        rc0, _, _, manifest = self._run()
        self.assertEqual(rc0, 0)
        self._write_hash(self.expected_proof_hash_file, str(manifest.get("proof_hash", "")))
        self._write_hash(
            self.expected_final_state_hash_file,
            str(manifest.get("final_state_hash", "")),
        )

        rc, report, verify, _ = self._run(
            expected_proof_hash=self.expected_proof_hash_file,
            expected_final_state_hash=self.expected_final_state_hash_file,
        )
        self.assertEqual(rc, 0)
        self.assertEqual(report.get("verdict"), "PASS")
        self.assertTrue(verify.get("expected_proof_hash_match"))
        self.assertTrue(verify.get("expected_final_state_hash_match"))

    def test_fail_on_missing_referenced_artifact(self) -> None:
        self._write_base_inputs()
        self.kernel_image_bin.unlink()
        rc, report, _, _ = self._run()
        self.assertEqual(rc, 2)
        self.assertTrue(
            any(v.startswith("missing_kernel_image_bin:") for v in report.get("violations", []))
        )

    def test_fail_on_invalid_input_hash_format(self) -> None:
        self._write_base_inputs()
        self.bcib_hash_file.write_text("not-a-hash\n", encoding="utf-8")
        rc, report, _, _ = self._run()
        self.assertEqual(rc, 2)
        self.assertTrue(
            any(
                v.startswith("invalid_bcib_plan_hash_format:")
                for v in report.get("violations", [])
            )
        )

    def test_fail_on_unsupported_manifest_version(self) -> None:
        self._write_base_inputs()
        rc0, _, _, manifest = self._run()
        self.assertEqual(rc0, 0)
        manifest["manifest_version"] = 99
        manifest["proof_hash"] = self._manifest_hash(manifest)
        self.input_manifest_json.write_text(
            json.dumps(manifest, sort_keys=True) + "\n", encoding="utf-8"
        )

        rc, report, _, _ = self._run(input_manifest=self.input_manifest_json)
        self.assertEqual(rc, 2)
        self.assertTrue(
            any(
                v.startswith("unsupported_manifest_version:")
                for v in report.get("violations", [])
            )
        )

    def test_fail_on_missing_required_field_in_manifest(self) -> None:
        self._write_base_inputs()
        rc0, _, _, manifest = self._run()
        self.assertEqual(rc0, 0)
        manifest.pop("final_state_hash", None)
        manifest["proof_hash"] = self._manifest_hash(manifest)
        self.input_manifest_json.write_text(
            json.dumps(manifest, sort_keys=True) + "\n", encoding="utf-8"
        )

        rc, report, _, _ = self._run(input_manifest=self.input_manifest_json)
        self.assertEqual(rc, 2)
        self.assertIn("missing_proof_manifest_field:final_state_hash", report.get("violations", []))

    def test_fail_on_manifest_self_hash_mismatch(self) -> None:
        self._write_base_inputs()
        rc0, _, _, manifest = self._run()
        self.assertEqual(rc0, 0)
        manifest["proof_hash"] = "f" * 64
        self.input_manifest_json.write_text(
            json.dumps(manifest, sort_keys=True) + "\n", encoding="utf-8"
        )

        rc, report, _, _ = self._run(input_manifest=self.input_manifest_json)
        self.assertEqual(rc, 2)
        self.assertTrue(
            any(v.startswith("proof_hash_mismatch:") for v in report.get("violations", []))
        )

    def test_fail_on_replay_result_hash_binding_mismatch(self) -> None:
        self._write_base_inputs()
        rc0, _, _, manifest = self._run()
        self.assertEqual(rc0, 0)
        manifest["replay_result_hash"] = "1" * 64
        manifest["proof_hash"] = self._manifest_hash(manifest)
        self.input_manifest_json.write_text(
            json.dumps(manifest, sort_keys=True) + "\n", encoding="utf-8"
        )

        rc, report, _, _ = self._run(input_manifest=self.input_manifest_json)
        self.assertEqual(rc, 2)
        self.assertIn("replay_result_hash_binding_mismatch", report.get("violations", []))

    def test_fail_on_expected_final_state_hash_mismatch(self) -> None:
        self._write_base_inputs()
        self._write_hash(self.expected_final_state_hash_file, "9" * 64)
        rc, report, _, _ = self._run(expected_final_state_hash=self.expected_final_state_hash_file)
        self.assertEqual(rc, 2)
        self.assertTrue(
            any(
                v.startswith("expected_final_state_hash_mismatch:")
                for v in report.get("violations", [])
            )
        )


if __name__ == "__main__":
    unittest.main()
