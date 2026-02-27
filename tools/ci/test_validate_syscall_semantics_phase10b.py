#!/usr/bin/env python3
"""Black-box tests for validate_syscall_semantics_phase10b.py."""

from __future__ import annotations

import json
import subprocess
import tempfile
import unittest
from pathlib import Path

TOKEN_ENTER = "P10_SYSCALL_ENTER"
TOKEN_CAP = "P10_CAP_ENFORCED"
TOKEN_RETURN = "P10_SYSCALL_RETURN"
TOKEN_USER = "P10_RING3_USER_CODE"


class Phase10BSemanticsValidatorTest(unittest.TestCase):
    def setUp(self) -> None:
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)
        self.validator = Path(__file__).with_name("validate_syscall_semantics_phase10b.py")
        self.events = self.root / "events.jsonl"
        self.marker_log = self.root / "marker.log"
        self.report = self.root / "report.json"

    def tearDown(self) -> None:
        self.tmp.cleanup()

    def _write_inputs(
        self,
        sequence: list[str],
        marker_sequence: list[str] | None = None,
    ) -> None:
        with self.events.open("w", encoding="utf-8") as fh:
            offset = 0
            for idx, token in enumerate(sequence, start=1):
                row = {
                    "type": token,
                    "marker": token,
                    "offset": offset,
                    "line": idx,
                }
                fh.write(json.dumps(row, sort_keys=True) + "\n")
                offset += len(token) + 1
        log_sequence = marker_sequence if marker_sequence is not None else sequence
        self.marker_log.write_text("\n".join(log_sequence) + "\n", encoding="utf-8")

    def _run(self, mode: str) -> tuple[int, dict]:
        proc = subprocess.run(
            [
                "python3",
                str(self.validator),
                "--events",
                str(self.events),
                "--log",
                str(self.marker_log),
                "--mode",
                mode,
                "--out",
                str(self.report),
            ],
            check=False,
        )
        payload = json.loads(self.report.read_text(encoding="utf-8"))
        return proc.returncode, payload

    def test_positive_pass(self) -> None:
        self._write_inputs([TOKEN_ENTER, TOKEN_RETURN, TOKEN_USER])
        rc, report = self._run("positive")
        self.assertEqual(rc, 0)
        self.assertEqual(report.get("verdict"), "PASS")
        self.assertEqual(report.get("violations_count"), 0)

    def test_positive_fail_when_cap_present(self) -> None:
        self._write_inputs([TOKEN_ENTER, TOKEN_RETURN, TOKEN_CAP, TOKEN_USER])
        rc, report = self._run("positive")
        self.assertEqual(rc, 2)
        self.assertEqual(report.get("verdict"), "FAIL")
        self.assertIn("cap_forbidden_in_positive", " ".join(report.get("violations", [])))

    def test_negative_pass(self) -> None:
        self._write_inputs([TOKEN_ENTER, TOKEN_RETURN, TOKEN_CAP, TOKEN_USER])
        rc, report = self._run("negative")
        self.assertEqual(rc, 0)
        self.assertEqual(report.get("verdict"), "PASS")
        self.assertEqual(report.get("violations_count"), 0)

    def test_negative_fail_when_cap_missing(self) -> None:
        self._write_inputs([TOKEN_ENTER, TOKEN_RETURN, TOKEN_USER])
        rc, report = self._run("negative")
        self.assertEqual(rc, 2)
        self.assertEqual(report.get("verdict"), "FAIL")
        self.assertIn(
            "cap_required_in_negative:missing",
            report.get("violations", []),
        )

    def test_negative_fail_when_forbidden_xprefixed_token_in_log(self) -> None:
        self._write_inputs(
            [TOKEN_ENTER, TOKEN_RETURN, TOKEN_CAP, TOKEN_USER],
            [TOKEN_ENTER, TOKEN_RETURN, TOKEN_CAP, "XP10_RING3_USER_CODE"],
        )
        rc, report = self._run("negative")
        self.assertEqual(rc, 2)
        self.assertEqual(report.get("verdict"), "FAIL")
        self.assertIn(
            "forbidden_marker_prefix:XP10_RING3_USER_CODE",
            report.get("violations", []),
        )


if __name__ == "__main__":
    unittest.main()
