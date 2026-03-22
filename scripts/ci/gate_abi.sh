#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CI_TOOLS="${ROOT}/tools/ci"
source "${CI_TOOLS}/lib.sh"

usage() {
  cat <<'USAGE'
Usage:
  scripts/ci/gate_abi.sh --evidence-dir evidence/run-<id>/gates/abi [--baseline-file scripts/ci/abi-baseline.lock.json] [--init-baseline] [--diff-range <git-range>]

Exit codes:
  0: pass
  2: ABI drift or baseline mismatch detected
  3: usage/tooling error
USAGE
}

EVIDENCE_DIR=""
BASELINE_FILE="${ROOT}/scripts/ci/abi-baseline.lock.json"
INIT_BASELINE=0
DIFF_RANGE="${ABI_DIFF_RANGE:-}"

ABI_H_REL="kernel/include/ayken_abi.h"
ABI_INC_REL="kernel/include/generated/ayken_abi.inc"
SYSCALL_H_REL="kernel/sys/syscall_v2.h"

ABI_H="${ROOT}/${ABI_H_REL}"
ABI_INC="${ROOT}/${ABI_INC_REL}"
SYSCALL_H="${ROOT}/${SYSCALL_H_REL}"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --evidence-dir)
      EVIDENCE_DIR="$2"
      shift 2
      ;;
    --baseline-file)
      BASELINE_FILE="$2"
      shift 2
      ;;
    --init-baseline)
      INIT_BASELINE=1
      shift 1
      ;;
    --diff-range)
      DIFF_RANGE="$2"
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

if ! command -v git >/dev/null 2>&1 || ! command -v make >/dev/null 2>&1 || ! command -v python3 >/dev/null 2>&1; then
  echo "ERROR: required tools missing (git/make/python3)" >&2
  exit 3
fi

mkdir -p "${EVIDENCE_DIR}"
HEADER_SHA="${EVIDENCE_DIR}/header.sha256"
GENERATED_SHA="${EVIDENCE_DIR}/generated.sha256"
SYSCALL_SHA="${EVIDENCE_DIR}/syscall_header.sha256"
LAYOUT_TXT="${EVIDENCE_DIR}/layout.txt"
LAYOUT_SHA="${EVIDENCE_DIR}/layout.sha256"
SYSCALL_TABLE_TXT="${EVIDENCE_DIR}/syscall_table.txt"
CONTRACT_TXT="${EVIDENCE_DIR}/contract.txt"
ACTUAL_LOCK_JSON="${EVIDENCE_DIR}/actual.lock.json"
BASELINE_DIFF_TXT="${EVIDENCE_DIR}/baseline.diff.txt"
GENERATE_LOG="${EVIDENCE_DIR}/generate.log"
GENERATED_DIFF="${EVIDENCE_DIR}/generated.diff.txt"
CHANGED_TXT="${EVIDENCE_DIR}/changed-files.txt"
ABI_AFFECTING_TXT="${EVIDENCE_DIR}/abi-affecting-files.txt"
VIOLATIONS_TXT="${EVIDENCE_DIR}/violations.txt"
META_TXT="${EVIDENCE_DIR}/meta.txt"
REPORT_JSON="${EVIDENCE_DIR}/report.json"
PARSE_ERR_TXT="${EVIDENCE_DIR}/contract.parse.err.txt"

: > "${HEADER_SHA}"
: > "${GENERATED_SHA}"
: > "${SYSCALL_SHA}"
: > "${LAYOUT_TXT}"
: > "${LAYOUT_SHA}"
: > "${SYSCALL_TABLE_TXT}"
: > "${CONTRACT_TXT}"
: > "${BASELINE_DIFF_TXT}"
: > "${VIOLATIONS_TXT}"
: > "${GENERATED_DIFF}"
: > "${PARSE_ERR_TXT}"
: > "${CHANGED_TXT}"
: > "${ABI_AFFECTING_TXT}"

