#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CI_TOOLS="${ROOT}/tools/ci"
source "${CI_TOOLS}/lib.sh"

usage() {
  cat <<'USAGE'
Usage:
  scripts/ci/gate_constitutional.sh --evidence-dir evidence/run-<id>/gates/constitutional
    [--strict|--no-strict]
    [--kernel-elf kernel.elf]
    [--kernel-whitelist scripts/ci/constitutional-ring0-whitelist.regex]
    [--ring0-symbol-whitelist scripts/ci/constitutional-ring0-symbol-whitelist.regex]
    [--source-deny scripts/ci/constitutional-source-deny.regex]
    [--source-allow scripts/ci/constitutional-source-allow.regex]
    [--ahs-config _ayken/steering/AHS_CONFIG.toml]
    [--non-overridable _ayken/steering/NON_OVERRIDABLE.md]
    [--waiver-dir docs/waivers]

Exit codes:
  0: pass
  2: constitutional violations detected
  3: usage/tooling error
USAGE
}

EVIDENCE_DIR=""
STRICT="${STRICT:-1}"
KERNEL_ELF="${ROOT}/kernel.elf"
KERNEL_WHITELIST="${ROOT}/scripts/ci/constitutional-ring0-whitelist.regex"
RING0_SYMBOL_WHITELIST="${ROOT}/scripts/ci/constitutional-ring0-symbol-whitelist.regex"
SOURCE_DENY="${ROOT}/scripts/ci/constitutional-source-deny.regex"
SOURCE_ALLOW="${ROOT}/scripts/ci/constitutional-source-allow.regex"
AHS_CONFIG="${ROOT}/_ayken/steering/AHS_CONFIG.toml"
NON_OVERRIDABLE="${ROOT}/_ayken/steering/NON_OVERRIDABLE.md"
WAIVER_DIR="${ROOT}/docs/waivers"
SYSCALL_H="${ROOT}/kernel/sys/syscall_v2.h"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --evidence-dir)
      EVIDENCE_DIR="$2"
      shift 2
      ;;
    --kernel-whitelist)
      KERNEL_WHITELIST="$2"
      shift 2
      ;;
    --ring0-symbol-whitelist)
      RING0_SYMBOL_WHITELIST="$2"
      shift 2
      ;;
    --kernel-elf)
      KERNEL_ELF="$2"
      shift 2
      ;;
    --strict)
      STRICT=1
      shift 1
      ;;
    --no-strict)
      STRICT=0
      shift 1
      ;;
    --source-deny)
      SOURCE_DENY="$2"
      shift 2
      ;;
    --source-allow)
      SOURCE_ALLOW="$2"
      shift 2
      ;;
    --ahs-config)
      AHS_CONFIG="$2"
      shift 2
      ;;
    --non-overridable)
      NON_OVERRIDABLE="$2"
      shift 2
      ;;
    --waiver-dir)
      WAIVER_DIR="$2"
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "Unknown arg: $1" >&2
      usage
      exit 3
      ;;
  esac
done

if [[ -z "${EVIDENCE_DIR}" ]]; then
  usage
  exit 3
fi

if ! command -v git >/dev/null 2>&1 || ! command -v python3 >/dev/null 2>&1; then
  echo "ERROR: required tools missing (git/python3)" >&2
  exit 3
fi
if [[ "${STRICT}" == "1" ]] && ! command -v nm >/dev/null 2>&1; then
  echo "ERROR: required tools missing (nm for strict symbol scan)" >&2
  exit 3
fi

mkdir -p "${EVIDENCE_DIR}"
TRACKED_KERNEL="${EVIDENCE_DIR}/tracked-kernel-files.txt"
RING0_WHITELIST_VIOLATIONS="${EVIDENCE_DIR}/ring0-whitelist-violations.txt"
RING0_SYMBOLS="${EVIDENCE_DIR}/ring0-symbols.txt"
RING0_SYMBOL_VIOLATIONS="${EVIDENCE_DIR}/ring0-symbol-violations.txt"
SOURCE_HITS="${EVIDENCE_DIR}/source-deny-hits.txt"
WAIVER_AUDIT="${EVIDENCE_DIR}/waiver-audit.txt"
CONTRACT_TXT="${EVIDENCE_DIR}/contract.txt"
AHS_CHECK_TXT="${EVIDENCE_DIR}/ahs-check.txt"
NON_OVERRIDABLE_CHECK_TXT="${EVIDENCE_DIR}/non-overridable-check.txt"
VIOLATIONS_TXT="${EVIDENCE_DIR}/violations.txt"
META_TXT="${EVIDENCE_DIR}/meta.txt"
REPORT_JSON="${EVIDENCE_DIR}/report.json"

