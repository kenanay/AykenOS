#!/usr/bin/env python3
"""Black-box tests for validate_bcib_trace_identity.py."""

from __future__ import annotations

# Author: Kenan AY

import hashlib
import json
import subprocess
import tempfile
import unittest
from pathlib import Path


class BcibTraceIdentityValidatorTest(unittest.TestCase):
    def setUp(self) -> None:
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)
        self.bcib_plan = self.root / "plan.bcib"
        self.eti_jsonl = self.root / "eti_transcript.jsonl"
        self.expected_plan_hash = self.root / "expected_plan_hash.txt"
        self.expected_trace_hash = self.root / "expected_trace_hash.txt"
        self.plan_hash_txt = self.root / "bcib_plan_hash.txt"
        self.trace_jsonl = self.root / "execution_trace.jsonl"
        self.trace_hash_txt = self.root / "execution_trace_hash.txt"
        self.trace_verify_json = self.root / "trace_verify.json"
        self.report = self.root / "report.json"
        self.validator = Path(__file__).with_name("validate_bcib_trace_identity.py")

    def tearDown(self) -> None:
        self.tmp.cleanup()

    def _write_plan(self, payload: bytes) -> None:
        self.bcib_plan.write_bytes(payload)

    def _write_eti_rows(self, rows: list[dict]) -> None:
        with self.eti_jsonl.open("w", encoding="utf-8") as fh:
            for row in rows:
                fh.write(json.dumps(row, sort_keys=True) + "\n")

    def _eti_row(self, event_seq: int, ltick: int) -> dict:
        return {
            "event_seq": event_seq,
            "ltick": ltick,
            "cpu_id": 0,
            "event_type": "AY_EVT_SYSCALL_ENTER",
        }

    def _run(
        self,
        expected_plan_hash: Path | None = None,
        expected_trace_hash: Path | None = None,
    ) -> tuple[int, dict, dict, str, str]:
        cmd = [
            "python3",
            str(self.validator),
            "--bcib-plan-bin",
            str(self.bcib_plan),
            "--eti-jsonl",
            str(self.eti_jsonl),
            "--out-plan-hash-txt",
            str(self.plan_hash_txt),
            "--out-execution-trace-jsonl",
            str(self.trace_jsonl),
            "--out-execution-trace-hash-txt",
            str(self.trace_hash_txt),
            "--out-trace-verify-json",
            str(self.trace_verify_json),
            "--out-report",
            str(self.report),
        ]
        if expected_plan_hash is not None:
            cmd.extend(["--expected-plan-hash-file", str(expected_plan_hash)])
        if expected_trace_hash is not None:
            cmd.extend(["--expected-trace-hash-file", str(expected_trace_hash)])

        proc = subprocess.run(cmd, check=False)
        report = json.loads(self.report.read_text(encoding="utf-8"))
        trace_verify = json.loads(self.trace_verify_json.read_text(encoding="utf-8"))
        plan_hash = self.plan_hash_txt.read_text(encoding="utf-8").strip()
        trace_hash = self.trace_hash_txt.read_text(encoding="utf-8").strip()
        return proc.returncode, report, trace_verify, plan_hash, trace_hash

    def test_pass_with_valid_plan_and_trace(self) -> None:
        self._write_plan(b"BCIB\x01\x02\x03")
        self._write_eti_rows([self._eti_row(1, 1), self._eti_row(2, 2)])
        rc, report, trace_verify, plan_hash, trace_hash = self._run()
        self.assertEqual(rc, 0)
        self.assertEqual(report.get("verdict"), "PASS")
        self.assertEqual(trace_verify.get("status"), "PASS")
        self.assertEqual(plan_hash, hashlib.sha256(b"BCIB\x01\x02\x03").hexdigest())
        self.assertEqual(str(report.get("bcib_plan_hash")), plan_hash)
        self.assertTrue(bool(trace_hash))

    def test_pass_with_expected_hash_files(self) -> None:
        self._write_plan(b"BCIB\x10\x20")
        self._write_eti_rows([self._eti_row(1, 1), self._eti_row(2, 2), self._eti_row(3, 3)])
        rc0, _, _, plan_hash, trace_hash = self._run()
        self.assertEqual(rc0, 0)
        self.expected_plan_hash.write_text(plan_hash + "\n", encoding="utf-8")
        self.expected_trace_hash.write_text(trace_hash + "\n", encoding="utf-8")

        rc, report, trace_verify, _, _ = self._run(
            expected_plan_hash=self.expected_plan_hash,
            expected_trace_hash=self.expected_trace_hash,
        )
        self.assertEqual(rc, 0)
        self.assertEqual(report.get("verdict"), "PASS")
        self.assertTrue(report.get("expected_plan_hash_match"))
        self.assertTrue(report.get("expected_trace_hash_match"))
        self.assertEqual(trace_verify.get("status"), "PASS")

    def test_fail_on_expected_plan_hash_mismatch(self) -> None:
        self._write_plan(b"BCIB\x01")
        self._write_eti_rows([self._eti_row(1, 1)])
        self.expected_plan_hash.write_text(("f" * 64) + "\n", encoding="utf-8")
        rc, report, _, _, _ = self._run(expected_plan_hash=self.expected_plan_hash)
        self.assertEqual(rc, 2)
        self.assertTrue(
            any(v.startswith("bcib_plan_hash_mismatch:") for v in report.get("violations", []))
        )

    def test_fail_on_expected_trace_hash_mismatch(self) -> None:
        self._write_plan(b"BCIB\x01")
        self._write_eti_rows([self._eti_row(1, 1)])
        self.expected_trace_hash.write_text(("e" * 64) + "\n", encoding="utf-8")
        rc, report, _, _, _ = self._run(expected_trace_hash=self.expected_trace_hash)
        self.assertEqual(rc, 2)
        self.assertTrue(
            any(
                v.startswith("execution_trace_hash_mismatch:")
                for v in report.get("violations", [])
            )
        )

    def test_fail_on_invalid_expected_hash_format(self) -> None:
        self._write_plan(b"BCIB\x01")
        self._write_eti_rows([self._eti_row(1, 1)])
        self.expected_plan_hash.write_text("not-a-hash\n", encoding="utf-8")
        rc, report, _, _, _ = self._run(expected_plan_hash=self.expected_plan_hash)
        self.assertEqual(rc, 2)
        self.assertTrue(
            any(
                v.startswith("invalid_expected_plan_hash_format:")
                for v in report.get("violations", [])
            )
        )

    def test_fail_on_empty_plan(self) -> None:
        self._write_plan(b"")
        self._write_eti_rows([self._eti_row(1, 1)])
        rc, report, _, _, _ = self._run()
        self.assertEqual(rc, 2)
        self.assertIn("empty_bcib_plan_bin", report.get("violations", []))

    def test_fail_on_missing_plan(self) -> None:
        self._write_eti_rows([self._eti_row(1, 1)])
        rc, report, _, _, _ = self._run()
        self.assertEqual(rc, 2)
        self.assertTrue(
            any(v.startswith("missing_bcib_plan_bin:") for v in report.get("violations", []))
        )

    def test_fail_on_empty_eti(self) -> None:
        self._write_plan(b"BCIB\x01")
        self._write_eti_rows([])
        rc, report, _, _, _ = self._run()
        self.assertEqual(rc, 2)
        self.assertIn("empty_eti_jsonl", report.get("violations", []))


if __name__ == "__main__":
    unittest.main()
