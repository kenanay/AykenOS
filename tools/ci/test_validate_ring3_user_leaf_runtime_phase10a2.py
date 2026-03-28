#!/usr/bin/env python3
"""Black-box tests for validate_ring3_user_leaf_runtime_phase10a2.py."""

from __future__ import annotations

import json
import subprocess
import tempfile
import unittest
from pathlib import Path


class Ring3UserLeafRuntimeValidatorTest(unittest.TestCase):
    def setUp(self) -> None:
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)
        self.validator = Path(__file__).with_name(
            "validate_ring3_user_leaf_runtime_phase10a2.py"
        )
        self.log = self.root / "marker.log"
        self.report = self.root / "report.json"

    def tearDown(self) -> None:
        self.tmp.cleanup()

    def _write_log(self, lines: list[str]) -> None:
        self.log.write_text("\n".join(lines) + "\n", encoding="utf-8")

    def _run(self) -> tuple[int, dict]:
        proc = subprocess.run(
            [
                "python3",
                str(self.validator),
                "--log",
                str(self.log),
                "--out",
                str(self.report),
            ],
            check=False,
        )
        payload = json.loads(self.report.read_text(encoding="utf-8"))
        return proc.returncode, payload

    def test_pass_with_high_phys_matching_probe(self) -> None:
        self._write_log(
            [
                "P10_TEXT_FRAME_WITNESS phase=load root=0000000007E45000 pte=0000000007E3F007 phys=0000000007E3F000 used=1 lo=4E53B800400100BB hi=0 hash=1",
                "P10_TEXT_FRAME_WITNESS phase=pre_dispatch root=0000000007E45000 pte=0000000007E3F007 phys=0000000007E3F000 used=1 lo=4E53B800400100BB hi=0 hash=1",
                "P10_POST_CR3_TEXT_PROBE CR3=0000000007E45000 RIP=0000000000400000 Q=4E53B800400100BB",
                "P10_RING3_USER_CODE",
            ]
        )
        rc, report = self._run()
        self.assertEqual(rc, 0)
        self.assertEqual(report.get("verdict"), "PASS")

    def test_fail_when_pte_phys_mismatches_witness(self) -> None:
        self._write_log(
            [
                "P10_TEXT_FRAME_WITNESS phase=pre_dispatch root=0000000007E45000 pte=0000000007E3E007 phys=0000000007E3F000 used=1 lo=4E53B800400100BB hi=0 hash=1",
                "P10_POST_CR3_TEXT_PROBE CR3=0000000007E45000 RIP=0000000000400000 Q=4E53B800400100BB",
                "P10_RING3_USER_CODE",
            ]
        )
        rc, report = self._run()
        self.assertEqual(rc, 2)
        self.assertIn("text_leaf_pte_phys_mismatch:pte_phys=0x0000000007E3E000:witness_phys=0x0000000007E3F000", report.get("violations", []))

    def test_fail_when_probe_mismatches_witness(self) -> None:
        self._write_log(
            [
                "P10_TEXT_FRAME_WITNESS phase=pre_dispatch root=0000000007E45000 pte=0000000007E3F007 phys=0000000007E3F000 used=1 lo=4E53B800400100BB hi=0 hash=1",
                "P10_POST_CR3_TEXT_PROBE CR3=0000000007E45000 RIP=0000000000400000 Q=0000000000000000",
                "P10_RING3_USER_CODE",
            ]
        )
        rc, report = self._run()
        self.assertEqual(rc, 2)
        self.assertTrue(
            any(v.startswith("witness_probe_qword_mismatch:") for v in report.get("violations", []))
        )

    def test_fail_when_user_marker_missing(self) -> None:
        self._write_log(
            [
                "P10_TEXT_FRAME_WITNESS phase=pre_dispatch root=0000000007E45000 pte=0000000007E3F007 phys=0000000007E3F000 used=1 lo=4E53B800400100BB hi=0 hash=1",
                "P10_POST_CR3_TEXT_PROBE CR3=0000000007E45000 RIP=0000000000400000 Q=4E53B800400100BB",
            ]
        )
        rc, report = self._run()
        self.assertEqual(rc, 2)
        self.assertIn("missing_required:P10_RING3_USER_CODE", report.get("violations", []))


if __name__ == "__main__":
    unittest.main()
