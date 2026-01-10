#!/usr/bin/env bash
# AykenOS QEMU Test Runner
# Author: Kenan AY
# Purpose: Advanced QEMU boot testing with log analysis and automation

set -e

# Default parameters
TIMEOUT=30
VERBOSE=false
SAVE_LOGS=false
TEST_NAME="boot-test"
INTERACTIVE=false

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
GRAY='\033[0;37m'
NC='\033[0m'

# Test configuration
QEMU_ARGS=(
    "-drive" "format=raw,file=EFI.img"
    "-serial" "stdio"
    "-m" "256M"
    "-no-reboot"
    "-no-shutdown"
)

SUCCESS_PATTERNS=(
    "AykenOS.*INIT"
    "Kernel.*init.*done"
    "kmain.*starting"
    "EARLY INIT.*done"
    "Scheduler.*ready"
)

ERROR_PATTERNS=(
    "PANIC"
    "ERROR"
    "FATAL"
    "Triple fault"
    "General Protection Fault"
)

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --timeout)
            TIMEOUT="$2"
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
        --test-name)
            TEST_NAME="$2"
            shift 2
            ;;
        --interactive)
            INTERACTIVE=true
            shift
            ;;
        --help)
            echo "AykenOS QEMU Test Runner"
            echo ""
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --timeout N        Set timeout in seconds (default: 30)"
            echo "  --verbose          Enable verbose output"
            echo "  --save-logs        Save log files after test"
            echo "  --test-name NAME   Set test name (default: boot-test)"
            echo "  --interactive      Show QEMU display"
            echo "  --help             Show this help message"
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            exit 1
            ;;
    esac
done

write_test_log() {
    local message="$1"
    local level="${2:-INFO}"
    local timestamp=$(date '+%H:%M:%S.%3N')
    
    case "$level" in
        "SUCCESS") echo -e "[$timestamp] [$level] ${GREEN}$message${NC}" ;;
        "ERROR")   echo -e "[$timestamp] [$level] ${RED}$message${NC}" ;;
        "WARNING") echo -e "[$timestamp] [$level] ${YELLOW}$message${NC}" ;;
        "INFO")    echo -e "[$timestamp] [$level] ${CYAN}$message${NC}" ;;
        *)         echo -e "[$timestamp] [$level] $message" ;;
    esac
}

test_qemu_availability() {
    if ! command -v qemu-system-x86_64 >/dev/null 2>&1; then
        write_test_log "QEMU not found in PATH" "ERROR"
        echo -e "${YELLOW}Please install QEMU:${NC}"
        echo -e "  ${NC}Ubuntu/Debian: sudo apt install qemu-system-x86${NC}"
        echo -e "  ${NC}RHEL/CentOS: sudo yum install qemu-system-x86${NC}"
        echo -e "  ${NC}Arch Linux: sudo pacman -S qemu${NC}"
        return 1
    fi
    
    local qemu_version=$(qemu-system-x86_64 --version | head -n1)
    write_test_log "QEMU found: $qemu_version" "SUCCESS"
    return 0
}

monitor_qemu_execution() {
    local process_pid="$1"
    local output_log="$2"
    local error_log="$3"
    local analysis_log="$4"
    
    local start_time=$(date +%s)
    local last_output_size=0
    local boot_success=false
    local error_detected=false
    local boot_stages=()
    
    write_test_log "Monitoring QEMU execution (timeout: ${TIMEOUT}s)..." "INFO"
    
    while kill -0 "$process_pid" 2>/dev/null; do
        local current_time=$(date +%s)
        if (( current_time - start_time > TIMEOUT )); then
            write_test_log "Test timeout reached" "WARNING"
            break
        fi
        
        # Analyze output
        if [[ -f "$output_log" ]]; then
            local current_size=$(wc -c < "$output_log" 2>/dev/null || echo 0)
            if (( current_size > last_output_size )); then
                local new_content=$(tail -c +$((last_output_size + 1)) "$output_log" 2>/dev/null || echo "")
                last_output_size=$current_size
                
                # Check for success patterns
                for pattern in "${SUCCESS_PATTERNS[@]}"; do
                    if echo "$new_content" | grep -qE "$pattern"; then
                        local match=$(echo "$new_content" | grep -oE "$pattern" | head -n1)
                        write_test_log "Boot stage detected: $match" "SUCCESS"
                        boot_stages+=("$(date '+%H:%M:%S.%3N'): $match")
                        boot_success=true
                    fi
                done
                
                # Check for error patterns
                for pattern in "${ERROR_PATTERNS[@]}"; do
                    if echo "$new_content" | grep -qE "$pattern"; then
                        local match=$(echo "$new_content" | grep -oE "$pattern" | head -n1)
                        write_test_log "Error detected: $match" "ERROR"
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
            write_test_log "Stopping test due to error detection" "ERROR"
            break
        fi
        
        sleep 0.5
    done
    
    local duration=$(($(date +%s) - start_time))
    write_test_log "Test completed in ${duration} seconds" "INFO"
    
    # Write analysis log
    cat > "$analysis_log" << EOF
{
    "test_name": "$TEST_NAME",
    "duration": $duration,
    "success": $([ "$boot_success" == "true" ] && [ "$error_detected" == "false" ] && echo "true" || echo "false"),
    "boot_stages": [$(printf '"%s",' "${boot_stages[@]}" | sed 's/,$//')]
    "error_detected": $([ "$error_detected" == "true" ] && echo "true" || echo "false"),
    "timeout_reached": $((current_time - start_time > TIMEOUT))
}
EOF
    
    # Return success status
    [ "$boot_success" == "true" ] && [ "$error_detected" == "false" ]
}

