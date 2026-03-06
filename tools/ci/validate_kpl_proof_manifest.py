#!/usr/bin/env python3
"""Validate Phase-11 bootstrap KPL proof manifest contract."""

from __future__ import annotations

# Author: Kenan AY

import argparse
import hashlib
import json
from pathlib import Path
from typing import Any

MANIFEST_VERSION = 1


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Validate bootstrap KPL proof manifest over identity-locked evidence."
    )
    parser.add_argument("--abdf-hash-file", required=True, help="abdf_snapshot_hash.txt path")
    parser.add_argument("--bcib-plan-hash-file", required=True, help="bcib_plan_hash.txt path")
    parser.add_argument(
        "--execution-trace-hash-file",
        required=True,
        help="execution_trace_hash.txt path",
    )
    parser.add_argument("--replay-report-json", required=True, help="replay_report.json path")
    parser.add_argument("--ledger-jsonl", required=True, help="decision_ledger.jsonl path")
    parser.add_argument("--eti-jsonl", required=True, help="eti_transcript.jsonl path")
    parser.add_argument("--kernel-image-bin", required=True, help="kernel image path")
    parser.add_argument("--config-json", required=True, help="config json path")
    parser.add_argument(
        "--in-proof-manifest-json",
        required=False,
        default="",
        help="Optional existing proof_manifest.json for strict verify mode",
    )
    parser.add_argument(
        "--expected-proof-hash-file",
        required=False,
        default="",
        help="Optional expected proof hash file (first token consumed)",
    )
    parser.add_argument(
        "--expected-final-state-hash-file",
        required=False,
        default="",
        help="Optional expected final state hash file (first token consumed)",
    )
    parser.add_argument("--out-proof-manifest-json", required=True, help="Output proof_manifest.json path")
    parser.add_argument("--out-proof-verify-json", required=True, help="Output proof_verify.json path")
    parser.add_argument("--out-report", required=True, help="Output report.json path")
    return parser.parse_args()


