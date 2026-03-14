#!/usr/bin/env python3
"""Black-box tests for validate_ledger_completeness.py."""

from __future__ import annotations

# Author: Kenan AY

import json
import struct
import subprocess
import tempfile
import unittest
from pathlib import Path

TOKEN_CTX_SWITCH = "[[AYKEN_CTX_SWITCH]]"
TOKEN_MAILBOX = "P10_MAILBOX_DECISION"


class LedgerCompletenessValidatorTest(unittest.TestCase):
    def setUp(self) -> None:
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)
        self.events = self.root / "events.jsonl"
        self.marker_log = self.root / "marker.log"
        self.report = self.root / "report.json"
        self.ledger_jsonl = self.root / "decision_ledger.jsonl"
        self.ledger_bin = self.root / "decision_ledger.bin"
        self.validator = Path(__file__).with_name("validate_ledger_completeness.py")

    def tearDown(self) -> None:
        self.tmp.cleanup()

    def _write_events(self, tokens: list[str]) -> None:
        with self.events.open("w", encoding="utf-8") as fh:
            offset = 0
            for idx, token in enumerate(tokens, start=1):
                row = {
                    "line": idx,
                    "marker": token,
                    "offset": offset,
                    "type": token,
                }
                fh.write(json.dumps(row, sort_keys=True) + "\n")
                offset += len(token) + 1

    def _write_log(self, lines: list[str]) -> None:
        self.marker_log.write_text("\n".join(lines) + "\n", encoding="utf-8")

    def _run(self) -> tuple[int, dict, list[dict]]:
        proc = subprocess.run(
            [
                "python3",
                str(self.validator),
                "--events",
                str(self.events),
                "--log",
                str(self.marker_log),
                "--out-report",
                str(self.report),
                "--out-ledger-jsonl",
                str(self.ledger_jsonl),
                "--out-ledger-bin",
                str(self.ledger_bin),
            ],
            check=False,
        )
        report = json.loads(self.report.read_text(encoding="utf-8"))
        rows = [
            json.loads(line)
            for line in self.ledger_jsonl.read_text(encoding="utf-8").splitlines()
            if line.strip()
        ]
        return proc.returncode, report, rows

    def test_pass_with_one_switch_one_decision(self) -> None:
        self._write_events(["P10_SCHED_DISPATCH", TOKEN_CTX_SWITCH, "P10_RING3_USER_CODE"])
        self._write_log(
            [
                "P10_SCHED_DISPATCH",
                "P10_MAILBOX_DECISION id=1 pid=2 valid=1 src=2",
                TOKEN_CTX_SWITCH,
            ]
        )

        rc, report, rows = self._run()
        self.assertEqual(rc, 0)
        self.assertEqual(report.get("verdict"), "PASS")
        self.assertEqual(report.get("entries_count"), 1)
        self.assertEqual(len(rows), 1)
        self.assertEqual(rows[0]["event_seq"], 2)
        self.assertEqual(rows[0]["ltick"], 2)

        blob = self.ledger_bin.read_bytes()
        self.assertGreaterEqual(len(blob), 64)
        self.assertEqual(blob[:4], b"LDG1")
        version = struct.unpack_from("<H", blob, 4)[0]
        count = struct.unpack_from("<Q", blob, 6)[0]
        self.assertEqual(version, 1)
        self.assertEqual(count, 1)

    def test_fail_when_switch_and_decision_counts_mismatch(self) -> None:
        self._write_events([TOKEN_CTX_SWITCH, TOKEN_CTX_SWITCH])
        self._write_log(
            [
                TOKEN_CTX_SWITCH,
                TOKEN_CTX_SWITCH,
                "P10_MAILBOX_DECISION id=1 pid=2 valid=1 src=2",
            ]
        )

        rc, report, rows = self._run()
        self.assertEqual(rc, 2)
        self.assertEqual(report.get("verdict"), "FAIL")
        self.assertIn(
            "switch_decision_count_mismatch:switch=2:decision=1",
            report.get("violations", []),
        )
        self.assertEqual(len(rows), 1)

    def test_fail_when_decision_id_non_monotonic(self) -> None:
        self._write_events([TOKEN_CTX_SWITCH, TOKEN_CTX_SWITCH])
        self._write_log(
            [
                TOKEN_CTX_SWITCH,
                TOKEN_CTX_SWITCH,
                "P10_MAILBOX_DECISION id=2 pid=2 valid=1 src=2",
                "P10_MAILBOX_DECISION id=1 pid=3 valid=1 src=2",
            ]
        )

        rc, report, _ = self._run()
        self.assertEqual(rc, 2)
        self.assertEqual(report.get("verdict"), "FAIL")
        self.assertIn(
            "decision_id_non_monotonic:prev=2:curr=1",
            report.get("violations", []),
        )


if __name__ == "__main__":
    unittest.main()
