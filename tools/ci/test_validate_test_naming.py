#!/usr/bin/env python3
"""Tests for validate_test_naming.py."""

from __future__ import annotations

import json
import subprocess
import tempfile
import unittest
from pathlib import Path


VALID_SCENARIO = {
    "schema_version": "1.0",
    "scenario_id": "AYK_SCN_KRN_RING3_FIRST_FETCH_BASE",
}

INVALID_SCENARIO = {
    "schema_version": "1.0",
    "scenario_id": "phase10_test",
}

VALID_VALIDATOR = """
VALIDATOR_ID = "AYK_KRN_L0_RING3_WITNESS_EQ_PROBE"
ERROR_CODE = "AYK-E101"

def validate(payload):
    return {"verdict": "PASS", "message": "ok", "details": {}}
"""

INVALID_VALIDATOR = """
VALIDATOR_ID = "ring3_test"
ERROR_CODE = "AYK-E101"

def validate(payload):
    return {"verdict": "PASS", "message": "ok", "details": {}}
"""


class TestValidateTestNaming(unittest.TestCase):
    def setUp(self) -> None:
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)
        self.validator = Path(__file__).with_name("validate_test_naming.py")
        (self.root / "tests/kernel/scenarios/ring3").mkdir(parents=True)
        (self.root / "tests/kernel/validators/l0").mkdir(parents=True)
        self.report = self.root / "report.json"

    def tearDown(self) -> None:
        self.tmp.cleanup()

    def _run(self) -> tuple[int, dict]:
        proc = subprocess.run(
            ["python3", str(self.validator), "--root", str(self.root), "--out", str(self.report)],
            check=False,
        )
        payload = json.loads(self.report.read_text(encoding="utf-8"))
        return proc.returncode, payload

    def test_pass_with_valid_ids(self) -> None:
        (self.root / "tests/kernel/scenarios/ring3/AYK_SCN_KRN_RING3_FIRST_FETCH_BASE.json").write_text(
            json.dumps(VALID_SCENARIO),
            encoding="utf-8",
        )
        (self.root / "tests/kernel/validators/l0/validate_AYK_KRN_L0_RING3_WITNESS_EQ_PROBE.py").write_text(
            VALID_VALIDATOR,
            encoding="utf-8",
        )
        rc, report = self._run()
        self.assertEqual(rc, 0)
        self.assertEqual(report.get("verdict"), "PASS")

    def test_fail_on_invalid_ids(self) -> None:
        (self.root / "tests/kernel/scenarios/ring3/AYK_SCN_KRN_RING3_FIRST_FETCH_BASE.json").write_text(
            json.dumps(INVALID_SCENARIO),
            encoding="utf-8",
        )
        (self.root / "tests/kernel/validators/l0/validate_AYK_KRN_L0_RING3_WITNESS_EQ_PROBE.py").write_text(
            INVALID_VALIDATOR,
            encoding="utf-8",
        )
        rc, report = self._run()
        self.assertEqual(rc, 2)
        self.assertTrue(any("invalid_scenario_id" in item for item in report.get("violations", [])))
        self.assertTrue(any("invalid_validator_id" in item for item in report.get("violations", [])))

    def test_fail_on_invalid_filenames_even_if_discovery_pattern_would_skip(self) -> None:
        (self.root / "tests/kernel/scenarios/ring3/phase10_first_fetch.json").write_text(
            json.dumps(VALID_SCENARIO),
            encoding="utf-8",
        )
        (self.root / "tests/kernel/validators/l0/ring3_test.py").write_text(
            VALID_VALIDATOR,
            encoding="utf-8",
        )
        rc, report = self._run()
        self.assertEqual(rc, 2)
        self.assertTrue(any("invalid_scenario_filename" in item for item in report.get("violations", [])))
        self.assertTrue(any("invalid_validator_filename" in item for item in report.get("violations", [])))


if __name__ == "__main__":
    unittest.main()
