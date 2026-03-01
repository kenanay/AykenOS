#!/usr/bin/env python3
"""Black-box tests for validate_scheduler_mailbox_phase10c.py."""

from __future__ import annotations

import json
import subprocess
import tempfile
import unittest
from pathlib import Path

TOKEN_DISPATCH = "P10_SCHED_DISPATCH"
TOKEN_MAILBOX = "P10_MAILBOX_DECISION"
TOKEN_APPLIED = "P10_DECISION_APPLIED"
TOKEN_USER = "P10_RING3_USER_CODE"
TOKEN_FALLBACK = "P10_SCHED_FALLBACK"
C2_ACCEPT = "[[AYKEN_SCHED_MB_ACCEPT]] owner=2 epoch=1 cand=42 site=IRQ"
C2_REJECT = "[[AYKEN_SCHED_MB_REJECT]] reason=EPOCH_STALE owner=2 epoch=1 cand=42 site=IRQ"
C2_ARB_1 = (
    "[[AYKEN_SCHED_ARBITER_DECISION]] "
    "decision_id=11 site=IRQ owner=2 from=10 to=42 epoch=1"
)
C2_ARB_2 = (
    "[[AYKEN_SCHED_ARBITER_DECISION]] "
    "decision_id=12 site=IRQ owner=3 from=42 to=43 epoch=1"
)
C2_SWITCH_1 = "[[AYKEN_CTX_SWITCH]] decision_id=11 from=10 to=42"
C2_SWITCH_2 = "[[AYKEN_CTX_SWITCH]] decision_id=12 from=42 to=43"
C2_CURSOR_1 = "[[AYKEN_SCHED_CURSOR_ADVANCE]] decision_id=11 owner=2 next_owner=3"
C2_CURSOR_2 = "[[AYKEN_SCHED_CURSOR_ADVANCE]] decision_id=12 owner=3 next_owner=2"