record_violation() {
  echo "$1" >> "${VIOLATIONS_TXT}"
}

SYSCALL_HEADER_CHANGED=0
BASELINE_FILE_CHANGED=0
BASELINE_REL_FOR_DIFF="${BASELINE_FILE}"
if [[ "${BASELINE_REL_FOR_DIFF}" == "${ROOT}/"* ]]; then
  BASELINE_REL_FOR_DIFF="${BASELINE_REL_FOR_DIFF#${ROOT}/}"
fi

resolve_diff_range() {
  if [[ -n "${DIFF_RANGE}" ]]; then
    echo "${DIFF_RANGE}"
    return 0
  fi

  if [[ -n "${GITHUB_BASE_REF:-}" ]]; then
    local base_ref="origin/${GITHUB_BASE_REF}"
    if ! git -C "${ROOT}" rev-parse --verify "${base_ref}" >/dev/null 2>&1; then
      git -C "${ROOT}" fetch --no-tags --depth=200 origin "${GITHUB_BASE_REF}:${base_ref}" >/dev/null 2>&1 || true
      git -C "${ROOT}" fetch --no-tags --depth=200 origin "${GITHUB_BASE_REF}" >/dev/null 2>&1 || true
    fi
    if git -C "${ROOT}" rev-parse --verify "${base_ref}" >/dev/null 2>&1; then
      local merge_base
      merge_base="$(git -C "${ROOT}" merge-base "${base_ref}" HEAD 2>/dev/null || true)"
      if [[ -n "${merge_base}" ]]; then
        echo "${merge_base}...HEAD"
        return 0
      fi
    fi
  fi

  if git -C "${ROOT}" rev-parse --verify origin/main >/dev/null 2>&1; then
    local merge_base
    merge_base="$(git -C "${ROOT}" merge-base origin/main HEAD 2>/dev/null || true)"
    if [[ -n "${merge_base}" ]]; then
      echo "${merge_base}...HEAD"
      return 0
    fi
  fi

  if git -C "${ROOT}" rev-parse --verify HEAD~1 >/dev/null 2>&1; then
    echo "HEAD~1...HEAD"
    return 0
  fi

  echo "HEAD"
  return 0
}

emit_skip_report() {
  local range="$1"
  local now git_sha changed_count abi_affecting_count
  now="$(ci_now_utc)"
  git_sha="$(git -C "${ROOT}" rev-parse HEAD 2>/dev/null || echo "NO_GIT")"
  changed_count="$(wc -l < "${CHANGED_TXT}" | tr -d ' ' || echo 0)"
  abi_affecting_count="$(wc -l < "${ABI_AFFECTING_TXT}" | tr -d ' ' || echo 0)"
  {
    echo "time_utc=${now}"
    echo "git_sha=${git_sha}"
    echo "diff_range=${range}"
    echo "skipped=1"
    echo "skip_reason=no_abi_affecting_changes"
    echo "changed_count=${changed_count}"
    echo "abi_affecting_count=${abi_affecting_count}"
    echo "baseline_file=${BASELINE_FILE}"
    echo "init_baseline=${INIT_BASELINE}"
    echo "violations_count=0"
  } > "${META_TXT}"

  EVIDENCE_DIR_ENV="${EVIDENCE_DIR}" python3 - <<'PY' > "${REPORT_JSON}"
import json
import os

base = os.environ["EVIDENCE_DIR_ENV"]

def read_lines(name):
    p = os.path.join(base, name)
    if not os.path.exists(p):
        return []
    with open(p, "r", encoding="utf-8", errors="replace") as fh:
        return [ln.rstrip("\n") for ln in fh if ln.strip()]

meta = {}
for line in read_lines("meta.txt"):
    if "=" not in line:
        continue
    k, v = line.split("=", 1)
    meta[k] = v

out = {
    "gate": "abi",
    "verdict": "PASS",
    "skipped": True,
    "skip_reason": "no_abi_affecting_changes",
    "violations_count": 0,
    "meta": meta,
    "changed_files": read_lines("changed-files.txt"),
    "abi_affecting_files": read_lines("abi-affecting-files.txt"),
    "contract": {},
    "syscall_table": [],
    "baseline_diff": [],
    "violations": [],
}
print(json.dumps(out, indent=2, sort_keys=True))
PY
}

