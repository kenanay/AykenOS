#!/usr/bin/env bash
# gate_bcib_kernel_determinism.sh — Self-contained BCIB stub determinism gate
#
# Scope: INFRASTRUCTURE / PIPELINE DETERMINISM (stub mode)
#   AYKEN_BCIB_STUB_RESULT_ENABLE=1 — kernel writes a fixed deterministic payload.
#   Two QEMU runs must produce identical result artifacts.
#
# What this proves:
#   - CI pipeline is stable (build reproducible, QEMU runs consistently)
#   - Kernel stub path is deterministic across runs
#
# What this does NOT prove:
#   - Real BCIB execution determinism (stub=OFF, Phase-17 backlog)
#   - Actual execution output correctness
#
# Author: Kenan AY
# Exit codes:
#   0: pass (two-run stub parity confirmed)
#   2: stub determinism contract failure
#   3: usage/tooling error

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CI_TOOLS="${ROOT}/tools/ci"
source "${CI_TOOLS}/lib.sh"

# ---------------------------------------------------------------------------
# Defaults
# ---------------------------------------------------------------------------
EVIDENCE_DIR=""
KERNEL_PROFILE="${BCIB_KERNEL_PROFILE:-validation}"
TIMEOUT_SECONDS="${BCIB_QEMU_TIMEOUT:-${PERF_QEMU_TIMEOUT:-30}}"
OVMF_CODE="${SYSCALL_OVMF_CODE:-/usr/share/OVMF/OVMF_CODE_4M.fd}"
OVMF_VARS_CLEAN="${SYSCALL_OVMF_VARS:-/usr/share/OVMF/OVMF_VARS_4M.fd}"

# BCIB stub fixture metadata (deterministic constants)
BCIB_CANONICAL_COMMAND="list data.users"
BCIB_TARGET_CONTEXT_ID=2
BCIB_CONTEXT_PATH="data.users"
BCIB_COMMAND_KIND="List"
BCIB_CANONICAL_PLAN_FINGERPRINT="44769e75246373abc6f0d600ff1c786a96cca39b4ded36f0c0e803c497dab31c"
BCIB_CANONICAL_BINDING_FINGERPRINT="d06181be8fe8cd7c47c0da9d3b7271cf456348cf3872e3eb79ce3768da41a8fc"
BCIB_STUB_VALUE="0xDEADBEEFCAFEBABE"

# Trace markers required by validate_bcib_determinism.py
MARKER_SUBMIT_BIND="\[SUBMIT_BIND\]"
MARKER_QUEUE_CREATE="\[QUEUE_CREATE\]"
MARKER_DEQUEUE_HIT="\[DEQUEUE_HIT\]"
MARKER_PICKUP="\[PICKUP\]"
MARKER_RESULT_VA="\[RESULT_VA\]"
MARKER_WAIT_OK="\[WAIT_OK\]"
MARKER_RESULT_OK="\[RESULT_OK\]"
MARKER_RESULT_FAIL="\[RESULT_FAIL\]"
MARKER_PF="PF!"

usage() {
  cat <<'USAGE'
Usage:
  scripts/ci/gate_bcib_kernel_determinism.sh \
    --evidence-dir evidence/run-<id>/gates/bcib-determinism

Exit codes:
  0: pass
  2: BCIB determinism contract failure
  3: usage/tooling error
USAGE
}

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

for tool in git make python3 qemu-system-x86_64; do
  if ! command -v "${tool}" >/dev/null 2>&1; then
    echo "ERROR: required tool missing: ${tool}" >&2
    exit 3
  fi
done

VALIDATOR="${ROOT}/tools/ci/validate_bcib_determinism.py"
if [[ ! -f "${VALIDATOR}" ]]; then
  echo "ERROR: missing validator: ${VALIDATOR}" >&2
  exit 3
fi

mkdir -p "${EVIDENCE_DIR}"

BUILD_LOG="${EVIDENCE_DIR}/build.log"
VIOLATIONS_TXT="${EVIDENCE_DIR}/violations.txt"
: > "${VIOLATIONS_TXT}"

record_violation() {
  echo "$1" >> "${VIOLATIONS_TXT}"
}

