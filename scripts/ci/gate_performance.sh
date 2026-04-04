#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CI_TOOLS="${ROOT}/tools/ci"
source "${CI_TOOLS}/lib.sh"
source "${ROOT}/scripts/ci/lib-drift-persistence.sh"
source "${ROOT}/scripts/ci/lib-drift-allowlist.sh"

usage() {
  cat <<'USAGE'
Usage:
  scripts/ci/gate_performance.sh --evidence-dir evidence/run-<id>/gates/performance
    [--baseline-file scripts/ci/perf-baseline.lock.json]
    [--drift-allowlist-file constitution/drift_blocking_allowlist.json]
    [--kernel-profile validation]
    [--qemu-timeout 30]
    [--env-mismatch-policy fail|waiver]
    [--init-baseline]

Env controls:
  PERF_BASELINE_AUTHORITY=<id>                 (default: scripts/ci/perf_authority.env)
  PERF_REQUIRE_CI_FOR_BASELINE_INIT=0|1        (default: 1)
  PERF_CI_IMAGE_DIGEST=<digest-or-build-id>    (default: unknown)
  PERF_ALLOW_UNTRACKED_BASELINE=0|1            (default: 0)
  PERF_BOOT_THRESHOLD_PERCENT=<pct>            (default: 10)
  PERF_CONTEXT_THRESHOLD_PERCENT=<pct>         (default: 5)
  PERF_SYSCALL_THRESHOLD_PERCENT=<pct>         (default: 5)
  PERF_BASELINE_MODE=constitutional|provisional (default: constitutional)
  PERF_REGRESSION_POLICY=fail                   (default: fail)
  PERF_ENV_MISMATCH_POLICY=fail|waiver          (default: fail)
  PERF_PREEMPT_FORCE_EFI_REBUILD=0|1           (default: 1)
  PERF_PREEMPT_USER_MINIMAL_MODE=<mode>        (default: syscall-v2-runtime)
  PERF_PREEMPT_BOOTSTRAP_POLICY=0|1            (default: 1)
  PERF_PREEMPT_MB_SELFTEST=0|1                 (default: 0)
  PERF_PREEMPT_DETERMINISTIC_EXIT=0|1          (default: 1)
  PERF_PREEMPT_RING3_ENTRY_GUARD=0|1           (default: 1 for syscall-v2-runtime, else 0)
  PERF_PREEMPT_BUILD_DEBUG_SCHED=0|1           (default: make profile)
  PERF_PREEMPT_BUILD_DEBUG_IRQ=0|1             (default: make profile)
  PERF_PREEMPT_EXPECTED_QEMU_EXIT_SET=csv      (default: 0,1)
  DRIFT_ALLOWLIST_FILE=<path>                  (default: constitution/drift_blocking_allowlist.json)

Exit codes:
  0: pass
  2: performance baseline mismatch / regression
  3: usage/tooling error
USAGE
}

EVIDENCE_DIR=""
BASELINE_FILE="${ROOT}/scripts/ci/perf-baseline.lock.json"
DRIFT_ALLOWLIST_FILE="${DRIFT_ALLOWLIST_FILE:-${ROOT}/constitution/drift_blocking_allowlist.json}"
PERF_AUTHORITY_ENV_FILE="${ROOT}/scripts/ci/perf_authority.env"
PERF_AUTHORITY_DEFAULT="$(sed -n 's/^PERF_BASELINE_AUTHORITY=//p' "${PERF_AUTHORITY_ENV_FILE}" 2>/dev/null | head -n1 || true)"
PERF_AUTHORITY_DEFAULT="${PERF_AUTHORITY_DEFAULT:-github-hosted-ubuntu-24.04-x64}"
KERNEL_PROFILE="${PERF_KERNEL_PROFILE:-validation}"
QEMU_TIMEOUT="${PERF_QEMU_TIMEOUT:-30}"
ENV_MISMATCH_POLICY="${PERF_ENV_MISMATCH_POLICY:-fail}"
REGRESSION_POLICY="${PERF_REGRESSION_POLICY:-fail}"
BASELINE_MODE="${PERF_BASELINE_MODE:-constitutional}"
BASELINE_AUTHORITY="${PERF_BASELINE_AUTHORITY:-${PERF_AUTHORITY_DEFAULT}}"
REQUIRE_CI_FOR_BASELINE_INIT="${PERF_REQUIRE_CI_FOR_BASELINE_INIT:-1}"
CI_IMAGE_DIGEST="${PERF_CI_IMAGE_DIGEST:-unknown}"
ALLOW_UNTRACKED_BASELINE="${PERF_ALLOW_UNTRACKED_BASELINE:-0}"
BOOT_THRESHOLD_PERCENT="${PERF_BOOT_THRESHOLD_PERCENT:-10}"
CONTEXT_THRESHOLD_PERCENT="${PERF_CONTEXT_THRESHOLD_PERCENT:-5}"
SYSCALL_THRESHOLD_PERCENT="${PERF_SYSCALL_THRESHOLD_PERCENT:-5}"
PREEMPT_FORCE_EFI_REBUILD="${PERF_PREEMPT_FORCE_EFI_REBUILD:-1}"
PREEMPT_USER_MINIMAL_MODE="${PERF_PREEMPT_USER_MINIMAL_MODE:-syscall-v2-runtime}"
PREEMPT_BOOTSTRAP_POLICY="${PERF_PREEMPT_BOOTSTRAP_POLICY:-1}"
PREEMPT_MB_SELFTEST="${PERF_PREEMPT_MB_SELFTEST:-0}"
PREEMPT_DETERMINISTIC_EXIT="${PERF_PREEMPT_DETERMINISTIC_EXIT:-1}"
PREEMPT_RING3_ENTRY_GUARD="${PERF_PREEMPT_RING3_ENTRY_GUARD:-}"
PREEMPT_BUILD_DEBUG_SCHED="${PERF_PREEMPT_BUILD_DEBUG_SCHED:-}"
PREEMPT_BUILD_DEBUG_IRQ="${PERF_PREEMPT_BUILD_DEBUG_IRQ:-}"
PREEMPT_EXPECTED_QEMU_EXIT_SET="${PERF_PREEMPT_EXPECTED_QEMU_EXIT_SET:-0,1}"
MEASUREMENT_CONTRACT="deterministic_preempt_harness"
SCHED_FALLBACK="${AYKEN_SCHED_FALLBACK:-0}"
BOOT_OK_MARKER="[K][BOOT_OK] Phase 4.4 minimal boot reached"
PREEMPT_SW_COUNT_PATTERN='[SW|MARK:SW] count:'
PREEMPT_IRET_COUNT_PATTERN='[IRET markers] count:'
INIT_BASELINE=0

if [[ -z "${PREEMPT_RING3_ENTRY_GUARD}" ]]; then
  if [[ "${PREEMPT_USER_MINIMAL_MODE}" == "syscall-v2-runtime" ]]; then
    PREEMPT_RING3_ENTRY_GUARD="1"
  else
    PREEMPT_RING3_ENTRY_GUARD="0"
  fi
fi

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
    --drift-allowlist-file)
      DRIFT_ALLOWLIST_FILE="$2"
      shift 2
      ;;
    --kernel-profile)
      KERNEL_PROFILE="$2"
      shift 2
      ;;
    --qemu-timeout)
      QEMU_TIMEOUT="$2"
      shift 2
      ;;
    --env-mismatch-policy)
      ENV_MISMATCH_POLICY="$2"
      shift 2
      ;;
    --init-baseline)
      INIT_BASELINE=1
      shift
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

case "${ENV_MISMATCH_POLICY}" in
  fail|waiver)
    ;;
  *)
    echo "ERROR: --env-mismatch-policy must be fail or waiver" >&2
    exit 3
    ;;
esac

case "${REQUIRE_CI_FOR_BASELINE_INIT}" in
  0|1)
    ;;
  *)
    echo "ERROR: PERF_REQUIRE_CI_FOR_BASELINE_INIT must be 0 or 1" >&2
    exit 3
    ;;
esac

case "${ALLOW_UNTRACKED_BASELINE}" in
  0|1)
    ;;
  *)
    echo "ERROR: PERF_ALLOW_UNTRACKED_BASELINE must be 0 or 1" >&2
    exit 3
    ;;
esac

case "${PREEMPT_FORCE_EFI_REBUILD}" in
  0|1)
    ;;
  *)
    echo "ERROR: PERF_PREEMPT_FORCE_EFI_REBUILD must be 0 or 1" >&2
    exit 3
    ;;
esac

case "${PREEMPT_BOOTSTRAP_POLICY}" in
  0|1)
    ;;
  *)
    echo "ERROR: PERF_PREEMPT_BOOTSTRAP_POLICY must be 0 or 1" >&2
    exit 3
    ;;
esac

case "${PREEMPT_MB_SELFTEST}" in
  0|1)
    ;;
  *)
    echo "ERROR: PERF_PREEMPT_MB_SELFTEST must be 0 or 1" >&2
    exit 3
    ;;
esac

case "${PREEMPT_DETERMINISTIC_EXIT}" in
  0|1)
    ;;
  *)
    echo "ERROR: PERF_PREEMPT_DETERMINISTIC_EXIT must be 0 or 1" >&2
    exit 3
    ;;
esac

case "${PREEMPT_RING3_ENTRY_GUARD}" in
  0|1)
    ;;
  *)
    echo "ERROR: PERF_PREEMPT_RING3_ENTRY_GUARD must be 0 or 1" >&2
    exit 3
    ;;
esac

if [[ -n "${PREEMPT_BUILD_DEBUG_SCHED}" ]] && [[ "${PREEMPT_BUILD_DEBUG_SCHED}" != "0" && "${PREEMPT_BUILD_DEBUG_SCHED}" != "1" ]]; then
  echo "ERROR: PERF_PREEMPT_BUILD_DEBUG_SCHED must be 0 or 1 when set" >&2
  exit 3
fi

if [[ -n "${PREEMPT_BUILD_DEBUG_IRQ}" ]] && [[ "${PREEMPT_BUILD_DEBUG_IRQ}" != "0" && "${PREEMPT_BUILD_DEBUG_IRQ}" != "1" ]]; then
  echo "ERROR: PERF_PREEMPT_BUILD_DEBUG_IRQ must be 0 or 1 when set" >&2
  exit 3
fi

if [[ ! "${PREEMPT_EXPECTED_QEMU_EXIT_SET}" =~ ^[0-9]+(,[0-9]+)*$ ]]; then
  echo "ERROR: PERF_PREEMPT_EXPECTED_QEMU_EXIT_SET must be comma-separated exit codes (got '${PREEMPT_EXPECTED_QEMU_EXIT_SET}')" >&2
  exit 3
fi

# ----------------------------------------------------------------------------
# Baseline Lock Immutability (PR guard)
# - PR'larda baseline lock dosyası değiştirilemez.
# - Sadece perf-baseline-init gibi yetkili akışlar bypass edebilir.
# ----------------------------------------------------------------------------
is_pr=false
if [[ "${GITHUB_EVENT_NAME:-}" == "pull_request" || "${GITHUB_EVENT_NAME:-}" == "pull_request_target" ]]; then
  is_pr=true
fi

if [[ "${is_pr}" == "true" && "${PERF_ALLOW_BASELINE_LOCK_MUTATION:-0}" != "1" ]]; then
  # origin/main yoksa fetch etmeyi dene (actions/checkout fetch-depth=1 ise gerekebilir)
  git -C "${ROOT}" fetch --no-tags --depth=1 origin main >/dev/null 2>&1 || true
  
  # Baseline dosyası değişmiş mi?
  if git -C "${ROOT}" diff --name-only "origin/main...HEAD" -- "${BASELINE_FILE}" 2>/dev/null | grep -q .; then
    echo "performance: FAIL (baseline lock immutability violation)"
    echo "Baseline lock file mutated in PR: ${BASELINE_FILE}"
    echo "If this change is intentional, regenerate via perf-baseline-init workflow (authorized path)."
    exit 2
  fi
fi

is_pinned_ci_digest() {
  local digest="$1"
  local authority="$2"
  if [[ -z "${digest}" || "${digest}" == "unknown" || "${digest}" == *unknown* ]]; then
    return 1
  fi
  if [[ "${authority}" == github-hosted-* ]]; then
    if [[ ! "${digest}" =~ ^gha-[A-Za-z0-9._-]+-[A-Za-z0-9._-]+-[A-Za-z0-9._-]+$ ]]; then
      return 1
    fi
  fi
  return 0
}

is_expected_qemu_exit_rc() {
  local rc="$1"
  local expected_csv="$2"
  local code
  IFS=',' read -r -a expected_codes <<< "${expected_csv}"
  for code in "${expected_codes[@]}"; do
    code="${code//[[:space:]]/}"
    if [[ -n "${code}" && "${rc}" == "${code}" ]]; then
      return 0
    fi
  done
  return 1
}

is_nonnegative_number() {
  [[ "$1" =~ ^[0-9]+([.][0-9]+)?$ ]]
}

for threshold_value in "${BOOT_THRESHOLD_PERCENT}" "${CONTEXT_THRESHOLD_PERCENT}" "${SYSCALL_THRESHOLD_PERCENT}"; do
  if ! is_nonnegative_number "${threshold_value}"; then
    echo "ERROR: performance thresholds must be non-negative numbers" >&2
    exit 3
  fi
done

for tool in git make python3 qemu-system-x86_64 jq; do
  if ! command -v "${tool}" >/dev/null 2>&1; then
    echo "ERROR: required tool missing (${tool})" >&2
    exit 3
  fi
done

if ! validate_drift_allowlist "${DRIFT_ALLOWLIST_FILE}"; then
  exit 3
fi
DRIFT_ALLOWLIST_VERSION="$(jq -r '.version' "${DRIFT_ALLOWLIST_FILE}")"

