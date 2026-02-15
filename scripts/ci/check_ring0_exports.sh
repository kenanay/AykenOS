#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CI_TOOLS="${ROOT}/tools/ci"
source "${CI_TOOLS}/lib.sh"

usage() {
  cat <<'USAGE'
Usage:
  scripts/ci/check_ring0_exports.sh --evidence-dir evidence/run-<id>/gates/ring0-exports
    [--kernel-profile validation]
    [--max-exports 165]
    [--kernel-elf kernel.elf]
    [--whitelist scripts/ci/constitutional-ring0-symbol-whitelist.regex]

Exit codes:
  0: pass
  2: export-surface violation detected
  3: usage/tooling error
USAGE
}

EVIDENCE_DIR=""
KERNEL_PROFILE="${KERNEL_PROFILE:-validation}"
MAX_EXPORTS="${RING0_EXPORT_MAX:-165}"
KERNEL_ELF="${ROOT}/kernel.elf"
WHITELIST="${ROOT}/scripts/ci/constitutional-ring0-symbol-whitelist.regex"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --evidence-dir)
      EVIDENCE_DIR="$2"
      shift 2
      ;;
    --kernel-profile)
      KERNEL_PROFILE="$2"
      shift 2
      ;;
    --max-exports)
      MAX_EXPORTS="$2"
      shift 2
      ;;
    --kernel-elf)
      KERNEL_ELF="$2"
      shift 2
      ;;
    --whitelist)
      WHITELIST="$2"
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

if ! command -v make >/dev/null 2>&1 || ! command -v nm >/dev/null 2>&1 || ! command -v python3 >/dev/null 2>&1; then
  echo "ERROR: required tools missing (make/nm/python3)" >&2
  exit 3
fi

if [[ ! "${MAX_EXPORTS}" =~ ^[0-9]+$ ]]; then
  echo "ERROR: --max-exports must be numeric, got '${MAX_EXPORTS}'" >&2
  exit 3
fi

mkdir -p "${EVIDENCE_DIR}"
BUILD_LOG="${EVIDENCE_DIR}/build.log"
GLOBALS_TXT="${EVIDENCE_DIR}/globals.txt"
UNMATCHED_TXT="${EVIDENCE_DIR}/unmatched.txt"
VIOLATIONS_TXT="${EVIDENCE_DIR}/violations.txt"
META_TXT="${EVIDENCE_DIR}/meta.txt"
REPORT_JSON="${EVIDENCE_DIR}/report.json"

: > "${BUILD_LOG}"
: > "${GLOBALS_TXT}"
: > "${UNMATCHED_TXT}"
: > "${VIOLATIONS_TXT}"

# Deterministic policy-on build for evidence.
if ! make -C "${ROOT}" clean >>"${BUILD_LOG}" 2>&1; then
  echo "build_failed:make clean" >> "${VIOLATIONS_TXT}"
fi
if ! make -C "${ROOT}" kernel KERNEL_PROFILE="${KERNEL_PROFILE}" KERNEL_EXPORT_POLICY=1 -j4 >>"${BUILD_LOG}" 2>&1; then
  echo "build_failed:make kernel" >> "${VIOLATIONS_TXT}"
fi

if [[ ! -f "${KERNEL_ELF}" ]]; then
  echo "missing_file:${KERNEL_ELF}" >> "${VIOLATIONS_TXT}"
else
  nm -g --defined-only "${KERNEL_ELF}" | awk '{print $3}' | sed '/^$/d' | sort -u > "${GLOBALS_TXT}" || true
fi

GLOBAL_COUNT="$(wc -l < "${GLOBALS_TXT}" | tr -d ' ' || echo 0)"
if [[ "${GLOBAL_COUNT}" -gt "${MAX_EXPORTS}" ]]; then
  echo "export_surface_too_wide:count=${GLOBAL_COUNT}:max=${MAX_EXPORTS}" >> "${VIOLATIONS_TXT}"
fi

WHITELIST_ENV="${WHITELIST}" GLOBALS_ENV="${GLOBALS_TXT}" UNMATCHED_ENV="${UNMATCHED_TXT}" python3 - <<'PY'
import re
from pathlib import Path
import os