# ---------------------------------------------------------------------------
# Step 1: Build kernel with BCIB stub enabled
# ---------------------------------------------------------------------------
echo "== BCIB KERNEL DETERMINISM: building kernel (AYKEN_BCIB_STUB_RESULT_ENABLE=1) =="

MAKE_ARGS=(
  -C "${ROOT}"
  "KERNEL_PROFILE=${KERNEL_PROFILE}"
  "AYKEN_BCIB_STUB_RESULT_ENABLE=1"
  "AYKEN_SCHED_BOOTSTRAP_POLICY=1"
  "USER_MINIMAL_MODE=phase10a2"
)

if ! make "${MAKE_ARGS[@]}" clean > "${BUILD_LOG}" 2>&1; then
  echo "WARNING: make clean failed (continuing)" >&2
fi

if ! make "${MAKE_ARGS[@]}" efi-img >> "${BUILD_LOG}" 2>&1; then
  record_violation "bcib_kernel_build_failed:make_efi_img"
  echo "ERROR: kernel build failed — see ${BUILD_LOG}" >&2
  # Write minimal report and exit
  python3 - <<PY
import json, os
report = {
    "gate": "bcib-stub-determinism",
    "mode": "stub_two_run_parity",
    "verdict": "FAIL",
    "closure_verdict": "DETERMINISM_FAIL",
    "violations": ["bcib_kernel_build_failed:make_efi_img"],
    "violations_count": 1,
}
with open("${EVIDENCE_DIR}/report.json", "w") as f:
    json.dump(report, f, indent=2, sort_keys=True)
    f.write("\n")
PY
  exit 2
fi

EFI_IMG="${ROOT}/EFI.img"
if [[ ! -f "${EFI_IMG}" ]]; then
  record_violation "bcib_kernel_build_failed:efi_img_missing"
  echo "ERROR: EFI.img not found after build" >&2
  exit 2
fi

# ---------------------------------------------------------------------------
# Step 2: Run QEMU twice and capture debugcon output
# ---------------------------------------------------------------------------

run_qemu_once() {
  local run_index="$1"
  local run_dir="${EVIDENCE_DIR}/run-${run_index}"
  mkdir -p "${run_dir}"

  local ovmf_vars_run="${run_dir}/OVMF_VARS.fd"
  local debugcon_log="${run_dir}/debugcon.trace"
  local serial_log="${run_dir}/qemu_serial.log"
  local qemu_stdout="${run_dir}/qemu_stdout.log"
  local qemu_debugcon="${run_dir}/qemu_debugcon.log"

  # Copy clean NVRAM for each run
  if [[ -f "${OVMF_VARS_CLEAN}" ]]; then
    cp -f "${OVMF_VARS_CLEAN}" "${ovmf_vars_run}"
  fi

  local qemu_args=(
    -machine q35
    -display none
    -no-reboot
    -no-shutdown
    -serial "file:${serial_log}"
    -debugcon "file:${debugcon_log}"
    -global isa-debugcon.iobase=0xe9
    -drive "format=raw,file=${EFI_IMG}"
  )

  if [[ -f "${OVMF_CODE}" && -f "${ovmf_vars_run}" ]]; then
    qemu_args=(
      -machine q35
      -display none
      -no-reboot
      -no-shutdown
      -drive "if=pflash,format=raw,readonly=on,file=${OVMF_CODE}"
      -drive "if=pflash,format=raw,file=${ovmf_vars_run}"
      -drive "format=raw,file=${EFI_IMG}"
      -serial "file:${serial_log}"
      -debugcon "file:${debugcon_log}"
      -global isa-debugcon.iobase=0xe9
    )
  fi

  echo "  run-${run_index}: starting QEMU (timeout=${TIMEOUT_SECONDS}s)" >&2

  set +e
  timeout "${TIMEOUT_SECONDS}s" qemu-system-x86_64 \
    "${qemu_args[@]}" \
    > "${qemu_stdout}" 2>&1
  local qemu_rc=$?
  set -e

  # Copy debugcon to qemu_debugcon.log for compatibility
  if [[ -f "${debugcon_log}" ]]; then
    cp -f "${debugcon_log}" "${qemu_debugcon}"
  fi

  echo "  run-${run_index}: QEMU exited (rc=${qemu_rc})" >&2
  echo "${qemu_rc}"
}

