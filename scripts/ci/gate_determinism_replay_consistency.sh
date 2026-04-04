#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

usage() {
  cat <<'USAGE'
Usage:
  scripts/ci/gate_determinism_replay_consistency.sh \
    --evidence-dir evidence/run-<id>/gates/determinism-replay-consistency \
    [--source-gate-dir evidence/run-<id>/gates/proofd-service] \
    [--run-id run-proofd-local-r1]

Exit codes:
  0: pass
  1: runtime / malformed / missing artifact failure
  2: determinism hash mismatch
  3: determinism incident lifecycle violation or usage error
USAGE
}

EVIDENCE_DIR=""
SOURCE_GATE_DIR=""
RUN_ID=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --evidence-dir)
      EVIDENCE_DIR="$2"
      shift 2
      ;;
    --source-gate-dir)
      SOURCE_GATE_DIR="$2"
      shift 2
      ;;
    --run-id)
      RUN_ID="$2"
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
if ! command -v cargo >/dev/null 2>&1; then
  echo "ERROR: missing required tool: cargo" >&2
  exit 3
fi
if ! command -v jq >/dev/null 2>&1; then
  echo "ERROR: missing required tool: jq" >&2
  exit 3
fi

mkdir -p "${EVIDENCE_DIR}"
if [[ -z "${SOURCE_GATE_DIR}" ]]; then
  SOURCE_GATE_DIR="$(cd "$(dirname "${EVIDENCE_DIR}")" && pwd)/proofd-service"
fi

REPORT_JSON="${EVIDENCE_DIR}/report.json"
DETAIL_REPORT_JSON="${EVIDENCE_DIR}/determinism_replay_consistency_report.json"
VIOLATIONS_TXT="${EVIDENCE_DIR}/violations.txt"
META_TXT="${EVIDENCE_DIR}/meta.txt"
REPLAY_STDERR_LOG="${EVIDENCE_DIR}/internal_replay.stderr.log"
REPLAY_OUTPUT_JSON="${EVIDENCE_DIR}/internal_replay_output.json"

SOURCE_VERIFY_REQUEST_PATH=""
SOURCE_MANIFEST_SNAPSHOT_PATH=""
SOURCE_RESPONSE_PATH=""
SOURCE_SERVICE_ROOT=""