whitelist_path = Path(os.environ["WHITELIST_ENV"])
globals_path = Path(os.environ["GLOBALS_ENV"])
unmatched_path = Path(os.environ["UNMATCHED_ENV"])

if not whitelist_path.exists():
    unmatched_path.write_text("__ERROR__:missing_whitelist\n", encoding="utf-8")
    raise SystemExit(0)

rules = []
for raw in whitelist_path.read_text(encoding="utf-8", errors="replace").splitlines():
    line = raw.strip()
    if not line or line.startswith("#"):
        continue
    rules.append(re.compile(line))

symbols = [s.strip() for s in globals_path.read_text(encoding="utf-8", errors="replace").splitlines() if s.strip()]
unmatched = [s for s in symbols if not any(r.search(s) for r in rules)]
if unmatched:
    unmatched_path.write_text("\n".join(unmatched) + "\n", encoding="utf-8")
PY

if [[ -s "${UNMATCHED_TXT}" ]]; then
  if grep -q "^__ERROR__:missing_whitelist" "${UNMATCHED_TXT}"; then
    echo "missing_file:${WHITELIST}" >> "${VIOLATIONS_TXT}"
  else
    while IFS= read -r sym; do
      [[ -z "${sym}" ]] && continue
      echo "whitelist_miss:${sym}" >> "${VIOLATIONS_TXT}"
    done < "${UNMATCHED_TXT}"
  fi
fi

NOW="$(ci_now_utc)"
GIT_SHA="$(git -C "${ROOT}" rev-parse HEAD 2>/dev/null || echo NO_GIT)"
UNMATCHED_COUNT=0
if [[ -s "${UNMATCHED_TXT}" ]]; then
  UNMATCHED_COUNT="$(grep -vc '^__ERROR__:missing_whitelist' "${UNMATCHED_TXT}" | tr -d ' ' || echo 0)"
fi
VIOLATIONS_COUNT="$(wc -l < "${VIOLATIONS_TXT}" | tr -d ' ' || echo 0)"

{
  echo "time_utc=${NOW}"
  echo "git_sha=${GIT_SHA}"
  echo "kernel_profile=${KERNEL_PROFILE}"
  echo "kernel_elf=${KERNEL_ELF}"
  echo "whitelist_file=${WHITELIST}"
  echo "global_count=${GLOBAL_COUNT}"
  echo "max_exports=${MAX_EXPORTS}"
  echo "unmatched_count=${UNMATCHED_COUNT}"
  echo "violations_count=${VIOLATIONS_COUNT}"
} > "${META_TXT}"

EVIDENCE_DIR_ENV="${EVIDENCE_DIR}" VIOL_COUNT_ENV="${VIOLATIONS_COUNT}" python3 - <<'PY' > "${REPORT_JSON}"
import json
import os
from pathlib import Path

e = Path(os.environ["EVIDENCE_DIR_ENV"])
viol_count = int(os.environ["VIOL_COUNT_ENV"])

meta = {}
for line in (e / "meta.txt").read_text(encoding="utf-8", errors="replace").splitlines():
    if "=" not in line:
        continue
    k, v = line.split("=", 1)
    meta[k] = v

def read_lines(path: Path):
    if not path.exists():
        return []
    return [ln.rstrip("\n") for ln in path.read_text(encoding="utf-8", errors="replace").splitlines() if ln.strip()]

report = {
    "gate": "ring0-exports",
    "verdict": "PASS" if viol_count == 0 else "FAIL",
    "violations_count": viol_count,
    "meta": meta,
    "global_symbols": read_lines(e / "globals.txt"),
    "unmatched_symbols": read_lines(e / "unmatched.txt"),
    "violations": read_lines(e / "violations.txt"),
}
print(json.dumps(report, indent=2, sort_keys=True))
PY

if [[ "${VIOLATIONS_COUNT}" -gt 0 ]]; then
  echo "ring0-exports: FAIL (${VIOLATIONS_COUNT} violations)"
  echo "See: ${VIOLATIONS_TXT}"
  exit 2
fi

echo "ring0-exports: PASS"
exit 0