class Phase10CMailboxValidatorTest(unittest.TestCase):
    def setUp(self) -> None:
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)
        self.validator = Path(__file__).with_name("validate_scheduler_mailbox_phase10c.py")
        self.events = self.root / "events.jsonl"
        self.marker_log = self.root / "marker.log"
        self.report = self.root / "report.json"

    def tearDown(self) -> None:
        self.tmp.cleanup()

    def _write_events(self, sequence: list[str]) -> None:
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

    def _write_marker_log(self, lines: list[str]) -> None:
        self.marker_log.write_text("\n".join(lines) + "\n", encoding="utf-8")

    def _run(
        self,
        require_metadata: str = "1",
        c2_strict: str = "0",
        c2_owner_set: str = "2",
        c2_require_cursor_marker: str = "1",
    ) -> tuple[int, dict]:
        proc = subprocess.run(
            [
                "python3",
                str(self.validator),
                "--events",
                str(self.events),
                "--log",
                str(self.marker_log),
                "--out",
                str(self.report),
                "--require-metadata",
                require_metadata,
                "--c2-strict",
                c2_strict,
                "--c2-owner-set",
                c2_owner_set,
                "--c2-require-cursor-marker",
                c2_require_cursor_marker,
            ],
            check=False,
        )
        payload = json.loads(self.report.read_text(encoding="utf-8"))
        return proc.returncode, payload

    def test_pass_with_valid_metadata(self) -> None:
        self._write_events([TOKEN_DISPATCH, TOKEN_MAILBOX, TOKEN_APPLIED, TOKEN_USER])
        self._write_marker_log(
            [
                TOKEN_DISPATCH,
                "P10_MAILBOX_DECISION id=9 pid=42 valid=1 src=2",
                "P10_DECISION_APPLIED id=9 pid=42 valid=0 src=2",
                TOKEN_USER,
            ]
        )
        rc, report = self._run()
        self.assertEqual(rc, 0)
        self.assertEqual(report.get("verdict"), "PASS")

    def test_fail_when_mailbox_missing(self) -> None:
        self._write_events([TOKEN_DISPATCH, TOKEN_APPLIED, TOKEN_USER])
        self._write_marker_log(
            [TOKEN_DISPATCH, "P10_DECISION_APPLIED id=7 pid=11 valid=0 src=2", TOKEN_USER]
        )
        rc, report = self._run()
        self.assertEqual(rc, 2)
        self.assertIn(f"missing_required:{TOKEN_MAILBOX}", report.get("violations", []))

    def test_fail_when_fallback_marker_present(self) -> None:
        self._write_events([TOKEN_DISPATCH, TOKEN_MAILBOX, TOKEN_APPLIED, TOKEN_USER])
        self._write_marker_log(
            [
                TOKEN_DISPATCH,
                "P10_MAILBOX_DECISION id=9 pid=42 valid=1 src=2",
                "P10_DECISION_APPLIED id=9 pid=42 valid=0 src=2",
                TOKEN_USER,
                "P10_SCHED_FALLBACK",
            ]
        )
        rc, report = self._run()
        self.assertEqual(rc, 2)
        self.assertIn(
            "forbidden_marker:P10_SCHED_FALLBACK:count=1",
            report.get("violations", []),
        )

    def test_fail_when_forbidden_fallback_present_in_events(self) -> None:
        self._write_events(
            [TOKEN_DISPATCH, TOKEN_MAILBOX, TOKEN_APPLIED, TOKEN_USER, TOKEN_FALLBACK]
        )
        self._write_marker_log(
            [
                TOKEN_DISPATCH,
                "P10_MAILBOX_DECISION id=9 pid=42 valid=1 src=2",
                "P10_DECISION_APPLIED id=9 pid=42 valid=0 src=2",
                TOKEN_USER,
            ]
        )
        rc, report = self._run()
        self.assertEqual(rc, 2)
        self.assertIn(
            "forbidden_event_marker:P10_SCHED_FALLBACK:count=1",
            report.get("violations", []),
        )

    def test_fail_when_metadata_missing_fields(self) -> None:
        self._write_events([TOKEN_DISPATCH, TOKEN_MAILBOX, TOKEN_APPLIED, TOKEN_USER])
        self._write_marker_log(
            [
                TOKEN_DISPATCH,
                TOKEN_MAILBOX,
                "P10_DECISION_APPLIED id=9 pid=42 valid=0 src=2",
                TOKEN_USER,
            ]
        )
        rc, report = self._run()
        self.assertEqual(rc, 2)
        self.assertTrue(
            any(v.startswith("metadata_missing_fields:P10_MAILBOX_DECISION:") for v in report.get("violations", []))
        )

    def test_fail_when_xprefixed_token_present(self) -> None:
        self._write_events([TOKEN_DISPATCH, TOKEN_MAILBOX, TOKEN_APPLIED, TOKEN_USER])
        self._write_marker_log(
            [
                TOKEN_DISPATCH,
                "P10_MAILBOX_DECISION id=9 pid=42 valid=1 src=2",
                "P10_DECISION_APPLIED id=9 pid=42 valid=0 src=2",
                "XP10_RING3_USER_CODE",
            ]
        )
        rc, report = self._run()
        self.assertEqual(rc, 2)
        self.assertIn(
            "forbidden_marker_prefix:XP10_RING3_USER_CODE",
            report.get("violations", []),
        )

    def test_fail_when_src_not_owner(self) -> None:
        self._write_events([TOKEN_DISPATCH, TOKEN_MAILBOX, TOKEN_APPLIED, TOKEN_USER])
        self._write_marker_log(
            [
                TOKEN_DISPATCH,
                "P10_MAILBOX_DECISION id=9 pid=42 valid=1 src=3",
                "P10_DECISION_APPLIED id=9 pid=42 valid=0 src=3",
                TOKEN_USER,
            ]
        )
        rc, report = self._run()
        self.assertEqual(rc, 2)
        self.assertIn(
            "decision_src_must_be_owner:got=3:expected=2",
            report.get("violations", []),
        )

    def test_fail_when_owner_mismatch_marker_present(self) -> None:
        self._write_events([TOKEN_DISPATCH, TOKEN_MAILBOX, TOKEN_APPLIED, TOKEN_USER])
        self._write_marker_log(
            [
                TOKEN_DISPATCH,
                "P10_MAILBOX_DECISION id=9 pid=42 valid=1 src=2",
                "P10_DECISION_APPLIED id=9 pid=42 valid=0 src=2",
                TOKEN_USER,
                "P10_MAILBOX_OWNER_MISMATCH",
            ]
        )
        rc, report = self._run()
        self.assertEqual(rc, 2)
        self.assertIn(
            "forbidden_marker:P10_MAILBOX_OWNER_MISMATCH:count=1",
            report.get("violations", []),
        )

    def test_c2_strict_pass_with_valid_markers(self) -> None:
        self._write_events(
            [TOKEN_DISPATCH, TOKEN_MAILBOX, TOKEN_APPLIED, TOKEN_USER, TOKEN_MAILBOX, TOKEN_APPLIED]
        )
        self._write_marker_log(
            [
                TOKEN_DISPATCH,
                C2_ACCEPT,
                C2_ARB_1,
                C2_SWITCH_1,
                C2_CURSOR_1,
                "[[AYKEN_SCHED_MB_ACCEPT]] owner=3 epoch=1 cand=43 site=IRQ",
                C2_ARB_2,
                C2_SWITCH_2,
                C2_CURSOR_2,
                TOKEN_USER,
            ]
        )
        rc, report = self._run(c2_strict="1", c2_owner_set="2,3")
        self.assertEqual(rc, 0)
        self.assertEqual(report.get("verdict"), "PASS")

    def test_c2_strict_fail_when_reject_followed_by_apply(self) -> None:
        self._write_events([TOKEN_DISPATCH, TOKEN_MAILBOX, TOKEN_APPLIED, TOKEN_USER])
        self._write_marker_log([TOKEN_DISPATCH, C2_REJECT, C2_ARB_1, C2_SWITCH_1, C2_CURSOR_1, TOKEN_USER])
        rc, report = self._run(c2_strict="1", c2_owner_set="2")
        self.assertEqual(rc, 2)
        self.assertIn(
            "reject_followed_by_apply:owner=2:epoch=1",
            report.get("violations", []),
        )

    def test_c2_strict_fail_when_cursor_marker_missing(self) -> None:
        self._write_events([TOKEN_DISPATCH, TOKEN_MAILBOX, TOKEN_APPLIED, TOKEN_USER])
        self._write_marker_log([TOKEN_DISPATCH, C2_ACCEPT, C2_ARB_1, C2_SWITCH_1, TOKEN_USER])
        rc, report = self._run(c2_strict="1", c2_owner_set="2", c2_require_cursor_marker="1")
        self.assertEqual(rc, 2)
        self.assertIn(
            "missing_required_c2:AYKEN_SCHED_CURSOR_ADVANCE",
            report.get("violations", []),
        )


if __name__ == "__main__":
    unittest.main()