write_reports() {
  local status="$1"
  local reason="$2"
  local exit_class="$3"
  local run_id_value="$4"
  local expected_hash="$5"
  local observed_hash="$6"
  local request_fingerprint="$7"
  local match_result="$8"
  local incident_present="$9"
  local contract_version="${10}"
  local manifest_request_fingerprint="${11}"
  local manifest_artifact_hash="${12}"
  local incident_path="${13}"
  local violations_json="${14}"

  local match_bool="false"
  if [[ "${match_result}" == "true" ]]; then
    match_bool="true"
  fi

  jq -n \
    --arg gate "determinism-replay-consistency" \
    --arg mode "phase14_proofd_determinism_replay_consistency" \
    --arg status "${status}" \
    --arg reason "${reason}" \
    --arg run_id "${run_id_value}" \
    --arg source_gate_dir "${SOURCE_GATE_DIR}" \
    --arg expected_hash "${expected_hash}" \
    --arg observed_hash "${observed_hash}" \
    --arg request_fingerprint "${request_fingerprint}" \
    --arg incident_path "${incident_path}" \
    --arg manifest_request_fingerprint "${manifest_request_fingerprint}" \
    --arg manifest_artifact_hash "${manifest_artifact_hash}" \
    --argjson match "${match_bool}" \
    --argjson incident_present "${incident_present}" \
    --argjson contract_version "${contract_version}" \
    --argjson manifest_request_fingerprint_matches_contract "$(
      if [[ -n "${manifest_request_fingerprint}" && "${manifest_request_fingerprint}" != "null" && -n "${request_fingerprint}" ]]; then
        if [[ "${manifest_request_fingerprint}" == "${request_fingerprint}" ]]; then
          printf 'true'
        else
          printf 'false'
        fi
      else
        printf 'null'
      fi
    )" \
    --argjson manifest_artifact_hash_matches_contract "$(
      if [[ -n "${manifest_artifact_hash}" && "${manifest_artifact_hash}" != "null" && -n "${expected_hash}" ]]; then
        if [[ "${manifest_artifact_hash}" == "${expected_hash}" ]]; then
          printf 'true'
        else
          printf 'false'
        fi
      else
        printf 'null'
      fi
    )" \
    --argjson violations "${violations_json}" \
    '{
      gate: $gate,
      mode: $mode,
      status: $status,
      reason: $reason,
      run_id: $run_id,
      source_gate_dir: $source_gate_dir,
      request_fingerprint: $request_fingerprint,
      expected_hash: $expected_hash,
      observed_hash: $observed_hash,
      match: $match,
      incident_present: $incident_present,
      incident_path: (if $incident_path == "" then null else $incident_path end),
      contract_version: $contract_version,
      manifest_authority: "non_authoritative",
      manifest_role: "diagnostic_only",
      manifest_request_fingerprint: (if $manifest_request_fingerprint == "" then null else $manifest_request_fingerprint end),
      manifest_request_fingerprint_matches_contract: $manifest_request_fingerprint_matches_contract,
      manifest_artifact_hash: (if $manifest_artifact_hash == "" then null else $manifest_artifact_hash end),
      manifest_artifact_hash_matches_contract: $manifest_artifact_hash_matches_contract,
      violations: $violations,
      violations_count: ($violations | length)
    }' > "${DETAIL_REPORT_JSON}"

  jq -n \
    --arg gate "determinism-replay-consistency" \
    --arg mode "phase14_proofd_determinism_replay_consistency" \
    --arg verdict "${status}" \
    --arg detail_report_path "$(basename "${DETAIL_REPORT_JSON}")" \
    --arg exit_class "${exit_class}" \
    --argjson violations "${violations_json}" \
    '{
      gate: $gate,
      mode: $mode,
      verdict: $verdict,
      exit_class: $exit_class,
      detail_report_path: $detail_report_path,
      violations: $violations,
      violations_count: ($violations | length)
    }' > "${REPORT_JSON}"

  jq -r '.[]' <<<"${violations_json}" > "${VIOLATIONS_TXT}"
}

runtime_fail() {
  local reason="$1"
  local run_id_value="${2:-${RUN_ID:-unknown}}"
  write_reports \
    "FAIL" \
    "${reason}" \
    "runtime" \
    "${run_id_value}" \
    "" \
    "" \
    "" \
    "false" \
    "false" \
    "null" \
    "" \
    "" \
    "" \
    "[\"${reason}\"]"
  {
    echo "time_utc=$(date -u +%Y-%m-%dT%H:%M:%SZ)"
    echo "source_gate_dir=${SOURCE_GATE_DIR}"
    echo "run_id=${run_id_value}"
    echo "reason=${reason}"
    echo "internal_replay_rc=<not_run>"
    echo "verify_request_path=${SOURCE_VERIFY_REQUEST_PATH}"
  } > "${META_TXT}"
  echo "determinism-replay-consistency: FAIL (${reason})"
  exit 1
}

read_optional_string() {
  local filter="$1"
  local path="$2"
  local reason="$3"
  local value
  if ! value="$(jq -r "${filter}" "${path}" 2>/dev/null)"; then
    runtime_fail "${reason}" "${RUN_ID:-unknown}"
  fi
  if [[ "${value}" == "null" ]]; then
    value=""
  fi
  printf '%s' "${value}"
}

if [[ ! -d "${SOURCE_GATE_DIR}" ]]; then
  runtime_fail "missing_source_gate_dir"
fi
SOURCE_GATE_DIR="$(cd "${SOURCE_GATE_DIR}" && pwd)"
SOURCE_VERIFY_REQUEST_PATH="${SOURCE_GATE_DIR}/proofd_verify_request.json"
SOURCE_MANIFEST_SNAPSHOT_PATH="${SOURCE_GATE_DIR}/proofd_run_manifest.json"
SOURCE_RESPONSE_PATH="${SOURCE_GATE_DIR}/proofd_verify_response.json"
SOURCE_SERVICE_ROOT="${SOURCE_GATE_DIR}/service-root"
if [[ ! -f "${SOURCE_VERIFY_REQUEST_PATH}" ]]; then
  runtime_fail "missing_verify_request_snapshot"
