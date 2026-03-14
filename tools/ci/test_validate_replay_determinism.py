#!/usr/bin/env python3
"""Black-box tests for validate_replay_determinism.py."""

from __future__ import annotations

# Author: Kenan AY

import hashlib
import json
import subprocess
import tempfile
import unittest
from pathlib import Path


class ReplayDeterminismValidatorTest(unittest.TestCase):
    def setUp(self) -> None:
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)
        self.abdf_hash_file = self.root / "abdf_snapshot_hash.txt"
        self.bcib_hash_file = self.root / "bcib_plan_hash.txt"
        self.record_trace_jsonl = self.root / "execution_trace.jsonl"
        self.record_trace_hash_file = self.root / "execution_trace_hash.txt"
        self.expected_final_state_hash_file = self.root / "expected_final_state_hash.txt"

        self.replay_trace_jsonl = self.root / "replay_trace.jsonl"
        self.replay_trace_hash_txt = self.root / "replay_trace_hash.txt"
        self.replay_report_json = self.root / "replay_report.json"
        self.event_diff_txt = self.root / "event_diff.txt"
        self.ltick_diff_txt = self.root / "ltick_diff.txt"
        self.report_json = self.root / "report.json"

        self.validator = Path(__file__).with_name("validate_replay_determinism.py")

    def tearDown(self) -> None:
        self.tmp.cleanup()

    def _write_hash_file(self, path: Path, value: str) -> None:
        path.write_text(value + "\n", encoding="utf-8")

    def _trace_row(self, event_seq: int, ltick: int) -> dict:
        return {
            "trace_seq": event_seq,
            "event_seq": event_seq,
            "ltick": ltick,
            "cpu_id": 0,
            "event_type": "AY_EVT_SYSCALL_ENTER",
        }

    def _write_record_trace(self, rows: list[dict]) -> str:
        with self.record_trace_jsonl.open("w", encoding="utf-8") as fh:
            for row in rows:
                fh.write(json.dumps(row, sort_keys=True, separators=(",", ":")) + "\n")
        trace_hash = hashlib.sha256(self.record_trace_jsonl.read_bytes()).hexdigest()
        self._write_hash_file(self.record_trace_hash_file, trace_hash)
        return trace_hash

    def _run(
        self, expected_final_state_hash_file: Path | None = None
    ) -> tuple[int, dict, dict, str, str, str]:
        cmd = [
            "python3",
            str(self.validator),
            "--abdf-hash-file",
            str(self.abdf_hash_file),
            "--bcib-plan-hash-file",
            str(self.bcib_hash_file),
            "--record-trace-jsonl",
            str(self.record_trace_jsonl),
            "--record-trace-hash-file",
            str(self.record_trace_hash_file),
            "--out-replay-trace-jsonl",
            str(self.replay_trace_jsonl),
            "--out-replay-trace-hash-txt",
            str(self.replay_trace_hash_txt),
            "--out-replay-report",
            str(self.replay_report_json),
            "--out-event-diff",
            str(self.event_diff_txt),
            "--out-ltick-diff",
            str(self.ltick_diff_txt),
            "--out-report",
            str(self.report_json),
        ]
        if expected_final_state_hash_file is not None:
            cmd.extend(["--expected-final-state-hash-file", str(expected_final_state_hash_file)])

        proc = subprocess.run(cmd, check=False)
        report = json.loads(self.report_json.read_text(encoding="utf-8"))
        replay_report = json.loads(self.replay_report_json.read_text(encoding="utf-8"))
        replay_hash = self.replay_trace_hash_txt.read_text(encoding="utf-8").strip()
        event_diff = self.event_diff_txt.read_text(encoding="utf-8")
        ltick_diff = self.ltick_diff_txt.read_text(encoding="utf-8")
        return proc.returncode, report, replay_report, replay_hash, event_diff, ltick_diff

    def test_pass_with_valid_identity_and_trace(self) -> None:
        self._write_hash_file(self.abdf_hash_file, "a" * 64)
        self._write_hash_file(self.bcib_hash_file, "b" * 64)
        record_hash = self._write_record_trace([self._trace_row(1, 1), self._trace_row(2, 2)])
        rc, report, replay_report, replay_hash, event_diff, ltick_diff = self._run()
        self.assertEqual(rc, 0)
        self.assertEqual(report.get("verdict"), "PASS")
        self.assertEqual(replay_report.get("status"), "PASS")
        self.assertEqual(int(report.get("mismatch_count")), 0)
        self.assertEqual(str(report.get("record_execution_trace_hash")), record_hash)
        self.assertEqual(str(report.get("replay_execution_trace_hash")), replay_hash)
        self.assertEqual(event_diff.strip(), "")
        self.assertEqual(ltick_diff.strip(), "")

    def test_pass_with_expected_final_state_hash(self) -> None:
        self._write_hash_file(self.abdf_hash_file, "c" * 64)
        self._write_hash_file(self.bcib_hash_file, "d" * 64)
        self._write_record_trace([self._trace_row(10, 10), self._trace_row(20, 20)])
        rc0, report0, _, _, _, _ = self._run()
        self.assertEqual(rc0, 0)
        self._write_hash_file(
            self.expected_final_state_hash_file, str(report0.get("final_state_hash", ""))
        )
        rc, report, replay_report, _, _, _ = self._run(
            expected_final_state_hash_file=self.expected_final_state_hash_file
        )
        self.assertEqual(rc, 0)
        self.assertEqual(report.get("verdict"), "PASS")
        self.assertTrue(report.get("expected_final_state_hash_match"))
        self.assertEqual(replay_report.get("status"), "PASS")

    def test_fail_on_missing_abdf_hash_file(self) -> None:
        self._write_hash_file(self.bcib_hash_file, "b" * 64)
        self._write_record_trace([self._trace_row(1, 1)])
        rc, report, replay_report, _, _, _ = self._run()
        self.assertEqual(rc, 2)
        self.assertEqual(report.get("verdict"), "FAIL")
        self.assertEqual(replay_report.get("status"), "FAIL")
        self.assertTrue(
            any(v.startswith("missing_abdf_snapshot_hash_file:") for v in report.get("violations", []))
        )

    def test_fail_on_record_trace_hash_mismatch(self) -> None:
        self._write_hash_file(self.abdf_hash_file, "a" * 64)
        self._write_hash_file(self.bcib_hash_file, "b" * 64)
        self._write_record_trace([self._trace_row(1, 1), self._trace_row(2, 2)])
        self._write_hash_file(self.record_trace_hash_file, "f" * 64)
        rc, report, _, _, _, _ = self._run()
        self.assertEqual(rc, 2)
        self.assertTrue(
            any(v.startswith("record_trace_hash_mismatch:") for v in report.get("violations", []))
        )

    def test_fail_on_non_monotonic_record_trace(self) -> None:
        self._write_hash_file(self.abdf_hash_file, "a" * 64)
        self._write_hash_file(self.bcib_hash_file, "b" * 64)
        self._write_record_trace([self._trace_row(2, 2), self._trace_row(1, 1)])
        rc, report, _, _, _, _ = self._run()
        self.assertEqual(rc, 2)
        self.assertIn("record_trace_event_seq_non_monotonic", report.get("violations", []))
        self.assertIn("record_trace_ltick_non_monotonic", report.get("violations", []))

    def test_fail_on_expected_final_state_hash_mismatch(self) -> None:
        self._write_hash_file(self.abdf_hash_file, "a" * 64)
        self._write_hash_file(self.bcib_hash_file, "b" * 64)
        self._write_record_trace([self._trace_row(1, 1)])
        self._write_hash_file(self.expected_final_state_hash_file, "e" * 64)
        rc, report, _, _, _, _ = self._run(
            expected_final_state_hash_file=self.expected_final_state_hash_file
        )
        self.assertEqual(rc, 2)
        self.assertTrue(
            any(v.startswith("final_state_hash_mismatch:") for v in report.get("violations", []))
        )

    def test_fail_on_invalid_expected_final_state_hash_format(self) -> None:
        self._write_hash_file(self.abdf_hash_file, "a" * 64)
        self._write_hash_file(self.bcib_hash_file, "b" * 64)
        self._write_record_trace([self._trace_row(1, 1)])
        self.expected_final_state_hash_file.write_text("not-a-hash\n", encoding="utf-8")
        rc, report, _, _, _, _ = self._run(
            expected_final_state_hash_file=self.expected_final_state_hash_file
        )
        self.assertEqual(rc, 2)
        self.assertTrue(
            any(
                v.startswith("invalid_expected_final_state_hash_format:")
                for v in report.get("violations", [])
            )
        )


if __name__ == "__main__":
    unittest.main()