now_ms() {
  python3 - <<'PY'
import time
print(int(time.time() * 1000))
PY
}

extract_kv_metric() {
  local key="$1"
  local file="$2"
  [[ -f "${file}" ]] || { echo 0; return; }
  awk -F '=' -v k="${key}" '
    $1==k {v=$2}
    END {
      if (v == "") {
        print 0
      } else {
        gsub(/[^0-9.]/, "", v)
        if (v == "") print 0
        else print v + 0
      }
    }
  ' "${file}"
}

extract_kv_text() {
  local key="$1"
  local file="$2"
  [[ -f "${file}" ]] || { echo ""; return; }
  awk -F '=' -v k="${key}" '
    $1==k {
      v=$2
      for (i=3; i<=NF; ++i) v=v"="$i
    }
    END { print v }
  ' "${file}"
}

extract_label_count() {
  local label="$1"
  local file="$2"
  [[ -f "${file}" ]] || { echo 0; return; }
  awk -v lbl="${label}" '
    index($0, lbl) {v=$0}
    END {
      gsub(/[^0-9]/, "", v)
      if (v == "") print 0
      else print v + 0
    }
  ' "${file}"
}

mkdir -p "${EVIDENCE_DIR}"
ENV_JSON="${EVIDENCE_DIR}/env.json"
RESULTS_JSON="${EVIDENCE_DIR}/results.json"
BUILD_LOG="${EVIDENCE_DIR}/build.log"
RAW_LOG="${EVIDENCE_DIR}/raw.log"
BOOT_AUDIT_LOG="${EVIDENCE_DIR}/boot-audit.log"
PREEMPT_LOG="${EVIDENCE_DIR}/preempt.log"
PREEMPT_METRICS_TXT="${EVIDENCE_DIR}/preempt.metrics.txt"
PREEMPT_ANALYSIS_LOG="${EVIDENCE_DIR}/preempt.analysis.log"
ACTUAL_LOCK_JSON="${EVIDENCE_DIR}/actual.lock.json"
BASELINE_DIFF_TXT="${EVIDENCE_DIR}/baseline.diff.txt"
VIOLATIONS_TXT="${EVIDENCE_DIR}/violations.txt"
ALLOWLIST_BYPASS_TXT="${EVIDENCE_DIR}/allowlist_bypass.txt"
META_TXT="${EVIDENCE_DIR}/meta.txt"
REPORT_JSON="${EVIDENCE_DIR}/report.json"

: > "${ENV_JSON}"
: > "${RESULTS_JSON}"
: > "${BUILD_LOG}"
: > "${RAW_LOG}"
: > "${BOOT_AUDIT_LOG}"
: > "${PREEMPT_LOG}"
: > "${PREEMPT_METRICS_TXT}"
: > "${PREEMPT_ANALYSIS_LOG}"
: > "${ACTUAL_LOCK_JSON}"
: > "${BASELINE_DIFF_TXT}"
: > "${VIOLATIONS_TXT}"
: > "${ALLOWLIST_BYPASS_TXT}"

record_violation() {
  echo "$1" >> "${VIOLATIONS_TXT}"
}

# 1) Capture environment manifest + hash.
CLANG_VERSION="$(clang --version 2>/dev/null | head -n1 || echo missing)"
LD_VERSION="$(ld.lld --version 2>/dev/null | head -n1 || echo missing)"
NASM_VERSION="$(nasm -v 2>/dev/null || echo missing)"
QEMU_VERSION="$(qemu-system-x86_64 --version 2>/dev/null | head -n1 || echo missing)"
HOST_OS="$(uname -s 2>/dev/null || echo unknown)"
HOST_ARCH="$(uname -m 2>/dev/null || echo unknown)"

CLANG_VERSION_ENV="${CLANG_VERSION}" \
LD_VERSION_ENV="${LD_VERSION}" \
NASM_VERSION_ENV="${NASM_VERSION}" \
QEMU_VERSION_ENV="${QEMU_VERSION}" \
HOST_OS_ENV="${HOST_OS}" \
HOST_ARCH_ENV="${HOST_ARCH}" \
KERNEL_PROFILE_ENV="${KERNEL_PROFILE}" \
QEMU_TIMEOUT_ENV="${QEMU_TIMEOUT}" \
BASELINE_AUTHORITY_ENV="${BASELINE_AUTHORITY}" \
CI_IMAGE_DIGEST_ENV="${CI_IMAGE_DIGEST}" \
BOOT_OK_MARKER_ENV="${BOOT_OK_MARKER}" \
PREEMPT_SW_COUNT_PATTERN_ENV="${PREEMPT_SW_COUNT_PATTERN}" \
PREEMPT_IRET_COUNT_PATTERN_ENV="${PREEMPT_IRET_COUNT_PATTERN}" \
PREEMPT_USER_MINIMAL_MODE_ENV="${PREEMPT_USER_MINIMAL_MODE}" \
PREEMPT_BOOTSTRAP_POLICY_ENV="${PREEMPT_BOOTSTRAP_POLICY}" \
PREEMPT_MB_SELFTEST_ENV="${PREEMPT_MB_SELFTEST}" \
PREEMPT_DETERMINISTIC_EXIT_ENV="${PREEMPT_DETERMINISTIC_EXIT}" \
PREEMPT_RING3_ENTRY_GUARD_ENV="${PREEMPT_RING3_ENTRY_GUARD}" \
PREEMPT_EXPECTED_QEMU_EXIT_SET_ENV="${PREEMPT_EXPECTED_QEMU_EXIT_SET}" \
MEASUREMENT_CONTRACT_ENV="${MEASUREMENT_CONTRACT}" \
ENV_JSON_ENV="${ENV_JSON}" \
python3 - <<'PY'
import hashlib
import json
import os

out = os.environ["ENV_JSON_ENV"]
payload = {
    "kernel_profile": os.environ["KERNEL_PROFILE_ENV"],
    "target_triple": "x86_64-elf",
    "qemu_timeout_seconds": int(os.environ["QEMU_TIMEOUT_ENV"]),
    "clang_version": os.environ["CLANG_VERSION_ENV"],
    "ld_version": os.environ["LD_VERSION_ENV"],
    "nasm_version": os.environ["NASM_VERSION_ENV"],
    "qemu_version": os.environ["QEMU_VERSION_ENV"],
    "host_os": os.environ["HOST_OS_ENV"],
    "host_arch": os.environ["HOST_ARCH_ENV"],
    "baseline_authority": os.environ["BASELINE_AUTHORITY_ENV"],
    "ci_image_digest": os.environ["CI_IMAGE_DIGEST_ENV"],
    "marker_contract": {
        "boot_ok_marker": os.environ["BOOT_OK_MARKER_ENV"],
        "preempt_sw_count_pattern": os.environ["PREEMPT_SW_COUNT_PATTERN_ENV"],
        "preempt_iret_count_pattern": os.environ["PREEMPT_IRET_COUNT_PATTERN_ENV"],
        "measurement_contract": os.environ["MEASUREMENT_CONTRACT_ENV"],
        # Performance gate measures deterministic harness mode, not constitutional default.
        "preempt_user_minimal_mode": os.environ["PREEMPT_USER_MINIMAL_MODE_ENV"],
        "preempt_bootstrap_policy": int(os.environ["PREEMPT_BOOTSTRAP_POLICY_ENV"]),
        "preempt_mb_selftest": int(os.environ["PREEMPT_MB_SELFTEST_ENV"]),
        "preempt_deterministic_exit": int(os.environ["PREEMPT_DETERMINISTIC_EXIT_ENV"]),
        "preempt_ring3_entry_guard": int(os.environ["PREEMPT_RING3_ENTRY_GUARD_ENV"]),
        "preempt_expected_qemu_exit_set": os.environ["PREEMPT_EXPECTED_QEMU_EXIT_SET_ENV"],
    },
}
hash_payload = dict(payload)
if str(payload.get("baseline_authority", "")).startswith("github-hosted-"):
    # GitHub-hosted ubuntu labels can rotate between image build digests even
    # when the effective toolchain surface stays identical. Keep the digest for
    # audit, but hash the stable authority + explicit tool versions instead.
    hash_payload.pop("ci_image_digest", None)
canonical = json.dumps(hash_payload, sort_keys=True, separators=(",", ":"))
payload["env_hash"] = hashlib.sha256(canonical.encode("utf-8")).hexdigest()
with open(out, "w", encoding="utf-8") as fh:
    json.dump(payload, fh, indent=2, sort_keys=True)
    fh.write("\n")
PY

ENV_HASH="$(python3 - <<'PY' "${ENV_JSON}"
import json, sys
print(json.load(open(sys.argv[1], encoding="utf-8"))["env_hash"])
PY
)"

# 2) Build the authoritative boot image under the exact preempt contract.
PREEMPT_BUILD_ARGS=(
  "KERNEL_PROFILE=${KERNEL_PROFILE}"
  "USER_MINIMAL_MODE=${PREEMPT_USER_MINIMAL_MODE}"
  "AYKEN_SCHED_BOOTSTRAP_POLICY=${PREEMPT_BOOTSTRAP_POLICY}"
  "AYKEN_MB_SELFTEST=${PREEMPT_MB_SELFTEST}"
  "AYKEN_DETERMINISTIC_EXIT=${PREEMPT_DETERMINISTIC_EXIT}"
  "AYKEN_RING3_ENTRY_GUARD=${PREEMPT_RING3_ENTRY_GUARD}"
)
if [[ -n "${PREEMPT_BUILD_DEBUG_SCHED}" ]]; then
  PREEMPT_BUILD_ARGS+=("AYKEN_DEBUG_SCHED=${PREEMPT_BUILD_DEBUG_SCHED}")
fi
if [[ -n "${PREEMPT_BUILD_DEBUG_IRQ}" ]]; then
  PREEMPT_BUILD_ARGS+=("AYKEN_DEBUG_IRQ=${PREEMPT_BUILD_DEBUG_IRQ}")
fi

{
  echo "[PERF] authoritative preempt build contract"
  echo "[PERF]   user_minimal_mode=${PREEMPT_USER_MINIMAL_MODE}"
  echo "[PERF]   bootstrap_policy=${PREEMPT_BOOTSTRAP_POLICY}"
  echo "[PERF]   mb_selftest=${PREEMPT_MB_SELFTEST}"
  echo "[PERF]   deterministic_exit=${PREEMPT_DETERMINISTIC_EXIT}"
  echo "[PERF]   ring3_entry_guard=${PREEMPT_RING3_ENTRY_GUARD}"
  echo "[PERF]   debug_sched=${PREEMPT_BUILD_DEBUG_SCHED:-<make-default>}"
  echo "[PERF]   debug_irq=${PREEMPT_BUILD_DEBUG_IRQ:-<make-default>}"
} >> "${BUILD_LOG}"

if [[ "${PREEMPT_FORCE_EFI_REBUILD}" == "1" ]]; then
  if ! make -C "${ROOT}" "${PREEMPT_BUILD_ARGS[@]}" clean >> "${BUILD_LOG}" 2>&1; then
    record_violation "build_failed:make clean"
  fi
fi
if ! make -C "${ROOT}" "${PREEMPT_BUILD_ARGS[@]}" efi-img >> "${BUILD_LOG}" 2>&1; then
  record_violation "build_failed:make efi-img"
fi
cp -f "${BUILD_LOG}" "${RAW_LOG}" 2>/dev/null || true

# 3) Measure boot marker time (proxy: boot audit wall duration).
BOOT_START_MS="$(now_ms)"
if ! (cd "${ROOT}" && tools/validation/phase_4_4_qemu_boot_audit.sh \
  --timeout "${QEMU_TIMEOUT}" \
  --marker "${BOOT_OK_MARKER}" \
  --out-dir "${EVIDENCE_DIR}/boot-audit") > "${BOOT_AUDIT_LOG}" 2>&1; then
  record_violation "boot_audit_failed:phase_4_4_qemu_boot_audit.sh"
fi
BOOT_END_MS="$(now_ms)"
BOOT_TIME_MS="$((BOOT_END_MS - BOOT_START_MS))"

# 4) Measure context-switch proxy from preempt validation.
PREEMPT_START_MS="$(now_ms)"
PREEMPT_TEST_ENV=(
  "QEMU_TIMEOUT=${QEMU_TIMEOUT}"
  "STRICT_MARKERS=1"
  "FORCE_EFI_REBUILD=0"
  "PREEMPT_CLEAN_REBUILD=0"
  "KERNEL_PROFILE=${KERNEL_PROFILE}"
  "USER_MINIMAL_MODE=${PREEMPT_USER_MINIMAL_MODE}"
  "AYKEN_SCHED_BOOTSTRAP_POLICY=${PREEMPT_BOOTSTRAP_POLICY}"
  "AYKEN_MB_SELFTEST=${PREEMPT_MB_SELFTEST}"
  "AYKEN_DETERMINISTIC_EXIT=${PREEMPT_DETERMINISTIC_EXIT}"
  "AYKEN_RING3_ENTRY_GUARD=${PREEMPT_RING3_ENTRY_GUARD}"
  "PREEMPT_METRICS_OUT=${PREEMPT_METRICS_TXT}"
  "PREEMPT_ANALYSIS_LOG_OUT=${PREEMPT_ANALYSIS_LOG}"
)
if [[ -n "${PREEMPT_BUILD_DEBUG_SCHED}" ]]; then
  PREEMPT_TEST_ENV+=("AYKEN_DEBUG_SCHED=${PREEMPT_BUILD_DEBUG_SCHED}")
fi
if [[ -n "${PREEMPT_BUILD_DEBUG_IRQ}" ]]; then
  PREEMPT_TEST_ENV+=("AYKEN_DEBUG_IRQ=${PREEMPT_BUILD_DEBUG_IRQ}")
