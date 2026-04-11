#!/bin/bash

# Phase-16 CI Gate: Kernel-Level Fail-Closed Proof Validation (Orchestration Layer)
# 
# This script is the ORCHESTRATION LAYER ONLY.
# All validation logic is delegated to validate_fail_closed_markers.py (SINGLE SOURCE OF TRUTH).
#
# Responsibilities:
#   - Check trace file existence
#   - Invoke Python validator
#   - Interpret validator exit code and JSON output
#   - Generate CI-compatible artifacts
#
# CRITICAL: Host tests DO NOT satisfy this requirement. Only QEMU kernel trace is authoritative.
#
# Requirements: 16.1-16.15

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
EVIDENCE_DIR="$PROJECT_ROOT/evidence/fail-closed-proof"
GATE_NAME="ci-gate-fail-closed-proof"
VALIDATOR="$SCRIPT_DIR/validate_fail_closed_markers.py"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Create evidence directory
mkdir -p "$EVIDENCE_DIR"

EVIDENCE_FILE="$EVIDENCE_DIR/failclosed_proof_evidence.json"
TRACE_LOG="$EVIDENCE_DIR/qemu_kernel_trace.log"

log_info "Starting $GATE_NAME orchestration..."
log_info "Evidence requirement: QEMU kernel trace with canonical marker flow"

# Check if QEMU trace file exists
if [[ ! -f "$TRACE_LOG" ]]; then
    log_error "QEMU kernel trace not found: $TRACE_LOG"
    log_error "This gate requires QEMU-based kernel trace evidence"
    log_error "Host tests and emulated tests DO NOT satisfy this requirement"
    
    cat > "$EVIDENCE_FILE" << EOF
{
  "gate": "$GATE_NAME",
  "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "result": "FAIL",
  "failure_code": "QEMU_TRACE_MISSING",
  "violations_detected": 1,
  "message": "QEMU kernel trace file not found. Kernel-level evidence is mandatory for this gate."
}
EOF
    echo "FAIL_CLOSED_PROOF_GATE=FAIL" > "$EVIDENCE_DIR/gate_result.env"
    echo "FAILURE_CODE=QEMU_TRACE_MISSING" >> "$EVIDENCE_DIR/gate_result.env"
    exit 1
fi

log_info "Found QEMU trace: $TRACE_LOG"

# Check if Python validator exists
if [[ ! -x "$VALIDATOR" ]]; then
    log_error "Python validator not found or not executable: $VALIDATOR"
    exit 1
fi

# Invoke Python validator (SINGLE SOURCE OF TRUTH)
log_info "Invoking Python validator (authoritative validation logic)..."

VALIDATOR_EXIT_CODE=0
"$VALIDATOR" "$TRACE_LOG" || VALIDATOR_EXIT_CODE=$?

# Check if validator produced evidence JSON
if [[ ! -f "$EVIDENCE_FILE" ]]; then
    log_error "Validator did not produce evidence JSON: $EVIDENCE_FILE"
    exit 1
fi

# Parse validator result from JSON
RESULT=$(python3 -c "import json; print(json.load(open('$EVIDENCE_FILE'))['result'])" 2>/dev/null || echo "UNKNOWN")
VIOLATIONS=$(python3 -c "import json; print(json.load(open('$EVIDENCE_FILE'))['violations_detected'])" 2>/dev/null || echo "0")
FAILURE_CODE=$(python3 -c "import json; print(json.load(open('$EVIDENCE_FILE')).get('failure_code', 'UNKNOWN'))" 2>/dev/null || echo "UNKNOWN")

# Generate CI artifacts based on validator result
if [[ "$RESULT" == "PASS" ]]; then
    log_info "$GATE_NAME: PASS - Kernel-level fail-closed proof validated"
    log_info "Evidence: $EVIDENCE_FILE"
    echo "FAIL_CLOSED_PROOF_GATE=PASS" > "$EVIDENCE_DIR/gate_result.env"
    exit 0
else
    log_error "$GATE_NAME: FAIL - $VIOLATIONS violations detected"
    log_error "Failure code: $FAILURE_CODE"
    log_error "Evidence: $EVIDENCE_FILE"
    log_error ""
    log_error "CRITICAL: This gate requires QEMU kernel trace evidence"
    log_error "Host tests and emulated tests DO NOT satisfy kernel-level claims"
    echo "FAIL_CLOSED_PROOF_GATE=FAIL" > "$EVIDENCE_DIR/gate_result.env"
    echo "VIOLATIONS_COUNT=$VIOLATIONS" >> "$EVIDENCE_DIR/gate_result.env"
    echo "FAILURE_CODE=$FAILURE_CODE" >> "$EVIDENCE_DIR/gate_result.env"
    exit 1
fi
