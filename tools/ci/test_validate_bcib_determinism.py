#!/usr/bin/env python3
"""Black-box tests for validate_bcib_determinism.py."""

from __future__ import annotations

# Author: Kenan AY

import hashlib
import json
import subprocess
import tempfile
import unittest
from pathlib import Path


class BcibDeterminismValidatorTest(unittest.TestCase):
    def setUp(self) -> None:
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)
        self.run_a_dir = self.root / "run-1"
        self.run_b_dir = self.root / "run-2"
        self.run_a_dir.mkdir()
        self.run_b_dir.mkdir()
        self.out_run_a = self.root / "bcib_determinism_run_1.json"
        self.out_run_b = self.root / "bcib_determinism_run_2.json"
        self.out_trace_a = self.root / "bcib_determinism_trace_run_1.log"
        self.out_trace_b = self.root / "bcib_determinism_trace_run_2.log"
        self.out_result_bin = self.root / "result.bin"
        self.out_result_sha256 = self.root / "result.sha256"
        self.out_result_metadata = self.root / "result_metadata.json"
        self.out_comparison_log = self.root / "result_sha256_comparison.log"
        self.out_evidence = self.root / "bcib_kernel_determinism_evidence.json"
        self.out_report = self.root / "report.json"
        self.validator = Path(__file__).with_name("validate_bcib_determinism.py")

    def tearDown(self) -> None:
        self.tmp.cleanup()

    def _write_run(
        self,
        run_dir: Path,
        run_index: int,
        trace_lines: list[str],
        result_payload: bytes,
        hash_payload: bytes,
        *,
        pf: int = 0,
        declared_result_size: int | None = None,
    ) -> None:
        result_path = run_dir / "result.bin"
        hash_path = run_dir / "result_hash.bin"
        trace_path = run_dir / "debugcon.trace"
        result_path.write_bytes(result_payload)
        hash_path.write_bytes(hash_payload)
        trace_path.write_text("".join(trace_lines), encoding="utf-8")

        result_sha256 = hashlib.sha256(result_payload).hexdigest()
        result_fingerprint = hashlib.sha256(b"fingerprint:" + result_payload).hexdigest()
        summary = {
            "gate": "ci-gate-bcib-kernel-determinism",
            "run_index": run_index,
            "run_count": 2,
            "trace_file": "debugcon.trace",
            "fixture_sha256": "a" * 64,
            "fixture_metadata": {
                "bcib_sha256": "a" * 64,
                "canonical_plan_fingerprint": "b" * 64,
                "canonical_binding_fingerprint": "c" * 64,
            },
            "result": "PASS",
            "failure_code": None,
            "marker_counts": {
                "submit_bind": 1,
                "queue_create": 1,
                "dequeue_hit": 1,
                "pickup": 1,
                "result_va": 1,
                "wait_ok": 1,
                "result_ok": 1,
                "result_fail": 0,
                "pf": pf,
            },
            "markers": {
                "submit_bind": {"line": 2, "text": "[SUBMIT_BIND]"},
                "queue_create": {"line": 3, "text": "[QUEUE_CREATE]"},
                "dequeue_hit": {"line": 4, "text": "[DEQUEUE_HIT]"},
                "pickup": {"line": 5, "text": "[PICKUP]"},
                "result_va": {"line": 6, "text": "[RESULT_VA]"},
                "wait_ok": {"line": 7, "text": "[WAIT_OK]"},
                "result_ok": {"line": 8, "text": "[RESULT_OK]"},
            },
            "result_artifact": "result.bin",
            "hash_artifact": "result_hash.bin",
            "result_header": {
                "magic": 1414876993,
                "abi_version": 1,
                "flags": 0,
                "bytes_written": len(result_payload)
                if declared_result_size is None
                else declared_result_size,
            },
            "hash_header": {
                "magic": 1213416769,
                "abi_version": 1,
                "algorithm": 1,
                "flags": 0,
                "hashed_size": len(result_payload)
                if declared_result_size is None
                else declared_result_size,
                "digest_hex": result_fingerprint,
            },
            "kernel_result_sha256": result_sha256,
            "kernel_result_fingerprint": result_fingerprint,
            "expected_sidecar_digest": result_fingerprint,
            "hash_sidecar_valid": True,
            "violations": [],
            "warnings": [],
        }
        (run_dir / "run_summary.json").write_text(
            json.dumps(summary, indent=2, sort_keys=True) + "\n",
            encoding="utf-8",
        )

    def _run(self) -> tuple[int, dict, dict]:
        proc = subprocess.run(
            [
                "python3",
                str(self.validator),
                "--run-a-dir",
                str(self.run_a_dir),
                "--run-b-dir",
                str(self.run_b_dir),
                "--out-run-a-json",
                str(self.out_run_a),
                "--out-run-b-json",
                str(self.out_run_b),
                "--out-trace-run-a",
                str(self.out_trace_a),
                "--out-trace-run-b",
                str(self.out_trace_b),
                "--out-result-bin",
                str(self.out_result_bin),
                "--out-result-sha256",
                str(self.out_result_sha256),
                "--out-result-metadata",
                str(self.out_result_metadata),
                "--out-comparison-log",
                str(self.out_comparison_log),
                "--out-determinism-evidence",
                str(self.out_evidence),
                "--out-report",
                str(self.out_report),
            ],
            check=False,
        )
        report = json.loads(self.out_report.read_text(encoding="utf-8"))
        evidence = json.loads(self.out_evidence.read_text(encoding="utf-8"))
        return proc.returncode, report, evidence

    def test_pass_when_two_runs_match(self) -> None:
        trace = [
            "boot\n",
            "[SUBMIT_BIND]\n",
            "[QUEUE_CREATE]\n",
            "[DEQUEUE_HIT]\n",
            "[PICKUP]\n",
            "[RESULT_VA]\n",
            "[WAIT_OK]\n",
            "[RESULT_OK]\n",
            "done\n",
        ]
        payload = b"stable-result"
        hash_payload = b"stable-hash"
        self._write_run(self.run_a_dir, 1, trace, payload, hash_payload)
        self._write_run(self.run_b_dir, 2, trace, payload, hash_payload)

        rc, report, evidence = self._run()
        self.assertEqual(rc, 0)
        self.assertEqual(report.get("verdict"), "PASS")
        self.assertEqual(report.get("closure_verdict"), "DETERMINISM_PASS")
        self.assertEqual(evidence.get("status"), "PASS")
        self.assertEqual(
            self.out_result_sha256.read_text(encoding="utf-8").strip(),
            hashlib.sha256(payload).hexdigest(),
        )

    def test_fail_when_result_artifact_differs(self) -> None:
        trace = [
            "boot\n",
            "[SUBMIT_BIND]\n",
            "[QUEUE_CREATE]\n",
            "[DEQUEUE_HIT]\n",
            "[PICKUP]\n",
            "[RESULT_VA]\n",
            "[WAIT_OK]\n",
            "[RESULT_OK]\n",
        ]
        self._write_run(self.run_a_dir, 1, trace, b"result-a", b"hash-a")
        self._write_run(self.run_b_dir, 2, trace, b"result-b", b"hash-b")

        rc, report, _ = self._run()
        self.assertEqual(rc, 2)
        self.assertIn("result_sha256_mismatch", report.get("violations", []))

    def test_fail_when_trace_window_differs(self) -> None:
        trace_a = [
            "boot\n",
            "[SUBMIT_BIND]\n",
            "[QUEUE_CREATE]\n",
            "[DEQUEUE_HIT]\n",
            "[PICKUP]\n",
            "[RESULT_VA]\n",
            "[WAIT_OK]\n",
            "[RESULT_OK]\n",
        ]
        trace_b = [
            "boot\n",
            "[SUBMIT_BIND]\n",
            "[QUEUE_CREATE]\n",
            "[DEQUEUE_HIT]\n",
            "[PICKUP]\n",
            "[RESULT_VA]\n",
            "[WAIT_OK]\n",
            "[RESULT_OK] extra-jitter\n",
        ]
        payload = b"stable-result"
        hash_payload = b"stable-hash"
        self._write_run(self.run_a_dir, 1, trace_a, payload, hash_payload)
        self._write_run(self.run_b_dir, 2, trace_b, payload, hash_payload)

        rc, report, evidence = self._run()
        self.assertEqual(rc, 2)
        self.assertIn("trace_window_sha256_mismatch", report.get("violations", []))
        self.assertFalse(evidence["trace_window_sha256"]["match"])

    def test_fail_when_fallback_is_visible_inside_execution_window(self) -> None:
        trace_a = [
            "boot\n",
            "[SUBMIT_BIND]\n",
            "[QUEUE_CREATE]\n",
            "[[AYKEN_PERF_MB_PATH]] name=fallback phase=enter\n",
            "[DEQUEUE_HIT]\n",
            "[PICKUP]\n",
            "[RESULT_VA]\n",
            "[WAIT_OK]\n",
            "[RESULT_OK]\n",
        ]
        trace_b = [
            "boot\n",
            "[SUBMIT_BIND]\n",
            "[QUEUE_CREATE]\n",
            "[DEQUEUE_HIT]\n",
            "[PICKUP]\n",
            "[RESULT_VA]\n",
            "[WAIT_OK]\n",
            "[RESULT_OK]\n",
            "done\n",
        ]
        payload = b"stable-result"
        hash_payload = b"stable-hash"
        self._write_run(self.run_a_dir, 1, trace_a, payload, hash_payload)
        self._write_run(self.run_b_dir, 2, trace_b, payload, hash_payload)

        summary_a = json.loads((self.run_a_dir / "run_summary.json").read_text(encoding="utf-8"))
        summary_a["markers"]["dequeue_hit"]["line"] = 5
        summary_a["markers"]["pickup"]["line"] = 6
        summary_a["markers"]["result_va"]["line"] = 7
        summary_a["markers"]["wait_ok"]["line"] = 8
        summary_a["markers"]["result_ok"]["line"] = 9
        (self.run_a_dir / "run_summary.json").write_text(
            json.dumps(summary_a, indent=2, sort_keys=True) + "\n",
            encoding="utf-8",
        )

        rc, report, _ = self._run()
        self.assertEqual(rc, 2)
        self.assertTrue(
            any(v.startswith("fallback_path_observed:run_a:") for v in report.get("violations", []))
        )

    def test_fail_when_result_payload_is_header_only(self) -> None:
        trace = [
            "boot\n",
            "[SUBMIT_BIND]\n",
            "[QUEUE_CREATE]\n",
            "[DEQUEUE_HIT]\n",
            "[PICKUP]\n",
            "[RESULT_VA]\n",
            "[WAIT_OK]\n",
            "[RESULT_OK]\n",
        ]
        header_only_payload = bytes(48)
        hash_payload = bytes(72)
        self._write_run(
            self.run_a_dir,
            1,
            trace,
            header_only_payload,
            hash_payload,
            declared_result_size=0,
        )
        self._write_run(
            self.run_b_dir,
            2,
            trace,
            header_only_payload,
            hash_payload,
            declared_result_size=0,
        )

        rc, report, evidence = self._run()
        self.assertEqual(rc, 2)
        self.assertIn("invalid_result_size:run_a:0", report.get("violations", []))
        self.assertIn("invalid_result_size:run_b:0", report.get("violations", []))
        self.assertIn("header_only_result:run_a:artifact_size=48", report.get("violations", []))
        self.assertIn("header_only_result:run_b:artifact_size=48", report.get("violations", []))
        self.assertEqual(evidence.get("status"), "FAIL")


if __name__ == "__main__":
    unittest.main()
