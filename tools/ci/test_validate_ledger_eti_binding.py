#!/usr/bin/env python3
"""Black-box tests for validate_ledger_eti_binding.py."""

from __future__ import annotations

# Author: Kenan AY

import json
import subprocess
import tempfile
import unittest
from pathlib import Path


class LedgerEtiBindingValidatorTest(unittest.TestCase):
    def setUp(self) -> None:
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)
        self.ledger_jsonl = self.root / "decision_ledger.jsonl"
        self.eti_jsonl = self.root / "eti_transcript.jsonl"
        self.binding_report = self.root / "binding_report.json"
        self.report = self.root / "report.json"
        self.validator = Path(__file__).with_name("validate_ledger_eti_binding.py")

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
                "--ledger-jsonl",
                str(self.ledger_jsonl),
                "--eti-jsonl",
                str(self.eti_jsonl),
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

    def test_pass_with_matching_event_seq_and_ltick(self) -> None:
        self._write_jsonl(
            self.ledger_jsonl,
            [
                {"event_seq": 6, "ltick": 6, "event_type": "AY_EVT_CTX_SWITCH"},
                {"event_seq": 9, "ltick": 9, "event_type": "AY_EVT_CTX_SWITCH"},
            ],
        )
        self._write_jsonl(
            self.eti_jsonl,
            [
                {"event_seq": 6, "ltick": 6, "event_type": "AY_EVT_CTX_SWITCH"},
                {"event_seq": 8, "ltick": 8, "event_type": "AY_EVT_SYSCALL_ENTER"},
                {"event_seq": 9, "ltick": 9, "event_type": "AY_EVT_CTX_SWITCH"},
            ],
        )
        rc, report, binding = self._run()
        self.assertEqual(rc, 0)
        self.assertEqual(report.get("verdict"), "PASS")
        self.assertEqual(binding.get("status"), "PASS")

    def test_fail_when_binding_missing(self) -> None:
        self._write_jsonl(
            self.ledger_jsonl,
            [{"event_seq": 7, "ltick": 7, "event_type": "AY_EVT_CTX_SWITCH"}],
        )
        self._write_jsonl(
            self.eti_jsonl,
            [{"event_seq": 6, "ltick": 6, "event_type": "AY_EVT_CTX_SWITCH"}],
        )
        rc, report, _ = self._run()
        self.assertEqual(rc, 2)
        self.assertIn("missing_eti_binding:event_seq=7", report.get("violations", []))

    def test_fail_when_ltick_mismatch(self) -> None:
        self._write_jsonl(
            self.ledger_jsonl,
            [{"event_seq": 12, "ltick": 13, "event_type": "AY_EVT_CTX_SWITCH"}],
        )
        self._write_jsonl(
            self.eti_jsonl,
            [{"event_seq": 12, "ltick": 12, "event_type": "AY_EVT_CTX_SWITCH"}],
        )
        rc, report, _ = self._run()
        self.assertEqual(rc, 2)
        self.assertTrue(
            any(v.startswith("ltick_binding_mismatch:event_seq=12") for v in report.get("violations", []))
        )


if __name__ == "__main__":
    unittest.main()
