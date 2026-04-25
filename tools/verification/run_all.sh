#!/usr/bin/env bash
#
# AykenOS Verification Layer - Orchestrator
# 
# Purpose: Execute verification gates in dependency order and generate evidence-based reports
# Design Principle: "Verification reads. It does not mutate."
#
# Author: Kenan AY - Architectural Steward
# Date: 2026-04-25
# Version: 1.0

set -euo pipefail

# ============================================================================
# RUNTIME CONTRACT ENFORCEMENT
# ============================================================================

# Enforce Bash 4+ requirement for associative arrays and modern features
if [[ "${BASH_VERSINFO[0]}" -lt 4 ]]; then
    echo "[VERIFY][FATAL] Bash 4+ required for verification layer (found ${BASH_VERSION})"
    echo "[VERIFY][FATAL] Current environment is not deterministic - verification cannot proceed"
    echo "[VERIFY][FATAL] Install modern bash: brew install bash (macOS) or apt-get install bash (Linux)"
    exit 127
fi

echo "[VERIFY][RUNTIME] Bash ${BASH_VERSION} - runtime contract satisfied"

# ============================================================================
# GLOBAL VARIABLES
# ============================================================================

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
VALIDATORS_DIR="${SCRIPT_DIR}/validators"

# Default configuration
DEFAULT_TIER="standard"
DEFAULT_MODE="hard_gate"
DEFAULT_MANIFEST="${SCRIPT_DIR}/manifest.json"
DEFAULT_TIMEOUT=300
VERBOSE=0

# Command-line arguments
TIER="${DEFAULT_TIER}"
MODE="${DEFAULT_MODE}"
MANIFEST_PATH="${DEFAULT_MANIFEST}"

# Run-specific variables
RUN_ID=""
EVIDENCE_BASE_DIR="${PROJECT_ROOT}/out/evidence/verification"
RUN_EVIDENCE_DIR=""

# Gate execution state
declare -A GATE_VERDICTS
declare -A GATE_BLOCKING
declare -A GATE_DETERMINISM_LEVELS
declare -A GATE_EVIDENCE_PATHS
declare -A GATE_DEPENDENCIES

# Report counters
GATES_CHECKED=0
GATES_PASSED=0
GATES_FAILED=0
GATES_SKIPPED=0
GATES_ERROR=0
GATES_TIMEOUT=0

# Determinism counters
DETERMINISM_ARTIFACT=0
DETERMINISM_TRACE=0
DETERMINISM_MARKER=0
DETERMINISM_SCHEDULING_INDEPENDENT=0

# Overall status
OVERALL_STATUS="PASS"

# Evidence files for hash chain
EVIDENCE_FILES_JSON="[]"

# ============================================================================
# SAFETY NET: EXIT HANDLER
# ============================================================================