# 0) Early skip for diffs that do not touch ABI contract inputs.
if [[ "${INIT_BASELINE}" -eq 0 ]]; then
  RANGE="$(resolve_diff_range)"
  if git -C "${ROOT}" diff --name-only --diff-filter=ACMRDT "${RANGE}" > "${CHANGED_TXT}" 2>/dev/null; then
    if [[ ! -s "${CHANGED_TXT}" && -f "${ROOT}/.git/HEAD" ]]; then
      git -C "${ROOT}" show --pretty="" --name-only HEAD > "${CHANGED_TXT}" 2>/dev/null || true
    fi
    ABI_TRIGGER_PAT='^(kernel/include/ayken_abi\.h|kernel/include/generated/ayken_abi\.inc|kernel/sys/syscall_v2\.h|scripts/ci/abi-baseline\.lock\.json)$'
    while IFS= read -r path; do
      [[ -z "${path}" ]] && continue
      if echo "${path}" | grep -E -q -- "${ABI_TRIGGER_PAT}"; then
        echo "${path}" >> "${ABI_AFFECTING_TXT}"
      fi
    done < "${CHANGED_TXT}"

    if grep -Fxq "${SYSCALL_H_REL}" "${CHANGED_TXT}" 2>/dev/null; then
      SYSCALL_HEADER_CHANGED=1
    fi
    if grep -Fxq "${BASELINE_REL_FOR_DIFF}" "${CHANGED_TXT}" 2>/dev/null; then
      BASELINE_FILE_CHANGED=1
    fi

    if [[ ! -s "${ABI_AFFECTING_TXT}" ]]; then
      emit_skip_report "${RANGE}"
      echo "abi: PASS (SKIP no ABI-affecting changes)"
      exit 0
    fi
  fi
fi

# Contract guard: syscall contract edits must carry baseline lock updates.
if [[ "${INIT_BASELINE}" -eq 0 && "${SYSCALL_HEADER_CHANGED}" -eq 1 && "${BASELINE_FILE_CHANGED}" -eq 0 ]]; then
  record_violation "baseline_update_required:${BASELINE_REL_FOR_DIFF}:because_${SYSCALL_H_REL}_changed"
fi

# 1) Regenerate ABI include from canonical header.
if ! make -C "${ROOT}" generate-abi > "${GENERATE_LOG}" 2>&1; then
  record_violation "generate_abi_failed:make generate-abi"
fi

if [[ ! -f "${ABI_H}" ]]; then
  record_violation "missing_file:${ABI_H_REL}"
fi
if [[ ! -f "${ABI_INC}" ]]; then
  record_violation "missing_file:${ABI_INC_REL}"
fi
if [[ ! -f "${SYSCALL_H}" ]]; then
  record_violation "missing_file:${SYSCALL_H_REL}"
fi

# 2) Determinism: generated include must not drift after generation.
if ! git -C "${ROOT}" diff --exit-code -- "${ABI_INC_REL}" >/dev/null 2>&1; then
  git -C "${ROOT}" diff -- "${ABI_INC_REL}" > "${GENERATED_DIFF}" || true
  record_violation "generated_include_drift:${ABI_INC_REL}"
fi
if ! git -C "${ROOT}" diff --cached --exit-code -- "${ABI_INC_REL}" >/dev/null 2>&1; then
  record_violation "generated_include_staged_drift:${ABI_INC_REL}"
fi

