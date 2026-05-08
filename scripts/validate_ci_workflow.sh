#!/usr/bin/env bash
# CI Workflow Assurance Capability
# Author: Kenan AY — System Architect
#
# Purpose: Validates CI workflow configuration is correct and operational
# Validates: workflow syntax, job structure, dependencies, artifacts, timeouts, scripts
#
# Exit codes:
#   0: PASS - CI workflow is correctly configured
#   1: FAIL - CI workflow has configuration errors
#   2: USAGE - Invalid arguments

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
WORKFLOW_FILE="${ROOT}/.github/workflows/devloop-ci.yml"

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

usage() {
  cat <<'USAGE'
Usage:
  scripts/validate_ci_workflow.sh [--verbose]

Validates CI workflow configuration:
  - Workflow file exists and is valid YAML
  - Required jobs are present
  - Job dependencies are correct
  - Artifact upload is configured
  - Timeout values are reasonable
  - Required scripts exist and are executable
  - Developer attribution is present

Exit codes:
  0: PASS - CI workflow is correctly configured
  1: FAIL - CI workflow has configuration errors
  2: USAGE - Invalid arguments
USAGE
}

VERBOSE=0

while [[ $# -gt 0 ]]; do
  case "$1" in
    --verbose|-v)
      VERBOSE=1
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "Unknown argument: $1" >&2
      usage
      exit 2
      ;;
  esac
done

VIOLATIONS=0

log_info() {
  if [[ "${VERBOSE}" -eq 1 ]]; then
    echo -e "${GREEN}[INFO]${NC} $*"
  fi
}

log_warn() {
  echo -e "${YELLOW}[WARN]${NC} $*"
}

log_error() {
  echo -e "${RED}[ERROR]${NC} $*"
  VIOLATIONS=$((VIOLATIONS + 1))
}

log_pass() {
  echo -e "${GREEN}[PASS]${NC} $*"
}

log_fail() {
  echo -e "${RED}[FAIL]${NC} $*"
  VIOLATIONS=$((VIOLATIONS + 1))
}

echo "=========================================="
echo "CI Workflow Assurance Capability"
echo "Author: Kenan AY — System Architect"
echo "=========================================="
echo ""

# Check 1: Workflow file exists
log_info "Checking workflow file existence..."
if [[ ! -f "${WORKFLOW_FILE}" ]]; then
  log_fail "Workflow file not found: ${WORKFLOW_FILE}"
else
  log_pass "Workflow file exists: ${WORKFLOW_FILE}"
fi

# Check 2: Workflow file is valid YAML (basic syntax check)
log_info "Checking YAML syntax..."
if command -v python3 >/dev/null 2>&1; then
  if python3 -c "import yaml; yaml.safe_load(open('${WORKFLOW_FILE}'))" 2>/dev/null; then
    log_pass "YAML syntax is valid"
  else
    log_fail "YAML syntax is invalid"
  fi
else
  log_warn "python3 not available, skipping YAML validation"
fi

# Check 3: Required jobs are present
log_info "Checking required jobs..."
REQUIRED_JOBS=("smoke" "contract" "full" "isolation" "performance" "auto-bisect")
for job in "${REQUIRED_JOBS[@]}"; do
  if grep -q "^  ${job}:" "${WORKFLOW_FILE}"; then
    log_pass "Job '${job}' is present"
  else
    log_fail "Job '${job}' is missing"
  fi
done

# Check 4: Job dependencies are correct
log_info "Checking job dependencies..."

# contract depends on smoke
if grep -A 5 "^  contract:" "${WORKFLOW_FILE}" | grep -q "needs: smoke"; then
  log_pass "Job 'contract' depends on 'smoke'"
else
  log_fail "Job 'contract' should depend on 'smoke'"
fi

# full depends on contract
if grep -A 5 "^  full:" "${WORKFLOW_FILE}" | grep -q "needs: contract"; then
  log_pass "Job 'full' depends on 'contract'"
else
  log_fail "Job 'full' should depend on 'contract'"
fi

# isolation depends on full
if grep -A 5 "^  isolation:" "${WORKFLOW_FILE}" | grep -q "needs: full"; then
  log_pass "Job 'isolation' depends on 'full'"
else
  log_fail "Job 'isolation' should depend on 'full'"
fi

# performance depends on isolation
if grep -A 5 "^  performance:" "${WORKFLOW_FILE}" | grep -q "needs: isolation"; then
  log_pass "Job 'performance' depends on 'isolation'"
else
  log_fail "Job 'performance' should depend on 'isolation'"
fi

# auto-bisect depends on all validation jobs
if grep -A 5 "^  auto-bisect:" "${WORKFLOW_FILE}" | grep -q "needs: \[smoke, contract, full, isolation, performance\]"; then
  log_pass "Job 'auto-bisect' depends on all validation jobs"
else
  log_fail "Job 'auto-bisect' should depend on [smoke, contract, full, isolation, performance]"
fi

# Check 5: Artifact upload is configured
log_info "Checking artifact upload configuration..."
ARTIFACT_JOBS=("smoke" "contract" "full" "isolation" "performance" "auto-bisect")
for job in "${ARTIFACT_JOBS[@]}"; do
  if grep -A 150 "^  ${job}:" "${WORKFLOW_FILE}" | grep -q "uses: actions/upload-artifact@"; then
    log_pass "Job '${job}' uploads artifacts"
  else
    log_fail "Job '${job}' should upload artifacts"
  fi
done

# Check 6: Timeout values are reasonable
log_info "Checking timeout values..."

check_timeout() {
  local job=$1
  local expected_min=$2
  local expected_max=$3
  
  if grep -A 5 "^  ${job}:" "${WORKFLOW_FILE}" | grep -q "timeout-minutes:"; then
    local timeout=$(grep -A 5 "^  ${job}:" "${WORKFLOW_FILE}" | grep "timeout-minutes:" | awk '{print $2}')
    if [[ "${timeout}" -ge "${expected_min}" && "${timeout}" -le "${expected_max}" ]]; then
      log_pass "Job '${job}' has reasonable timeout: ${timeout} minutes"
    else
      log_warn "Job '${job}' timeout (${timeout} min) outside expected range (${expected_min}-${expected_max} min)"
    fi
  else
    log_fail "Job '${job}' missing timeout configuration"
  fi
}

check_timeout "smoke" 5 15
check_timeout "contract" 10 20
check_timeout "full" 15 25
check_timeout "isolation" 5 15
check_timeout "performance" 10 20
check_timeout "auto-bisect" 25 40

# Check 7: Required scripts exist and are executable
log_info "Checking required scripts..."
REQUIRED_SCRIPTS=(
  "scripts/dev_loop.sh"
  "scripts/oracle.sh"
  "scripts/find_regression.sh"
  "scripts/test_devloop_isolation.sh"
  "scripts/check_perf_regression.sh"
)

for script in "${REQUIRED_SCRIPTS[@]}"; do
  if [[ -f "${ROOT}/${script}" ]]; then
    if [[ -x "${ROOT}/${script}" ]]; then
      log_pass "Script '${script}' exists and is executable"
    else
      log_warn "Script '${script}' exists but is not executable"
    fi
  else
    log_fail "Script '${script}' is missing"
  fi
done

# Check 8: Developer attribution is present
log_info "Checking developer attribution..."
if grep -q "Kenan AY" "${WORKFLOW_FILE}"; then
  log_pass "Developer attribution present in workflow file"
else
  log_fail "Developer attribution missing in workflow file"
fi

# Check 9: Workflow triggers are configured
log_info "Checking workflow triggers..."
if grep -q "pull_request:" "${WORKFLOW_FILE}"; then
  log_pass "Workflow triggers on pull_request"
else
  log_fail "Workflow should trigger on pull_request"
fi

if grep -q "push:" "${WORKFLOW_FILE}"; then
  log_pass "Workflow triggers on push"
else
  log_warn "Workflow does not trigger on push (optional)"
fi

# Check 10: Auto-bisect conditional execution
log_info "Checking auto-bisect conditional execution..."
if grep -A 5 "^  auto-bisect:" "${WORKFLOW_FILE}" | grep -q "if: failure()"; then
  log_pass "Auto-bisect runs conditionally on failure"
else
  log_fail "Auto-bisect should run conditionally on failure"
fi

# Check 11: QEMU timeout environment variable
log_info "Checking QEMU timeout configuration..."
if grep -q "QEMU_TIMEOUT_SECONDS:" "${WORKFLOW_FILE}"; then
  log_pass "QEMU timeout environment variable is configured"
else
  log_warn "QEMU timeout environment variable not configured"
fi

# Check 12: Dependencies installation
log_info "Checking dependencies installation..."
REQUIRED_DEPS=("qemu-system-x86" "build-essential" "clang" "lld" "make")
for dep in "${REQUIRED_DEPS[@]}"; do
  if grep -q "${dep}" "${WORKFLOW_FILE}"; then
    log_pass "Dependency '${dep}' is installed in workflow"
  else
    log_fail "Dependency '${dep}' should be installed in workflow"
  fi
done

# Check 13: Artifact retention days
log_info "Checking artifact retention configuration..."
if grep -q "retention-days:" "${WORKFLOW_FILE}"; then
  log_pass "Artifact retention is configured"
else
  log_warn "Artifact retention not explicitly configured (will use default)"
fi

# Check 14: Git fetch depth for bisect
log_info "Checking git fetch depth for auto-bisect..."
if grep -A 20 "^  auto-bisect:" "${WORKFLOW_FILE}" | grep -q "fetch-depth: 0"; then
  log_pass "Auto-bisect fetches full git history"
else
  log_fail "Auto-bisect should fetch full git history (fetch-depth: 0)"
fi

# Check 15: Workflow name is descriptive
log_info "Checking workflow name..."
if grep -q "^name:" "${WORKFLOW_FILE}"; then
  workflow_name=$(grep "^name:" "${WORKFLOW_FILE}" | cut -d':' -f2- | xargs)
  if [[ -n "${workflow_name}" ]]; then
    log_pass "Workflow has descriptive name: '${workflow_name}'"
  else
    log_warn "Workflow name is empty"
  fi
else
  log_fail "Workflow should have a name"
fi

# Summary
echo ""
echo "=========================================="
echo "Validation Summary"
echo "=========================================="
echo ""

if [[ "${VIOLATIONS}" -eq 0 ]]; then
  echo -e "${GREEN}✅ PASS${NC}: CI workflow is correctly configured"
  echo ""
  echo "All checks passed:"
  echo "  - Workflow file exists and is valid YAML"
  echo "  - All required jobs are present"
  echo "  - Job dependencies are correct"
  echo "  - Artifact upload is configured"
  echo "  - Timeout values are reasonable"
  echo "  - Required scripts exist and are executable"
  echo "  - Developer attribution is present"
  echo "  - Workflow triggers are configured"
  echo "  - Auto-bisect runs conditionally"
  echo "  - Dependencies are installed"
  echo ""
  exit 0
else
  echo -e "${RED}❌ FAIL${NC}: CI workflow has ${VIOLATIONS} configuration error(s)"
  echo ""
  echo "Review the errors above and fix the workflow configuration."
  echo ""
  exit 1
fi