NOW="$(ci_now_utc)"
GIT_SHA="$(git -C "${ROOT}" rev-parse HEAD 2>/dev/null || echo NO_GIT)"

ROOT_ENV="${ROOT}" \
EVIDENCE_DIR_ENV="${EVIDENCE_DIR}" \
STRICT_ENV="${STRICT}" \
KERNEL_ELF_ENV="${KERNEL_ELF}" \
KERNEL_WHITELIST_ENV="${KERNEL_WHITELIST}" \
RING0_SYMBOL_WHITELIST_ENV="${RING0_SYMBOL_WHITELIST}" \
SOURCE_DENY_ENV="${SOURCE_DENY}" \
SOURCE_ALLOW_ENV="${SOURCE_ALLOW}" \
AHS_CONFIG_ENV="${AHS_CONFIG}" \
NON_OVERRIDABLE_ENV="${NON_OVERRIDABLE}" \
WAIVER_DIR_ENV="${WAIVER_DIR}" \
SYSCALL_H_ENV="${SYSCALL_H}" \
TRACKED_KERNEL_ENV="${TRACKED_KERNEL}" \
RING0_WHITELIST_VIOLATIONS_ENV="${RING0_WHITELIST_VIOLATIONS}" \
RING0_SYMBOLS_ENV="${RING0_SYMBOLS}" \
RING0_SYMBOL_VIOLATIONS_ENV="${RING0_SYMBOL_VIOLATIONS}" \
SOURCE_HITS_ENV="${SOURCE_HITS}" \
WAIVER_AUDIT_ENV="${WAIVER_AUDIT}" \
CONTRACT_TXT_ENV="${CONTRACT_TXT}" \
AHS_CHECK_TXT_ENV="${AHS_CHECK_TXT}" \
NON_OVERRIDABLE_CHECK_TXT_ENV="${NON_OVERRIDABLE_CHECK_TXT}" \
VIOLATIONS_TXT_ENV="${VIOLATIONS_TXT}" \
META_TXT_ENV="${META_TXT}" \
REPORT_JSON_ENV="${REPORT_JSON}" \
NOW_ENV="${NOW}" \
GIT_SHA_ENV="${GIT_SHA}" \
python3 - <<'PY'
import ast
import json
import os
import re
import subprocess
from datetime import date
from pathlib import Path

ROOT = Path(os.environ["ROOT_ENV"])
EVIDENCE_DIR = Path(os.environ["EVIDENCE_DIR_ENV"])
STRICT_MODE = os.environ.get("STRICT_ENV", "1") == "1"
KERNEL_ELF = Path(os.environ["KERNEL_ELF_ENV"])
KERNEL_WHITELIST = Path(os.environ["KERNEL_WHITELIST_ENV"])
RING0_SYMBOL_WHITELIST = Path(os.environ["RING0_SYMBOL_WHITELIST_ENV"])
SOURCE_DENY = Path(os.environ["SOURCE_DENY_ENV"])
SOURCE_ALLOW = Path(os.environ["SOURCE_ALLOW_ENV"])
AHS_CONFIG = Path(os.environ["AHS_CONFIG_ENV"])
NON_OVERRIDABLE = Path(os.environ["NON_OVERRIDABLE_ENV"])
WAIVER_DIR = Path(os.environ["WAIVER_DIR_ENV"])
SYSCALL_H = Path(os.environ["SYSCALL_H_ENV"])

