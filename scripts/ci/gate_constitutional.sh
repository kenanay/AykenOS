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
    [--non-overridable _ayken/steering/NON_OVERRIDABLE.md]
    [--syscall-h kernel/sys/syscall_v2.h]
    [--makefile Makefile]
    [--sched-h kernel/sched/sched.h]
    [--arch-freeze ARCHITECTURE_FREEZE.md]
    [--sched-arb-decision docs/architecture-board/decisions/20260214-scheduler-arbitration-contract.md]
    [--ring0-export-map kernel/include/generated/ring0.exports.map]
    [--governance-boundary docs/governance/CONSTITUTION_BOUNDARY.md]
    [--drift-activation constitution/drift_blocking_activation.md]

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
NON_OVERRIDABLE="${ROOT}/_ayken/steering/NON_OVERRIDABLE.md"
SYSCALL_H="${ROOT}/kernel/sys/syscall_v2.h"
MAKEFILE_PATH="${ROOT}/Makefile"
SCHED_H="${ROOT}/kernel/sched/sched.h"
ARCH_FREEZE="${ROOT}/ARCHITECTURE_FREEZE.md"
SCHED_ARB_DECISION="${ROOT}/docs/architecture-board/decisions/20260214-scheduler-arbitration-contract.md"
RING0_EXPORT_MAP="${ROOT}/kernel/include/generated/ring0.exports.map"
GOVERNANCE_BOUNDARY="${ROOT}/docs/governance/CONSTITUTION_BOUNDARY.md"
DRIFT_BLOCKING_ACTIVATION="${ROOT}/constitution/drift_blocking_activation.md"

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
    --non-overridable)
      NON_OVERRIDABLE="$2"
      shift 2
      ;;
    --syscall-h)
      SYSCALL_H="$2"
      shift 2
      ;;
    --makefile)
      MAKEFILE_PATH="$2"
      shift 2
      ;;
    --sched-h)
      SCHED_H="$2"
      shift 2
      ;;
    --arch-freeze)
      ARCH_FREEZE="$2"
      shift 2
      ;;
    --sched-arb-decision)
      SCHED_ARB_DECISION="$2"
      shift 2
      ;;
    --ring0-export-map)
      RING0_EXPORT_MAP="$2"
      shift 2
      ;;
    --governance-boundary)
      GOVERNANCE_BOUNDARY="$2"
      shift 2
      ;;
    --drift-activation)
      DRIFT_BLOCKING_ACTIVATION="$2"
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
CONTRACT_TXT="${EVIDENCE_DIR}/contract.txt"
NON_OVERRIDABLE_CHECK_TXT="${EVIDENCE_DIR}/non-overridable-check.txt"
SCHED_FALLBACK_CHECK_TXT="${EVIDENCE_DIR}/sched-fallback-check.txt"
GOVERNANCE_BOUNDARY_CHECK_TXT="${EVIDENCE_DIR}/governance-boundary-check.txt"
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
NON_OVERRIDABLE_ENV="${NON_OVERRIDABLE}" \
SYSCALL_H_ENV="${SYSCALL_H}" \
MAKEFILE_PATH_ENV="${MAKEFILE_PATH}" \
SCHED_H_ENV="${SCHED_H}" \
ARCH_FREEZE_ENV="${ARCH_FREEZE}" \
SCHED_ARB_DECISION_ENV="${SCHED_ARB_DECISION}" \
RING0_EXPORT_MAP_ENV="${RING0_EXPORT_MAP}" \
GOVERNANCE_BOUNDARY_ENV="${GOVERNANCE_BOUNDARY}" \
DRIFT_BLOCKING_ACTIVATION_ENV="${DRIFT_BLOCKING_ACTIVATION}" \
TRACKED_KERNEL_ENV="${TRACKED_KERNEL}" \
RING0_WHITELIST_VIOLATIONS_ENV="${RING0_WHITELIST_VIOLATIONS}" \
RING0_SYMBOLS_ENV="${RING0_SYMBOLS}" \
RING0_SYMBOL_VIOLATIONS_ENV="${RING0_SYMBOL_VIOLATIONS}" \
CONTRACT_TXT_ENV="${CONTRACT_TXT}" \
NON_OVERRIDABLE_CHECK_TXT_ENV="${NON_OVERRIDABLE_CHECK_TXT}" \
SCHED_FALLBACK_CHECK_TXT_ENV="${SCHED_FALLBACK_CHECK_TXT}" \
GOVERNANCE_BOUNDARY_CHECK_TXT_ENV="${GOVERNANCE_BOUNDARY_CHECK_TXT}" \
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
from pathlib import Path

