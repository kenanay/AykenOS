#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CI_TOOLS="${ROOT}/tools/ci"
source "${CI_TOOLS}/lib.sh"

usage() {
  cat <<'USAGE'
Usage:
  scripts/ci/gate_workspace.sh --evidence-dir evidence/run-<id>/gates/workspace [--strict|--no-strict] [--kernel-profile validation]

Exit codes:
  0: pass
  2: workspace violations found
  3: usage/tooling error
USAGE
}

EVIDENCE_DIR=""
STRICT_MODE="${WORKSPACE_STRICT:-1}"
KERNEL_PROFILE="validation"
ABI_INC_REL="kernel/include/generated/ayken_abi.inc"
BASELINE_REL="scripts/ci/abi-baseline.lock.json"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --evidence-dir)
      EVIDENCE_DIR="$2"
      shift 2
      ;;
    --strict)
      STRICT_MODE=1
      shift
      ;;
    --no-strict)
      STRICT_MODE=0
      shift
      ;;
    --kernel-profile)
      KERNEL_PROFILE="$2"
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

for tool in git make python3; do
  if ! command -v "${tool}" >/dev/null 2>&1; then
    echo "ERROR: required tool missing: ${tool}" >&2
    exit 3
  fi
done

mkdir -p "${EVIDENCE_DIR}"
GIT_STATUS_TXT="${EVIDENCE_DIR}/git-status.txt"
UNTRACKED_TXT="${EVIDENCE_DIR}/untracked.txt"
MODIFIED_TXT="${EVIDENCE_DIR}/modified-tracked.txt"
STAGED_TXT="${EVIDENCE_DIR}/staged-tracked.txt"
GENERATE_LOG="${EVIDENCE_DIR}/generate-abi.log"
GENERATED_DIFF_TXT="${EVIDENCE_DIR}/generated-abi.diff.txt"
BASELINE_STATUS_TXT="${EVIDENCE_DIR}/baseline-status.txt"
BUILD1_LOG="${EVIDENCE_DIR}/build-1.log"
BUILD2_LOG="${EVIDENCE_DIR}/build-2.log"
KERNEL_MAP="${EVIDENCE_DIR}/kernel.map"
KERNEL_ELF_1="${EVIDENCE_DIR}/kernel.elf.build1"
KERNEL_ELF_2="${EVIDENCE_DIR}/kernel.elf.build2"
KERNEL_SHA1="${EVIDENCE_DIR}/kernel.build1.sha256"
KERNEL_SHA2="${EVIDENCE_DIR}/kernel.build2.sha256"
REPRO_TXT="${EVIDENCE_DIR}/reproducibility.txt"
LINKSET_TXT="${EVIDENCE_DIR}/linkset.txt"
LINKSET_VIOLATIONS_TXT="${EVIDENCE_DIR}/linkset-violations.txt"
VIOLATIONS_TXT="${EVIDENCE_DIR}/violations.txt"
META_TXT="${EVIDENCE_DIR}/meta.txt"
REPORT_JSON="${EVIDENCE_DIR}/report.json"

: > "${GIT_STATUS_TXT}"
: > "${UNTRACKED_TXT}"
: > "${MODIFIED_TXT}"
: > "${STAGED_TXT}"
: > "${GENERATE_LOG}"
: > "${GENERATED_DIFF_TXT}"
: > "${BASELINE_STATUS_TXT}"
: > "${BUILD1_LOG}"
: > "${BUILD2_LOG}"
: > "${REPRO_TXT}"
: > "${LINKSET_TXT}"
: > "${LINKSET_VIOLATIONS_TXT}"
: > "${VIOLATIONS_TXT}"

record_violation() {
  echo "$1" >> "${VIOLATIONS_TXT}"
}

# 1) Git state discipline.
git -C "${ROOT}" status --porcelain > "${GIT_STATUS_TXT}"
awk '/^\?\?/ {print $2}' "${GIT_STATUS_TXT}" > "${UNTRACKED_TXT}" || true
git -C "${ROOT}" diff --name-only > "${MODIFIED_TXT}"
git -C "${ROOT}" diff --cached --name-only > "${STAGED_TXT}"

if [[ -s "${MODIFIED_TXT}" ]]; then
  while IFS= read -r path; do
    [[ -z "${path}" ]] && continue
    record_violation "dirty_tracked:${path}"
  done < "${MODIFIED_TXT}"
fi

if [[ -s "${STAGED_TXT}" ]]; then
  while IFS= read -r path; do
    [[ -z "${path}" ]] && continue
    record_violation "dirty_staged:${path}"
  done < "${STAGED_TXT}"
fi

