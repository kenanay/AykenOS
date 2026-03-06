#!/usr/bin/env python3
"""Black-box tests for validate_eti_dlt_binding.py."""

from __future__ import annotations

# Author: Kenan AY

import json
import subprocess
import tempfile
import unittest
from pathlib import Path


class EtiDltBindingValidatorTest(unittest.TestCase):
    def setUp(self) -> None:
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)
        self.eti_jsonl = self.root / "eti_transcript.jsonl"
        self.ltick_trace = self.root / "ltick_trace.jsonl"
        self.binding_report = self.root / "binding_report.json"
        self.report = self.root / "report.json"
        self.validator = Path(__file__).with_name("validate_eti_dlt_binding.py")

    def tearDown(self) -> None:
        self.tmp.cleanup()

    def _write_jsonl(self, path: Path, rows: list[dict]) -> None:
        with path.open("w", encoding="utf-8") as fh:
            for row in rows:
                fh.write(json.dumps(row, sort_keys=True) + "\n")

    def _run(self) -> tuple[int, dict, dict]:
        proc = subprocess.run(
            [
                "python3",
                str(self.validator),
                "--eti-jsonl",
                str(self.eti_jsonl),
                "--ltick-trace-jsonl",
                str(self.ltick_trace),
                "--out-binding-report",
                str(self.binding_report),
                "--out-report",
                str(self.report),
            ],
            check=False,
        )
        report = json.loads(self.report.read_text(encoding="utf-8"))
        binding = json.loads(self.binding_report.read_text(encoding="utf-8"))
        return proc.returncode, report, binding

    def _eti_row(self, event_seq: int, ltick: int) -> dict:
        return {
            "event_seq": event_seq,
            "ltick": ltick,
            "event_type": "AY_EVT_SYSCALL_ENTER",
        }

    def _dlt_row(self, event_seq: int, ltick: int, source_event_seq: int, source_ltick: int) -> dict:
        return {
            "event_seq": event_seq,
            "ltick": ltick,
            "source_event_seq": source_event_seq,
            "source_ltick": source_ltick,
            "cpu_id": 0,
            "event_type": "AY_EVT_SYSCALL_ENTER",
        }

    def test_pass_on_strict_source_identity_match(self) -> None:
        self._write_jsonl(self.eti_jsonl, [self._eti_row(2, 2), self._eti_row(5, 5)])
        self._write_jsonl(
            self.ltick_trace,
            [
                self._dlt_row(1, 1, 2, 2),
                self._dlt_row(2, 2, 5, 5),
            ],
        )
        rc, report, binding = self._run()
        self.assertEqual(rc, 0)
        self.assertEqual(report.get("verdict"), "PASS")
        self.assertEqual(binding.get("status"), "PASS")

    def test_fail_on_missing_dlt_binding(self) -> None:
        self._write_jsonl(self.eti_jsonl, [self._eti_row(7, 7)])
        self._write_jsonl(self.ltick_trace, [self._dlt_row(1, 1, 6, 6)])
        rc, report, _ = self._run()
        self.assertEqual(rc, 2)
        self.assertIn("missing_dlt_binding:event_seq=7", report.get("violations", []))

    def test_fail_on_source_ltick_mismatch(self) -> None:
        self._write_jsonl(self.eti_jsonl, [self._eti_row(12, 12)])
        self._write_jsonl(self.ltick_trace, [self._dlt_row(1, 1, 12, 13)])
        rc, report, _ = self._run()
        self.assertEqual(rc, 2)
        self.assertTrue(
            any(v.startswith("source_ltick_mismatch:event_seq=12") for v in report.get("violations", []))
        )


if __name__ == "__main__":
    unittest.main()
