#!/usr/bin/env bash
# ==========================================
# Pre-CI Discipline Gate Simulation
# ==========================================
# Purpose:
#   Local fail-closed discipline layer before real CI.
#   Does NOT replace CI. CI remains mandatory for merge.
#
# Policy:
#   - Strict execution order
#   - Stop on first failure
#   - No auto-fix
#   - No bypass
#   - No interpretation of intent
#   - Manual intervention required on failure
#
# Development Awareness:
#   - During active development, hygiene failures may occur
#     if changes are intentionally uncommitted.
#   - This hook does NOT modify code or attempt fixes.
#   - Developer is responsible for resolving violations.
#
# Enforcement Mode:
#   FAIL-CLOSED
# ==========================================

set -euo pipefail

# RUN_ID: YYYYMMDDTHHMMSSZ-<git-short-sha>
# Matches evidence/run-<RUN_ID>/ directory naming convention.
_TS="$(date -u '+%Y%m%dT%H%M%SZ')"
_SHA="$(git rev-parse --short HEAD 2>/dev/null || echo "unknown")"
RUN_ID="${_TS}-${_SHA}"
EVIDENCE_DIR="${EVIDENCE_ROOT:-out/evidence}/run-${RUN_ID}/reports"

echo "== PRE-CI DISCIPLINE: START =="
echo "   RUN_ID: ${RUN_ID}"

run_gate() {
    local gate_cmd="$1"
    local gate_name="$2"

    echo ""
    echo ">> Running: $gate_name"
    echo "--------------------------------"

    if ! $gate_cmd; then
        echo ""
        echo "❌ GATE FAILURE: $gate_name"
        echo "Stopping execution (fail-closed)."
        echo ""
        echo "Inspect evidence under:"
        echo "  ${EVIDENCE_DIR}/"
        echo ""
        exit 2
    fi

    echo "✅ PASS: $gate_name"
}

# Strict execution order
# PRE_CI_MODE=1: boundary gate uses existing kernel.elf artifact (skip rebuild).
# CI remains mandatory for merge — full rebuild happens there.
run_gate "make ci-gate-abi" "ABI Gate"
run_gate "make PRE_CI_MODE=1 ci-gate-boundary" "Boundary Gate"
run_gate "make ci-gate-hygiene" "Hygiene Gate"
run_gate "make ci-gate-constitutional" "Constitutional Gate"
run_gate "make ci-gate-determinism-replay-consistency" "Determinism Replay Consistency Gate"

echo ""
echo "== PRE-CI DISCIPLINE: ALL GATES PASS =="
echo "Local discipline satisfied."
echo "Real CI remains mandatory for merge."
echo ""
echo "Note: kill-switch coverage (Phase-13) is CI-only."
echo "      Run 'make ci-kill-switch-phase13' for full kill-switch validation."
