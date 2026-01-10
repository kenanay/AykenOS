#!/usr/bin/env bash
# AykenOS Ring3 User Process Execution Validation
# Author: Kenan AY
# Purpose: Specialized testing for Ring3 context switching and user process execution

set -e

# Configuration
TIMEOUT=60
VERBOSE=false
SAVE_LOGS=false

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
GRAY='\033[0;37m'
NC='\033[0m'

# Ring3 specific test patterns
RING3_INIT_PATTERNS=(
    "GDT.*init"
    "TSS.*init"
    "IDT.*init"
    "Ring3.*selector.*0x23"
    "Ring3.*selector.*0x1b"
)

RING3_PROCESS_PATTERNS=(
    "user.*process.*created"
    "ai-service.*Ring3"
    "user AI service scheduled"
    "PID.*running"
    "context.*switch"
)

RING3_SYSCALL_PATTERNS=(
    "syscall.*installing.*INT.*0x80"
    "SYS_write"
    "syscall.*handler"
    "Ring3.*Ring0.*transition"
)

RING3_MEMORY_PATTERNS=(
    "User.*stack.*TOP"
    "USER_TEXT_BASE"
    "paging.*user.*pml4"
    "user.*space.*memory"
)

# Parse arguments
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
        --help)
            echo "AykenOS Ring3 Validation Test"
            echo ""
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --timeout N        Set timeout in seconds (default: 60)"
            echo "  --verbose          Enable verbose output"
            echo "  --save-logs        Save log files after test"
            echo "  --help             Show this help message"
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            exit 1
            ;;
    esac
done

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

check_ring3_prerequisites() {
    write_log "Checking Ring3 test prerequisites..." "INFO"
    
    # Check for GDT constants in source
    if ! grep -r "0x23\|0x1b" kernel/arch/x86_64/ >/dev/null 2>&1; then
        write_log "Ring3 GDT selectors not found in source code" "ERROR"
        return 1
    fi
    
    # Check for context switch implementation
    if [[ ! -f "kernel/arch/x86_64/context_switch.asm" ]]; then
        write_log "Context switch assembly file not found" "ERROR"
        return 1
    fi
    
    # Check for user process creation code
    if ! grep -r "proc_create_user_process\|PROC_TYPE_USER" kernel/ >/dev/null 2>&1; then
        write_log "User process creation code not found" "ERROR"
        return 1
    fi
    
    write_log "Ring3 prerequisites check passed" "SUCCESS"
    return 0
}