ROOT = Path(os.environ["ROOT_ENV"])
STRICT_MODE = os.environ.get("STRICT_ENV", "1") == "1"

KERNEL_ELF = Path(os.environ["KERNEL_ELF_ENV"])
KERNEL_WHITELIST = Path(os.environ["KERNEL_WHITELIST_ENV"])
RING0_SYMBOL_WHITELIST = Path(os.environ["RING0_SYMBOL_WHITELIST_ENV"])
NON_OVERRIDABLE = Path(os.environ["NON_OVERRIDABLE_ENV"])
SYSCALL_H = Path(os.environ["SYSCALL_H_ENV"])
MAKEFILE_PATH = Path(os.environ["MAKEFILE_PATH_ENV"])
SCHED_H = Path(os.environ["SCHED_H_ENV"])
ARCH_FREEZE = Path(os.environ["ARCH_FREEZE_ENV"])
SCHED_ARB_DECISION = Path(os.environ["SCHED_ARB_DECISION_ENV"])
RING0_EXPORT_MAP = Path(os.environ["RING0_EXPORT_MAP_ENV"])
GOVERNANCE_BOUNDARY = Path(os.environ["GOVERNANCE_BOUNDARY_ENV"])
DRIFT_BLOCKING_ACTIVATION = Path(os.environ["DRIFT_BLOCKING_ACTIVATION_ENV"])

TRACKED_KERNEL = Path(os.environ["TRACKED_KERNEL_ENV"])
RING0_WHITELIST_VIOLATIONS = Path(os.environ["RING0_WHITELIST_VIOLATIONS_ENV"])
RING0_SYMBOLS = Path(os.environ["RING0_SYMBOLS_ENV"])
RING0_SYMBOL_VIOLATIONS = Path(os.environ["RING0_SYMBOL_VIOLATIONS_ENV"])
CONTRACT_TXT = Path(os.environ["CONTRACT_TXT_ENV"])
NON_OVERRIDABLE_CHECK_TXT = Path(os.environ["NON_OVERRIDABLE_CHECK_TXT_ENV"])
SCHED_FALLBACK_CHECK_TXT = Path(os.environ["SCHED_FALLBACK_CHECK_TXT_ENV"])
GOVERNANCE_BOUNDARY_CHECK_TXT = Path(os.environ["GOVERNANCE_BOUNDARY_CHECK_TXT_ENV"])
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
    CONTRACT_TXT,
    NON_OVERRIDABLE_CHECK_TXT,
    SCHED_FALLBACK_CHECK_TXT,
    GOVERNANCE_BOUNDARY_CHECK_TXT,
    VIOLATIONS_TXT,
):
    path.write_text("", encoding="utf-8")

violations = []
ring0_whitelist_hits = []
ring0_symbol_list = []
ring0_symbol_hits = []
contract = {}
sched_fallback_violations_count = 0
sched_fallback_check_lines = []
sched_arbitration_violations_count = 0
sched_arbitration_check_lines = []
linker_enforcement_violations_count = 0
linker_enforcement_check_lines = []
governance_boundary_violations_count = 0
governance_boundary_check_lines = []
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


try:
    tracked_kernel = subprocess.check_output(
        ["git", "-C", str(ROOT), "ls-files", "kernel"], text=True
    ).splitlines()
