#!/usr/bin/env python3
"""Validate the Phase10-A2 Ring3 executable-leaf runtime rule."""

from __future__ import annotations

import argparse
import json
import re
from pathlib import Path

AYKEN_PTE_ADDR_MASK = 0x000FFFFFFFFFF000
AYKEN_PTE_PRESENT = 1 << 0
AYKEN_PTE_USER = 1 << 2
AYKEN_PTE_NO_EXEC = 1 << 63
USER_TEXT_BASE = 0x400000

WITNESS_RE = re.compile(
    r"P10_TEXT_FRAME_WITNESS "
    r"phase=(?P<phase>\S+) "
    r"root=(?P<root>[0-9A-Fa-f]+) "
    r"pte=(?P<pte>[0-9A-Fa-f]+) "
    r"phys=(?P<phys>[0-9A-Fa-f]+) "
    r"used=(?P<used>\d+) "
    r"lo=(?P<lo>[0-9A-Fa-f]+) "
    r"hi=(?P<hi>[0-9A-Fa-f]+) "
    r"hash=(?P<hash>[0-9A-Fa-f]+)"
)
PROBE_RE = re.compile(
    r"P10_POST_CR3_TEXT_PROBE "
    r"CR3=(?P<cr3>[0-9A-Fa-f]+) "
    r"RIP=(?P<rip>[0-9A-Fa-f]+) "
    r"Q=(?P<q>[0-9A-Fa-f]+)"
)
USER_MARKER = "P10_RING3_USER_CODE"


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Validate authoritative executable-leaf runtime evidence for Phase10-A2."
    )
    parser.add_argument("--log", required=True, help="Input authoritative debugcon log.")
    parser.add_argument("--out", required=True, help="Output report.json path.")
    return parser.parse_args()


def parse_hex(raw: str) -> int:
    return int(raw, 16)


def write_json(path: Path, payload: object) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("w", encoding="utf-8") as handle:
        json.dump(payload, handle, indent=2, sort_keys=True)
        handle.write("\n")


def collect_rows(log_text: str) -> tuple[list[dict], list[dict], list[dict]]:
    witnesses: list[dict] = []
    probes: list[dict] = []
    users: list[dict] = []

    for line_no, line in enumerate(log_text.splitlines(), start=1):
        witness_match = WITNESS_RE.search(line)
        if witness_match is not None:
            row = {"line": line_no, "raw": line}
            for key, value in witness_match.groupdict().items():
                if key == "phase":
                    row[key] = value
                elif key == "used":
                    row[key] = int(value, 10)
                else:
                    row[key] = parse_hex(value)
            witnesses.append(row)

        probe_match = PROBE_RE.search(line)
        if probe_match is not None:
            row = {"line": line_no, "raw": line}
            for key, value in probe_match.groupdict().items():
                row[key] = parse_hex(value)
            probes.append(row)

        if USER_MARKER in line:
            users.append({"line": line_no, "raw": line})

    return witnesses, probes, users


def latest_before(rows: list[dict], line_limit: int) -> dict | None:
    chosen = None
    for row in rows:
        if int(row["line"]) <= line_limit:
            chosen = row
    return chosen


def witness_violations(witness: dict | None) -> list[str]:
    violations: list[str] = []
    if witness is None:
        violations.append("missing_required:P10_TEXT_FRAME_WITNESS:phase=pre_dispatch")
        return violations

    witness_phys = int(witness["phys"])
    witness_pte = int(witness["pte"])
    if witness_phys == 0:
        violations.append("text_leaf_phys_zero")
    if (witness_pte & AYKEN_PTE_ADDR_MASK) != witness_phys:
        violations.append(
            "text_leaf_pte_phys_mismatch:"
            f"pte_phys=0x{(witness_pte & AYKEN_PTE_ADDR_MASK):016X}:"
            f"witness_phys=0x{witness_phys:016X}"
        )
    if (witness_pte & AYKEN_PTE_PRESENT) == 0:
        violations.append("text_leaf_pte_not_present")
    if (witness_pte & AYKEN_PTE_USER) == 0:
        violations.append("text_leaf_pte_not_user")
    if (witness_pte & AYKEN_PTE_NO_EXEC) != 0:
        violations.append("text_leaf_pte_noexec")
    if int(witness["used"]) != 1:
        violations.append(f"text_leaf_not_allocated:used={int(witness['used'])}")
    return violations


