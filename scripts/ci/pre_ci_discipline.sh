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

echo "== PRE-CI DISCIPLINE: START =="

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
        echo "  evidence/run-<RUN_ID>/reports/"
        echo ""
        exit 2
    fi

    echo "✅ PASS: $gate_name"
}

# Strict execution order
run_gate "make ci-gate-abi" "ABI Gate"
run_gate "make ci-gate-boundary" "Boundary Gate"
run_gate "make ci-gate-hygiene" "Hygiene Gate"
run_gate "make ci-gate-constitutional" "Constitutional Gate"

echo ""
echo "== PRE-CI DISCIPLINE: ALL GATES PASS =="
echo "Local discipline satisfied."
echo "Real CI remains mandatory for merge."