except subprocess.CalledProcessError:
    add_violation("git_ls_files_failed", "kernel")
    tracked_kernel = []

TRACKED_KERNEL.write_text(
    "\n".join(tracked_kernel) + ("\n" if tracked_kernel else ""),
    encoding="utf-8",
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
        "\n".join(ring0_whitelist_hits) + "\n",
        encoding="utf-8",
    )

# 2) Ring0 exported symbol whitelist enforcement (strict mode).
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
                ["nm", "-g", "--defined-only", str(KERNEL_ELF)],
                text=True,
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
                "\n".join(ring0_symbol_hits) + "\n",
                encoding="utf-8",
            )

# 3) Syscall contract lock (must remain frozen).
contract_macros = {}
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
    "".join(f"{k}={v}\n" for k, v in sorted(contract.items())),
    encoding="utf-8",
)

# 4) Scheduler fallback isolation contract.
sched_fallback_env = os.environ.get("AYKEN_SCHED_FALLBACK", "0").strip()
sched_fallback_check_lines.append(f"env_AYKEN_SCHED_FALLBACK={sched_fallback_env}")

if sched_fallback_env not in {"0", "1"}:
    sched_fallback_violations_count += 1
    add_violation("sched_fallback_env_invalid", sched_fallback_env or "MISSING")

if STRICT_MODE and sched_fallback_env != "0":
    sched_fallback_violations_count += 1
    add_violation("sched_fallback_strict_mode", f"expected=0:actual={sched_fallback_env}")

if not MAKEFILE_PATH.exists():
    sched_fallback_violations_count += 1
    add_violation("missing_file", str(MAKEFILE_PATH))
    sched_fallback_check_lines.append("makefile_default=missing")
else:
    mk_text = MAKEFILE_PATH.read_text(encoding="utf-8", errors="replace")
    mk_default_ok = re.search(r"^\s*AYKEN_SCHED_FALLBACK\s*\?=\s*0\s*$", mk_text, flags=re.MULTILINE) is not None
    sched_fallback_check_lines.append(f"makefile_default={'ok' if mk_default_ok else 'missing_or_not_zero'}")
    if not mk_default_ok:
        sched_fallback_violations_count += 1
        add_violation("sched_fallback_makefile_default", "AYKEN_SCHED_FALLBACK ?= 0")

if not SCHED_H.exists():
    sched_fallback_violations_count += 1
    add_violation("missing_file", str(SCHED_H))
    sched_fallback_check_lines.append("sched_header_default=missing")
else:
    sh_text = SCHED_H.read_text(encoding="utf-8", errors="replace")
    header_default_ok = re.search(
        r"^\s*#define\s+AYKEN_SCHED_FALLBACK\s+0\s*$", sh_text, flags=re.MULTILINE
    ) is not None
    sched_fallback_check_lines.append(
        f"sched_header_default={'ok' if header_default_ok else 'missing_or_not_zero'}"
    )
    if not header_default_ok:
        sched_fallback_violations_count += 1
        add_violation("sched_fallback_header_default", "AYKEN_SCHED_FALLBACK 0")

# 4b) Scheduler arbitration contract freeze guard.
if not ARCH_FREEZE.exists():
    sched_arbitration_violations_count += 1
    add_violation("missing_file", str(ARCH_FREEZE))
    sched_arbitration_check_lines.append("freeze_doc_contract=missing_file")
else:
    freeze_text = ARCH_FREEZE.read_text(encoding="utf-8", errors="replace")
    if "Scheduler Arbitration Contract" in freeze_text:
        sched_arbitration_check_lines.append("freeze_doc_contract=present")
    else:
        sched_arbitration_violations_count += 1
        add_violation("scheduler_arbitration_contract_missing", str(ARCH_FREEZE))
        sched_arbitration_check_lines.append("freeze_doc_contract=missing_marker")

