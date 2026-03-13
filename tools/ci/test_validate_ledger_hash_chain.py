#!/usr/bin/env python3
"""Black-box tests for validate_ledger_hash_chain.py."""

from __future__ import annotations

# Author: Kenan AY

import hashlib
import json
import subprocess
import tempfile
import unittest
from pathlib import Path


def sha256_hex(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def canonical_payload(row: dict) -> bytes:
    payload = {
        "decision_id": int(row["aux0"]),
        "decision_pid": int(row["next_ctx"]),
        "decision_src_pid": int(row["aux1"]),
        "decision_valid": int(row["decision_valid"]),
        "origin_event_line": int(row["origin_line"]),
        "origin_event_offset": int(row["origin_offset"]),
        "origin_event_type": str(row["origin_event_type"]),
    }
    return json.dumps(payload, sort_keys=True, separators=(",", ":")).encode("utf-8")


def build_entry(
    *,
    event_seq: int,
    ltick: int,
    prev_hash_hex: str,
    decision_id: int,
    decision_pid: int,
    decision_src_pid: int,
    origin_line: int,
    origin_offset: int,
    origin_event_type: str,
) -> dict:
    row = {
        "event_seq": event_seq,
        "ltick": ltick,
        "cpu_id": 0,
        "event_type": "AY_EVT_CTX_SWITCH",
        "event_type_value": 1,
        "prev_ctx": 0,
        "next_ctx": decision_pid,
        "decision_cap": 0,
        "reason_code": 1,
        "aux0": decision_id,
        "aux1": decision_src_pid,
        "decision_valid": 1,
        "origin_marker": "P10_MAILBOX_DECISION",
        "origin_event_type": origin_event_type,
        "origin_line": origin_line,
        "origin_offset": origin_offset,
        "magic": 0x3147444C,
        "version": 1,
        "flags": 0,
    }

    payload_hash = sha256_hex(canonical_payload(row))
    entry_hash = sha256_hex(bytes.fromhex(prev_hash_hex) + bytes.fromhex(payload_hash))

    row["payload_hash"] = payload_hash
    row["prev_hash"] = prev_hash_hex
    row["entry_hash"] = entry_hash
    return row


class LedgerHashChainValidatorTest(unittest.TestCase):
    def setUp(self) -> None:
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)
        self.ledger_jsonl = self.root / "decision_ledger.jsonl"
        self.chain_verify = self.root / "chain_verify.json"
        self.tamper_test = self.root / "tamper_test.json"
        self.report = self.root / "report.json"
        self.validator = Path(__file__).with_name("validate_ledger_hash_chain.py")

    def tearDown(self) -> None:
        self.tmp.cleanup()

    def _write_valid_ledger(self) -> None:
        e1 = build_entry(
            event_seq=2,
            ltick=2,
            prev_hash_hex=("00" * 32),
            decision_id=1,
            decision_pid=2,
            decision_src_pid=2,
            origin_line=10,
            origin_offset=100,
            origin_event_type="P10_MAILBOX_DECISION",
        )
        e2 = build_entry(
            event_seq=3,
            ltick=3,
            prev_hash_hex=e1["entry_hash"],
            decision_id=2,
            decision_pid=3,
            decision_src_pid=2,
            origin_line=11,
            origin_offset=120,
            origin_event_type="P10_MAILBOX_DECISION",
        )
        with self.ledger_jsonl.open("w", encoding="utf-8") as fh:
            fh.write(json.dumps(e1, sort_keys=True) + "\n")
            fh.write(json.dumps(e2, sort_keys=True) + "\n")

    def _run(self) -> tuple[int, dict, dict, dict]:
        proc = subprocess.run(
            [
                "python3",
                str(self.validator),
                "--ledger-jsonl",
                str(self.ledger_jsonl),
                "--out-chain-verify",
                str(self.chain_verify),
                "--out-tamper-test",
                str(self.tamper_test),
                "--out-report",
                str(self.report),
            ],
            check=False,
        )
        report = json.loads(self.report.read_text(encoding="utf-8"))
        chain_verify = json.loads(self.chain_verify.read_text(encoding="utf-8"))
        tamper_test = json.loads(self.tamper_test.read_text(encoding="utf-8"))
        return proc.returncode, report, chain_verify, tamper_test

    def test_pass_with_valid_chain_and_detect_tamper(self) -> None:
        self._write_valid_ledger()
        rc, report, chain_verify, tamper_test = self._run()
        self.assertEqual(rc, 0)
        self.assertEqual(report.get("verdict"), "PASS")
        self.assertEqual(chain_verify.get("verdict"), "PASS")
        self.assertEqual(tamper_test.get("detected"), 1)
        self.assertEqual(tamper_test.get("actual_verdict"), "FAIL")

    def test_fail_when_prev_hash_continuity_broken(self) -> None:
        self._write_valid_ledger()
        rows = [
            json.loads(line)
            for line in self.ledger_jsonl.read_text(encoding="utf-8").splitlines()
            if line.strip()
        ]
        rows[1]["prev_hash"] = "ff" * 32
        with self.ledger_jsonl.open("w", encoding="utf-8") as fh:
            for row in rows:
                fh.write(json.dumps(row, sort_keys=True) + "\n")

        rc, report, chain_verify, _ = self._run()
        self.assertEqual(rc, 2)
        self.assertEqual(report.get("verdict"), "FAIL")
        self.assertEqual(chain_verify.get("verdict"), "FAIL")
        self.assertIn(
            "prev_hash_continuity_mismatch:entry=2",
            report.get("violations", []),
        )


if __name__ == "__main__":
    unittest.main()