fi
if [[ ! -f "${SOURCE_MANIFEST_SNAPSHOT_PATH}" ]]; then
  runtime_fail "missing_source_manifest_snapshot"
fi

if [[ -z "${RUN_ID}" ]]; then
  RUN_ID="$(read_optional_string '.run_id // empty' "${SOURCE_MANIFEST_SNAPSHOT_PATH}" "invalid_source_manifest_snapshot")"
fi
if [[ -z "${RUN_ID}" ]]; then
  runtime_fail "missing_run_id_in_source_manifest"
fi

RUN_DIR="${SOURCE_SERVICE_ROOT}/${RUN_ID}"
RUN_MANIFEST_PATH="${RUN_DIR}/proofd_run_manifest.json"
if [[ ! -d "${RUN_DIR}" ]]; then
  runtime_fail "missing_run_dir" "${RUN_ID}"
fi
if [[ ! -f "${RUN_MANIFEST_PATH}" ]]; then
  runtime_fail "missing_run_manifest" "${RUN_ID}"
fi

CONTRACT_RELATIVE_PATH="$(read_optional_string '.verification_determinism_contract_path // "verification_determinism_contract.json"' "${RUN_MANIFEST_PATH}" "invalid_run_manifest")"
if [[ -z "${CONTRACT_RELATIVE_PATH}" || "${CONTRACT_RELATIVE_PATH}" == "null" ]]; then
  runtime_fail "missing_determinism_contract_path" "${RUN_ID}"
fi
CONTRACT_PATH="${RUN_DIR}/${CONTRACT_RELATIVE_PATH}"
if [[ ! -f "${CONTRACT_PATH}" ]]; then
  runtime_fail "missing_determinism_contract" "${RUN_ID}"
fi

cp -f "${SOURCE_VERIFY_REQUEST_PATH}" "${EVIDENCE_DIR}/proofd_verify_request.json"
cp -f "${RUN_MANIFEST_PATH}" "${EVIDENCE_DIR}/proofd_run_manifest.json"
cp -f "${CONTRACT_PATH}" "${EVIDENCE_DIR}/verification_determinism_contract.json"
if [[ -f "${SOURCE_RESPONSE_PATH}" ]]; then
  cp -f "${SOURCE_RESPONSE_PATH}" "${EVIDENCE_DIR}/proofd_verify_response.json"
fi

EXPECTED_HASH="$(read_optional_string '.artifact_hash // empty' "${CONTRACT_PATH}" "invalid_determinism_contract")"
CONTRACT_REQUEST_FINGERPRINT="$(read_optional_string '.contract.request_fingerprint // empty' "${CONTRACT_PATH}" "invalid_determinism_contract")"
CONTRACT_VERSION="$(read_optional_string '.contract.contract_version // "null"' "${CONTRACT_PATH}" "invalid_determinism_contract")"
MANIFEST_REQUEST_FINGERPRINT="$(read_optional_string '.request_fingerprint // empty' "${RUN_MANIFEST_PATH}" "invalid_run_manifest")"
MANIFEST_ARTIFACT_HASH="$(read_optional_string '.verification_determinism_artifact_hash // empty' "${RUN_MANIFEST_PATH}" "invalid_run_manifest")"

if [[ -z "${EXPECTED_HASH}" ]]; then
  runtime_fail "invalid_contract_missing_artifact_hash" "${RUN_ID}"
fi
if [[ -z "${CONTRACT_REQUEST_FINGERPRINT}" ]]; then
  runtime_fail "invalid_contract_missing_request_fingerprint" "${RUN_ID}"
fi

set +e
cargo run --quiet --manifest-path "${ROOT}/userspace/Cargo.toml" \
  -p proofd \
  -- \
  --internal-replay \
  --run-dir "${RUN_DIR}" \
  --verify-request-path "${SOURCE_VERIFY_REQUEST_PATH}" \
  > "${REPLAY_OUTPUT_JSON}" 2> "${REPLAY_STDERR_LOG}"