TRACKED_KERNEL = Path(os.environ["TRACKED_KERNEL_ENV"])
RING0_WHITELIST_VIOLATIONS = Path(os.environ["RING0_WHITELIST_VIOLATIONS_ENV"])
RING0_SYMBOLS = Path(os.environ["RING0_SYMBOLS_ENV"])
RING0_SYMBOL_VIOLATIONS = Path(os.environ["RING0_SYMBOL_VIOLATIONS_ENV"])
SOURCE_HITS = Path(os.environ["SOURCE_HITS_ENV"])
WAIVER_AUDIT = Path(os.environ["WAIVER_AUDIT_ENV"])
CONTRACT_TXT = Path(os.environ["CONTRACT_TXT_ENV"])
AHS_CHECK_TXT = Path(os.environ["AHS_CHECK_TXT_ENV"])
NON_OVERRIDABLE_CHECK_TXT = Path(os.environ["NON_OVERRIDABLE_CHECK_TXT_ENV"])
VIOLATIONS_TXT = Path(os.environ["VIOLATIONS_TXT_ENV"])
META_TXT = Path(os.environ["META_TXT_ENV"])
REPORT_JSON = Path(os.environ["REPORT_JSON_ENV"])

NOW = os.environ["NOW_ENV"]
GIT_SHA = os.environ["GIT_SHA_ENV"]

for path in (
    TRACKED_KERNEL,
    RING0_WHITELIST_VIOLATIONS,
    RING0_SYMBOLS,
    RING0_SYMBOL_VIOLATIONS,
    SOURCE_HITS,
    WAIVER_AUDIT,
    CONTRACT_TXT,
    AHS_CHECK_TXT,
    NON_OVERRIDABLE_CHECK_TXT,
    VIOLATIONS_TXT,
):
    path.write_text("", encoding="utf-8")

violations = []
ring0_whitelist_hits = []
ring0_symbol_list = []
ring0_symbol_hits = []
source_hits = []
waiver_audit_rows = []
waiver_violations_count = 0
non_overridable_missing_count = 0


def add_violation(kind: str, detail: str) -> None:
    violations.append(f"{kind}:{detail}")


def load_regex_lines(path: Path):
    if not path.exists():
        return None
    rows = []
    for raw in path.read_text(encoding="utf-8", errors="replace").splitlines():
        line = raw.strip()
        if not line or line.startswith("#"):
            continue
        rows.append(line)
    return rows


def parse_allow_rules(path: Path):
    rules = []
    rows = load_regex_lines(path)
    if rows is None:
        return None
    for line in rows:
        if ":" in line:
            file_rgx, line_rgx = line.split(":", 1)
            rules.append(("scoped", file_rgx.strip(), line_rgx.strip()))
        else:
            rules.append(("global", line))
    return rules


def allow_matches(allow_rules, rel_path: str, line: str):
    if not allow_rules:
        return False
    for rule in allow_rules:
        if rule[0] == "global":
            if re.search(rule[1], line):
                return True
        else:
            _, file_rgx, line_rgx = rule
            if re.search(file_rgx, rel_path) and re.search(line_rgx, line):
                return True
    return False


try:
    tracked_kernel = subprocess.check_output(
        ["git", "-C", str(ROOT), "ls-files", "kernel"], text=True
    ).splitlines()
except subprocess.CalledProcessError:
    add_violation("git_ls_files_failed", "kernel")
    tracked_kernel = []

TRACKED_KERNEL.write_text(
    "\n".join(tracked_kernel) + ("\n" if tracked_kernel else ""), encoding="utf-8"
)

# 1) Ring0 tracked-path whitelist enforcement.
whitelist_rows = load_regex_lines(KERNEL_WHITELIST)
if whitelist_rows is None:
    add_violation("missing_file", str(KERNEL_WHITELIST))
else:
    whitelist = [re.compile(pat) for pat in whitelist_rows]
    for rel in tracked_kernel:
        if not rel.startswith("kernel/"):
            continue
        if not any(rule.search(rel) for rule in whitelist):
            ring0_whitelist_hits.append(rel)
            add_violation("ring0_whitelist", rel)

if ring0_whitelist_hits:
    RING0_WHITELIST_VIOLATIONS.write_text(
        "\n".join(ring0_whitelist_hits) + "\n", encoding="utf-8"
    )

