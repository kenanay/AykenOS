#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CI_TOOLS="${ROOT}/tools/ci"
source "${CI_TOOLS}/lib.sh"

usage() {
  cat <<'USAGE'
Usage:
  scripts/ci/gate_embedded_elf_hash.sh --evidence-dir evidence/run-<id>/gates/embedded-elf-hash
    [--kernel-profile validation]
    [--user-minimal-mode phase10a2]
    [--embedded-header kernel/include/embedded_elf.h]
    [--embedded-elf userspace/minimal/minimal.elf]
    [--symbol embedded_elf]

Exit codes:
  0: pass
  2: embedded ELF hash drift detected
  3: usage/tooling error
USAGE
}

EVIDENCE_DIR=""
KERNEL_PROFILE="validation"
EXPECTED_USER_MINIMAL_MODE="phase10a2"
EMBEDDED_HEADER_REL="kernel/include/embedded_elf.h"
EMBEDDED_ELF_REL="userspace/minimal/minimal.elf"
EMBEDDED_SYMBOL="embedded_elf"
OBSERVED_USER_MINIMAL_MODE="${USER_MINIMAL_MODE:-}"

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
    --user-minimal-mode)
      EXPECTED_USER_MINIMAL_MODE="$2"
      shift 2
      ;;
    --embedded-header)
      EMBEDDED_HEADER_REL="$2"
      shift 2
      ;;
    --embedded-elf)
      EMBEDDED_ELF_REL="$2"
      shift 2
      ;;
    --symbol)
      EMBEDDED_SYMBOL="$2"
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

if [[ "${OBSERVED_USER_MINIMAL_MODE}" != "${EXPECTED_USER_MINIMAL_MODE}" ]]; then
  echo "FATAL: embedded-elf-hash gate invoked with USER_MINIMAL_MODE=${OBSERVED_USER_MINIMAL_MODE:-unset} (expected=${EXPECTED_USER_MINIMAL_MODE})" >&2
  exit 2
fi

for tool in make python3; do
  if ! command -v "${tool}" >/dev/null 2>&1; then
    echo "ERROR: required tool missing (${tool})" >&2
    exit 3
  fi
done

mkdir -p "${EVIDENCE_DIR}"

BUILD_LOG="${EVIDENCE_DIR}/build.log"
META_TXT="${EVIDENCE_DIR}/meta.txt"
REPORT_JSON="${EVIDENCE_DIR}/report.json"
VIOLATIONS_TXT="${EVIDENCE_DIR}/violations.txt"

: > "${BUILD_LOG}"
: > "${META_TXT}"
: > "${VIOLATIONS_TXT}"

record_violation() {
  echo "$1" >> "${VIOLATIONS_TXT}"
}

MAKE_ARGS=(
  -C "${ROOT}"
  "KERNEL_PROFILE=${KERNEL_PROFILE}"
  "USER_MINIMAL_MODE=${EXPECTED_USER_MINIMAL_MODE}"
)

if ! make "${MAKE_ARGS[@]}" clean-noimg > "${BUILD_LOG}" 2>&1; then
  record_violation "build_failed:clean-noimg"
fi
if ! make "${MAKE_ARGS[@]}" "${EMBEDDED_HEADER_REL}" "${EMBEDDED_ELF_REL}" >> "${BUILD_LOG}" 2>&1; then
  record_violation "build_failed:embedded-elf"
fi

RUN_ID_VAL="${RUN_ID:-unknown}"
NOW_UTC="$(ci_now_utc)"
GIT_SHA="$(git -C "${ROOT}" rev-parse HEAD 2>/dev/null || echo NO_GIT)"

set +e
ROOT_ENV="${ROOT}" \
EMBEDDED_HEADER_REL_ENV="${EMBEDDED_HEADER_REL}" \
EMBEDDED_ELF_REL_ENV="${EMBEDDED_ELF_REL}" \
EMBEDDED_SYMBOL_ENV="${EMBEDDED_SYMBOL}" \
VIOLATIONS_TXT_ENV="${VIOLATIONS_TXT}" \
META_TXT_ENV="${META_TXT}" \
REPORT_JSON_ENV="${REPORT_JSON}" \
RUN_ID_ENV="${RUN_ID_VAL}" \
NOW_UTC_ENV="${NOW_UTC}" \
GIT_SHA_ENV="${GIT_SHA}" \
KERNEL_PROFILE_ENV="${KERNEL_PROFILE}" \
USER_MINIMAL_MODE_ENV="${EXPECTED_USER_MINIMAL_MODE}" \
python3 - <<'PY'
import hashlib
import json
import os
import re
from pathlib import Path