generate_test_report() {
    local test_name="$1"
    local output_log="$2"
    local error_log="$3"
    local analysis_log="$4"
    
    echo ""
    echo -e "${CYAN}============================================================${NC}"
    echo -e "${CYAN}QEMU Test Report: $test_name${NC}"
    echo -e "${CYAN}============================================================${NC}"
    
    if [[ -f "$analysis_log" ]]; then
        local success=$(grep -o '"success": *[^,]*' "$analysis_log" | cut -d: -f2 | tr -d ' "')
        local duration=$(grep -o '"duration": *[^,]*' "$analysis_log" | cut -d: -f2 | tr -d ' "')
        local error_detected=$(grep -o '"error_detected": *[^,]*' "$analysis_log" | cut -d: -f2 | tr -d ' "')
        
        echo ""
        echo -e "${NC}Test Results:${NC}"
        echo -e "  Status: $([ "$success" == "true" ] && echo -e "${GREEN}✓ PASS${NC}" || echo -e "${RED}✗ FAIL${NC}")"
        echo -e "  Duration: ${duration} seconds"
        echo -e "  Errors Detected: $([ "$error_detected" == "true" ] && echo -e "${RED}Yes${NC}" || echo -e "${GREEN}No${NC}")"
        
        # Extract boot stages
        local boot_stages=$(grep -o '"boot_stages": *\[[^]]*\]' "$analysis_log" | sed 's/"boot_stages": *\[//; s/\]$//' | tr ',' '\n' | sed 's/^"//; s/"$//')
        if [[ -n "$boot_stages" ]]; then
            echo ""
            echo -e "${GREEN}Boot Stages:${NC}"
            echo "$boot_stages" | while IFS= read -r stage; do
                if [[ -n "$stage" ]]; then
                    echo -e "  ${GREEN}$stage${NC}"
                fi
            done
        fi
    fi
    
    if [[ -f "$error_log" && -s "$error_log" ]]; then
        echo ""
        echo -e "${RED}QEMU Errors:${NC}"
        sed 's/^/  /' "$error_log"
    fi
    
    echo ""
    echo -e "${CYAN}Log Files:${NC}"
    if [[ "$SAVE_LOGS" == "true" ]]; then
        echo -e "  Output: $output_log"
        echo -e "  Errors: $error_log"
        echo -e "  Analysis: $analysis_log"
    else
        echo -e "  ${GRAY}Logs cleaned up (use --save-logs to preserve)${NC}"
    fi
    
    echo -e "${CYAN}============================================================${NC}"
}

start_qemu_test() {
    local test_name="$1"
    
    write_test_log "Starting QEMU test: $test_name" "INFO"
    
    # Ensure EFI image exists
    if [[ ! -f "EFI.img" ]]; then
        write_test_log "EFI.img not found, creating..." "WARNING"
        if [[ -x "./make_efi_img.sh" ]]; then
            ./make_efi_img.sh
        elif command -v make >/dev/null 2>&1; then
            make efi-img
        else
            write_test_log "Failed to create EFI.img: no creation method available" "ERROR"
            return 1
        fi
        write_test_log "EFI.img created successfully" "SUCCESS"
    fi
    
    # Prepare log files
    local output_log="${test_name}_output.log"
    local error_log="${test_name}_error.log"
    local analysis_log="${test_name}_analysis.log"
    
    # Clean old logs
    rm -f "$output_log" "$error_log" "$analysis_log"
    
    # Configure QEMU arguments
    local qemu_args=("${QEMU_ARGS[@]}")
    if [[ "$INTERACTIVE" != "true" ]]; then
        qemu_args+=("-display" "none")
    fi
    
    write_test_log "QEMU command: qemu-system-x86_64 ${qemu_args[*]}" "INFO"
    
    # Start QEMU process
    qemu-system-x86_64 "${qemu_args[@]}" > "$output_log" 2> "$error_log" &
    local qemu_pid=$!
    
    write_test_log "QEMU process started (PID: $qemu_pid)" "INFO"
    
    # Monitor the test
    local test_success=false
    if monitor_qemu_execution "$qemu_pid" "$output_log" "$error_log" "$analysis_log"; then
        test_success=true
    fi
    
    # Cleanup
    if kill -0 "$qemu_pid" 2>/dev/null; then
        write_test_log "Terminating QEMU process..." "INFO"
        kill "$qemu_pid" 2>/dev/null || true
        wait "$qemu_pid" 2>/dev/null || true
    fi
    
    # Generate report
    generate_test_report "$test_name" "$output_log" "$error_log" "$analysis_log"
    
    # Cleanup logs if not saving
    if [[ "$SAVE_LOGS" != "true" ]]; then
        rm -f "$output_log" "$error_log" "$analysis_log"
    fi
    
    return $([ "$test_success" == "true" ] && echo 0 || echo 1)
}

# Main execution
echo -e "${GREEN}AykenOS QEMU Test Runner${NC}"
echo -e "${GRAY}Author: Kenan AY${NC}"
echo ""

if ! test_qemu_availability; then
    exit 1
fi

if start_qemu_test "$TEST_NAME"; then
    write_test_log "Test completed successfully" "SUCCESS"
    exit 0
else
    write_test_log "Test failed" "ERROR"
    exit 1
fi