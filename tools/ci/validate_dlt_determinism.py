#!/usr/bin/env python3
"""Validate Phase-11 bootstrap DLT determinism (same ETI -> same DLT trace)."""

from __future__ import annotations

# Author: Kenan AY

import argparse
import hashlib
import json
import subprocess
import sys
from pathlib import Path
from typing import Any


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Validate reproducibility of bootstrap DLT materialization from ETI."
    )
    parser.add_argument("--eti-jsonl", required=True, help="eti_transcript.jsonl path")
    parser.add_argument("--out-ltick-trace-a", required=True, help="Output ltick_trace_a.jsonl path")
    parser.add_argument("--out-ltick-trace-b", required=True, help="Output ltick_trace_b.jsonl path")
    parser.add_argument(
        "--out-determinism-report",
        required=True,
        help="Output dlt_determinism_report.json path",
    )
    parser.add_argument("--out-report", required=True, help="Output report.json path")
    return parser.parse_args()


def write_json(path: Path, payload: dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def sha256_hex(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def fail(report_path: Path, determinism_path: Path, report: dict[str, Any]) -> int:
    report["verdict"] = "FAIL"
    report["violations_count"] = len(report.get("violations", []))
    write_json(report_path, report)
    determinism_payload = {
        "status": "FAIL",
        "mode": "bootstrap_reproducibility",
        "hash_a": str(report.get("hash_a", "")),
        "hash_b": str(report.get("hash_b", "")),
        "trace_hash_equal": bool(report.get("trace_hash_equal", False)),
        "run_a_rc": int(report.get("run_a_rc", -1)),
        "run_b_rc": int(report.get("run_b_rc", -1)),
        "violations": list(report.get("violations", [])),
        "violations_count": len(report.get("violations", [])),
    }
    write_json(determinism_path, determinism_payload)
    return 2


def pass_(
    report_path: Path,
    determinism_path: Path,
    report: dict[str, Any],
    determinism_payload: dict[str, Any],
) -> int:
    report["verdict"] = "PASS"
    report["violations"] = []
    report["violations_count"] = 0
    write_json(report_path, report)
    write_json(determinism_path, determinism_payload)
    return 0


def run_materializer(
    eti_jsonl: Path,
    trace_path: Path,
    run_report_path: Path,
    materializer: Path,
) -> tuple[int, dict[str, Any]]:
    trace_path.parent.mkdir(parents=True, exist_ok=True)
    run_report_path.parent.mkdir(parents=True, exist_ok=True)

    proc = subprocess.run(
        [
            sys.executable,
            str(materializer),
            "--eti-jsonl",
            str(eti_jsonl),
            "--out-ltick-trace",
            str(trace_path),
            "--out-report",
            str(run_report_path),
        ],
        check=False,
    )
    if not run_report_path.is_file():
        return proc.returncode, {}
    try:
        payload = json.loads(run_report_path.read_text(encoding="utf-8"))
    except Exception:
        return proc.returncode, {}
    return proc.returncode, payload if isinstance(payload, dict) else {}


def main() -> int:
    args = parse_args()

    eti_jsonl_path = Path(args.eti_jsonl)
    ltick_trace_a_path = Path(args.out_ltick_trace_a)
    ltick_trace_b_path = Path(args.out_ltick_trace_b)
    determinism_report_path = Path(args.out_determinism_report)
    report_path = Path(args.out_report)

    materializer = Path(__file__).with_name("validate_dlt_monotonicity.py")

    report: dict[str, Any] = {
        "gate": "dlt-determinism",
        "mode": "bootstrap_reproducibility",
        "eti_jsonl": str(eti_jsonl_path),
        "ltick_trace_a": str(ltick_trace_a_path),
        "ltick_trace_b": str(ltick_trace_b_path),
        "violations": [],
    }

    if not eti_jsonl_path.is_file():
        report["violations"].append(f"missing_eti_jsonl:{eti_jsonl_path}")
    if not materializer.is_file():
        report["violations"].append(f"missing_materializer:{materializer}")
    if report["violations"]:
        return fail(report_path, determinism_report_path, report)

    run_a_report_path = report_path.parent / "dlt_monotonicity_run_a_report.json"
    run_b_report_path = report_path.parent / "dlt_monotonicity_run_b_report.json"

    run_a_rc, run_a_report = run_materializer(
        eti_jsonl_path, ltick_trace_a_path, run_a_report_path, materializer
    )
    run_b_rc, run_b_report = run_materializer(
        eti_jsonl_path, ltick_trace_b_path, run_b_report_path, materializer
    )

    report["run_a_rc"] = run_a_rc
    report["run_b_rc"] = run_b_rc
    report["run_a_verdict"] = str(run_a_report.get("verdict", "UNKNOWN"))
    report["run_b_verdict"] = str(run_b_report.get("verdict", "UNKNOWN"))

    if run_a_rc != 0:
        report["violations"].append(f"dlt_materialization_failed:run=a:rc={run_a_rc}")
    if run_b_rc != 0:
        report["violations"].append(f"dlt_materialization_failed:run=b:rc={run_b_rc}")

    if not ltick_trace_a_path.is_file():
        report["violations"].append(f"missing_ltick_trace_a:{ltick_trace_a_path}")
    if not ltick_trace_b_path.is_file():
        report["violations"].append(f"missing_ltick_trace_b:{ltick_trace_b_path}")

    hash_a = sha256_hex(ltick_trace_a_path) if ltick_trace_a_path.is_file() else ""
    hash_b = sha256_hex(ltick_trace_b_path) if ltick_trace_b_path.is_file() else ""
    trace_hash_equal = bool(hash_a) and bool(hash_b) and hash_a == hash_b
    report["hash_a"] = hash_a
    report["hash_b"] = hash_b
    report["trace_hash_equal"] = trace_hash_equal

    if hash_a and hash_b and hash_a != hash_b:
        report["violations"].append("ltick_trace_hash_mismatch")

    if report["violations"]:
        return fail(report_path, determinism_report_path, report)

    determinism_payload = {
        "status": "PASS",
        "mode": "bootstrap_reproducibility",
        "hash_a": hash_a,
        "hash_b": hash_b,
        "trace_hash_equal": trace_hash_equal,
        "run_a_rc": run_a_rc,
        "run_b_rc": run_b_rc,
        "run_a_verdict": str(run_a_report.get("verdict", "UNKNOWN")),
        "run_b_verdict": str(run_b_report.get("verdict", "UNKNOWN")),
        "violations": [],
        "violations_count": 0,
    }
    return pass_(report_path, determinism_report_path, report, determinism_payload)


if __name__ == "__main__":
    raise SystemExit(main())