# 1b) Ring0 exported symbol whitelist enforcement (strict mode).
if STRICT_MODE:
    symbol_rows = load_regex_lines(RING0_SYMBOL_WHITELIST)
    if symbol_rows is None:
        add_violation("missing_file", str(RING0_SYMBOL_WHITELIST))
    if not KERNEL_ELF.exists():
        add_violation("missing_file", str(KERNEL_ELF))
    if symbol_rows is not None and KERNEL_ELF.exists():
        symbol_rules = [re.compile(pat) for pat in symbol_rows]
        try:
            nm_lines = subprocess.check_output(
                ["nm", "-g", "--defined-only", str(KERNEL_ELF)], text=True
            ).splitlines()
        except Exception as exc:
            add_violation("ring0_symbol_scan_failed", type(exc).__name__)
            nm_lines = []

        ignore_linker_names = {
            "_bss_start",
            "_bss_end",
            "_text_start",
            "_text_end",
            "_data_start",
            "_data_end",
            "_rodata_start",
            "_rodata_end",
            "_kernel_end",
            "_cpu_start",
            "_cpu_end",
            "KERNEL_PHYS_BASE",
            "KERNEL_VIRT_BASE",
            "PHYS_TO_VIRT_OFFSET",
        }

        for row in nm_lines:
            parts = row.strip().split()
            if len(parts) < 2:
                continue
            name = parts[-1]
            if not name:
                continue
            if name in ignore_linker_names:
                continue
            if name.startswith("__") or name.startswith(".L") or name.startswith("$"):
                continue
            ring0_symbol_list.append(name)

        ring0_symbol_list = sorted(set(ring0_symbol_list))
        if ring0_symbol_list:
            RING0_SYMBOLS.write_text("\n".join(ring0_symbol_list) + "\n", encoding="utf-8")

        for sym in ring0_symbol_list:
            if not any(rule.search(sym) for rule in symbol_rules):
                ring0_symbol_hits.append(sym)
                add_violation("ring0_symbol_whitelist", sym)

        if ring0_symbol_hits:
            RING0_SYMBOL_VIOLATIONS.write_text(
                "\n".join(ring0_symbol_hits) + "\n", encoding="utf-8"
            )

# 2) Kernel source deny/allow scan.
deny_rows = load_regex_lines(SOURCE_DENY)
allow_rules = parse_allow_rules(SOURCE_ALLOW)
if deny_rows is None:
    add_violation("missing_file", str(SOURCE_DENY))
else:
    deny_patterns = [re.compile(pat) for pat in deny_rows]
    src_ext = {".c", ".h", ".S", ".s", ".asm", ".inc", ".ld"}
    for rel in tracked_kernel:
        # Validation/test fixtures are reviewed separately; strict source deny targets default path.
        if re.search(r"(?:^|/).*(?:_test|validation_test)\.c$", rel):
            continue
        p = ROOT / rel
        if p.suffix not in src_ext or not p.is_file():
            continue
        try:
            lines = p.read_text(encoding="utf-8", errors="replace").splitlines()
        except Exception as exc:
            add_violation("source_read_error", f"{rel}:{type(exc).__name__}")
            continue
        for idx, line in enumerate(lines, 1):
            stripped = line.strip()
            if (
                stripped.startswith("//")
                or stripped.startswith("/*")
                or stripped.startswith("*")
                or stripped.startswith(";")
            ):
                continue
            # Remove inline comments and string/char literals to reduce false positives.
            scan_line = re.sub(r"//.*$", "", line)
            scan_line = re.sub(r'"(?:\\.|[^"\\])*"', '""', scan_line)
            scan_line = re.sub(r"'(?:\\.|[^'\\])*'", "''", scan_line)
            for pat in deny_patterns:
                if pat.search(scan_line):
                    if allow_matches(allow_rules, rel, scan_line):
                        continue
                    snippet = line.strip()
                    if len(snippet) > 160:
                        snippet = snippet[:157] + "..."
                    hit = f"{rel}:{idx}:pattern={pat.pattern}:line={snippet}"
                    source_hits.append(hit)
                    add_violation("source_deny", hit)
                    break

if source_hits:
    SOURCE_HITS.write_text("\n".join(source_hits) + "\n", encoding="utf-8")

