#!/usr/bin/env python3
"""Black-box tests for validate_mailbox_capability_negative.py."""

from __future__ import annotations

import json
import subprocess
import tempfile
import unittest
from pathlib import Path


class MailboxCapabilityNegativeValidatorTest(unittest.TestCase):
    def setUp(self) -> None:
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)
        self.report = self.root / "report.json"
        self.matrix = self.root / "negative_matrix.json"
        self.validator = Path(__file__).with_name(
            "validate_mailbox_capability_negative.py"
        )
        self.repo_root = Path(__file__).resolve().parents[2]
        self.header = self.repo_root / "kernel/include/sched_mailbox_abi.h"
        self.source = self.repo_root / "kernel/sched/sched_mailbox.c"

    def tearDown(self) -> None:
        self.tmp.cleanup()

    def _run(self, header: Path, source: Path) -> tuple[int, dict, dict]:
        proc = subprocess.run(
            [
                "python3",
                str(self.validator),
                "--header",
                str(header),
                "--source",
                str(source),
                "--out-report",
                str(self.report),
                "--out-matrix",
                str(self.matrix),
            ],
            check=False,
        )
        report_payload = json.loads(self.report.read_text(encoding="utf-8"))
        matrix_payload = json.loads(self.matrix.read_text(encoding="utf-8"))
        return proc.returncode, report_payload, matrix_payload

    def test_pass_with_repository_sources(self) -> None:
        rc, report, matrix = self._run(self.header, self.source)
        self.assertEqual(rc, 0)
        self.assertEqual(report.get("verdict"), "PASS")
        self.assertEqual(report.get("violations_count"), 0)
        case_ids = {row.get("id") for row in matrix.get("cases", [])}
        self.assertIn("bad_signature", case_ids)
        self.assertIn("capability_missing", case_ids)
        self.assertIn("budget_exceeded_by_flag", case_ids)
        self.assertIn("invalid_pid_zero", case_ids)

    def test_fail_when_required_symbol_is_missing(self) -> None:
        bad_header = self.root / "sched_mailbox_abi.h"
        bad_source = self.root / "sched_mailbox.c"
        bad_header.write_text(
            self.header.read_text(encoding="utf-8").replace("REJ_BAD_SIG", "REJ_BAD_SIG_REMOVED"),
            encoding="utf-8",
        )
        bad_source.write_text(self.source.read_text(encoding="utf-8"), encoding="utf-8")

        rc, report, _ = self._run(bad_header, bad_source)
        self.assertEqual(rc, 2)
        self.assertEqual(report.get("verdict"), "FAIL")
        self.assertTrue(
            any(
                violation.startswith("missing_header_symbol:REJ_BAD_SIG")
                for violation in report.get("violations", [])
            )
        )


if __name__ == "__main__":
    unittest.main()