fi
if ! (cd "${ROOT}" && env "${PREEMPT_TEST_ENV[@]}" ./run_preempt_test.sh) > "${PREEMPT_LOG}" 2>&1; then
  record_violation "preempt_test_failed:run_preempt_test.sh"
fi
PREEMPT_END_MS="$(now_ms)"
PREEMPT_TIME_MS_WALL="$((PREEMPT_END_MS - PREEMPT_START_MS))"

PREEMPT_SW_COUNT="$(extract_kv_metric "sw_count" "${PREEMPT_METRICS_TXT}")"
PREEMPT_IRET_COUNT="$(extract_kv_metric "iret_count" "${PREEMPT_METRICS_TXT}")"
PREEMPT_QEMU_RUN_TIME_MS="$(extract_kv_metric "qemu_run_time_ms" "${PREEMPT_METRICS_TXT}")"
MARK_SW_COUNT="$(extract_kv_metric "mark_sw_count" "${PREEMPT_METRICS_TXT}")"
MARK_IRET_COUNT="$(extract_kv_metric "mark_iret_count" "${PREEMPT_METRICS_TXT}")"
SCHED_IDLE_COUNT="$(extract_kv_metric "sched_idle_count" "${PREEMPT_METRICS_TXT}")"
STAGE_HINT_MISSING_SIGNAL="$(extract_kv_metric "stage_hint_missing" "${PREEMPT_METRICS_TXT}")"
PREEMPT_QEMU_EXIT_RC="$(extract_kv_metric "qemu_exit_rc" "${PREEMPT_METRICS_TXT}")"
PREEMPT_QEMU_TIMEOUT_HIT="$(extract_kv_metric "qemu_timeout_hit" "${PREEMPT_METRICS_TXT}")"
PREEMPT_PROOF_DONE_SEEN="$(extract_kv_metric "proof_done_seen" "${PREEMPT_METRICS_TXT}")"
PREEMPT_CONTRACT_USER_MODE="$(extract_kv_text "contract_user_minimal_mode" "${PREEMPT_METRICS_TXT}")"
PREEMPT_CONTRACT_BOOTSTRAP="$(extract_kv_text "contract_bootstrap_policy" "${PREEMPT_METRICS_TXT}")"
PREEMPT_CONTRACT_MB_SELFTEST="$(extract_kv_text "contract_mb_selftest" "${PREEMPT_METRICS_TXT}")"
PREEMPT_CONTRACT_DETERMINISTIC_EXIT="$(extract_kv_text "contract_deterministic_exit" "${PREEMPT_METRICS_TXT}")"
PREEMPT_CONTRACT_USER_MODE_SOURCE="$(extract_kv_text "contract_user_minimal_mode_source" "${PREEMPT_METRICS_TXT}")"
PREEMPT_CONTRACT_BOOTSTRAP_SOURCE="$(extract_kv_text "contract_bootstrap_policy_source" "${PREEMPT_METRICS_TXT}")"
PREEMPT_CONTRACT_MB_SELFTEST_SOURCE="$(extract_kv_text "contract_mb_selftest_source" "${PREEMPT_METRICS_TXT}")"
PREEMPT_CONTRACT_DETERMINISTIC_EXIT_SOURCE="$(extract_kv_text "contract_deterministic_exit_source" "${PREEMPT_METRICS_TXT}")"
PREEMPT_CONTRACT_BUILD_DEBUG_SCHED="$(extract_kv_text "contract_build_debug_sched" "${PREEMPT_METRICS_TXT}")"
PREEMPT_CONTRACT_BUILD_DEBUG_IRQ="$(extract_kv_text "contract_build_debug_irq" "${PREEMPT_METRICS_TXT}")"
PREEMPT_CONTRACT_BUILD_DEBUG_SCHED_SOURCE="$(extract_kv_text "contract_build_debug_sched_source" "${PREEMPT_METRICS_TXT}")"
PREEMPT_CONTRACT_BUILD_DEBUG_IRQ_SOURCE="$(extract_kv_text "contract_build_debug_irq_source" "${PREEMPT_METRICS_TXT}")"
PREEMPT_CONTRACT_RING3_ENTRY_GUARD="$(extract_kv_text "contract_ring3_entry_guard" "${PREEMPT_METRICS_TXT}")"
PREEMPT_CONTRACT_RING3_ENTRY_GUARD_SOURCE="$(extract_kv_text "contract_ring3_entry_guard_source" "${PREEMPT_METRICS_TXT}")"
PREEMPT_OBSERVED_USER_MODE="$(extract_kv_text "observed_user_minimal_mode" "${PREEMPT_METRICS_TXT}")"
PREEMPT_OBSERVED_BOOTSTRAP="$(extract_kv_text "observed_bootstrap_policy" "${PREEMPT_METRICS_TXT}")"
PREEMPT_OBSERVED_MB_SELFTEST="$(extract_kv_text "observed_mb_selftest" "${PREEMPT_METRICS_TXT}")"
PREEMPT_OBSERVED_DETERMINISTIC_EXIT="$(extract_kv_text "observed_deterministic_exit" "${PREEMPT_METRICS_TXT}")"
PREEMPT_OBSERVED_RING3_ENTRY_GUARD="$(extract_kv_text "observed_ring3_entry_guard" "${PREEMPT_METRICS_TXT}")"
PHASE_BOOT_START_TICKS="$(extract_kv_metric "phase_boot_start_ticks" "${PREEMPT_METRICS_TXT}")"
PHASE_BOOT_START_TICK_VALID="$(extract_kv_metric "phase_boot_start_tick_valid" "${PREEMPT_METRICS_TXT}")"
PHASE_CORE_READY_TICKS="$(extract_kv_metric "phase_core_ready_ticks" "${PREEMPT_METRICS_TXT}")"
PHASE_CORE_READY_TICK_VALID="$(extract_kv_metric "phase_core_ready_tick_valid" "${PREEMPT_METRICS_TXT}")"
PHASE_FIRST_SCHED_ACTIVITY_TICKS="$(extract_kv_metric "phase_first_sched_activity_ticks" "${PREEMPT_METRICS_TXT}")"
PHASE_FIRST_SCHED_ACTIVITY_TICK_VALID="$(extract_kv_metric "phase_first_sched_activity_tick_valid" "${PREEMPT_METRICS_TXT}")"
PHASE_FIRST_USER_ENTRY_TICKS="$(extract_kv_metric "phase_first_user_entry_ticks" "${PREEMPT_METRICS_TXT}")"
PHASE_FIRST_USER_ENTRY_TICK_VALID="$(extract_kv_metric "phase_first_user_entry_tick_valid" "${PREEMPT_METRICS_TXT}")"
PHASE_FIRST_SYSCALL_GATE_ENTRY_TICKS="$(extract_kv_metric "phase_first_syscall_gate_entry_ticks" "${PREEMPT_METRICS_TXT}")"
PHASE_FIRST_SYSCALL_GATE_ENTRY_TICK_VALID="$(extract_kv_metric "phase_first_syscall_gate_entry_tick_valid" "${PREEMPT_METRICS_TXT}")"
PHASE_FIRST_SYSCALL_GATE_RETURN_TICKS="$(extract_kv_metric "phase_first_syscall_gate_return_ticks" "${PREEMPT_METRICS_TXT}")"
PHASE_FIRST_SYSCALL_GATE_RETURN_TICK_VALID="$(extract_kv_metric "phase_first_syscall_gate_return_tick_valid" "${PREEMPT_METRICS_TXT}")"
PHASE_FIRST_SYSCALL_ENTRY_TICKS="$(extract_kv_metric "phase_first_syscall_entry_ticks" "${PREEMPT_METRICS_TXT}")"
PHASE_FIRST_SYSCALL_ENTRY_TICK_VALID="$(extract_kv_metric "phase_first_syscall_entry_tick_valid" "${PREEMPT_METRICS_TXT}")"
PHASE_FIRST_SYSCALL_EXIT_TICKS="$(extract_kv_metric "phase_first_syscall_exit_ticks" "${PREEMPT_METRICS_TXT}")"
PHASE_FIRST_SYSCALL_EXIT_TICK_VALID="$(extract_kv_metric "phase_first_syscall_exit_tick_valid" "${PREEMPT_METRICS_TXT}")"
PHASE_BOOT_START_TO_CORE_READY_TICKS="$(extract_kv_metric "phase_boot_start_to_core_ready_ticks" "${PREEMPT_METRICS_TXT}")"
PHASE_BOOT_START_TO_CORE_READY_AVAILABLE="$(extract_kv_metric "phase_boot_start_to_core_ready_available" "${PREEMPT_METRICS_TXT}")"
PHASE_CORE_READY_TO_FIRST_SCHED_ACTIVITY_TICKS="$(extract_kv_metric "phase_core_ready_to_first_sched_activity_ticks" "${PREEMPT_METRICS_TXT}")"
PHASE_CORE_READY_TO_FIRST_SCHED_ACTIVITY_AVAILABLE="$(extract_kv_metric "phase_core_ready_to_first_sched_activity_available" "${PREEMPT_METRICS_TXT}")"
PHASE_FIRST_SCHED_ACTIVITY_TO_FIRST_USER_ENTRY_TICKS="$(extract_kv_metric "phase_first_sched_activity_to_first_user_entry_ticks" "${PREEMPT_METRICS_TXT}")"
PHASE_FIRST_SCHED_ACTIVITY_TO_FIRST_USER_ENTRY_AVAILABLE="$(extract_kv_metric "phase_first_sched_activity_to_first_user_entry_available" "${PREEMPT_METRICS_TXT}")"
PHASE_FIRST_USER_ENTRY_TO_FIRST_SYSCALL_GATE_ENTRY_TICKS="$(extract_kv_metric "phase_first_user_entry_to_first_syscall_gate_entry_ticks" "${PREEMPT_METRICS_TXT}")"
PHASE_FIRST_USER_ENTRY_TO_FIRST_SYSCALL_GATE_ENTRY_AVAILABLE="$(extract_kv_metric "phase_first_user_entry_to_first_syscall_gate_entry_available" "${PREEMPT_METRICS_TXT}")"
PHASE_FIRST_SYSCALL_GATE_ENTRY_TO_FIRST_SYSCALL_ENTRY_TICKS="$(extract_kv_metric "phase_first_syscall_gate_entry_to_first_syscall_entry_ticks" "${PREEMPT_METRICS_TXT}")"
PHASE_FIRST_SYSCALL_GATE_ENTRY_TO_FIRST_SYSCALL_ENTRY_AVAILABLE="$(extract_kv_metric "phase_first_syscall_gate_entry_to_first_syscall_entry_available" "${PREEMPT_METRICS_TXT}")"
PHASE_FIRST_SYSCALL_GATE_ENTRY_TO_FIRST_SYSCALL_EXIT_TICKS="$(extract_kv_metric "phase_first_syscall_gate_entry_to_first_syscall_exit_ticks" "${PREEMPT_METRICS_TXT}")"
PHASE_FIRST_SYSCALL_GATE_ENTRY_TO_FIRST_SYSCALL_EXIT_AVAILABLE="$(extract_kv_metric "phase_first_syscall_gate_entry_to_first_syscall_exit_available" "${PREEMPT_METRICS_TXT}")"
PHASE_FIRST_SYSCALL_GATE_ENTRY_TO_FIRST_SYSCALL_GATE_RETURN_TICKS="$(extract_kv_metric "phase_first_syscall_gate_entry_to_first_syscall_gate_return_ticks" "${PREEMPT_METRICS_TXT}")"
PHASE_FIRST_SYSCALL_GATE_ENTRY_TO_FIRST_SYSCALL_GATE_RETURN_AVAILABLE="$(extract_kv_metric "phase_first_syscall_gate_entry_to_first_syscall_gate_return_available" "${PREEMPT_METRICS_TXT}")"
PHASE_FIRST_USER_ENTRY_TO_FIRST_SYSCALL_ENTRY_TICKS="$(extract_kv_metric "phase_first_user_entry_to_first_syscall_entry_ticks" "${PREEMPT_METRICS_TXT}")"
PHASE_FIRST_USER_ENTRY_TO_FIRST_SYSCALL_ENTRY_AVAILABLE="$(extract_kv_metric "phase_first_user_entry_to_first_syscall_entry_available" "${PREEMPT_METRICS_TXT}")"
PHASE_FIRST_USER_ENTRY_TO_FIRST_SYSCALL_EXIT_TICKS="$(extract_kv_metric "phase_first_user_entry_to_first_syscall_exit_ticks" "${PREEMPT_METRICS_TXT}")"
PHASE_FIRST_USER_ENTRY_TO_FIRST_SYSCALL_EXIT_AVAILABLE="$(extract_kv_metric "phase_first_user_entry_to_first_syscall_exit_available" "${PREEMPT_METRICS_TXT}")"
MAILBOX_PHASE_SNAPSHOT_ENTER_TICKS="$(extract_kv_metric "mailbox_phase_snapshot_enter_ticks" "${PREEMPT_METRICS_TXT}")"
MAILBOX_PHASE_SNAPSHOT_ENTER_TICK_VALID="$(extract_kv_metric "mailbox_phase_snapshot_enter_tick_valid" "${PREEMPT_METRICS_TXT}")"
MAILBOX_PHASE_SNAPSHOT_EXIT_TICKS="$(extract_kv_metric "mailbox_phase_snapshot_exit_ticks" "${PREEMPT_METRICS_TXT}")"
MAILBOX_PHASE_SNAPSHOT_EXIT_TICK_VALID="$(extract_kv_metric "mailbox_phase_snapshot_exit_tick_valid" "${PREEMPT_METRICS_TXT}")"
MAILBOX_PHASE_EXTRACT_ENTER_TICKS="$(extract_kv_metric "mailbox_phase_extract_enter_ticks" "${PREEMPT_METRICS_TXT}")"
MAILBOX_PHASE_EXTRACT_ENTER_TICK_VALID="$(extract_kv_metric "mailbox_phase_extract_enter_tick_valid" "${PREEMPT_METRICS_TXT}")"
MAILBOX_PHASE_EXTRACT_EXIT_TICKS="$(extract_kv_metric "mailbox_phase_extract_exit_ticks" "${PREEMPT_METRICS_TXT}")"
MAILBOX_PHASE_EXTRACT_EXIT_TICK_VALID="$(extract_kv_metric "mailbox_phase_extract_exit_tick_valid" "${PREEMPT_METRICS_TXT}")"
MAILBOX_PHASE_VALIDATE_ENTER_TICKS="$(extract_kv_metric "mailbox_phase_validate_enter_ticks" "${PREEMPT_METRICS_TXT}")"
MAILBOX_PHASE_VALIDATE_ENTER_TICK_VALID="$(extract_kv_metric "mailbox_phase_validate_enter_tick_valid" "${PREEMPT_METRICS_TXT}")"
MAILBOX_PHASE_VALIDATE_EXIT_TICKS="$(extract_kv_metric "mailbox_phase_validate_exit_ticks" "${PREEMPT_METRICS_TXT}")"
MAILBOX_PHASE_VALIDATE_EXIT_TICK_VALID="$(extract_kv_metric "mailbox_phase_validate_exit_tick_valid" "${PREEMPT_METRICS_TXT}")"
MAILBOX_PHASE_ARBITER_ENTER_TICKS="$(extract_kv_metric "mailbox_phase_arbiter_enter_ticks" "${PREEMPT_METRICS_TXT}")"
MAILBOX_PHASE_ARBITER_ENTER_TICK_VALID="$(extract_kv_metric "mailbox_phase_arbiter_enter_tick_valid" "${PREEMPT_METRICS_TXT}")"
MAILBOX_PHASE_ARBITER_EXIT_TICKS="$(extract_kv_metric "mailbox_phase_arbiter_exit_ticks" "${PREEMPT_METRICS_TXT}")"
MAILBOX_PHASE_ARBITER_EXIT_TICK_VALID="$(extract_kv_metric "mailbox_phase_arbiter_exit_tick_valid" "${PREEMPT_METRICS_TXT}")"
MAILBOX_PHASE_HANDOFF_ENTER_TICKS="$(extract_kv_metric "mailbox_phase_handoff_enter_ticks" "${PREEMPT_METRICS_TXT}")"
MAILBOX_PHASE_HANDOFF_ENTER_TICK_VALID="$(extract_kv_metric "mailbox_phase_handoff_enter_tick_valid" "${PREEMPT_METRICS_TXT}")"
MAILBOX_PHASE_HANDOFF_EXIT_TICKS="$(extract_kv_metric "mailbox_phase_handoff_exit_ticks" "${PREEMPT_METRICS_TXT}")"
MAILBOX_PHASE_HANDOFF_EXIT_TICK_VALID="$(extract_kv_metric "mailbox_phase_handoff_exit_tick_valid" "${PREEMPT_METRICS_TXT}")"
MAILBOX_PHASE_SNAPSHOT_TICKS="$(extract_kv_metric "mailbox_phase_snapshot_ticks" "${PREEMPT_METRICS_TXT}")"
MAILBOX_PHASE_SNAPSHOT_AVAILABLE="$(extract_kv_metric "mailbox_phase_snapshot_available" "${PREEMPT_METRICS_TXT}")"
MAILBOX_PHASE_EXTRACT_TICKS="$(extract_kv_metric "mailbox_phase_extract_ticks" "${PREEMPT_METRICS_TXT}")"
MAILBOX_PHASE_EXTRACT_AVAILABLE="$(extract_kv_metric "mailbox_phase_extract_available" "${PREEMPT_METRICS_TXT}")"
MAILBOX_PHASE_VALIDATE_TICKS="$(extract_kv_metric "mailbox_phase_validate_ticks" "${PREEMPT_METRICS_TXT}")"
MAILBOX_PHASE_VALIDATE_AVAILABLE="$(extract_kv_metric "mailbox_phase_validate_available" "${PREEMPT_METRICS_TXT}")"
MAILBOX_PHASE_ARBITER_TICKS="$(extract_kv_metric "mailbox_phase_arbiter_ticks" "${PREEMPT_METRICS_TXT}")"
MAILBOX_PHASE_ARBITER_AVAILABLE="$(extract_kv_metric "mailbox_phase_arbiter_available" "${PREEMPT_METRICS_TXT}")"
MAILBOX_PHASE_HANDOFF_TICKS="$(extract_kv_metric "mailbox_phase_handoff_ticks" "${PREEMPT_METRICS_TXT}")"
MAILBOX_PHASE_HANDOFF_AVAILABLE="$(extract_kv_metric "mailbox_phase_handoff_available" "${PREEMPT_METRICS_TXT}")"

