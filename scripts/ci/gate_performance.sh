#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CI_TOOLS="${ROOT}/tools/ci"
source "${CI_TOOLS}/lib.sh"
source "${ROOT}/scripts/ci/lib-drift-persistence.sh"

usage() {
  cat <<'USAGE'
Usage:
  scripts/ci/gate_performance.sh --evidence-dir evidence/run-<id>/gates/performance
    [--baseline-file scripts/ci/perf-baseline.lock.json]
    [--kernel-profile validation]
    [--qemu-timeout 30]
    [--env-mismatch-policy fail|waiver]
    [--init-baseline]

Env controls:
  PERF_BASELINE_AUTHORITY=<id>                 (default: scripts/ci/perf_authority.env)
  PERF_REQUIRE_CI_FOR_BASELINE_INIT=0|1        (default: 1)
  PERF_CI_IMAGE_DIGEST=<digest-or-build-id>    (default: unknown)
  PERF_PREEMPT_FORCE_EFI_REBUILD=0|1           (default: 1)

Exit codes:
  0: pass
  2: performance baseline mismatch / regression
  3: usage/tooling error
USAGE
}

EVIDENCE_DIR=""
BASELINE_FILE="${ROOT}/scripts/ci/perf-baseline.lock.json"
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
PREEMPT_FORCE_EFI_REBUILD="${PERF_PREEMPT_FORCE_EFI_REBUILD:-1}"
SCHED_FALLBACK="${AYKEN_SCHED_FALLBACK:-0}"
BOOT_OK_MARKER="[K][BOOT_OK] Phase 4.4 minimal boot reached"
PREEMPT_SW_COUNT_PATTERN='[SW|MARK:SW] count:'
PREEMPT_IRET_COUNT_PATTERN='[IRET markers] count:'
INIT_BASELINE=0

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

case "${PREEMPT_FORCE_EFI_REBUILD}" in
  0|1)
    ;;
  *)
    echo "ERROR: PERF_PREEMPT_FORCE_EFI_REBUILD must be 0 or 1" >&2
    exit 3
    ;;
esac

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

for tool in git make python3 qemu-system-x86_64; do
  if ! command -v "${tool}" >/dev/null 2>&1; then
    echo "ERROR: required tool missing (${tool})" >&2
    exit 3
  fi
done

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
ACTUAL_LOCK_JSON="${EVIDENCE_DIR}/actual.lock.json"
BASELINE_DIFF_TXT="${EVIDENCE_DIR}/baseline.diff.txt"
VIOLATIONS_TXT="${EVIDENCE_DIR}/violations.txt"
META_TXT="${EVIDENCE_DIR}/meta.txt"
REPORT_JSON="${EVIDENCE_DIR}/report.json"

: > "${ENV_JSON}"
: > "${RESULTS_JSON}"
: > "${BUILD_LOG}"
: > "${RAW_LOG}"
: > "${BOOT_AUDIT_LOG}"
: > "${PREEMPT_LOG}"
: > "${PREEMPT_METRICS_TXT}"
: > "${ACTUAL_LOCK_JSON}"
: > "${BASELINE_DIFF_TXT}"
: > "${VIOLATIONS_TXT}"

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
    },
}
canonical = json.dumps(payload, sort_keys=True, separators=(",", ":"))
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

# 2) Build boot image once.
if ! make -C "${ROOT}" KERNEL_PROFILE="${KERNEL_PROFILE}" efi-img >> "${BUILD_LOG}" 2>&1; then
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
if ! (cd "${ROOT}" && QEMU_TIMEOUT="${QEMU_TIMEOUT}" STRICT_MARKERS=1 FORCE_EFI_REBUILD="${PREEMPT_FORCE_EFI_REBUILD}" KERNEL_PROFILE="${KERNEL_PROFILE}" PREEMPT_METRICS_OUT="${PREEMPT_METRICS_TXT}" ./run_preempt_test.sh) > "${PREEMPT_LOG}" 2>&1; then
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
RESULTS_JSON_ENV="${RESULTS_JSON}" \
python3 - <<'PY'
import json
import os

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
}
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
            "syscall_latency_ms_proxy": 5,
            "context_switch_latency_ms_proxy": 5,
            "boot_time_ms": 10,
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

b_authority = baseline.get("policy", {}).get("baseline_authority")
a_authority = actual.get("policy", {}).get("baseline_authority")
if b_authority != a_authority:
    diffs.append(f"baseline_authority_mismatch: baseline={b_authority} actual={a_authority}")

b_ci_image = baseline.get("env", {}).get("ci_image_digest")
a_ci_image = actual.get("env", {}).get("ci_image_digest")
if b_ci_image != a_ci_image:
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
    record_violation "baseline_mismatch:${BASELINE_FILE}"
    
    # Advisory side-channel: increment drift counters for regressions
    # This does NOT affect verdict (violations still recorded)
    while IFS= read -r line; do
      [[ -z "${line}" ]] && continue
      record_violation "baseline_diff:${line}"
      
      # Extract metric name from regression line
      if [[ "${line}" =~ ^metric_regression:([^:]+): ]]; then
        metric="${BASH_REMATCH[1]}"
        counter_after="$(increment_counter "${metric}")"
        
        # Log to evidence (advisory only, does not affect verdict)
        echo "drift_counter_increment:${metric}:counter=${counter_after}" >> "${EVIDENCE_DIR}/drift_counters.txt"
      fi
    done < "${BASELINE_DIFF_TXT}"
  fi
else
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
  echo "boot_ok_marker=${BOOT_OK_MARKER}"
  echo "preempt_sw_count_pattern=${PREEMPT_SW_COUNT_PATTERN}"
  echo "preempt_iret_count_pattern=${PREEMPT_IRET_COUNT_PATTERN}"
  echo "preempt_force_efi_rebuild=${PREEMPT_FORCE_EFI_REBUILD}"
  echo "ayken_sched_fallback=${SCHED_FALLBACK}"
  echo "preempt_sched_idle_count=${SCHED_IDLE_COUNT}"
  echo "preempt_stage_hint_missing=${STAGE_HINT_MISSING_SIGNAL}"
  echo "env_hash=${ENV_HASH}"
  echo "drift_authority_hash=${DRIFT_AUTHORITY_HASH}"
  echo "boot_time_ms=${BOOT_TIME_MS}"
  echo "preempt_wall_time_ms=${PREEMPT_TIME_MS_WALL}"
  echo "preempt_qemu_run_time_ms=${PREEMPT_QEMU_RUN_TIME_MS}"
  echo "context_switch_latency_ms_proxy=${CONTEXT_SWITCH_LATENCY_MS_PROXY}"
  echo "syscall_latency_ms_proxy=${SYSCALL_LATENCY_MS_PROXY}"
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
    "meta": meta,
    "env": read_json("env.json"),
    "results": read_json("results.json"),
    "baseline_diff": read_lines("baseline.diff.txt"),
    "violations": read_lines("violations.txt"),
    "drift_counters": read_lines("drift_counters.txt"),
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