ROOT = Path(os.environ["ROOT_ENV"])
header_rel = os.environ["EMBEDDED_HEADER_REL_ENV"]
elf_rel = os.environ["EMBEDDED_ELF_REL_ENV"]
symbol = os.environ["EMBEDDED_SYMBOL_ENV"]
violations_path = Path(os.environ["VIOLATIONS_TXT_ENV"])
meta_path = Path(os.environ["META_TXT_ENV"])
report_path = Path(os.environ["REPORT_JSON_ENV"])
run_id = os.environ["RUN_ID_ENV"]
now_utc = os.environ["NOW_UTC_ENV"]
git_sha = os.environ["GIT_SHA_ENV"]
kernel_profile = os.environ["KERNEL_PROFILE_ENV"]
user_minimal_mode = os.environ["USER_MINIMAL_MODE_ENV"]

header_path = ROOT / header_rel
elf_path = ROOT / elf_rel

def read_lines(path: Path) -> list[str]:
    if not path.exists():
        return []
    return [line.strip() for line in path.read_text(encoding="utf-8", errors="replace").splitlines() if line.strip()]

violations = read_lines(violations_path)

def add_violation(msg: str) -> None:
    if msg not in violations:
        violations.append(msg)

header_hash = ""
elf_hash = ""

if not header_path.is_file():
    add_violation(f"missing_file:{header_rel}")
if not elf_path.is_file():
    add_violation(f"missing_file:{elf_rel}")

if header_path.is_file() and elf_path.is_file():
    elf_hash = hashlib.sha256(elf_path.read_bytes()).hexdigest()
    header_text = header_path.read_text(encoding="utf-8", errors="replace")
    pattern = re.compile(
        rf'static const char\s+{re.escape(symbol)}_sha256\[\]\s*=\s*"([0-9a-fA-F]{{64}})";'
    )
    match = pattern.search(header_text)
    if not match:
        add_violation(f"header_hash_symbol_missing:{symbol}_sha256")
    else:
        header_hash = match.group(1).lower()
        if header_hash != elf_hash:
            add_violation(
                f"embedded_elf_hash_mismatch:header={header_hash}:elf={elf_hash}"
            )

violations_path.write_text(
    "".join(f"{line}\n" for line in violations),
    encoding="utf-8",
)

verdict = "PASS" if not violations else "FAIL"
meta = {
    "run_id": run_id,
    "time_utc": now_utc,
    "git_sha": git_sha,
    "kernel_profile": kernel_profile,
    "user_minimal_mode": user_minimal_mode,
    "embedded_header": header_rel,
    "embedded_elf": elf_rel,
    "embedded_symbol": symbol,
    "header_sha256": header_hash,
    "elf_sha256": elf_hash,
    "violations_count": len(violations),
}
meta_path.write_text(
    "".join(f"{k}={v}\n" for k, v in meta.items()),
    encoding="utf-8",
)

report = {
    "gate": "embedded-elf-hash",
    "verdict": verdict,
    "violations_count": len(violations),
    "checks": {
        "header_hash_present": bool(header_hash),
        "header_matches_elf": bool(header_hash) and bool(elf_hash) and header_hash == elf_hash,
    },
    "meta": meta,
    "violations": violations,
}
report_path.write_text(
    json.dumps(report, indent=2, sort_keys=True) + "\n",
    encoding="utf-8",
)

raise SystemExit(0 if verdict == "PASS" else 2)
PY
GATE_RC=$?
set -e

if [[ "${GATE_RC}" -eq 0 ]]; then
  echo "embedded-elf-hash: PASS"
  exit 0
fi

echo "embedded-elf-hash: FAIL"
echo "See: ${VIOLATIONS_TXT}"
exit "${GATE_RC}"