# 3) Syscall contract lock (must remain frozen).
contract_macros = {}
contract = {}
if not SYSCALL_H.exists():
    add_violation("missing_file", str(SYSCALL_H))
else:
    macro_re = re.compile(r"^\s*#define\s+(SYS_V2_[A-Za-z0-9_]+)\s+(.+)$")
    for raw in SYSCALL_H.read_text(encoding="utf-8", errors="replace").splitlines():
        m = macro_re.match(raw)
        if not m:
            continue
        expr = m.group(2).split("//", 1)[0].split("/*", 1)[0].strip()
        if expr:
            contract_macros[m.group(1)] = expr

    required = [
        "SYS_V2_BASE",
        "SYS_V2_MAX_INDEX",
        "SYS_V2_NR",
        "SYS_V2_LAST",
        "SYS_V2_MAX_SYSCALL",
        "SYS_V2_DEBUG_PUTCHAR",
    ]
    missing = [k for k in required if k not in contract_macros]
    if missing:
        add_violation("syscall_contract_missing_macro", ",".join(missing))
    else:
        cache = {}
        stack = set()

        def assert_ast_safe(node):
            allowed = (
                ast.Expression,
                ast.BinOp,
                ast.UnaryOp,
                ast.Constant,
                ast.Add,
                ast.Sub,
                ast.Mult,
                ast.Div,
                ast.FloorDiv,
                ast.Mod,
                ast.UAdd,
                ast.USub,
                ast.Load,
            )
            for n in ast.walk(node):
                if not isinstance(n, allowed):
                    raise ValueError(f"unsupported expression node: {type(n).__name__}")

        def eval_macro(name):
            if name in cache:
                return cache[name]
            if name in stack:
                raise ValueError(f"circular macro dependency: {name}")
            if name not in contract_macros:
                raise ValueError(f"macro not found: {name}")
            stack.add(name)
            expr = contract_macros[name]

            def repl(match):
                token = match.group(0)
                if token in contract_macros:
                    return str(eval_macro(token))
                return token

            expanded = re.sub(r"\b[A-Za-z_][A-Za-z0-9_]*\b", repl, expr)
            if re.search(r"[^0-9+\-*/%() \t]", expanded):
                raise ValueError(f"unsafe expression for {name}: {expanded}")
            node = ast.parse(expanded, mode="eval")
            assert_ast_safe(node)
            value = int(eval(compile(node, "<sys_v2_expr>", "eval"), {"__builtins__": {}}, {}))
            stack.remove(name)
            cache[name] = value
            return value

        try:
            contract = {k: eval_macro(k) for k in required}
        except Exception as exc:
            add_violation("syscall_contract_parse_error", type(exc).__name__)
            contract = {}

        if contract:
            expected = {
                "SYS_V2_BASE": 1000,
                "SYS_V2_MAX_INDEX": 10,
                "SYS_V2_NR": 11,
                "SYS_V2_LAST": 1010,
                "SYS_V2_MAX_SYSCALL": 10,
            }
            for key, exp in expected.items():
                got = contract.get(key)
                if got != exp:
                    add_violation("syscall_contract_value", f"{key}:expected={exp}:actual={got}")
            if contract.get("SYS_V2_DEBUG_PUTCHAR", -1) > contract.get("SYS_V2_MAX_INDEX", -1):
                add_violation("syscall_contract_value", "SYS_V2_DEBUG_PUTCHAR out of range")
            if contract.get("SYS_V2_LAST") != contract.get("SYS_V2_BASE", 0) + contract.get("SYS_V2_MAX_INDEX", 0):
                add_violation("syscall_contract_math", "SYS_V2_LAST mismatch")
            if contract.get("SYS_V2_NR") != contract.get("SYS_V2_MAX_INDEX", -1) + 1:
                add_violation("syscall_contract_math", "SYS_V2_NR mismatch")

CONTRACT_TXT.write_text(
    "".join(f"{k}={v}\n" for k, v in sorted(contract.items())), encoding="utf-8"
)

# 4) AHS threshold guard.
ahs_check_lines = []
if not AHS_CONFIG.exists():
    add_violation("missing_file", str(AHS_CONFIG))