run_ring3_validation() {
    local test_name="ring3_comprehensive"
    local output_log="${test_name}_output.log"
    local error_log="${test_name}_error.log"
    local analysis_log="${test_name}_analysis.log"
    
    write_log "Starting comprehensive Ring3 validation..." "INFO"
    
    # Clean old logs
    rm -f "$output_log" "$error_log" "$analysis_log"
    
    # QEMU arguments optimized for Ring3 testing
    local qemu_args=(
        "-drive" "format=raw,file=EFI.img"
        "-serial" "stdio"
        "-m" "512M"  # More memory for user processes
        "-no-reboot"
        "-no-shutdown"
        "-display" "none"
        "-d" "int,cpu_reset"  # Debug interrupts and CPU resets
        "-D" "qemu_debug.log"  # QEMU debug log
    )
    
    write_log "QEMU command: qemu-system-x86_64 ${qemu_args[*]}" "DEBUG"
    
    # Start QEMU
    qemu-system-x86_64 "${qemu_args[@]}" > "$output_log" 2> "$error_log" &
    local qemu_pid=$!
    
    write_log "QEMU process started (PID: $qemu_pid)" "DEBUG"
    
    # Monitor execution with Ring3-specific analysis
    local start_time=$(date +%s)
    local last_output_size=0
    local ring3_stages=()
    local init_detected=0
    local process_detected=0
    local syscall_detected=0
    local memory_detected=0
    local error_detected=false
    
    write_log "Monitoring Ring3 execution phases..." "INFO"
    
    while kill -0 "$qemu_pid" 2>/dev/null; do
        local current_time=$(date +%s)
        if (( current_time - start_time > TIMEOUT )); then
            write_log "Ring3 test timeout reached" "WARNING"
            break
        fi
        
        # Analyze output
        if [[ -f "$output_log" ]]; then
            local current_size=$(wc -c < "$output_log" 2>/dev/null || echo 0)
            if (( current_size > last_output_size )); then
                local new_content=$(tail -c +$((last_output_size + 1)) "$output_log" 2>/dev/null || echo "")
                last_output_size=$current_size
                
                # Check Ring3 initialization patterns
                for pattern in "${RING3_INIT_PATTERNS[@]}"; do
                    if echo "$new_content" | grep -qE "$pattern"; then
                        local match=$(echo "$new_content" | grep -oE "$pattern" | head -n1)
                        write_log "Ring3 Init: $match" "SUCCESS"
                        ring3_stages+=("INIT: $match")
                        ((init_detected++))
                    fi
                done
                
                # Check Ring3 process patterns
                for pattern in "${RING3_PROCESS_PATTERNS[@]}"; do
                    if echo "$new_content" | grep -qE "$pattern"; then
                        local match=$(echo "$new_content" | grep -oE "$pattern" | head -n1)
                        write_log "Ring3 Process: $match" "SUCCESS"
                        ring3_stages+=("PROCESS: $match")
                        ((process_detected++))
                    fi
                done
                
                # Check Ring3 syscall patterns
                for pattern in "${RING3_SYSCALL_PATTERNS[@]}"; do
                    if echo "$new_content" | grep -qE "$pattern"; then
                        local match=$(echo "$new_content" | grep -oE "$pattern" | head -n1)
                        write_log "Ring3 Syscall: $match" "SUCCESS"
                        ring3_stages+=("SYSCALL: $match")
                        ((syscall_detected++))
                    fi
                done
                
                # Check Ring3 memory patterns
                for pattern in "${RING3_MEMORY_PATTERNS[@]}"; do
                    if echo "$new_content" | grep -qE "$pattern"; then
                        local match=$(echo "$new_content" | grep -oE "$pattern" | head -n1)
                        write_log "Ring3 Memory: $match" "SUCCESS"
                        ring3_stages+=("MEMORY: $match")
                        ((memory_detected++))
                    fi
                done
                
                # Check for errors
                if echo "$new_content" | grep -qE "PANIC|ERROR|FATAL|Triple fault|General Protection Fault"; then
                    local match=$(echo "$new_content" | grep -oE "PANIC|ERROR|FATAL|Triple fault|General Protection Fault" | head -n1)
                    write_log "Ring3 Error detected: $match" "ERROR"
                    error_detected=true
                    break
                fi
                
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
        
        sleep 0.5
    done
    
    # Cleanup QEMU
    if kill -0 "$qemu_pid" 2>/dev/null; then
        write_log "Terminating QEMU process..." "DEBUG"
        kill "$qemu_pid" 2>/dev/null || true
        wait "$qemu_pid" 2>/dev/null || true
    fi
    
    local duration=$(($(date +%s) - start_time))
    
    # Analyze results
    local total_detections=$((init_detected + process_detected + syscall_detected + memory_detected))
    local ring3_success=false
    
    # Ring3 validation criteria:
    # - At least 2 initialization patterns
    # - At least 1 process creation pattern
    # - At least 1 syscall pattern
    # - No critical errors
    if (( init_detected >= 2 && process_detected >= 1 && syscall_detected >= 1 && !error_detected )); then
        ring3_success=true
    fi
    
    # Generate analysis report
    cat > "$analysis_log" << EOF
{
    "test_name": "$test_name",
    "duration": $duration,
    "success": $([ "$ring3_success" == "true" ] && echo "true" || echo "false"),
    "init_patterns": $init_detected,
    "process_patterns": $process_detected,
    "syscall_patterns": $syscall_detected,
    "memory_patterns": $memory_detected,
    "total_detections": $total_detections,
    "error_detected": $([ "$error_detected" == "true" ] && echo "true" || echo "false"),
    "stages": [$(printf '"%s",' "${ring3_stages[@]}" | sed 's/,$//')]
}
EOF
    
    # Generate detailed report
    echo ""
    echo -e "${CYAN}============================================================${NC}"
    echo -e "${CYAN}RING3 VALIDATION TEST REPORT${NC}"
    echo -e "${CYAN}============================================================${NC}"
    echo ""
    echo -e "${NC}Test Results:${NC}"
    echo -e "  Status: $([ "$ring3_success" == "true" ] && echo -e "${GREEN}✓ PASS${NC}" || echo -e "${RED}✗ FAIL${NC}")"
    echo -e "  Duration: ${duration} seconds"
    echo -e "  Total Detections: $total_detections"
    echo ""
    echo -e "${NC}Ring3 Component Analysis:${NC}"
    echo -e "  Initialization Patterns: $init_detected (required: ≥2)"
    echo -e "  Process Creation Patterns: $process_detected (required: ≥1)"
    echo -e "  Syscall Interface Patterns: $syscall_detected (required: ≥1)"
    echo -e "  Memory Management Patterns: $memory_detected"
    echo -e "  Errors Detected: $([ "$error_detected" == "true" ] && echo -e "${RED}Yes${NC}" || echo -e "${GREEN}No${NC}")"
    
    if (( ${#ring3_stages[@]} > 0 )); then
        echo ""
        echo -e "${GREEN}Detected Ring3 Stages:${NC}"
        for stage in "${ring3_stages[@]}"; do
            echo -e "  ${GREEN}$stage${NC}"
        done
    fi
    
    # Check QEMU debug log if available
    if [[ -f "qemu_debug.log" ]]; then
        echo ""
        echo -e "${CYAN}QEMU Debug Analysis:${NC}"
        local interrupt_count=$(grep -c "interrupt" qemu_debug.log 2>/dev/null || echo 0)
        local exception_count=$(grep -c "exception\|fault" qemu_debug.log 2>/dev/null || echo 0)
        echo -e "  Interrupts: $interrupt_count"
        echo -e "  Exceptions/Faults: $exception_count"
        
        if [[ "$SAVE_LOGS" != "true" ]]; then
            rm -f qemu_debug.log
        fi
    fi
    
    echo ""
    echo -e "${CYAN}Ring3 Validation Criteria:${NC}"
    echo -e "  ✓ GDT/IDT/TSS initialization: $([ $init_detected -ge 2 ] && echo -e "${GREEN}PASS${NC}" || echo -e "${RED}FAIL${NC}")"
    echo -e "  ✓ User process creation: $([ $process_detected -ge 1 ] && echo -e "${GREEN}PASS${NC}" || echo -e "${RED}FAIL${NC}")"
    echo -e "  ✓ Syscall interface setup: $([ $syscall_detected -ge 1 ] && echo -e "${GREEN}PASS${NC}" || echo -e "${RED}FAIL${NC}")"
    echo -e "  ✓ No critical errors: $([ "$error_detected" == "false" ] && echo -e "${GREEN}PASS${NC}" || echo -e "${RED}FAIL${NC}")"
    
    echo ""
    echo -e "${CYAN}Log Files:${NC}"
    if [[ "$SAVE_LOGS" == "true" ]]; then
        echo -e "  Output: $output_log"
        echo -e "  Errors: $error_log"
        echo -e "  Analysis: $analysis_log"
    else
        echo -e "  ${GRAY}Logs cleaned up (use --save-logs to preserve)${NC}"
        rm -f "$output_log" "$error_log" "$analysis_log"
    fi
    
    echo -e "${CYAN}============================================================${NC}"
    
    return $([ "$ring3_success" == "true" ] && echo 0 || echo 1)
}

# Main execution
echo -e "${GREEN}AykenOS Ring3 User Process Execution Validation${NC}"
echo -e "${GRAY}Author: Kenan AY${NC}"
echo -e "${GRAY}Specialized Ring3 Context Switching Test${NC}"
echo ""

if ! check_ring3_prerequisites; then
    exit 1
fi

if run_ring3_validation; then
    write_log "Ring3 validation completed successfully" "SUCCESS"
    exit 0
else
    write_log "Ring3 validation failed" "ERROR"
    exit 1
fi