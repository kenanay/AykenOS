#!/usr/bin/env python3
"""
Generate a linker version-script from the constitutional Ring0 symbol whitelist.

This enforces exported-symbol policy at link time:
  - symbols matching whitelist patterns are kept global
  - all other symbols are localized via `local: *;`
"""

from __future__ import annotations

import argparse
import re
import subprocess
import sys
from pathlib import Path


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Generate linker export map from Ring0 symbol whitelist."
    )
    parser.add_argument("--whitelist", required=True, help="Regex whitelist file path.")
    parser.add_argument("--output", required=True, help="Output map file path.")
    parser.add_argument(
        "--nm",
        default="nm",
        help="nm executable to use (default: nm).",
    )
    parser.add_argument(
        "--objects",
        nargs="+",
        required=True,
        help="Kernel object files to scan.",
    )
    return parser.parse_args()


def load_whitelist(path: Path) -> list[re.Pattern[str]]:
    if not path.is_file():
        raise FileNotFoundError(f"Whitelist file not found: {path}")
    rows: list[re.Pattern[str]] = []
    for raw in path.read_text(encoding="utf-8", errors="replace").splitlines():
        line = raw.strip()
        if not line or line.startswith("#"):
            continue
        rows.append(re.compile(line))
    if not rows:
        raise ValueError(f"Whitelist is empty: {path}")
    return rows


def collect_defined_symbols(nm_bin: str, objects: list[Path]) -> list[str]:
    cmd = [nm_bin, "-g", "--defined-only", *[str(p) for p in objects]]
    proc = subprocess.run(cmd, capture_output=True, text=True, check=False)
    if proc.returncode != 0:
        raise RuntimeError(
            f"nm failed (rc={proc.returncode}): {proc.stderr.strip() or proc.stdout.strip()}"
        )

    out: set[str] = set()
    for raw in proc.stdout.splitlines():
        line = raw.strip()
        if not line:
            continue
        # nm prints "file.o:" headers when multiple objects are scanned.
        if line.endswith(":"):
            continue
        parts = line.split()
        if len(parts) < 2:
            continue
        name = parts[-1]
        if not re.match(r"^[A-Za-z_][A-Za-z0-9_]*$", name):
            continue
        if name.startswith("__") or name.startswith(".L") or name.startswith("$"):
            continue
        out.add(name)
    return sorted(out)


def filter_exported(symbols: list[str], whitelist: list[re.Pattern[str]]) -> list[str]:
    return sorted({sym for sym in symbols if any(p.search(sym) for p in whitelist)})


def render_map(exported: list[str], source_whitelist: Path) -> str:
    lines = [
        "/* AUTO-GENERATED. DO NOT EDIT. */",
        f"/* Source whitelist: {source_whitelist.as_posix()} */",
        "AYKEN_KERNEL_EXPORTS {",
        "  global:",
    ]
    for sym in exported:
        lines.append(f"    {sym};")
    lines.extend(
        [
            "  local:",
            "    *;",
            "};",
            "",
        ]
    )
    return "\n".join(lines)


def main() -> int:
    args = parse_args()
    whitelist_path = Path(args.whitelist)
    output_path = Path(args.output)
    object_paths = [Path(p) for p in args.objects]

    missing = [str(p) for p in object_paths if not p.is_file()]
    if missing:
        print("ERROR: object file(s) missing for export map generation:", file=sys.stderr)
        for item in missing:
            print(f"  - {item}", file=sys.stderr)
        return 3

    whitelist = load_whitelist(whitelist_path)
    symbols = collect_defined_symbols(args.nm, object_paths)
    exported = filter_exported(symbols, whitelist)
    if not exported:
        print(
            "ERROR: no exported symbols matched whitelist; refusing empty export map",
            file=sys.stderr,
        )
        return 4

    output_path.parent.mkdir(parents=True, exist_ok=True)
    output_path.write_text(render_map(exported, whitelist_path), encoding="utf-8")
    print(f"generated export map: {output_path} ({len(exported)} symbols)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