if [[ "${PREEMPT_QEMU_TIMEOUT_HIT}" -gt 0 ]]; then
  record_violation "preempt_qemu_timeout:exit_rc=${PREEMPT_QEMU_EXIT_RC}:timeout_seconds=${QEMU_TIMEOUT}"
fi
if [[ "${PREEMPT_QEMU_TIMEOUT_HIT}" -eq 0 && "${PREEMPT_PROOF_DONE_SEEN}" -le 0 ]]; then
  record_violation "preempt_proof_done_missing:qemu_exit_rc=${PREEMPT_QEMU_EXIT_RC}"
fi
if [[ "${PREEMPT_QEMU_TIMEOUT_HIT}" -eq 0 && "${PREEMPT_PROOF_DONE_SEEN}" -gt 0 ]]; then
  if ! is_expected_qemu_exit_rc "${PREEMPT_QEMU_EXIT_RC}" "${PREEMPT_EXPECTED_QEMU_EXIT_SET}"; then
    record_violation "preempt_qemu_exit_rc_unexpected:expected_set=${PREEMPT_EXPECTED_QEMU_EXIT_SET}:actual=${PREEMPT_QEMU_EXIT_RC}"
  fi
fi
if [[ "${PREEMPT_CONTRACT_USER_MODE}" != "${PREEMPT_USER_MINIMAL_MODE}" ]]; then
  record_violation "preempt_contract_not_consumed:user_minimal_mode:expected=${PREEMPT_USER_MINIMAL_MODE}:actual=${PREEMPT_CONTRACT_USER_MODE:-missing}"
fi
if [[ "${PREEMPT_CONTRACT_BOOTSTRAP}" != "${PREEMPT_BOOTSTRAP_POLICY}" ]]; then
  record_violation "preempt_contract_not_consumed:bootstrap_policy:expected=${PREEMPT_BOOTSTRAP_POLICY}:actual=${PREEMPT_CONTRACT_BOOTSTRAP:-missing}"
fi
if [[ "${PREEMPT_CONTRACT_MB_SELFTEST}" != "${PREEMPT_MB_SELFTEST}" ]]; then
  record_violation "preempt_contract_not_consumed:mb_selftest:expected=${PREEMPT_MB_SELFTEST}:actual=${PREEMPT_CONTRACT_MB_SELFTEST:-missing}"
fi
if [[ "${PREEMPT_CONTRACT_DETERMINISTIC_EXIT}" != "${PREEMPT_DETERMINISTIC_EXIT}" ]]; then
  record_violation "preempt_contract_not_consumed:deterministic_exit:expected=${PREEMPT_DETERMINISTIC_EXIT}:actual=${PREEMPT_CONTRACT_DETERMINISTIC_EXIT:-missing}"
fi
if [[ "${PREEMPT_CONTRACT_RING3_ENTRY_GUARD}" != "${PREEMPT_RING3_ENTRY_GUARD}" ]]; then
  record_violation "preempt_contract_not_consumed:ring3_entry_guard:expected=${PREEMPT_RING3_ENTRY_GUARD}:actual=${PREEMPT_CONTRACT_RING3_ENTRY_GUARD:-missing}"
fi
if [[ "${PREEMPT_OBSERVED_USER_MODE}" != "${PREEMPT_USER_MINIMAL_MODE}" ]]; then
  record_violation "preempt_observed_mismatch:user_minimal_mode:expected=${PREEMPT_USER_MINIMAL_MODE}:observed=${PREEMPT_OBSERVED_USER_MODE:-missing}"
fi
if [[ "${PREEMPT_OBSERVED_BOOTSTRAP}" != "${PREEMPT_BOOTSTRAP_POLICY}" ]]; then
  record_violation "preempt_observed_mismatch:bootstrap_policy:expected=${PREEMPT_BOOTSTRAP_POLICY}:observed=${PREEMPT_OBSERVED_BOOTSTRAP:-missing}"
fi
if [[ "${PREEMPT_OBSERVED_MB_SELFTEST}" != "${PREEMPT_MB_SELFTEST}" ]]; then
  record_violation "preempt_observed_mismatch:mb_selftest:expected=${PREEMPT_MB_SELFTEST}:observed=${PREEMPT_OBSERVED_MB_SELFTEST:-missing}"
fi
if [[ "${PREEMPT_OBSERVED_DETERMINISTIC_EXIT}" != "${PREEMPT_DETERMINISTIC_EXIT}" ]]; then
  record_violation "preempt_observed_mismatch:deterministic_exit:expected=${PREEMPT_DETERMINISTIC_EXIT}:observed=${PREEMPT_OBSERVED_DETERMINISTIC_EXIT:-missing}"
fi
if [[ "${PREEMPT_OBSERVED_RING3_ENTRY_GUARD}" != "${PREEMPT_RING3_ENTRY_GUARD}" ]]; then
  record_violation "preempt_observed_mismatch:ring3_entry_guard:expected=${PREEMPT_RING3_ENTRY_GUARD}:observed=${PREEMPT_OBSERVED_RING3_ENTRY_GUARD:-missing}"
fi

PREEMPT_TIME_MS="${PREEMPT_TIME_MS_WALL}"
if [[ "${PREEMPT_QEMU_RUN_TIME_MS}" -gt 0 ]]; then
  PREEMPT_TIME_MS="${PREEMPT_QEMU_RUN_TIME_MS}"
fi

if [[ "${PREEMPT_SW_COUNT}" -le 0 ]]; then
  PREEMPT_SW_COUNT="$(extract_label_count "${PREEMPT_SW_COUNT_PATTERN}" "${PREEMPT_LOG}")"
fi
if [[ "${PREEMPT_IRET_COUNT}" -le 0 ]]; then
  PREEMPT_IRET_COUNT="$(extract_label_count "${PREEMPT_IRET_COUNT_PATTERN}" "${PREEMPT_LOG}")"
fi
if [[ "${PREEMPT_SW_COUNT}" -le 0 && "${MARK_SW_COUNT}" -gt 0 ]]; then
  PREEMPT_SW_COUNT="${MARK_SW_COUNT}"
fi
if [[ "${PREEMPT_IRET_COUNT}" -le 0 && "${MARK_IRET_COUNT}" -gt 0 ]]; then
  PREEMPT_IRET_COUNT="${MARK_IRET_COUNT}"
fi
if [[ "${SCHED_FALLBACK}" == "0" ]] && [[ "${STAGE_HINT_MISSING_SIGNAL}" -gt 0 ]]; then
  record_violation "preempt_stage_hint_missing:sel_idle=${SCHED_IDLE_COUNT}"
fi
if [[ "${PREEMPT_SW_COUNT}" -le 0 ]]; then
  record_violation "preempt_marker_missing:sw_count=0"
fi
if [[ "${PREEMPT_IRET_COUNT}" -le 0 ]]; then
  record_violation "preempt_marker_missing:iret_count=0"
fi

CONTEXT_SWITCH_LATENCY_MS_PROXY="$(python3 - <<'PY' "${PREEMPT_TIME_MS}" "${PREEMPT_SW_COUNT}"
import sys
dur = float(sys.argv[1])
cnt = int(sys.argv[2])
if cnt <= 0:
    print("INF")
else:
    print(f"{dur/cnt:.6f}")
PY
)"
if [[ "${CONTEXT_SWITCH_LATENCY_MS_PROXY}" == "INF" ]]; then
  record_violation "context_switch_latency_proxy_invalid:INF"
fi

# 5) Measure syscall proxy from iret marker cadence in strict preempt run.
SYSCALL_LATENCY_MS_PROXY="$(python3 - <<'PY' "${PREEMPT_TIME_MS}" "${PREEMPT_IRET_COUNT}"
import sys
dur = float(sys.argv[1])
cnt = int(sys.argv[2])
if cnt <= 0:
    print("INF")
else:
    print(f"{dur/cnt:.6f}")
PY
)"
if [[ "${SYSCALL_LATENCY_MS_PROXY}" == "INF" ]]; then
  record_violation "syscall_latency_proxy_invalid:INF"
fi

