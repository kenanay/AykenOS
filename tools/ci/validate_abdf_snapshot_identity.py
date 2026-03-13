#!/usr/bin/env python3
"""Validate Phase-11 ABDF snapshot identity from canonical binary bytes."""

from __future__ import annotations

# Author: Kenan AY

import argparse
import hashlib
import json
from pathlib import Path
from typing import Any


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Validate ABDF snapshot identity hash for replay-root input integrity."
    )
    parser.add_argument("--snapshot-bin", required=True, help="ABDF snapshot binary path")
    parser.add_argument("--out-hash-txt", required=True, help="Output abdf_snapshot_hash.txt path")
    parser.add_argument(
        "--out-identity-report",
        required=True,
        help="Output snapshot_identity_report.json path",
    )
    parser.add_argument(
        "--out-consistency-report",
        required=True,
        help="Output snapshot_identity_consistency.json path",
    )
    parser.add_argument("--out-report", required=True, help="Output report.json path")
    parser.add_argument(
        "--expected-hash-file",
        required=False,
        default="",
        help="Optional expected hash file (first token is consumed as expected hash)",
    )
    return parser.parse_args()


def write_json(path: Path, payload: dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def write_hash(path: Path, hash_value: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text((hash_value or "") + "\n", encoding="utf-8")


def sha256_hex(payload: bytes) -> str:
    return hashlib.sha256(payload).hexdigest()


def is_sha256_hex(value: str) -> bool:
    if not isinstance(value, str) or len(value) != 64:
        return False
    return all(ch in "0123456789abcdef" for ch in value.lower())


def normalize_expected_hash(raw_text: str) -> str:
    for line in raw_text.splitlines():
        tokenized = line.strip()
        if not tokenized:
            continue
        return tokenized.split()[0].strip().lower()
    return ""


def fail(
    report_path: Path,
    hash_path: Path,
    identity_report_path: Path,
    consistency_report_path: Path,
    report: dict[str, Any],
) -> int:
    report["verdict"] = "FAIL"
    report["violations_count"] = len(report.get("violations", []))
    write_json(report_path, report)
    write_hash(hash_path, str(report.get("abdf_snapshot_hash", "")))

    identity_payload = {
        "status": "FAIL",
        "mode": "bootstrap_abdf_snapshot_identity",
        "hash_algorithm": "sha256",
        "canonical_input": "snapshot_binary_bytes",
        "snapshot_bin": str(report.get("snapshot_bin", "")),
        "snapshot_size_bytes": int(report.get("snapshot_size_bytes", 0)),
        "abdf_snapshot_hash": str(report.get("abdf_snapshot_hash", "")),
        "expected_hash_file": str(report.get("expected_hash_file", "")),
        "expected_hash": str(report.get("expected_hash", "")),
        "expected_hash_match": bool(report.get("expected_hash_match", False)),
        "hash_recomputed_match": bool(report.get("hash_recomputed_match", False)),
        "violations": list(report.get("violations", [])),
        "violations_count": len(report.get("violations", [])),
    }
    write_json(identity_report_path, identity_payload)

    consistency_payload = {
        "status": "FAIL",
        "mode": "bootstrap_abdf_snapshot_identity",
        "snapshot_size_bytes": int(report.get("snapshot_size_bytes", 0)),
        "abdf_snapshot_hash": str(report.get("abdf_snapshot_hash", "")),
        "expected_hash": str(report.get("expected_hash", "")),
        "expected_hash_match": bool(report.get("expected_hash_match", False)),
        "hash_recomputed_match": bool(report.get("hash_recomputed_match", False)),
        "violations": list(report.get("violations", [])),
        "violations_count": len(report.get("violations", [])),
    }
    write_json(consistency_report_path, consistency_payload)
    return 2


def pass_(
    report_path: Path,
    hash_path: Path,
    identity_report_path: Path,
    consistency_report_path: Path,
    report: dict[str, Any],
    identity_payload: dict[str, Any],
    consistency_payload: dict[str, Any],
) -> int:
    report["verdict"] = "PASS"
    report["violations"] = []
    report["violations_count"] = 0
    write_json(report_path, report)
    write_hash(hash_path, str(report.get("abdf_snapshot_hash", "")))
    write_json(identity_report_path, identity_payload)
    write_json(consistency_report_path, consistency_payload)
    return 0


def main() -> int:
    args = parse_args()

    snapshot_path = Path(args.snapshot_bin)
    hash_path = Path(args.out_hash_txt)
    identity_report_path = Path(args.out_identity_report)
    consistency_report_path = Path(args.out_consistency_report)
    report_path = Path(args.out_report)
    expected_hash_path = Path(args.expected_hash_file) if str(args.expected_hash_file).strip() else None

    report: dict[str, Any] = {
        "gate": "abdf-snapshot-identity",
        "mode": "bootstrap_binary_snapshot_hash",
        "snapshot_bin": str(snapshot_path),
        "out_hash_txt": str(hash_path),
        "expected_hash_file": str(expected_hash_path) if expected_hash_path else "",
        "violations": [],
    }

    if not snapshot_path.is_file():
        report["violations"].append(f"missing_abdf_snapshot_bin:{snapshot_path}")
        return fail(report_path, hash_path, identity_report_path, consistency_report_path, report)

    try:
        snapshot_bytes = snapshot_path.read_bytes()
    except Exception as exc:  # pragma: no cover
        report["violations"].append(
            f"abdf_snapshot_read_error:{snapshot_path}:{type(exc).__name__}"
        )
        return fail(report_path, hash_path, identity_report_path, consistency_report_path, report)

    report["snapshot_size_bytes"] = len(snapshot_bytes)
    if len(snapshot_bytes) == 0:
        report["violations"].append("empty_abdf_snapshot_bin")
        return fail(report_path, hash_path, identity_report_path, consistency_report_path, report)

    computed_hash = sha256_hex(snapshot_bytes)
    recomputed_hash = ""
    try:
        recomputed_hash = sha256_hex(snapshot_path.read_bytes())
    except Exception as exc:  # pragma: no cover
        report["violations"].append(
            f"abdf_snapshot_reread_error:{snapshot_path}:{type(exc).__name__}"
        )
    hash_recomputed_match = bool(recomputed_hash) and computed_hash == recomputed_hash
    report["abdf_snapshot_hash"] = computed_hash
    report["hash_recomputed_match"] = hash_recomputed_match
    if not hash_recomputed_match:
        report["violations"].append("abdf_snapshot_hash_recompute_mismatch")

    expected_hash = ""
    expected_hash_match = False
    if expected_hash_path is not None:
        if not expected_hash_path.is_file():
            report["violations"].append(f"missing_expected_hash_file:{expected_hash_path}")
        else:
            try:
                expected_hash_raw = expected_hash_path.read_text(encoding="utf-8", errors="replace")
            except Exception as exc:  # pragma: no cover
                report["violations"].append(
                    f"expected_hash_read_error:{expected_hash_path}:{type(exc).__name__}"
                )
            else:
                expected_hash = normalize_expected_hash(expected_hash_raw)
                if not expected_hash:
                    report["violations"].append(f"empty_expected_hash_file:{expected_hash_path}")
                elif not is_sha256_hex(expected_hash):
                    report["violations"].append(
                        f"invalid_expected_hash_format:{expected_hash_path}:{expected_hash}"
                    )
                else:
                    expected_hash_match = expected_hash == computed_hash
                    if not expected_hash_match:
                        report["violations"].append(
                            f"abdf_snapshot_hash_mismatch:expected={expected_hash}:actual={computed_hash}"
                        )

    report["expected_hash"] = expected_hash
    report["expected_hash_match"] = expected_hash_match

    identity_payload = {
        "status": "FAIL" if report["violations"] else "PASS",
        "mode": "bootstrap_abdf_snapshot_identity",
        "hash_algorithm": "sha256",
        "canonical_input": "snapshot_binary_bytes",
        "snapshot_bin": str(snapshot_path),
        "snapshot_size_bytes": len(snapshot_bytes),
        "abdf_snapshot_hash": computed_hash,
        "expected_hash_file": str(expected_hash_path) if expected_hash_path else "",
        "expected_hash": expected_hash,
        "expected_hash_match": expected_hash_match,
        "hash_recomputed_match": hash_recomputed_match,
        "violations": list(report["violations"]),
        "violations_count": len(report["violations"]),
    }

    consistency_payload = {
        "status": "FAIL" if report["violations"] else "PASS",
        "mode": "bootstrap_abdf_snapshot_identity",
        "snapshot_size_bytes": len(snapshot_bytes),
        "abdf_snapshot_hash": computed_hash,
        "expected_hash": expected_hash,
        "expected_hash_match": expected_hash_match,
        "hash_recomputed_match": hash_recomputed_match,
        "violations": list(report["violations"]),
        "violations_count": len(report["violations"]),
    }

    if report["violations"]:
        return fail(report_path, hash_path, identity_report_path, consistency_report_path, report)
    return pass_(
        report_path,
        hash_path,
        identity_report_path,
        consistency_report_path,
        report,
        identity_payload,
        consistency_payload,
    )


if __name__ == "__main__":
    raise SystemExit(main())
