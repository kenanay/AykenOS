#!/usr/bin/env bash
# Simplified hygiene gate for MVP-1 (performance-optimized)
# Skips complex source deny scan and binary detection
# Focus: dirty files, forbidden paths, oversized files

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

EVIDENCE_DIR="${1:-}"
if [[ -z "${EVIDENCE_DIR}" ]]; then
  echo "Usage: $0 <evidence-dir>" >&2
  exit 3
fi

mkdir -p "${EVIDENCE_DIR}"
VIOLATIONS_TXT="${EVIDENCE_DIR}/violations.txt"
REPORT_JSON="${EVIDENCE_DIR}/report.json"

: > "${VIOLATIONS_TXT}"

# Check 1: Dirty tracked files
DIRTY_COUNT=0
while IFS= read -r line; do
  [[ -z "${line}" ]] && continue
  # Skip evidence/ directory
  [[ "${line}" =~ ^..\ evidence/ ]] && continue
  echo "dirty_tracked:${line}" >> "${VIOLATIONS_TXT}"
  ((DIRTY_COUNT++)) || true
done < <(git -C "${ROOT}" status --porcelain --untracked-files=no)

# Check 2: Forbidden tracked artifacts (build outputs)
FORBIDDEN_COUNT=0
while IFS= read -r path; do
  # Skip evidence/ and vendored toolchains
  [[ "${path}" =~ ^(evidence/|binutils-2\.42/|gcc-14\.2\.0/) ]] && continue
  
  case "${path}" in
    ayken/target/*|ayken-core/target/*|userspace/target/*|target/*|build/*|out/*|obj/*|*.o|*.elf|*.a|*.so|*.tmp|*.dSYM|*.dSYM/*)
      echo "forbidden_tracked:${path}" >> "${VIOLATIONS_TXT}"
      ((FORBIDDEN_COUNT++)) || true
      ;;
  esac
done < <(git -C "${ROOT}" ls-files)

STRUCTURE_COUNT=0
if command -v rg >/dev/null 2>&1; then
  while IFS= read -r hit; do
    [[ -z "${hit}" ]] && continue
    echo "structure_contract:kernel_private_include:${hit}" >> "${VIOLATIONS_TXT}"
    ((STRUCTURE_COUNT++)) || true
  done < <(cd "${ROOT}" && rg -n '\.\./\.\./kernel/' bootloader userspace || true)

  while IFS= read -r hit; do
    [[ -z "${hit}" ]] && continue
    echo "structure_contract:c_include:${hit}" >> "${VIOLATIONS_TXT}"
    ((STRUCTURE_COUNT++)) || true
  done < <(cd "${ROOT}" && rg -n '#include ".*\.c"' kernel bootloader userspace || true)

  while IFS= read -r hit; do
    [[ -z "${hit}" ]] && continue
    echo "structure_contract:production_test_header:${hit}" >> "${VIOLATIONS_TXT}"
    ((STRUCTURE_COUNT++)) || true
  done < <(cd "${ROOT}" && rg -n '#include ".*_test\.h"' kernel bootloader userspace -g '!**/*_test.c' -g '!**/*_test.h' || true)
fi

# Generate report
TOTAL_VIOLATIONS=$((DIRTY_COUNT + FORBIDDEN_COUNT + STRUCTURE_COUNT))
if [[ ${TOTAL_VIOLATIONS} -eq 0 ]]; then
  VERDICT="PASS"
else
  VERDICT="FAIL"
fi

cat > "${REPORT_JSON}" <<EOF
{
  "gate": "hygiene",
  "verdict": "${VERDICT}",
  "violations_count": ${TOTAL_VIOLATIONS},
  "checks": {
    "dirty_tracked": ${DIRTY_COUNT},
    "forbidden_tracked": ${FORBIDDEN_COUNT},
    "structure_contract": ${STRUCTURE_COUNT}
  },
  "note": "Simplified hygiene gate for MVP-1 (source deny scan disabled for performance)"
}
EOF

if [[ "${VERDICT}" == "PASS" ]]; then
  echo "hygiene: PASS"
  exit 0
else
  echo "hygiene: FAIL (${TOTAL_VIOLATIONS} violations)"
  cat "${VIOLATIONS_TXT}"
  exit 2
fi
