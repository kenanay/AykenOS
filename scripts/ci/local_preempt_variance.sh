#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

RUNS=5
WARMUP=1
QEMU_TIMEOUT=12
KERNEL_PROFILE="validation"
STRICT_MARKERS=1
FORCE_EFI_REBUILD=0
OUT_DIR=""
CV_THRESHOLD_RUNTIME=20.0
CV_THRESHOLD_CONTEXT=15.0
CV_THRESHOLD_SYSCALL=15.0

usage() {
  cat <<'USAGE'
Usage:
  scripts/ci/local_preempt_variance.sh [options]

Options:
  --runs N                    Number of measured runs (default: 5)
  --warmup N                  Number of warmup runs before measurement (default: 1)
  --qemu-timeout SEC          QEMU timeout for each run (default: 12)
  --kernel-profile NAME       Kernel profile for run_preempt_test.sh (default: validation)
  --strict-markers 0|1        Pass STRICT_MARKERS to preempt test (default: 1)
  --force-efi-rebuild 0|1     Pass FORCE_EFI_REBUILD to preempt test (default: 0)
  --cv-runtime PCT            Max allowed CV% for run_time_ms (default: 20.0)
  --cv-context PCT            Max allowed CV% for context latency proxy (default: 15.0)
  --cv-syscall PCT            Max allowed CV% for syscall latency proxy (default: 15.0)
  --out-dir PATH              Output directory (default: evidence/run-local-preempt-variance-<utc>)
  -h, --help                  Show this help
USAGE
}

while [[ $# -gt 0 ]]; do
  case "$1" in
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
    --strict-markers)
      STRICT_MARKERS="$2"
      shift 2
      ;;
    --force-efi-rebuild)
      FORCE_EFI_REBUILD="$2"
      shift 2
      ;;
    --cv-runtime)
      CV_THRESHOLD_RUNTIME="$2"
      shift 2
      ;;
    --cv-context)
      CV_THRESHOLD_CONTEXT="$2"
      shift 2
      ;;
    --cv-syscall)
      CV_THRESHOLD_SYSCALL="$2"
      shift 2
      ;;
    --out-dir)
      OUT_DIR="$2"
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

is_nonneg_int() {
  [[ "$1" =~ ^[0-9]+$ ]]
}

is_binary_flag() {
  [[ "$1" == "0" || "$1" == "1" ]]
}

if ! is_nonneg_int "${RUNS}" || [[ "${RUNS}" -lt 1 ]]; then
  echo "ERROR: --runs must be integer >= 1" >&2
  exit 3
fi
if ! is_nonneg_int "${WARMUP}"; then
  echo "ERROR: --warmup must be integer >= 0" >&2
  exit 3
fi
if ! is_nonneg_int "${QEMU_TIMEOUT}" || [[ "${QEMU_TIMEOUT}" -lt 1 ]]; then
  echo "ERROR: --qemu-timeout must be integer >= 1" >&2
  exit 3
fi
if ! is_binary_flag "${STRICT_MARKERS}"; then
  echo "ERROR: --strict-markers must be 0 or 1" >&2
  exit 3
fi
if ! is_binary_flag "${FORCE_EFI_REBUILD}"; then
  echo "ERROR: --force-efi-rebuild must be 0 or 1" >&2
  exit 3
fi

if [[ -z "${OUT_DIR}" ]]; then
  OUT_DIR="${ROOT}/evidence/run-local-preempt-variance-$(date -u +%Y%m%dT%H%M%SZ)"
fi

RUNS_DIR="${OUT_DIR}/runs"
RUNS_TSV="${OUT_DIR}/runs.tsv"
SUMMARY_JSON="${OUT_DIR}/summary.json"
SUMMARY_TXT="${OUT_DIR}/summary.txt"
VIOLATIONS_TXT="${OUT_DIR}/violations.txt"
META_TXT="${OUT_DIR}/meta.txt"

