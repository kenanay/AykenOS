#!/usr/bin/env python3
"""Validate the Phase10B fail-closed runtime proof transcript."""

from __future__ import annotations

import argparse
import hashlib
import json
import re
from pathlib import Path
from typing import Any

EXEC_SLOT_CREATED = 0
EXEC_SLOT_READY = 1
EXEC_SLOT_RUNNING = 2
EXEC_SLOT_COMPLETED = 3
EXEC_TRACE_ACTOR_VALIDATION = 7

BEGIN_MARKER = "[[P10B_FAIL_CLOSED_BEGIN]]"
END_MARKER = "[[P10B_FAIL_CLOSED_END]]"
EXPECTED_SITE = "phase10b_fail_closed_selftest.trigger"
REPLAY_MANIFEST_VERSION = 1
REPLAY_MODE = "phase10b_fail_closed_replay_v1"
REPLAY_TRACE_SCHEMA = "phase10b_fail_closed_replay_trace_v1"
REPLAY_FINAL_STATE_SCHEMA = "phase10b_fail_closed_final_state_v1"
SOURCE_GATE = "phase10b-fail-closed-proof"

META_RE = re.compile(
    r"^\[\[P10B_FAIL_CLOSED_META\]\] "
    r"site=(?P<site>\S+) "
    r"exec_id=(?P<exec_id>\d+) "
    r"generation=(?P<generation>\d+) "
    r"current=(?P<current>\d+) "
    r"expected=(?P<expected>\d+) "
    r"next=(?P<next>\d+) "
    r"final_state=(?P<final_state>\d+) "
    r"invariants_ok=(?P<invariants_ok>[01]) "
    r"trace_count=(?P<trace_count>\d+)$"
)
TRACE_RE = re.compile(
    r"^\[\[P10B_FAIL_CLOSED_TRACE\]\] "
    r"idx=(?P<idx>\d+) "
    r"tick=(?P<tick>\d+) "
    r"exec_id=(?P<exec_id>\d+) "
    r"generation=(?P<generation>\d+) "
    r"actor=(?P<actor>\d+) "
    r"from=(?P<from_state>\d+) "
    r"to=(?P<to_state>\d+)$"
)
HASH_RE = re.compile(r"^\[\[P10B_FAIL_CLOSED_HASH\]\] sha256=(?P<sha256>[0-9a-f]{64})$")


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Validate the Phase10B fail-closed runtime proof transcript."
    )
    parser.add_argument("--log", required=True, help="marker.log path")
    parser.add_argument("--out", required=True, help="report.json path")
    parser.add_argument("--out-proof", required=True, help="proof.json path")
    parser.add_argument(
        "--out-replay-trace-jsonl", required=True, help="Output replay_trace.jsonl path"
    )
    parser.add_argument(
        "--out-replay-trace-hash-txt", required=True, help="Output replay_trace_hash.txt path"
    )
    parser.add_argument("--out-replay-report", required=True, help="Output replay_report.json path")
    parser.add_argument(
        "--out-replay-manifest-json",
        required=True,
        help="Output replay_manifest.json path",
    )
    parser.add_argument(
        "--out-final-state-hash-txt",
        required=True,
        help="Output final_state_hash.txt path",
    )
    parser.add_argument(
        "--out-replay-result-hash-txt",
        required=True,
        help="Output replay_result_hash.txt path",
    )
    return parser.parse_args()