if [[ "${STRICT_MODE}" == "1" ]] && [[ -s "${UNTRACKED_TXT}" ]]; then
  while IFS= read -r path; do
    [[ -z "${path}" ]] && continue
    record_violation "dirty_untracked:${path}"
  done < "${UNTRACKED_TXT}"
fi

if ! git -C "${ROOT}" diff --quiet; then
  record_violation "git_diff_nonzero:tracked"
fi
if ! git -C "${ROOT}" diff --cached --quiet; then
  record_violation "git_diff_nonzero:staged"
fi

# 2) Generated include determinism.
if ! make -C "${ROOT}" generate-abi > "${GENERATE_LOG}" 2>&1; then
  record_violation "generate_abi_failed:make generate-abi"
fi
if ! git -C "${ROOT}" diff --exit-code -- "${ABI_INC_REL}" >/dev/null 2>&1; then
  git -C "${ROOT}" diff -- "${ABI_INC_REL}" > "${GENERATED_DIFF_TXT}" || true
  record_violation "generated_include_drift:${ABI_INC_REL}"
fi
if ! git -C "${ROOT}" diff --cached --exit-code -- "${ABI_INC_REL}" >/dev/null 2>&1; then
  record_violation "generated_include_staged_drift:${ABI_INC_REL}"
fi

# 3) Baseline file discipline.
if [[ ! -f "${ROOT}/${BASELINE_REL}" ]]; then
  echo "missing:${BASELINE_REL}" >> "${BASELINE_STATUS_TXT}"
  record_violation "baseline_missing:${BASELINE_REL}"
else
  if git -C "${ROOT}" ls-files --error-unmatch -- "${BASELINE_REL}" >/dev/null 2>&1; then
    echo "tracked:${BASELINE_REL}" >> "${BASELINE_STATUS_TXT}"
  else
    echo "not_tracked:${BASELINE_REL}" >> "${BASELINE_STATUS_TXT}"
    record_violation "baseline_not_tracked:${BASELINE_REL}"
  fi
  if ! git -C "${ROOT}" diff --exit-code -- "${BASELINE_REL}" >/dev/null 2>&1; then
    echo "dirty_worktree:${BASELINE_REL}" >> "${BASELINE_STATUS_TXT}"
    record_violation "baseline_dirty_worktree:${BASELINE_REL}"
  fi
  if ! git -C "${ROOT}" diff --cached --exit-code -- "${BASELINE_REL}" >/dev/null 2>&1; then
    echo "dirty_index:${BASELINE_REL}" >> "${BASELINE_STATUS_TXT}"
    record_violation "baseline_dirty_index:${BASELINE_REL}"
  fi
fi

# 4) Reproducibility signal + link set check.
if ! make -C "${ROOT}" clean > "${BUILD1_LOG}" 2>&1; then
  record_violation "build_failed:clean#1"
fi
if ! make -C "${ROOT}" KERNEL_PROFILE="${KERNEL_PROFILE}" KERNEL_MAP="${KERNEL_MAP}" kernel >> "${BUILD1_LOG}" 2>&1; then
  record_violation "build_failed:kernel#1"
fi

if [[ -f "${ROOT}/kernel.elf" ]]; then
  cp -f "${ROOT}/kernel.elf" "${KERNEL_ELF_1}"
  sha256_file "${KERNEL_ELF_1}" > "${KERNEL_SHA1}"
else
  record_violation "missing_kernel_elf:build#1"
fi

if ! make -C "${ROOT}" clean > "${BUILD2_LOG}" 2>&1; then
  record_violation "build_failed:clean#2"
fi
if ! make -C "${ROOT}" KERNEL_PROFILE="${KERNEL_PROFILE}" kernel >> "${BUILD2_LOG}" 2>&1; then
  record_violation "build_failed:kernel#2"
fi

if [[ -f "${ROOT}/kernel.elf" ]]; then
  cp -f "${ROOT}/kernel.elf" "${KERNEL_ELF_2}"
  sha256_file "${KERNEL_ELF_2}" > "${KERNEL_SHA2}"
else
  record_violation "missing_kernel_elf:build#2"
fi

HASH1="$(cat "${KERNEL_SHA1}" 2>/dev/null || echo MISSING)"
HASH2="$(cat "${KERNEL_SHA2}" 2>/dev/null || echo MISSING)"
{
  echo "build_1_sha256=${HASH1}"
  echo "build_2_sha256=${HASH2}"
  if [[ "${HASH1}" == "${HASH2}" && "${HASH1}" != "MISSING" ]]; then
    echo "reproducible=1"
  else
    echo "reproducible=0"
  fi
} > "${REPRO_TXT}"

