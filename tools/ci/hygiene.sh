#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CI_TOOLS="${ROOT}/tools/ci"
source "${CI_TOOLS}/lib.sh"

usage() {
  cat <<'USAGE'
Usage:
  tools/ci/hygiene.sh --evidence-dir evidence/run-<id>/gates/hygiene

Exit codes:
  0: pass
  2: hygiene violations found
  3: usage/tooling error
USAGE
}

EVIDENCE_DIR=""
while [[ $# -gt 0 ]]; do
  case "$1" in
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

if [[ -z "${EVIDENCE_DIR}" ]]; then
  usage
  exit 3
fi

if ! command -v git >/dev/null 2>&1; then
  echo "ERROR: git not found" >&2
  exit 3
fi

mkdir -p "${EVIDENCE_DIR}"
TRACKED_TXT="${EVIDENCE_DIR}/tracked.files.txt"
FORBIDDEN_TXT="${EVIDENCE_DIR}/forbidden-tracked.txt"
DIRTY_TRACKED_TXT="${EVIDENCE_DIR}/dirty-tracked.txt"
VIOLATIONS_TXT="${EVIDENCE_DIR}/violations.txt"
META_TXT="${EVIDENCE_DIR}/meta.txt"
REPORT_JSON="${EVIDENCE_DIR}/report.json"

: > "${TRACKED_TXT}"
: > "${FORBIDDEN_TXT}"
: > "${DIRTY_TRACKED_TXT}"
: > "${VIOLATIONS_TXT}"

git -C "${ROOT}" ls-files > "${TRACKED_TXT}"
git -C "${ROOT}" status --porcelain --untracked-files=no > "${DIRTY_TRACKED_TXT}"

# Detect forbidden tracked artifact paths.
while IFS= read -r path; do
  case "${path}" in
    # Vendored toolchain sources may legitimately contain .o/.a/.d test fixtures.
    binutils-2.42/*|gcc-14.2.0/*)
      continue
      ;;
  esac

  case "${path}" in
    ayken/target/*|ayken-core/target/*|target/*|build/*|obj/*|*.o|*.elf|*.a|*.so|*.tmp|*.dSYM|*.dSYM/*)
      echo "${path}" >> "${FORBIDDEN_TXT}"
      ;;
  esac
done < "${TRACKED_TXT}"

# Consolidate violations with reason prefixes.
if [[ -s "${FORBIDDEN_TXT}" ]]; then
  while IFS= read -r line; do
    [[ -z "${line}" ]] && continue
    echo "forbidden_tracked:${line}" >> "${VIOLATIONS_TXT}"
  done < "${FORBIDDEN_TXT}"
fi

if [[ -s "${DIRTY_TRACKED_TXT}" ]]; then
  while IFS= read -r line; do
    [[ -z "${line}" ]] && continue
    echo "dirty_tracked:${line}" >> "${VIOLATIONS_TXT}"
  done < "${DIRTY_TRACKED_TXT}"
fi

NOW="$(ci_now_utc)"
GIT_SHA="$(git -C "${ROOT}" rev-parse HEAD 2>/dev/null || echo "NO_GIT")"
TRACKED_COUNT="$(wc -l < "${TRACKED_TXT}" | tr -d ' ')"
FORBIDDEN_COUNT="$(wc -l < "${FORBIDDEN_TXT}" | tr -d ' ')"
DIRTY_TRACKED_COUNT="$(wc -l < "${DIRTY_TRACKED_TXT}" | tr -d ' ')"
VIOLATIONS_COUNT="$(wc -l < "${VIOLATIONS_TXT}" | tr -d ' ')"

{
  echo "time_utc=${NOW}"
  echo "git_sha=${GIT_SHA}"
  echo "tracked_count=${TRACKED_COUNT}"
  echo "forbidden_tracked_count=${FORBIDDEN_COUNT}"
  echo "dirty_tracked_count=${DIRTY_TRACKED_COUNT}"
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
    "gate": "hygiene",
    "verdict": "PASS" if violations_count == 0 else "FAIL",
    "violations_count": violations_count,
    "meta": meta,
    "forbidden_tracked": read_lines("forbidden-tracked.txt"),
    "dirty_tracked": read_lines("dirty-tracked.txt"),
    "violations": read_lines("violations.txt"),
}
print(json.dumps(out, indent=2, sort_keys=True))
PY

if [[ "${VIOLATIONS_COUNT}" -gt 0 ]]; then
  echo "hygiene: FAIL (${VIOLATIONS_COUNT} violations)"
  echo "See: ${VIOLATIONS_TXT}"
  exit 2
fi

echo "hygiene: PASS"
exit 0
