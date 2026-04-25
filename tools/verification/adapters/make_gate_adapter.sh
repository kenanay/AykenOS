#!/usr/bin/env bash
#
# Make Gate Adapter - Minimal adapter for existing AykenOS gates
#
# CRITICAL: This is a THIN WRAPPER around evidence_adapter.py
# ALL logic is in Python - bash only validates inputs and delegates
#
# This ensures:
# - Single source of truth (Python)
# - No jq JSON construction (fragile)
# - No verdict determination in bash
# - No fake data generation
# - Deterministic evidence generation
#
# Requirements: 10.1, 10.2, 10.3, 10.4, 10.6, 10.7, 10.8
#
# Usage:
#   AYKEN_RUN_ID=<run_id> AYKEN_EVIDENCE_DIR=<dir> make_gate_adapter.sh \
#     --gate-id <id> \
#     --command <cmd> \
#     --exit-code <code> \
#     --duration-ms <ms> \
#     --determinism-level <level> \
#     --raw-output <path> \
#     [--build-fingerprint-required]
#

set -euo pipefail

# Script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
EVIDENCE_HELPER="${SCRIPT_DIR}/evidence_adapter.py"

# Verify Python helper exists
if [[ ! -f "${EVIDENCE_HELPER}" ]]; then
    echo "ERROR: Evidence helper not found: ${EVIDENCE_HELPER}" >&2
    exit 1
fi

# Parse arguments
GATE_ID=""
COMMAND=""
EXIT_CODE=""
DURATION_MS=""
DETERMINISM_LEVEL=""
RAW_OUTPUT=""
BUILD_FINGERPRINT_REQUIRED=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --gate-id)
            GATE_ID="$2"
            shift 2
            ;;
        --command)
            COMMAND="$2"
            shift 2
            ;;
        --exit-code)
            EXIT_CODE="$2"
            shift 2
            ;;
        --duration-ms)
            DURATION_MS="$2"
            shift 2
            ;;
        --determinism-level)
            DETERMINISM_LEVEL="$2"
            shift 2
            ;;
        --raw-output)
            RAW_OUTPUT="$2"
            shift 2
            ;;
        --build-fingerprint-required)
            BUILD_FINGERPRINT_REQUIRED=true
            shift
            ;;
        *)
            echo "ERROR: Unknown argument: $1" >&2
            exit 1
            ;;
    esac
done

# Validate required arguments
if [[ -z "${GATE_ID}" ]]; then
    echo "ERROR: --gate-id is required" >&2
    exit 1
fi

if [[ -z "${COMMAND}" ]]; then
    echo "ERROR: --command is required" >&2
    exit 1
fi

if [[ -z "${EXIT_CODE}" ]]; then
    echo "ERROR: --exit-code is required" >&2
    exit 1
fi

if [[ -z "${DURATION_MS}" ]]; then
    echo "ERROR: --duration-ms is required" >&2
    exit 1
fi

if [[ -z "${DETERMINISM_LEVEL}" ]]; then
    echo "ERROR: --determinism-level is required" >&2
    exit 1
fi

if [[ -z "${RAW_OUTPUT}" ]]; then
    echo "ERROR: --raw-output is required" >&2
    exit 1
fi

# Validate environment variables (CRITICAL)
if [[ -z "${AYKEN_RUN_ID:-}" ]]; then
    echo "ERROR: AYKEN_RUN_ID environment variable not set" >&2
    echo "This adapter must be called by the verification orchestrator" >&2
    exit 1
fi

if [[ -z "${AYKEN_EVIDENCE_DIR:-}" ]]; then
    echo "ERROR: AYKEN_EVIDENCE_DIR environment variable not set" >&2
    echo "This adapter must be called by the verification orchestrator" >&2
    exit 1
fi

# Validate run_id format (ISO 8601)
if ! echo "${AYKEN_RUN_ID}" | grep -qE '^[0-9]{4}-[0-9]{2}-[0-9]{2}T[0-9]{2}:[0-9]{2}:[0-9]{2}Z$'; then
    echo "ERROR: AYKEN_RUN_ID has invalid format: ${AYKEN_RUN_ID}" >&2
    echo "Expected ISO 8601 format: YYYY-MM-DDTHH:MM:SSZ" >&2
    exit 1
fi

# Validate raw output file exists
if [[ ! -f "${RAW_OUTPUT}" ]]; then
    echo "ERROR: Raw output file not found: ${RAW_OUTPUT}" >&2
    exit 1
fi

# Validate determinism level
case "${DETERMINISM_LEVEL}" in
    artifact|trace|marker|scheduling-independent)
        # Valid
        ;;
    *)
        echo "ERROR: Invalid determinism level: ${DETERMINISM_LEVEL}" >&2
        echo "Valid values: artifact, trace, marker, scheduling-independent" >&2
        exit 1
        ;;
esac

# Create evidence directory if it doesn't exist
mkdir -p "${AYKEN_EVIDENCE_DIR}"

# Output file
OUTPUT_FILE="${AYKEN_EVIDENCE_DIR}/report.json"

# CRITICAL: Delegate ALL logic to Python
# This ensures single source of truth and eliminates jq fragility
PYTHON_ARGS=(
    "generate"
    "--gate-id" "${GATE_ID}"
    "--run-id" "${AYKEN_RUN_ID}"
    "--command" "${COMMAND}"
    "--exit-code" "${EXIT_CODE}"
    "--duration-ms" "${DURATION_MS}"
    "--determinism-level" "${DETERMINISM_LEVEL}"
    "--raw-output" "${RAW_OUTPUT}"
    "--output" "${OUTPUT_FILE}"
)

if [[ "${BUILD_FINGERPRINT_REQUIRED}" == true ]]; then
    PYTHON_ARGS+=("--build-fingerprint-required")
fi

# Execute Python helper
# All validation, extraction, and evidence generation happens here
python3 "${EVIDENCE_HELPER}" "${PYTHON_ARGS[@]}"

# Python helper handles all output and exit codes
# If we reach here, evidence was generated successfully
exit 0