if [[ "${HASH1}" == "MISSING" || "${HASH2}" == "MISSING" ]]; then
  record_violation "repro_missing_hash"
elif [[ "${HASH1}" != "${HASH2}" ]]; then
  record_violation "repro_hash_mismatch:${HASH1}:${HASH2}"
fi

if [[ ! -f "${KERNEL_MAP}" ]]; then
  record_violation "link_map_missing:${KERNEL_MAP}"
else
  KERNEL_MAP_ENV="${KERNEL_MAP}" LINKSET_TXT_ENV="${LINKSET_TXT}" python3 - <<'PY'
import os
import re

map_path = os.environ["KERNEL_MAP_ENV"]
out_path = os.environ["LINKSET_TXT_ENV"]
obj_re = re.compile(r"([A-Za-z0-9_./+-]+\.o)\b")
objs = set()

with open(map_path, "r", encoding="utf-8", errors="replace") as fh:
    for line in fh:
        for m in obj_re.finditer(line):
            objs.add(m.group(1))

with open(out_path, "w", encoding="utf-8") as out:
    for obj in sorted(objs):
        out.write(obj + "\n")
PY
  if [[ -s "${LINKSET_TXT}" ]]; then
    while IFS= read -r obj; do
      [[ -z "${obj}" ]] && continue
      case "${obj}" in
        userspace/*|*/userspace/*)
          echo "userspace_obj:${obj}" >> "${LINKSET_VIOLATIONS_TXT}"
          record_violation "linkset_userspace_obj:${obj}"
          ;;
      esac
      case "${obj}" in
        *libayken* )
          echo "libayken_obj:${obj}" >> "${LINKSET_VIOLATIONS_TXT}"
          record_violation "linkset_libayken_obj:${obj}"
          ;;
      esac
      case "${obj}" in
        *_test.o|*/_test.o|*validation_test.o|*/validation_test.o)
          echo "test_obj:${obj}" >> "${LINKSET_VIOLATIONS_TXT}"
          record_violation "linkset_test_obj:${obj}"
          ;;
      esac
    done < "${LINKSET_TXT}"
  fi
fi

NOW="$(ci_now_utc)"
GIT_SHA="$(git -C "${ROOT}" rev-parse HEAD 2>/dev/null || echo "NO_GIT")"
UNTRACKED_COUNT="$(wc -l < "${UNTRACKED_TXT}" | tr -d ' ')"
MODIFIED_COUNT="$(wc -l < "${MODIFIED_TXT}" | tr -d ' ')"
STAGED_COUNT="$(wc -l < "${STAGED_TXT}" | tr -d ' ')"
LINKSET_COUNT="$(wc -l < "${LINKSET_TXT}" | tr -d ' ')"
LINKSET_VIOLATIONS_COUNT="$(wc -l < "${LINKSET_VIOLATIONS_TXT}" | tr -d ' ')"
VIOLATIONS_COUNT="$(wc -l < "${VIOLATIONS_TXT}" | tr -d ' ')"

{
  echo "time_utc=${NOW}"
  echo "git_sha=${GIT_SHA}"
  echo "strict_mode=${STRICT_MODE}"
  echo "kernel_profile=${KERNEL_PROFILE}"
  echo "abi_include=${ABI_INC_REL}"
  echo "baseline_file=${BASELINE_REL}"
  echo "untracked_count=${UNTRACKED_COUNT}"
  echo "modified_count=${MODIFIED_COUNT}"
  echo "staged_count=${STAGED_COUNT}"
  echo "linkset_count=${LINKSET_COUNT}"
  echo "linkset_violations_count=${LINKSET_VIOLATIONS_COUNT}"
  echo "repro_hash_1=${HASH1}"
  echo "repro_hash_2=${HASH2}"
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

out = {
    "gate": "workspace",
    "verdict": "PASS" if violations_count == 0 else "FAIL",
    "violations_count": violations_count,
    "meta": meta,
    "reproducibility": read_lines("reproducibility.txt"),
    "linkset": read_lines("linkset.txt"),
    "linkset_violations": read_lines("linkset-violations.txt"),
    "untracked": read_lines("untracked.txt"),
    "modified_tracked": read_lines("modified-tracked.txt"),
    "staged_tracked": read_lines("staged-tracked.txt"),
    "violations": read_lines("violations.txt"),
}
print(json.dumps(out, indent=2, sort_keys=True))
PY

if [[ "${VIOLATIONS_COUNT}" -gt 0 ]]; then
  echo "workspace: FAIL (${VIOLATIONS_COUNT} violations)"
  echo "See: ${VIOLATIONS_TXT}"
  exit 2
fi

echo "workspace: PASS"
exit 0
