#!/usr/bin/env python3
"""Black-box tests for validate_deol_sequence.py."""

from __future__ import annotations

# Author: Kenan AY

import json
import subprocess
import tempfile
import unittest
from pathlib import Path


class DeolSequenceValidatorTest(unittest.TestCase):
    def setUp(self) -> None:
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)
        self.ledger_jsonl = self.root / "decision_ledger.jsonl"
        self.event_seq_jsonl = self.root / "event_seq.jsonl"
        self.sequence_report = self.root / "sequence_report.json"
        self.report = self.root / "report.json"
        self.validator = Path(__file__).with_name("validate_deol_sequence.py")

    def tearDown(self) -> None:
        self.tmp.cleanup()

    def _write_ledger_rows(self, rows: list[dict]) -> None:
        with self.ledger_jsonl.open("w", encoding="utf-8") as fh:
            for row in rows:
                fh.write(json.dumps(row, sort_keys=True) + "\n")

    def _base_entry(self, event_seq: int, ltick: int) -> dict:
        return {
            "event_seq": event_seq,
            "ltick": ltick,
            "event_type": "AY_EVT_CTX_SWITCH",
        }

    def _run(self) -> tuple[int, dict, dict, list[dict]]:
        proc = subprocess.run(
            [
                "python3",
                str(self.validator),
                "--ledger-jsonl",
                str(self.ledger_jsonl),
                "--out-event-seq",
                str(self.event_seq_jsonl),
                "--out-sequence-report",
                str(self.sequence_report),
                "--out-report",
                str(self.report),
            ],
            check=False,
        )
        report = json.loads(self.report.read_text(encoding="utf-8"))
        sequence_report = json.loads(self.sequence_report.read_text(encoding="utf-8"))
        event_rows = [
            json.loads(line)
            for line in self.event_seq_jsonl.read_text(encoding="utf-8").splitlines()
            if line.strip()
        ]
        return proc.returncode, report, sequence_report, event_rows

    def test_pass_with_monotonic_unique_source(self) -> None:
        self._write_ledger_rows(
            [
                self._base_entry(6, 6),
                self._base_entry(10, 10),
                self._base_entry(15, 15),
            ]
        )
        rc, report, sequence_report, event_rows = self._run()
        self.assertEqual(rc, 0)
        self.assertEqual(report.get("verdict"), "PASS")
        self.assertEqual(sequence_report.get("status"), "PASS")
        self.assertEqual([row["event_seq"] for row in event_rows], [1, 2, 3])
        self.assertEqual([row["ltick"] for row in event_rows], [1, 2, 3])

    def test_fail_on_duplicate_source_event_seq(self) -> None:
        self._write_ledger_rows(
            [
                self._base_entry(6, 6),
                self._base_entry(6, 7),
            ]
        )
        rc, report, sequence_report, _ = self._run()
        self.assertEqual(rc, 2)
        self.assertEqual(report.get("verdict"), "FAIL")
        self.assertEqual(sequence_report.get("status"), "FAIL")
        self.assertIn("source_event_seq_duplicate", report.get("violations", []))

    def test_fail_on_non_monotonic_source_ltick(self) -> None:
        self._write_ledger_rows(
            [
                self._base_entry(6, 7),
                self._base_entry(8, 6),
            ]
        )
        rc, report, _, _ = self._run()
        self.assertEqual(rc, 2)
        self.assertIn("source_ltick_non_monotonic", report.get("violations", []))


if __name__ == "__main__":
    unittest.main()