# Cleanup handler to finalize any RUNNING gates on unexpected exit
cleanup_running_gates() {
    verbose "Cleanup handler: checking for RUNNING gates..."
    
    # Find all status.json files with RUNNING status
    if [[ -d "${RUN_EVIDENCE_DIR}/gates" ]]; then
        find "${RUN_EVIDENCE_DIR}/gates" -name "status.json" -type f 2>/dev/null | while read -r status_file; do
            local status
            status=$(python3 -c "
import json
try:
    with open('${status_file}', 'r') as f:
        data = json.load(f)
    print(data.get('status', ''))
except:
    print('')
" 2>/dev/null)
            
            if [[ "${status}" == "RUNNING" ]]; then
                local gate_id
                gate_id=$(python3 -c "
import json
try:
    with open('${status_file}', 'r') as f:
        data = json.load(f)
    print(data.get('gate_id', ''))
except:
    print('')
" 2>/dev/null)
                
                if [[ -n "${gate_id}" ]]; then
                    warn "Cleanup: Gate ${gate_id} left in RUNNING state - marking as ERROR"
                    write_gate_status "${gate_id}" "ERROR" "orchestrator_interrupted"
                fi
            fi
        done
    fi
}

# Register cleanup handler
trap cleanup_running_gates EXIT

# ============================================================================
# UTILITY FUNCTIONS
# ============================================================================

# Print error message to stderr
error() {
    echo "ERROR: $*" >&2
}

# Print warning message to stderr
warn() {
    echo "WARNING: $*" >&2
}

# Print info message
info() {
    echo "INFO: $*"
}

# Print verbose message (only if --verbose is enabled)
verbose() {
    if [[ ${VERBOSE} -eq 1 ]]; then
        echo "VERBOSE: $*" >&2
    fi
}

# Print usage information
usage() {
    cat <<EOF
Usage: $(basename "$0") [OPTIONS]

AykenOS Verification Layer - Execute verification gates and generate reports

OPTIONS:
    --tier TIER           Performance tier to execute (fast, standard, heavy)
                          Default: ${DEFAULT_TIER}
    
    --mode MODE           Execution mode (shadow, hard_gate)
                          shadow: failures logged but don't block (exit 0)
                          hard_gate: failures block execution (exit 1)
                          Default: ${DEFAULT_MODE}
    
    --manifest PATH       Path to manifest.json
                          Default: ${DEFAULT_MANIFEST}
    
    --verbose             Enable verbose diagnostic output
    
    -h, --help            Show this help message

EXAMPLES:
    # Run standard tier in hard gate mode (default)
    $(basename "$0")
    
    # Run fast tier only
    $(basename "$0") --tier fast
    
    # Run in shadow mode (non-blocking)
    $(basename "$0") --mode shadow
    
    # Run with custom manifest
    $(basename "$0") --manifest /path/to/custom-manifest.json
    
    # Run with verbose output
    $(basename "$0") --verbose

EXIT CODES:
    0    Success (all blocking gates passed) or shadow mode
    1    Failure (one or more blocking gates failed) in hard_gate mode
    2    Invalid arguments or configuration error

EOF
}

# Generate unique run_id in ISO 8601 format
generate_run_id() {
    # Format: YYYY-MM-DDTHH:MM:SSZ (colons, not dashes in time)
    date -u +"%Y-%m-%dT%H:%M:%SZ"
}

# Setup evidence directory structure
setup_evidence_directory() {
    local run_id="$1"
    
    RUN_EVIDENCE_DIR="${EVIDENCE_BASE_DIR}/${run_id}"
    
    verbose "Creating evidence directory: ${RUN_EVIDENCE_DIR}"
    
    mkdir -p "${RUN_EVIDENCE_DIR}/gates"
    
    if [[ ! -d "${RUN_EVIDENCE_DIR}/gates" ]]; then
        error "Failed to create evidence directory: ${RUN_EVIDENCE_DIR}/gates"
        return 1
    fi
    
    verbose "Evidence directory created successfully"
    return 0
}

# ============================================================================
# MANIFEST VALIDATION AND PARSING
# ============================================================================

# Validate manifest using Python validator
validate_manifest() {
    local manifest_path="$1"
    
    info "Validating manifest: ${manifest_path}"
    
    if [[ ! -f "${manifest_path}" ]]; then
        error "Manifest file not found: ${manifest_path}"
        return 1
    fi
    
    # Call Python validator
    if ! python3 "${VALIDATORS_DIR}/validate_manifest.py" "${manifest_path}"; then
        error "Manifest validation failed"
        return 1
    fi
    
    info "Manifest validation passed"
    return 0
}

# Parse manifest and extract gate definitions
parse_manifest() {
    local manifest_path="$1"
    
    verbose "Parsing manifest: ${manifest_path}"
    
    # Extract gate IDs
    local gate_ids
    gate_ids=$(python3 -c "
import json
import sys

with open('${manifest_path}', 'r') as f:
    manifest = json.load(f)

for gate in manifest.get('gates', []):
    print(gate['id'])
" 2>/dev/null)
    
    if [[ -z "${gate_ids}" ]]; then
        error "No gates found in manifest"
        return 1
    fi
    
    echo "${gate_ids}"
    return 0
}

# Get gate configuration from manifest
get_gate_config() {
    local manifest_path="$1"
    local gate_id="$2"
    local field="$3"
    
    python3 -c "
import json
import sys

with open('${manifest_path}', 'r') as f:
    manifest = json.load(f)

for gate in manifest.get('gates', []):
    if gate['id'] == '${gate_id}':
        value = gate.get('${field}')
        if value is not None:
            if isinstance(value, bool):
                print('true' if value else 'false')
            elif isinstance(value, list):
                print(' '.join(value))
            else:
                print(value)
        break
" 2>/dev/null
}

# ============================================================================
# DEPENDENCY RESOLUTION
# ============================================================================

# Build dependency graph and perform topological sort using Kahn's algorithm
topological_sort() {
    local manifest_path="$1"
    
    verbose "Building dependency graph and performing topological sort"
    
    # Use Python for dependency resolution
    local sorted_gates
    sorted_gates=$(python3 -c "
import json
import sys
from collections import defaultdict, deque

with open('${manifest_path}', 'r') as f:
    manifest = json.load(f)

gates = manifest.get('gates', [])

# Build adjacency list and in-degree map
adj_list = defaultdict(list)
in_degree = defaultdict(int)
gate_ids = set()

for gate in gates:
    gate_id = gate['id']
    gate_ids.add(gate_id)
    if gate_id not in in_degree:
        in_degree[gate_id] = 0
    
    for dep in gate.get('depends_on', []):
        adj_list[dep].append(gate_id)
        in_degree[gate_id] += 1

# Kahn's algorithm
queue = deque([gid for gid in gate_ids if in_degree[gid] == 0])
sorted_list = []

while queue:
    current = queue.popleft()
    sorted_list.append(current)
    
    for neighbor in adj_list[current]:
        in_degree[neighbor] -= 1
        if in_degree[neighbor] == 0:
            queue.append(neighbor)

# Check for circular dependencies
if len(sorted_list) != len(gate_ids):
    print('CIRCULAR_DEPENDENCY_DETECTED', file=sys.stderr)
    sys.exit(1)

# Output sorted gate IDs
for gate_id in sorted_list:
    print(gate_id)
" 2>&1)
    
    local exit_code=$?
    
    if [[ ${exit_code} -ne 0 ]]; then
        error "Circular dependency detected in manifest"
        return 1
    fi
    
    if [[ -z "${sorted_gates}" ]]; then
        error "Failed to sort gates"
        return 1
    fi
    
    verbose "Topological sort completed successfully"
    echo "${sorted_gates}"
    return 0
}

# ============================================================================
# TIER FILTERING
# ============================================================================

# Filter gates by performance tier
filter_gates_by_tier() {
    local manifest_path="$1"
    local tier="$2"
    shift 2
    local gate_ids=("$@")
    
    verbose "Filtering gates by tier: ${tier}"
    
    local filtered_gates=()
    
    for gate_id in "${gate_ids[@]}"; do
        local gate_tier
        gate_tier=$(get_gate_config "${manifest_path}" "${gate_id}" "performance_tier")
        
        # Determine if gate should be included based on tier
        local include=0
        case "${tier}" in
            fast)
                [[ "${gate_tier}" == "fast" ]] && include=1
                ;;
            standard)
                [[ "${gate_tier}" == "fast" || "${gate_tier}" == "standard" ]] && include=1
                ;;
            heavy)
                include=1  # Include all gates
                ;;
            *)
                error "Invalid tier: ${tier}"
                return 1
                ;;
        esac
        
        if [[ ${include} -eq 1 ]]; then
            filtered_gates+=("${gate_id}")
            verbose "  Including gate: ${gate_id} (tier: ${gate_tier})"
        else
            verbose "  Excluding gate: ${gate_id} (tier: ${gate_tier})"
        fi
    done
    
    if [[ ${#filtered_gates[@]} -eq 0 ]]; then
        warn "No gates match tier filter: ${tier}"
    fi
    
    echo "${filtered_gates[@]}"
    return 0
}

# ============================================================================
# GATE EXECUTION
# ============================================================================

# CRITICAL: Write atomic gate status
# Status transitions: NOT_STARTED → RUNNING → PASS/FAIL/ERROR/TIMEOUT/SKIPPED
# Atomic write prevents partial state on crash
write_gate_status() {
    local gate_id="$1"
    local status="$2"
    local reason="${3:-}"
    
    local status_dir="${RUN_EVIDENCE_DIR}/gates/${gate_id}"
    mkdir -p "${status_dir}"
    
    local status_file="${status_dir}/status.json"
    local temp_file="${status_dir}/.status.json.tmp"
    
    # Write to temp file
    cat > "${temp_file}" <<EOF
{
  "gate_id": "${gate_id}",
  "status": "${status}",
  "reason": "${reason}",
  "timestamp": "$(date -u +"%Y-%m-%dT%H:%M:%SZ")",
  "run_id": "${RUN_ID}"
}
EOF
    
    # Atomic rename
    mv "${temp_file}" "${status_file}"
    
    verbose "  Gate status: ${status} (${reason})"
}

# Validate command against allowlist
validate_command() {
    local command="$1"
    
    # Allowlist pattern: make ci-gate-*
    if [[ "${command}" =~ ^make\ ci-gate- ]]; then
        return 0
    fi
    
    error "Command not in allowlist: ${command}"
    error "Allowed pattern: make ci-gate-*"
    return 1
}

# Execute a single gate
execute_gate() {
    local gate_id="$1"
    local manifest_path="$2"
    
    info "Executing gate: ${gate_id}"
    
    # Get gate configuration
    local command
    command=$(get_gate_config "${manifest_path}" "${gate_id}" "command")
    
    local timeout
    timeout=$(get_gate_config "${manifest_path}" "${gate_id}" "timeout")
    [[ -z "${timeout}" ]] && timeout=${DEFAULT_TIMEOUT}
    
    local blocking
    blocking=$(get_gate_config "${manifest_path}" "${gate_id}" "blocking")
    
    local determinism_level
    determinism_level=$(get_gate_config "${manifest_path}" "${gate_id}" "determinism_level")
    
    local evidence_path
    evidence_path=$(get_gate_config "${manifest_path}" "${gate_id}" "evidence")
    
    # Store gate metadata
    GATE_BLOCKING["${gate_id}"]="${blocking}"
    GATE_DETERMINISM_LEVELS["${gate_id}"]="${determinism_level}"
    GATE_EVIDENCE_PATHS["${gate_id}"]="${evidence_path}"
    
    # CRITICAL: Check dependencies (SKIPPED propagates through chain)
    local depends_on
    depends_on=$(get_gate_config "${manifest_path}" "${gate_id}" "depends_on")
    
    if [[ -n "${depends_on}" ]]; then
        for dep in ${depends_on}; do
            local dep_verdict="${GATE_VERDICTS[${dep}]:-}"
            
            # CRITICAL: Any non-PASS dependency causes skip
            # This includes: FAIL, ERROR, TIMEOUT, SKIPPED
            if [[ "${dep_verdict}" != "PASS" ]]; then
                warn "Skipping gate ${gate_id}: dependency ${dep} has verdict ${dep_verdict}"
                GATE_VERDICTS["${gate_id}"]="SKIPPED"
                ((GATES_SKIPPED++))
                
                # CRITICAL: Write atomic gate status
                write_gate_status "${gate_id}" "SKIPPED" "dependency_${dep}_${dep_verdict}"
                
                return 0
            fi
        done
    fi
    
    # Validate command
    if ! validate_command "${command}"; then
        error "Gate ${gate_id}: command validation failed"
        GATE_VERDICTS["${gate_id}"]="ERROR"
        ((GATES_ERROR++))
        write_gate_status "${gate_id}" "ERROR" "command_validation_failed"
        return 0  # ✅ NEVER FAIL - orchestrator must continue
    fi
    
    # CRITICAL: Find latest attempt directory (or create attempt-1)
    # Race condition protection: use atomic directory creation
    local gate_base_dir="${RUN_EVIDENCE_DIR}/gates/${gate_id}"
    mkdir -p "${gate_base_dir}"
    
    local attempt_num=1
    local gate_evidence_dir=""
    local max_retries=10
    local retry=0
    
    # CRITICAL: Atomic attempt directory creation (prevents concurrent overwrite)
    while [[ ${retry} -lt ${max_retries} ]]; do
        # Find highest existing attempt number
        local max_attempt=0
        if [[ -d "${gate_base_dir}" ]]; then
            local found_attempt
            found_attempt=$(find "${gate_base_dir}" -maxdepth 1 -type d -name "attempt-*" 2>/dev/null | \
                          sed 's/.*attempt-//' | sort -n | tail -1)
            if [[ -n "${found_attempt}" ]]; then
                max_attempt=${found_attempt}
            fi
        fi
        
        attempt_num=$((max_attempt + 1))
        gate_evidence_dir="${gate_base_dir}/attempt-${attempt_num}"
        
        # CRITICAL: Atomic directory creation (fails if exists)
        if mkdir "${gate_evidence_dir}" 2>/dev/null; then
            # Success - we own this attempt directory
            break
        else
            # Another process created this attempt, retry with next number
            ((retry++))
            verbose "  Attempt ${attempt_num} already exists, retrying..."
            sleep 0.1
        fi
    done
    
    if [[ ${retry} -ge ${max_retries} ]]; then
        error "Gate ${gate_id}: failed to create attempt directory after ${max_retries} retries"
        GATE_VERDICTS["${gate_id}"]="ERROR"
        ((GATES_ERROR++))
        write_gate_status "${gate_id}" "ERROR" "attempt_creation_failed"
        return 0  # ✅ NEVER FAIL - orchestrator must continue
    fi
    
    # CRITICAL: Write atomic RUNNING status
    write_gate_status "${gate_id}" "RUNNING" "executing_attempt_${attempt_num}"
    
    # Set environment variables
    export AYKEN_RUN_ID="${RUN_ID}"
    export AYKEN_EVIDENCE_DIR="${gate_evidence_dir}"
    
    verbose "  Command: ${command}"
    verbose "  Timeout: ${timeout}s"
    verbose "  Attempt: ${attempt_num}"
    verbose "  Evidence dir: ${gate_evidence_dir}"
    verbose "  AYKEN_RUN_ID: ${AYKEN_RUN_ID}"
    verbose "  AYKEN_EVIDENCE_DIR: ${AYKEN_EVIDENCE_DIR}"
    
    # Execute gate command with timeout
    local start_time
    start_time=$(date +%s)
    
    local exit_code=0
    local timed_out=0
    
    # CRITICAL: Use array execution to prevent command injection
    # Split command into array (assumes "make ci-gate-*" format)
    # Additional safety: validate no shell metacharacters in command
    local cmd_array=()
    read -ra cmd_array <<< "${command}"
    
    # CRITICAL: Validate no shell metacharacters (defense in depth)
    # Allowlist already checked "make ci-gate-*" pattern
    # This adds extra protection against edge cases
    if [[ "${command}" =~ [';|&$`<>(){}'] ]]; then
        error "Gate ${gate_id}: command contains shell metacharacters"
        error "  Command: ${command}"
        error "  This should have been caught by allowlist validation"
        GATE_VERDICTS["${gate_id}"]="ERROR"
        ((GATES_ERROR++))
        write_gate_status "${gate_id}" "ERROR" "command_injection_attempt"
        
        if [[ "${blocking}" == "true" ]]; then
            OVERALL_STATUS="FAIL"
        fi
        
        return 0  # ✅ NEVER FAIL - orchestrator must continue
    fi
    
    # Execute with timeout (safer than bash -c with string interpolation)
    set +e
    if timeout "${timeout}s" "${cmd_array[@]}" > "${gate_evidence_dir}/gate_output.log" 2>&1; then
        exit_code=0
    else
        exit_code=$?
        # Exit code 124 indicates timeout
        if [[ ${exit_code} -eq 124 ]]; then
            timed_out=1
        fi
    fi
    set -e
    
    local end_time
    end_time=$(date +%s)
    local duration=$((end_time - start_time))
    
    verbose "  Exit code: ${exit_code}"
    verbose "  Duration: ${duration}s"
    
    # Handle timeout
    if [[ ${timed_out} -eq 1 ]]; then
        error "Gate ${gate_id}: command timed out after ${timeout}s"
        GATE_VERDICTS["${gate_id}"]="TIMEOUT"
        ((GATES_TIMEOUT++))
        
        write_gate_status "${gate_id}" "TIMEOUT" "exceeded_${timeout}s"
        
        if [[ "${blocking}" == "true" ]]; then
            OVERALL_STATUS="FAIL"
        fi
        
        return 0
    fi
    
    # CRITICAL: Locate evidence file in AYKEN_EVIDENCE_DIR (NOT manifest path)
    # This enforces evidence path determinism and run_id isolation
    local evidence_file="${gate_evidence_dir}/report.json"
    
    if [[ ! -f "${evidence_file}" ]]; then
        error "Gate ${gate_id}: evidence file not found: ${evidence_file}"
        error "  Expected at: ${gate_evidence_dir}/report.json"
        error "  Gate MUST write evidence to AYKEN_EVIDENCE_DIR"
        error "  AYKEN_EVIDENCE_DIR was set to: ${gate_evidence_dir}"
        error "  This enforces run_id isolation and prevents stale evidence reads"
        GATE_VERDICTS["${gate_id}"]="ERROR"
        ((GATES_ERROR++))
        
        write_gate_status "${gate_id}" "ERROR" "evidence_missing"
        
        if [[ "${blocking}" == "true" ]]; then
            OVERALL_STATUS="FAIL"
        fi
        
        return 0
    fi
    
    # CRITICAL: Create temporary gate config for validator
    local gate_config_file="${gate_evidence_dir}/gate_config.json"
    python3 -c "
import json
import sys

with open('${manifest_path}', 'r') as f:
    manifest = json.load(f)

for gate in manifest.get('gates', []):
    if gate['id'] == '${gate_id}':
        with open('${gate_config_file}', 'w') as out:
            json.dump(gate, out, indent=2)
        break
" 2>/dev/null
    
    # CRITICAL: Validate evidence and get gate_pass from validator
    verbose "  Validating evidence: ${evidence_file}"
    
    # CRITICAL: Disable set -e temporarily to capture exit code
    # Validator exit 1 is expected for gate failures, not script errors
    set +e
    
    # CRITICAL: Separate stdout (JSON) from stderr (human messages)
    local validation_stdout_file="${gate_evidence_dir}/validation_stdout.json"
    local validation_stderr_file="${gate_evidence_dir}/validation_stderr.log"
    
    python3 "${VALIDATORS_DIR}/validate_evidence.py" \
        "${evidence_file}" \
        "${gate_config_file}" \
        "${RUN_ID}" \
        "${command}" \
        > "${validation_stdout_file}" \
        2> "${validation_stderr_file}"
    
    local validation_exit_code=$?
    set -e
    
    verbose "  Validation exit code: ${validation_exit_code}"
    
    # Read stderr for debugging
    if [[ -s "${validation_stderr_file}" ]]; then
        verbose "  Validation stderr:"
        verbose "$(cat "${validation_stderr_file}")"
    fi
    
    # CRITICAL: Parse JSON output from validator (stdout only)
    # If parse fails, this is validator crash (ERROR), not gate failure (FAIL)
    local gate_pass="false"
    local verdict="ERROR"
    local valid="false"
    local parse_success="true"
    
    # Check if stdout file exists and is not empty
    if [[ ! -s "${validation_stdout_file}" ]]; then
        error "Validator produced no JSON output (crash or no stdout)"
        parse_success="false"
    else
        # Try to parse JSON from stdout file
        if ! gate_pass=$(python3 -c "
import json
try:
    with open('${validation_stdout_file}', 'r') as f:
        data = json.load(f)
    print('true' if data.get('gate_pass', False) else 'false')
except Exception as e:
    print('false')
    exit(1)
" 2>/dev/null); then
            parse_success="false"
        fi
        
        if ! verdict=$(python3 -c "
import json
try:
    with open('${validation_stdout_file}', 'r') as f:
        data = json.load(f)
    print(data.get('verdict', 'ERROR'))
except Exception as e:
    print('ERROR')
    exit(1)
" 2>/dev/null); then
            parse_success="false"
        fi
        
        if ! valid=$(python3 -c "
import json
try:
    with open('${validation_stdout_file}', 'r') as f:
        data = json.load(f)
    print('true' if data.get('valid', False) else 'false')
except Exception as e:
    print('false')
    exit(1)
" 2>/dev/null); then
            parse_success="false"
        fi
    fi
    
    # Clean up temporary files
    rm -f "${validation_stdout_file}" "${validation_stderr_file}"
    
    # CRITICAL: Distinguish validator crash from gate failure
    if [[ "${parse_success}" == "false" ]]; then
        error "Gate ${gate_id}: ERROR (validator output not parseable)"
        error "  Validator crash ≠ gate failure"
        error "  Check validation logs in: ${gate_evidence_dir}"
        GATE_VERDICTS["${gate_id}"]="ERROR"
        ((GATES_ERROR++))
        write_gate_status "${gate_id}" "ERROR" "validator_crash"
        
        if [[ "${blocking}" == "true" ]]; then
            OVERALL_STATUS="FAIL"
        fi
        
        return 0
    fi
    
    # CRITICAL: Trust validator authority completely
    # gate_pass from JSON is single source of truth
    # BUT: Gate command exit code MUST also be checked
    # 
    # Gate PASS requires:
    # 1. Command exit code == 0 (gate executed successfully)
    # 2. Validator gate_pass == true (evidence valid and verdict matches)
    #
    # This prevents: command fail → evidence PASS → gate PASS (wrong!)
    
    local final_verdict
    
    # CRITICAL: Check command exit code first
    if [[ ${exit_code} -ne 0 ]]; then
        # Command failed → gate cannot pass regardless of evidence
        final_verdict="ERROR"
        ((GATES_ERROR++))
        error "Gate ${gate_id}: ERROR (command exit code ${exit_code})"
        error "  Command failure overrides evidence verdict"
        write_gate_status "${gate_id}" "ERROR" "command_exit_${exit_code}"
        
        if [[ "${blocking}" == "true" ]]; then
            OVERALL_STATUS="FAIL"
        fi
    elif [[ "${gate_pass}" == "true" ]]; then
        # Command succeeded AND validator approved
        final_verdict="PASS"
        ((GATES_PASSED++))
        info "Gate ${gate_id}: PASS"
        write_gate_status "${gate_id}" "PASS" "validator_approved"
    else
        # Command succeeded but validator rejected
        # Check if validation itself failed (valid=false) or gate just didn't pass
        if [[ "${valid}" == "false" ]]; then
            final_verdict="ERROR"
            ((GATES_ERROR++))
            error "Gate ${gate_id}: ERROR (validation failed)"
            error "  Check validation logs in: ${gate_evidence_dir}"
            write_gate_status "${gate_id}" "ERROR" "validation_failed"
        else
            # Evidence valid but gate didn't pass
            final_verdict="FAIL"
            ((GATES_FAILED++))
            error "Gate ${gate_id}: FAIL (verdict: ${verdict})"
            write_gate_status "${gate_id}" "FAIL" "verdict_${verdict}"
        fi
        
        if [[ "${blocking}" == "true" ]]; then
            OVERALL_STATUS="FAIL"
        fi
    fi
    
    GATE_VERDICTS["${gate_id}"]="${final_verdict}"
    
    # Update determinism counters
    case "${determinism_level}" in
        artifact)
            ((DETERMINISM_ARTIFACT++))
            ;;
        trace)
            ((DETERMINISM_TRACE++))
            ;;
        marker)
            ((DETERMINISM_MARKER++))
            ;;
        scheduling-independent)
            ((DETERMINISM_SCHEDULING_INDEPENDENT++))
            ;;
    esac
    
    return 0
}

# ============================================================================
# REPORT GENERATION
# ============================================================================

# Compute canonical evidence hash
compute_evidence_hash() {
    local manifest_path="$1"
    shift
    local gate_ids=("$@")
    
    verbose "Computing canonical evidence hash"
    
    # Sort gate IDs
    local sorted_gate_ids
    IFS=$'\n' sorted_gate_ids=($(sort <<<"${gate_ids[*]}"))
    unset IFS
    
    # CRITICAL: Compute canonical hash (not raw file hash)
    # Algorithm:
    # 1. For each evidence file: compute canonical JSON hash (excluding integrity.file_hash)
    # 2. Concatenate hashes in sorted gate_id order
    # 3. Compute final SHA256 of concatenated hashes
    
    local concatenated_hashes=""
    local evidence_files_array=()
    
    for gate_id in "${sorted_gate_ids[@]}"; do
        # Find latest attempt directory
        local gate_base_dir="${RUN_EVIDENCE_DIR}/gates/${gate_id}"
        
        if [[ ! -d "${gate_base_dir}" ]]; then
            continue
        fi
        
        # Find highest attempt number
        local max_attempt
        max_attempt=$(find "${gate_base_dir}" -maxdepth 1 -type d -name "attempt-*" 2>/dev/null | \
                      sed 's/.*attempt-//' | sort -n | tail -1)
        
        if [[ -z "${max_attempt}" ]]; then
            continue
        fi
        
        local evidence_file="${gate_base_dir}/attempt-${max_attempt}/report.json"
        
        if [[ -f "${evidence_file}" ]]; then
            # Compute canonical hash using Python (same as validator)
            local canonical_hash
            canonical_hash=$(python3 -c "
import json
import hashlib

with open('${evidence_file}', 'r') as f:
    evidence = json.load(f)

# Remove integrity.file_hash for canonical hash
if 'integrity' in evidence and 'file_hash' in evidence['integrity']:
    del evidence['integrity']['file_hash']

# Compute SHA256 of canonical JSON (sorted keys)
canonical_json = json.dumps(evidence, sort_keys=True, separators=(',', ':'))
canonical_hash = hashlib.sha256(canonical_json.encode('utf-8')).hexdigest()
print(canonical_hash)
" 2>/dev/null)
            
            if [[ -n "${canonical_hash}" ]]; then
                concatenated_hashes="${concatenated_hashes}${canonical_hash}"
                
                # CRITICAL: Store relative path from report directory (not project root)
                # Report is at: out/evidence/verification/${RUN_ID}/report.json
                # Evidence is at: out/evidence/verification/${RUN_ID}/gates/${gate_id}/attempt-N/report.json
                # Relative path from report dir: gates/${gate_id}/attempt-N/report.json
                local relative_path="gates/${gate_id}/attempt-${max_attempt}/report.json"
                evidence_files_array+=("${relative_path}")
            fi
        fi
    done
    
    # Compute final hash
    local final_hash
    if [[ -n "${concatenated_hashes}" ]]; then
        final_hash=$(echo -n "${concatenated_hashes}" | sha256sum | awk '{print $1}')
    else
        final_hash="0000000000000000000000000000000000000000000000000000000000000000"
    fi
    
    # Store evidence files array for report (global variable)
    declare -g EVIDENCE_FILES_JSON
    EVIDENCE_FILES_JSON=$(printf '%s\n' "${evidence_files_array[@]}" | python3 -c "
import sys
import json
files = [line.strip() for line in sys.stdin if line.strip()]
print(json.dumps(files))
")
    
    verbose "Evidence files array: ${EVIDENCE_FILES_JSON}"
    verbose "Evidence files count: ${#evidence_files_array[@]}"
    
    # Return both hash and files array (space-separated)
    echo "${final_hash}|${EVIDENCE_FILES_JSON}"
}

# Generate report JSON
generate_report() {
    local manifest_path="$1"
    shift
    local gate_ids=("$@")
    
    info "Generating verification report"
    
    local report_file="${RUN_EVIDENCE_DIR}/report.json"
    
    # Compute evidence hash and get files array
    local hash_and_files
    hash_and_files=$(compute_evidence_hash "${manifest_path}" "${gate_ids[@]}")
    
    local evidence_hash="${hash_and_files%%|*}"
    local evidence_files_json="${hash_and_files##*|}"
    
    # Build gates object
    local gates_json="{"
    local first=1
    
    for gate_id in "${gate_ids[@]}"; do
        local verdict="${GATE_VERDICTS[${gate_id}]:-SKIPPED}"
        local blocking="${GATE_BLOCKING[${gate_id}]:-false}"
        local determinism_level="${GATE_DETERMINISM_LEVELS[${gate_id}]:-}"
        
        # CRITICAL: Use actual evidence path (attempt-based, relative to report dir)
        local gate_base_dir="${RUN_EVIDENCE_DIR}/gates/${gate_id}"
        local evidence_path=""
        
        if [[ -d "${gate_base_dir}" ]]; then
            local max_attempt
            max_attempt=$(find "${gate_base_dir}" -maxdepth 1 -type d -name "attempt-*" 2>/dev/null | \
                          sed 's/.*attempt-//' | sort -n | tail -1)
            
            if [[ -n "${max_attempt}" ]]; then
                # Relative to report directory
                evidence_path="gates/${gate_id}/attempt-${max_attempt}/report.json"
            fi
        fi
        
        if [[ ${first} -eq 0 ]]; then
            gates_json="${gates_json},"
        fi
        first=0
        
        gates_json="${gates_json}
    \"${gate_id}\": {
      \"verdict\": \"${verdict}\",
      \"blocking\": ${blocking},
      \"determinism_level\": \"${determinism_level}\",
      \"evidence_path\": \"${evidence_path}\"
    }"
    done
    
    gates_json="${gates_json}
  }"
    
    verbose "Evidence hash: ${evidence_hash}"
    verbose "Evidence files JSON: ${evidence_files_json}"
    
    # Generate report JSON
    cat > "${report_file}" <<EOF
{
  "run_id": "${RUN_ID}",
  "timestamp": "$(date -u +"%Y-%m-%dT%H:%M:%SZ")",
  "status": "${OVERALL_STATUS}",
  "mode": "verification_layer",
  "mutation": false,
  "tier": "${TIER}",
  "gates_checked": ${GATES_CHECKED},
  "gates_passed": ${GATES_PASSED},
  "gates_failed": ${GATES_FAILED},
  "gates_skipped": ${GATES_SKIPPED},
  "gates_error": ${GATES_ERROR},
  "gates_timeout": ${GATES_TIMEOUT},
  "gates": ${gates_json},
  "determinism_summary": {
    "artifact": ${DETERMINISM_ARTIFACT},
    "trace": ${DETERMINISM_TRACE},
    "marker": ${DETERMINISM_MARKER},
    "scheduling-independent": ${DETERMINISM_SCHEDULING_INDEPENDENT}
  },
  "evidence_files": ${evidence_files_json},
  "evidence_hash": "${evidence_hash}"
}
EOF
    
    verbose "Report written to: ${report_file}"
    
    # Validate report
    if ! python3 "${VALIDATORS_DIR}/validate_report.py" "${report_file}"; then
        error "Report validation failed"
        return 1
    fi
    
    info "Report validation passed"
    echo "${report_file}"
    return 0
}

# ============================================================================
# SYMLINK MANAGEMENT
# ============================================================================

# Create or update latest symlink
update_latest_symlink() {
    local run_id="$1"
    
    verbose "Updating latest symlink"
    
    local latest_link="${EVIDENCE_BASE_DIR}/latest"
    local target="${run_id}"
    
    # Remove existing symlink if present
    if [[ -L "${latest_link}" ]]; then
        rm -f "${latest_link}"
    fi
    
    # Create new symlink
    if ! ln -s "${target}" "${latest_link}"; then
        error "Failed to create latest symlink"
        return 1
    fi
    
    verbose "Latest symlink updated: ${latest_link} -> ${target}"
    return 0
}

# ============================================================================
# MAIN EXECUTION
# ============================================================================

# Parse command-line arguments
parse_arguments() {
    while [[ $# -gt 0 ]]; do
        case "$1" in
            --tier)
                TIER="$2"
                shift 2
                ;;
            --mode)
                MODE="$2"
                shift 2
                ;;
            --manifest)
                MANIFEST_PATH="$2"
                shift 2
                ;;
            --verbose)
                VERBOSE=1
                shift
                ;;
            -h|--help)
                usage
                exit 0
                ;;
            *)
                error "Unknown argument: $1"
                usage
                exit 2
                ;;
        esac
    done
    
    # Validate arguments
    case "${TIER}" in
        fast|standard|heavy)
            ;;
        *)
            error "Invalid tier: ${TIER}"
            error "Valid tiers: fast, standard, heavy"
            exit 2
            ;;
    esac
    
    case "${MODE}" in
        shadow|hard_gate)
            ;;
        *)
            error "Invalid mode: ${MODE}"
            error "Valid modes: shadow, hard_gate"
            exit 2
            ;;
    esac
    
    if [[ ! -f "${MANIFEST_PATH}" ]]; then
        error "Manifest file not found: ${MANIFEST_PATH}"
        exit 2
    fi
}