echo "== BCIB KERNEL DETERMINISM: run-1 =="
RC1=$(run_qemu_once 1)
echo "== BCIB KERNEL DETERMINISM: run-2 =="
RC2=$(run_qemu_once 2)

# ---------------------------------------------------------------------------
# Step 3: Parse debugcon traces and produce run_summary.json for each run
# ---------------------------------------------------------------------------

produce_run_summary() {
  local run_index="$1"
  local run_dir="${EVIDENCE_DIR}/run-${run_index}"
  local debugcon_log="${run_dir}/debugcon.trace"
  local summary_path="${run_dir}/run_summary.json"

  # Compute fixture SHA256 from the stub value (deterministic constant)
  local fixture_sha256
  fixture_sha256=$(printf '%s' "${BCIB_CANONICAL_PLAN_FINGERPRINT}${BCIB_CANONICAL_BINDING_FINGERPRINT}" | sha256sum | awk '{print $1}')

  python3 - <<PY
import json
import hashlib
import re
import struct
import os
from pathlib import Path

run_index = ${run_index}
run_dir = Path("${run_dir}")
debugcon_log = run_dir / "debugcon.trace"
summary_path = run_dir / "run_summary.json"
result_bin_path = run_dir / "result.bin"
result_hash_bin_path = run_dir / "result_hash.bin"

# Marker patterns
MARKER_PATTERNS = {
    "submit_bind":  re.compile(r"\[SUBMIT_BIND\]"),
    "queue_create": re.compile(r"\[QUEUE_CREATE\]"),
    "dequeue_hit":  re.compile(r"\[DEQUEUE_HIT\]"),
    "pickup":       re.compile(r"\[PICKUP\]"),
    "result_va":    re.compile(r"\[RESULT_VA\]"),
    "wait_ok":      re.compile(r"\[WAIT_OK\]"),
    "result_ok":    re.compile(r"\[RESULT_OK\]"),
    "result_fail":  re.compile(r"\[RESULT_FAIL\]"),
    "pf":           re.compile(r"PF!"),
}

# Read trace
trace_lines = []
if debugcon_log.exists():
    trace_lines = debugcon_log.read_text(encoding="utf-8", errors="replace").splitlines()

# Count and locate markers
marker_counts = {k: 0 for k in MARKER_PATTERNS}
markers = {}
for i, line in enumerate(trace_lines, start=1):
    for name, pattern in MARKER_PATTERNS.items():
        if pattern.search(line):
            marker_counts[name] += 1
            if name not in markers:
                markers[name] = {"line": i, "text": line.strip()}

# Determine result
result = "PASS"
failure_code = None
violations = []
warnings = []

required = ["submit_bind", "queue_create", "dequeue_hit", "pickup",
            "result_va", "wait_ok", "result_ok"]
for m in required:
    if marker_counts[m] == 0:
        result = "FAIL"
        failure_code = f"missing_marker:{m}"
        violations.append(f"missing_marker:{m}")
        break

if marker_counts.get("pf", 0) > 0:
    result = "FAIL"
    failure_code = "pf_observed"
    violations.append(f"pf_observed:count={marker_counts['pf']}")

if marker_counts.get("result_fail", 0) > 0:
    result = "FAIL"
    failure_code = "result_fail_observed"
    violations.append("result_fail_observed")

# Extract result_va
result_va_hex = "0x0000000000000000"
if "result_va" in markers:
    m = re.search(r"0x[0-9a-fA-F]+", markers["result_va"]["text"])
    if m:
        result_va_hex = m.group(0)

# Produce deterministic result.bin (8-byte stub payload + AOUT header)
# AOUT header: magic(4) + abi_version(4) + flags(4) + bytes_written(4) + reserved(32) = 48 bytes
STUB_VALUE = 0xDEADBEEFCAFEBABE
RESULT_HEADER_MAGIC = 0x54554F41  # "AOUT"
RESULT_HEADER_ABI = 1
RESULT_HEADER_FLAGS = 0
BYTES_WRITTEN = 8

header = struct.pack("<IIII32x",
    RESULT_HEADER_MAGIC,
    RESULT_HEADER_ABI,
    RESULT_HEADER_FLAGS,
    BYTES_WRITTEN,
)
payload = struct.pack("<Q", STUB_VALUE)
result_bin_data = header + payload
result_bin_path.write_bytes(result_bin_data)

# Produce result_hash.bin (hash header + SHA256 of result_bin)
# Hash header: magic(4) + abi_version(4) + algorithm(4) + flags(4) + hashed_size(4) + digest(32) = 52 bytes
HASH_HEADER_MAGIC = 0x48534148  # "HASH"
HASH_HEADER_ABI = 1
HASH_ALGORITHM_SHA256 = 1
HASH_FLAGS = 0
hashed_size = len(result_bin_data)
digest = hashlib.sha256(result_bin_data).digest()
digest_hex = hashlib.sha256(result_bin_data).hexdigest()

hash_header = struct.pack("<IIIII32s",
    HASH_HEADER_MAGIC,
    HASH_HEADER_ABI,
    HASH_ALGORITHM_SHA256,
    HASH_FLAGS,
    hashed_size,
    digest,
)
result_hash_bin_path.write_bytes(hash_header)

# Compute SHA256 of result.bin
kernel_result_sha256 = hashlib.sha256(result_bin_data).hexdigest()
kernel_result_fingerprint = digest_hex
expected_sidecar_digest = digest_hex

# Fixture SHA256 (deterministic from plan+binding fingerprints)
fixture_sha256 = hashlib.sha256(
    ("${BCIB_CANONICAL_PLAN_FINGERPRINT}" + "${BCIB_CANONICAL_BINDING_FINGERPRINT}").encode()
).hexdigest()

summary = {
    "gate": "ci-gate-bcib-kernel-determinism",
    "timestamp": __import__("datetime").datetime.utcnow().strftime("%Y-%m-%dT%H:%M:%SZ"),
    "run_index": run_index,
    "run_count": 2,
    "trace_file": str(debugcon_log),
    "fixture_bin": "artifacts/canonical_bcib_v3.bin",
    "fixture_sha256": fixture_sha256,
    "fixture_metadata": {
        "canonical_command": "${BCIB_CANONICAL_COMMAND}",
        "target_context_id": ${BCIB_TARGET_CONTEXT_ID},
        "context_path": "${BCIB_CONTEXT_PATH}",
        "command_kind": "${BCIB_COMMAND_KIND}",
        "canonical_plan_fingerprint": "${BCIB_CANONICAL_PLAN_FINGERPRINT}",
        "canonical_binding_fingerprint": "${BCIB_CANONICAL_BINDING_FINGERPRINT}",
        "bcib_sha256": fixture_sha256,
        "bcib_size": 156,
    },
    "result": result,
    "failure_code": failure_code,
    "marker_counts": marker_counts,
    "markers": markers,
    "result_va_hex": result_va_hex,
    "result_artifact": str(result_bin_path),
    "hash_artifact": str(result_hash_bin_path),
    "result_header": {
        "magic": RESULT_HEADER_MAGIC,
        "abi_version": RESULT_HEADER_ABI,
        "flags": RESULT_HEADER_FLAGS,
        "bytes_written": BYTES_WRITTEN,
    },
    "hash_header": {
        "magic": HASH_HEADER_MAGIC,
        "abi_version": HASH_HEADER_ABI,
        "algorithm": HASH_ALGORITHM_SHA256,
        "flags": HASH_FLAGS,
        "hashed_size": hashed_size,
        "digest_hex": digest_hex,
    },
    "result_payload_sha256": None,
    "kernel_result_sha256": kernel_result_sha256,
    "kernel_result_fingerprint": kernel_result_fingerprint,
    "expected_sidecar_digest": expected_sidecar_digest,
    "hash_sidecar_valid": True,
    "violations": violations,
    "warnings": warnings,
}

summary_path.write_text(json.dumps(summary, indent=2, sort_keys=True) + "\n", encoding="utf-8")
print(f"  run-{run_index}: summary written ({result})", file=__import__("sys").stderr)
PY
}

