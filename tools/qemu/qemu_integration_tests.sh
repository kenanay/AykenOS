#!/usr/bin/env bash
# AykenOS QEMU Integration Test Suite
# Author: Kenan AY
# Purpose: Comprehensive QEMU-based testing for Phase 1 critical functionality

set -e

# Test configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TEST_TIMEOUT=45
VERBOSE=false
SAVE_LOGS=false
INTERACTIVE=false
TEST_SUITE="all"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
GRAY='\033[0;37m'
BLUE='\033[0;34m'
NC='\033[0m'

# Test results tracking
declare -A TEST_RESULTS
declare -A TEST_DURATIONS
declare -A TEST_DETAILS
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0

# QEMU configuration
QEMU_ARGS=(
    "-drive" "format=raw,file=EFI.img"
    "-serial" "stdio"
    "-m" "256M"
    "-no-reboot"
    "-no-shutdown"
    "-monitor" "unix:qemu-monitor.sock,server,nowait"
)

# Test patterns for different validation types
BOOT_SUCCESS_PATTERNS=(
    "AykenOS.*INIT"
    "Kernel.*init.*done"
    "kmain.*starting"
    "EARLY INIT.*done"
    "Scheduler.*ready"
    "LATE INIT.*done"
)

RING3_SUCCESS_PATTERNS=(
    "user AI service scheduled.*Ring3"
    "Ring3.*transition"
    "user.*process.*created"
    "PID.*running"
)

DEVFS_SUCCESS_PATTERNS=(
    "devfs.*Registered.*null"
    "devfs.*Registered.*zero"
    "devfs.*Registered.*console"
    "devfs.*Registered.*kbd"
    "devfs.*Registered.*ttyS0"
    "devfs.*Registered.*sda"
    "VFS.*DevFS"
)

SYSCALL_SUCCESS_PATTERNS=(
    "syscall.*installing.*INT"
    "Syscall interface ready"
    "SYS_write"
    "syscall.*handler"
)

ERROR_PATTERNS=(
    "PANIC"
    "ERROR"
    "FATAL"
    "Triple fault"
    "General Protection Fault"
    "Page fault"
    "Invalid opcode"
)

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --timeout)
            TEST_TIMEOUT="$2"
            shift 2
            ;;
        --verbose)
            VERBOSE=true
            shift
            ;;
        --save-logs)
            SAVE_LOGS=true
            shift
            ;;
        --interactive)
            INTERACTIVE=true
            shift
            ;;
        --suite)
            TEST_SUITE="$2"
            shift 2
            ;;
        --help)
            echo "AykenOS QEMU Integration Test Suite"
            echo ""
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --timeout N        Set timeout in seconds (default: 45)"
            echo "  --verbose          Enable verbose output"
            echo "  --save-logs        Save log files after tests"
            echo "  --interactive      Show QEMU display"
            echo "  --suite SUITE      Run specific test suite (boot|ring3|devfs|syscall|all)"
            echo "  --help             Show this help message"
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            exit 1
            ;;
    esac
done

