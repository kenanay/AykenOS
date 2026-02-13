#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CI_TOOLS="${ROOT}/tools/ci"
source "${CI_TOOLS}/lib.sh"

usage() {
  cat <<'EOF'
Usage:
  tools/ci/symbol-scan.sh --targets "<file1> <file2> ..." \
    --deny tools/ci/deny.symbols \
    --allow tools/ci/allow.symbols \
    --evidence-dir evidence/run-<id>/gates/symbol-scan

Exit codes:
  0: ok
  2: forbidden symbols found (not allowlisted)
  3: tooling/usage error
EOF
}

TARGETS=""
DENY_FILE="${CI_TOOLS}/deny.symbols"
ALLOW_FILE="${CI_TOOLS}/allow.symbols"
EVIDENCE_DIR=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --targets)
      TARGETS="$2"
      shift 2
      ;;
    --deny)
      DENY_FILE="$2"
      shift 2
      ;;
    --allow)
      ALLOW_FILE="$2"
      shift 2
      ;;
    --evidence-dir)
      EVIDENCE_DIR="$2"
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

if [[ -z "${TARGETS}" || -z "${EVIDENCE_DIR}" ]]; then
  usage
  exit 3
fi

if [[ ! -f "${DENY_FILE}" || ! -f "${ALLOW_FILE}" ]]; then
  echo "ERROR: deny/allow file missing" >&2
  exit 3
fi

mkdir -p "${EVIDENCE_DIR}"
RAW_SYMS="${EVIDENCE_DIR}/symbols.raw.txt"
FILTERED_SYMS="${EVIDENCE_DIR}/symbols.filtered.txt"
DENY_HITS="${EVIDENCE_DIR}/deny.hits.txt"
FINAL_VIOLATIONS="${EVIDENCE_DIR}/violations.txt"
REPORT_JSON="${EVIDENCE_DIR}/report.json"
META_TXT="${EVIDENCE_DIR}/meta.txt"

: > "${RAW_SYMS}"
: > "${FILTERED_SYMS}"
: > "${DENY_HITS}"
: > "${FINAL_VIOLATIONS}"

NM_TOOL=""
if command -v llvm-nm >/dev/null 2>&1; then
  NM_TOOL="llvm-nm"
elif command -v nm >/dev/null 2>&1; then
  NM_TOOL="nm"
fi

if [[ -z "${NM_TOOL}" ]]; then
  echo "ERROR: nm/llvm-nm not found" >&2
  exit 3
fi

extract_symbols() {
  local target_file="$1"
  "${NM_TOOL}" -a "${target_file}" 2>/dev/null | awk '{print $NF}' | sed '/^$/d' || true
}

is_allowed() {
  local target_file="$1"
  local symbol="$2"
  while IFS= read -r line; do
    line="${line%%#*}"
    line="$(echo -n "${line}" | sed 's/^[[:space:]]*//;s/[[:space:]]*$//')"
    [[ -z "${line}" ]] && continue

    if [[ "${line}" == *:* ]]; then
      local file_regex="${line%%:*}"
      local symbol_regex="${line#*:}"
      if [[ "${target_file}" =~ ${file_regex} ]]; then
        if echo "${symbol}" | grep -E -q -- "${symbol_regex}"; then
          return 0
        fi
      fi
    else
      if echo "${symbol}" | grep -E -q -- "${line}"; then
        return 0
      fi
    fi
  done < "${ALLOW_FILE}"
  return 1
}

scan_one() {
  local target_file="$1"
  if [[ ! -f "${target_file}" ]]; then
    echo "WARN: target missing: ${target_file}" >&2
    return 0
  fi

  local sha
  sha="$(sha256_file "${target_file}")"
  {
    echo "# TARGET:${target_file}"
    echo "# SHA256:${sha}"
  } >> "${RAW_SYMS}"

  extract_symbols "${target_file}" | sort -u | while IFS= read -r sym; do
    echo "${target_file}:${sym}" >> "${RAW_SYMS}"
  done
}

# 1) Collect symbol lists from all targets.
for target in ${TARGETS}; do
  scan_one "${target}"
done