# 6) Persist measured results.
BOOT_TIME_MS_ENV="${BOOT_TIME_MS}" \
PREEMPT_TIME_MS_ENV="${PREEMPT_TIME_MS}" \
PREEMPT_TIME_MS_WALL_ENV="${PREEMPT_TIME_MS_WALL}" \
PREEMPT_QEMU_RUN_TIME_MS_ENV="${PREEMPT_QEMU_RUN_TIME_MS}" \
PREEMPT_SW_COUNT_ENV="${PREEMPT_SW_COUNT}" \
PREEMPT_IRET_COUNT_ENV="${PREEMPT_IRET_COUNT}" \
CONTEXT_SWITCH_LATENCY_MS_PROXY_ENV="${CONTEXT_SWITCH_LATENCY_MS_PROXY}" \
SYSCALL_LATENCY_MS_PROXY_ENV="${SYSCALL_LATENCY_MS_PROXY}" \
PHASE_BOOT_START_TICKS_ENV="${PHASE_BOOT_START_TICKS}" \
PHASE_BOOT_START_TICK_VALID_ENV="${PHASE_BOOT_START_TICK_VALID}" \
PHASE_CORE_READY_TICKS_ENV="${PHASE_CORE_READY_TICKS}" \
PHASE_CORE_READY_TICK_VALID_ENV="${PHASE_CORE_READY_TICK_VALID}" \
PHASE_FIRST_SCHED_ACTIVITY_TICKS_ENV="${PHASE_FIRST_SCHED_ACTIVITY_TICKS}" \
PHASE_FIRST_SCHED_ACTIVITY_TICK_VALID_ENV="${PHASE_FIRST_SCHED_ACTIVITY_TICK_VALID}" \
PHASE_FIRST_USER_ENTRY_TICKS_ENV="${PHASE_FIRST_USER_ENTRY_TICKS}" \
PHASE_FIRST_USER_ENTRY_TICK_VALID_ENV="${PHASE_FIRST_USER_ENTRY_TICK_VALID}" \
PHASE_FIRST_SYSCALL_GATE_ENTRY_TICKS_ENV="${PHASE_FIRST_SYSCALL_GATE_ENTRY_TICKS}" \
PHASE_FIRST_SYSCALL_GATE_ENTRY_TICK_VALID_ENV="${PHASE_FIRST_SYSCALL_GATE_ENTRY_TICK_VALID}" \
PHASE_FIRST_SYSCALL_GATE_RETURN_TICKS_ENV="${PHASE_FIRST_SYSCALL_GATE_RETURN_TICKS}" \
PHASE_FIRST_SYSCALL_GATE_RETURN_TICK_VALID_ENV="${PHASE_FIRST_SYSCALL_GATE_RETURN_TICK_VALID}" \
PHASE_FIRST_SYSCALL_ENTRY_TICKS_ENV="${PHASE_FIRST_SYSCALL_ENTRY_TICKS}" \
PHASE_FIRST_SYSCALL_ENTRY_TICK_VALID_ENV="${PHASE_FIRST_SYSCALL_ENTRY_TICK_VALID}" \
PHASE_FIRST_SYSCALL_EXIT_TICKS_ENV="${PHASE_FIRST_SYSCALL_EXIT_TICKS}" \
PHASE_FIRST_SYSCALL_EXIT_TICK_VALID_ENV="${PHASE_FIRST_SYSCALL_EXIT_TICK_VALID}" \
PHASE_BOOT_START_TO_CORE_READY_TICKS_ENV="${PHASE_BOOT_START_TO_CORE_READY_TICKS}" \
PHASE_BOOT_START_TO_CORE_READY_AVAILABLE_ENV="${PHASE_BOOT_START_TO_CORE_READY_AVAILABLE}" \
PHASE_CORE_READY_TO_FIRST_SCHED_ACTIVITY_TICKS_ENV="${PHASE_CORE_READY_TO_FIRST_SCHED_ACTIVITY_TICKS}" \
PHASE_CORE_READY_TO_FIRST_SCHED_ACTIVITY_AVAILABLE_ENV="${PHASE_CORE_READY_TO_FIRST_SCHED_ACTIVITY_AVAILABLE}" \
PHASE_FIRST_SCHED_ACTIVITY_TO_FIRST_USER_ENTRY_TICKS_ENV="${PHASE_FIRST_SCHED_ACTIVITY_TO_FIRST_USER_ENTRY_TICKS}" \
PHASE_FIRST_SCHED_ACTIVITY_TO_FIRST_USER_ENTRY_AVAILABLE_ENV="${PHASE_FIRST_SCHED_ACTIVITY_TO_FIRST_USER_ENTRY_AVAILABLE}" \
PHASE_FIRST_USER_ENTRY_TO_FIRST_SYSCALL_GATE_ENTRY_TICKS_ENV="${PHASE_FIRST_USER_ENTRY_TO_FIRST_SYSCALL_GATE_ENTRY_TICKS}" \
PHASE_FIRST_USER_ENTRY_TO_FIRST_SYSCALL_GATE_ENTRY_AVAILABLE_ENV="${PHASE_FIRST_USER_ENTRY_TO_FIRST_SYSCALL_GATE_ENTRY_AVAILABLE}" \
PHASE_FIRST_SYSCALL_GATE_ENTRY_TO_FIRST_SYSCALL_ENTRY_TICKS_ENV="${PHASE_FIRST_SYSCALL_GATE_ENTRY_TO_FIRST_SYSCALL_ENTRY_TICKS}" \
PHASE_FIRST_SYSCALL_GATE_ENTRY_TO_FIRST_SYSCALL_ENTRY_AVAILABLE_ENV="${PHASE_FIRST_SYSCALL_GATE_ENTRY_TO_FIRST_SYSCALL_ENTRY_AVAILABLE}" \
PHASE_FIRST_SYSCALL_GATE_ENTRY_TO_FIRST_SYSCALL_EXIT_TICKS_ENV="${PHASE_FIRST_SYSCALL_GATE_ENTRY_TO_FIRST_SYSCALL_EXIT_TICKS}" \
PHASE_FIRST_SYSCALL_GATE_ENTRY_TO_FIRST_SYSCALL_EXIT_AVAILABLE_ENV="${PHASE_FIRST_SYSCALL_GATE_ENTRY_TO_FIRST_SYSCALL_EXIT_AVAILABLE}" \
PHASE_FIRST_SYSCALL_GATE_ENTRY_TO_FIRST_SYSCALL_GATE_RETURN_TICKS_ENV="${PHASE_FIRST_SYSCALL_GATE_ENTRY_TO_FIRST_SYSCALL_GATE_RETURN_TICKS}" \
PHASE_FIRST_SYSCALL_GATE_ENTRY_TO_FIRST_SYSCALL_GATE_RETURN_AVAILABLE_ENV="${PHASE_FIRST_SYSCALL_GATE_ENTRY_TO_FIRST_SYSCALL_GATE_RETURN_AVAILABLE}" \
PHASE_FIRST_USER_ENTRY_TO_FIRST_SYSCALL_ENTRY_TICKS_ENV="${PHASE_FIRST_USER_ENTRY_TO_FIRST_SYSCALL_ENTRY_TICKS}" \
PHASE_FIRST_USER_ENTRY_TO_FIRST_SYSCALL_ENTRY_AVAILABLE_ENV="${PHASE_FIRST_USER_ENTRY_TO_FIRST_SYSCALL_ENTRY_AVAILABLE}" \
PHASE_FIRST_USER_ENTRY_TO_FIRST_SYSCALL_EXIT_TICKS_ENV="${PHASE_FIRST_USER_ENTRY_TO_FIRST_SYSCALL_EXIT_TICKS}" \
PHASE_FIRST_USER_ENTRY_TO_FIRST_SYSCALL_EXIT_AVAILABLE_ENV="${PHASE_FIRST_USER_ENTRY_TO_FIRST_SYSCALL_EXIT_AVAILABLE}" \
MAILBOX_PHASE_SNAPSHOT_ENTER_TICKS_ENV="${MAILBOX_PHASE_SNAPSHOT_ENTER_TICKS}" \
MAILBOX_PHASE_SNAPSHOT_ENTER_TICK_VALID_ENV="${MAILBOX_PHASE_SNAPSHOT_ENTER_TICK_VALID}" \
MAILBOX_PHASE_SNAPSHOT_EXIT_TICKS_ENV="${MAILBOX_PHASE_SNAPSHOT_EXIT_TICKS}" \
MAILBOX_PHASE_SNAPSHOT_EXIT_TICK_VALID_ENV="${MAILBOX_PHASE_SNAPSHOT_EXIT_TICK_VALID}" \
MAILBOX_PHASE_EXTRACT_ENTER_TICKS_ENV="${MAILBOX_PHASE_EXTRACT_ENTER_TICKS}" \
MAILBOX_PHASE_EXTRACT_ENTER_TICK_VALID_ENV="${MAILBOX_PHASE_EXTRACT_ENTER_TICK_VALID}" \
MAILBOX_PHASE_EXTRACT_EXIT_TICKS_ENV="${MAILBOX_PHASE_EXTRACT_EXIT_TICKS}" \
MAILBOX_PHASE_EXTRACT_EXIT_TICK_VALID_ENV="${MAILBOX_PHASE_EXTRACT_EXIT_TICK_VALID}" \
MAILBOX_PHASE_VALIDATE_ENTER_TICKS_ENV="${MAILBOX_PHASE_VALIDATE_ENTER_TICKS}" \
MAILBOX_PHASE_VALIDATE_ENTER_TICK_VALID_ENV="${MAILBOX_PHASE_VALIDATE_ENTER_TICK_VALID}" \
MAILBOX_PHASE_VALIDATE_EXIT_TICKS_ENV="${MAILBOX_PHASE_VALIDATE_EXIT_TICKS}" \
MAILBOX_PHASE_VALIDATE_EXIT_TICK_VALID_ENV="${MAILBOX_PHASE_VALIDATE_EXIT_TICK_VALID}" \
MAILBOX_PHASE_ARBITER_ENTER_TICKS_ENV="${MAILBOX_PHASE_ARBITER_ENTER_TICKS}" \
MAILBOX_PHASE_ARBITER_ENTER_TICK_VALID_ENV="${MAILBOX_PHASE_ARBITER_ENTER_TICK_VALID}" \
MAILBOX_PHASE_ARBITER_EXIT_TICKS_ENV="${MAILBOX_PHASE_ARBITER_EXIT_TICKS}" \
MAILBOX_PHASE_ARBITER_EXIT_TICK_VALID_ENV="${MAILBOX_PHASE_ARBITER_EXIT_TICK_VALID}" \
MAILBOX_PHASE_HANDOFF_ENTER_TICKS_ENV="${MAILBOX_PHASE_HANDOFF_ENTER_TICKS}" \
MAILBOX_PHASE_HANDOFF_ENTER_TICK_VALID_ENV="${MAILBOX_PHASE_HANDOFF_ENTER_TICK_VALID}" \
MAILBOX_PHASE_HANDOFF_EXIT_TICKS_ENV="${MAILBOX_PHASE_HANDOFF_EXIT_TICKS}" \
MAILBOX_PHASE_HANDOFF_EXIT_TICK_VALID_ENV="${MAILBOX_PHASE_HANDOFF_EXIT_TICK_VALID}" \
MAILBOX_PHASE_SNAPSHOT_TICKS_ENV="${MAILBOX_PHASE_SNAPSHOT_TICKS}" \
MAILBOX_PHASE_SNAPSHOT_AVAILABLE_ENV="${MAILBOX_PHASE_SNAPSHOT_AVAILABLE}" \
MAILBOX_PHASE_EXTRACT_TICKS_ENV="${MAILBOX_PHASE_EXTRACT_TICKS}" \
MAILBOX_PHASE_EXTRACT_AVAILABLE_ENV="${MAILBOX_PHASE_EXTRACT_AVAILABLE}" \
MAILBOX_PHASE_VALIDATE_TICKS_ENV="${MAILBOX_PHASE_VALIDATE_TICKS}" \
MAILBOX_PHASE_VALIDATE_AVAILABLE_ENV="${MAILBOX_PHASE_VALIDATE_AVAILABLE}" \
MAILBOX_PHASE_ARBITER_TICKS_ENV="${MAILBOX_PHASE_ARBITER_TICKS}" \
MAILBOX_PHASE_ARBITER_AVAILABLE_ENV="${MAILBOX_PHASE_ARBITER_AVAILABLE}" \
MAILBOX_PHASE_HANDOFF_TICKS_ENV="${MAILBOX_PHASE_HANDOFF_TICKS}" \
MAILBOX_PHASE_HANDOFF_AVAILABLE_ENV="${MAILBOX_PHASE_HANDOFF_AVAILABLE}" \
PREEMPT_METRICS_TXT_ENV="${PREEMPT_METRICS_TXT}" \
RESULTS_JSON_ENV="${RESULTS_JSON}" \
python3 - <<'PY'
import json
import os

def load_kv(path: str) -> dict[str, str]:
    data: dict[str, str] = {}
    with open(path, "r", encoding="utf-8") as fh:
        for line in fh:
            if "=" not in line:
                continue
            key, value = line.rstrip("\n").split("=", 1)
            data[key] = value
    return data

