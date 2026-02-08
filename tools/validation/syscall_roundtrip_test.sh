#!/usr/bin/env bash
# AykenOS Syscall Roundtrip Testing
# Author: Kenan AY
# Purpose: Specialized testing for syscall interface and kernel-user transitions

set -e

# Configuration
TIMEOUT=50
VERBOSE=false
SAVE_LOGS=false

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
GRAY='\033[0;37m'
NC='\033[0m'

# Syscall specific test patterns
SYSCALL_INIT_PATTERNS=(
    "syscall.*installing.*INT.*0x80"
    "IDT.*gate.*0x80"
    "Syscall interface ready"
    "syscall.*init"
)

SYSCALL_HANDLER_PATTERNS=(
    "syscall.*handler"
    "SYS_read"
    "SYS_write"
    "SYS_open"
    "SYS_close"
    "SYS_exit"
)

SYSCALL_EXECUTION_PATTERNS=(
    "\\[U\\]\\[SYSCALL_OK\\]"
    "syscall.*dispatcher"
    "Ring3.*Ring0.*transition"
    "user.*mode.*syscall"
    "INT.*0x80.*called"
)

SYSCALL_USER_PATTERNS=(
    "user AI service.*syscall"
    "stdout.*write"
    "Process exit requested"
    "syscall.*from.*user"
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
            echo "AykenOS Syscall Roundtrip Test"
            echo ""
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --timeout N        Set timeout in seconds (default: 50)"
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

check_syscall_prerequisites() {
    write_log "Checking syscall test prerequisites..." "INFO"
    
    # Check for syscall implementation
    if [[ ! -f "kernel/sys/syscall.c" ]]; then
        write_log "Syscall implementation file not found" "ERROR"
        return 1
    fi
    
    # Check for syscall handler functions
    if ! grep -q "syscall_handler\|syscall_init" kernel/sys/syscall.c; then
        write_log "Syscall handler functions not found" "ERROR"
        return 1
    fi
    
    # Check for interrupt handling
    if ! grep -q "INT.*0x80\|syscall_isr" kernel/sys/syscall.c; then
        write_log "Syscall interrupt handling not found" "ERROR"
        return 1
    fi
    
    # Check for user process with syscalls
    if ! grep -q "ai_service_stub\|SYS_write\|int.*0x80" kernel/proc/proc.c; then
        write_log "User process syscall usage not found" "WARNING"
    fi
    
    write_log "Syscall prerequisites check passed" "SUCCESS"
    return 0
}

analyze_syscall_flow() {
    local output_content="$1"
    local syscall_flow=()
    
    # Track syscall initialization
    if echo "$output_content" | grep -qE "syscall.*installing.*INT.*0x80"; then
        syscall_flow+=("INT_0x80_INSTALLED")
        write_log "Syscall interrupt gate installed" "SUCCESS"
    fi
    
    # Track syscall interface readiness
    if echo "$output_content" | grep -qE "Syscall interface ready"; then
        syscall_flow+=("INTERFACE_READY")
        write_log "Syscall interface ready" "SUCCESS"
    fi
    
    # Track user process syscall attempts
    if echo "$output_content" | grep -qE "user.*AI.*service.*scheduled"; then
        syscall_flow+=("USER_PROCESS_CREATED")
        write_log "User process with syscalls created" "SUCCESS"
    fi
    
    # Track syscall handler invocations
    local syscall_count=$(echo "$output_content" | grep -cE "syscall.*handler|SYS_.*called" || echo 0)
    if (( syscall_count > 0 )); then
        syscall_flow+=("HANDLER_INVOKED:$syscall_count")
        write_log "Syscall handler invoked $syscall_count times" "SUCCESS"
    fi
    
    # Track specific syscall types
    local write_syscalls=$(echo "$output_content" | grep -cE "SYS_write" || echo 0)
    local read_syscalls=$(echo "$output_content" | grep -cE "SYS_read" || echo 0)
    local exit_syscalls=$(echo "$output_content" | grep -cE "SYS_exit|Process exit requested" || echo 0)
    
    if (( write_syscalls > 0 )); then
        syscall_flow+=("WRITE_SYSCALLS:$write_syscalls")
        write_log "Write syscalls detected: $write_syscalls" "SUCCESS"
    fi
    
    if (( read_syscalls > 0 )); then
        syscall_flow+=("READ_SYSCALLS:$read_syscalls")
        write_log "Read syscalls detected: $read_syscalls" "SUCCESS"
    fi
    
    if (( exit_syscalls > 0 )); then
        syscall_flow+=("EXIT_SYSCALLS:$exit_syscalls")
        write_log "Exit syscalls detected: $exit_syscalls" "SUCCESS"
    fi
    
    echo "${syscall_flow[*]}"
}

run_syscall_validation() {
    local test_name="syscall_roundtrip"
    local output_log="${test_name}_output.log"
    local error_log="${test_name}_error.log"
    local analysis_log="${test_name}_analysis.log"
    
    write_log "Starting comprehensive syscall roundtrip validation..." "INFO"
    
    # Clean old logs
    rm -f "$output_log" "$error_log" "$analysis_log"
    
    # QEMU arguments optimized for syscall testing
    local serial_arg="stdio"
    if [[ -n "${SYSCALL_SERIAL_LOG:-}" ]]; then
        serial_arg="file:${SYSCALL_SERIAL_LOG}"
    fi
    local debugcon_args=()
    if [[ -n "${SYSCALL_DEBUGCON_LOG:-}" ]]; then
        debugcon_args=("-debugcon" "file:${SYSCALL_DEBUGCON_LOG}" "-global" "isa-debugcon.iobase=0xe9")
    fi


    # Optional UEFI firmware (OVMF) for deterministic boot
    local ovmf_args=()
    if [[ -f "OVMF_CODE.fd" && -f "OVMF_VARS.fd" ]]; then
        ovmf_args=(
            "-machine" "q35"
            "-drive" "if=pflash,format=raw,readonly=on,file=OVMF_CODE.fd"
            "-drive" "if=pflash,format=raw,file=OVMF_VARS.fd"
        )
    fi
    local qemu_args=(
        "-drive" "format=raw,file=EFI.img"
        "-serial" "$serial_arg"
        "-m" "256M"
        "-no-reboot"
        "-no-shutdown"
        "-display" "none"
        "-d" "int"  # Debug interrupts to catch syscalls
        "-D" "qemu_syscall_debug.log"
    )
    if (( ${#debugcon_args[@]} > 0 )); then
        qemu_args+=("${debugcon_args[@]}")
    fi
    if (( ${#ovmf_args[@]} > 0 )); then
        qemu_args+=("${ovmf_args[@]}")
    fi
    
    write_log "QEMU command: qemu-system-x86_64 ${qemu_args[*]}" "DEBUG"
    
    # Start QEMU
    qemu-system-x86_64 "${qemu_args[@]}" > "$output_log" 2> "$error_log" &
    local qemu_pid=$!
    
    write_log "QEMU process started (PID: $qemu_pid)" "DEBUG"
    
    # Monitor execution with syscall-specific analysis
    local start_time=$(date +%s)
    local last_output_size=0
    local syscall_stages=()
    local init_detected=0
    local handler_detected=0
    local execution_detected=0
    local user_detected=0
    local error_detected=false
    local full_output=""
    
    write_log "Monitoring syscall interface and roundtrip execution..." "INFO"
    
    while kill -0 "$qemu_pid" 2>/dev/null; do
        local current_time=$(date +%s)
        if (( current_time - start_time > TIMEOUT )); then
            write_log "Syscall test timeout reached" "WARNING"
            break
        fi
        
        # Analyze output
        if [[ -f "$output_log" ]]; then
            local current_size=$(wc -c < "$output_log" 2>/dev/null || echo 0)
            if (( current_size > last_output_size )); then
                local new_content=$(tail -c +$((last_output_size + 1)) "$output_log" 2>/dev/null || echo "")
                last_output_size=$current_size
                full_output+="$new_content"
                
                # Check syscall initialization patterns
                for pattern in "${SYSCALL_INIT_PATTERNS[@]}"; do
                    if echo "$new_content" | grep -qE "$pattern"; then
                        local match=$(echo "$new_content" | grep -oE "$pattern" | head -n1)
                        write_log "Syscall Init: $match" "SUCCESS"
                        syscall_stages+=("INIT: $match")
                        ((init_detected++))
                    fi
                done
                
                # Check syscall handler patterns
                for pattern in "${SYSCALL_HANDLER_PATTERNS[@]}"; do
                    if echo "$new_content" | grep -qE "$pattern"; then
                        local match=$(echo "$new_content" | grep -oE "$pattern" | head -n1)
                        write_log "Syscall Handler: $match" "SUCCESS"
                        syscall_stages+=("HANDLER: $match")
                        ((handler_detected++))
                    fi
                done
                
                # Check syscall execution patterns
                for pattern in "${SYSCALL_EXECUTION_PATTERNS[@]}"; do
                    if echo "$new_content" | grep -qE "$pattern"; then
                        local match=$(echo "$new_content" | grep -oE "$pattern" | head -n1)
                        write_log "Syscall Execution: $match" "SUCCESS"
                        syscall_stages+=("EXECUTION: $match")
                        ((execution_detected++))
                    fi
                done
                
                # Check user syscall patterns
                for pattern in "${SYSCALL_USER_PATTERNS[@]}"; do
                    if echo "$new_content" | grep -qE "$pattern"; then
                        local match=$(echo "$new_content" | grep -oE "$pattern" | head -n1)
                        write_log "User Syscall: $match" "SUCCESS"
                        syscall_stages+=("USER: $match")
                        ((user_detected++))
                    fi
                done
                
                # Check for errors
                if echo "$new_content" | grep -qE "PANIC|FATAL|syscall.*ERROR|Invalid.*syscall|ENOSYS"; then
                    local match=$(echo "$new_content" | grep -oE "PANIC|FATAL|syscall.*ERROR|Invalid.*syscall|ENOSYS" | head -n1)
                    write_log "Syscall Error detected: $match" "ERROR"
                    error_detected=true
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
    
    # Analyze syscall flow
    local syscall_flow=$(analyze_syscall_flow "$full_output")
    
    # Syscall validation criteria:
    # - Syscall interface initialization (INT 0x80 gate)
    # - Syscall handler registration
    # - User process creation with syscalls
    # - No critical syscall errors
    local syscall_success=false
    if (( init_detected >= 1 && handler_detected >= 1 && !error_detected )); then
        syscall_success=true
    fi
    
    # Check QEMU debug log for interrupt information
    local interrupt_analysis=""
    if [[ -f "qemu_syscall_debug.log" ]]; then
        local int80_count=$(grep -c "int.*0x80\|interrupt.*128" qemu_syscall_debug.log 2>/dev/null || echo 0)
        if (( int80_count > 0 )); then
            interrupt_analysis="INT_0x80_TRIGGERED:$int80_count"
            write_log "INT 0x80 interrupts detected: $int80_count" "SUCCESS"
        fi
    fi
    
    # Generate analysis report
    cat > "$analysis_log" << EOF
{
    "test_name": "$test_name",
    "duration": $duration,
    "success": $([ "$syscall_success" == "true" ] && echo "true" || echo "false"),
    "init_detected": $init_detected,
    "handler_detected": $handler_detected,
    "execution_detected": $execution_detected,
    "user_detected": $user_detected,
    "syscall_flow": "$syscall_flow",
    "interrupt_analysis": "$interrupt_analysis",
    "error_detected": $([ "$error_detected" == "true" ] && echo "true" || echo "false"),
    "stages": [$(printf '"%s",' "${syscall_stages[@]}" | sed 's/,$//')]
}
EOF
    
    # Generate detailed report
    echo ""
    echo -e "${CYAN}============================================================${NC}"
    echo -e "${CYAN}SYSCALL ROUNDTRIP TEST REPORT${NC}"
    echo -e "${CYAN}============================================================${NC}"
    echo ""
    echo -e "${NC}Test Results:${NC}"
    echo -e "  Status: $([ "$syscall_success" == "true" ] && echo -e "${GREEN}✓ PASS${NC}" || echo -e "${RED}✗ FAIL${NC}")"
    echo -e "  Duration: ${duration} seconds"
    echo ""
    echo -e "${NC}Syscall Component Analysis:${NC}"
    echo -e "  Interface Initialization: $init_detected (required: ≥1)"
    echo -e "  Handler Registration: $handler_detected (required: ≥1)"
    echo -e "  Execution Detection: $execution_detected"
    echo -e "  User Process Syscalls: $user_detected"
    echo -e "  Errors Detected: $([ "$error_detected" == "true" ] && echo -e "${RED}Yes${NC}" || echo -e "${GREEN}No${NC}")"
    
    if [[ -n "$syscall_flow" ]]; then
        echo ""
        echo -e "${GREEN}Syscall Flow Analysis:${NC}"
        for flow_item in $syscall_flow; do
            echo -e "  ${GREEN}$flow_item${NC}"
        done
    fi
    
    if [[ -n "$interrupt_analysis" ]]; then
        echo ""
        echo -e "${GREEN}Interrupt Analysis:${NC}"
        echo -e "  ${GREEN}$interrupt_analysis${NC}"
    fi
    
    if (( ${#syscall_stages[@]} > 0 )); then
        echo ""
        echo -e "${GREEN}Syscall Execution Stages:${NC}"
        for stage in "${syscall_stages[@]}"; do
            echo -e "  ${GREEN}$stage${NC}"
        done
    fi
    
    echo ""
    echo -e "${CYAN}Syscall Validation Criteria:${NC}"
    echo -e "  ✓ INT 0x80 gate installation: $([ $init_detected -ge 1 ] && echo -e "${GREEN}PASS${NC}" || echo -e "${RED}FAIL${NC}")"
    echo -e "  ✓ Syscall handler registration: $([ $handler_detected -ge 1 ] && echo -e "${GREEN}PASS${NC}" || echo -e "${RED}FAIL${NC}")"
    echo -e "  ✓ No critical syscall errors: $([ "$error_detected" == "false" ] && echo -e "${GREEN}PASS${NC}" || echo -e "${RED}FAIL${NC}")"
    
    echo ""
    echo -e "${CYAN}Syscall Interface Coverage:${NC}"
    echo -e "  • INT 0x80 interrupt gate: $(echo "$syscall_flow" | grep -q "INT_0x80" && echo -e "${GREEN}✓${NC}" || echo -e "${RED}✗${NC}")"
    echo -e "  • Syscall interface ready: $(echo "$syscall_flow" | grep -q "INTERFACE_READY" && echo -e "${GREEN}✓${NC}" || echo -e "${RED}✗${NC}")"
    echo -e "  • User process creation: $(echo "$syscall_flow" | grep -q "USER_PROCESS" && echo -e "${GREEN}✓${NC}" || echo -e "${RED}✗${NC}")"
    echo -e "  • Handler invocation: $(echo "$syscall_flow" | grep -q "HANDLER_INVOKED" && echo -e "${GREEN}✓${NC}" || echo -e "${RED}✗${NC}")"
    echo -e "  • Write syscalls: $(echo "$syscall_flow" | grep -q "WRITE_SYSCALLS" && echo -e "${GREEN}✓${NC}" || echo -e "${RED}✗${NC}")"
    
    echo ""
    echo -e "${CYAN}Log Files:${NC}"
    if [[ "$SAVE_LOGS" == "true" ]]; then
        echo -e "  Output: $output_log"
        echo -e "  Errors: $error_log"
        echo -e "  Analysis: $analysis_log"
        if [[ -f "qemu_syscall_debug.log" ]]; then
            echo -e "  QEMU Debug: qemu_syscall_debug.log"
        fi
    else
        echo -e "  ${GRAY}Logs cleaned up (use --save-logs to preserve)${NC}"
        rm -f "$output_log" "$error_log" "$analysis_log" qemu_syscall_debug.log
    fi
    
    echo -e "${CYAN}============================================================${NC}"
    
    return $([ "$syscall_success" == "true" ] && echo 0 || echo 1)
}

# Main execution
echo -e "${GREEN}AykenOS Syscall Roundtrip Testing${NC}"
echo -e "${GRAY}Author: Kenan AY${NC}"
echo -e "${GRAY}Specialized Syscall Interface Validation${NC}"
echo ""

if ! check_syscall_prerequisites; then
    exit 1
fi

if run_syscall_validation; then
    write_log "Syscall roundtrip validation completed successfully" "SUCCESS"
    exit 0
else
    write_log "Syscall roundtrip validation failed" "ERROR"
    exit 1
fi