# 3) Hashes + ABI layout snapshot.
if [[ -f "${ABI_H}" ]]; then
  sha256_file "${ABI_H}" > "${HEADER_SHA}"
  awk '$1=="#define" && ($2=="AYKEN_ABI_VERSION" || $2 ~ /^(CTX_|IRQF_)/) { val=$3; gsub(/[uU]/,"",val); printf("%s=%s\n", $2, val); }' "${ABI_H}" > "${LAYOUT_TXT}"
else
  echo "MISSING" > "${HEADER_SHA}"
fi

if [[ -f "${ABI_INC}" ]]; then
  sha256_file "${ABI_INC}" > "${GENERATED_SHA}"
else
  echo "MISSING" > "${GENERATED_SHA}"
fi

if [[ -f "${SYSCALL_H}" ]]; then
  sha256_file "${SYSCALL_H}" > "${SYSCALL_SHA}"
else
  echo "MISSING" > "${SYSCALL_SHA}"
fi

if [[ -f "${LAYOUT_TXT}" ]]; then
  sha256_file "${LAYOUT_TXT}" > "${LAYOUT_SHA}"
fi

ABI_H_SHA_VAL="$(cat "${HEADER_SHA}" 2>/dev/null || echo MISSING)"
ABI_INC_SHA_VAL="$(cat "${GENERATED_SHA}" 2>/dev/null || echo MISSING)"
SYSCALL_H_SHA_VAL="$(cat "${SYSCALL_SHA}" 2>/dev/null || echo MISSING)"
LAYOUT_SHA_VAL="$(cat "${LAYOUT_SHA}" 2>/dev/null || echo MISSING)"

# 4) Parse syscall contract and emit actual lock payload.
if ! SYSCALL_H_ENV="${SYSCALL_H}" \
  SYSCALL_TABLE_TXT_ENV="${SYSCALL_TABLE_TXT}" \
  CONTRACT_TXT_ENV="${CONTRACT_TXT}" \
  ACTUAL_LOCK_JSON_ENV="${ACTUAL_LOCK_JSON}" \
  ABI_H_SHA_ENV="${ABI_H_SHA_VAL}" \
  ABI_INC_SHA_ENV="${ABI_INC_SHA_VAL}" \
  SYSCALL_H_SHA_ENV="${SYSCALL_H_SHA_VAL}" \
  LAYOUT_SHA_ENV="${LAYOUT_SHA_VAL}" \
  ABI_H_REL_ENV="${ABI_H_REL}" \
  ABI_INC_REL_ENV="${ABI_INC_REL}" \
  SYSCALL_H_REL_ENV="${SYSCALL_H_REL}" \
  python3 - <<'PY' > /dev/null 2> "${PARSE_ERR_TXT}"
import ast
import json
import os
import re

syscall_h = os.environ["SYSCALL_H_ENV"]
syscall_table_txt = os.environ["SYSCALL_TABLE_TXT_ENV"]
contract_txt = os.environ["CONTRACT_TXT_ENV"]
actual_lock_json = os.environ["ACTUAL_LOCK_JSON_ENV"]

abi_h_sha = os.environ["ABI_H_SHA_ENV"]
abi_inc_sha = os.environ["ABI_INC_SHA_ENV"]
syscall_h_sha = os.environ["SYSCALL_H_SHA_ENV"]
layout_sha = os.environ["LAYOUT_SHA_ENV"]

abi_h_rel = os.environ["ABI_H_REL_ENV"]
abi_inc_rel = os.environ["ABI_INC_REL_ENV"]
syscall_h_rel = os.environ["SYSCALL_H_REL_ENV"]

macro_re = re.compile(r"^\s*#define\s+(SYS_V2_[A-Za-z0-9_]+)\s+(.+)$")
macros = {}
with open(syscall_h, "r", encoding="utf-8", errors="replace") as fh:
    for line in fh:
        m = macro_re.match(line)
        if not m:
            continue
        name = m.group(1)
        expr = m.group(2).split("//", 1)[0].split("/*", 1)[0].strip()
        if expr:
            macros[name] = expr