payload = {
    "boot_time_ms": int(os.environ["BOOT_TIME_MS_ENV"]),
    "preempt_run_time_ms": int(os.environ["PREEMPT_TIME_MS_ENV"]),
    "preempt_wall_time_ms": int(os.environ["PREEMPT_TIME_MS_WALL_ENV"]),
    "preempt_qemu_run_time_ms": int(os.environ["PREEMPT_QEMU_RUN_TIME_MS_ENV"]),
    "preempt_sw_count": int(os.environ["PREEMPT_SW_COUNT_ENV"]),
    "preempt_iret_count": int(os.environ["PREEMPT_IRET_COUNT_ENV"]),
    "context_switch_latency_ms_proxy": (
        float(os.environ["CONTEXT_SWITCH_LATENCY_MS_PROXY_ENV"])
        if os.environ["CONTEXT_SWITCH_LATENCY_MS_PROXY_ENV"] != "INF"
        else None
    ),
    "syscall_latency_ms_proxy": (
        float(os.environ["SYSCALL_LATENCY_MS_PROXY_ENV"])
        if os.environ["SYSCALL_LATENCY_MS_PROXY_ENV"] != "INF"
        else None
    ),
    "entry_latency_ticks": {
        "ticks": int(os.environ["PHASE_FIRST_USER_ENTRY_TO_FIRST_SYSCALL_GATE_ENTRY_TICKS_ENV"]),
        "available": bool(int(os.environ["PHASE_FIRST_USER_ENTRY_TO_FIRST_SYSCALL_GATE_ENTRY_AVAILABLE_ENV"])),
    },
    "syscall_latency_ticks_pure": {
        "ticks": int(os.environ["PHASE_FIRST_SYSCALL_GATE_ENTRY_TO_FIRST_SYSCALL_EXIT_TICKS_ENV"]),
        "available": bool(int(os.environ["PHASE_FIRST_SYSCALL_GATE_ENTRY_TO_FIRST_SYSCALL_EXIT_AVAILABLE_ENV"])),
    },
    "syscall_gate_return_latency_ticks": {
        "ticks": int(os.environ["PHASE_FIRST_SYSCALL_GATE_ENTRY_TO_FIRST_SYSCALL_GATE_RETURN_TICKS_ENV"]),
        "available": bool(int(os.environ["PHASE_FIRST_SYSCALL_GATE_ENTRY_TO_FIRST_SYSCALL_GATE_RETURN_AVAILABLE_ENV"])),
    },
    "metric_model": {
        "context_switch_latency_ms_proxy": {
            "role": "authoritative_proxy",
            "units": "ms",
            "source": "preempt_run_time_ms / preempt_sw_count",
            "enforcement": "baseline_threshold",
        },
        "syscall_latency_ms_proxy": {
            "role": "authoritative_proxy",
            "units": "ms",
            "source": "preempt_run_time_ms / preempt_iret_count",
            "enforcement": "baseline_threshold",
            "note": "Includes the guarded first-entry window; keep for continuity while split diagnostics expose the pure syscall phase.",
        },
        "entry_latency_ticks": {
            "role": "diagnostic_split",
            "units": "ticks",
            "source": "first_user_entry -> first_syscall_gate_entry",
            "enforcement": "informational_only",
        },
        "syscall_latency_ticks_pure": {
            "role": "diagnostic_split",
            "units": "ticks",
            "source": "first_syscall_gate_entry -> first_syscall_exit",
            "enforcement": "informational_only",
        },
        "syscall_gate_return_latency_ticks": {
            "role": "diagnostic_split",
            "units": "ticks",
            "source": "first_syscall_gate_entry -> first_syscall_gate_return",
            "enforcement": "informational_only",
        },
    },
    "phase_breakdown_ticks": {
        "raw_markers": {
            "boot_start": {
                "ticks": int(os.environ["PHASE_BOOT_START_TICKS_ENV"]),
                "tick_valid": bool(int(os.environ["PHASE_BOOT_START_TICK_VALID_ENV"])),
            },
            "core_ready": {
                "ticks": int(os.environ["PHASE_CORE_READY_TICKS_ENV"]),
                "tick_valid": bool(int(os.environ["PHASE_CORE_READY_TICK_VALID_ENV"])),
            },
            "first_sched_activity": {
                "ticks": int(os.environ["PHASE_FIRST_SCHED_ACTIVITY_TICKS_ENV"]),
                "tick_valid": bool(int(os.environ["PHASE_FIRST_SCHED_ACTIVITY_TICK_VALID_ENV"])),
            },
            "first_user_entry": {
                "ticks": int(os.environ["PHASE_FIRST_USER_ENTRY_TICKS_ENV"]),
                "tick_valid": bool(int(os.environ["PHASE_FIRST_USER_ENTRY_TICK_VALID_ENV"])),
            },
            "first_syscall_gate_entry": {
                "ticks": int(os.environ["PHASE_FIRST_SYSCALL_GATE_ENTRY_TICKS_ENV"]),
                "tick_valid": bool(int(os.environ["PHASE_FIRST_SYSCALL_GATE_ENTRY_TICK_VALID_ENV"])),
            },
            "first_syscall_gate_return": {
                "ticks": int(os.environ["PHASE_FIRST_SYSCALL_GATE_RETURN_TICKS_ENV"]),
                "tick_valid": bool(int(os.environ["PHASE_FIRST_SYSCALL_GATE_RETURN_TICK_VALID_ENV"])),
            },
            "first_syscall_entry": {
                "ticks": int(os.environ["PHASE_FIRST_SYSCALL_ENTRY_TICKS_ENV"]),
                "tick_valid": bool(int(os.environ["PHASE_FIRST_SYSCALL_ENTRY_TICK_VALID_ENV"])),
            },
            "first_syscall_exit": {
                "ticks": int(os.environ["PHASE_FIRST_SYSCALL_EXIT_TICKS_ENV"]),
                "tick_valid": bool(int(os.environ["PHASE_FIRST_SYSCALL_EXIT_TICK_VALID_ENV"])),
            },
        },
        "durations": {
            "boot_start_to_core_ready": {
                "ticks": int(os.environ["PHASE_BOOT_START_TO_CORE_READY_TICKS_ENV"]),
                "available": bool(int(os.environ["PHASE_BOOT_START_TO_CORE_READY_AVAILABLE_ENV"])),
            },
            "core_ready_to_first_sched_activity": {
                "ticks": int(os.environ["PHASE_CORE_READY_TO_FIRST_SCHED_ACTIVITY_TICKS_ENV"]),
                "available": bool(int(os.environ["PHASE_CORE_READY_TO_FIRST_SCHED_ACTIVITY_AVAILABLE_ENV"])),
            },
            "first_sched_activity_to_first_user_entry": {
                "ticks": int(os.environ["PHASE_FIRST_SCHED_ACTIVITY_TO_FIRST_USER_ENTRY_TICKS_ENV"]),
                "available": bool(int(os.environ["PHASE_FIRST_SCHED_ACTIVITY_TO_FIRST_USER_ENTRY_AVAILABLE_ENV"])),
            },
            "first_user_entry_to_first_syscall_gate_entry": {
                "ticks": int(os.environ["PHASE_FIRST_USER_ENTRY_TO_FIRST_SYSCALL_GATE_ENTRY_TICKS_ENV"]),
                "available": bool(int(os.environ["PHASE_FIRST_USER_ENTRY_TO_FIRST_SYSCALL_GATE_ENTRY_AVAILABLE_ENV"])),
            },
            "first_syscall_gate_entry_to_first_syscall_entry": {
                "ticks": int(os.environ["PHASE_FIRST_SYSCALL_GATE_ENTRY_TO_FIRST_SYSCALL_ENTRY_TICKS_ENV"]),
                "available": bool(int(os.environ["PHASE_FIRST_SYSCALL_GATE_ENTRY_TO_FIRST_SYSCALL_ENTRY_AVAILABLE_ENV"])),
            },
            "first_syscall_gate_entry_to_first_syscall_exit": {
                "ticks": int(os.environ["PHASE_FIRST_SYSCALL_GATE_ENTRY_TO_FIRST_SYSCALL_EXIT_TICKS_ENV"]),
                "available": bool(int(os.environ["PHASE_FIRST_SYSCALL_GATE_ENTRY_TO_FIRST_SYSCALL_EXIT_AVAILABLE_ENV"])),
            },
            "first_syscall_gate_entry_to_first_syscall_gate_return": {
                "ticks": int(os.environ["PHASE_FIRST_SYSCALL_GATE_ENTRY_TO_FIRST_SYSCALL_GATE_RETURN_TICKS_ENV"]),
                "available": bool(int(os.environ["PHASE_FIRST_SYSCALL_GATE_ENTRY_TO_FIRST_SYSCALL_GATE_RETURN_AVAILABLE_ENV"])),
            },
            "first_user_entry_to_first_syscall_entry": {
                "ticks": int(os.environ["PHASE_FIRST_USER_ENTRY_TO_FIRST_SYSCALL_ENTRY_TICKS_ENV"]),
                "available": bool(int(os.environ["PHASE_FIRST_USER_ENTRY_TO_FIRST_SYSCALL_ENTRY_AVAILABLE_ENV"])),
            },
            "first_user_entry_to_first_syscall_exit": {
                "ticks": int(os.environ["PHASE_FIRST_USER_ENTRY_TO_FIRST_SYSCALL_EXIT_TICKS_ENV"]),
                "available": bool(int(os.environ["PHASE_FIRST_USER_ENTRY_TO_FIRST_SYSCALL_EXIT_AVAILABLE_ENV"])),
            },
        },
    },
    "mailbox_phase_breakdown_ticks": {
        "raw_markers": {
            "snapshot_enter": {
                "ticks": int(os.environ["MAILBOX_PHASE_SNAPSHOT_ENTER_TICKS_ENV"]),
                "tick_valid": bool(int(os.environ["MAILBOX_PHASE_SNAPSHOT_ENTER_TICK_VALID_ENV"])),
            },
            "snapshot_exit": {
                "ticks": int(os.environ["MAILBOX_PHASE_SNAPSHOT_EXIT_TICKS_ENV"]),
                "tick_valid": bool(int(os.environ["MAILBOX_PHASE_SNAPSHOT_EXIT_TICK_VALID_ENV"])),
            },
            "extract_enter": {
                "ticks": int(os.environ["MAILBOX_PHASE_EXTRACT_ENTER_TICKS_ENV"]),
                "tick_valid": bool(int(os.environ["MAILBOX_PHASE_EXTRACT_ENTER_TICK_VALID_ENV"])),
            },
            "extract_exit": {
                "ticks": int(os.environ["MAILBOX_PHASE_EXTRACT_EXIT_TICKS_ENV"]),
                "tick_valid": bool(int(os.environ["MAILBOX_PHASE_EXTRACT_EXIT_TICK_VALID_ENV"])),
            },
            "validate_enter": {
                "ticks": int(os.environ["MAILBOX_PHASE_VALIDATE_ENTER_TICKS_ENV"]),
                "tick_valid": bool(int(os.environ["MAILBOX_PHASE_VALIDATE_ENTER_TICK_VALID_ENV"])),
            },
            "validate_exit": {
                "ticks": int(os.environ["MAILBOX_PHASE_VALIDATE_EXIT_TICKS_ENV"]),
                "tick_valid": bool(int(os.environ["MAILBOX_PHASE_VALIDATE_EXIT_TICK_VALID_ENV"])),
            },
            "arbiter_enter": {
                "ticks": int(os.environ["MAILBOX_PHASE_ARBITER_ENTER_TICKS_ENV"]),
                "tick_valid": bool(int(os.environ["MAILBOX_PHASE_ARBITER_ENTER_TICK_VALID_ENV"])),
            },
            "arbiter_exit": {
                "ticks": int(os.environ["MAILBOX_PHASE_ARBITER_EXIT_TICKS_ENV"]),
                "tick_valid": bool(int(os.environ["MAILBOX_PHASE_ARBITER_EXIT_TICK_VALID_ENV"])),
            },
            "handoff_enter": {
                "ticks": int(os.environ["MAILBOX_PHASE_HANDOFF_ENTER_TICKS_ENV"]),
                "tick_valid": bool(int(os.environ["MAILBOX_PHASE_HANDOFF_ENTER_TICK_VALID_ENV"])),
            },
            "handoff_exit": {
                "ticks": int(os.environ["MAILBOX_PHASE_HANDOFF_EXIT_TICKS_ENV"]),
                "tick_valid": bool(int(os.environ["MAILBOX_PHASE_HANDOFF_EXIT_TICK_VALID_ENV"])),
            },
        },
        "durations": {
            "snapshot": {
                "ticks": int(os.environ["MAILBOX_PHASE_SNAPSHOT_TICKS_ENV"]),
                "available": bool(int(os.environ["MAILBOX_PHASE_SNAPSHOT_AVAILABLE_ENV"])),
            },
            "extract": {
                "ticks": int(os.environ["MAILBOX_PHASE_EXTRACT_TICKS_ENV"]),
                "available": bool(int(os.environ["MAILBOX_PHASE_EXTRACT_AVAILABLE_ENV"])),
            },
            "validate": {
                "ticks": int(os.environ["MAILBOX_PHASE_VALIDATE_TICKS_ENV"]),
                "available": bool(int(os.environ["MAILBOX_PHASE_VALIDATE_AVAILABLE_ENV"])),
            },
            "arbiter": {
                "ticks": int(os.environ["MAILBOX_PHASE_ARBITER_TICKS_ENV"]),
                "available": bool(int(os.environ["MAILBOX_PHASE_ARBITER_AVAILABLE_ENV"])),
            },
            "handoff": {
                "ticks": int(os.environ["MAILBOX_PHASE_HANDOFF_TICKS_ENV"]),
                "available": bool(int(os.environ["MAILBOX_PHASE_HANDOFF_AVAILABLE_ENV"])),
            },
        },
    },
}

metrics_kv = load_kv(os.environ["PREEMPT_METRICS_TXT_ENV"])
mailbox_extra_markers = (
    "arbiter_owner_lookup_enter",
    "arbiter_owner_lookup_exit",
    "arbiter_candidate_lookup_enter",
    "arbiter_candidate_lookup_exit",
    "arbiter_decision_enter",
    "arbiter_decision_exit",
    "arbiter_decision_path_switch",
    "arbiter_decision_path_keep_running",
    "arbiter_decision_path_reject",
    "arbiter_decision_path_fallback",
    "arbiter_candidate_accept_keep_running",
    "arbiter_candidate_accept_switch",
    "arbiter_candidate_reject",
    "arbiter_keep_running_fallback",
    "arbiter_return_null",
    "arbiter_ready_head_fallback",
)
mailbox_extra_durations = (
    ("arbiter_owner_lookup", "arbiter_owner_lookup_enter", "arbiter_owner_lookup_exit"),
    ("arbiter_candidate_lookup", "arbiter_candidate_lookup_enter", "arbiter_candidate_lookup_exit"),
    ("arbiter_decision", "arbiter_decision_enter", "arbiter_decision_exit"),
)
mailbox_event_markers = (
    "arbiter_decision_path_switch",
    "arbiter_decision_path_keep_running",
    "arbiter_decision_path_reject",
    "arbiter_decision_path_fallback",
    "arbiter_candidate_accept_keep_running",
    "arbiter_candidate_accept_switch",
    "arbiter_candidate_reject",
    "arbiter_keep_running_fallback",
    "arbiter_return_null",
    "arbiter_ready_head_fallback",
)
mailbox_extract_reason_names = (
    "snapshot_fail",
    "bad_magic",
    "bad_version",
    "bad_kind",
    "epoch_stale",
    "pid_zero",
    "ok",
)
mailbox_candidate_visibility_names = (
    "visible",
    "proc_missing",
    "proc_not_schedulable",
)
mailbox_consume_reason_names = (
    "timer_validate_accept_consume",
    "timer_validate_accept_deferred",
    "scheduler_keep_running_consume",
    "scheduler_switch_consume",
    "gate4_epoch1_pending_bypass",
    "gate45_self_keep_running_bypass",
)
mailbox_consume_site_names = (
    "timer_validate_irq",
    "START",
    "YIELD",
    "BLOCK",
    "IRQ",
)

