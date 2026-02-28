#!/usr/bin/env python3
"""Validate scheduler mailbox semantics (Phase10-C / C2 strict)."""

from __future__ import annotations

import argparse
import json
import re
from pathlib import Path
from typing import Any

S0 = "S0_START"
S1 = "S1_DISPATCH"
S2 = "S2_MAILBOX_DECISION"
S3 = "S3_DECISION_APPLIED"
S4 = "S4_USER_CONTINUE"

TOKEN_DISPATCH = "P10_SCHED_DISPATCH"
TOKEN_MAILBOX = "P10_MAILBOX_DECISION"
TOKEN_APPLIED = "P10_DECISION_APPLIED"
TOKEN_USER = "P10_RING3_USER_CODE"
OWNER_PID = 2

REQUIRED_SEQUENCE = [TOKEN_DISPATCH, TOKEN_MAILBOX, TOKEN_APPLIED, TOKEN_USER]
SEMANTIC_TOKENS = tuple(REQUIRED_SEQUENCE)
FORBIDDEN_TOKENS = (
    "P10_SCHED_FALLBACK",
    "P10_READY_HEAD_FALLBACK",
    "P10_MAILBOX_OWNER_MISMATCH",
    "P10_MAILBOX_OWNER_MISSING_FATAL",
    "P10_MAILBOX_OWNER_NOT_READY_FATAL",
)

TOKEN_PATTERNS = {
    token: re.compile(rf"(?<![A-Za-z0-9_]){re.escape(token)}(?![A-Za-z0-9_])")
    for token in SEMANTIC_TOKENS + FORBIDDEN_TOKENS
}
FORBIDDEN_X_PREFIX_PATTERN = re.compile(r"(?<![A-Za-z0-9_])(XP10_[A-Za-z0-9_]+)")

MAILBOX_META_PATTERN = re.compile(
    r"(?<![A-Za-z0-9_])"
    r"P10_MAILBOX_DECISION"
    r"(?:\s+id=(?P<id>\d+)\s+pid=(?P<pid>\d+)\s+valid=(?P<valid>[01])\s+src=(?P<src>\d+))?"
    r"(?![A-Za-z0-9_])"
)

APPLIED_META_PATTERN = re.compile(
    r"(?<![A-Za-z0-9_])"
    r"P10_DECISION_APPLIED"
    r"(?:\s+id=(?P<id>\d+)\s+pid=(?P<pid>\d+)\s+valid=(?P<valid>[01])\s+src=(?P<src>\d+))?"
    r"(?![A-Za-z0-9_])"
)

C2_SITE_VALUES = ("START", "YIELD", "BLOCK", "IRQ")
C2_REASON_VALUES = (
    "NON_OWNER",
    "EPOCH_DUP",
    "EPOCH_STALE",
    "CAND_NOT_RUNNABLE",
    "OWNERSET_VIOLATION",
    "NO_ELIGIBLE_STRICT",
    "MALFORMED_REQUEST",
)

C2_ACCEPT_TOKEN = "[[AYKEN_SCHED_MB_ACCEPT]]"
C2_REJECT_TOKEN = "[[AYKEN_SCHED_MB_REJECT]]"
C2_ARBITER_TOKEN = "[[AYKEN_SCHED_ARBITER_DECISION]]"
C2_SWITCH_TOKEN = "[[AYKEN_CTX_SWITCH]]"
C2_CURSOR_TOKEN = "[[AYKEN_SCHED_CURSOR_ADVANCE]]"

C2_ACCEPT_PATTERN = re.compile(
    r"\[\[AYKEN_SCHED_MB_ACCEPT\]\]\s+"
    r"owner=(?P<owner>\d+)\s+"
    r"epoch=(?P<epoch>\d+)\s+"
    r"cand=(?P<cand>\d+)\s+"
    r"site=(?P<site>START|YIELD|BLOCK|IRQ)"
)

