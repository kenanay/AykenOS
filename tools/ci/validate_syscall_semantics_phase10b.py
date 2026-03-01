#!/usr/bin/env python3
"""Validate Phase10-B syscall boundary semantics (fail-closed).

V1 is aligned with the current Phase10-A2 marker topology:
START -> ENTER -> RETURN -> (CAP in negative mode) -> USER_CONTINUE.

If --log is provided, semantic token counts in marker.log must match
counts extracted from events.jsonl (extractor drift guard).
"""

from __future__ import annotations

import argparse
import json
import re
from pathlib import Path
from typing import Any

S0 = "S0_START"
S1 = "S1_ENTER"
S2 = "S2_RETURN"
S3 = "S3_CAP"
S4 = "S4_USER_CONTINUE"

TOKEN_ENTER = "P10_SYSCALL_ENTER"
TOKEN_CAP = "P10_CAP_ENFORCED"
TOKEN_RETURN = "P10_SYSCALL_RETURN"
TOKEN_USER = "P10_RING3_USER_CODE"

ALLOWED_MODES = {"positive", "negative"}
SEMANTIC_TOKENS = (TOKEN_ENTER, TOKEN_CAP, TOKEN_RETURN, TOKEN_USER)
TOKEN_PATTERNS = {
    token: re.compile(rf"(?<![A-Za-z0-9_]){re.escape(token)}(?![A-Za-z0-9_])")
    for token in SEMANTIC_TOKENS
}
FORBIDDEN_X_PREFIX_PATTERN = re.compile(r"(?<![A-Za-z0-9_])(XP10_[A-Za-z0-9_]+)")


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Validate Phase10-B syscall semantic state machine."
    )
    parser.add_argument("--events", required=True, help="Input events.jsonl")
    parser.add_argument("--mode", required=True, choices=sorted(ALLOWED_MODES))
    parser.add_argument("--out", required=True, help="Output report.json")
    parser.add_argument("--log", required=False, help="Optional marker.log path")
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
                obj = json.loads(line)
            except Exception as exc:  # pragma: no cover - fail-closed path
                raise RuntimeError(
                    f"events_parse_error:{path}:line={line_no}:{type(exc).__name__}"
                ) from exc
            if not isinstance(obj, dict):
                raise RuntimeError(f"events_type_error:{path}:line={line_no}")
            rows.append(obj)
    return rows