def pair_violations(witness: dict | None, probe: dict | None, user_line: int) -> list[str]:
    violations: list[str] = []
    if probe is None:
        violations.append("missing_required:P10_POST_CR3_TEXT_PROBE")
        return violations
    if witness is None:
        violations.append("missing_required:P10_TEXT_FRAME_WITNESS:phase=pre_dispatch")
        return violations

    if int(probe["rip"]) != USER_TEXT_BASE:
        violations.append(
            f"post_cr3_probe_unexpected_rip:rip=0x{int(probe['rip']):016X}"
        )
    if int(probe["line"]) <= int(witness["line"]):
        violations.append(
            "witness_probe_order_invalid:"
            f"witness_line={int(witness['line'])}:probe_line={int(probe['line'])}"
        )
    if int(probe["cr3"]) != int(witness["root"]):
        violations.append(
            "witness_probe_root_mismatch:"
            f"witness_root=0x{int(witness['root']):016X}:"
            f"probe_cr3=0x{int(probe['cr3']):016X}"
        )
    if int(probe["q"]) != int(witness["lo"]):
        violations.append(
            "witness_probe_qword_mismatch:"
            f"witness_lo=0x{int(witness['lo']):016X}:"
            f"probe_q=0x{int(probe['q']):016X}"
        )
    if user_line >= 0 and int(probe["line"]) >= user_line:
        violations.append(
            f"user_marker_not_after_probe:probe_line={int(probe['line'])}:user_line={user_line}"
        )
    return violations


def validate(log_text: str) -> dict:
    violations: list[str] = []
    witnesses, probes, users = collect_rows(log_text)

    user_line = int(users[0]["line"]) if users else -1
    pre_dispatch_witnesses = [row for row in witnesses if row.get("phase") == "pre_dispatch"]
    chosen_probe = None
    chosen_witness = None
    fallback_probe = latest_before(probes, user_line if user_line >= 0 else 1 << 30)
    fallback_witness = latest_before(
        pre_dispatch_witnesses,
        int(fallback_probe["line"]) if fallback_probe is not None else (user_line if user_line >= 0 else 1 << 30),
    )

    for probe in probes:
        if user_line >= 0 and int(probe["line"]) >= user_line:
            continue
        witness = latest_before(pre_dispatch_witnesses, int(probe["line"]))
        if witness is None:
            continue
        if witness_violations(witness):
            if chosen_probe is None:
                chosen_probe = probe
                chosen_witness = witness
            continue
        if not pair_violations(witness, probe, user_line):
            chosen_probe = probe
            chosen_witness = witness
            break
        if chosen_probe is None:
            chosen_probe = probe
            chosen_witness = witness

    if chosen_probe is None:
        chosen_probe = fallback_probe
    if chosen_witness is None:
        chosen_witness = fallback_witness

    if not users:
        violations.append("missing_required:P10_RING3_USER_CODE")
    violations.extend(witness_violations(chosen_witness))
    violations.extend(pair_violations(chosen_witness, chosen_probe, user_line))

    return {
        "gate": "ring3-execution-phase10a2-runtime-rule",
        "verdict": "PASS" if not violations else "FAIL",
        "violations": violations,
        "violations_count": len(violations),
        "authoritative_marker_chain": {
            "witness_marker": "P10_TEXT_FRAME_WITNESS",
            "probe_marker": "P10_POST_CR3_TEXT_PROBE",
            "user_marker": USER_MARKER,
            "kernel_cr3_safe_walk_required": True,
            "walk_diagnostics_authoritative": False,
            "allocator_class_authority": "source_guard",
        },
        "selected_pre_dispatch_witness": chosen_witness,
        "selected_post_cr3_probe": chosen_probe,
        "selected_user_marker": users[0] if users else None,
        "observed_counts": {
            "pre_dispatch_witness": len(pre_dispatch_witnesses),
            "post_cr3_probe": len(probes),
            "user_marker": len(users),
        },
    }


def main() -> int:
    args = parse_args()
    log_path = Path(args.log)
    out_path = Path(args.out)

    if not log_path.is_file():
        raise SystemExit(f"missing input log: {log_path}")

    report = validate(log_path.read_text(encoding="utf-8", errors="replace"))
    write_json(out_path, report)
    return 0 if report["violations_count"] == 0 else 2


if __name__ == "__main__":
    raise SystemExit(main())
