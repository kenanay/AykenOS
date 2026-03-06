#!/usr/bin/env python3
"""Black-box tests for validate_gcp_finalization.py."""

from __future__ import annotations

# Author: Kenan AY

import json
import subprocess
import tempfile
import unittest
from pathlib import Path


class GcpFinalizationValidatorTest(unittest.TestCase):
    def setUp(self) -> None:
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)
        self.dlt_trace = self.root / "ltick_trace.jsonl"
        self.gcp_snapshot = self.root / "gcp_snapshot.json"
        self.gcp_record = self.root / "gcp_record.json"
        self.gcp_consistency_report = self.root / "gcp_consistency_report.json"
        self.report = self.root / "report.json"
        self.previous_gcp = self.root / "previous_gcp.json"
        self.validator = Path(__file__).with_name("validate_gcp_finalization.py")

    def tearDown(self) -> None:
        self.tmp.cleanup()

    def _write_dlt_rows(self, rows: list[dict]) -> None:
        with self.dlt_trace.open("w", encoding="utf-8") as fh:
            for row in rows:
                fh.write(json.dumps(row, sort_keys=True) + "\n")

    def _run(self, previous_gcp: Path | None = None) -> tuple[int, dict, dict, dict]:
        cmd = [
            "python3",
            str(self.validator),
            "--dlt-trace-jsonl",
            str(self.dlt_trace),
            "--out-gcp-snapshot",
            str(self.gcp_snapshot),
            "--out-gcp-record",
            str(self.gcp_record),
            "--out-gcp-consistency-report",
            str(self.gcp_consistency_report),
            "--out-report",
            str(self.report),
        ]
        if previous_gcp is not None:
            cmd.extend(["--previous-gcp", str(previous_gcp)])

        proc = subprocess.run(cmd, check=False)
        report = json.loads(self.report.read_text(encoding="utf-8"))
        snapshot = json.loads(self.gcp_snapshot.read_text(encoding="utf-8"))
        consistency = json.loads(self.gcp_consistency_report.read_text(encoding="utf-8"))
        return proc.returncode, report, snapshot, consistency

    def _dlt_row(self, event_seq: int, ltick: int) -> dict:
        return {
            "event_seq": event_seq,
            "ltick": ltick,
            "source_event_seq": 10 + event_seq,
            "source_ltick": 10 + ltick,
            "cpu_id": 0,
            "event_type": "AY_EVT_SYSCALL_ENTER",
        }

    def test_pass_with_valid_trace(self) -> None:
        self._write_dlt_rows([self._dlt_row(1, 1), self._dlt_row(2, 2), self._dlt_row(3, 3)])
        rc, report, snapshot, consistency = self._run()
        self.assertEqual(rc, 0)
        self.assertEqual(report.get("verdict"), "PASS")
        self.assertEqual(snapshot.get("status"), "PASS")
        self.assertEqual(consistency.get("status"), "PASS")
        self.assertEqual(int(snapshot.get("gcp_ltick")), 3)
        self.assertEqual(int(snapshot.get("gcp_event_seq")), 3)

    def test_fail_on_ltick_gap(self) -> None:
        self._write_dlt_rows([self._dlt_row(1, 1), self._dlt_row(2, 3)])
        rc, report, _, consistency = self._run()
        self.assertEqual(rc, 2)
        self.assertEqual(report.get("verdict"), "FAIL")
        self.assertEqual(consistency.get("status"), "FAIL")
        self.assertIn("dlt_ltick_gap", report.get("violations", []))

    def test_fail_on_previous_gcp_non_monotonic(self) -> None:
        self._write_dlt_rows([self._dlt_row(1, 1), self._dlt_row(2, 2), self._dlt_row(3, 3)])
        self.previous_gcp.write_text(
            json.dumps({"gcp_ltick": 9}, sort_keys=True) + "\n",
            encoding="utf-8",
        )
        rc, report, _, _ = self._run(previous_gcp=self.previous_gcp)
        self.assertEqual(rc, 2)
        self.assertTrue(
            any(v.startswith("gcp_non_monotonic_previous:") for v in report.get("violations", []))
        )


if __name__ == "__main__":
    unittest.main()