required = [
    "SYS_V2_BASE",
    "SYS_V2_MAX_INDEX",
    "SYS_V2_NR",
    "SYS_V2_LAST",
    "SYS_V2_MAX_SYSCALL",
    "SYS_V2_DEBUG_PUTCHAR",
]
required_syscalls = [
    ("SYS_V2_MAP_MEMORY", 0),
    ("SYS_V2_UNMAP_MEMORY", 1),
    ("SYS_V2_SWITCH_CONTEXT", 2),
    ("SYS_V2_SUBMIT_EXECUTION", 3),
    ("SYS_V2_WAIT_RESULT", 4),
    ("SYS_V2_INTERRUPT_RETURN", 5),
    ("SYS_V2_TIME_QUERY", 6),
    ("SYS_V2_CAPABILITY_BIND", 7),
    ("SYS_V2_CAPABILITY_REVOKE", 8),
    ("SYS_V2_EXIT", 9),
    ("SYS_V2_DEBUG_PUTCHAR", 10),
    ("SYS_V2_COMPLETE_EXECUTION", 11),
]

missing = [name for name in required if name not in macros]
if missing:
    raise SystemExit("missing required macros: " + ", ".join(missing))

missing_syscalls = [name for name, _ in required_syscalls if name not in macros]
if missing_syscalls:
    raise SystemExit("missing required syscall macros: " + ", ".join(missing_syscalls))

cache = {}
stack = set()


def _assert_ast_safe(node):
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
    if name not in macros:
        raise ValueError(f"macro not found: {name}")

    stack.add(name)
    expr = macros[name]

    def repl(match):
        token = match.group(0)
        if token in macros:
            return str(eval_macro(token))
        return token

    expanded = re.sub(r"\b[A-Za-z_][A-Za-z0-9_]*\b", repl, expr)
    if re.search(r"[^0-9+\-*/%() \t]", expanded):
        raise ValueError(f"unsafe expression for {name}: {expanded}")

    node = ast.parse(expanded, mode="eval")
    _assert_ast_safe(node)
    value = eval(compile(node, "<sys_v2_expr>", "eval"), {"__builtins__": {}}, {})
    value = int(value)

    stack.remove(name)
    cache[name] = value
    return value


contract = {name: eval_macro(name) for name in required}

if contract["SYS_V2_LAST"] != contract["SYS_V2_BASE"] + contract["SYS_V2_MAX_INDEX"]:
    raise SystemExit("contract mismatch: SYS_V2_LAST != SYS_V2_BASE + SYS_V2_MAX_INDEX")
if contract["SYS_V2_NR"] != contract["SYS_V2_MAX_INDEX"] + 1:
    raise SystemExit("contract mismatch: SYS_V2_NR != SYS_V2_MAX_INDEX + 1")
if contract["SYS_V2_MAX_SYSCALL"] != contract["SYS_V2_MAX_INDEX"]:
    raise SystemExit("contract mismatch: SYS_V2_MAX_SYSCALL != SYS_V2_MAX_INDEX")
if contract["SYS_V2_NR"] != len(required_syscalls):
    raise SystemExit("contract mismatch: SYS_V2_NR != required syscall count")

for name in sorted(contract):
    with open(contract_txt, "a", encoding="utf-8") as out:
        out.write(f"{name}={contract[name]}\n")

table = []
seen_indices = set()
for name, expected_idx in required_syscalls:
    actual_idx = eval_macro(name)
    if actual_idx != expected_idx:
        raise SystemExit(f"syscall index mismatch: {name} expected={expected_idx} actual={actual_idx}")
    if actual_idx in seen_indices:
        raise SystemExit(f"duplicate syscall index: {actual_idx}")
    seen_indices.add(actual_idx)
    table.append((actual_idx, name))
table.sort(key=lambda x: x[0])

expected_indices = set(range(contract["SYS_V2_NR"]))
if seen_indices != expected_indices:
    raise SystemExit(
        "syscall table not contiguous: expected="
        + ",".join(str(x) for x in sorted(expected_indices))
        + " actual="
        + ",".join(str(x) for x in sorted(seen_indices))
    )