INTERNAL_REPLAY_RC=$?
set -e

{
  echo "time_utc=$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  echo "source_gate_dir=${SOURCE_GATE_DIR}"
  echo "run_id=${RUN_ID}"
  echo "run_dir=${RUN_DIR}"
  echo "verify_request_path=${SOURCE_VERIFY_REQUEST_PATH}"
  echo "internal_replay_rc=${INTERNAL_REPLAY_RC}"
} > "${META_TXT}"

if [[ "${INTERNAL_REPLAY_RC}" -ne 0 ]]; then
  runtime_fail "internal_replay_execution_failed" "${RUN_ID}"
fi

OBSERVED_HASH="$(read_optional_string '.artifact_hash // empty' "${REPLAY_OUTPUT_JSON}" "malformed_internal_replay_output")"
REPLAY_REQUEST_FINGERPRINT="$(read_optional_string '.request_fingerprint // empty' "${REPLAY_OUTPUT_JSON}" "malformed_internal_replay_output")"
MATCH_RESULT="$(read_optional_string '.match_result // empty' "${REPLAY_OUTPUT_JSON}" "malformed_internal_replay_output")"
INCIDENT_PATH_OUTPUT="$(read_optional_string '.incident_path // empty' "${REPLAY_OUTPUT_JSON}" "malformed_internal_replay_output")"

if [[ -z "${OBSERVED_HASH}" || -z "${REPLAY_REQUEST_FINGERPRINT}" || -z "${MATCH_RESULT}" ]]; then
  runtime_fail "malformed_internal_replay_output" "${RUN_ID}"
fi
if [[ "${REPLAY_REQUEST_FINGERPRINT}" != "${CONTRACT_REQUEST_FINGERPRINT}" ]]; then
  write_reports \
    "FAIL" \
    "request_fingerprint_mismatch" \
    "runtime" \
    "${RUN_ID}" \
    "${EXPECTED_HASH}" \
    "${OBSERVED_HASH}" \
    "${CONTRACT_REQUEST_FINGERPRINT}" \
    "false" \
    "false" \
    "${CONTRACT_VERSION}" \
    "${MANIFEST_REQUEST_FINGERPRINT}" \
    "${MANIFEST_ARTIFACT_HASH}" \
    "${INCIDENT_PATH_OUTPUT}" \
    "[\"request_fingerprint_mismatch\"]"
  echo "determinism-replay-consistency: FAIL (request_fingerprint_mismatch)"
  exit 1
fi

REPLAY_REPORT_PATH="${RUN_DIR}/verification_determinism_replay_report.json"
INCIDENT_PATH="${RUN_DIR}/verification_determinism_incident.json"
INCIDENT_PRESENT="false"
if [[ -f "${REPLAY_REPORT_PATH}" ]]; then
  cp -f "${REPLAY_REPORT_PATH}" "${EVIDENCE_DIR}/verification_determinism_replay_report.json"
fi
if [[ -f "${INCIDENT_PATH}" ]]; then
  INCIDENT_PRESENT="true"
  cp -f "${INCIDENT_PATH}" "${EVIDENCE_DIR}/verification_determinism_incident.json"
fi