mkdir -p "${RUNS_DIR}"
: > "${RUNS_TSV}"
: > "${VIOLATIONS_TXT}"

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

div_or_inf() {
  python3 - <<'PY' "$1" "$2"
import sys
num = float(sys.argv[1])
den = float(sys.argv[2])
if den <= 0:
    print("INF")
else:
    print(f"{num/den:.6f}")
PY
}

{
  echo "time_utc=$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  echo "root=${ROOT}"
  echo "runs=${RUNS}"
  echo "warmup=${WARMUP}"
  echo "qemu_timeout=${QEMU_TIMEOUT}"
  echo "kernel_profile=${KERNEL_PROFILE}"
  echo "strict_markers=${STRICT_MARKERS}"
  echo "force_efi_rebuild=${FORCE_EFI_REBUILD}"
  echo "cv_runtime_threshold_pct=${CV_THRESHOLD_RUNTIME}"
  echo "cv_context_threshold_pct=${CV_THRESHOLD_CONTEXT}"
  echo "cv_syscall_threshold_pct=${CV_THRESHOLD_SYSCALL}"
} > "${META_TXT}"

printf "run_idx\tphase\tstatus\trc\trun_time_ms\tqemu_run_time_ms\tsw_count\tiret_count\tmark_sw_count\tmark_iret_count\tassert_fail\tcontext_switch_latency_ms_proxy\tsyscall_latency_ms_proxy\tlog_file\tmetrics_file\n" >> "${RUNS_TSV}"

TOTAL_RUNS=$((RUNS + WARMUP))
SAMPLE_IDX=0

echo "== Local Preempt Variance =="
echo "out_dir: ${OUT_DIR}"
echo "total_runs: ${TOTAL_RUNS} (warmup=${WARMUP}, sample=${RUNS})"

for ((i=1; i<=TOTAL_RUNS; i++)); do
  phase="warmup"
  if (( i > WARMUP )); then
    phase="sample"
    SAMPLE_IDX=$((SAMPLE_IDX + 1))
    run_label="sample-${SAMPLE_IDX}"
  else
    run_label="warmup-${i}"
  fi

  log_file="${RUNS_DIR}/${run_label}.log"
  metrics_file="${RUNS_DIR}/${run_label}.metrics.txt"

  start_ms="$(now_ms)"
  set +e
  (
    cd "${ROOT}"
    QEMU_TIMEOUT="${QEMU_TIMEOUT}" \
    STRICT_MARKERS="${STRICT_MARKERS}" \
    FORCE_EFI_REBUILD="${FORCE_EFI_REBUILD}" \
    KERNEL_PROFILE="${KERNEL_PROFILE}" \
    PREEMPT_METRICS_OUT="${metrics_file}" \
    ./run_preempt_test.sh
  ) > "${log_file}" 2>&1
  rc=$?
  set -e
  end_ms="$(now_ms)"
  run_time_ms=$((end_ms - start_ms))

  status="ok"
  if [[ "${rc}" -ne 0 ]]; then
    status="fail"
  fi

  sw_count="$(extract_kv_metric "sw_count" "${metrics_file}")"
  iret_count="$(extract_kv_metric "iret_count" "${metrics_file}")"
  qemu_run_time_ms="$(extract_kv_metric "qemu_run_time_ms" "${metrics_file}")"
  mark_sw_count="$(extract_kv_metric "mark_sw_count" "${metrics_file}")"
  mark_iret_count="$(extract_kv_metric "mark_iret_count" "${metrics_file}")"
  assert_fail="$(extract_kv_metric "assert_fail" "${metrics_file}")"
  ctx_proxy="$(div_or_inf "${run_time_ms}" "${sw_count}")"
  sys_proxy="$(div_or_inf "${run_time_ms}" "${iret_count}")"

  printf "%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\n" \
    "${i}" "${phase}" "${status}" "${rc}" "${run_time_ms}" "${qemu_run_time_ms}" "${sw_count}" "${iret_count}" \
    "${mark_sw_count}" "${mark_iret_count}" "${assert_fail}" "${ctx_proxy}" "${sys_proxy}" \
    "${log_file}" "${metrics_file}" >> "${RUNS_TSV}"

  echo "[${i}/${TOTAL_RUNS}] phase=${phase} status=${status} rc=${rc} time_ms=${run_time_ms} sw=${sw_count} iret=${iret_count}"