with open(syscall_table_txt, "w", encoding="utf-8") as out:
    for idx, name in table:
        public_no = contract["SYS_V2_BASE"] + idx
        out.write(f"{name} index={idx} public={public_no}\n")

actual = {
    "schema_version": 1,
    "sources": {
        "abi_header": abi_h_rel,
        "abi_generated": abi_inc_rel,
        "syscall_header": syscall_h_rel,
    },
    "hashes": {
        "abi_header_sha256": abi_h_sha,
        "abi_generated_sha256": abi_inc_sha,
        "syscall_header_sha256": syscall_h_sha,
        "abi_layout_sha256": layout_sha,
    },
    "contract": {
        "sys_v2_base": contract["SYS_V2_BASE"],
        "sys_v2_last": contract["SYS_V2_LAST"],
        "sys_v2_nr": contract["SYS_V2_NR"],
        "sys_v2_max_index": contract["SYS_V2_MAX_INDEX"],
        "sys_v2_debug_index": contract["SYS_V2_DEBUG_PUTCHAR"],
    },
    "syscalls": [{"name": name, "index": idx, "public": contract["SYS_V2_BASE"] + idx} for idx, name in table],
}

with open(actual_lock_json, "w", encoding="utf-8") as out:
    json.dump(actual, out, indent=2, sort_keys=True)
PY
then
  record_violation "syscall_contract_parse_failed:${SYSCALL_H_REL}"
fi

# 5) Baseline policy.
if [[ -f "${BASELINE_FILE}" ]]; then
  BASELINE_REL=""
  if [[ "${BASELINE_FILE}" == "${ROOT}/"* ]]; then
    BASELINE_REL="${BASELINE_FILE#${ROOT}/}"
  fi
  if [[ -n "${BASELINE_REL}" ]]; then
    if ! git -C "${ROOT}" ls-files --error-unmatch -- "${BASELINE_REL}" >/dev/null 2>&1; then
      record_violation "baseline_not_tracked:${BASELINE_REL}"
    fi
    if ! git -C "${ROOT}" diff --exit-code -- "${BASELINE_REL}" >/dev/null 2>&1; then
      record_violation "baseline_dirty_worktree:${BASELINE_REL}"
    fi
    if ! git -C "${ROOT}" diff --cached --exit-code -- "${BASELINE_REL}" >/dev/null 2>&1; then
      record_violation "baseline_dirty_index:${BASELINE_REL}"
    fi
  fi

  if BASELINE_FILE_ENV="${BASELINE_FILE}" ACTUAL_LOCK_JSON_ENV="${ACTUAL_LOCK_JSON}" python3 - <<'PY' > "${BASELINE_DIFF_TXT}" 2>/dev/null
import json
import os

baseline_file = os.environ["BASELINE_FILE_ENV"]
actual_file = os.environ["ACTUAL_LOCK_JSON_ENV"]

with open(baseline_file, "r", encoding="utf-8") as fh:
    baseline = json.load(fh)
with open(actual_file, "r", encoding="utf-8") as fh:
    actual = json.load(fh)

def flatten(value, prefix="", out=None):
    if out is None:
        out = {}
    if isinstance(value, dict):
        for k in sorted(value.keys()):
            key = f"{prefix}.{k}" if prefix else str(k)
            flatten(value[k], key, out)
        return out
    if isinstance(value, list):
        for i, item in enumerate(value):
            key = f"{prefix}[{i}]"
            flatten(item, key, out)
        return out
    out[prefix] = value
    return out

bflat = flatten(baseline)
aflat = flatten(actual)
keys = sorted(set(bflat.keys()) | set(aflat.keys()))
diffs = []
for key in keys:
    bv = bflat.get(key, "<missing>")
    av = aflat.get(key, "<missing>")
    if bv != av:
        diffs.append(f"{key}: baseline={bv} actual={av}")