C2_REJECT_PATTERN = re.compile(
    r"\[\[AYKEN_SCHED_MB_REJECT\]\]\s+"
    r"reason=(?P<reason>NON_OWNER|EPOCH_DUP|EPOCH_STALE|CAND_NOT_RUNNABLE|"
    r"OWNERSET_VIOLATION|NO_ELIGIBLE_STRICT|MALFORMED_REQUEST)\s+"
    r"owner=(?P<owner>\d+)\s+"
    r"epoch=(?P<epoch>\d+)\s+"
    r"cand=(?P<cand>\d+)\s+"
    r"site=(?P<site>START|YIELD|BLOCK|IRQ)"
)

C2_ARBITER_PATTERN = re.compile(
    r"\[\[AYKEN_SCHED_ARBITER_DECISION\]\]\s+"
    r"decision_id=(?P<decision_id>\d+)\s+"
    r"site=(?P<site>START|YIELD|BLOCK|IRQ)\s+"
    r"owner=(?P<owner>\d+)\s+"
    r"from=(?P<from_pid>\d+)\s+"
    r"to=(?P<to_pid>\d+)\s+"
    r"epoch=(?P<epoch>\d+)"
)

C2_SWITCH_PATTERN = re.compile(
    r"\[\[AYKEN_CTX_SWITCH\]\]\s+"
    r"decision_id=(?P<decision_id>\d+)\s+"
    r"from=(?P<from_pid>\d+)\s+"
    r"to=(?P<to_pid>\d+)"
)

C2_CURSOR_PATTERN = re.compile(
    r"\[\[AYKEN_SCHED_CURSOR_ADVANCE\]\]\s+"
    r"decision_id=(?P<decision_id>\d+)\s+"
    r"owner=(?P<owner>\d+)\s+"
    r"next_owner=(?P<next_owner>\d+)"
)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Validate scheduler mailbox semantic state machine."
    )
    parser.add_argument("--events", required=True, help="Input events.jsonl path")
    parser.add_argument("--log", required=True, help="Input marker.log path")
    parser.add_argument("--out", required=True, help="Output report.json path")
    parser.add_argument(
        "--require-metadata",
        choices=("0", "1"),
        default="1",
        help="Require id/pid/valid metadata on mailbox markers (default: 1)",
    )
    parser.add_argument(
        "--c2-strict",
        choices=("0", "1"),
        default="0",
        help="Enable C2 strict mailbox/arbitration assertions (default: 0).",
    )
    parser.add_argument(
        "--c2-owner-set",
        default="2",
        help="CSV owner PID set used for C2 strict immutability checks.",
    )
    parser.add_argument(
        "--c2-require-cursor-marker",
        choices=("0", "1"),
        default="1",
        help="Require [[AYKEN_SCHED_CURSOR_ADVANCE]] for each applied decision in C2 strict mode (default: 1).",
    )
    return parser.parse_args()


