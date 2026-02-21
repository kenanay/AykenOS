#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CI_TOOLS="${ROOT}/tools/ci"
source "${CI_TOOLS}/lib.sh"

usage() {
  cat <<'USAGE'
Usage:
  scripts/ci/gate_tooling_isolation.sh --evidence-dir evidence/run-<id>/gates/tooling-isolation
    [--diff-range <git-range>]

Exit codes:
  0: pass
  2: isolation violations found
  3: usage/tooling error
USAGE
}

EVIDENCE_DIR=""
DIFF_RANGE="${TOOLING_ISOLATION_DIFF_RANGE:-}"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --evidence-dir)
      EVIDENCE_DIR="$2"
      shift 2
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

if ! command -v git >/dev/null 2>&1 || ! command -v python3 >/dev/null 2>&1; then
  echo "ERROR: required tools missing (git/python3)" >&2
  exit 3
fi

mkdir -p "${EVIDENCE_DIR}"
CHANGED_TXT="${EVIDENCE_DIR}/changed-files.txt"
TRIGGERED_TXT="${EVIDENCE_DIR}/triggered-files.txt"
KERNEL_TOUCH_TXT="${EVIDENCE_DIR}/kernel-touches.txt"
VIOLATIONS_TXT="${EVIDENCE_DIR}/violations.txt"
META_TXT="${EVIDENCE_DIR}/meta.txt"
REPORT_JSON="${EVIDENCE_DIR}/report.json"

: > "${CHANGED_TXT}"
: > "${TRIGGERED_TXT}"
: > "${KERNEL_TOUCH_TXT}"
: > "${VIOLATIONS_TXT}"

record_violation() {
  echo "$1" >> "${VIOLATIONS_TXT}"
}

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

RANGE="$(resolve_diff_range)"
if ! git -C "${ROOT}" diff --name-only --diff-filter=ACMRDT "${RANGE}" > "${CHANGED_TXT}" 2>/dev/null; then
  record_violation "diff_range_invalid:${RANGE}"
fi

if [[ ! -s "${CHANGED_TXT}" && -f "${ROOT}/.git/HEAD" ]]; then
  git -C "${ROOT}" show --pretty="" --name-only HEAD > "${CHANGED_TXT}" 2>/dev/null || true
fi

TRIGGER_PAT='^(scripts/ci/gate_performance\.sh|scripts/ci/local_preempt_variance\.sh|scripts/ci/local_perf_baseline_init\.sh|scripts/ci/perf-baseline(\.local)?\.lock\.json|\.github/workflows/ci-freeze\.yml|\.github/workflows/perf-baseline-init\.yml|tools/validation/phase_4_4_qemu_boot_audit\.sh|run_preempt_test\.sh)$'

while IFS= read -r path; do
  [[ -z "${path}" ]] && continue
  if echo "${path}" | grep -E -q -- "${TRIGGER_PAT}"; then
    echo "${path}" >> "${TRIGGERED_TXT}"
  fi
done < "${CHANGED_TXT}"

TRIGGERED=0
if [[ -s "${TRIGGERED_TXT}" ]]; then
  TRIGGERED=1
fi

if [[ "${TRIGGERED}" -eq 1 ]]; then
  # Check for waiver
  WAIVER_FOUND=0
  if [[ -f "${ROOT}/docs/waivers/tooling-isolation-perf-governance-hardening.md" ]]; then
    WAIVER_FOUND=1
    echo "INFO: Waiver found: tooling-isolation-perf-governance-hardening" >&2
  fi
  
  if [[ "${WAIVER_FOUND}" -eq 0 ]]; then
    while IFS= read -r path; do
      [[ -z "${path}" ]] && continue
      if echo "${path}" | grep -E -q '^kernel/'; then
        echo "${path}" >> "${KERNEL_TOUCH_TXT}"
        record_violation "kernel_touch_forbidden:${path}"
      fi
    done < "${CHANGED_TXT}"
  fi
fi

NOW="$(ci_now_utc)"
GIT_SHA="$(git -C "${ROOT}" rev-parse HEAD 2>/dev/null || echo NO_GIT)"
CHANGED_COUNT="$(wc -l < "${CHANGED_TXT}" | tr -d ' ' || echo 0)"
TRIGGERED_COUNT="$(wc -l < "${TRIGGERED_TXT}" | tr -d ' ' || echo 0)"
KERNEL_TOUCH_COUNT="$(wc -l < "${KERNEL_TOUCH_TXT}" | tr -d ' ' || echo 0)"
VIOLATIONS_COUNT="$(wc -l < "${VIOLATIONS_TXT}" | tr -d ' ' || echo 0)"

{
  echo "time_utc=${NOW}"
  echo "git_sha=${GIT_SHA}"
  echo "diff_range=${RANGE}"
  echo "triggered=${TRIGGERED}"
  echo "changed_count=${CHANGED_COUNT}"
  echo "triggered_count=${TRIGGERED_COUNT}"
  echo "kernel_touch_count=${KERNEL_TOUCH_COUNT}"
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
    "gate": "tooling-isolation",
    "verdict": "PASS" if violations_count == 0 else "FAIL",
    "violations_count": violations_count,
    "meta": meta,
    "changed_files": read_lines("changed-files.txt"),
    "triggered_files": read_lines("triggered-files.txt"),
    "kernel_touches": read_lines("kernel-touches.txt"),
    "violations": read_lines("violations.txt"),
}
print(json.dumps(out, indent=2, sort_keys=True))
PY

if [[ "${VIOLATIONS_COUNT}" -gt 0 ]]; then
  echo "tooling-isolation: FAIL (${VIOLATIONS_COUNT} violations)"
  echo "See: ${VIOLATIONS_TXT}"
  exit 2
fi

echo "tooling-isolation: PASS"
exit 0
