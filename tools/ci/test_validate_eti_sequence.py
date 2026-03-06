#!/usr/bin/env python3
"""Black-box tests for validate_eti_sequence.py."""

from __future__ import annotations

# Author: Kenan AY

import json
import subprocess
import tempfile
import unittest
from pathlib import Path


class EtiSequenceValidatorTest(unittest.TestCase):
    def setUp(self) -> None:
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)
        self.events = self.root / "events.jsonl"
        self.eti_jsonl = self.root / "eti_transcript.jsonl"
        self.eti_bin = self.root / "eti_transcript.bin"
        self.chain_verify = self.root / "eti_chain_verify.json"
        self.report = self.root / "report.json"
        self.validator = Path(__file__).with_name("validate_eti_sequence.py")

    def tearDown(self) -> None:
        self.tmp.cleanup()

    def _write_events(self, rows: list[dict]) -> None:
        with self.events.open("w", encoding="utf-8") as fh:
            for row in rows:
                fh.write(json.dumps(row, sort_keys=True) + "\n")

    def _run(self) -> tuple[int, dict, dict, list[dict]]:
        proc = subprocess.run(
            [
                "python3",
                str(self.validator),
                "--events",
                str(self.events),
                "--out-eti-jsonl",
                str(self.eti_jsonl),
                "--out-eti-bin",
                str(self.eti_bin),
                "--out-chain-verify",
                str(self.chain_verify),
                "--out-report",
                str(self.report),
            ],
            check=False,
        )
        report = json.loads(self.report.read_text(encoding="utf-8"))
        chain = json.loads(self.chain_verify.read_text(encoding="utf-8"))
        rows = [
            json.loads(line)
            for line in self.eti_jsonl.read_text(encoding="utf-8").splitlines()
            if line.strip()
        ]
        return proc.returncode, report, chain, rows

    def test_pass_with_required_events(self) -> None:
        self._write_events(
            [
                {"line": 1, "offset": 11, "marker": "IGNORED", "type": "IGNORED"},
                {
                    "line": 2,
                    "offset": 22,
                    "marker": "[[AYKEN_CTX_SWITCH]]",
                    "type": "AYKEN_CTX_SWITCH",
                },
                {
                    "line": 3,
                    "offset": 33,
                    "marker": "[[AYKEN_SYSCALL_ENTER]]",
                    "type": "AYKEN_SYSCALL_ENTER",
                },
                {
                    "line": 4,
                    "offset": 44,
                    "marker": "[[AYKEN_SYSCALL_RETURN]]",
                    "type": "AYKEN_SYSCALL_RETURN",
                },
            ]
        )
        rc, report, chain, rows = self._run()
        self.assertEqual(rc, 0)
        self.assertEqual(report.get("verdict"), "PASS")
        self.assertEqual(chain.get("status"), "PASS")
        self.assertEqual([int(row["event_seq"]) for row in rows], [2, 3, 4])
        self.assertGreater(len(self.eti_bin.read_bytes()), 0)

    def test_fail_when_required_exit_missing(self) -> None:
        self._write_events(
            [
                {
                    "line": 2,
                    "offset": 22,
                    "marker": "[[AYKEN_CTX_SWITCH]]",
                    "type": "AYKEN_CTX_SWITCH",
                },
                {
                    "line": 3,
                    "offset": 33,
                    "marker": "[[AYKEN_SYSCALL_ENTER]]",
                    "type": "AYKEN_SYSCALL_ENTER",
                },
            ]
        )
        rc, report, chain, _ = self._run()
        self.assertEqual(rc, 2)
        self.assertEqual(report.get("verdict"), "FAIL")
        self.assertEqual(chain.get("status"), "FAIL")
        self.assertIn(
            "missing_required_event_type:AY_EVT_SYSCALL_EXIT",
            report.get("violations", []),
        )

    def test_fail_with_empty_stream(self) -> None:
        self._write_events([])
        rc, report, _, rows = self._run()
        self.assertEqual(rc, 2)
        self.assertEqual(rows, [])
        self.assertIn("empty_eti_stream", report.get("violations", []))


if __name__ == "__main__":
    unittest.main()