if [[ "${EXPECTED_HASH}" != "${OBSERVED_HASH}" ]]; then
  if [[ "${MATCH_RESULT}" != "false" ]]; then
    write_reports \
      "FAIL" \
      "hash_mismatch_with_invalid_match_result" \
      "runtime" \
      "${RUN_ID}" \
      "${EXPECTED_HASH}" \
      "${OBSERVED_HASH}" \
      "${CONTRACT_REQUEST_FINGERPRINT}" \
      "${MATCH_RESULT}" \
      "${INCIDENT_PRESENT}" \
      "${CONTRACT_VERSION}" \
      "${MANIFEST_REQUEST_FINGERPRINT}" \
      "${MANIFEST_ARTIFACT_HASH}" \
      "${INCIDENT_PATH_OUTPUT}" \
      "[\"hash_mismatch_with_invalid_match_result\"]"
    echo "determinism-replay-consistency: FAIL (hash_mismatch_with_invalid_match_result)"
    exit 1
  fi
  if [[ "${INCIDENT_PRESENT}" != "true" ]]; then
    write_reports \
      "FAIL" \
      "hash_mismatch_without_incident" \
      "incident_lifecycle" \
      "${RUN_ID}" \
      "${EXPECTED_HASH}" \
      "${OBSERVED_HASH}" \
      "${CONTRACT_REQUEST_FINGERPRINT}" \
      "${MATCH_RESULT}" \
      "${INCIDENT_PRESENT}" \
      "${CONTRACT_VERSION}" \
      "${MANIFEST_REQUEST_FINGERPRINT}" \
      "${MANIFEST_ARTIFACT_HASH}" \
      "${INCIDENT_PATH_OUTPUT}" \
      "[\"hash_mismatch_without_incident\"]"
    echo "determinism-replay-consistency: FAIL (hash_mismatch_without_incident)"
    exit 3
  fi
  write_reports \
    "FAIL" \
    "hash_mismatch" \
    "determinism" \
    "${RUN_ID}" \
    "${EXPECTED_HASH}" \
    "${OBSERVED_HASH}" \
    "${CONTRACT_REQUEST_FINGERPRINT}" \
    "${MATCH_RESULT}" \
    "${INCIDENT_PRESENT}" \
    "${CONTRACT_VERSION}" \
    "${MANIFEST_REQUEST_FINGERPRINT}" \
    "${MANIFEST_ARTIFACT_HASH}" \
    "${INCIDENT_PATH_OUTPUT}" \
    "[\"hash_mismatch\"]"
  echo "determinism-replay-consistency: FAIL (hash_mismatch)"
  exit 2
fi

if [[ "${MATCH_RESULT}" != "true" ]]; then
  write_reports \
    "FAIL" \
    "hash_match_with_invalid_match_result" \
    "runtime" \
    "${RUN_ID}" \
    "${EXPECTED_HASH}" \
    "${OBSERVED_HASH}" \
    "${CONTRACT_REQUEST_FINGERPRINT}" \
    "${MATCH_RESULT}" \
    "${INCIDENT_PRESENT}" \
    "${CONTRACT_VERSION}" \
    "${MANIFEST_REQUEST_FINGERPRINT}" \
    "${MANIFEST_ARTIFACT_HASH}" \
    "${INCIDENT_PATH_OUTPUT}" \
    "[\"hash_match_with_invalid_match_result\"]"
  echo "determinism-replay-consistency: FAIL (hash_match_with_invalid_match_result)"
  exit 1
fi

if [[ "${INCIDENT_PRESENT}" == "true" || -n "${INCIDENT_PATH_OUTPUT}" ]]; then
  write_reports \
    "FAIL" \
    "unexpected_incident_on_match" \
    "incident_lifecycle" \
    "${RUN_ID}" \
    "${EXPECTED_HASH}" \
    "${OBSERVED_HASH}" \
    "${CONTRACT_REQUEST_FINGERPRINT}" \
    "${MATCH_RESULT}" \
    "${INCIDENT_PRESENT}" \
    "${CONTRACT_VERSION}" \
    "${MANIFEST_REQUEST_FINGERPRINT}" \
    "${MANIFEST_ARTIFACT_HASH}" \
    "${INCIDENT_PATH_OUTPUT}" \
    "[\"unexpected_incident_on_match\"]"
  echo "determinism-replay-consistency: FAIL (unexpected_incident_on_match)"
  exit 3
fi

write_reports \
  "PASS" \
  "hash_match" \
  "pass" \
  "${RUN_ID}" \
  "${EXPECTED_HASH}" \
  "${OBSERVED_HASH}" \
  "${CONTRACT_REQUEST_FINGERPRINT}" \
  "${MATCH_RESULT}" \
  "${INCIDENT_PRESENT}" \
  "${CONTRACT_VERSION}" \
  "${MANIFEST_REQUEST_FINGERPRINT}" \
  "${MANIFEST_ARTIFACT_HASH}" \
  "${INCIDENT_PATH_OUTPUT}" \
  "[]"
echo "determinism-replay-consistency: PASS"
exit 0