if not SCHED_ARB_DECISION.exists():
    sched_arbitration_violations_count += 1
    add_violation("missing_file", str(SCHED_ARB_DECISION))
    sched_arbitration_check_lines.append("decision_record=missing")
else:
    sched_arbitration_check_lines.append("decision_record=present")

# 4c) Linker-level Ring0 export enforcement contract.
if not MAKEFILE_PATH.exists():
    linker_enforcement_violations_count += 1
    add_violation("missing_file", str(MAKEFILE_PATH))
    linker_enforcement_check_lines.append("makefile=missing")
else:
    mk_text = MAKEFILE_PATH.read_text(encoding="utf-8", errors="replace")
    policy_default_ok = re.search(
        r"^\s*KERNEL_EXPORT_POLICY\s*\?=\s*1\s*$", mk_text, flags=re.MULTILINE
    ) is not None
    if policy_default_ok:
        linker_enforcement_check_lines.append("makefile_policy_default=ok")
    else:
        linker_enforcement_violations_count += 1
        add_violation("linker_export_policy_default", "KERNEL_EXPORT_POLICY ?= 1")
        linker_enforcement_check_lines.append("makefile_policy_default=missing_or_not_one")

    linker_flag_ok = "--version-script=$(RING0_EXPORT_MAP)" in mk_text
    if linker_flag_ok:
        linker_enforcement_check_lines.append("makefile_version_script_flag=ok")
    else:
        linker_enforcement_violations_count += 1
        add_violation("linker_export_policy_flag", "--version-script=$(RING0_EXPORT_MAP)")
        linker_enforcement_check_lines.append("makefile_version_script_flag=missing")

if STRICT_MODE:
    if not RING0_EXPORT_MAP.exists():
        linker_enforcement_violations_count += 1
        add_violation("missing_file", str(RING0_EXPORT_MAP))
        linker_enforcement_check_lines.append("export_map=missing")
    else:
        export_text = RING0_EXPORT_MAP.read_text(encoding="utf-8", errors="replace")
        has_local_all = bool(re.search(r"local:\s*\n\s*\*;", export_text))
        if has_local_all:
            linker_enforcement_check_lines.append("export_map_local_all=ok")
        else:
            linker_enforcement_violations_count += 1
            add_violation("linker_export_map_invalid", "local:* missing")
            linker_enforcement_check_lines.append("export_map_local_all=missing")

# 5) Constitutional boundary lock for governance layers.
governance_doc_specs = [
    (
        "constitution_boundary_doc",
        GOVERNANCE_BOUNDARY,
        [
            ("constitutional_surface", r"Constitutional Surface"),
            ("abi_contract_lock", r"ABI contract"),
            ("marker_contract_lock", r"Marker contract"),
            ("tier3_surface", r"Non-Constitutional Governance Surface"),
            ("phase7_8_non_blocking", r"Phase\s*7\s*and\s*Phase\s*8"),
            ("auto_rewrite_forbidden", r"auto-rewrite is forbidden"),
        ],
    ),
    (
        "drift_activation_protocol",
        DRIFT_BLOCKING_ACTIVATION,
        [
            ("phase_min_guard", r"Phase is\s*`?>=\s*9`?"),
            ("policy_default_disabled", r"enabled=false"),
            ("no_auto_activation", r"No auto-activation"),
        ],
    ),
]

for doc_label, doc_path, required_patterns in governance_doc_specs:
    if not doc_path.exists():
        governance_boundary_violations_count += 1
        add_violation("missing_file", str(doc_path))
        governance_boundary_check_lines.append(f"{doc_label}=missing")
        continue

    governance_boundary_check_lines.append(f"{doc_label}=present")
    doc_text = doc_path.read_text(encoding="utf-8", errors="replace")
    for token_name, token_pattern in required_patterns:
        present = re.search(token_pattern, doc_text, flags=re.IGNORECASE | re.MULTILINE) is not None
        governance_boundary_check_lines.append(
            f"{doc_label}:{token_name}={'present' if present else 'missing'}"
        )
        if not present:
            governance_boundary_violations_count += 1
            add_violation("governance_boundary_missing", f"{doc_path}:{token_name}")

