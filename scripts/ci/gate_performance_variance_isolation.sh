#!/usr/bin/env bash
set -euo pipefail

# Author: Kenan AY
# Attribution is tooling metadata only and has no runtime or acceptance authority.

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

usage() {
  cat <<'USAGE'
Usage:
  scripts/ci/gate_performance_variance_isolation.sh \
    --evidence-dir evidence/run-<id>/gates/phase17-performance-variance-isolation \
    [--runs 3] [--warmup 1] [--qemu-timeout 20] [--kernel-profile validation]

Behavior:
  - Collects deterministic_preempt_harness measurements in two conditions:
    image-reuse and rebuild-per-run.
  - Uses the same syscall-v2-runtime/deterministic-exit performance contract
    as the Phase-17 PR-4 local readiness surface.
  - Generates diagnostic input only; it cannot accept performance, renew a
    baseline, change a threshold, or close a phase.
USAGE
}

EVIDENCE_DIR=""
RUNS=3
WARMUP=1
QEMU_TIMEOUT=20
KERNEL_PROFILE="validation"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --evidence-dir)
      EVIDENCE_DIR="$2"
      shift 2
      ;;
    --runs)
      RUNS="$2"
      shift 2
      ;;
    --warmup)
      WARMUP="$2"
      shift 2
      ;;
    --qemu-timeout)
      QEMU_TIMEOUT="$2"
      shift 2
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
      echo "ERROR: unknown argument: $1" >&2
      usage
      exit 2
      ;;
  esac
done

if [[ -z "${EVIDENCE_DIR}" ]]; then
  usage
  exit 2
fi
for value in "${RUNS}" "${WARMUP}" "${QEMU_TIMEOUT}"; do
  if [[ ! "${value}" =~ ^[0-9]+$ ]]; then
    echo "ERROR: runs, warmup and qemu-timeout must be non-negative integers" >&2
    exit 2
  fi
done
if [[ "${RUNS}" -lt 3 || "${QEMU_TIMEOUT}" -lt 1 ]]; then
  echo "ERROR: runs must be >= 3 and qemu-timeout must be >= 1" >&2
  exit 2
fi

mkdir -p "${EVIDENCE_DIR}/image-reuse" "${EVIDENCE_DIR}/rebuild-per-run"
{
  echo "authority_status=diagnostic_collection_only"
  echo "runs=${RUNS}"
  echo "warmup=${WARMUP}"
  echo "qemu_timeout=${QEMU_TIMEOUT}"
  echo "kernel_profile=${KERNEL_PROFILE}"
  echo "measurement_contract=deterministic_preempt_harness"
  echo "runtime_mode=syscall-v2-runtime"
  echo "deterministic_exit=1"
  echo "image_reuse_measured_force_efi_rebuild=0"
  echo "rebuild_per_run_measured_force_efi_rebuild=1"
  echo "baseline_mutation=false"
  echo "threshold_mutation=false"
} > "${EVIDENCE_DIR}/collection.meta.txt"

run_sample() {
  local group="$1"
  local sample_label="$2"
  local force_rebuild="$3"
  local sample_dir="${EVIDENCE_DIR}/${group}/${sample_label}"
  mkdir -p "${sample_dir}"
  echo "[${group}] ${sample_label} force_efi_rebuild=${force_rebuild}"
  PERF_BASELINE_MODE="provisional" \
  PERF_BASELINE_AUTHORITY="local-phase17-variance-isolation" \
  PERF_REQUIRE_CI_FOR_BASELINE_INIT="0" \
  PERF_CI_IMAGE_DIGEST="local-variance-isolation" \
  PERF_ALLOW_UNTRACKED_BASELINE="1" \
  PERF_BOOT_THRESHOLD_PERCENT="100" \
  PERF_CONTEXT_THRESHOLD_PERCENT="100" \
  PERF_SYSCALL_THRESHOLD_PERCENT="100" \
  PERF_PREEMPT_FORCE_EFI_REBUILD="${force_rebuild}" \
  PERF_PREEMPT_USER_MINIMAL_MODE="syscall-v2-runtime" \
  PERF_PREEMPT_BOOTSTRAP_POLICY="1" \
  PERF_PREEMPT_MB_SELFTEST="0" \
  PERF_PREEMPT_DETERMINISTIC_EXIT="1" \
  PERF_PREEMPT_RING3_ENTRY_GUARD="1" \
  AYKEN_SCHED_FALLBACK="0" \
  CI="false" \
  "${ROOT}/scripts/ci/gate_performance.sh" \
    --evidence-dir "${sample_dir}" \
    --baseline-file "${sample_dir}/provisional-baseline.lock.json" \
    --kernel-profile "${KERNEL_PROFILE}" \
    --qemu-timeout "${QEMU_TIMEOUT}" \
    --env-mismatch-policy "waiver" \
    >/dev/null
}

# The non-measured warmup starts from a clean controlled build so image-reuse
# samples cannot inherit an unrelated validation image from prior work.
if [[ "${WARMUP}" -gt 0 ]]; then
  run_sample "image-reuse" "warmup-1" "1"
  for ((idx=2; idx<=WARMUP; idx++)); do
    run_sample "image-reuse" "warmup-${idx}" "0"
  done
else
  run_sample "image-reuse" "setup-warmup" "1"
fi
for ((idx=1; idx<=RUNS; idx++)); do
  run_sample "image-reuse" "sample-${idx}" "0"
done

for ((idx=1; idx<=WARMUP; idx++)); do
  run_sample "rebuild-per-run" "warmup-${idx}" "1"
done
for ((idx=1; idx<=RUNS; idx++)); do
  run_sample "rebuild-per-run" "sample-${idx}" "1"
done

echo "performance-variance-isolation-collection: PASS"