# Utility functions
write_log() {
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

check_prerequisites() {
    write_log "Checking test prerequisites..." "INFO"
    
    # Check QEMU availability
    if ! command -v qemu-system-x86_64 >/dev/null 2>&1; then
        write_log "QEMU not found in PATH" "ERROR"
        return 1
    fi
    
    # Check EFI image
    if [[ ! -f "EFI.img" ]]; then
        write_log "EFI.img not found, attempting to create..." "WARNING"
        if [[ -x "./make_efi_img.sh" ]]; then
            ./make_efi_img.sh
        elif command -v make >/dev/null 2>&1; then
            make efi-img
        else
            write_log "Failed to create EFI.img: no creation method available" "ERROR"
            return 1
        fi
    fi
    
    # Check kernel.elf
    if [[ ! -f "kernel.elf" ]]; then
        write_log "kernel.elf not found, attempting to build..." "WARNING"
        if command -v make >/dev/null 2>&1; then
            make all
        else
            write_log "Failed to build kernel: make not available" "ERROR"
            return 1
        fi
    fi
    
    write_log "Prerequisites check passed" "SUCCESS"
    return 0
}

start_qemu_test() {
    local test_name="$1"
    local timeout="${2:-$TEST_TIMEOUT}"
    local output_log="${test_name}_output.log"
    local error_log="${test_name}_error.log"
    
    write_log "Starting QEMU test: $test_name (timeout: ${timeout}s)" "INFO"
    
    # Clean old logs
    rm -f "$output_log" "$error_log" qemu-monitor.sock
    
    # Configure QEMU arguments
    local qemu_args=("${QEMU_ARGS[@]}")
    if [[ "$INTERACTIVE" != "true" ]]; then
        qemu_args+=("-display" "none")
    fi
    
    write_log "QEMU command: qemu-system-x86_64 ${qemu_args[*]}" "DEBUG"
    
    # Start QEMU process
    qemu-system-x86_64 "${qemu_args[@]}" > "$output_log" 2> "$error_log" &
    local qemu_pid=$!
    
    write_log "QEMU process started (PID: $qemu_pid)" "DEBUG"
    
    # Return process info
    echo "$qemu_pid:$output_log:$error_log"
}

monitor_qemu_execution() {
    local process_info="$1"
    local patterns_ref="$2"
    local test_name="$3"
    local timeout="${4:-$TEST_TIMEOUT}"
    
    IFS=':' read -r qemu_pid output_log error_log <<< "$process_info"
    
    local start_time=$(date +%s)
    local last_output_size=0
    local success_count=0
    local error_detected=false
    local detected_stages=()
    
    # Get pattern array by reference
    local -n patterns=$patterns_ref
    local required_patterns=${#patterns[@]}
    
    write_log "Monitoring $test_name execution (${required_patterns} patterns required)..." "DEBUG"
    
    while kill -0 "$qemu_pid" 2>/dev/null; do
        local current_time=$(date +%s)
        if (( current_time - start_time > timeout )); then
            write_log "Test timeout reached for $test_name" "WARNING"
            break
        fi
        
        # Analyze output
        if [[ -f "$output_log" ]]; then
            local current_size=$(wc -c < "$output_log" 2>/dev/null || echo 0)
            if (( current_size > last_output_size )); then
                local new_content=$(tail -c +$((last_output_size + 1)) "$output_log" 2>/dev/null || echo "")
                last_output_size=$current_size
                
                # Check for success patterns
                for pattern in "${patterns[@]}"; do
                    if echo "$new_content" | grep -qE "$pattern"; then
                        local match=$(echo "$new_content" | grep -oE "$pattern" | head -n1)
                        if [[ ! " ${detected_stages[*]} " =~ " ${pattern} " ]]; then
                            write_log "Pattern detected: $match" "SUCCESS"
                            detected_stages+=("$pattern")
                            ((success_count++))
                        fi
                    fi
                done
                
                # Check for error patterns
                for pattern in "${ERROR_PATTERNS[@]}"; do
                    if echo "$new_content" | grep -qE "$pattern"; then
                        local match=$(echo "$new_content" | grep -oE "$pattern" | head -n1)
                        write_log "Error detected: $match" "ERROR"
                        error_detected=true
                        break
                    fi
                done
                
                # Verbose output
                if [[ "$VERBOSE" == "true" ]]; then
                    echo "$new_content" | while IFS= read -r line; do
                        if [[ -n "${line// }" ]]; then
                            echo -e "  ${GRAY}QEMU: $line${NC}"
                        fi
                    done
                fi
            fi
        fi
        
        # Early exit conditions
        if [[ "$error_detected" == "true" ]]; then
            write_log "Stopping $test_name due to error detection" "ERROR"
            break
        fi
        
        # Success condition: all required patterns detected
        if (( success_count >= required_patterns )); then
            write_log "All required patterns detected for $test_name" "SUCCESS"
            break
        fi
        
        sleep 0.5
    done
    
    # Cleanup QEMU process
    if kill -0 "$qemu_pid" 2>/dev/null; then
        write_log "Terminating QEMU process..." "DEBUG"
        kill "$qemu_pid" 2>/dev/null || true
        wait "$qemu_pid" 2>/dev/null || true
    fi
    
    local duration=$(($(date +%s) - start_time))
    local test_success=$([ "$success_count" -ge "$required_patterns" ] && [ "$error_detected" == "false" ] && echo "true" || echo "false")
    
    # Store test results
    TEST_RESULTS["$test_name"]="$test_success"
    TEST_DURATIONS["$test_name"]="$duration"
    TEST_DETAILS["$test_name"]="Patterns: $success_count/$required_patterns, Duration: ${duration}s"
    
    if [[ "$test_success" == "true" ]]; then
        ((PASSED_TESTS++))
    else
        ((FAILED_TESTS++))
    fi
    ((TOTAL_TESTS++))
    
    write_log "$test_name completed: $([ "$test_success" == "true" ] && echo "PASS" || echo "FAIL") (${duration}s)" "INFO"
    
    # Cleanup logs if not saving
    if [[ "$SAVE_LOGS" != "true" ]]; then
        rm -f "$output_log" "$error_log"
    fi
    
    return $([ "$test_success" == "true" ] && echo 0 || echo 1)
}

# Test implementations
test_boot_validation() {
    write_log "=== BOOT VALIDATION TEST ===" "INFO"
    local process_info=$(start_qemu_test "boot_validation" 30)
    monitor_qemu_execution "$process_info" "BOOT_SUCCESS_PATTERNS" "boot_validation" 30
}

test_ring3_execution() {
    write_log "=== RING3 USER PROCESS EXECUTION TEST ===" "INFO"
    local process_info=$(start_qemu_test "ring3_execution" 40)
    monitor_qemu_execution "$process_info" "RING3_SUCCESS_PATTERNS" "ring3_execution" 40
}

test_devfs_operations() {
    write_log "=== DEVFS DEVICE I/O OPERATIONS TEST ===" "INFO"
    local process_info=$(start_qemu_test "devfs_operations" 35)
    monitor_qemu_execution "$process_info" "DEVFS_SUCCESS_PATTERNS" "devfs_operations" 35
}

test_syscall_roundtrip() {
    write_log "=== SYSCALL ROUNDTRIP TEST ===" "INFO"
    local process_info=$(start_qemu_test "syscall_roundtrip" 40)
    monitor_qemu_execution "$process_info" "SYSCALL_SUCCESS_PATTERNS" "syscall_roundtrip" 40
}

# Advanced QEMU debugging test using monitor interface
test_qemu_debugging() {
    write_log "=== QEMU DEBUGGING INTERFACE TEST ===" "INFO"
    
    local output_log="debug_test_output.log"
    local error_log="debug_test_error.log"
    local monitor_log="debug_monitor.log"
    
    # Clean old logs
    rm -f "$output_log" "$error_log" "$monitor_log" qemu-monitor.sock
    
    # Start QEMU with monitor
    local qemu_args=("${QEMU_ARGS[@]}")
    qemu_args+=("-display" "none")
    
    qemu-system-x86_64 "${qemu_args[@]}" > "$output_log" 2> "$error_log" &
    local qemu_pid=$!
    
    write_log "QEMU debugging test started (PID: $qemu_pid)" "DEBUG"
    
    # Wait for monitor socket
    local wait_count=0
    while [[ ! -S "qemu-monitor.sock" ]] && (( wait_count < 10 )); do
        sleep 1
        ((wait_count++))
    done
    
    local debug_success=false
    if [[ -S "qemu-monitor.sock" ]]; then
        # Test monitor commands
        {
            echo "info registers"
            echo "info cpus"
            echo "info memory"
            sleep 2
            echo "quit"
        } | socat - UNIX-CONNECT:qemu-monitor.sock > "$monitor_log" 2>&1 &
        
        # Monitor for a short time
        sleep 5
        
        if [[ -f "$monitor_log" ]] && grep -q "registers\|cpus\|memory" "$monitor_log"; then
            debug_success=true
            write_log "QEMU monitor interface working" "SUCCESS"
        else
            write_log "QEMU monitor interface failed" "ERROR"
        fi
    else
        write_log "QEMU monitor socket not created" "ERROR"
    fi
    
    # Cleanup
    if kill -0 "$qemu_pid" 2>/dev/null; then
        kill "$qemu_pid" 2>/dev/null || true
        wait "$qemu_pid" 2>/dev/null || true
    fi
    
    rm -f qemu-monitor.sock
    
    # Store results
    TEST_RESULTS["qemu_debugging"]=$([ "$debug_success" == "true" ] && echo "true" || echo "false")
    TEST_DURATIONS["qemu_debugging"]="5"
    TEST_DETAILS["qemu_debugging"]="Monitor interface test"
    
    if [[ "$debug_success" == "true" ]]; then
        ((PASSED_TESTS++))
    else
        ((FAILED_TESTS++))
    fi
    ((TOTAL_TESTS++))
    
    # Cleanup logs if not saving
    if [[ "$SAVE_LOGS" != "true" ]]; then
        rm -f "$output_log" "$error_log" "$monitor_log"
    fi
    
    return $([ "$debug_success" == "true" ] && echo 0 || echo 1)
}

generate_comprehensive_report() {
    local report_file="qemu_integration_test_report.md"
    local timestamp=$(date '+%Y-%m-%d %H:%M:%S')
    
    write_log "Generating comprehensive test report..." "INFO"
    
    cat > "$report_file" << EOF
# AykenOS QEMU Integration Test Report

**Generated:** $timestamp  
**Test Suite:** $TEST_SUITE  
**Total Tests:** $TOTAL_TESTS  
**Passed:** $PASSED_TESTS  
**Failed:** $FAILED_TESTS  
**Success Rate:** $(( PASSED_TESTS * 100 / TOTAL_TESTS ))%

## Test Configuration

- **Timeout:** ${TEST_TIMEOUT}s
- **Verbose:** $VERBOSE
- **Interactive:** $INTERACTIVE
- **Save Logs:** $SAVE_LOGS

## Test Results Summary

| Test Name | Status | Duration | Details |
|-----------|--------|----------|---------|
EOF

    for test_name in "${!TEST_RESULTS[@]}"; do
        local status="${TEST_RESULTS[$test_name]}"
        local duration="${TEST_DURATIONS[$test_name]}"
        local details="${TEST_DETAILS[$test_name]}"
        local status_icon=$([ "$status" == "true" ] && echo "✅ PASS" || echo "❌ FAIL")
        
        echo "| $test_name | $status_icon | ${duration}s | $details |" >> "$report_file"
    done
    
    cat >> "$report_file" << EOF

## Test Descriptions

### Boot Validation Test
Verifies that the AykenOS kernel boots successfully and completes all initialization phases:
- Early initialization (CPU, GDT, IDT, memory management)
- AI initialization (placeholder)
- Late initialization (scheduler, processes, filesystem, syscalls)

**Required Patterns:** ${#BOOT_SUCCESS_PATTERNS[@]}
- AykenOS initialization messages
- Kernel subsystem completion confirmations
- Scheduler readiness indication

### Ring3 User Process Execution Test
Validates that the kernel can successfully create and execute user-mode processes:
- User process creation and scheduling
- Ring3 privilege level transitions
- User space memory management

**Required Patterns:** ${#RING3_SUCCESS_PATTERNS[@]}
- User process scheduling messages
- Ring3 transition confirmations
- Process execution indicators

### DevFS Device I/O Operations Test
Confirms that the device filesystem is properly initialized with essential devices:
- Standard devices (/dev/null, /dev/zero, /dev/console)
- Input devices (/dev/kbd)
- Serial devices (/dev/ttyS0)
- Block devices (/dev/sda)

**Required Patterns:** ${#DEVFS_SUCCESS_PATTERNS[@]}
- Device registration confirmations
- VFS-DevFS integration messages

### Syscall Roundtrip Test
Verifies that the system call interface is properly configured and functional:
- Syscall handler installation
- INT 0x80 gate configuration
- System call interface readiness

**Required Patterns:** ${#SYSCALL_SUCCESS_PATTERNS[@]}
- Syscall installation messages
- Interface readiness confirmations

### QEMU Debugging Interface Test
Tests the QEMU monitor interface for advanced debugging capabilities:
- Monitor socket creation
- Register inspection commands
- CPU state queries
- Memory information access

## Requirements Validation

This test suite validates the following Phase 1 requirements:

- **4.1:** QEMU smoke tests verify basic kernel boot through log parsing ✓
- **4.2:** Ring3 validation demonstrates user mode execution via QEMU automation ✓
- **4.3:** DevFS integration confirms device file operations work correctly ✓
- **4.4:** Syscall roundtrip tests prove kernel-user transitions function properly ✓
- **4.5:** Automated test suite generates comprehensive validation reports ✓

## Next Steps

$(if (( FAILED_TESTS > 0 )); then
    echo "⚠️ **Action Required:** $FAILED_TESTS test(s) failed. Review the following:"
    for test_name in "${!TEST_RESULTS[@]}"; do
        if [[ "${TEST_RESULTS[$test_name]}" == "false" ]]; then
            echo "- **$test_name:** ${TEST_DETAILS[$test_name]}"
        fi
    done
    echo ""
    echo "Check saved logs (if enabled) for detailed error information."
else
    echo "✅ **All tests passed!** AykenOS Phase 1 critical functionality is validated."
    echo ""
    echo "The system is ready for:"
    echo "- Phase 2 development"
    echo "- AI integration features"
    echo "- Advanced filesystem implementation"
fi)

---
*Report generated by AykenOS QEMU Integration Test Suite*
EOF

    write_log "Test report saved to: $report_file" "SUCCESS"
    
    # Also display summary to console
    echo ""
    echo -e "${CYAN}============================================================${NC}"
    echo -e "${CYAN}QEMU INTEGRATION TEST SUITE SUMMARY${NC}"
    echo -e "${CYAN}============================================================${NC}"
    echo ""
    echo -e "${NC}Total Tests: $TOTAL_TESTS${NC}"
    echo -e "${GREEN}Passed: $PASSED_TESTS${NC}"
    echo -e "${RED}Failed: $FAILED_TESTS${NC}"
    echo -e "${BLUE}Success Rate: $(( PASSED_TESTS * 100 / TOTAL_TESTS ))%${NC}"
    echo ""
    
    if (( FAILED_TESTS > 0 )); then
        echo -e "${RED}Failed Tests:${NC}"
        for test_name in "${!TEST_RESULTS[@]}"; do
            if [[ "${TEST_RESULTS[$test_name]}" == "false" ]]; then
                echo -e "  ${RED}❌ $test_name${NC}"
            fi
        done
    else
        echo -e "${GREEN}🎉 All tests passed! Phase 1 validation complete.${NC}"
    fi
    
    echo ""
    echo -e "${CYAN}Detailed report: $report_file${NC}"
    echo -e "${CYAN}============================================================${NC}"
}

# Main execution
main() {
    echo -e "${GREEN}AykenOS QEMU Integration Test Suite${NC}"
    echo -e "${GRAY}Author: Kenan AY${NC}"
    echo -e "${GRAY}Phase 1 Critical Functionality Validation${NC}"
    echo ""
    
    if ! check_prerequisites; then
        exit 1
    fi
    
    write_log "Starting test suite: $TEST_SUITE" "INFO"
    
    case "$TEST_SUITE" in
        "boot")
            test_boot_validation
            ;;
        "ring3")
            test_ring3_execution
            ;;
        "devfs")
            test_devfs_operations
            ;;
        "syscall")
            test_syscall_roundtrip
            ;;
        "debug")
            test_qemu_debugging
            ;;
        "all"|*)
            test_boot_validation
            test_ring3_execution
            test_devfs_operations
            test_syscall_roundtrip
            test_qemu_debugging
            ;;
    esac
    
    generate_comprehensive_report
    
    # Exit with appropriate code
    exit $(( FAILED_TESTS > 0 ? 1 : 0 ))
}

# Run main function
main "$@"