#!/usr/bin/env python3
"""Tests for validate_error_codes.py."""

from __future__ import annotations

import json
import subprocess
import tempfile
import unittest
from pathlib import Path


ERROR_DB = {
    "AYK-E101": {"name": "witness_probe_mismatch", "layer": "L0", "description": "x"},
    "AYK-E102": {"name": "user_code_not_reached", "layer": "L0", "description": "y"},
    "AYK-E201": {"name": "probe_count_drift", "layer": "L1", "description": "z"},
}

VALID_VALIDATOR = """
VALIDATOR_ID = "AYK_KRN_L0_RING3_WITNESS_EQ_PROBE"
ERROR_CODE = "AYK-E101"

def validate(payload):
    return {"verdict": "PASS", "message": "ok", "details": {}}
"""

INVALID_VALIDATOR = """
VALIDATOR_ID = "AYK_KRN_L0_RING3_WITNESS_EQ_PROBE"
ERROR_CODE = "AYK-E999"

def validate(payload):
    return {"verdict": "PASS", "message": "ok", "details": {}}
"""

LAYER_MISMATCH_VALIDATOR = """
VALIDATOR_ID = "AYK_KRN_L0_RING3_WITNESS_EQ_PROBE"
ERROR_CODE = "AYK-E201"

def validate(payload):
    return {"verdict": "PASS", "message": "ok", "details": {}}
"""


class TestValidateErrorCodes(unittest.TestCase):
    def setUp(self) -> None:
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)
        self.validator = Path(__file__).with_name("validate_error_codes.py")
        (self.root / "docs/governance").mkdir(parents=True)
        (self.root / "tests/kernel/validators/l0").mkdir(parents=True)
        self.report = self.root / "report.json"
        (self.root / "docs/governance/error_codes.json").write_text(
            json.dumps(ERROR_DB),
            encoding="utf-8",
        )

    def tearDown(self) -> None:
        self.tmp.cleanup()

    def _run(self) -> tuple[int, dict]:
        proc = subprocess.run(
            [
                "python3",
                str(self.validator),
                "--root",
                str(self.root),
                "--error-db",
                "docs/governance/error_codes.json",
                "--out",
                str(self.report),
            ],
            check=False,
        )
        payload = json.loads(self.report.read_text(encoding="utf-8"))
        return proc.returncode, payload

    def test_pass_when_codes_are_defined(self) -> None:
        (self.root / "tests/kernel/validators/l0/validate_AYK_KRN_L0_RING3_WITNESS_EQ_PROBE.py").write_text(
            VALID_VALIDATOR,
            encoding="utf-8",
        )
        rc, report = self._run()
        self.assertEqual(rc, 0)
        self.assertEqual(report.get("verdict"), "PASS")

    def test_fail_when_code_is_undefined(self) -> None:
        (self.root / "tests/kernel/validators/l0/validate_AYK_KRN_L0_RING3_WITNESS_EQ_PROBE.py").write_text(
            INVALID_VALIDATOR,
            encoding="utf-8",
        )
        rc, report = self._run()
        self.assertEqual(rc, 2)
        self.assertIn("undefined_error_code:AYK-E999", report.get("violations", []))

    def test_fail_when_validator_layer_mismatches_error_layer(self) -> None:
        (self.root / "tests/kernel/validators/l0/validate_AYK_KRN_L0_RING3_WITNESS_EQ_PROBE.py").write_text(
            LAYER_MISMATCH_VALIDATOR,
            encoding="utf-8",
        )
        rc, report = self._run()
        self.assertEqual(rc, 2)
        self.assertTrue(
            any("validator_error_layer_mismatch" in item for item in report.get("violations", []))
        )


if __name__ == "__main__":
    unittest.main()
