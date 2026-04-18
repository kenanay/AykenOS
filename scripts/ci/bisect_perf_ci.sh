#!/usr/bin/env bash
set -euo pipefail

# CI-style deterministic performance bisect
# Usage: git bisect run scripts/ci/bisect_perf_ci.sh

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "${ROOT}"

# --- CONFIG (CI ile birebir tut) ---
THRESHOLD_BOOT_MS=11752          # 10% üst sınır (10684 baseline)
RUNS=1                           # determinism için tekrar sayısı (bisect için 1 yeterli)
TIMEOUT=15
MODE="syscall-v2-runtime"

echo "=== BISECT PERF CI ==="
echo "commit: $(git rev-parse --short HEAD)"
echo "threshold: ${THRESHOLD_BOOT_MS}ms"
echo "runs: ${RUNS}"

# --- TEMİZ ORTAM ---
echo "[1/4] Cleaning..."
git clean -fdx out >/dev/null 2>&1 || true
make clean >/dev/null 2>&1 || true

# --- BUILD ---
echo "[2/4] Building..."
if ! make -j KERNEL_PROFILE=validation USER_MINIMAL_MODE="${MODE}" efi-img >/dev/null 2>&1; then
  echo "[SKIP] build failed"
  exit 125
fi

# --- MEASURE ---
echo "[3/4] Measuring (${RUNS} runs)..."
METRICS_FILE="${ROOT}/out/bisect_metrics.txt"
measure_once() {
  rm -f "${METRICS_FILE}"
  PREEMPT_METRICS_OUT="${METRICS_FILE}" \
  USER_MINIMAL_MODE="${MODE}" \
  QEMU_TIMEOUT="${TIMEOUT}" \
  ./run_preempt_test.sh >/dev/null 2>&1
  
  if [[ ! -f "${METRICS_FILE}" ]]; then
    echo "0"
    return
  fi
  
  grep 'qemu_run_time_ms=' "${METRICS_FILE}" | cut -d'=' -f2
}

SUM=0
COUNT=0

for i in $(seq 1 ${RUNS}); do
  echo "  run $i/${RUNS}..."
  val=$(measure_once || echo "0")
  if [[ "$val" -eq 0 ]]; then
    echo "[SKIP] measurement failed"
    exit 125
  fi
  echo "    boot_time_ms=$val"
  SUM=$((SUM + val))
  COUNT=$((COUNT + 1))
done

AVG=$((SUM / COUNT))

# --- DECISION ---
echo "[4/4] Decision..."
echo "=== RESULT ==="
echo "boot_time_avg_ms=${AVG}"
echo "threshold=${THRESHOLD_BOOT_MS}"

if [[ "${AVG}" -le "${THRESHOLD_BOOT_MS}" ]]; then
  echo "VERDICT: GOOD (${AVG} <= ${THRESHOLD_BOOT_MS})"
  exit 0
else
  echo "VERDICT: BAD (${AVG} > ${THRESHOLD_BOOT_MS})"
  exit 1
fi