def write_json(path: Path, payload: dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def write_text(path: Path, value: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text((value or "") + "\n", encoding="utf-8")


def canonical_json(payload: dict[str, Any]) -> bytes:
    return json.dumps(payload, sort_keys=True, separators=(",", ":")).encode("utf-8")


def sha256_hex(payload: bytes) -> str:
    return hashlib.sha256(payload).hexdigest()


def serialize_trace_rows_bytes(rows: list[dict[str, Any]]) -> bytes:
    return b"".join(
        json.dumps(row, sort_keys=True, separators=(",", ":")).encode("utf-8") + b"\n"
        for row in rows
    )


def write_trace_rows(path: Path, rows: list[dict[str, Any]]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(serialize_trace_rows_bytes(rows))


def manifest_without_hash(payload: dict[str, Any]) -> dict[str, Any]:
    stripped = dict(payload)
    stripped.pop("manifest_hash", None)
    return stripped


def compute_manifest_hash(payload: dict[str, Any]) -> str:
    return sha256_hex(canonical_json(manifest_without_hash(payload)))


def compute_replay_result_hash(replay_trace_hash: str, final_state_hash: str) -> str:
    payload = {
        "mode": REPLAY_MODE,
        "replay_execution_trace_hash": replay_trace_hash,
        "final_state_hash": final_state_hash,
    }
    return sha256_hex(canonical_json(payload))


def can_transition(from_state: int, to_state: int) -> bool:
    if from_state == EXEC_SLOT_CREATED:
        return to_state in {EXEC_SLOT_READY, 4}
    if from_state == EXEC_SLOT_READY:
        return to_state in {EXEC_SLOT_RUNNING, 5, 7}
    if from_state == EXEC_SLOT_RUNNING:
        return to_state in {EXEC_SLOT_COMPLETED, 4, 5, 7}
    if from_state == EXEC_SLOT_COMPLETED:
        return to_state == 6
    if from_state in {4, 5, 7}:
        return False
    if from_state == 6:
        return to_state == 6
    return False


def build_normalized_replay_rows(trace_rows: list[dict[str, Any]]) -> tuple[list[dict[str, Any]], int]:
    if not trace_rows:
        return [], 0

    tick_base = int(trace_rows[0]["tick"])
    replay_rows: list[dict[str, Any]] = []
    for idx, row in enumerate(trace_rows, start=1):
        replay_rows.append(
            {
                "trace_seq": idx,
                "slot_ordinal": 1,
                "ltick": int(row["tick"]) - tick_base,
                "event_type": "execution_slot_transition",
                "actor": int(row["actor"]),
                "from_state": int(row["from_state"]),
                "to_state": int(row["to_state"]),
            }
        )

    return replay_rows, tick_base


def default_replay_report() -> dict[str, Any]:
    return {
        "status": "FAIL",
        "mode": REPLAY_MODE,
        "source_gate": SOURCE_GATE,
        "source_site": EXPECTED_SITE,
        "trace_schema": REPLAY_TRACE_SCHEMA,
        "final_state_schema": REPLAY_FINAL_STATE_SCHEMA,
        "normalized_slot_ordinal": 1,
        "raw_execution_id_observed": 0,
        "raw_generation_observed": 0,
        "normalized_tick_base": 0,
        "normalized_tick_zero_origin": False,
        "replay_execution_trace_hash": "",
        "final_state_hash": "",
        "replay_result_hash": "",
        "replay_manifest_hash": "",
        "replay_event_count": 0,
        "violations": [],
        "violations_count": 0,
    }


def default_replay_manifest() -> dict[str, Any]:
    return {
        "manifest_version": REPLAY_MANIFEST_VERSION,
        "mode": REPLAY_MODE,
        "source_gate": SOURCE_GATE,
        "source_site": EXPECTED_SITE,
        "trace_schema": REPLAY_TRACE_SCHEMA,
        "final_state_schema": REPLAY_FINAL_STATE_SCHEMA,
        "slot_ordinal": 1,
        "replay_event_count": 0,
        "replay_execution_trace_hash": "",
        "final_state_hash": "",
        "replay_result_hash": "",
        "manifest_hash": "",
    }


def write_outputs(
    report_path: Path,
    proof_path: Path,
    replay_trace_path: Path,
    replay_trace_hash_path: Path,
    replay_report_path: Path,
    replay_manifest_path: Path,
    final_state_hash_path: Path,
    replay_result_hash_path: Path,
    report: dict[str, Any],
    proof: dict[str, Any],
    replay_rows: list[dict[str, Any]],
    replay_trace_hash: str,
    replay_report: dict[str, Any],
    replay_manifest: dict[str, Any],
    final_state_hash: str,
    replay_result_hash: str,
) -> None:
    write_json(report_path, report)
    write_json(proof_path, proof)
    write_trace_rows(replay_trace_path, replay_rows)
    write_text(replay_trace_hash_path, replay_trace_hash)
    write_json(replay_report_path, replay_report)
    write_json(replay_manifest_path, replay_manifest)
    write_text(final_state_hash_path, final_state_hash)
    write_text(replay_result_hash_path, replay_result_hash)


def finalize(
    ok: bool,
    report_path: Path,
    proof_path: Path,
    replay_trace_path: Path,
    replay_trace_hash_path: Path,
    replay_report_path: Path,
    replay_manifest_path: Path,
    final_state_hash_path: Path,
    replay_result_hash_path: Path,
    report: dict[str, Any],
    proof: dict[str, Any],
    replay_rows: list[dict[str, Any]],
    replay_trace_hash: str,
    replay_report: dict[str, Any],
    replay_manifest: dict[str, Any],
    final_state_hash: str,
    replay_result_hash: str,
) -> int:
    if ok:
        report["verdict"] = "PASS"
        report["violations"] = []
        report["violations_count"] = 0
        replay_report["status"] = "PASS"
        replay_report["violations"] = []
        replay_report["violations_count"] = 0
    else:
        report["verdict"] = "FAIL"
        report["violations_count"] = len(report.get("violations", []))
        replay_report["status"] = "FAIL"
        replay_report["violations"] = list(report.get("violations", []))
        replay_report["violations_count"] = len(report.get("violations", []))

    write_outputs(
        report_path,
        proof_path,
        replay_trace_path,
        replay_trace_hash_path,
        replay_report_path,
        replay_manifest_path,
        final_state_hash_path,
        replay_result_hash_path,
        report,
        proof,
        replay_rows,
        replay_trace_hash,
        replay_report,
        replay_manifest,
        final_state_hash,
        replay_result_hash,
    )
    return 0 if ok else 2


def main() -> int:
    args = parse_args()
    log_path = Path(args.log)
    report_path = Path(args.out)
    proof_path = Path(args.out_proof)
    replay_trace_path = Path(args.out_replay_trace_jsonl)
    replay_trace_hash_path = Path(args.out_replay_trace_hash_txt)
    replay_report_path = Path(args.out_replay_report)
    replay_manifest_path = Path(args.out_replay_manifest_json)
    final_state_hash_path = Path(args.out_final_state_hash_txt)
    replay_result_hash_path = Path(args.out_replay_result_hash_txt)

    report: dict[str, Any] = {
        "gate": SOURCE_GATE,
        "verdict": "FAIL",
        "violations_count": 0,
        "violations": [],
        "expected_site": EXPECTED_SITE,
        "proof_hash_match": False,
        "invariants_ok": False,
        "trace_count_match": False,
        "replay_format_frozen": True,
        "replay_mode": REPLAY_MODE,
        "replay_trace_schema": REPLAY_TRACE_SCHEMA,
        "replay_final_state_schema": REPLAY_FINAL_STATE_SCHEMA,
        "normalized_slot_ordinal": 1,
        "normalized_tick_zero_origin": False,
    }
    proof: dict[str, Any] = {
        "expected_site": EXPECTED_SITE,
        "meta": {},
        "trace": [],
        "reported_sha256": "",
        "computed_sha256": "",
    }
    replay_rows: list[dict[str, Any]] = []
    replay_trace_hash = ""
    replay_report = default_replay_report()
    replay_manifest = default_replay_manifest()
    final_state_hash = ""
    replay_result_hash = ""

    if not log_path.is_file():
        report["violations"].append(f"missing_log_file:{log_path}")
        return finalize(
            False,
            report_path,
            proof_path,
            replay_trace_path,
            replay_trace_hash_path,
            replay_report_path,
            replay_manifest_path,
            final_state_hash_path,
            replay_result_hash_path,
            report,
            proof,
            replay_rows,
            replay_trace_hash,
            replay_report,
            replay_manifest,
            final_state_hash,
            replay_result_hash,
        )

    lines = log_path.read_text(encoding="utf-8", errors="replace").splitlines()
    begin_lines = [idx for idx, line in enumerate(lines) if line.strip() == BEGIN_MARKER]
    end_lines = [idx for idx, line in enumerate(lines) if line.strip() == END_MARKER]

    if len(begin_lines) != 1:
        report["violations"].append(f"expected_single_begin_marker:count={len(begin_lines)}")
    if len(end_lines) != 1:
        report["violations"].append(f"expected_single_end_marker:count={len(end_lines)}")
    if report["violations"]:
        return finalize(
            False,
            report_path,
            proof_path,
            replay_trace_path,
            replay_trace_hash_path,
            replay_report_path,
            replay_manifest_path,
            final_state_hash_path,
            replay_result_hash_path,
            report,
            proof,
            replay_rows,
            replay_trace_hash,
            replay_report,
            replay_manifest,
            final_state_hash,
            replay_result_hash,
        )
    if begin_lines[0] >= end_lines[0]:
        report["violations"].append("marker_order_invalid:begin_after_end")
        return finalize(
            False,
            report_path,
            proof_path,
            replay_trace_path,
            replay_trace_hash_path,
            replay_report_path,
            replay_manifest_path,
            final_state_hash_path,
            replay_result_hash_path,
            report,
            proof,
            replay_rows,
            replay_trace_hash,
            replay_report,
            replay_manifest,
            final_state_hash,
            replay_result_hash,
        )

    transcript = lines[begin_lines[0] + 1 : end_lines[0]]
    meta_lines = [line for line in transcript if META_RE.match(line)]
    hash_lines = [line for line in transcript if HASH_RE.match(line)]
    trace_lines = [line for line in transcript if TRACE_RE.match(line)]

    if len(meta_lines) != 1:
        report["violations"].append(f"expected_single_meta_line:count={len(meta_lines)}")
    if len(hash_lines) != 1:
        report["violations"].append(f"expected_single_hash_line:count={len(hash_lines)}")
    if report["violations"]:
        return finalize(
            False,
            report_path,
            proof_path,
            replay_trace_path,
            replay_trace_hash_path,
            replay_report_path,
            replay_manifest_path,
            final_state_hash_path,
            replay_result_hash_path,
            report,
            proof,
            replay_rows,
            replay_trace_hash,
            replay_report,
            replay_manifest,
            final_state_hash,
            replay_result_hash,
        )

    meta_match = META_RE.match(meta_lines[0])
    hash_match = HASH_RE.match(hash_lines[0])
    assert meta_match is not None
    assert hash_match is not None

    meta = {
        key: int(value) if key != "site" else value for key, value in meta_match.groupdict().items()
    }
    proof["meta"] = meta
    proof["reported_sha256"] = hash_match.group("sha256")

    for line in trace_lines:
        match = TRACE_RE.match(line)
        assert match is not None
        proof["trace"].append({key: int(value) for key, value in match.groupdict().items()})

    canonical_lines = [meta_lines[0], *trace_lines]
    proof["computed_sha256"] = sha256_hex(
        "".join(f"{line}\n" for line in canonical_lines).encode("utf-8")
    )

    report["proof_hash_match"] = proof["reported_sha256"] == proof["computed_sha256"]
    report["invariants_ok"] = meta["invariants_ok"] == 1
    report["trace_count_match"] = meta["trace_count"] == len(proof["trace"])

    if meta["site"] != EXPECTED_SITE:
        report["violations"].append(f"unexpected_site:{meta['site']}")
    if meta["exec_id"] == 0:
        report["violations"].append("exec_id_zero")
    if meta["generation"] == 0:
        report["violations"].append("generation_zero")
    if meta["current"] != EXEC_SLOT_READY:
        report["violations"].append(f"unexpected_current_state:{meta['current']}")
    if meta["expected"] != EXEC_SLOT_RUNNING:
        report["violations"].append(f"unexpected_expected_state:{meta['expected']}")
    if meta["next"] != EXEC_SLOT_COMPLETED:
        report["violations"].append(f"unexpected_next_state:{meta['next']}")
    if meta["final_state"] != EXEC_SLOT_READY:
        report["violations"].append(f"unexpected_final_state:{meta['final_state']}")
    if meta["invariants_ok"] != 1:
        report["violations"].append("invariants_failed")
    if meta["trace_count"] != 1:
        report["violations"].append(f"unexpected_trace_count:{meta['trace_count']}")
    if not report["proof_hash_match"]:
        report["violations"].append("proof_hash_mismatch")
    if not report["trace_count_match"]:
        report["violations"].append(
            f"trace_count_mismatch:meta={meta['trace_count']}:actual={len(proof['trace'])}"
        )

    if len(proof["trace"]) != 1:
        report["violations"].append(f"unexpected_trace_rows:{len(proof['trace'])}")
    else:
        trace = proof["trace"][0]
        if trace["idx"] != 0:
            report["violations"].append(f"unexpected_trace_index:{trace['idx']}")
        if trace["exec_id"] != meta["exec_id"]:
            report["violations"].append("trace_exec_id_mismatch")
        if trace["generation"] != meta["generation"]:
            report["violations"].append("trace_generation_mismatch")
        if trace["actor"] != EXEC_TRACE_ACTOR_VALIDATION:
            report["violations"].append(f"unexpected_trace_actor:{trace['actor']}")
        if trace["from_state"] != EXEC_SLOT_CREATED:
            report["violations"].append(f"unexpected_trace_from:{trace['from_state']}")
        if trace["to_state"] != EXEC_SLOT_READY:
            report["violations"].append(f"unexpected_trace_to:{trace['to_state']}")
        if trace["tick"] == 0:
            report["violations"].append("trace_tick_zero")
        if not can_transition(trace["from_state"], trace["to_state"]):
            report["violations"].append(
                f"trace_transition_invalid:{trace['from_state']}->{trace['to_state']}"
            )

    replay_rows, tick_base = build_normalized_replay_rows(proof["trace"])
    replay_trace_hash = sha256_hex(serialize_trace_rows_bytes(replay_rows))
    report["replay_event_count"] = len(replay_rows)
    report["normalized_tick_base"] = tick_base
    report["normalized_tick_zero_origin"] = bool(replay_rows) and replay_rows[0]["ltick"] == 0

    if replay_rows:
        previous_ltick = -1
        for idx, row in enumerate(replay_rows, start=1):
            ltick = int(row["ltick"])
            if ltick < 0:
                report["violations"].append(f"negative_replay_ltick:entry={idx}")
            if ltick < previous_ltick:
                report["violations"].append(f"non_monotonic_replay_ltick:entry={idx}")
            previous_ltick = ltick

    final_state_payload = {
        "schema_version": REPLAY_MANIFEST_VERSION,
        "mode": REPLAY_MODE,
        "source_gate": SOURCE_GATE,
        "source_site": meta.get("site", ""),
        "slot_ordinal": 1,
        "current_state": int(meta.get("current", 0)),
        "expected_state": int(meta.get("expected", 0)),
        "attempted_next_state": int(meta.get("next", 0)),
        "final_state": int(meta.get("final_state", 0)),
        "invariants_ok": int(meta.get("invariants_ok", 0)),
        "replay_event_count": len(replay_rows),
    }
    final_state_hash = sha256_hex(canonical_json(final_state_payload))
    replay_result_hash = compute_replay_result_hash(replay_trace_hash, final_state_hash)

    replay_manifest = {
        "manifest_version": REPLAY_MANIFEST_VERSION,
        "mode": REPLAY_MODE,
        "source_gate": SOURCE_GATE,
        "source_site": meta.get("site", ""),
        "trace_schema": REPLAY_TRACE_SCHEMA,
        "final_state_schema": REPLAY_FINAL_STATE_SCHEMA,
        "slot_ordinal": 1,
        "replay_event_count": len(replay_rows),
        "replay_execution_trace_hash": replay_trace_hash,
        "final_state_hash": final_state_hash,
        "replay_result_hash": replay_result_hash,
        "manifest_hash": "",
    }
    replay_manifest["manifest_hash"] = compute_manifest_hash(replay_manifest)

    replay_report = {
        "status": "FAIL",
        "mode": REPLAY_MODE,
        "source_gate": SOURCE_GATE,
        "source_site": meta.get("site", ""),
        "trace_schema": REPLAY_TRACE_SCHEMA,
        "final_state_schema": REPLAY_FINAL_STATE_SCHEMA,
        "normalized_slot_ordinal": 1,
        "raw_execution_id_observed": int(meta.get("exec_id", 0)),
        "raw_generation_observed": int(meta.get("generation", 0)),
        "normalized_tick_base": tick_base,
        "normalized_tick_zero_origin": report["normalized_tick_zero_origin"],
        "replay_execution_trace_hash": replay_trace_hash,
        "final_state_hash": final_state_hash,
        "replay_result_hash": replay_result_hash,
        "replay_manifest_hash": str(replay_manifest["manifest_hash"]),
        "replay_event_count": len(replay_rows),
        "violations": list(report["violations"]),
        "violations_count": len(report["violations"]),
    }

    report["replay_execution_trace_hash"] = replay_trace_hash
    report["final_state_hash"] = final_state_hash
    report["replay_result_hash"] = replay_result_hash
    report["replay_manifest_hash"] = str(replay_manifest["manifest_hash"])

    return finalize(
        not report["violations"],
        report_path,
        proof_path,
        replay_trace_path,
        replay_trace_hash_path,
        replay_report_path,
        replay_manifest_path,
        final_state_hash_path,
        replay_result_hash_path,
        report,
        proof,
        replay_rows,
        replay_trace_hash,
        replay_report,
        replay_manifest,
        final_state_hash,
        replay_result_hash,
    )


if __name__ == "__main__":
    raise SystemExit(main())
