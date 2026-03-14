#!/usr/bin/env bash
# AykenOS QEMU Environment Validator
# Author: Kenan AY
# Purpose: Comprehensive QEMU environment validation for Phase 1.5
# Task: 1.5.1.3 - QEMU environment validation

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
source "${ROOT}/tools/lib/ayken_path_contract.sh"
cd "${ROOT}"
ayken_prepare_out_dirs

# Default parameters
VERBOSE=false
SAVE_LOGS=false
TIMEOUT=30
SKIP_BUILD=false

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
GRAY='\033[0;37m'
NC='\033[0m'

# Test configuration
QEMU_EXECUTABLE="qemu-system-x86_64"
REQUIRED_FILES=("${AYKEN_KERNEL_ELF}" "${AYKEN_BOOT_EFI}")
EFI_IMAGE="${EFI_IMG:-${AYKEN_EFI_IMG}}"
TEST_TIMEOUT=$TIMEOUT
MAKE_RUN_OUTPUT_LOG="${AYKEN_LOG_DIR}/make_run_output.log"
MAKE_RUN_ERROR_LOG="${AYKEN_LOG_DIR}/make_run_error.log"
BOOT_TEST_OUTPUT_LOG="${AYKEN_LOG_DIR}/boot_test_output.log"
BOOT_TEST_ERROR_LOG="${AYKEN_LOG_DIR}/boot_test_error.log"

# Success patterns
SUCCESS_PATTERNS=(
    "AykenOS.*INIT"
    "Kernel.*init.*done"
    "kmain.*starting"
    "EARLY INIT.*done"
    "Scheduler.*ready"
)

# Error patterns
ERROR_PATTERNS=(
    "PANIC"
    "ERROR"
    "FATAL"
    "Triple fault"
    "General Protection Fault"
)

# Boot patterns
BOOT_PATTERNS=(
    "Booting.*AykenOS"
    "EFI.*loader"
    "Kernel.*loaded"
)

# Validation report
declare -A VALIDATION_REPORT
VALIDATION_REPORT[qemu_installation]=false
VALIDATION_REPORT[qemu_version]=""
VALIDATION_REPORT[build_artifacts]=false
VALIDATION_REPORT[efi_image_creation]=false
VALIDATION_REPORT[make_run_automation]=false
VALIDATION_REPORT[log_parsing]=false
VALIDATION_REPORT[boot_capability]=false
VALIDATION_REPORT[success_failure_detection]=false
VALIDATION_REPORT[overall_success]=false

declare -A VALIDATION_DETAILS

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --verbose)
            VERBOSE=true
            shift
            ;;
        --save-logs)
            SAVE_LOGS=true
            shift
            ;;
        --timeout)
            TIMEOUT="$2"
            TEST_TIMEOUT="$2"
            shift 2
            ;;
        --skip-build)
            SKIP_BUILD=true
            shift
            ;;
        --help)
            echo "AykenOS QEMU Environment Validator"
            echo ""
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --verbose      Enable verbose output"
            echo "  --save-logs    Save log files after tests"
            echo "  --timeout N    Set timeout in seconds (default: 30)"
            echo "  --skip-build   Skip automatic build attempts"
            echo "  --help         Show this help message"
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            exit 1
            ;;
    esac
done

write_validation_log() {
    local message="$1"
    local level="${2:-INFO}"
    local timestamp=$(date '+%H:%M:%S.%3N')
    
    case "$level" in
        "SUCCESS") echo -e "[$timestamp] [$level] ${GREEN}$message${NC}" ;;
        "ERROR")   echo -e "[$timestamp] [$level] ${RED}$message${NC}" ;;
        "WARNING") echo -e "[$timestamp] [$level] ${YELLOW}$message${NC}" ;;
        "INFO")    echo -e "[$timestamp] [$level] ${CYAN}$message${NC}" ;;
        "DEBUG")   [[ "$VERBOSE" == "true" ]] && echo -e "[$timestamp] [$level] ${GRAY}$message${NC}" ;;
        *)         echo -e "[$timestamp] [$level] $message" ;;
    esac
}