def write_report(path: Path, report: dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def fail(path: Path, report: dict[str, Any]) -> int:
    report["verdict"] = "FAIL"
    report["violations_count"] = len(report.get("violations", []))
    write_report(path, report)
    return 2


def succeed(path: Path, report: dict[str, Any]) -> int:
    report["verdict"] = "PASS"
    report["violations_count"] = 0
    report["violations"] = []
    write_report(path, report)
    return 0


def load_events(path: Path) -> list[dict[str, Any]]:
    rows: list[dict[str, Any]] = []
    with path.open("r", encoding="utf-8", errors="replace") as fh:
        for line_no, raw in enumerate(fh, start=1):
            line = raw.strip()
            if not line:
                continue
            try:
                row = json.loads(line)
            except Exception as exc:  # pragma: no cover - fail-closed path
                raise RuntimeError(
                    f"events_parse_error:{path}:line={line_no}:{type(exc).__name__}"
                ) from exc
            if not isinstance(row, dict):
                raise RuntimeError(f"events_type_error:{path}:line={line_no}")
            rows.append(row)
    return rows


def expected_for(state: str) -> list[str]:
    if state == S0:
        return [TOKEN_DISPATCH]
    if state == S1:
        return [TOKEN_MAILBOX]
    if state == S2:
        return [TOKEN_APPLIED]
    if state == S3:
        return [TOKEN_USER]
    return []


def parse_meta_matches(
    pattern: re.Pattern[str], text: str, token: str
) -> list[dict[str, Any]]:
    rows: list[dict[str, Any]] = []
    for match in pattern.finditer(text):
        row: dict[str, Any] = {
            "token": token,
            "offset": match.start(),
            "raw": match.group(0),
            "id": None,
            "pid": None,
            "valid": None,
            "src": None,
        }
        if match.group("id") is not None:
            row["id"] = int(match.group("id"))
        if match.group("pid") is not None:
            row["pid"] = int(match.group("pid"))
        if match.group("valid") is not None:
            row["valid"] = int(match.group("valid"))
        if match.group("src") is not None:
            row["src"] = int(match.group("src"))
        rows.append(row)
    return rows


def parse_csv_pid_set(value: str) -> list[int]:
    raw_parts = [part.strip() for part in value.split(",") if part.strip()]
    if not raw_parts:
        raise ValueError("owner_set_empty")
    out: list[int] = []
    seen: set[int] = set()
    for part in raw_parts:
        if not part.isdigit():
            raise ValueError(f"owner_set_bad_token:{part}")
        pid = int(part)
        if pid <= 0:
            raise ValueError(f"owner_set_non_positive:{pid}")
        if pid in seen:
            raise ValueError(f"owner_set_duplicate:{pid}")
        seen.add(pid)
        out.append(pid)
    return out


def parse_c2_rows(
    pattern: re.Pattern[str],
    text: str,
    token: str,
    int_fields: tuple[str, ...],
    str_fields: tuple[str, ...],
) -> list[dict[str, Any]]:
    rows: list[dict[str, Any]] = []
    for match in pattern.finditer(text):
        row: dict[str, Any] = {
            "token": token,
            "offset": match.start(),
            "raw": match.group(0),
        }
        for field in int_fields:
            row[field] = int(match.group(field))
        for field in str_fields:
            row[field] = match.group(field)
        rows.append(row)
    return rows


def raw_token_count(text: str, token: str) -> int:
    return len(re.findall(re.escape(token), text))


def validate_c1(
    report: dict[str, Any],
    sequence: list[str],
    log_text: str,
    require_metadata: bool,
    owner_pid: int,
) -> None:
    for token in SEMANTIC_TOKENS:
        event_count = report["observed_events"][token]
        log_count = report["observed_log"][token]
        if event_count == 0:
            report["violations"].append(f"missing_required:{token}")
        if event_count > 1:
            report["violations"].append(f"duplicate_token:{token}:count={event_count}")
        if event_count != log_count:
            report["violations"].append(
                f"events_log_mismatch:{token}:events={event_count}:log={log_count}"
            )

    mailbox_rows = parse_meta_matches(MAILBOX_META_PATTERN, log_text, TOKEN_MAILBOX)
    applied_rows = parse_meta_matches(APPLIED_META_PATTERN, log_text, TOKEN_APPLIED)
    report["metadata"]["mailbox_decisions"] = mailbox_rows
    report["metadata"]["applied_decisions"] = applied_rows

    if require_metadata:
        for row in mailbox_rows:
            if (
                row["id"] is None
                or row["pid"] is None
                or row["valid"] is None
                or row["src"] is None
            ):
                report["violations"].append(
                    f"metadata_missing_fields:{TOKEN_MAILBOX}:offset={row['offset']}"
                )
        for row in applied_rows:
            if (
                row["id"] is None
                or row["pid"] is None
                or row["valid"] is None
                or row["src"] is None
            ):
                report["violations"].append(
                    f"metadata_missing_fields:{TOKEN_APPLIED}:offset={row['offset']}"
                )

    if (
        len(mailbox_rows) == 1
        and len(applied_rows) == 1
        and mailbox_rows[0]["id"] is not None
        and applied_rows[0]["id"] is not None
    ):
        decision = mailbox_rows[0]
        applied = applied_rows[0]
        if decision["valid"] != 1:
            report["violations"].append(
                f"decision_valid_must_be_1:got={decision['valid']}"
            )
        if applied["valid"] != 0:
            report["violations"].append(
                f"applied_valid_must_be_0:got={applied['valid']}"
            )
        if applied["id"] != decision["id"]:
            report["violations"].append(
                f"decision_id_mismatch:decision={decision['id']}:applied={applied['id']}"
            )
        if applied["pid"] != decision["pid"]:
            report["violations"].append(
                f"decision_pid_mismatch:decision={decision['pid']}:applied={applied['pid']}"
            )
        if decision["src"] != owner_pid:
            report["violations"].append(
                f"decision_src_must_be_owner:got={decision['src']}:expected={owner_pid}"
            )
        if applied["src"] != owner_pid:
            report["violations"].append(
                f"applied_src_must_be_owner:got={applied['src']}:expected={owner_pid}"
            )
        if applied["src"] != decision["src"]:
            report["violations"].append(
                f"decision_src_mismatch:decision={decision['src']}:applied={applied['src']}"
            )
        if decision["id"] <= 0:
            report["violations"].append(f"decision_id_non_positive:{decision['id']}")

    if len(mailbox_rows) > 1:
        ids = [row["id"] for row in mailbox_rows]
        if all(item is not None for item in ids):
            prev = int(ids[0])
            for idx in range(1, len(ids)):
                cur = int(ids[idx])
                if cur <= prev:
                    report["violations"].append(
                        f"decision_id_not_strictly_increasing:index={idx}:prev={prev}:cur={cur}"
                    )
                    break
                prev = cur

    if report["violations"]:
        return

    state = S0
    report["state_trace"].append(state)

    def set_first_divergence(seq_idx: int, observed_token: str) -> None:
        if report.get("first_divergence") is None:
            report["first_divergence"] = {
                "seq_idx": seq_idx,
                "state": state,
                "observed_token": observed_token,
                "expected_next": expected_for(state),
            }

    def step(new_state: str) -> None:
        nonlocal state
        state = new_state
        report["state_trace"].append(state)

    for seq_idx, token in enumerate(sequence):
        if state == S0:
            if token == TOKEN_DISPATCH:
                step(S1)
                continue
        elif state == S1:
            if token == TOKEN_MAILBOX:
                step(S2)
                continue
        elif state == S2:
            if token == TOKEN_APPLIED:
                step(S3)
                continue
        elif state == S3:
            if token == TOKEN_USER:
                step(S4)
                continue
        else:
            set_first_divergence(seq_idx, token)
            report["violations"].append(
                f"extra_token_after_accept:{token}:seq_idx={seq_idx}"
            )
            return

        set_first_divergence(seq_idx, token)
        report["violations"].append(
            "invalid_transition:"
            f"{state}->{token}:seq_idx={seq_idx}:"
            f"expected={','.join(expected_for(state))}"
        )
        return

    if state != S4:
        set_first_divergence(len(sequence), "EOF")
        report["violations"].append(f"incomplete_state_machine:ended_at={state}")


def validate_c2_strict(
    report: dict[str, Any],
    log_text: str,
    owner_set: list[int],
    require_cursor_marker: bool,
) -> None:
    accept_rows = parse_c2_rows(
        C2_ACCEPT_PATTERN,
        log_text,
        "AYKEN_SCHED_MB_ACCEPT",
        ("owner", "epoch", "cand"),
        ("site",),
    )
    reject_rows = parse_c2_rows(
        C2_REJECT_PATTERN,
        log_text,
        "AYKEN_SCHED_MB_REJECT",
        ("owner", "epoch", "cand"),
        ("reason", "site"),
    )
    arbiter_rows = parse_c2_rows(
        C2_ARBITER_PATTERN,
        log_text,
        "AYKEN_SCHED_ARBITER_DECISION",
        ("decision_id", "owner", "from_pid", "to_pid", "epoch"),
        ("site",),
    )
    switch_rows = parse_c2_rows(
        C2_SWITCH_PATTERN,
        log_text,
        "AYKEN_CTX_SWITCH",
        ("decision_id", "from_pid", "to_pid"),
        (),
    )
    cursor_rows = parse_c2_rows(
        C2_CURSOR_PATTERN,
        log_text,
        "AYKEN_SCHED_CURSOR_ADVANCE",
        ("decision_id", "owner", "next_owner"),
        (),
    )

    report["metadata"]["c2"] = {
        "owner_set": owner_set,
        "accept_rows": accept_rows,
        "reject_rows": reject_rows,
        "arbiter_rows": arbiter_rows,
        "switch_rows": switch_rows,
        "cursor_rows": cursor_rows,
    }

    raw_vs_parsed = (
        (C2_ACCEPT_TOKEN, len(accept_rows)),
        (C2_REJECT_TOKEN, len(reject_rows)),
        (C2_ARBITER_TOKEN, len(arbiter_rows)),
        (C2_SWITCH_TOKEN, len(switch_rows)),
        (C2_CURSOR_TOKEN, len(cursor_rows)),
    )
    for token, parsed_count in raw_vs_parsed:
        raw_count = raw_token_count(log_text, token)
        if raw_count != parsed_count:
            report["violations"].append(
                f"malformed_marker_shape:{token}:raw={raw_count}:parsed={parsed_count}"
            )

    if len(arbiter_rows) == 0:
        report["violations"].append("missing_required_c2:AYKEN_SCHED_ARBITER_DECISION")
        return
    if len(switch_rows) == 0:
        report["violations"].append("missing_required_c2:AYKEN_CTX_SWITCH")
        return

    owner_set_values = set(owner_set)
    for row in accept_rows + reject_rows + arbiter_rows:
        owner = int(row["owner"])
        if owner not in owner_set_values:
            report["violations"].append(
                f"owner_not_in_static_set:owner={owner}:owner_set={','.join(str(x) for x in owner_set)}"
            )
    for row in accept_rows + reject_rows:
        if int(row["epoch"]) <= 0:
            report["violations"].append(
                f"owner_epoch_non_positive:owner={row['owner']}:epoch={row['epoch']}"
            )
        if int(row["cand"]) <= 0:
            report["violations"].append(
                f"candidate_pid_non_positive:owner={row['owner']}:cand={row['cand']}"
            )

    switches_by_id: dict[int, dict[str, Any]] = {}
    for idx, row in enumerate(switch_rows):
        did = int(row["decision_id"])
        if did <= 0:
            report["violations"].append(f"decision_id_non_positive:switch:index={idx}:id={did}")
        if int(row["from_pid"]) == int(row["to_pid"]):
            report["violations"].append(
                f"ctx_switch_noop_forbidden:decision_id={did}:from={row['from_pid']}:to={row['to_pid']}"
            )
        if did in switches_by_id:
            report["violations"].append(f"duplicate_ctx_switch_decision_id:{did}")
        else:
            switches_by_id[did] = row

    arbiter_ids: list[int] = []
    owner_last_epoch: dict[int, int] = {}
    prev_id = 0
    for idx, row in enumerate(arbiter_rows):
        did = int(row["decision_id"])
        owner = int(row["owner"])
        epoch = int(row["epoch"])
        from_pid = int(row["from_pid"])
        to_pid = int(row["to_pid"])
        arbiter_ids.append(did)

        if did <= 0:
            report["violations"].append(f"decision_id_non_positive:arbiter:index={idx}:id={did}")
        if did <= prev_id:
            report["violations"].append(
                f"decision_id_not_strictly_increasing:index={idx}:prev={prev_id}:cur={did}"
            )
        prev_id = did

        if from_pid == to_pid:
            report["violations"].append(
                f"arbiter_noop_forbidden:decision_id={did}:from={from_pid}:to={to_pid}"
            )

        prior_epoch = owner_last_epoch.get(owner)
        if prior_epoch is not None and epoch <= prior_epoch:
            report["violations"].append(
                f"owner_epoch_not_strictly_increasing:owner={owner}:prev={prior_epoch}:cur={epoch}"
            )
        owner_last_epoch[owner] = epoch

        sw = switches_by_id.get(did)
        if sw is None:
            report["violations"].append(f"missing_ctx_switch_for_decision_id:{did}")
        else:
            if int(sw["from_pid"]) != from_pid or int(sw["to_pid"]) != to_pid:
                report["violations"].append(
                    "decision_switch_endpoint_mismatch:"
                    f"decision_id={did}:arbiter={from_pid}->{to_pid}:"
                    f"switch={sw['from_pid']}->{sw['to_pid']}"
                )

    for did in switches_by_id:
        if did not in set(arbiter_ids):
            report["violations"].append(f"orphan_ctx_switch_decision_id:{did}")

    rejected_owner_epoch = {(int(r["owner"]), int(r["epoch"])) for r in reject_rows}
    for row in arbiter_rows:
        key = (int(row["owner"]), int(row["epoch"]))
        if key in rejected_owner_epoch:
            report["violations"].append(
                f"reject_followed_by_apply:owner={key[0]}:epoch={key[1]}"
            )

    if require_cursor_marker:
        if len(cursor_rows) == 0 and len(arbiter_rows) > 0:
            report["violations"].append("missing_required_c2:AYKEN_SCHED_CURSOR_ADVANCE")
        if len(cursor_rows) != len(arbiter_rows):
            report["violations"].append(
                f"cursor_advance_count_mismatch:cursor={len(cursor_rows)}:applied={len(arbiter_rows)}"
            )

    if len(cursor_rows) > 0:
        cursor_ids = [int(row["decision_id"]) for row in cursor_rows]
        prev_cursor_id = 0
        for idx, did in enumerate(cursor_ids):
            if did <= 0:
                report["violations"].append(f"cursor_decision_id_non_positive:index={idx}:id={did}")
            if did <= prev_cursor_id:
                report["violations"].append(
                    f"cursor_decision_id_not_strictly_increasing:index={idx}:prev={prev_cursor_id}:cur={did}"
                )
            prev_cursor_id = did
        expected_ids = arbiter_ids
        if require_cursor_marker and cursor_ids != expected_ids:
            report["violations"].append(
                "cursor_applied_id_mismatch:"
                f"cursor_ids={','.join(str(x) for x in cursor_ids)}:"
                f"applied_ids={','.join(str(x) for x in expected_ids)}"
            )
        else:
            expected_set = set(expected_ids)
            for did in cursor_ids:
                if did not in expected_set:
                    report["violations"].append(f"cursor_id_without_applied:{did}")

    owner_count = len(owner_set)
    if owner_count > 0:
        if len(arbiter_rows) < owner_count:
            report["violations"].append(
                f"fairness_insufficient_applied:need={owner_count}:got={len(arbiter_rows)}"
            )
        else:
            first_window = {int(row["owner"]) for row in arbiter_rows[:owner_count]}
            expected = set(owner_set)
            if first_window != expected:
                report["violations"].append(
                    "fairness_smoke_failed:"
                    f"expected={','.join(str(x) for x in sorted(expected))}:"
                    f"observed={','.join(str(x) for x in sorted(first_window))}"
                )


def main() -> int:
    args = parse_args()
    events_path = Path(args.events)
    log_path = Path(args.log)
    out_path = Path(args.out)
    require_metadata = args.require_metadata == "1"
    c2_strict = args.c2_strict == "1"
    c2_require_cursor_marker = args.c2_require_cursor_marker == "1"

    report: dict[str, Any] = {
        "gate": "scheduler-mailbox-phase10c",
        "verdict": "FAIL",
        "expected_owner_pid": OWNER_PID,
        "require_metadata": 1 if require_metadata else 0,
        "c2_strict": 1 if c2_strict else 0,
        "c2_owner_set": [],
        "c2_require_cursor_marker": 1 if c2_require_cursor_marker else 0,
        "c2_assertions": [
            "decision_id_monotonic",
            "owner_epoch_monotonic_applied",
            "reject_not_applied",
            "cursor_applied_only",
            "fairness_smoke",
            "owner_pid_immutability",
        ],
        "violations_count": 0,
        "violations": [],
        "expected_sequence": REQUIRED_SEQUENCE,
        "semantic_sequence": [],
        "state_trace": [],
        "first_divergence": None,
        "observed_events": {token: 0 for token in SEMANTIC_TOKENS},
        "observed_log": {token: 0 for token in SEMANTIC_TOKENS},
        "forbidden_event_counts": {token: 0 for token in FORBIDDEN_TOKENS},
        "forbidden_log_counts": {token: 0 for token in FORBIDDEN_TOKENS},
        "forbidden_log_tokens": [],
        "metadata": {
            "mailbox_decisions": [],
            "applied_decisions": [],
            "c2": {
                "owner_set": [],
                "accept_rows": [],
                "reject_rows": [],
                "arbiter_rows": [],
                "switch_rows": [],
                "cursor_rows": [],
            },
        },
    }

    try:
        c2_owner_set = parse_csv_pid_set(args.c2_owner_set)
    except ValueError as exc:
        report["violations"].append(str(exc))
        return fail(out_path, report)
    report["c2_owner_set"] = c2_owner_set
    report["metadata"]["c2"]["owner_set"] = c2_owner_set

    if not events_path.is_file():
        report["violations"].append(f"missing_events_file:{events_path}")
        return fail(out_path, report)
    if not log_path.is_file():
        report["violations"].append(f"missing_log_file:{log_path}")
        return fail(out_path, report)

    try:
        events = load_events(events_path)
    except Exception as exc:
        report["violations"].append(str(exc))
        return fail(out_path, report)

    sequence: list[str] = []
    for event in events:
        token = str(event.get("type") or event.get("name") or "")
        if token in SEMANTIC_TOKENS:
            sequence.append(token)
            report["observed_events"][token] += 1
        if token in FORBIDDEN_TOKENS:
            report["forbidden_event_counts"][token] += 1
    report["semantic_sequence"] = sequence

    log_text = log_path.read_text(encoding="utf-8", errors="replace")
    for token in SEMANTIC_TOKENS:
        report["observed_log"][token] = len(TOKEN_PATTERNS[token].findall(log_text))
    for token in FORBIDDEN_TOKENS:
        event_count = report["forbidden_event_counts"][token]
        if event_count > 0:
            report["violations"].append(f"forbidden_event_marker:{token}:count={event_count}")
        count = len(TOKEN_PATTERNS[token].findall(log_text))
        report["forbidden_log_counts"][token] = count
        if count > 0:
            report["violations"].append(f"forbidden_marker:{token}:count={count}")

    forbidden_prefixes = sorted(set(FORBIDDEN_X_PREFIX_PATTERN.findall(log_text)))
    report["forbidden_log_tokens"] = forbidden_prefixes
    for token in forbidden_prefixes:
        report["violations"].append(f"forbidden_marker_prefix:{token}")

    if c2_strict:
        validate_c2_strict(
            report=report,
            log_text=log_text,
            owner_set=c2_owner_set,
            require_cursor_marker=c2_require_cursor_marker,
        )
    else:
        validate_c1(
            report=report,
            sequence=sequence,
            log_text=log_text,
            require_metadata=require_metadata,
            owner_pid=OWNER_PID,
        )

    if report["violations"]:
        return fail(out_path, report)
    return succeed(out_path, report)


if __name__ == "__main__":
    raise SystemExit(main())
