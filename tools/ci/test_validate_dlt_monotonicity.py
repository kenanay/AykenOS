#!/usr/bin/env python3
"""Black-box tests for validate_dlt_monotonicity.py."""

from __future__ import annotations

# Author: Kenan AY

import json
import subprocess
import tempfile
import unittest
from pathlib import Path


class DltMonotonicityValidatorTest(unittest.TestCase):
    def setUp(self) -> None:
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)
        self.eti_jsonl = self.root / "eti_transcript.jsonl"
        self.ltick_trace = self.root / "ltick_trace.jsonl"
        self.report = self.root / "report.json"
        self.validator = Path(__file__).with_name("validate_dlt_monotonicity.py")

    def tearDown(self) -> None:
        self.tmp.cleanup()

    def _write_eti_rows(self, rows: list[dict]) -> None:
        with self.eti_jsonl.open("w", encoding="utf-8") as fh:
            for row in rows:
                fh.write(json.dumps(row, sort_keys=True) + "\n")

    def _run(self) -> tuple[int, dict, list[dict]]:
        proc = subprocess.run(
            [
                "python3",
                str(self.validator),
                "--eti-jsonl",
                str(self.eti_jsonl),
                "--out-ltick-trace",
                str(self.ltick_trace),
                "--out-report",
                str(self.report),
            ],
            check=False,
        )
        report = json.loads(self.report.read_text(encoding="utf-8"))
        rows = [
            json.loads(line)
            for line in self.ltick_trace.read_text(encoding="utf-8").splitlines()
            if line.strip()
        ]
        return proc.returncode, report, rows

    def _eti_row(self, event_seq: int, ltick: int) -> dict:
        return {
            "event_seq": event_seq,
            "ltick": ltick,
            "cpu_id": 0,
            "event_type": "AY_EVT_SYSCALL_ENTER",
        }

    def test_pass_with_monotonic_unique_source_order(self) -> None:
        self._write_eti_rows(
            [
                self._eti_row(2, 2),
                self._eti_row(5, 5),
                self._eti_row(9, 9),
            ]
        )
        rc, report, rows = self._run()
        self.assertEqual(rc, 0)
        self.assertEqual(report.get("verdict"), "PASS")
        self.assertEqual([row["ltick"] for row in rows], [1, 2, 3])
        self.assertEqual([row["source_event_seq"] for row in rows], [2, 5, 9])

    def test_fail_on_duplicate_source_ltick(self) -> None:
        self._write_eti_rows(
            [
                self._eti_row(2, 2),
                self._eti_row(3, 2),
            ]
        )
        rc, report, _ = self._run()
        self.assertEqual(rc, 2)
        self.assertIn("source_ltick_duplicate", report.get("violations", []))

    def test_fail_on_non_monotonic_source_event_seq(self) -> None:
        self._write_eti_rows(
            [
                self._eti_row(4, 4),
                self._eti_row(3, 3),
            ]
        )
        rc, report, _ = self._run()
        self.assertEqual(rc, 2)
        self.assertIn("source_event_seq_non_monotonic", report.get("violations", []))


if __name__ == "__main__":
    unittest.main()