if diffs:
    for row in diffs:
        print(row)
    raise SystemExit(1)
PY
  then
    :
  else
    record_violation "baseline_mismatch:${BASELINE_FILE}"
    while IFS= read -r line; do
      [[ -z "${line}" ]] && continue
      record_violation "baseline_diff:${line}"
    done < "${BASELINE_DIFF_TXT}"
  fi
else
  if [[ "${INIT_BASELINE}" -eq 1 ]]; then
    mkdir -p "$(dirname "${BASELINE_FILE}")"
    cp -f "${ACTUAL_LOCK_JSON}" "${BASELINE_FILE}"
    record_violation "baseline_initialized_requires_commit:${BASELINE_FILE}"
  else
    record_violation "baseline_missing:${BASELINE_FILE}"
  fi
fi

NOW="$(ci_now_utc)"
GIT_SHA="$(git -C "${ROOT}" rev-parse HEAD 2>/dev/null || echo "NO_GIT")"
VIOLATIONS_COUNT="$(wc -l < "${VIOLATIONS_TXT}" | tr -d ' ' || echo 0)"

{
  echo "time_utc=${NOW}"
  echo "git_sha=${GIT_SHA}"
  echo "abi_header=${ABI_H_REL}"
  echo "abi_generated=${ABI_INC_REL}"
  echo "syscall_header=${SYSCALL_H_REL}"
  echo "baseline_file=${BASELINE_FILE}"
  echo "init_baseline=${INIT_BASELINE}"
  echo "abi_header_sha256=${ABI_H_SHA_VAL}"
  echo "abi_generated_sha256=${ABI_INC_SHA_VAL}"
  echo "syscall_header_sha256=${SYSCALL_H_SHA_VAL}"
  echo "abi_layout_sha256=${LAYOUT_SHA_VAL}"
  echo "violations_count=${VIOLATIONS_COUNT}"
} > "${META_TXT}"

EVIDENCE_DIR_ENV="${EVIDENCE_DIR}" VIOLATIONS_COUNT_ENV="${VIOLATIONS_COUNT}" python3 - <<'PY' > "${REPORT_JSON}"
import json
import os

base = os.environ["EVIDENCE_DIR_ENV"]
violations_count = int(os.environ["VIOLATIONS_COUNT_ENV"])


def read_lines(name):
    p = os.path.join(base, name)
    if not os.path.exists(p):
        return []
    with open(p, "r", encoding="utf-8", errors="replace") as fh:
        return [ln.rstrip("\n") for ln in fh if ln.strip()]


meta = {}
for line in read_lines("meta.txt"):
    if "=" not in line:
        continue
    k, v = line.split("=", 1)
    meta[k] = v

contract = {}
for line in read_lines("contract.txt"):
    if "=" not in line:
        continue
    k, v = line.split("=", 1)
    try:
        contract[k] = int(v)
    except ValueError:
        contract[k] = v

out = {
    "gate": "abi",
    "verdict": "PASS" if violations_count == 0 else "FAIL",
    "violations_count": violations_count,
    "skipped": meta.get("skipped", "0") == "1",
    "skip_reason": meta.get("skip_reason", ""),
    "meta": meta,
    "changed_files": read_lines("changed-files.txt"),
    "abi_affecting_files": read_lines("abi-affecting-files.txt"),
    "contract": contract,
    "syscall_table": read_lines("syscall_table.txt"),
    "baseline_diff": read_lines("baseline.diff.txt"),
    "violations": read_lines("violations.txt"),
}
print(json.dumps(out, indent=2, sort_keys=True))
PY

if [[ "${VIOLATIONS_COUNT}" -gt 0 ]]; then
  echo "abi: FAIL (${VIOLATIONS_COUNT} violations)"
  echo "See: ${VIOLATIONS_TXT}"
  exit 2
fi

echo "abi: PASS"
exit 0