test_qemu_installation() {
    write_validation_log "=== TESTING QEMU INSTALLATION ===" "INFO"
    
    # Check if QEMU executable exists
    if ! command -v "$QEMU_EXECUTABLE" >/dev/null 2>&1; then
        write_validation_log "QEMU not found in PATH" "ERROR"
        VALIDATION_DETAILS[qemu_installation]="QEMU executable not found in PATH"
        return 1
    fi
    
    # Get QEMU version
    local version_output
    if version_output=$($QEMU_EXECUTABLE --version 2>&1); then
        VALIDATION_REPORT[qemu_version]=$(echo "$version_output" | head -n1 | tr -d '\n')
        write_validation_log "QEMU found: ${VALIDATION_REPORT[qemu_version]}" "SUCCESS"
        
        # Test basic QEMU functionality with help command
        if $QEMU_EXECUTABLE --help >/dev/null 2>&1; then
            write_validation_log "QEMU help command works correctly" "SUCCESS"
            VALIDATION_REPORT[qemu_installation]=true
            VALIDATION_DETAILS[qemu_installation]="QEMU installation verified: ${VALIDATION_REPORT[qemu_version]}"
            return 0
        else
            write_validation_log "QEMU help command failed" "ERROR"
            VALIDATION_DETAILS[qemu_installation]="QEMU help command failed"
            return 1
        fi
    else
        write_validation_log "QEMU version check failed" "ERROR"
        VALIDATION_DETAILS[qemu_installation]="QEMU version check failed"
        return 1
    fi
}