# 2) Match deny patterns.
# Keep filtered symbol lines as evidence and avoid shell-specific process substitution.
grep -E '^[^#].+:[A-Za-z_][A-Za-z0-9_.$@]*$' "${RAW_SYMS}" > "${FILTERED_SYMS}" || true
while IFS= read -r line; do
  target_file="${line%%:*}"
  sym="${line#*:}"

  while IFS= read -r pat; do
    pat="${pat%%#*}"
    pat="$(echo -n "${pat}" | sed 's/^[[:space:]]*//;s/[[:space:]]*$//')"
    [[ -z "${pat}" ]] && continue

    if echo "${sym}" | grep -E -q -- "${pat}"; then
      echo "${target_file}:${sym}:deny=${pat}" >> "${DENY_HITS}"
      break
    fi
  done < "${DENY_FILE}"
done < "${FILTERED_SYMS}"

# 3) Apply allowlist.
if [[ -s "${DENY_HITS}" ]]; then
  while IFS= read -r hit; do
    target_file="$(echo "${hit}" | cut -d: -f1)"
    sym="$(echo "${hit}" | cut -d: -f2)"
    if ! is_allowed "${target_file}" "${sym}"; then
      echo "${hit}" >> "${FINAL_VIOLATIONS}"
    fi
  done < "${DENY_HITS}"
fi

# 4) Emit metadata + JSON report.
NOW="$(ci_now_utc)"
GIT_SHA="$(git -C "${ROOT}" rev-parse HEAD 2>/dev/null || echo "NO_GIT")"
GIT_DIRTY="$(git -C "${ROOT}" status --porcelain 2>/dev/null | wc -l | tr -d ' ' || echo "NA")"
NM_VER="$(${NM_TOOL} --version 2>/dev/null | head -n 1 || echo "${NM_TOOL}")"
RAW_COUNT="$(grep -E '^[^#].+:[A-Za-z_][A-Za-z0-9_.$@]*$' "${RAW_SYMS}" | wc -l | tr -d ' ' || echo 0)"
FILTERED_COUNT="$(wc -l < "${FILTERED_SYMS}" | tr -d ' ' || echo 0)"
DENY_HITS_COUNT="$(wc -l < "${DENY_HITS}" | tr -d ' ' || echo 0)"

{
  echo "time_utc=${NOW}"
  echo "git_sha=${GIT_SHA}"
  echo "git_dirty_lines=${GIT_DIRTY}"
  echo "nm_tool=${NM_TOOL}"
  echo "nm_version=${NM_VER}"
  echo "deny_file=${DENY_FILE}"
  echo "allow_file=${ALLOW_FILE}"
  echo "targets=${TARGETS}"
  echo "raw_count=${RAW_COUNT}"
  echo "filtered_count=${FILTERED_COUNT}"
  echo "deny_hits_count=${DENY_HITS_COUNT}"
} > "${META_TXT}"

VIOL_COUNT=0
if [[ -s "${FINAL_VIOLATIONS}" ]]; then
  VIOL_COUNT="$(wc -l < "${FINAL_VIOLATIONS}" | tr -d ' ')"
fi

EVIDENCE_DIR_ENV="${EVIDENCE_DIR}" VIOL_COUNT_ENV="${VIOL_COUNT}" python3 - <<'PY' > "${REPORT_JSON}"
import json
import os

e = os.environ["EVIDENCE_DIR_ENV"]
viol_count = int(os.environ["VIOL_COUNT_ENV"])

def read_lines(path):
    if not os.path.exists(path):
        return []
    with open(path, "r", encoding="utf-8", errors="replace") as fh:
        return [line.rstrip("\\n") for line in fh if line.strip()]

meta = {}
with open(os.path.join(e, "meta.txt"), "r", encoding="utf-8") as fh:
    for line in fh:
        line = line.strip()
        if not line or "=" not in line:
            continue
        k, v = line.split("=", 1)
        meta[k] = v

out = {
  "gate": "symbol-scan",
  "verdict": "PASS" if viol_count == 0 else "FAIL",
  "violations_count": viol_count,
  "meta": meta,
  "violations": read_lines(os.path.join(e, "violations.txt")),
  "deny_hits_all": read_lines(os.path.join(e, "deny.hits.txt")),
}
print(json.dumps(out, indent=2, sort_keys=True))
PY

if [[ "${VIOL_COUNT}" -gt 0 ]]; then
  echo "symbol-scan: FAIL (${VIOL_COUNT} violations)"
  echo "See: ${FINAL_VIOLATIONS}"
  exit 2
fi

echo "symbol-scan: PASS"
exit 0
