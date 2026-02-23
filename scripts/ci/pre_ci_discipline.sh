#!/usr/bin/env bash
# ==========================================
# Pre-CI Discipline Gate Simulation
# ==========================================
# Purpose:
#   Layered local fail-closed discipline before real CI.
#   Does NOT replace CI. CI remains mandatory for merge.
#
# Layers:
#   fast - Core discipline gates (4 gates, ~30-60s)
#          ABI, Boundary, Hygiene, Constitutional
#          Purpose: Quick "is constitutional backbone broken?" check
#          Use: Daily development, reflex layer
#
#   full - Full discipline gates (9 gates, ~3-6min)
#          fast + Ring0 Exports, Workspace, Syscall v2 Runtime,
#          Sched Bridge Runtime, Policy Accept
#          Purpose: Pre-PR mandatory full discipline
#          Use: Before opening PR
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
#   - This script does NOT modify code or attempt fixes.
#   - Developer is responsible for resolving violations.
#
# Enforcement Mode:
#   FAIL-CLOSED (advisory, does not block push)
#
# Authority:
#   Local environment (non-deterministic)
#   Real authority = GitHub Actions CI
# ==========================================

set -euo pipefail

# Parse mode argument
MODE="${1:-full}"

if [ "$MODE" != "fast" ] && [ "$MODE" != "full" ]; then
    echo "ERROR: Invalid mode '$MODE'"
    echo "Usage: $0 [fast|full]"
    echo ""
    echo "  fast - Core discipline gates (4 gates, ~30-60s)"
    echo "  full - Full discipline gates (9 gates, ~3-6min, default)"
    exit 1
fi

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo ""
echo -e "${BLUE}=========================================="
echo "PRE-CI DISCIPLINE: START"
echo -e "==========================================${NC}"
echo ""
echo "Mode: $MODE"
echo "Environment: LOCAL (non-authoritative)"
echo "Enforcement: FAIL-CLOSED (advisory)"
echo "Authority: GitHub Actions CI (mandatory)"
echo ""

# Validate environment
if [ -z "${KERNEL_PROFILE:-}" ]; then
    echo -e "${YELLOW}⚠️  KERNEL_PROFILE not set, defaulting to 'validation'${NC}"
    export KERNEL_PROFILE=validation
fi

echo "Profile: $KERNEL_PROFILE"
echo ""

# Gate execution function
run_gate() {
    local gate_cmd="$1"
    local gate_name="$2"

    echo ""
    echo -e "${BLUE}>> Running: $gate_name${NC}"
    echo "--------------------------------"

    if ! $gate_cmd; then
        echo ""
        echo -e "${RED}❌ GATE FAILURE: $gate_name${NC}"
        echo "Stopping execution (fail-closed)."
        echo ""
        echo "Inspect evidence under:"
        echo "  evidence/run-<RUN_ID>/reports/"
        echo ""
        echo "Resolution:"
        echo "  1. Review gate failure details"
        echo "  2. Fix violations manually"
        echo "  3. Re-run: make pre-ci"
        echo ""
        echo "This is an advisory check. CI remains mandatory."
        echo ""
        exit 2
    fi

    echo -e "${GREEN}✅ PASS: $gate_name${NC}"
}

# Strict execution order (constitutional gate sequence)
if [ "$MODE" = "fast" ]; then
    echo "Gate Sequence (FAST - Core Discipline):"
    echo "  1. ABI Stability"
    echo "  2. Boundary Enforcement"
    echo "  3. Hygiene"
    echo "  4. Constitutional Compliance"
    echo ""
    echo "Estimated time: ~30-60 seconds"
    echo ""
    
    run_gate "make ci-gate-abi" "ABI Gate"
    run_gate "make ci-gate-boundary" "Boundary Gate"
    run_gate "make ci-gate-hygiene" "Hygiene Gate"
    run_gate "make ci-gate-constitutional" "Constitutional Gate"
else
    echo "Gate Sequence (FULL - Complete Discipline):"
    echo "  1. ABI Stability"
    echo "  2. Boundary Enforcement"
    echo "  3. Ring0 Export Surface"
    echo "  4. Hygiene"
    echo "  5. Constitutional Compliance"
    echo "  6. Workspace Integrity"
    echo "  7. Syscall v2 Runtime"
    echo "  8. Sched Bridge Runtime"
    echo "  9. Policy Accept"
    echo ""
    echo "Estimated time: ~3-6 minutes"
    echo ""
    
    run_gate "make ci-gate-abi" "ABI Gate"
    run_gate "make ci-gate-boundary" "Boundary Gate"
    run_gate "make ci-gate-ring0-exports" "Ring0 Export Surface Gate"
    run_gate "make ci-gate-hygiene" "Hygiene Gate"
    run_gate "make ci-gate-constitutional" "Constitutional Gate"
    run_gate "make ci-gate-workspace" "Workspace Gate"
    run_gate "make ci-gate-syscall-v2-runtime" "Syscall v2 Runtime Gate"
    run_gate "make ci-gate-sched-bridge-runtime" "Sched Bridge Runtime Gate"
    run_gate "make ci-gate-policy-accept" "Policy Accept Gate"
fi

echo ""
echo -e "${GREEN}=========================================="
if [ "$MODE" = "fast" ]; then
    echo "PRE-CI DISCIPLINE (FAST): ALL GATES PASS"
else
    echo "PRE-CI DISCIPLINE (FULL): ALL GATES PASS"
fi
echo -e "==========================================${NC}"
echo ""
echo "Local discipline satisfied."
echo ""
echo -e "${YELLOW}⚠️  IMPORTANT:${NC}"
echo "  - This is an advisory check only"
echo "  - Real CI (GitHub Actions) remains mandatory"
if [ "$MODE" = "fast" ]; then
    echo "  - Runtime gates skipped (use 'make pre-ci' for full check)"
fi
echo "  - Performance gate skipped (requires CI authority)"
echo "  - Tooling isolation skipped (requires CI authority)"
echo ""
echo "Next steps:"
if [ "$MODE" = "fast" ]; then
    echo "  1. Continue development"
    echo "  2. Run 'make pre-ci' before opening PR"
else
    echo "  1. Commit changes if needed"
    echo "  2. Push branch"
    echo "  3. Open PR"
    echo "  4. Wait for CI freeze (authoritative)"
fi
echo ""