def main() -> int:
    args = parse_args()
    events_path = Path(args.events)
    out_path = Path(args.out)
    log_path = Path(args.log) if args.log else None

    report: dict[str, Any] = {
        "gate": "syscall-semantics-phase10b",
        "mode": args.mode,
        "verdict": "FAIL",
        "violations_count": 0,
        "violations": [],
        "state_trace": [],
        "semantic_sequence": [],
        "first_divergence": None,
        "observed": {
            TOKEN_ENTER: 0,
            TOKEN_CAP: 0,
            TOKEN_RETURN: 0,
            TOKEN_USER: 0,
        },
    }
    if log_path is not None:
        report["log_path"] = str(log_path)
        report["log_observed"] = {
            TOKEN_ENTER: 0,
            TOKEN_CAP: 0,
            TOKEN_RETURN: 0,
            TOKEN_USER: 0,
        }
        report["forbidden_log_tokens"] = []

    if not events_path.is_file():
        report["violations"].append(f"missing_events_file:{events_path}")
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
            report["observed"][token] += 1

    report["semantic_sequence"] = sequence

    if log_path is not None:
        if not log_path.is_file():
            report["violations"].append(f"missing_log_file:{log_path}")
        else:
            text = log_path.read_text(encoding="utf-8", errors="replace")
            forbidden = sorted(set(FORBIDDEN_X_PREFIX_PATTERN.findall(text)))
            report["forbidden_log_tokens"] = forbidden
            for token in forbidden:
                report["violations"].append(f"forbidden_marker_prefix:{token}")
            for token in SEMANTIC_TOKENS:
                log_count = len(TOKEN_PATTERNS[token].findall(text))
                report["log_observed"][token] = log_count
                events_count = report["observed"][token]
                if log_count != events_count:
                    report["violations"].append(
                        f"events_log_mismatch:{token}:events={events_count}:log={log_count}"
                    )

    if report["observed"][TOKEN_ENTER] == 0:
        report["violations"].append(f"missing_required:{TOKEN_ENTER}")
    if report["observed"][TOKEN_RETURN] == 0:
        report["violations"].append(f"missing_required:{TOKEN_RETURN}")
    if report["observed"][TOKEN_USER] == 0:
        report["violations"].append(f"missing_required:{TOKEN_USER}")

    for token in (TOKEN_ENTER, TOKEN_RETURN, TOKEN_USER):
        if report["observed"][token] > 1:
            report["violations"].append(
                f"duplicate_token:{token}:count={report['observed'][token]}"
            )

    cap_count = report["observed"][TOKEN_CAP]
    if args.mode == "positive":
        if cap_count != 0:
            report["violations"].append(f"cap_forbidden_in_positive:count={cap_count}")
    else:
        if cap_count == 0:
            report["violations"].append("cap_required_in_negative:missing")
        if cap_count > 1:
            report["violations"].append(f"duplicate_token:{TOKEN_CAP}:count={cap_count}")

    if report["violations"]:
        return fail(out_path, report)

    state = S0
    report["state_trace"].append(state)

    def expected_for(current_state: str, mode: str) -> list[str]:
        if current_state == S0:
            return [TOKEN_ENTER]
        if current_state == S1:
            return [TOKEN_RETURN]
        if current_state == S2:
            return [TOKEN_CAP] if mode == "negative" else [TOKEN_USER]
        if current_state == S3:
            return [TOKEN_USER]
        return []

    def set_first_divergence(seq_idx: int, current_state: str, observed_token: str) -> None:
        if report.get("first_divergence") is None:
            report["first_divergence"] = {
                "seq_idx": seq_idx,
                "state": current_state,
                "observed_token": observed_token,
                "expected_next": expected_for(current_state, args.mode),
            }

    def step(new_state: str) -> None:
        nonlocal state
        state = new_state
        report["state_trace"].append(state)

    for seq_idx, token in enumerate(sequence):
        if state == S0:
            if token == TOKEN_ENTER:
                step(S1)
            else:
                set_first_divergence(seq_idx, state, token)
                report["violations"].append(
                    "invalid_transition:"
                    f"{state}->{token}:seq_idx={seq_idx}:"
                    f"expected={','.join(expected_for(state, args.mode))}"
                )
                return fail(out_path, report)

        elif state == S1:
            if token == TOKEN_RETURN:
                step(S2)
            else:
                set_first_divergence(seq_idx, state, token)
                report["violations"].append(
                    "invalid_transition:"
                    f"{state}->{token}:seq_idx={seq_idx}:"
                    f"expected={','.join(expected_for(state, args.mode))}"
                )
                return fail(out_path, report)

        elif state == S2:
            if token == TOKEN_CAP:
                if args.mode != "negative":
                    set_first_divergence(seq_idx, state, token)
                    report["violations"].append(
                        "cap_seen_but_mode_positive:"
                        f"seq_idx={seq_idx}:expected={','.join(expected_for(state, args.mode))}"
                    )
                    return fail(out_path, report)
                step(S3)
            elif token == TOKEN_USER:
                if args.mode == "negative":
                    set_first_divergence(seq_idx, state, token)
                    report["violations"].append(
                        "cap_required_before_user_in_negative:"
                        f"seq_idx={seq_idx}:expected={','.join(expected_for(state, args.mode))}"
                    )
                    return fail(out_path, report)
                step(S4)
            else:
                set_first_divergence(seq_idx, state, token)
                report["violations"].append(
                    "invalid_transition:"
                    f"{state}->{token}:seq_idx={seq_idx}:"
                    f"expected={','.join(expected_for(state, args.mode))}"
                )
                return fail(out_path, report)

        elif state == S3:
            if token == TOKEN_USER:
                step(S4)
            else:
                set_first_divergence(seq_idx, state, token)
                report["violations"].append(
                    "invalid_transition:"
                    f"{state}->{token}:seq_idx={seq_idx}:"
                    f"expected={','.join(expected_for(state, args.mode))}"
                )
                return fail(out_path, report)

        elif state == S4:
            set_first_divergence(seq_idx, state, token)
            report["violations"].append(
                f"extra_token_after_accept:{token}:seq_idx={seq_idx}"
            )
            return fail(out_path, report)

    if state != S4:
        set_first_divergence(len(sequence), state, "EOF")
        report["violations"].append(f"incomplete_state_machine:ended_at={state}")
        return fail(out_path, report)

    return succeed(out_path, report)


if __name__ == "__main__":
    raise SystemExit(main())