# Main function
main() {
    info "AykenOS Verification Layer - Starting"
    info "Tier: ${TIER}"
    info "Mode: ${MODE}"
    info "Manifest: ${MANIFEST_PATH}"
    
    # Generate run ID
    RUN_ID=$(generate_run_id)
    info "Run ID: ${RUN_ID}"
    
    # Setup evidence directory
    if ! setup_evidence_directory "${RUN_ID}"; then
        error "Failed to setup evidence directory"
        exit 2
    fi
    
    # Validate manifest
    if ! validate_manifest "${MANIFEST_PATH}"; then
        error "Manifest validation failed"
        exit 2
    fi
    
    # Parse manifest and get gate IDs
    local all_gate_ids
    all_gate_ids=$(parse_manifest "${MANIFEST_PATH}")
    
    if [[ -z "${all_gate_ids}" ]]; then
        error "No gates found in manifest"
        exit 2
    fi
    
    # Convert to array
    local gate_ids_array=()
    while IFS= read -r gate_id; do
        gate_ids_array+=("${gate_id}")
    done <<< "${all_gate_ids}"
    
    # Perform topological sort
    local sorted_gate_ids
    sorted_gate_ids=$(topological_sort "${MANIFEST_PATH}")
    
    if [[ $? -ne 0 ]]; then
        error "Dependency resolution failed"
        exit 2
    fi
    
    # Convert to array
    local sorted_gates_array=()
    while IFS= read -r gate_id; do
        sorted_gates_array+=("${gate_id}")
    done <<< "${sorted_gate_ids}"
    
    # Filter by tier
    local filtered_gates
    filtered_gates=$(filter_gates_by_tier "${MANIFEST_PATH}" "${TIER}" "${sorted_gates_array[@]}")
    
    if [[ -z "${filtered_gates}" ]]; then
        warn "No gates to execute after tier filtering"
    fi
    
    # Convert to array
    local filtered_gates_array=()
    if [[ -n "${filtered_gates}" ]]; then
        for gate_id in ${filtered_gates}; do
            filtered_gates_array+=("${gate_id}")
        done
    fi
    
    GATES_CHECKED=${#filtered_gates_array[@]}
    info "Gates to execute: ${GATES_CHECKED}"
    
    # Execute gates sequentially
    for gate_id in "${filtered_gates_array[@]}"; do
        # CRITICAL: Never let gate execution abort the orchestrator
        # All gate failures are captured in GATE_VERDICTS
        execute_gate "${gate_id}" "${MANIFEST_PATH}" || true
    done
    
    # Generate report
    local report_file
    report_file=$(generate_report "${MANIFEST_PATH}" "${filtered_gates_array[@]}")
    
    if [[ $? -ne 0 ]]; then
        error "Report generation failed"
        exit 2
    fi
    
    # Update latest symlink
    if ! update_latest_symlink "${RUN_ID}"; then
        warn "Failed to update latest symlink"
    fi
    
    # Display summary
    echo ""
    echo "========================================"
    echo "Verification Summary"
    echo "========================================"
    echo "Run ID: ${RUN_ID}"
    echo "Status: ${OVERALL_STATUS}"
    echo "Mode: ${MODE}"
    echo "Tier: ${TIER}"
    echo ""
    echo "Gates Checked: ${GATES_CHECKED}"
    echo "Gates Passed: ${GATES_PASSED}"
    echo "Gates Failed: ${GATES_FAILED}"
    echo "Gates Skipped: ${GATES_SKIPPED}"
    echo "Gates Error: ${GATES_ERROR}"
    echo "Gates Timeout: ${GATES_TIMEOUT}"
    echo ""
    echo "Report: ${report_file}"
    echo "========================================"
    
    # Exit with appropriate status code
    if [[ "${MODE}" == "shadow" ]]; then
        info "Shadow mode: exiting with status 0 regardless of result"
        exit 0
    elif [[ "${OVERALL_STATUS}" == "PASS" ]]; then
        info "Verification PASSED"
        exit 0
    else
        error "Verification FAILED"
        exit 1
    fi
}

# ============================================================================
# ENTRY POINT
# ============================================================================

# Parse arguments and run main
parse_arguments "$@"
main
