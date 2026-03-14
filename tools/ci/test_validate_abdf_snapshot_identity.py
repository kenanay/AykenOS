#!/usr/bin/env python3
"""Black-box tests for validate_abdf_snapshot_identity.py."""

from __future__ import annotations

# Author: Kenan AY

import hashlib
import json
import subprocess
import tempfile
import unittest
from pathlib import Path


class AbdfSnapshotIdentityValidatorTest(unittest.TestCase):
    def setUp(self) -> None:
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)
        self.snapshot_bin = self.root / "snapshot.abdf"
        self.expected_hash = self.root / "expected_hash.txt"
        self.hash_txt = self.root / "abdf_snapshot_hash.txt"
        self.identity_report = self.root / "snapshot_identity_report.json"
        self.consistency_report = self.root / "snapshot_identity_consistency.json"
        self.report = self.root / "report.json"
        self.validator = Path(__file__).with_name("validate_abdf_snapshot_identity.py")

    def tearDown(self) -> None:
        self.tmp.cleanup()

    def _write_snapshot(self, payload: bytes) -> None:
        self.snapshot_bin.write_bytes(payload)

    def _run(self, expected_hash: Path | None = None) -> tuple[int, dict, dict, dict, str]:
        cmd = [
            "python3",
            str(self.validator),
            "--snapshot-bin",
            str(self.snapshot_bin),
            "--out-hash-txt",
            str(self.hash_txt),
            "--out-identity-report",
            str(self.identity_report),
            "--out-consistency-report",
            str(self.consistency_report),
            "--out-report",
            str(self.report),
        ]
        if expected_hash is not None:
            cmd.extend(["--expected-hash-file", str(expected_hash)])
        proc = subprocess.run(cmd, check=False)
        report = json.loads(self.report.read_text(encoding="utf-8"))
        identity = json.loads(self.identity_report.read_text(encoding="utf-8"))
        consistency = json.loads(self.consistency_report.read_text(encoding="utf-8"))
        computed_hash = self.hash_txt.read_text(encoding="utf-8").strip()
        return proc.returncode, report, identity, consistency, computed_hash

    def test_pass_computes_hash_from_binary_snapshot(self) -> None:
        payload = b"ABDF\x00\x01\x02\x03"
        self._write_snapshot(payload)
        expected = hashlib.sha256(payload).hexdigest()
        rc, report, identity, consistency, computed = self._run()
        self.assertEqual(rc, 0)
        self.assertEqual(report.get("verdict"), "PASS")
        self.assertEqual(identity.get("status"), "PASS")
        self.assertEqual(consistency.get("status"), "PASS")
        self.assertEqual(computed, expected)
        self.assertEqual(str(report.get("abdf_snapshot_hash")), expected)

    def test_pass_when_expected_hash_matches(self) -> None:
        payload = b"ABDF\x10\x20\x30"
        self._write_snapshot(payload)
        self.expected_hash.write_text(hashlib.sha256(payload).hexdigest() + "\n", encoding="utf-8")
        rc, report, _, consistency, _ = self._run(expected_hash=self.expected_hash)
        self.assertEqual(rc, 0)
        self.assertEqual(report.get("verdict"), "PASS")
        self.assertTrue(report.get("expected_hash_match"))
        self.assertTrue(consistency.get("expected_hash_match"))

    def test_fail_on_expected_hash_mismatch(self) -> None:
        self._write_snapshot(b"ABDF\x01\x02")
        self.expected_hash.write_text(("f" * 64) + "\n", encoding="utf-8")
        rc, report, identity, consistency, _ = self._run(expected_hash=self.expected_hash)
        self.assertEqual(rc, 2)
        self.assertEqual(report.get("verdict"), "FAIL")
        self.assertEqual(identity.get("status"), "FAIL")
        self.assertEqual(consistency.get("status"), "FAIL")
        self.assertTrue(
            any(v.startswith("abdf_snapshot_hash_mismatch:") for v in report.get("violations", []))
        )

    def test_fail_on_invalid_expected_hash_format(self) -> None:
        self._write_snapshot(b"ABDF\x01\x02")
        self.expected_hash.write_text("not-a-hash\n", encoding="utf-8")
        rc, report, _, _, _ = self._run(expected_hash=self.expected_hash)
        self.assertEqual(rc, 2)
        self.assertTrue(
            any(v.startswith("invalid_expected_hash_format:") for v in report.get("violations", []))
        )

    def test_fail_on_empty_snapshot(self) -> None:
        self._write_snapshot(b"")
        rc, report, _, _, _ = self._run()
        self.assertEqual(rc, 2)
        self.assertIn("empty_abdf_snapshot_bin", report.get("violations", []))

    def test_fail_on_missing_snapshot(self) -> None:
        # Intentionally keep snapshot absent.
        rc, report, _, _, _ = self._run()
        self.assertEqual(rc, 2)
        self.assertTrue(
            any(v.startswith("missing_abdf_snapshot_bin:") for v in report.get("violations", []))
        )


if __name__ == "__main__":
    unittest.main()