test_build_artifacts() {
    write_validation_log "=== TESTING BUILD ARTIFACTS ===" "INFO"
    
    local missing_files=()
    
    for file in "${REQUIRED_FILES[@]}"; do
        if [[ ! -f "$file" ]]; then
            missing_files+=("$file")
            write_validation_log "Missing required file: $file" "WARNING"
        else
            write_validation_log "Found required file: $file" "SUCCESS"
        fi
    done
    
    if [[ ${#missing_files[@]} -gt 0 ]] && [[ "$SKIP_BUILD" != "true" ]]; then
        write_validation_log "Attempting to build missing artifacts..." "INFO"
        
        if command -v make >/dev/null 2>&1; then
            if make all >/dev/null 2>&1; then
                write_validation_log "Build completed successfully" "SUCCESS"
                
                # Re-check missing files
                local still_missing=()
                for file in "${missing_files[@]}"; do
                    if [[ ! -f "$file" ]]; then
                        still_missing+=("$file")
                    fi
                done
                
                if [[ ${#still_missing[@]} -eq 0 ]]; then
                    write_validation_log "All required files now present after build" "SUCCESS"
                    VALIDATION_REPORT[build_artifacts]=true
                    VALIDATION_DETAILS[build_artifacts]="Build successful, all artifacts present"
                    return 0
                else
                    write_validation_log "Build completed but some files still missing: ${still_missing[*]}" "ERROR"
                    VALIDATION_DETAILS[build_artifacts]="Build completed but missing: ${still_missing[*]}"
                    return 1
                fi
            else
                write_validation_log "Build failed" "ERROR"
                VALIDATION_DETAILS[build_artifacts]="Build failed"
                return 1
            fi
        else
            write_validation_log "Make command not available" "ERROR"
            VALIDATION_DETAILS[build_artifacts]="Make command not available"
            return 1
        fi
    elif [[ ${#missing_files[@]} -gt 0 ]]; then
        write_validation_log "Missing files and build skipped: ${missing_files[*]}" "ERROR"
        VALIDATION_DETAILS[build_artifacts]="Missing files, build skipped: ${missing_files[*]}"
        return 1
    else
        write_validation_log "All required build artifacts present" "SUCCESS"
        VALIDATION_REPORT[build_artifacts]=true
        VALIDATION_DETAILS[build_artifacts]="All required artifacts present"
        return 0
    fi
}

test_efi_image_creation() {
    write_validation_log "=== TESTING EFI IMAGE CREATION ===" "INFO"
    
    # Remove existing EFI image if present
    if [[ -f "$EFI_IMAGE" ]]; then
        rm -f "$EFI_IMAGE"
        write_validation_log "Removed existing EFI image" "DEBUG"
    fi
    
    # Try to create EFI image using make
    write_validation_log "Creating EFI image using make..." "INFO"
    
    if command -v make >/dev/null 2>&1; then
        if make efi-img >/dev/null 2>&1 && [[ -f "$EFI_IMAGE" ]]; then
            local image_size=$(stat -c%s "$EFI_IMAGE" 2>/dev/null || echo 0)
            local image_size_mb=$((image_size / 1024 / 1024))
            write_validation_log "EFI image created successfully (size: ${image_size_mb} MB)" "SUCCESS"
            VALIDATION_REPORT[efi_image_creation]=true
            VALIDATION_DETAILS[efi_image_creation]="EFI image created successfully via make"
            return 0
        else
            write_validation_log "Make efi-img failed, trying shell script..." "WARNING"
            
            # Try shell script as fallback
            if [[ -x "tools/build/make_efi_img.sh" ]]; then
                if ./tools/build/make_efi_img.sh >/dev/null 2>&1 && [[ -f "$EFI_IMAGE" ]]; then
                    local image_size=$(stat -c%s "$EFI_IMAGE" 2>/dev/null || echo 0)
                    local image_size_mb=$((image_size / 1024 / 1024))
                    write_validation_log "EFI image created via shell script (size: ${image_size_mb} MB)" "SUCCESS"
                    VALIDATION_REPORT[efi_image_creation]=true
                    VALIDATION_DETAILS[efi_image_creation]="EFI image created via shell script"
                    return 0
                else
                    write_validation_log "Shell script failed to create EFI image" "ERROR"
                    VALIDATION_DETAILS[efi_image_creation]="Both make and shell script failed"
                    return 1
                fi
            else
                write_validation_log "No shell script available for EFI image creation" "ERROR"
                VALIDATION_DETAILS[efi_image_creation]="Make failed and no shell script available"
                return 1
            fi
        fi
    else
        write_validation_log "Make command not available" "ERROR"
        VALIDATION_DETAILS[efi_image_creation]="Make command not available"
        return 1
    fi
}

test_make_run_automation() {
    write_validation_log "=== TESTING MAKE RUN AUTOMATION ===" "INFO"
    
    if [[ ! -f "$EFI_IMAGE" ]]; then
        write_validation_log "EFI image not available for make run test" "ERROR"
        VALIDATION_DETAILS[make_run_automation]="EFI image not available"
        return 1
    fi
    
    write_validation_log "Testing make run command (timeout: ${TEST_TIMEOUT}s)..." "INFO"
    
    # Start make run process in background
    local output_log="${MAKE_RUN_OUTPUT_LOG}"
    local error_log="${MAKE_RUN_ERROR_LOG}"
    
    # Clean old logs
    rm -f "$output_log" "$error_log"
    
    if command -v make >/dev/null 2>&1; then
        # Start make run in background
        timeout "${TEST_TIMEOUT}s" make run > "$output_log" 2> "$error_log" &
        local make_pid=$!
        
        write_validation_log "Make run process started (PID: $make_pid)" "DEBUG"
        
        # Wait for process to complete or timeout
        wait $make_pid 2>/dev/null || true
        
        # Check if make run executed properly
        if [[ -f "$output_log" ]]; then
            local output=$(cat "$output_log" 2>/dev/null || echo "")
            if [[ "$output" =~ qemu-system-x86_64|QEMU ]]; then
                write_validation_log "Make run automation works - QEMU was invoked" "SUCCESS"
                VALIDATION_REPORT[make_run_automation]=true
                VALIDATION_DETAILS[make_run_automation]="Make run successfully invokes QEMU"
                
                # Clean up logs if not saving
                if [[ "$SAVE_LOGS" != "true" ]]; then
                    rm -f "$output_log" "$error_log"
                fi
                return 0
            else
                write_validation_log "Make run did not invoke QEMU properly" "ERROR"
                VALIDATION_DETAILS[make_run_automation]="Make run did not invoke QEMU"
                return 1
            fi
        else
            write_validation_log "Make run produced no output" "ERROR"
            VALIDATION_DETAILS[make_run_automation]="Make run produced no output"
            return 1
        fi
    else
        write_validation_log "Make command not available" "ERROR"
        VALIDATION_DETAILS[make_run_automation]="Make command not available"
        return 1
    fi
}

test_log_parsing() {
    write_validation_log "=== TESTING QEMU LOG PARSING ===" "INFO"
    
    # Create test log content with known patterns
    local test_log_file="test_log_parsing.log"
    cat > "$test_log_file" << 'EOF'
[00:00:01.234] AykenOS INIT starting...
[00:00:01.456] Kernel init done
[00:00:01.678] kmain starting
[00:00:01.890] EARLY INIT done
[00:00:02.123] Scheduler ready
[00:00:02.345] Some other message
[00:00:02.567] ERROR: Test error message
[00:00:02.789] PANIC: Test panic message
EOF
    
    write_validation_log "Testing success pattern detection..." "DEBUG"
    
    local success_count=0
    local error_count=0
    
    # Test success patterns
    for pattern in "${SUCCESS_PATTERNS[@]}"; do
        if grep -qE "$pattern" "$test_log_file"; then
            ((success_count++))
            write_validation_log "Success pattern detected: $pattern" "DEBUG"
        fi
    done
    
    # Test error patterns
    for pattern in "${ERROR_PATTERNS[@]}"; do
        if grep -qE "$pattern" "$test_log_file"; then
            ((error_count++))
            write_validation_log "Error pattern detected: $pattern" "DEBUG"
        fi
    done
    
    local expected_success_count=${#SUCCESS_PATTERNS[@]}
    local expected_error_count=2  # We have ERROR and PANIC in test log
    
    if [[ $success_count -eq $expected_success_count ]] && [[ $error_count -eq $expected_error_count ]]; then
        write_validation_log "Log parsing works correctly (Success: $success_count/$expected_success_count, Errors: $error_count/$expected_error_count)" "SUCCESS"
        VALIDATION_REPORT[log_parsing]=true
        VALIDATION_DETAILS[log_parsing]="Log parsing patterns work correctly"
        
        # Clean up test log
        rm -f "$test_log_file"
        return 0
    else
        write_validation_log "Log parsing failed (Success: $success_count/$expected_success_count, Errors: $error_count/$expected_error_count)" "ERROR"
        VALIDATION_DETAILS[log_parsing]="Log parsing pattern detection failed"
        return 1
    fi
}

test_boot_capability() {
    write_validation_log "=== TESTING QEMU BOOT CAPABILITY ===" "INFO"
    
    if [[ ! -f "$EFI_IMAGE" ]]; then
        write_validation_log "EFI image not available for boot test" "ERROR"
        VALIDATION_DETAILS[boot_capability]="EFI image not available"
        return 1
    fi
    
    local output_log="${BOOT_TEST_OUTPUT_LOG}"
    local error_log="${BOOT_TEST_ERROR_LOG}"
    
    # Clean old logs
    rm -f "$output_log" "$error_log"
    
    # QEMU arguments for boot test
    local qemu_args=(
        "-drive" "format=raw,file=$EFI_IMAGE"
        "-serial" "stdio"
        "-m" "256M"
        "-no-reboot"
        "-no-shutdown"
        "-display" "none"
    )
    
    write_validation_log "Starting QEMU boot test (timeout: ${TEST_TIMEOUT}s)..." "INFO"
    
    # Start QEMU process in background
    "$QEMU_EXECUTABLE" "${qemu_args[@]}" > "$output_log" 2> "$error_log" &
    local qemu_pid=$!
    
    write_validation_log "QEMU boot process started (PID: $qemu_pid)" "DEBUG"
    
    # Monitor boot process
    local start_time=$(date +%s)
    local last_output_size=0
    local boot_stages=()
    local error_detected=false
    
    while kill -0 $qemu_pid 2>/dev/null; do
        local current_time=$(date +%s)
        if (( current_time - start_time > TEST_TIMEOUT )); then
            write_validation_log "Test timeout reached" "DEBUG"
            break
        fi
        
        # Analyze output
        if [[ -f "$output_log" ]]; then
            local current_size=$(wc -c < "$output_log" 2>/dev/null || echo 0)
            if (( current_size > last_output_size )); then
                local new_content=$(tail -c +$((last_output_size + 1)) "$output_log" 2>/dev/null || echo "")
                last_output_size=$current_size
                
                # Check for boot patterns
                for pattern in "${BOOT_PATTERNS[@]}"; do
                    if echo "$new_content" | grep -qE "$pattern"; then
                        local match=$(echo "$new_content" | grep -oE "$pattern" | head -n1)
                        write_validation_log "Boot stage detected: $match" "SUCCESS"
                        boot_stages+=("$match")
                    fi
                done
                
                # Check for success patterns
                for pattern in "${SUCCESS_PATTERNS[@]}"; do
                    if echo "$new_content" | grep -qE "$pattern"; then
                        local match=$(echo "$new_content" | grep -oE "$pattern" | head -n1)
                        write_validation_log "Success pattern detected: $match" "SUCCESS"
                        boot_stages+=("$match")
                    fi
                done
                
                # Check for error patterns
                for pattern in "${ERROR_PATTERNS[@]}"; do
                    if echo "$new_content" | grep -qE "$pattern"; then
                        local match=$(echo "$new_content" | grep -oE "$pattern" | head -n1)
                        write_validation_log "Error detected during boot: $match" "ERROR"
                        error_detected=true
                        break
                    fi
                done
                
                if [[ "$VERBOSE" == "true" ]]; then
                    echo "$new_content" | while IFS= read -r line; do
                        if [[ -n "${line// }" ]]; then
                            echo -e "  ${GRAY}QEMU: $line${NC}"
                        fi
                    done
                fi
            fi
        fi
        
        # Early exit on error
        if [[ "$error_detected" == "true" ]]; then
            break
        fi
        
        sleep 0.5
    done
    
    # Cleanup QEMU process
    if kill -0 $qemu_pid 2>/dev/null; then
        write_validation_log "Terminating QEMU process..." "DEBUG"
        kill $qemu_pid 2>/dev/null || true
        wait $qemu_pid 2>/dev/null || true
    fi
    
    local duration=$(($(date +%s) - start_time))
    
    # Evaluate boot success
    local boot_success=false
    if [[ ${#boot_stages[@]} -gt 0 ]] && [[ "$error_detected" != "true" ]]; then
        boot_success=true
    fi
    
    if [[ "$boot_success" == "true" ]]; then
        write_validation_log "QEMU boot capability verified (${duration}s, ${#boot_stages[@]} stages detected)" "SUCCESS"
        VALIDATION_REPORT[boot_capability]=true
        VALIDATION_DETAILS[boot_capability]="Boot successful, ${#boot_stages[@]} stages detected in ${duration}s"
    else
        write_validation_log "QEMU boot capability failed (${duration}s, errors: $error_detected)" "ERROR"
        VALIDATION_DETAILS[boot_capability]="Boot failed, errors detected: $error_detected"
    fi
    
    # Clean up logs if not saving
    if [[ "$SAVE_LOGS" != "true" ]]; then
        rm -f "$output_log" "$error_log"
    fi
    
    return $([ "$boot_success" == "true" ] && echo 0 || echo 1)
}

test_success_failure_detection() {
    write_validation_log "=== TESTING SUCCESS/FAILURE DETECTION ===" "INFO"
    
    # Test 1: Success detection with mock successful output
    local success_test_log="success_detection_test.log"
    cat > "$success_test_log" << 'EOF'
AykenOS INIT starting
Kernel init done
EARLY INIT done
Scheduler ready
EOF
    
    local success_detected=false
    for pattern in "${SUCCESS_PATTERNS[@]}"; do
        if grep -qE "$pattern" "$success_test_log"; then
            success_detected=true
            break
        fi
    done
    
    # Test 2: Failure detection with mock error output
    local failure_test_log="failure_detection_test.log"
    cat > "$failure_test_log" << 'EOF'
Starting system...
Loading kernel...
PANIC: Memory allocation failed
System halted
EOF
    
    local failure_detected=false
    for pattern in "${ERROR_PATTERNS[@]}"; do
        if grep -qE "$pattern" "$failure_test_log"; then
            failure_detected=true
            break
        fi
    done
    
    # Clean up test logs
    rm -f "$success_test_log" "$failure_test_log"
    
    if [[ "$success_detected" == "true" ]] && [[ "$failure_detected" == "true" ]]; then
        write_validation_log "Success/failure detection works correctly" "SUCCESS"
        VALIDATION_REPORT[success_failure_detection]=true
        VALIDATION_DETAILS[success_failure_detection]="Both success and failure patterns detected correctly"
        return 0
    else
        write_validation_log "Success/failure detection failed (Success: $success_detected, Failure: $failure_detected)" "ERROR"
        VALIDATION_DETAILS[success_failure_detection]="Pattern detection failed"
        return 1
    fi
}

generate_validation_report() {
    write_validation_log "Generating comprehensive validation report..." "INFO"
    
    # Calculate overall success
    local overall_success=true
    for key in qemu_installation build_artifacts efi_image_creation make_run_automation log_parsing boot_capability success_failure_detection; do
        if [[ "${VALIDATION_REPORT[$key]}" != "true" ]]; then
            overall_success=false
            break
        fi
    done
    VALIDATION_REPORT[overall_success]=$overall_success
    
    local report_file="qemu_environment_validation_report.md"
    local timestamp=$(date '+%Y-%m-%d %H:%M:%S')
    
    cat > "$report_file" << EOF
# AykenOS QEMU Environment Validation Report

**Generated:** $timestamp  
**Task:** 1.5.1.3 - QEMU environment validation  
**Overall Status:** $([ "$overall_success" == "true" ] && echo "✅ PASS" || echo "❌ FAIL")

## Validation Summary

| Test Component | Status | Details |
|----------------|--------|---------|
| QEMU Installation | $([ "${VALIDATION_REPORT[qemu_installation]}" == "true" ] && echo "✅ PASS" || echo "❌ FAIL") | ${VALIDATION_DETAILS[qemu_installation]} |
| Build Artifacts | $([ "${VALIDATION_REPORT[build_artifacts]}" == "true" ] && echo "✅ PASS" || echo "❌ FAIL") | ${VALIDATION_DETAILS[build_artifacts]} |
| EFI Image Creation | $([ "${VALIDATION_REPORT[efi_image_creation]}" == "true" ] && echo "✅ PASS" || echo "❌ FAIL") | ${VALIDATION_DETAILS[efi_image_creation]} |
| Make Run Automation | $([ "${VALIDATION_REPORT[make_run_automation]}" == "true" ] && echo "✅ PASS" || echo "❌ FAIL") | ${VALIDATION_DETAILS[make_run_automation]} |
| Log Parsing | $([ "${VALIDATION_REPORT[log_parsing]}" == "true" ] && echo "✅ PASS" || echo "❌ FAIL") | ${VALIDATION_DETAILS[log_parsing]} |
| Boot Capability | $([ "${VALIDATION_REPORT[boot_capability]}" == "true" ] && echo "✅ PASS" || echo "❌ FAIL") | ${VALIDATION_DETAILS[boot_capability]} |
| Success/Failure Detection | $([ "${VALIDATION_REPORT[success_failure_detection]}" == "true" ] && echo "✅ PASS" || echo "❌ FAIL") | ${VALIDATION_DETAILS[success_failure_detection]} |

## QEMU Configuration

- **QEMU Version:** ${VALIDATION_REPORT[qemu_version]}
- **Test Timeout:** $TEST_TIMEOUT seconds
- **EFI Image:** $EFI_IMAGE
- **Required Files:** ${REQUIRED_FILES[*]}

## Requirements Validation

This validation addresses the following task requirements:

✅ **Validate QEMU installation and boot capability**
- QEMU installation verified: $([ "${VALIDATION_REPORT[qemu_installation]}" == "true" ] && echo "PASS" || echo "FAIL")
- Boot capability tested: $([ "${VALIDATION_REPORT[boot_capability]}" == "true" ] && echo "PASS" || echo "FAIL")

✅ **Test make run automation with success/failure detection**
- Make run automation: $([ "${VALIDATION_REPORT[make_run_automation]}" == "true" ] && echo "PASS" || echo "FAIL")
- Success/failure detection: $([ "${VALIDATION_REPORT[success_failure_detection]}" == "true" ] && echo "PASS" || echo "FAIL")

✅ **Ensure QEMU log parsing works correctly**
- Log parsing patterns: $([ "${VALIDATION_REPORT[log_parsing]}" == "true" ] && echo "PASS" || echo "FAIL")

## Next Steps

$(if [ "$overall_success" == "true" ]; then
    echo "🎉 **QEMU environment validation completed successfully!**

The QEMU environment is properly configured and ready for:
- Phase 1.5 Ring3 validation testing
- Automated boot testing and validation
- Reliable QEMU-based development workflow

**Phase 1.5 can proceed to task 1.5.2.1 - Ring3 test process creation.**"
else
    echo "⚠️ **Action Required:** QEMU environment validation failed.

Failed components need to be addressed before proceeding.

**Phase 1.5 is blocked until QEMU environment issues are resolved.**"
fi)

---
*Report generated by AykenOS QEMU Environment Validator*  
*Task: 1.5.1.3 - QEMU environment validation*
EOF

    write_validation_log "Validation report saved to: $report_file" "SUCCESS"
    
    # Display summary to console
    echo ""
    echo -e "${CYAN}============================================================${NC}"
    echo -e "${CYAN}QEMU ENVIRONMENT VALIDATION SUMMARY${NC}"
    echo -e "${CYAN}============================================================${NC}"
    echo ""
    echo -e "${NC}Overall Status: $([ "$overall_success" == "true" ] && echo -e "${GREEN}✅ PASS${NC}" || echo -e "${RED}❌ FAIL${NC}")${NC}"
    echo -e "${NC}QEMU Version: ${VALIDATION_REPORT[qemu_version]}${NC}"
    echo ""
    
    echo -e "${NC}Component Results:${NC}"
    echo -e "  QEMU Installation: $([ "${VALIDATION_REPORT[qemu_installation]}" == "true" ] && echo -e "${GREEN}✅ PASS${NC}" || echo -e "${RED}❌ FAIL${NC}")"
    echo -e "  Build Artifacts: $([ "${VALIDATION_REPORT[build_artifacts]}" == "true" ] && echo -e "${GREEN}✅ PASS${NC}" || echo -e "${RED}❌ FAIL${NC}")"
    echo -e "  EFI Image Creation: $([ "${VALIDATION_REPORT[efi_image_creation]}" == "true" ] && echo -e "${GREEN}✅ PASS${NC}" || echo -e "${RED}❌ FAIL${NC}")"
    echo -e "  Make Run Automation: $([ "${VALIDATION_REPORT[make_run_automation]}" == "true" ] && echo -e "${GREEN}✅ PASS${NC}" || echo -e "${RED}❌ FAIL${NC}")"
    echo -e "  Log Parsing: $([ "${VALIDATION_REPORT[log_parsing]}" == "true" ] && echo -e "${GREEN}✅ PASS${NC}" || echo -e "${RED}❌ FAIL${NC}")"
    echo -e "  Boot Capability: $([ "${VALIDATION_REPORT[boot_capability]}" == "true" ] && echo -e "${GREEN}✅ PASS${NC}" || echo -e "${RED}❌ FAIL${NC}")"
    echo -e "  Success/Failure Detection: $([ "${VALIDATION_REPORT[success_failure_detection]}" == "true" ] && echo -e "${GREEN}✅ PASS${NC}" || echo -e "${RED}❌ FAIL${NC}")"
    echo ""
    echo -e "${CYAN}Detailed report: $report_file${NC}"
    echo -e "${CYAN}============================================================${NC}"
}

# Main execution
main() {
    echo -e "${GREEN}AykenOS QEMU Environment Validator${NC}"
    echo -e "${GRAY}Author: Kenan AY${NC}"
    echo -e "${GRAY}Task: 1.5.1.3 - QEMU environment validation${NC}"
    echo ""
    
    write_validation_log "Starting QEMU environment validation..." "INFO"
    
    # Run all validation tests
    local tests=(
        "test_qemu_installation"
        "test_build_artifacts"
        "test_efi_image_creation"
        "test_make_run_automation"
        "test_log_parsing"
        "test_boot_capability"
        "test_success_failure_detection"
    )
    
    local passed_tests=0
    local total_tests=${#tests[@]}
    
    for test_func in "${tests[@]}"; do
        local test_name=$(echo "$test_func" | sed 's/test_//; s/_/ /g')
        write_validation_log "Running test: $test_name" "INFO"
        
        if $test_func; then
            ((passed_tests++))
            write_validation_log "$test_name: PASS" "SUCCESS"
        else
            write_validation_log "$test_name: FAIL" "ERROR"
        fi
        echo ""
    done
    
    write_validation_log "Validation completed: $passed_tests/$total_tests tests passed" "INFO"
    
    # Generate comprehensive report
    generate_validation_report
    
    # Exit with appropriate code
    if [[ "${VALIDATION_REPORT[overall_success]}" == "true" ]]; then
        write_validation_log "QEMU environment validation successful!" "SUCCESS"
        exit 0
    else
        write_validation_log "QEMU environment validation failed!" "ERROR"
        exit 1
    fi
}

# Run main function
main "$@"