for name in mailbox_extra_markers:
    payload["mailbox_phase_breakdown_ticks"]["raw_markers"][name] = {
        "ticks": int(metrics_kv.get(f"mailbox_phase_{name}_ticks", "0")),
        "tick_valid": bool(int(metrics_kv.get(f"mailbox_phase_{name}_tick_valid", "0"))),
    }

for label, start, end in mailbox_extra_durations:
    payload["mailbox_phase_breakdown_ticks"]["durations"][label] = {
        "ticks": int(metrics_kv.get(f"mailbox_phase_{label}_ticks", "0")),
        "available": bool(int(metrics_kv.get(f"mailbox_phase_{label}_available", "0"))),
    }

payload["mailbox_phase_breakdown_ticks"]["events"] = {}
for name in mailbox_event_markers:
    tick_valid = int(metrics_kv.get(f"mailbox_phase_{name}_tick_valid", "0"))
    payload["mailbox_phase_breakdown_ticks"]["events"][name] = {
        "ticks": int(metrics_kv.get(f"mailbox_phase_{name}_ticks", "0")),
        "available": tick_valid in (1, 2),
    }

payload["mailbox_phase_breakdown_ticks"]["path_durations"] = {}
for name in ("switch", "keep_running", "reject", "fallback"):
    payload["mailbox_phase_breakdown_ticks"]["path_durations"][name] = {
        "enter_count": int(metrics_kv.get(f"mailbox_path_{name}_enter_count", "0")),
        "exit_count": int(metrics_kv.get(f"mailbox_path_{name}_exit_count", "0")),
        "count": int(metrics_kv.get(f"mailbox_path_{name}_count", "0")),
        "total_ticks": int(metrics_kv.get(f"mailbox_path_{name}_total_ticks", "0")),
        "mean_ticks": int(metrics_kv.get(f"mailbox_path_{name}_mean_ticks", "0")),
        "min_ticks": int(metrics_kv.get(f"mailbox_path_{name}_min_ticks", "0")),
        "max_ticks": int(metrics_kv.get(f"mailbox_path_{name}_max_ticks", "0")),
        "available": bool(int(metrics_kv.get(f"mailbox_path_{name}_available", "0"))),
    }

payload["mailbox_phase_breakdown_ticks"]["fallback_reasons"] = {}
for name in (
    "gate45_non_owner",
    "owner_missing",
    "owner_not_ready",
    "owner_mismatch",
    "candidate_proc_missing",
    "candidate_proc_not_schedulable",
    "no_candidate",
    "invalid_state",
    "bootstrap_keep_running",
    "pre_user_bypass",
    "yield_fatal",
    "ready_head_fallback",
    "fallback_forbidden",
    "block_fatal",
    "bootstrap_fatal",
    "yield_null",
):
    payload["mailbox_phase_breakdown_ticks"]["fallback_reasons"][name] = int(
        metrics_kv.get(f"mailbox_reason_{name}_count", "0")
    )

payload["mailbox_phase_breakdown_ticks"]["extract_diagnostics"] = {
    "extract_reasons": {},
    "raw_observations": {
        "count": int(metrics_kv.get("mailbox_extract_raw_observation_count", "0")),
        "latest_epoch": int(metrics_kv.get("mailbox_extract_raw_latest_epoch", "0")),
        "latest_candidate_pid": int(metrics_kv.get("mailbox_extract_raw_latest_candidate_pid", "0")),
        "latest_owner_last_epoch": int(metrics_kv.get("mailbox_extract_raw_latest_owner_last_epoch", "0")),
        "epoch_zero_count": int(metrics_kv.get("mailbox_extract_raw_epoch_zero_count", "0")),
        "epoch_lte_owner_last_epoch_count": int(
            metrics_kv.get("mailbox_extract_raw_epoch_lte_owner_last_epoch_count", "0")
        ),
        "epoch_gt_owner_last_epoch_count": int(
            metrics_kv.get("mailbox_extract_raw_epoch_gt_owner_last_epoch_count", "0")
        ),
        "candidate_pid_zero_count": int(
            metrics_kv.get("mailbox_extract_raw_candidate_pid_zero_count", "0")
        ),
        "candidate_pid_nonzero_count": int(
            metrics_kv.get("mailbox_extract_raw_candidate_pid_nonzero_count", "0")
        ),
    },
    "candidate_visibility": {},
}
for name in mailbox_extract_reason_names:
    payload["mailbox_phase_breakdown_ticks"]["extract_diagnostics"]["extract_reasons"][name] = int(
        metrics_kv.get(f"mailbox_extract_reason_{name}_count", "0")
    )
for name in mailbox_candidate_visibility_names:
    payload["mailbox_phase_breakdown_ticks"]["extract_diagnostics"]["candidate_visibility"][name] = int(
        metrics_kv.get(f"mailbox_candidate_visibility_{name}_count", "0")
    )

payload["mailbox_phase_breakdown_ticks"]["consume_trace"] = {
    "count": int(metrics_kv.get("mailbox_consume_observation_count", "0")),
    "latest": {
        "site": metrics_kv.get("mailbox_consume_latest_site", ""),
        "old_last_epoch": int(metrics_kv.get("mailbox_consume_latest_old_last_epoch", "0")),
        "new_last_epoch": int(metrics_kv.get("mailbox_consume_latest_new_last_epoch", "0")),
        "candidate_epoch": int(metrics_kv.get("mailbox_consume_latest_candidate_epoch", "0")),
        "reason": metrics_kv.get("mailbox_consume_latest_reason", ""),
        "ticks": int(metrics_kv.get("mailbox_consume_latest_ticks", "0")),
        "tick_valid": int(metrics_kv.get("mailbox_consume_latest_tick_valid", "0")),
    },
    "reason_counts": {},
    "site_counts": {},
}
for name in mailbox_consume_reason_names:
    payload["mailbox_phase_breakdown_ticks"]["consume_trace"]["reason_counts"][name] = int(
        metrics_kv.get(f"mailbox_consume_reason_{name}_count", "0")
    )
for name in mailbox_consume_site_names:
    payload["mailbox_phase_breakdown_ticks"]["consume_trace"]["site_counts"][name] = int(
        metrics_kv.get(f"mailbox_consume_site_{name}_count", "0")
    )

with open(os.environ["RESULTS_JSON_ENV"], "w", encoding="utf-8") as fh:
    json.dump(payload, fh, indent=2, sort_keys=True)
    fh.write("\n")
PY

# 7) Build current lock payload.
NOW="$(ci_now_utc)"
GIT_SHA="$(git -C "${ROOT}" rev-parse HEAD 2>/dev/null || echo NO_GIT)"

NOW_ENV="${NOW}" \
GIT_SHA_ENV="${GIT_SHA}" \
ENV_MISMATCH_POLICY_ENV="${ENV_MISMATCH_POLICY}" \
BASELINE_AUTHORITY_ENV="${BASELINE_AUTHORITY}" \
BOOT_THRESHOLD_PERCENT_ENV="${BOOT_THRESHOLD_PERCENT}" \
CONTEXT_THRESHOLD_PERCENT_ENV="${CONTEXT_THRESHOLD_PERCENT}" \
SYSCALL_THRESHOLD_PERCENT_ENV="${SYSCALL_THRESHOLD_PERCENT}" \
ENV_JSON_ENV="${ENV_JSON}" \
RESULTS_JSON_ENV="${RESULTS_JSON}" \
ACTUAL_LOCK_JSON_ENV="${ACTUAL_LOCK_JSON}" \
python3 - <<'PY'
import json
import os

env = json.load(open(os.environ["ENV_JSON_ENV"], encoding="utf-8"))
results = json.load(open(os.environ["RESULTS_JSON_ENV"], encoding="utf-8"))
payload = {
    "schema_version": 1,
    "created_at_utc": os.environ["NOW_ENV"],
    "git_sha": os.environ["GIT_SHA_ENV"],
    "policy": {
        "env_mismatch_policy": os.environ["ENV_MISMATCH_POLICY_ENV"],
        "baseline_authority": os.environ["BASELINE_AUTHORITY_ENV"],
        "marker_contract": env.get("marker_contract", {}),
        "thresholds_percent": {
            "syscall_latency_ms_proxy": float(os.environ["SYSCALL_THRESHOLD_PERCENT_ENV"]),
            "context_switch_latency_ms_proxy": float(os.environ["CONTEXT_THRESHOLD_PERCENT_ENV"]),
            "boot_time_ms": float(os.environ["BOOT_THRESHOLD_PERCENT_ENV"]),
        },
    },
    "env": env,
    "metrics": {
        "boot_time_ms": results["boot_time_ms"],
        "context_switch_latency_ms_proxy": results["context_switch_latency_ms_proxy"],
        "syscall_latency_ms_proxy": results["syscall_latency_ms_proxy"],
    },
    "raw_metrics": results,
}
with open(os.environ["ACTUAL_LOCK_JSON_ENV"], "w", encoding="utf-8") as fh:
    json.dump(payload, fh, indent=2, sort_keys=True)
    fh.write("\n")
PY

# 8) Baseline policy and compare.
DRIFT_ALLOWLIST_BYPASS_COUNT=0

if [[ "${INIT_BASELINE}" -eq 1 ]]; then
  IS_CI="0"
  if [[ "${CI:-}" == "1" || "${CI:-}" == "true" || "${CI:-}" == "TRUE" ]]; then
    IS_CI="1"
  fi
  if [[ "${REQUIRE_CI_FOR_BASELINE_INIT}" -eq 1 && "${IS_CI}" -ne 1 ]]; then
    record_violation "baseline_init_requires_ci:CI env is not true/1"
  elif [[ "${IS_CI}" -eq 1 ]] && ! is_pinned_ci_digest "${CI_IMAGE_DIGEST}" "${BASELINE_AUTHORITY}"; then
    record_violation "baseline_init_requires_ci_image_digest:PERF_CI_IMAGE_DIGEST must be pinned (authority=${BASELINE_AUTHORITY}, digest=${CI_IMAGE_DIGEST})"
  else
    mkdir -p "$(dirname "${BASELINE_FILE}")"
    cp -f "${ACTUAL_LOCK_JSON}" "${BASELINE_FILE}"
    record_violation "baseline_initialized_requires_commit:${BASELINE_FILE}"
  fi
elif [[ -f "${BASELINE_FILE}" ]]; then
  BASELINE_REL=""
  if [[ "${BASELINE_FILE}" == "${ROOT}/"* ]]; then
    BASELINE_REL="${BASELINE_FILE#${ROOT}/}"
  fi
  if [[ -n "${BASELINE_REL}" ]]; then
    if [[ "${ALLOW_UNTRACKED_BASELINE}" != "1" ]] && ! git -C "${ROOT}" ls-files --error-unmatch -- "${BASELINE_REL}" >/dev/null 2>&1; then
      record_violation "baseline_not_tracked:${BASELINE_REL}"
    fi
    if ! git -C "${ROOT}" diff --exit-code -- "${BASELINE_REL}" >/dev/null 2>&1; then
      record_violation "baseline_dirty_worktree:${BASELINE_REL}"
    fi
    if ! git -C "${ROOT}" diff --cached --exit-code -- "${BASELINE_REL}" >/dev/null 2>&1; then
      record_violation "baseline_dirty_index:${BASELINE_REL}"
    fi
  fi

  if BASELINE_FILE_ENV="${BASELINE_FILE}" ACTUAL_LOCK_JSON_ENV="${ACTUAL_LOCK_JSON}" ENV_MISMATCH_POLICY_ENV="${ENV_MISMATCH_POLICY}" python3 - <<'PY' > "${BASELINE_DIFF_TXT}" 2>/dev/null
import json
import os

baseline = json.load(open(os.environ["BASELINE_FILE_ENV"], encoding="utf-8"))
actual = json.load(open(os.environ["ACTUAL_LOCK_JSON_ENV"], encoding="utf-8"))
policy = os.environ["ENV_MISMATCH_POLICY_ENV"]

diffs = []

b_schema = baseline.get("schema_version")
a_schema = actual.get("schema_version")
if b_schema != a_schema:
    diffs.append(f"schema_version_mismatch: baseline={b_schema} actual={a_schema}")

b_env_hash = baseline.get("env", {}).get("env_hash")
a_env_hash = actual.get("env", {}).get("env_hash")
if b_env_hash != a_env_hash:
    if policy == "fail":
        diffs.append(f"env_hash_mismatch: baseline={b_env_hash} actual={a_env_hash}")
    else:
        diffs.append(f"env_hash_mismatch_waiver_required: baseline={b_env_hash} actual={a_env_hash}")

thr = baseline.get("policy", {}).get("thresholds_percent", {})
bm = baseline.get("metrics", {})
am = actual.get("metrics", {})

b_marker_contract = baseline.get("policy", {}).get("marker_contract")
a_marker_contract = actual.get("policy", {}).get("marker_contract")
if b_marker_contract != a_marker_contract:
    diffs.append("marker_contract_mismatch")