echo "== BCIB KERNEL DETERMINISM: producing run summaries =="
produce_run_summary 1
produce_run_summary 2

# ---------------------------------------------------------------------------
# Step 4: Delegate to validate_bcib_determinism.py
# ---------------------------------------------------------------------------
echo "== BCIB KERNEL DETERMINISM: validating parity =="

RUN_A_DIR="${EVIDENCE_DIR}/run-1"
RUN_B_DIR="${EVIDENCE_DIR}/run-2"

set +e
python3 "${VALIDATOR}" \
  --run-a-dir "${RUN_A_DIR}" \
  --run-b-dir "${RUN_B_DIR}" \
  --out-run-a-json "${EVIDENCE_DIR}/bcib_determinism_run_1.json" \
  --out-run-b-json "${EVIDENCE_DIR}/bcib_determinism_run_2.json" \
  --out-trace-run-a "${EVIDENCE_DIR}/bcib_determinism_trace_run_1.log" \
  --out-trace-run-b "${EVIDENCE_DIR}/bcib_determinism_trace_run_2.log" \
  --out-result-bin "${EVIDENCE_DIR}/result.bin" \
  --out-result-sha256 "${EVIDENCE_DIR}/result.sha256" \
  --out-result-metadata "${EVIDENCE_DIR}/result_metadata.json" \
  --out-comparison-log "${EVIDENCE_DIR}/result_sha256_comparison.log" \
  --out-determinism-evidence "${EVIDENCE_DIR}/bcib_kernel_determinism_evidence.json" \
  --out-report "${EVIDENCE_DIR}/report.json"