else:
    ahs_text = AHS_CONFIG.read_text(encoding="utf-8", errors="replace")
    checks = {
        "P5_minimum": 95,
        "P4_5_minimum": 90,
        "clean_threshold": 90,
    }
    for key, min_val in checks.items():
        m = re.search(rf"^\s*{re.escape(key)}\s*=\s*([0-9]+)\b", ahs_text, flags=re.MULTILINE)
        if not m:
            ahs_check_lines.append(f"{key}=MISSING")
            add_violation("ahs_config_missing", key)
            continue
        val = int(m.group(1))
        ahs_check_lines.append(f"{key}={val}")
        if val < min_val:
            add_violation("ahs_threshold", f"{key}:required>={min_val}:actual={val}")

AHS_CHECK_TXT.write_text(
    "\n".join(ahs_check_lines) + ("\n" if ahs_check_lines else ""), encoding="utf-8"
)

# 5) NON_OVERRIDABLE integrity check.
non_over_required = [
    "KERNEL.RING0.POLICY",
    "KERNEL.CAPABILITY.BYPASS",
    "SECURITY.BOUNDARY.VIOLATION",
    "CONSTITUTIONAL.ENFORCEMENT.BYPASS",
    "No Waivers",
]
non_over_lines = []
if not NON_OVERRIDABLE.exists():
    add_violation("missing_file", str(NON_OVERRIDABLE))
else:
    txt = NON_OVERRIDABLE.read_text(encoding="utf-8", errors="replace")
    for token in non_over_required:
        present = token.lower() in txt.lower()
        non_over_lines.append(f"{token}={'present' if present else 'missing'}")
        if not present:
            non_overridable_missing_count += 1
            add_violation("non_overridable_missing", token)

NON_OVERRIDABLE_CHECK_TXT.write_text(
    "\n".join(non_over_lines) + ("\n" if non_over_lines else ""), encoding="utf-8"
)

# 6) Waiver policy checks.
waiver_files = []
if not WAIVER_DIR.exists():
    add_violation("missing_dir", str(WAIVER_DIR))
else:
    for p in sorted(WAIVER_DIR.glob("*.md")):
        if p.name in {"README.md", "WAIVER_TEMPLATE.md"}:
            continue
        waiver_files.append(p)

today = date.today()
critical_tokens = [
    "KERNEL.RING0.POLICY",
    "KERNEL.CAPABILITY.BYPASS",
    "SECURITY.BOUNDARY.VIOLATION",
    "CONSTITUTIONAL.ENFORCEMENT.BYPASS",
    "NON_OVERRIDABLE",
]

for wf in waiver_files:
    text = wf.read_text(encoding="utf-8", errors="replace")
    rel = str(wf.relative_to(ROOT))

    exp_match = re.search(r"^\s*-\s*Expiry Date:\s*([0-9]{4}-[0-9]{2}-[0-9]{2})\s*$", text, flags=re.MULTILINE)
    date_match = re.search(r"^\s*-\s*Date:\s*([0-9]{4}-[0-9]{2}-[0-9]{2})\s*$", text, flags=re.MULTILINE)
    issue_match = re.search(r"^\s*-\s*Related Issue:\s*(.+)$", text, flags=re.MULTILINE)

    row = [rel]
    if not exp_match:
        waiver_violations_count += 1
        add_violation("waiver_missing_expiry", rel)
        row.append("expiry=MISSING")
        expiry_date = None
    else:
        row.append(f"expiry={exp_match.group(1)}")
        try:
            y, m, d = map(int, exp_match.group(1).split("-"))
            expiry_date = date(y, m, d)
            if expiry_date < today:
                waiver_violations_count += 1
                add_violation("waiver_expired", f"{rel}:{expiry_date.isoformat()}")
        except Exception:
            waiver_violations_count += 1
            add_violation("waiver_invalid_expiry", rel)
            expiry_date = None

    if not issue_match or not issue_match.group(1).strip():
        waiver_violations_count += 1
        add_violation("waiver_missing_issue", rel)
        row.append("issue=MISSING")
    else:
        row.append("issue=OK")

    if date_match and expiry_date:
        try:
            y, m, d = map(int, date_match.group(1).split("-"))
            start_date = date(y, m, d)
            duration = (expiry_date - start_date).days
            row.append(f"duration_days={duration}")
            if duration > 90:
                waiver_violations_count += 1
                add_violation("waiver_duration_exceeds_90", f"{rel}:{duration}")
        except Exception:
            waiver_violations_count += 1
            add_violation("waiver_invalid_date", rel)
            row.append("duration_days=INVALID")
    else:
        row.append("duration_days=UNKNOWN")

    for token in critical_tokens:
        if token.lower() in text.lower():
            waiver_violations_count += 1
            add_violation("waiver_non_overridable", f"{rel}:{token}")
            row.append("non_overridable_ref=YES")
            break

    waiver_audit_rows.append(" ".join(row))

