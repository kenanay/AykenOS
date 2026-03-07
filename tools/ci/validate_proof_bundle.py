#!/usr/bin/env python3
"""Generate and validate Phase-11 bootstrap proof bundles."""

from __future__ import annotations

# Author: Kenan AY

import argparse
import hashlib
import json
import shutil
from pathlib import Path
from typing import Any

BUNDLE_VERSION = 1
KPL_MANIFEST_VERSION = 1
REQUIRED_BUNDLE_FILES = (
    "evidence/abdf_snapshot_hash.txt",
    "evidence/bcib_plan_hash.txt",
    "evidence/execution_trace_hash.txt",
    "evidence/replay_trace_hash.txt",
    "evidence/decision_ledger.jsonl",
    "evidence/eti_transcript.jsonl",
    "evidence/kernel.elf",
    "traces/execution_trace.jsonl",
    "traces/replay_trace.jsonl",
    "reports/proof_manifest.json",
    "reports/proof_verify.json",
    "reports/report.json",
    "reports/replay_report.json",
    "reports/summary.json",
    "meta/run.json",
)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Generate or validate bootstrap proof bundle portability artifacts."
    )
    subparsers = parser.add_subparsers(dest="mode", required=True)

    gen = subparsers.add_parser("generate", help="Generate a portable proof bundle directory")
    gen.add_argument("--bundle-root", required=True, help="Output proof bundle root directory")
    gen.add_argument("--abdf-evidence", required=True, help="ABDF identity evidence directory")
    gen.add_argument(
        "--execution-evidence", required=True, help="Execution identity evidence directory"
    )
    gen.add_argument("--replay-evidence", required=True, help="Replay determinism evidence directory")
    gen.add_argument("--kpl-evidence", required=True, help="KPL proof evidence directory")
    gen.add_argument("--ledger-evidence", required=True, help="Ledger evidence directory")
    gen.add_argument("--eti-evidence", required=True, help="ETI evidence directory")
    gen.add_argument("--kernel-image-bin", required=True, help="Kernel image binary path")
    gen.add_argument("--summary-json", required=True, help="Source summary.json path")
    gen.add_argument("--meta-run-json", required=True, help="Source meta/run.json path")

    verify = subparsers.add_parser("verify", help="Verify an existing portable proof bundle")
    verify.add_argument("--bundle-root", required=True, help="Input proof bundle root directory")
    verify.add_argument(
        "--out-bundle-verify-json",
        required=True,
        help="Output bundle_verify.json path",
    )
    verify.add_argument("--out-report", required=True, help="Output report.json path")
    return parser.parse_args()