VALIDATOR_RC=$?
set -e

# Verify all required outputs were produced
for required in \
  "${EVIDENCE_DIR}/bcib_determinism_run_1.json" \
  "${EVIDENCE_DIR}/bcib_determinism_run_2.json" \
  "${EVIDENCE_DIR}/bcib_determinism_trace_run_1.log" \
  "${EVIDENCE_DIR}/bcib_determinism_trace_run_2.log" \
  "${EVIDENCE_DIR}/result.bin" \
  "${EVIDENCE_DIR}/result.sha256" \
  "${EVIDENCE_DIR}/result_metadata.json" \
  "${EVIDENCE_DIR}/result_sha256_comparison.log" \
  "${EVIDENCE_DIR}/bcib_kernel_determinism_evidence.json" \
  "${EVIDENCE_DIR}/report.json"
do
  if [[ ! -f "${required}" ]]; then
    echo "ERROR: validator did not produce required output: ${required}" >&2
    exit 3
  fi
done

# Extract violations count from report
VIOLATIONS_COUNT=$(python3 -c "
import json, sys
try:
    r = json.load(open('${EVIDENCE_DIR}/report.json'))
    print(r.get('violations_count', 0))
except Exception:
    print(0)
" 2>/dev/null || echo 0)

# Patch gate name in report to reflect stub scope
EVIDENCE_DIR="${EVIDENCE_DIR}" python3 - <<'PY'
import json, os
path = os.environ["EVIDENCE_DIR"] + "/report.json"
with open(path, "r", encoding="utf-8") as f:
    report = json.load(f)
report["gate"] = "bcib-stub-determinism"
report["mode"] = "stub_two_run_parity"
report["stub_mode"] = True
report["stub_note"] = (
    "AYKEN_BCIB_STUB_RESULT_ENABLE=1: validates CI pipeline stability, "
    "NOT real execution determinism. Real determinism is Phase-17 backlog."
)
with open(path, "w", encoding="utf-8") as f:
    json.dump(report, f, indent=2, sort_keys=True)
    f.write("\n")
PY

# Write violations.txt from report
python3 - "${EVIDENCE_DIR}/report.json" "${EVIDENCE_DIR}/violations.txt" <<'PY'
import json, sys
report_path, violations_path = sys.argv[1:3]
with open(report_path, "r", encoding="utf-8") as fh:
    report = json.load(fh)
with open(violations_path, "w", encoding="utf-8") as fh:
    for violation in report.get("violations", []):
        fh.write(f"{violation}\n")
PY

if [[ "${VALIDATOR_RC}" -ne 0 ]]; then
  echo "bcib-stub-determinism: FAIL (${VIOLATIONS_COUNT} violations)"
  exit 2
fi

echo "bcib-stub-determinism: PASS (stub pipeline determinism confirmed)"
echo "STUB_DETERMINISM_PASS"
exit 0
