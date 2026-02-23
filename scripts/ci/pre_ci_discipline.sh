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
#   FAIL-CLOSED (advisory, does not block push)
#
# Authority:
#   Local environment (non-deterministic)
#   Real authority = GitHub Actions CI
# ==========================================

set -euo pipefail

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
echo "Environment: LOCAL (non-authoritative)"
echo "Mode: FAIL-CLOSED (advisory)"
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
echo "Gate Sequence:"
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

run_gate "make ci-gate-abi" "ABI Gate"
run_gate "make ci-gate-boundary" "Boundary Gate"
run_gate "make ci-gate-ring0-exports" "Ring0 Export Surface Gate"
run_gate "make ci-gate-hygiene" "Hygiene Gate"
run_gate "make ci-gate-constitutional" "Constitutional Gate"
run_gate "make ci-gate-workspace" "Workspace Gate"
run_gate "make ci-gate-syscall-v2-runtime" "Syscall v2 Runtime Gate"
run_gate "make ci-gate-sched-bridge-runtime" "Sched Bridge Runtime Gate"
run_gate "make ci-gate-policy-accept" "Policy Accept Gate"

echo ""
echo -e "${GREEN}=========================================="
echo "PRE-CI DISCIPLINE: ALL GATES PASS"
echo -e "==========================================${NC}"
echo ""
echo "Local discipline satisfied."
echo ""
echo -e "${YELLOW}⚠️  IMPORTANT:${NC}"
echo "  - This is an advisory check only"
echo "  - Real CI (GitHub Actions) remains mandatory"
echo "  - Performance gate skipped (requires CI authority)"
echo "  - Tooling isolation skipped (requires CI authority)"
echo ""
echo "Next steps:"
echo "  1. Commit changes if needed"
echo "  2. Push branch"
echo "  3. Open PR"
echo "  4. Wait for CI freeze (authoritative)"
echo ""