# 6) NON_OVERRIDABLE integrity check.
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
    "\n".join(non_over_lines) + ("\n" if non_over_lines else ""),
    encoding="utf-8",
)

SCHED_FALLBACK_CHECK_TXT.write_text(
    "\n".join(
        sched_fallback_check_lines
        + sched_arbitration_check_lines
        + linker_enforcement_check_lines
    )
    + "\n",
    encoding="utf-8",
)

GOVERNANCE_BOUNDARY_CHECK_TXT.write_text(
    "\n".join(governance_boundary_check_lines)
    + ("\n" if governance_boundary_check_lines else ""),
    encoding="utf-8",
)

violations = sorted(set(v for v in violations if v))
VIOLATIONS_TXT.write_text(
    "\n".join(violations) + ("\n" if violations else ""),
    encoding="utf-8",
)

meta = {
    "time_utc": NOW,
    "git_sha": GIT_SHA,
    "strict_mode": "1" if STRICT_MODE else "0",
    "kernel_elf": str(KERNEL_ELF),
    "kernel_whitelist_file": str(KERNEL_WHITELIST),
    "ring0_symbol_whitelist_file": str(RING0_SYMBOL_WHITELIST),
    "non_overridable_file": str(NON_OVERRIDABLE),
    "tracked_kernel_file_count": len(tracked_kernel),
    "ring0_whitelist_violations_count": len(ring0_whitelist_hits),
    "ring0_symbol_count": len(ring0_symbol_list),
    "ring0_symbol_violations_count": len(ring0_symbol_hits),
    "sched_fallback_env": sched_fallback_env,
    "sched_fallback_violations_count": sched_fallback_violations_count,
    "sched_arbitration_violations_count": sched_arbitration_violations_count,
    "linker_enforcement_violations_count": linker_enforcement_violations_count,
    "governance_boundary_violations_count": governance_boundary_violations_count,
    "non_overridable_missing_count": non_overridable_missing_count,
    "violations_count": len(violations),
}

META_TXT.write_text(
    "".join(f"{k}={v}\n" for k, v in meta.items()),
    encoding="utf-8",
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
        "syscall_contract": "PASS" if contract and not any(v.startswith("syscall_contract_") for v in violations) else "FAIL",
        "scheduler_fallback_policy": "PASS" if sched_fallback_violations_count == 0 else "FAIL",
        "scheduler_arbitration_contract": "PASS" if sched_arbitration_violations_count == 0 else "FAIL",
        "linker_symbol_enforcement": "PASS" if linker_enforcement_violations_count == 0 else "FAIL",
        "governance_boundary_lock": "PASS" if governance_boundary_violations_count == 0 else "FAIL",
        "non_overridable_integrity": "PASS" if non_overridable_missing_count == 0 else "FAIL",
    },
    "violations": violations,
    "ring0_whitelist_violations": ring0_whitelist_hits,
    "ring0_symbol_violations": ring0_symbol_hits,
    "sched_fallback_checks": sched_fallback_check_lines,
    "sched_arbitration_checks": sched_arbitration_check_lines,
    "linker_enforcement_checks": linker_enforcement_check_lines,
    "governance_boundary_checks": governance_boundary_check_lines,
}

REPORT_JSON.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n", encoding="utf-8")

raise SystemExit(0 if not violations else 2)
PY

VIOLATIONS_COUNT="$(grep -c . "${VIOLATIONS_TXT}" 2>/dev/null || true)"
if [[ "${VIOLATIONS_COUNT}" -gt 0 ]]; then
  echo "constitutional: FAIL (${VIOLATIONS_COUNT} violations)"
  echo "See: ${VIOLATIONS_TXT}"
  exit 2
fi

echo "constitutional: PASS"
exit 0