done

RUNS_TSV_ENV="${RUNS_TSV}" \
SUMMARY_JSON_ENV="${SUMMARY_JSON}" \
SUMMARY_TXT_ENV="${SUMMARY_TXT}" \
VIOLATIONS_TXT_ENV="${VIOLATIONS_TXT}" \
RUNS_EXPECTED_ENV="${RUNS}" \
CV_RUNTIME_THRESHOLD_ENV="${CV_THRESHOLD_RUNTIME}" \
CV_CONTEXT_THRESHOLD_ENV="${CV_THRESHOLD_CONTEXT}" \
CV_SYSCALL_THRESHOLD_ENV="${CV_THRESHOLD_SYSCALL}" \
python3 - <<'PY'
import csv
import json
import math
import os
import statistics

runs_tsv = os.environ["RUNS_TSV_ENV"]
summary_json = os.environ["SUMMARY_JSON_ENV"]
summary_txt = os.environ["SUMMARY_TXT_ENV"]
violations_txt = os.environ["VIOLATIONS_TXT_ENV"]
runs_expected = int(os.environ["RUNS_EXPECTED_ENV"])
cv_runtime_threshold = float(os.environ["CV_RUNTIME_THRESHOLD_ENV"])
cv_context_threshold = float(os.environ["CV_CONTEXT_THRESHOLD_ENV"])
cv_syscall_threshold = float(os.environ["CV_SYSCALL_THRESHOLD_ENV"])

rows = []
with open(runs_tsv, "r", encoding="utf-8") as fh:
    reader = csv.DictReader(fh, delimiter="\t")
    for row in reader:
        if row.get("phase") == "sample":
            rows.append(row)

violations = []
if len(rows) != runs_expected:
    violations.append(f"sample_count_mismatch: expected={runs_expected} actual={len(rows)}")

def parse_float(v):
    if v is None:
        return None
    v = v.strip()
    if not v or v == "INF":
        return None
    return float(v)

def parse_int(v):
    if v is None:
        return 0
    v = v.strip()
    return int(v) if v else 0

valid_rows = []
for idx, row in enumerate(rows, start=1):
    status = row["status"]
    rc = parse_int(row["rc"])
    assert_fail = parse_int(row["assert_fail"])
    sw_count = parse_int(row["sw_count"])
    iret_count = parse_int(row["iret_count"])
    ctx_proxy = parse_float(row["context_switch_latency_ms_proxy"])
    sys_proxy = parse_float(row["syscall_latency_ms_proxy"])

    if status != "ok" or rc != 0:
        violations.append(f"sample_run_failed:idx={idx}:rc={rc}:status={status}")
        continue
    if assert_fail != 0:
        violations.append(f"sample_assert_fail:idx={idx}")
        continue
    if sw_count <= 0:
        violations.append(f"sample_sw_count_invalid:idx={idx}:value={sw_count}")
        continue
    if iret_count <= 0:
        violations.append(f"sample_iret_count_invalid:idx={idx}:value={iret_count}")
        continue
    if ctx_proxy is None:
        violations.append(f"sample_context_proxy_invalid:idx={idx}")
        continue
    if sys_proxy is None:
        violations.append(f"sample_syscall_proxy_invalid:idx={idx}")
        continue
    valid_rows.append(row)

def metric_stats(values):
    mean = statistics.mean(values)
    stdev = statistics.stdev(values) if len(values) >= 2 else 0.0
    cv = (stdev / mean * 100.0) if mean > 0 else math.inf
    return {
        "n": len(values),
        "min": min(values),
        "max": max(values),
        "mean": mean,
        "stdev": stdev,
        "cv_percent": cv,
    }

