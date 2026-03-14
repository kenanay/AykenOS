#!/usr/bin/env python3
"""Black-box tests for validate_dlt_determinism.py."""

from __future__ import annotations

# Author: Kenan AY

import json
import subprocess
import tempfile
import unittest
from pathlib import Path


class DltDeterminismValidatorTest(unittest.TestCase):
    def setUp(self) -> None:
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)
        self.eti_jsonl = self.root / "eti_transcript.jsonl"
        self.ltick_trace_a = self.root / "ltick_trace_a.jsonl"
        self.ltick_trace_b = self.root / "ltick_trace_b.jsonl"
        self.determinism_report = self.root / "dlt_determinism_report.json"
        self.report = self.root / "report.json"
        self.validator = Path(__file__).with_name("validate_dlt_determinism.py")

    def tearDown(self) -> None:
        self.tmp.cleanup()

    def _write_eti_rows(self, rows: list[dict]) -> None:
        with self.eti_jsonl.open("w", encoding="utf-8") as fh:
            for row in rows:
                fh.write(json.dumps(row, sort_keys=True) + "\n")

    def _run(self) -> tuple[int, dict, dict]:
        proc = subprocess.run(
            [
                "python3",
                str(self.validator),
                "--eti-jsonl",
                str(self.eti_jsonl),
                "--out-ltick-trace-a",
                str(self.ltick_trace_a),
                "--out-ltick-trace-b",
                str(self.ltick_trace_b),
                "--out-determinism-report",
                str(self.determinism_report),
                "--out-report",
                str(self.report),
            ],
            check=False,
        )
        report = json.loads(self.report.read_text(encoding="utf-8"))
        determinism = json.loads(self.determinism_report.read_text(encoding="utf-8"))
        return proc.returncode, report, determinism

    def _eti_row(self, event_seq: int, ltick: int) -> dict:
        return {
            "event_seq": event_seq,
            "ltick": ltick,
            "cpu_id": 0,
            "event_type": "AY_EVT_SYSCALL_ENTER",
        }

    def test_pass_when_same_eti_produces_same_hash(self) -> None:
        self._write_eti_rows([self._eti_row(30, 30), self._eti_row(31, 31)])
        rc, report, determinism = self._run()
        self.assertEqual(rc, 0)
        self.assertEqual(report.get("verdict"), "PASS")
        self.assertEqual(determinism.get("status"), "PASS")
        self.assertTrue(determinism.get("trace_hash_equal"))

    def test_fail_when_materialization_fails(self) -> None:
        self._write_eti_rows([self._eti_row(30, 30), self._eti_row(30, 30)])
        rc, report, determinism = self._run()
        self.assertEqual(rc, 2)
        self.assertEqual(report.get("verdict"), "FAIL")
        self.assertEqual(determinism.get("status"), "FAIL")
        self.assertIn("dlt_materialization_failed:run=a:rc=2", report.get("violations", []))

    def test_fail_when_eti_missing(self) -> None:
        # Intentionally keep ETI input absent.
        rc, report, determinism = self._run()
        self.assertEqual(rc, 2)
        self.assertEqual(report.get("verdict"), "FAIL")
        self.assertEqual(determinism.get("status"), "FAIL")
        self.assertTrue(
            any(v.startswith("missing_eti_jsonl:") for v in report.get("violations", []))
        )


if __name__ == "__main__":
    unittest.main()