def write_json(path: Path, payload: dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def sha256_hex(payload: bytes) -> str:
    return hashlib.sha256(payload).hexdigest()


def canonical_json(payload: dict[str, Any]) -> bytes:
    return json.dumps(payload, sort_keys=True, separators=(",", ":")).encode("utf-8")


def is_sha256_hex(value: str) -> bool:
    if not isinstance(value, str) or len(value) != 64:
        return False
    return all(ch in "0123456789abcdef" for ch in value.lower())


def normalize_hash_text(raw_text: str) -> str:
    for line in raw_text.splitlines():
        token = line.strip()
        if not token:
            continue
        return token.split()[0].strip().lower()
    return ""


def load_json_file(path: Path, label: str, violations: list[str]) -> dict[str, Any]:
    if not path.is_file():
        violations.append(f"missing_{label}:{path}")
        return {}
    try:
        payload = json.loads(path.read_text(encoding="utf-8", errors="replace"))
    except Exception as exc:
        violations.append(f"invalid_{label}_json:{path}:{type(exc).__name__}")
        return {}
    if not isinstance(payload, dict):
        violations.append(f"invalid_{label}_type:{path}:expected_object")
        return {}
    return payload


def read_hash_text(path: Path, label: str, violations: list[str]) -> str:
    if not path.is_file():
        violations.append(f"missing_{label}:{path}")
        return ""
    try:
        normalized = normalize_hash_text(path.read_text(encoding="utf-8", errors="replace"))
    except Exception as exc:
        violations.append(f"{label}_read_error:{path}:{type(exc).__name__}")
        return ""
    if not normalized:
        violations.append(f"empty_{label}:{path}")
        return ""
    if not is_sha256_hex(normalized):
        violations.append(f"invalid_{label}_format:{path}:{normalized}")
        return ""
    return normalized


def required_hash(payload: dict[str, Any], key: str, label: str, violations: list[str]) -> str:
    value = str(payload.get(key, "") or "").lower()
    if not value:
        violations.append(f"missing_{label}_field:{key}")
        return ""
    if not is_sha256_hex(value):
        violations.append(f"invalid_{label}_field_hash:{key}:{value}")
        return ""
    return value


def required_int(payload: dict[str, Any], key: str, label: str, violations: list[str]) -> int:
    value = payload.get(key)
    if value in (None, ""):
        violations.append(f"missing_{label}_field:{key}")
        return 0
    try:
        return int(value)
    except Exception:
        violations.append(f"invalid_{label}_field_type:{key}")
        return 0


def manifest_without_bundle_id(payload: dict[str, Any]) -> dict[str, Any]:
    stripped = dict(payload)
    stripped.pop("bundle_id", None)
    return stripped


def compute_bundle_id(bundle_manifest: dict[str, Any], checksums_payload: dict[str, Any]) -> str:
    material = canonical_json(manifest_without_bundle_id(bundle_manifest)) + canonical_json(
        checksums_payload
    )
    return sha256_hex(material)


def compute_kpl_proof_hash(proof_manifest: dict[str, Any]) -> str:
    stripped = dict(proof_manifest)
    stripped.pop("proof_hash", None)
    return sha256_hex(canonical_json(stripped))


def copy_required_file(src: Path, dst: Path, label: str) -> None:
    dst.parent.mkdir(parents=True, exist_ok=True)
    if not src.is_file():
        raise FileNotFoundError(f"missing_{label}:{src}")
    shutil.copy2(src, dst)


def bundle_source_map(args: argparse.Namespace, bundle_root: Path) -> dict[str, Path]:
    return {
        "evidence/abdf_snapshot_hash.txt": Path(args.abdf_evidence) / "abdf_snapshot_hash.txt",
        "evidence/bcib_plan_hash.txt": Path(args.execution_evidence) / "bcib_plan_hash.txt",
        "evidence/execution_trace_hash.txt": Path(args.execution_evidence)
        / "execution_trace_hash.txt",
        "evidence/replay_trace_hash.txt": Path(args.replay_evidence) / "replay_trace_hash.txt",
        "evidence/decision_ledger.jsonl": Path(args.ledger_evidence) / "decision_ledger.jsonl",
        "evidence/eti_transcript.jsonl": Path(args.eti_evidence) / "eti_transcript.jsonl",
        "evidence/kernel.elf": Path(args.kernel_image_bin),
        "traces/execution_trace.jsonl": Path(args.execution_evidence) / "execution_trace.jsonl",
        "traces/replay_trace.jsonl": Path(args.replay_evidence) / "replay_trace.jsonl",
        "reports/proof_manifest.json": Path(args.kpl_evidence) / "proof_manifest.json",
        "reports/proof_verify.json": Path(args.kpl_evidence) / "proof_verify.json",
        "reports/report.json": Path(args.kpl_evidence) / "report.json",
        "reports/replay_report.json": Path(args.replay_evidence) / "replay_report.json",
        "reports/summary.json": Path(args.summary_json),
        "meta/run.json": Path(args.meta_run_json),
    }


def generate_bundle(args: argparse.Namespace) -> int:
    bundle_root = Path(args.bundle_root)
    if bundle_root.exists():
        shutil.rmtree(bundle_root)
    bundle_root.mkdir(parents=True, exist_ok=True)

    source_map = bundle_source_map(args, bundle_root)
    for rel_path, src in source_map.items():
        copy_required_file(src, bundle_root / rel_path, rel_path.replace("/", "_"))

    violations: list[str] = []
    proof_manifest = load_json_file(
        bundle_root / "reports/proof_manifest.json", "bundle_proof_manifest", violations
    )
    proof_verify = load_json_file(
        bundle_root / "reports/proof_verify.json", "bundle_proof_verify", violations
    )
    report_json = load_json_file(
        bundle_root / "reports/report.json", "bundle_kpl_report", violations
    )
    summary_json = load_json_file(
        bundle_root / "reports/summary.json", "bundle_summary_report", violations
    )

    if violations:
        raise RuntimeError(";".join(violations))

    source_report_verdict = str(report_json.get("verdict", "") or "")
    source_proof_verify_status = str(proof_verify.get("status", "") or "")
    source_summary_verdict = str(summary_json.get("verdict", "") or "")
    source_proof_hash = str(proof_manifest.get("proof_hash", "") or "").lower()
    source_final_state_hash = str(proof_manifest.get("final_state_hash", "") or "").lower()

    checksums_payload = {
        "bundle_version": BUNDLE_VERSION,
        "algorithm": "sha256",
        "files": {
            rel_path: sha256_hex((bundle_root / rel_path).read_bytes())
            for rel_path in REQUIRED_BUNDLE_FILES
        },
    }
    write_json(bundle_root / "checksums.json", checksums_payload)

    bundle_manifest = {
        "bundle_version": BUNDLE_VERSION,
        "mode": "bootstrap_proof_bundle",
        "checksums_file": "checksums.json",
        "source_report_verdict": source_report_verdict,
        "source_proof_verify_status": source_proof_verify_status,
        "source_summary_verdict": source_summary_verdict,
        "source_proof_hash": source_proof_hash,
        "source_final_state_hash": source_final_state_hash,
        "required_files": list(REQUIRED_BUNDLE_FILES),
        "bundle_id": "",
    }
    bundle_manifest["bundle_id"] = compute_bundle_id(bundle_manifest, checksums_payload)
    write_json(bundle_root / "manifest.json", bundle_manifest)
    return 0


def verify_bundle_schema(
    bundle_root: Path, report: dict[str, Any]
) -> tuple[dict[str, Any], dict[str, Any], list[str]]:
    violations = report["violations"]
    manifest_path = bundle_root / "manifest.json"
    checksums_path = bundle_root / "checksums.json"

    bundle_manifest = load_json_file(manifest_path, "bundle_manifest", violations)
    checksums_payload = load_json_file(checksums_path, "bundle_checksums", violations)
    if not bundle_manifest or not checksums_payload:
        return bundle_manifest, checksums_payload, violations

    bundle_version = required_int(bundle_manifest, "bundle_version", "bundle_manifest", violations)
    if bundle_version != BUNDLE_VERSION:
        violations.append(
            f"unsupported_bundle_version:expected={BUNDLE_VERSION}:actual={bundle_version}"
        )

    checksums_file = str(bundle_manifest.get("checksums_file", "") or "")
    if checksums_file != "checksums.json":
        violations.append(f"invalid_checksums_file_reference:{checksums_file}")

    required_files = bundle_manifest.get("required_files")
    if not isinstance(required_files, list):
        violations.append("invalid_bundle_manifest_field_type:required_files")
        required_files = []
    required_file_set = {str(item) for item in required_files}
    expected_file_set = set(REQUIRED_BUNDLE_FILES)
    if required_file_set != expected_file_set:
        violations.append("bundle_required_files_mismatch")

    bundle_id = str(bundle_manifest.get("bundle_id", "") or "").lower()
    if not is_sha256_hex(bundle_id):
        violations.append(f"invalid_bundle_manifest_field_hash:bundle_id:{bundle_id}")
    recomputed_bundle_id = compute_bundle_id(bundle_manifest, checksums_payload)
    if bundle_id and bundle_id != recomputed_bundle_id:
        violations.append(
            f"bundle_id_mismatch:expected={recomputed_bundle_id}:actual={bundle_id}"
        )

    source_proof_hash = required_hash(
        bundle_manifest, "source_proof_hash", "bundle_manifest", violations
    )
    source_final_state_hash = required_hash(
        bundle_manifest, "source_final_state_hash", "bundle_manifest", violations
    )
    report["bundle_id_recomputed"] = recomputed_bundle_id
    report["source_proof_hash"] = source_proof_hash
    report["source_final_state_hash"] = source_final_state_hash

    checksums_version = required_int(
        checksums_payload, "bundle_version", "bundle_checksums", violations
    )
    if checksums_version != BUNDLE_VERSION:
        violations.append(
            f"unsupported_checksums_version:expected={BUNDLE_VERSION}:actual={checksums_version}"
        )
    algorithm = str(checksums_payload.get("algorithm", "") or "")
    if algorithm != "sha256":
        violations.append(f"unsupported_checksums_algorithm:{algorithm}")

    files_map = checksums_payload.get("files")
    if not isinstance(files_map, dict):
        violations.append("invalid_bundle_checksums_field_type:files")
        files_map = {}
    if set(files_map.keys()) != expected_file_set:
        violations.append("bundle_checksums_files_mismatch")
    report["checksums_entry_count"] = len(files_map)

    for rel_path in REQUIRED_BUNDLE_FILES:
        expected_hash = str(files_map.get(rel_path, "") or "").lower()
        if not expected_hash:
            violations.append(f"missing_bundle_checksum_entry:{rel_path}")
            continue
        if not is_sha256_hex(expected_hash):
            violations.append(f"invalid_bundle_checksum_hash:{rel_path}:{expected_hash}")
            continue
        file_path = bundle_root / rel_path
        if not file_path.is_file():
            violations.append(f"missing_bundle_required_file:{rel_path}")
            continue
        actual_hash = sha256_hex(file_path.read_bytes())
        if actual_hash != expected_hash:
            violations.append(
                f"bundle_checksum_mismatch:{rel_path}:expected={expected_hash}:actual={actual_hash}"
            )

    return bundle_manifest, checksums_payload, violations


def reproduce_kpl_verdict(bundle_root: Path, report: dict[str, Any]) -> tuple[str, str]:
    violations = report["violations"]

    abdf_hash = read_hash_text(
        bundle_root / "evidence/abdf_snapshot_hash.txt", "bundle_abdf_snapshot_hash", violations
    )
    bcib_plan_hash = read_hash_text(
        bundle_root / "evidence/bcib_plan_hash.txt", "bundle_bcib_plan_hash", violations
    )
    execution_trace_hash = read_hash_text(
        bundle_root / "evidence/execution_trace_hash.txt",
        "bundle_execution_trace_hash",
        violations,
    )
    replay_trace_hash = read_hash_text(
        bundle_root / "evidence/replay_trace_hash.txt", "bundle_replay_trace_hash", violations
    )
    proof_manifest = load_json_file(
        bundle_root / "reports/proof_manifest.json", "bundle_proof_manifest", violations
    )
    proof_verify = load_json_file(
        bundle_root / "reports/proof_verify.json", "bundle_proof_verify", violations
    )
    report_json = load_json_file(
        bundle_root / "reports/report.json", "bundle_kpl_report", violations
    )
    replay_report = load_json_file(
        bundle_root / "reports/replay_report.json", "bundle_replay_report", violations
    )
    summary_json = load_json_file(
        bundle_root / "reports/summary.json", "bundle_summary", violations
    )

    ledger_path = bundle_root / "evidence/decision_ledger.jsonl"
    eti_path = bundle_root / "evidence/eti_transcript.jsonl"
    kernel_path = bundle_root / "evidence/kernel.elf"
    config_path = bundle_root / "meta/run.json"
    execution_trace_path = bundle_root / "traces/execution_trace.jsonl"
    replay_trace_path = bundle_root / "traces/replay_trace.jsonl"

    ledger_root_hash = sha256_hex(ledger_path.read_bytes()) if ledger_path.is_file() else ""
    transcript_root_hash = sha256_hex(eti_path.read_bytes()) if eti_path.is_file() else ""
    kernel_image_hash = sha256_hex(kernel_path.read_bytes()) if kernel_path.is_file() else ""
    config_hash = sha256_hex(config_path.read_bytes()) if config_path.is_file() else ""
    execution_trace_hash_recomputed = (
        sha256_hex(execution_trace_path.read_bytes()) if execution_trace_path.is_file() else ""
    )
    replay_trace_hash_recomputed = (
        sha256_hex(replay_trace_path.read_bytes()) if replay_trace_path.is_file() else ""
    )
    if not ledger_root_hash:
        violations.append("bundle_ledger_root_hash_missing")
    if not transcript_root_hash:
        violations.append("bundle_transcript_root_hash_missing")
    if not kernel_image_hash:
        violations.append("bundle_kernel_image_hash_missing")
    if not config_hash:
        violations.append("bundle_config_hash_missing")
    if not execution_trace_hash_recomputed:
        violations.append("bundle_execution_trace_missing")
    if not replay_trace_hash_recomputed:
        violations.append("bundle_replay_trace_missing")

    proof_manifest_version = required_int(
        proof_manifest, "manifest_version", "proof_manifest", violations
    )
    if proof_manifest_version != KPL_MANIFEST_VERSION:
        violations.append(
            "unsupported_proof_manifest_version:"
            f"expected={KPL_MANIFEST_VERSION}:actual={proof_manifest_version}"
        )

    manifest_proof_hash = required_hash(proof_manifest, "proof_hash", "proof_manifest", violations)
    manifest_kernel_hash = required_hash(
        proof_manifest, "kernel_image_hash", "proof_manifest", violations
    )
    manifest_config_hash = required_hash(
        proof_manifest, "config_hash", "proof_manifest", violations
    )
    manifest_ledger_hash = required_hash(
        proof_manifest, "ledger_root_hash", "proof_manifest", violations
    )
    manifest_transcript_hash = required_hash(
        proof_manifest, "transcript_root_hash", "proof_manifest", violations
    )
    manifest_abdf_hash = required_hash(
        proof_manifest, "abdf_snapshot_hash", "proof_manifest", violations
    )
    manifest_bcib_hash = required_hash(
        proof_manifest, "bcib_plan_hash", "proof_manifest", violations
    )
    manifest_execution_trace_hash = required_hash(
        proof_manifest, "execution_trace_hash", "proof_manifest", violations
    )
    manifest_replay_result_hash = required_hash(
        proof_manifest, "replay_result_hash", "proof_manifest", violations
    )
    manifest_final_state_hash = required_hash(
        proof_manifest, "final_state_hash", "proof_manifest", violations
    )
    manifest_event_count = required_int(
        proof_manifest, "event_count", "proof_manifest", violations
    )
    manifest_violation_count = required_int(
        proof_manifest, "violation_count", "proof_manifest", violations
    )

    replay_result_hash = required_hash(
        replay_report, "replay_result_hash", "replay_report", violations
    )
    final_state_hash = required_hash(
        replay_report, "final_state_hash", "replay_report", violations
    )
    replay_report_trace_hash = required_hash(
        replay_report, "replay_execution_trace_hash", "replay_report", violations
    )
    replay_event_count = required_int(
        replay_report, "replay_event_count", "replay_report", violations
    )
    replay_violations_count = required_int(
        replay_report, "violations_count", "replay_report", violations
    )

    signature_mode = str(proof_manifest.get("signature_mode", "") or "")
    if not signature_mode:
        violations.append("missing_proof_manifest_field:signature_mode")
    recomputed_proof_hash = compute_kpl_proof_hash(proof_manifest)
    if manifest_proof_hash and manifest_proof_hash != recomputed_proof_hash:
        violations.append(
            f"bundle_proof_hash_mismatch:expected={recomputed_proof_hash}:actual={manifest_proof_hash}"
        )

    if manifest_kernel_hash and manifest_kernel_hash != kernel_image_hash:
        violations.append("bundle_kernel_image_hash_binding_mismatch")
    if manifest_config_hash and manifest_config_hash != config_hash:
        violations.append("bundle_config_hash_binding_mismatch")
    if manifest_ledger_hash and manifest_ledger_hash != ledger_root_hash:
        violations.append("bundle_ledger_root_hash_binding_mismatch")
    if manifest_transcript_hash and manifest_transcript_hash != transcript_root_hash:
        violations.append("bundle_transcript_root_hash_binding_mismatch")
    if manifest_abdf_hash and manifest_abdf_hash != abdf_hash:
        violations.append("bundle_abdf_snapshot_hash_binding_mismatch")
    if manifest_bcib_hash and manifest_bcib_hash != bcib_plan_hash:
        violations.append("bundle_bcib_plan_hash_binding_mismatch")
    if manifest_execution_trace_hash and manifest_execution_trace_hash != execution_trace_hash:
        violations.append("bundle_execution_trace_hash_binding_mismatch")
    if execution_trace_hash and execution_trace_hash_recomputed:
        if execution_trace_hash != execution_trace_hash_recomputed:
            violations.append(
                "bundle_execution_trace_hash_parity_mismatch:"
                f"expected={execution_trace_hash_recomputed}:actual={execution_trace_hash}"
            )
    if replay_trace_hash and replay_trace_hash_recomputed:
        if replay_trace_hash != replay_trace_hash_recomputed:
            violations.append(
                "bundle_replay_trace_hash_parity_mismatch:"
                f"expected={replay_trace_hash_recomputed}:actual={replay_trace_hash}"
            )
    if replay_report_trace_hash and replay_trace_hash:
        if replay_report_trace_hash != replay_trace_hash:
            violations.append("bundle_replay_report_trace_hash_binding_mismatch")
    if manifest_replay_result_hash and manifest_replay_result_hash != replay_result_hash:
        violations.append("bundle_replay_result_hash_binding_mismatch")
    if manifest_final_state_hash and manifest_final_state_hash != final_state_hash:
        violations.append("bundle_final_state_hash_binding_mismatch")
    if manifest_event_count != replay_event_count:
        violations.append(
            "bundle_event_count_binding_mismatch:"
            f"expected={replay_event_count}:actual={manifest_event_count}"
        )
    if manifest_violation_count != replay_violations_count:
        violations.append(
            "bundle_violation_count_binding_mismatch:"
            f"expected={replay_violations_count}:actual={manifest_violation_count}"
        )

    source_report_verdict = str(report_json.get("verdict", "") or "")
    source_proof_verify_status = str(proof_verify.get("status", "") or "")
    source_summary_verdict = str(summary_json.get("verdict", "") or "")
    if not source_report_verdict:
        violations.append("missing_bundle_kpl_report_field:verdict")
    if not source_proof_verify_status:
        violations.append("missing_bundle_proof_verify_field:status")
    if not source_summary_verdict:
        violations.append("missing_bundle_summary_field:verdict")

    report["recomputed_proof_hash"] = recomputed_proof_hash
    report["source_report_verdict"] = source_report_verdict
    report["source_proof_verify_status"] = source_proof_verify_status
    report["source_summary_verdict"] = source_summary_verdict
    report["bundle_execution_trace_hash_recomputed"] = execution_trace_hash_recomputed
    report["bundle_replay_trace_hash"] = replay_trace_hash
    report["bundle_replay_trace_hash_recomputed"] = replay_trace_hash_recomputed

    reproduced_verdict = "FAIL" if violations else "PASS"
    reproduced_status = "FAIL" if violations else "PASS"
    return reproduced_verdict, reproduced_status


def verify_bundle(args: argparse.Namespace) -> int:
    bundle_root = Path(args.bundle_root)
    verify_path = Path(args.out_bundle_verify_json)
    report_path = Path(args.out_report)

    report: dict[str, Any] = {
        "gate": "proof-bundle",
        "mode": "bootstrap_proof_bundle",
        "bundle_root": str(bundle_root),
        "violations": [],
    }

    if not bundle_root.is_dir():
        report["violations"].append(f"missing_bundle_root:{bundle_root}")
        report["verdict"] = "FAIL"
        report["violations_count"] = len(report["violations"])
        write_json(report_path, report)
        write_json(
            verify_path,
            {
                "status": "FAIL",
                "mode": "bootstrap_proof_bundle",
                "bundle_root": str(bundle_root),
                "violations": list(report["violations"]),
                "violations_count": len(report["violations"]),
            },
        )
        return 2

    bundle_manifest, checksums_payload, violations = verify_bundle_schema(bundle_root, report)
    reproduced_verdict, reproduced_status = reproduce_kpl_verdict(bundle_root, report)

    source_report_verdict = str(bundle_manifest.get("source_report_verdict", "") or "")
    source_proof_verify_status = str(bundle_manifest.get("source_proof_verify_status", "") or "")
    source_summary_verdict = str(bundle_manifest.get("source_summary_verdict", "") or "")
    if source_report_verdict and source_report_verdict != report.get("source_report_verdict", ""):
        violations.append(
            "bundle_source_report_verdict_mismatch:"
            f"expected={source_report_verdict}:actual={report.get('source_report_verdict', '')}"
        )
    if source_proof_verify_status and source_proof_verify_status != report.get(
        "source_proof_verify_status", ""
    ):
        violations.append(
            "bundle_source_proof_verify_status_mismatch:"
            f"expected={source_proof_verify_status}:actual={report.get('source_proof_verify_status', '')}"
        )
    if source_summary_verdict and source_summary_verdict != report.get("source_summary_verdict", ""):
        violations.append(
            "bundle_source_summary_verdict_mismatch:"
            f"expected={source_summary_verdict}:actual={report.get('source_summary_verdict', '')}"
        )

    source_proof_hash = str(bundle_manifest.get("source_proof_hash", "") or "").lower()
    source_final_state_hash = str(bundle_manifest.get("source_final_state_hash", "") or "").lower()
    proof_manifest = load_json_file(
        bundle_root / "reports/proof_manifest.json", "bundle_proof_manifest", violations
    )
    proof_hash = str(proof_manifest.get("proof_hash", "") or "").lower()
    final_state_hash = str(proof_manifest.get("final_state_hash", "") or "").lower()
    if source_proof_hash and proof_hash and source_proof_hash != proof_hash:
        violations.append(
            f"bundle_source_proof_hash_mismatch:expected={source_proof_hash}:actual={proof_hash}"
        )
    if source_final_state_hash and final_state_hash and source_final_state_hash != final_state_hash:
        violations.append(
            "bundle_source_final_state_hash_mismatch:"
            f"expected={source_final_state_hash}:actual={final_state_hash}"
        )

    portability_parity = (
        source_report_verdict == reproduced_verdict
        and source_proof_verify_status == reproduced_status
    )
    if not portability_parity:
        violations.append(
            "bundle_portability_parity_mismatch:"
            f"source_report={source_report_verdict}:reproduced_report={reproduced_verdict}:"
            f"source_status={source_proof_verify_status}:reproduced_status={reproduced_status}"
        )

    report["source_report_verdict"] = source_report_verdict
    report["source_proof_verify_status"] = source_proof_verify_status
    report["reproduced_manifest_verdict"] = reproduced_verdict
    report["reproduced_proof_verify_status"] = reproduced_status
    report["portability_parity"] = portability_parity
    report["bundle_required_files_count"] = len(REQUIRED_BUNDLE_FILES)

    verify_payload = {
        "status": "FAIL" if violations else "PASS",
        "mode": "bootstrap_proof_bundle",
        "bundle_version": int(bundle_manifest.get("bundle_version", 0) or 0),
        "bundle_root": str(bundle_root),
        "bundle_id": str(bundle_manifest.get("bundle_id", "") or ""),
        "bundle_id_recomputed": str(report.get("bundle_id_recomputed", "") or ""),
        "source_report_verdict": source_report_verdict,
        "reproduced_manifest_verdict": reproduced_verdict,
        "source_proof_verify_status": source_proof_verify_status,
        "reproduced_proof_verify_status": reproduced_status,
        "source_summary_verdict": source_summary_verdict,
        "portability_parity": portability_parity,
        "checksums_entry_count": int(report.get("checksums_entry_count", 0)),
        "violations": list(violations),
        "violations_count": len(violations),
    }

    report["verdict"] = "FAIL" if violations else "PASS"
    report["violations_count"] = len(violations)
    write_json(report_path, report)
    write_json(verify_path, verify_payload)
    return 2 if violations else 0


def main() -> int:
    args = parse_args()
    if args.mode == "generate":
        return generate_bundle(args)
    return verify_bundle(args)


if __name__ == "__main__":
    raise SystemExit(main())