def write_json(path: Path, payload: dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def sha256_hex(payload: bytes) -> str:
    return hashlib.sha256(payload).hexdigest()


def is_sha256_hex(value: str) -> bool:
    if not isinstance(value, str) or len(value) != 64:
        return False
    return all(ch in "0123456789abcdef" for ch in value.lower())


def canonical_json(payload: dict[str, Any]) -> bytes:
    return json.dumps(payload, sort_keys=True, separators=(",", ":")).encode("utf-8")


def normalize_hash_token(raw_text: str) -> str:
    for line in raw_text.splitlines():
        tokenized = line.strip()
        if not tokenized:
            continue
        return tokenized.split()[0].strip().lower()
    return ""


def load_hash_file(path: Path, label: str, report: dict[str, Any]) -> str:
    if not path.is_file():
        report["violations"].append(f"missing_{label}_hash_file:{path}")
        return ""
    try:
        raw = path.read_text(encoding="utf-8", errors="replace")
    except Exception as exc:  # pragma: no cover
        report["violations"].append(f"{label}_hash_read_error:{path}:{type(exc).__name__}")
        return ""

    normalized = normalize_hash_token(raw)
    if not normalized:
        report["violations"].append(f"empty_{label}_hash_file:{path}")
        return ""
    if not is_sha256_hex(normalized):
        report["violations"].append(f"invalid_{label}_hash_format:{path}:{normalized}")
        return ""
    return normalized


def load_json_file(path: Path, label: str, report: dict[str, Any]) -> dict[str, Any]:
    if not path.is_file():
        report["violations"].append(f"missing_{label}:{path}")
        return {}
    try:
        payload = json.loads(path.read_text(encoding="utf-8", errors="replace"))
    except Exception as exc:
        report["violations"].append(f"invalid_{label}_json:{path}:{type(exc).__name__}")
        return {}
    if not isinstance(payload, dict):
        report["violations"].append(f"invalid_{label}_type:{path}:expected_object")
        return {}
    return payload


def required_int(payload: dict[str, Any], key: str, label: str, report: dict[str, Any]) -> int:
    value = payload.get(key)
    if value in (None, ""):
        report["violations"].append(f"missing_{label}_field:{key}")
        return 0
    try:
        return int(value)
    except Exception:
        report["violations"].append(f"invalid_{label}_field_type:{key}")
        return 0


def required_hash(payload: dict[str, Any], key: str, label: str, report: dict[str, Any]) -> str:
    value = str(payload.get(key, "") or "").lower()
    if not value:
        report["violations"].append(f"missing_{label}_field:{key}")
        return ""
    if not is_sha256_hex(value):
        report["violations"].append(f"invalid_{label}_field_hash:{key}:{value}")
        return ""
    return value


def manifest_without_proof_hash(payload: dict[str, Any]) -> dict[str, Any]:
    stripped = dict(payload)
    stripped.pop("proof_hash", None)
    return stripped


def compute_proof_hash(payload: dict[str, Any]) -> str:
    return sha256_hex(canonical_json(manifest_without_proof_hash(payload)))


def fail(
    report_path: Path,
    proof_manifest_path: Path,
    proof_verify_path: Path,
    report: dict[str, Any],
    proof_manifest: dict[str, Any],
) -> int:
    report["verdict"] = "FAIL"
    report["violations_count"] = len(report.get("violations", []))
    write_json(report_path, report)

    manifest_payload = dict(proof_manifest)
    if not manifest_payload:
        manifest_payload = {
            "manifest_version": MANIFEST_VERSION,
            "mode": "bootstrap_kpl_proof_manifest",
            "status": "FAIL",
            "proof_hash": "",
            "violations": list(report.get("violations", [])),
            "violations_count": len(report.get("violations", [])),
        }
    write_json(proof_manifest_path, manifest_payload)

    verify_payload = {
        "status": "FAIL",
        "mode": "bootstrap_kpl_proof_manifest",
        "manifest_version": int(manifest_payload.get("manifest_version", MANIFEST_VERSION)),
        "proof_hash": str(manifest_payload.get("proof_hash", "")),
        "proof_hash_recomputed": str(report.get("proof_hash_recomputed", "")),
        "proof_hash_match": bool(report.get("proof_hash_match", False)),
        "replay_result_hash_match": bool(report.get("replay_result_hash_match", False)),
        "final_state_hash_match": bool(report.get("final_state_hash_match", False)),
        "event_count_match": bool(report.get("event_count_match", False)),
        "violation_count_match": bool(report.get("violation_count_match", False)),
        "expected_proof_hash_match": bool(report.get("expected_proof_hash_match", False)),
        "expected_final_state_hash_match": bool(report.get("expected_final_state_hash_match", False)),
        "signature_mode": str(manifest_payload.get("signature_mode", "")),
        "signer_sig": str(manifest_payload.get("signer_sig", "")),
        "violations": list(report.get("violations", [])),
        "violations_count": len(report.get("violations", [])),
    }
    write_json(proof_verify_path, verify_payload)
    return 2


def pass_(
    report_path: Path,
    proof_manifest_path: Path,
    proof_verify_path: Path,
    report: dict[str, Any],
    proof_manifest: dict[str, Any],
    proof_verify: dict[str, Any],
) -> int:
    report["verdict"] = "PASS"
    report["violations"] = []
    report["violations_count"] = 0
    write_json(report_path, report)
    write_json(proof_manifest_path, proof_manifest)
    write_json(proof_verify_path, proof_verify)
    return 0


def main() -> int:
    args = parse_args()

    abdf_hash_path = Path(args.abdf_hash_file)
    bcib_plan_hash_path = Path(args.bcib_plan_hash_file)
    execution_trace_hash_path = Path(args.execution_trace_hash_file)
    replay_report_path = Path(args.replay_report_json)
    ledger_jsonl_path = Path(args.ledger_jsonl)
    eti_jsonl_path = Path(args.eti_jsonl)
    kernel_image_path = Path(args.kernel_image_bin)
    config_json_path = Path(args.config_json)
    input_manifest_path = Path(args.in_proof_manifest_json) if str(args.in_proof_manifest_json).strip() else None
    expected_proof_hash_path = (
        Path(args.expected_proof_hash_file) if str(args.expected_proof_hash_file).strip() else None
    )
    expected_final_state_hash_path = (
        Path(args.expected_final_state_hash_file)
        if str(args.expected_final_state_hash_file).strip()
        else None
    )
    out_proof_manifest_path = Path(args.out_proof_manifest_json)
    out_proof_verify_path = Path(args.out_proof_verify_json)
    out_report_path = Path(args.out_report)

    report: dict[str, Any] = {
        "gate": "kpl-proof-verify",
        "mode": "bootstrap_kpl_proof_manifest",
        "abdf_hash_file": str(abdf_hash_path),
        "bcib_plan_hash_file": str(bcib_plan_hash_path),
        "execution_trace_hash_file": str(execution_trace_hash_path),
        "replay_report_json": str(replay_report_path),
        "ledger_jsonl": str(ledger_jsonl_path),
        "eti_jsonl": str(eti_jsonl_path),
        "kernel_image_bin": str(kernel_image_path),
        "config_json": str(config_json_path),
        "in_proof_manifest_json": str(input_manifest_path) if input_manifest_path else "",
        "expected_proof_hash_file": str(expected_proof_hash_path) if expected_proof_hash_path else "",
        "expected_final_state_hash_file": str(expected_final_state_hash_path)
        if expected_final_state_hash_path
        else "",
        "violations": [],
    }

    abdf_hash = load_hash_file(abdf_hash_path, "abdf_snapshot", report)
    bcib_plan_hash = load_hash_file(bcib_plan_hash_path, "bcib_plan", report)
    execution_trace_hash = load_hash_file(execution_trace_hash_path, "execution_trace", report)

    replay_report = load_json_file(replay_report_path, "replay_report", report)
    replay_result_hash = required_hash(replay_report, "replay_result_hash", "replay_report", report)
    final_state_hash = required_hash(replay_report, "final_state_hash", "replay_report", report)
    replay_event_count = required_int(replay_report, "replay_event_count", "replay_report", report)
    replay_violations_count = required_int(replay_report, "violations_count", "replay_report", report)

    if not ledger_jsonl_path.is_file():
        report["violations"].append(f"missing_ledger_jsonl:{ledger_jsonl_path}")
        ledger_root_hash = ""
    else:
        try:
            ledger_bytes = ledger_jsonl_path.read_bytes()
            if len(ledger_bytes) == 0:
                report["violations"].append("empty_ledger_jsonl")
                ledger_root_hash = ""
            else:
                ledger_root_hash = sha256_hex(ledger_bytes)
        except Exception as exc:  # pragma: no cover
            report["violations"].append(
                f"ledger_jsonl_read_error:{ledger_jsonl_path}:{type(exc).__name__}"
            )
            ledger_root_hash = ""

    if not eti_jsonl_path.is_file():
        report["violations"].append(f"missing_eti_jsonl:{eti_jsonl_path}")
        transcript_root_hash = ""
    else:
        try:
            eti_bytes = eti_jsonl_path.read_bytes()
            if len(eti_bytes) == 0:
                report["violations"].append("empty_eti_jsonl")
                transcript_root_hash = ""
            else:
                transcript_root_hash = sha256_hex(eti_bytes)
        except Exception as exc:  # pragma: no cover
            report["violations"].append(
                f"eti_jsonl_read_error:{eti_jsonl_path}:{type(exc).__name__}"
            )
            transcript_root_hash = ""

    if not kernel_image_path.is_file():
        report["violations"].append(f"missing_kernel_image_bin:{kernel_image_path}")
        kernel_image_hash = ""
    else:
        try:
            kernel_bytes = kernel_image_path.read_bytes()
            if len(kernel_bytes) == 0:
                report["violations"].append("empty_kernel_image_bin")
                kernel_image_hash = ""
            else:
                kernel_image_hash = sha256_hex(kernel_bytes)
        except Exception as exc:  # pragma: no cover
            report["violations"].append(
                f"kernel_image_read_error:{kernel_image_path}:{type(exc).__name__}"
            )
            kernel_image_hash = ""

    if not config_json_path.is_file():
        report["violations"].append(f"missing_config_json:{config_json_path}")
        config_hash = ""
    else:
        try:
            config_bytes = config_json_path.read_bytes()
            if len(config_bytes) == 0:
                report["violations"].append("empty_config_json")
                config_hash = ""
            else:
                config_hash = sha256_hex(config_bytes)
        except Exception as exc:  # pragma: no cover
            report["violations"].append(
                f"config_json_read_error:{config_json_path}:{type(exc).__name__}"
            )
            config_hash = ""

    generated_manifest: dict[str, Any] = {
        "manifest_version": MANIFEST_VERSION,
        "mode": "bootstrap_kpl_proof_manifest",
        "signature_mode": "bootstrap-none",
        "signer_sig": "",
        "hash_algorithm": "sha256",
        "kernel_image_hash": kernel_image_hash,
        "config_hash": config_hash,
        "ledger_root_hash": ledger_root_hash,
        "transcript_root_hash": transcript_root_hash,
        "abdf_snapshot_hash": abdf_hash,
        "bcib_plan_hash": bcib_plan_hash,
        "execution_trace_hash": execution_trace_hash,
        "replay_result_hash": replay_result_hash,
        "final_state_hash": final_state_hash,
        "event_count": replay_event_count,
        "violation_count": replay_violations_count,
    }
    generated_manifest["proof_hash"] = compute_proof_hash(generated_manifest)

    manifest = generated_manifest
    manifest_source = "generated"
    if input_manifest_path is not None:
        manifest_source = "input"
        manifest = load_json_file(input_manifest_path, "proof_manifest", report)
        if not manifest:
            return fail(out_report_path, out_proof_manifest_path, out_proof_verify_path, report, {})

    required_hash_fields = (
        "kernel_image_hash",
        "config_hash",
        "ledger_root_hash",
        "transcript_root_hash",
        "abdf_snapshot_hash",
        "bcib_plan_hash",
        "execution_trace_hash",
        "replay_result_hash",
        "final_state_hash",
        "proof_hash",
    )
    for field in required_hash_fields:
        _ = required_hash(manifest, field, "proof_manifest", report)

    manifest_version = required_int(manifest, "manifest_version", "proof_manifest", report)
    if manifest_version != MANIFEST_VERSION:
        report["violations"].append(
            f"unsupported_manifest_version:expected={MANIFEST_VERSION}:actual={manifest_version}"
        )

    event_count = required_int(manifest, "event_count", "proof_manifest", report)
    violation_count = required_int(manifest, "violation_count", "proof_manifest", report)

    signature_mode = str(manifest.get("signature_mode", "") or "")
    if not signature_mode:
        report["violations"].append("missing_proof_manifest_field:signature_mode")

    proof_hash_recomputed = compute_proof_hash(manifest)
    proof_hash_value = str(manifest.get("proof_hash", "") or "").lower()
    proof_hash_match = bool(proof_hash_value) and proof_hash_value == proof_hash_recomputed
    if not proof_hash_match:
        report["violations"].append(
            f"proof_hash_mismatch:expected={proof_hash_recomputed}:actual={proof_hash_value}"
        )

    replay_result_hash_match = str(manifest.get("replay_result_hash", "")).lower() == replay_result_hash
    if not replay_result_hash_match:
        report["violations"].append("replay_result_hash_binding_mismatch")

    final_state_hash_match = str(manifest.get("final_state_hash", "")).lower() == final_state_hash
    if not final_state_hash_match:
        report["violations"].append("final_state_hash_binding_mismatch")

    event_count_match = event_count == replay_event_count
    if not event_count_match:
        report["violations"].append(
            f"event_count_binding_mismatch:expected={replay_event_count}:actual={event_count}"
        )

    violation_count_match = violation_count == replay_violations_count
    if not violation_count_match:
        report["violations"].append(
            "violation_count_binding_mismatch:"
            f"expected={replay_violations_count}:actual={violation_count}"
        )

    expected_proof_hash = ""
    expected_proof_hash_match = False
    if expected_proof_hash_path is not None:
        expected_proof_hash = load_hash_file(expected_proof_hash_path, "expected_proof", report)
        if expected_proof_hash:
            expected_proof_hash_match = expected_proof_hash == proof_hash_value
            if not expected_proof_hash_match:
                report["violations"].append(
                    "expected_proof_hash_mismatch:"
                    f"expected={expected_proof_hash}:actual={proof_hash_value}"
                )

    expected_final_state_hash = ""
    expected_final_state_hash_match = False
    if expected_final_state_hash_path is not None:
        expected_final_state_hash = load_hash_file(
            expected_final_state_hash_path, "expected_final_state", report
        )
        if expected_final_state_hash:
            expected_final_state_hash_match = expected_final_state_hash == final_state_hash
            if not expected_final_state_hash_match:
                report["violations"].append(
                    "expected_final_state_hash_mismatch:"
                    f"expected={expected_final_state_hash}:actual={final_state_hash}"
                )

    report["manifest_source"] = manifest_source
    report["proof_hash_recomputed"] = proof_hash_recomputed
    report["proof_hash_match"] = proof_hash_match
    report["replay_result_hash_match"] = replay_result_hash_match
    report["final_state_hash_match"] = final_state_hash_match
    report["event_count_match"] = event_count_match
    report["violation_count_match"] = violation_count_match
    report["expected_proof_hash"] = expected_proof_hash
    report["expected_proof_hash_match"] = expected_proof_hash_match
    report["expected_final_state_hash"] = expected_final_state_hash
    report["expected_final_state_hash_match"] = expected_final_state_hash_match

    report["proof_manifest_hash"] = proof_hash_value
    report["replay_result_hash"] = replay_result_hash
    report["final_state_hash"] = final_state_hash

    proof_verify = {
        "status": "FAIL" if report["violations"] else "PASS",
        "mode": "bootstrap_kpl_proof_manifest",
        "manifest_version": manifest_version,
        "manifest_source": manifest_source,
        "proof_hash": proof_hash_value,
        "proof_hash_recomputed": proof_hash_recomputed,
        "proof_hash_match": proof_hash_match,
        "replay_result_hash_match": replay_result_hash_match,
        "final_state_hash_match": final_state_hash_match,
        "event_count_match": event_count_match,
        "violation_count_match": violation_count_match,
        "expected_proof_hash": expected_proof_hash,
        "expected_proof_hash_match": expected_proof_hash_match,
        "expected_final_state_hash": expected_final_state_hash,
        "expected_final_state_hash_match": expected_final_state_hash_match,
        "signature_mode": signature_mode,
        "signer_sig": str(manifest.get("signer_sig", "")),
        "violations": list(report["violations"]),
        "violations_count": len(report["violations"]),
    }

    if report["violations"]:
        return fail(out_report_path, out_proof_manifest_path, out_proof_verify_path, report, manifest)

    return pass_(
        out_report_path,
        out_proof_manifest_path,
        out_proof_verify_path,
        report,
        manifest,
        proof_verify,
    )


if __name__ == "__main__":
    raise SystemExit(main())
