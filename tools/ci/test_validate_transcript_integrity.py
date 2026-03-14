#!/usr/bin/env python3
"""Black-box tests for validate_transcript_integrity.py."""

from __future__ import annotations

# Author: Kenan AY

import json
import subprocess
import tempfile
import unittest
from pathlib import Path


class TranscriptIntegrityValidatorTest(unittest.TestCase):
    def setUp(self) -> None:
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)
        self.events = self.root / "events.jsonl"
        self.eti_jsonl = self.root / "eti_transcript.jsonl"
        self.eti_bin = self.root / "eti_transcript.bin"
        self.chain_verify = self.root / "eti_chain_verify.json"
        self.eti_report = self.root / "eti_report.json"
        self.report = self.root / "report.json"
        self.eti_validator = Path(__file__).with_name("validate_eti_sequence.py")
        self.validator = Path(__file__).with_name("validate_transcript_integrity.py")

    def tearDown(self) -> None:
        self.tmp.cleanup()

    def _write_events(self) -> None:
        rows = [
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
        with self.events.open("w", encoding="utf-8") as fh:
            for row in rows:
                fh.write(json.dumps(row, sort_keys=True) + "\n")

    def _materialize_eti(self) -> None:
        self._write_events()
        proc = subprocess.run(
            [
                "python3",
                str(self.eti_validator),
                "--events",
                str(self.events),
                "--out-eti-jsonl",
                str(self.eti_jsonl),
                "--out-eti-bin",
                str(self.eti_bin),
                "--out-chain-verify",
                str(self.chain_verify),
                "--out-report",
                str(self.eti_report),
            ],
            check=False,
        )
        if proc.returncode != 0:
            raise AssertionError("failed to materialize ETI test fixture")

    def _run(self) -> tuple[int, dict]:
        proc = subprocess.run(
            [
                "python3",
                str(self.validator),
                "--eti-jsonl",
                str(self.eti_jsonl),
                "--eti-bin",
                str(self.eti_bin),
                "--out-report",
                str(self.report),
            ],
            check=False,
        )
        report = json.loads(self.report.read_text(encoding="utf-8"))
        return proc.returncode, report

    def test_pass_on_valid_transcript(self) -> None:
        self._materialize_eti()
        rc, report = self._run()
        self.assertEqual(rc, 0)
        self.assertEqual(report.get("verdict"), "PASS")

    def test_fail_on_jsonl_tamper(self) -> None:
        self._materialize_eti()
        rows = [
            json.loads(line)
            for line in self.eti_jsonl.read_text(encoding="utf-8").splitlines()
            if line.strip()
        ]
        rows[0]["entry_hash"] = "00" * 32
        with self.eti_jsonl.open("w", encoding="utf-8") as fh:
            for row in rows:
                fh.write(json.dumps(row, sort_keys=True) + "\n")

        rc, report = self._run()
        self.assertEqual(rc, 2)
        self.assertIn("entry_hash_mismatch:entry=1", report.get("violations", []))

    def test_fail_on_bin_tamper(self) -> None:
        self._materialize_eti()
        blob = bytearray(self.eti_bin.read_bytes())
        blob[0] ^= 0x01
        self.eti_bin.write_bytes(bytes(blob))

        rc, report = self._run()
        self.assertEqual(rc, 2)
        self.assertIn("invalid_eti_bin_magic", report.get("violations", []))


if __name__ == "__main__":
    unittest.main()
