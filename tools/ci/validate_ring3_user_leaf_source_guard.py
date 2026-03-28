#!/usr/bin/env python3
"""Validate Ring3 executable-leaf source-path and diagnostic guardrails."""

from __future__ import annotations

import argparse
import json
from dataclasses import dataclass
from pathlib import Path


@dataclass
class FunctionBlock:
    name: str
    start_line: int
    lines: list[tuple[int, str]]

    @property
    def text(self) -> str:
        return "\n".join(line for _, line in self.lines)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Validate Ring3 executable-leaf source-path enforcement."
    )
    parser.add_argument("--source-root", required=True, help="Repository root.")
    parser.add_argument("--out-report", required=True, help="Output report.json path.")
    parser.add_argument("--violations-out", required=True, help="Output violations.txt path.")
    return parser.parse_args()


def write_json(path: Path, payload: object) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("w", encoding="utf-8") as handle:
        json.dump(payload, handle, indent=2, sort_keys=True)
        handle.write("\n")


def write_violations(path: Path, violations: list[str]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text("\n".join(violations) + ("\n" if violations else ""), encoding="utf-8")


def parse_functions(text: str) -> dict[str, FunctionBlock]:
    functions: dict[str, FunctionBlock] = {}
    current_name: str | None = None
    current_start = 0
    current_lines: list[tuple[int, str]] = []
    brace_depth = 0
    saw_open_brace = False

    for line_number, line in enumerate(text.splitlines(), start=1):
        if current_name is None:
            stripped = line.strip()
            if "(" not in line or ";" in stripped:
                continue
            prefix = line.split("(", 1)[0].strip()
            if not prefix:
                continue
            current_name = prefix.split()[-1].lstrip("*")
            current_start = line_number
            current_lines = [(line_number, line)]
            brace_depth = line.count("{") - line.count("}")
            saw_open_brace = "{" in line
            if saw_open_brace and brace_depth == 0:
                functions[current_name] = FunctionBlock(current_name, current_start, current_lines)
                current_name = None
            continue

        current_lines.append((line_number, line))
        brace_depth += line.count("{") - line.count("}")
        saw_open_brace = saw_open_brace or ("{" in line)
        if saw_open_brace and brace_depth == 0:
            functions[current_name] = FunctionBlock(current_name, current_start, current_lines)
            current_name = None
            current_lines = []
            brace_depth = 0
            saw_open_brace = False

    return functions


def require_token(
    violations: list[str],
    block: FunctionBlock | None,
    token: str,
    violation: str,
) -> None:
    if block is None or token not in block.text:
        violations.append(violation)


def forbid_token(
    violations: list[str],
    block: FunctionBlock | None,
    token: str,
    violation: str,
) -> None:
    if block is not None and token in block.text:
        violations.append(violation)


def validate(source_root: Path) -> dict:
    violations: list[str] = []
    proc_path = source_root / "kernel/proc/proc.c"
    sched_path = source_root / "kernel/sched/sched.c"

    if not proc_path.is_file():
        violations.append(f"missing_required_source:{proc_path}")
        proc_functions: dict[str, FunctionBlock] = {}
    else:
        proc_functions = parse_functions(proc_path.read_text(encoding="utf-8"))

    if not sched_path.is_file():
        violations.append(f"missing_required_source:{sched_path}")
        sched_functions: dict[str, FunctionBlock] = {}
    else:
        sched_functions = parse_functions(sched_path.read_text(encoding="utf-8"))

    image_alloc = proc_functions.get("proc_alloc_user_image_frame")
    flat_load = proc_functions.get("load_flat_image")
    elf_load = proc_functions.get("load_elf_image")
    walk_snapshot = sched_functions.get("sched_capture_walk_snapshot")

    if image_alloc is None:
        violations.append("missing_required_function:proc_alloc_user_image_frame")
    else:
        require_token(
            violations,
            image_alloc,
            "phys_alloc_frame_high(",
            "user_image_allocator_not_high_phys",
        )
        forbid_token(
            violations,
            image_alloc,
            "phys_alloc_frame(",
            "forbidden_low_phys_allocator_in_user_image_allocator",
        )

    for name, block in (("load_flat_image", flat_load), ("load_elf_image", elf_load)):
        if block is None:
            violations.append(f"missing_required_function:{name}")
            continue
        require_token(
            violations,
            block,
            "proc_alloc_user_image_frame(",
            f"missing_user_image_allocator_call:{name}",
        )
        forbid_token(
            violations,
            block,
            "phys_alloc_frame(",
            f"forbidden_low_phys_allocator_call:{name}",
        )

    if walk_snapshot is None:
        violations.append("missing_required_function:sched_capture_walk_snapshot")
    else:
        require_token(
            violations,
            walk_snapshot,
            "paging_get_kernel_pml4_phys()",
            "walk_snapshot_not_kernel_cr3_safe:missing_kernel_root_lookup",
        )
        if walk_snapshot.text.count('mov %0, %%cr3') < 2:
            violations.append("walk_snapshot_not_kernel_cr3_safe:missing_cr3_switch_restore")
        require_token(
            violations,
            walk_snapshot,
            'pushfq; popq',
            "walk_snapshot_not_kernel_cr3_safe:missing_rflags_save",
        )
        require_token(
            violations,
            walk_snapshot,
            "__asm__ volatile(\"sti\")",
            "walk_snapshot_not_kernel_cr3_safe:missing_rflags_restore",
        )

    return {
        "gate": "ring3-execution-phase10a2-source-guard",
        "verdict": "PASS" if not violations else "FAIL",
        "violations": violations,
        "violations_count": len(violations),
        "checked_files": {
            "proc": str(proc_path),
            "sched": str(sched_path),
        },
        "checked_functions": {
            "proc_alloc_user_image_frame": image_alloc.start_line if image_alloc else 0,
            "load_flat_image": flat_load.start_line if flat_load else 0,
            "load_elf_image": elf_load.start_line if elf_load else 0,
            "sched_capture_walk_snapshot": walk_snapshot.start_line if walk_snapshot else 0,
        },
    }


def main() -> int:
    args = parse_args()
    source_root = Path(args.source_root).resolve()
    report_path = Path(args.out_report)
    violations_path = Path(args.violations_out)

    report = validate(source_root)
    write_json(report_path, report)
    write_violations(violations_path, list(report["violations"]))
    return 0 if report["violations_count"] == 0 else 2


if __name__ == "__main__":
    raise SystemExit(main())