b_measurement_contract = (b_marker_contract or {}).get("measurement_contract")
a_measurement_contract = (a_marker_contract or {}).get("measurement_contract")
if b_measurement_contract != a_measurement_contract:
    diffs.append(
        f"measurement_contract_mismatch: baseline={b_measurement_contract} actual={a_measurement_contract}"
    )

b_authority = baseline.get("policy", {}).get("baseline_authority")
a_authority = actual.get("policy", {}).get("baseline_authority")
if b_authority != a_authority:
    diffs.append(f"baseline_authority_mismatch: baseline={b_authority} actual={a_authority}")

b_ci_image = baseline.get("env", {}).get("ci_image_digest")
a_ci_image = actual.get("env", {}).get("ci_image_digest")
digest_is_blocking = not (
    isinstance(b_authority, str)
    and isinstance(a_authority, str)
    and b_authority.startswith("github-hosted-")
    and a_authority.startswith("github-hosted-")
)
if b_ci_image != a_ci_image and digest_is_blocking:
    diffs.append(f"ci_image_digest_mismatch: baseline={b_ci_image} actual={a_ci_image}")

def check_metric(key):
    base_val = bm.get(key)
    actual_val = am.get(key)
    pct = float(thr.get(key, 0))
    if base_val is None or actual_val is None:
        diffs.append(f"metric_missing:{key}:baseline={base_val}:actual={actual_val}")
        return
    if float(base_val) <= 0:
        diffs.append(f"metric_invalid_baseline:{key}:{base_val}")
        return
    max_allowed = float(base_val) * (1.0 + (pct / 100.0))
    if float(actual_val) > max_allowed:
        diffs.append(
            f"metric_regression:{key}:baseline={base_val}:actual={actual_val}:threshold_percent={pct}:max_allowed={max_allowed}"
        )

check_metric("syscall_latency_ms_proxy")
check_metric("context_switch_latency_ms_proxy")
check_metric("boot_time_ms")

if diffs:
    for row in diffs:
        print(row)
    raise SystemExit(1)
PY
  then
    :
  else
    BLOCKING_DIFF_COUNT=0

    # Process baseline diffs and bypass allowlisted metric regressions.
    while IFS= read -r line; do
      [[ -z "${line}" ]] && continue

      # Metric regressions may be bypassed via drift allowlist.
      if [[ "${line}" =~ ^metric_regression:([^:]+): ]]; then
        metric="${BASH_REMATCH[1]}"
        if is_metric_allowlisted "${metric}" "${DRIFT_ALLOWLIST_FILE}"; then
          echo "allowlist_bypass:${metric}:${line}" >> "${ALLOWLIST_BYPASS_TXT}"
          DRIFT_ALLOWLIST_BYPASS_COUNT=$((DRIFT_ALLOWLIST_BYPASS_COUNT + 1))
          continue
        fi

        counter_after="$(increment_counter "${metric}")"
        echo "drift_counter_increment:${metric}:counter=${counter_after}" >> "${EVIDENCE_DIR}/drift_counters.txt"
      fi
      if [[ "${line}" == "marker_contract_mismatch" ]]; then
        record_violation "contract_violation:marker_contract_mismatch"
      fi
      if [[ "${line}" =~ ^measurement_contract_mismatch: ]]; then
        record_violation "contract_violation:measurement_contract_mismatch"
      fi

      record_violation "baseline_diff:${line}"
      BLOCKING_DIFF_COUNT=$((BLOCKING_DIFF_COUNT + 1))
    done < "${BASELINE_DIFF_TXT}"

    if [[ "${BLOCKING_DIFF_COUNT}" -gt 0 ]]; then
      record_violation "baseline_mismatch:${BASELINE_FILE}"
    fi
  fi
else
  # Baseline missing
  if [[ "${BASELINE_MODE}" == "provisional" ]]; then
    # Provisional mode: baseline missing is acceptable, skip gate
    echo "WARN: Baseline missing in provisional mode, skipping enforcement" >&2
  else
    # Constitutional mode: baseline missing is a violation
    record_violation "baseline_missing:${BASELINE_FILE}"
  fi
fi

VIOLATIONS_COUNT="$(wc -l < "${VIOLATIONS_TXT}" | tr -d ' ' || echo 0)"

# Compute drift authority hash for evidence
DRIFT_AUTHORITY_HASH="$(compute_authority_hash)"

{
  echo "time_utc=${NOW}"
  echo "git_sha=${GIT_SHA}"
  echo "baseline_file=${BASELINE_FILE}"
  echo "baseline_mode=${BASELINE_MODE}"
  echo "regression_policy=${REGRESSION_POLICY}"
  echo "init_baseline=${INIT_BASELINE}"
  echo "kernel_profile=${KERNEL_PROFILE}"
  echo "qemu_timeout=${QEMU_TIMEOUT}"
  echo "env_mismatch_policy=${ENV_MISMATCH_POLICY}"
  echo "baseline_authority=${BASELINE_AUTHORITY}"
  echo "ci_image_digest=${CI_IMAGE_DIGEST}"
  echo "require_ci_for_baseline_init=${REQUIRE_CI_FOR_BASELINE_INIT}"
  echo "allow_untracked_baseline=${ALLOW_UNTRACKED_BASELINE}"
  echo "boot_threshold_percent=${BOOT_THRESHOLD_PERCENT}"
  echo "context_threshold_percent=${CONTEXT_THRESHOLD_PERCENT}"
  echo "syscall_threshold_percent=${SYSCALL_THRESHOLD_PERCENT}"
  echo "boot_ok_marker=${BOOT_OK_MARKER}"
  echo "preempt_sw_count_pattern=${PREEMPT_SW_COUNT_PATTERN}"
  echo "preempt_iret_count_pattern=${PREEMPT_IRET_COUNT_PATTERN}"
  echo "preempt_force_efi_rebuild=${PREEMPT_FORCE_EFI_REBUILD}"
  echo "preempt_user_minimal_mode=${PREEMPT_USER_MINIMAL_MODE}"
  echo "preempt_bootstrap_policy=${PREEMPT_BOOTSTRAP_POLICY}"
  echo "preempt_mb_selftest=${PREEMPT_MB_SELFTEST}"
  echo "preempt_deterministic_exit=${PREEMPT_DETERMINISTIC_EXIT}"
  echo "preempt_build_debug_sched=${PREEMPT_BUILD_DEBUG_SCHED:-<make-default>}"
  echo "preempt_build_debug_irq=${PREEMPT_BUILD_DEBUG_IRQ:-<make-default>}"
  echo "preempt_expected_qemu_exit_set=${PREEMPT_EXPECTED_QEMU_EXIT_SET}"
  echo "measurement_contract=${MEASUREMENT_CONTRACT}"
  echo "ayken_sched_fallback=${SCHED_FALLBACK}"
  echo "preempt_sched_idle_count=${SCHED_IDLE_COUNT}"
  echo "preempt_stage_hint_missing=${STAGE_HINT_MISSING_SIGNAL}"
  echo "preempt_qemu_exit_rc=${PREEMPT_QEMU_EXIT_RC}"
  echo "preempt_qemu_timeout_hit=${PREEMPT_QEMU_TIMEOUT_HIT}"
  echo "preempt_proof_done_seen=${PREEMPT_PROOF_DONE_SEEN}"
  echo "preempt_contract_user_minimal_mode=${PREEMPT_CONTRACT_USER_MODE:-missing}"
  echo "preempt_contract_bootstrap_policy=${PREEMPT_CONTRACT_BOOTSTRAP:-missing}"
  echo "preempt_contract_mb_selftest=${PREEMPT_CONTRACT_MB_SELFTEST:-missing}"
  echo "preempt_contract_deterministic_exit=${PREEMPT_CONTRACT_DETERMINISTIC_EXIT:-missing}"
  echo "preempt_contract_build_debug_sched=${PREEMPT_CONTRACT_BUILD_DEBUG_SCHED:-missing}"
  echo "preempt_contract_build_debug_irq=${PREEMPT_CONTRACT_BUILD_DEBUG_IRQ:-missing}"
  echo "preempt_contract_ring3_entry_guard=${PREEMPT_CONTRACT_RING3_ENTRY_GUARD:-missing}"
  echo "preempt_contract_user_minimal_mode_source=${PREEMPT_CONTRACT_USER_MODE_SOURCE:-missing}"
  echo "preempt_contract_bootstrap_policy_source=${PREEMPT_CONTRACT_BOOTSTRAP_SOURCE:-missing}"
  echo "preempt_contract_mb_selftest_source=${PREEMPT_CONTRACT_MB_SELFTEST_SOURCE:-missing}"
  echo "preempt_contract_deterministic_exit_source=${PREEMPT_CONTRACT_DETERMINISTIC_EXIT_SOURCE:-missing}"
  echo "preempt_contract_build_debug_sched_source=${PREEMPT_CONTRACT_BUILD_DEBUG_SCHED_SOURCE:-missing}"
  echo "preempt_contract_build_debug_irq_source=${PREEMPT_CONTRACT_BUILD_DEBUG_IRQ_SOURCE:-missing}"
  echo "preempt_contract_ring3_entry_guard_source=${PREEMPT_CONTRACT_RING3_ENTRY_GUARD_SOURCE:-missing}"
  echo "preempt_observed_user_minimal_mode=${PREEMPT_OBSERVED_USER_MODE:-missing}"
  echo "preempt_observed_bootstrap_policy=${PREEMPT_OBSERVED_BOOTSTRAP:-missing}"
  echo "preempt_observed_mb_selftest=${PREEMPT_OBSERVED_MB_SELFTEST:-missing}"
  echo "preempt_observed_deterministic_exit=${PREEMPT_OBSERVED_DETERMINISTIC_EXIT:-missing}"
  echo "preempt_observed_ring3_entry_guard=${PREEMPT_OBSERVED_RING3_ENTRY_GUARD:-missing}"
  echo "env_hash=${ENV_HASH}"
  echo "drift_authority_hash=${DRIFT_AUTHORITY_HASH}"
  echo "drift_allowlist_file=${DRIFT_ALLOWLIST_FILE}"
  echo "drift_allowlist_version=${DRIFT_ALLOWLIST_VERSION}"
  echo "drift_allowlist_bypass_count=${DRIFT_ALLOWLIST_BYPASS_COUNT}"
  echo "boot_time_ms=${BOOT_TIME_MS}"
  echo "preempt_wall_time_ms=${PREEMPT_TIME_MS_WALL}"
  echo "preempt_qemu_run_time_ms=${PREEMPT_QEMU_RUN_TIME_MS}"
  echo "context_switch_latency_ms_proxy=${CONTEXT_SWITCH_LATENCY_MS_PROXY}"
  echo "syscall_latency_ms_proxy=${SYSCALL_LATENCY_MS_PROXY}"
  echo "entry_latency_ticks=${PHASE_FIRST_USER_ENTRY_TO_FIRST_SYSCALL_GATE_ENTRY_TICKS}"
  echo "entry_latency_ticks_available=${PHASE_FIRST_USER_ENTRY_TO_FIRST_SYSCALL_GATE_ENTRY_AVAILABLE}"
  echo "syscall_latency_ticks_pure=${PHASE_FIRST_SYSCALL_GATE_ENTRY_TO_FIRST_SYSCALL_EXIT_TICKS}"
  echo "syscall_latency_ticks_pure_available=${PHASE_FIRST_SYSCALL_GATE_ENTRY_TO_FIRST_SYSCALL_EXIT_AVAILABLE}"
  echo "syscall_gate_return_latency_ticks=${PHASE_FIRST_SYSCALL_GATE_ENTRY_TO_FIRST_SYSCALL_GATE_RETURN_TICKS}"
  echo "syscall_gate_return_latency_ticks_available=${PHASE_FIRST_SYSCALL_GATE_ENTRY_TO_FIRST_SYSCALL_GATE_RETURN_AVAILABLE}"
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

def read_json(name):
    p = os.path.join(base, name)
    if not os.path.exists(p):
        return {}
    with open(p, "r", encoding="utf-8", errors="replace") as fh:
        return json.load(fh)

out = {
    "gate": "performance",
    "verdict": "PASS" if violations_count == 0 else "FAIL",
    "violations_count": violations_count,
    "measurement_contract": meta.get("measurement_contract", "unknown"),
    "measurement_contract_note": "Deterministic preempt harness scenario is enforced (not constitutional default runtime).",
    "metric_model_note": "Legacy ms proxy metrics remain the baseline-enforced surface; split tick metrics expose entry-window and pure syscall timing after the ring3 entry guard change.",
    "meta": meta,
    "env": read_json("env.json"),
    "results": read_json("results.json"),
    "baseline_diff": read_lines("baseline.diff.txt"),
    "violations": read_lines("violations.txt"),
    "drift_counters": read_lines("drift_counters.txt"),
    "allowlist_bypass": read_lines("allowlist_bypass.txt"),
}

# Provisional mode override
baseline_mode = meta.get("baseline_mode", "constitutional")
if baseline_mode == "provisional" and violations_count > 0:
    out["verdict"] = "WARN"
    out["provisional_note"] = "Violations present but not enforced in provisional mode"

print(json.dumps(out, indent=2, sort_keys=True))
PY

# Provisional mode: warn instead of fail
if [[ "${VIOLATIONS_COUNT}" -gt 0 ]]; then
  if [[ "${BASELINE_MODE}" == "provisional" ]]; then
    echo "performance: WARN (${VIOLATIONS_COUNT} violations, provisional mode)"
    echo "See: ${VIOLATIONS_TXT}"
    exit 0  # Warn, not fail
  else
    echo "performance: FAIL (${VIOLATIONS_COUNT} violations)"
    echo "See: ${VIOLATIONS_TXT}"
    exit 2
  fi
fi

echo "performance: PASS"
exit 0