stats = {}
if valid_rows:
    runtime_vals = [float(r["run_time_ms"]) for r in valid_rows]
    qemu_runtime_vals = [float(r["qemu_run_time_ms"]) for r in valid_rows if float(r["qemu_run_time_ms"]) > 0]
    context_vals = [float(r["context_switch_latency_ms_proxy"]) for r in valid_rows]
    syscall_vals = [float(r["syscall_latency_ms_proxy"]) for r in valid_rows]
    sw_vals = [float(r["sw_count"]) for r in valid_rows]
    iret_vals = [float(r["iret_count"]) for r in valid_rows]

    stats["run_time_ms"] = metric_stats(runtime_vals)
    if qemu_runtime_vals:
        stats["qemu_run_time_ms"] = metric_stats(qemu_runtime_vals)
    stats["context_switch_latency_ms_proxy"] = metric_stats(context_vals)
    stats["syscall_latency_ms_proxy"] = metric_stats(syscall_vals)
    stats["sw_count"] = metric_stats(sw_vals)
    stats["iret_count"] = metric_stats(iret_vals)

    if stats["run_time_ms"]["cv_percent"] > cv_runtime_threshold:
        violations.append(
            f"cv_regression:run_time_ms:cv={stats['run_time_ms']['cv_percent']:.4f}:threshold={cv_runtime_threshold:.4f}"
        )
    if stats["context_switch_latency_ms_proxy"]["cv_percent"] > cv_context_threshold:
        violations.append(
            f"cv_regression:context_switch_latency_ms_proxy:cv={stats['context_switch_latency_ms_proxy']['cv_percent']:.4f}:threshold={cv_context_threshold:.4f}"
        )
    if stats["syscall_latency_ms_proxy"]["cv_percent"] > cv_syscall_threshold:
        violations.append(
            f"cv_regression:syscall_latency_ms_proxy:cv={stats['syscall_latency_ms_proxy']['cv_percent']:.4f}:threshold={cv_syscall_threshold:.4f}"
        )
else:
    violations.append("no_valid_sample_rows")

with open(violations_txt, "w", encoding="utf-8") as fh:
    for line in violations:
        fh.write(line + "\n")

summary = {
    "gate": "local_preempt_variance",
    "verdict": "PASS" if not violations else "FAIL",
    "runs_expected": runs_expected,
    "sample_rows": len(rows),
    "valid_rows": len(valid_rows),
    "thresholds": {
        "run_time_cv_percent": cv_runtime_threshold,
        "context_latency_cv_percent": cv_context_threshold,
        "syscall_latency_cv_percent": cv_syscall_threshold,
    },
    "stats": stats,
    "violations": violations,
}

with open(summary_json, "w", encoding="utf-8") as fh:
    json.dump(summary, fh, indent=2, sort_keys=True)
    fh.write("\n")

with open(summary_txt, "w", encoding="utf-8") as fh:
    fh.write(f"verdict={summary['verdict']}\n")
    fh.write(f"runs_expected={runs_expected}\n")
    fh.write(f"sample_rows={len(rows)}\n")
    fh.write(f"valid_rows={len(valid_rows)}\n")
    for key, val in stats.items():
        fh.write(
            f"{key}: n={val['n']} min={val['min']:.6f} max={val['max']:.6f} mean={val['mean']:.6f} stdev={val['stdev']:.6f} cv%={val['cv_percent']:.6f}\n"
        )
    if violations:
        fh.write("violations:\n")
        for line in violations:
            fh.write(f"  - {line}\n")
PY

echo "summary: ${SUMMARY_JSON}"
echo "runs: ${RUNS_TSV}"
echo "violations: ${VIOLATIONS_TXT}"

if [[ -s "${VIOLATIONS_TXT}" ]]; then
  echo "local_preempt_variance: FAIL"
  exit 2
fi

echo "local_preempt_variance: PASS"
exit 0