if waiver_audit_rows:
    WAIVER_AUDIT.write_text("\n".join(waiver_audit_rows) + "\n", encoding="utf-8")

violations.sort()
VIOLATIONS_TXT.write_text(
    "\n".join(violations) + ("\n" if violations else ""), encoding="utf-8"
)

meta = {
    "time_utc": NOW,
    "git_sha": GIT_SHA,
    "strict_mode": "1" if STRICT_MODE else "0",
    "kernel_elf": str(KERNEL_ELF),
    "kernel_whitelist_file": str(KERNEL_WHITELIST),
    "ring0_symbol_whitelist_file": str(RING0_SYMBOL_WHITELIST),
    "source_deny_file": str(SOURCE_DENY),
    "source_allow_file": str(SOURCE_ALLOW),
    "ahs_config_file": str(AHS_CONFIG),
    "non_overridable_file": str(NON_OVERRIDABLE),
    "waiver_dir": str(WAIVER_DIR),
    "tracked_kernel_file_count": len(tracked_kernel),
    "ring0_whitelist_violations_count": len(ring0_whitelist_hits),
    "ring0_symbol_count": len(ring0_symbol_list),
    "ring0_symbol_violations_count": len(ring0_symbol_hits),
    "source_deny_hits_count": len(source_hits),
    "waiver_file_count": len(waiver_files),
    "waiver_violations_count": waiver_violations_count,
    "non_overridable_missing_count": non_overridable_missing_count,
    "violations_count": len(violations),
}

META_TXT.write_text(
    "".join(f"{k}={v}\n" for k, v in meta.items()), encoding="utf-8"
)

report = {
    "gate": "constitutional",
    "verdict": "PASS" if not violations else "FAIL",
    "violations_count": len(violations),
    "meta": meta,
    "contract": contract,
    "checks": {
        "ring0_whitelist": "PASS" if not ring0_whitelist_hits else "FAIL",
        "ring0_symbol_whitelist": (
            "PASS" if not ring0_symbol_hits else "FAIL"
        ) if STRICT_MODE else "SKIP",
        "source_deny_scan": "PASS" if not source_hits else "FAIL",
        "syscall_contract": "PASS" if contract and not any(v.startswith("syscall_contract_") for v in violations) else "FAIL",
        "ahs_thresholds": "PASS" if not any(v.startswith("ahs_") for v in violations) else "FAIL",
        "non_overridable_integrity": "PASS" if non_overridable_missing_count == 0 else "FAIL",
        "waiver_policy": "PASS" if waiver_violations_count == 0 else "FAIL",
    },
    "violations": violations,
    "ring0_whitelist_violations": ring0_whitelist_hits,
    "ring0_symbol_violations": ring0_symbol_hits,
    "source_deny_hits": source_hits,
    "waiver_audit": waiver_audit_rows,
}

REPORT_JSON.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n", encoding="utf-8")

raise SystemExit(0 if not violations else 2)
PY

VIOLATIONS_COUNT="$(wc -l < "${VIOLATIONS_TXT}" | tr -d ' ' || echo 0)"
if [[ "${VIOLATIONS_COUNT}" -gt 0 ]]; then
  echo "constitutional: FAIL (${VIOLATIONS_COUNT} violations)"
  echo "See: ${VIOLATIONS_TXT}"
  exit 2
fi

echo "constitutional: PASS"
exit 0
