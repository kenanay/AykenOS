#!/usr/bin/env bash
# AykenOS Ring3 User Process Execution Validation
# Author: Kenan AY
# Purpose: Specialized testing for Ring3 context switching and user process execution

set -euo pipefail

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
    "\\[U\\]\\[RING3_OK\\]"
    "user.*process.*created"
    "ai-service.*Ring3"
    "user AI service scheduled"
    "PID.*running"
    "context.*switch"
)

RING3_SYSCALL_PATTERNS=(
    "\\[U\\]\\[SYSCALL_OK\\]"
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

timestamp() {
    date '+%H:%M:%S.%3N' 2>/dev/null || date '+%H:%M:%S'
}

write_log() {
    local message="$1"
    local level="${2:-INFO}"
    local timestamp
    timestamp="$(timestamp)"
    
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
    if ! grep -R --line-number -E 'GDT_USER_CODE.*0x23|GDT_USER_DATA.*0x1b' kernel/ >/dev/null 2>&1; then
        write_log "Ring3 GDT selector constants not found (GDT_USER_CODE/DATA)" "ERROR"
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
    local serial_arg="stdio"
    if [[ -n "${RING3_SERIAL_LOG:-}" ]]; then
        serial_arg="file:${RING3_SERIAL_LOG}"
    fi
    local debugcon_args=()
    if [[ -n "${RING3_DEBUGCON_LOG:-}" ]]; then
        debugcon_args=(
            "-chardev" "file,id=dbgcon,path=${RING3_DEBUGCON_LOG}"
            "-device" "isa-debugcon,iobase=0xe9,chardev=dbgcon"
        )
    fi


    # Optional UEFI firmware (OVMF) for deterministic boot
    local ovmf_args=()
    local ovmf_code=""
    local ovmf_vars=""
    local ovmf_vars_copy=""
    if [[ -f "OVMF_CODE.fd" && -f "OVMF_VARS.fd" ]]; then
        ovmf_code="OVMF_CODE.fd"
        ovmf_vars="OVMF_VARS.fd"
    elif [[ -f "firmware/ovmf/OVMF_CODE.fd" && -f "firmware/ovmf/OVMF_VARS.fd" ]]; then
        ovmf_code="firmware/ovmf/OVMF_CODE.fd"
        ovmf_vars="firmware/ovmf/OVMF_VARS.fd"
    elif [[ -f "/opt/homebrew/share/qemu/edk2-x86_64-code.fd" && -f "/opt/homebrew/share/qemu/edk2-x86_64-vars.fd" ]]; then
        ovmf_code="/opt/homebrew/share/qemu/edk2-x86_64-code.fd"
        ovmf_vars="/opt/homebrew/share/qemu/edk2-x86_64-vars.fd"
    fi
    if [[ -n "$ovmf_code" && -n "$ovmf_vars" ]]; then
        ovmf_vars_copy="ring3_ovmf_vars.fd"
        cp -f "$ovmf_vars" "$ovmf_vars_copy"
        ovmf_args=(
            "-machine" "q35"
            "-drive" "if=pflash,format=raw,readonly=on,file=${ovmf_code}"
            "-drive" "if=pflash,format=raw,file=${ovmf_vars_copy}"
        )
    fi
    local qemu_args=()
    if (( ${#ovmf_args[@]} > 0 )); then
        qemu_args+=("${ovmf_args[@]}")
    fi
    qemu_args+=(
        "-drive" "format=raw,file=EFI.img"
        "-serial" "$serial_arg"
        "-m" "512M"  # More memory for user processes
        "-no-reboot"
        "-no-shutdown"
        "-display" "none"
        "-d" "int,cpu_reset"  # Debug interrupts and CPU resets
        "-D" "qemu_debug.log"  # QEMU debug log
    )
    if (( ${#debugcon_args[@]} > 0 )); then
        qemu_args+=("${debugcon_args[@]}")
    fi
    
    write_log "QEMU command: qemu-system-x86_64 ${qemu_args[*]}" "DEBUG"
    
    # Start QEMU
    qemu-system-x86_64 "${qemu_args[@]}" > "$output_log" 2> "$error_log" &
    local qemu_pid=$!
    
    write_log "QEMU process started (PID: $qemu_pid)" "DEBUG"
    
    # Monitor execution with Ring3-specific analysis
    local start_time=$(date +%s)
    local last_output_size=0
    local last_debugcon_size=0
    local -a ring3_stages=()
    local init_detected=0
    local marker_process_detected=0
    local marker_syscall_detected=0
    local process_detected=0
    local syscall_detected=0
    local memory_detected=0
    local error_detected=0
    
    write_log "Monitoring Ring3 execution phases..." "INFO"
    
    while kill -0 "$qemu_pid" 2>/dev/null; do
        local current_time=$(date +%s)
        if (( current_time - start_time > TIMEOUT )); then
            write_log "Ring3 test timeout reached" "WARNING"
            break
        fi
        
        # Analyze output (serial + debugcon)
        local stream_log="$output_log"
        if [[ -n "${RING3_SERIAL_LOG:-}" ]]; then
            stream_log="${RING3_SERIAL_LOG}"
        fi
        local debugcon_log=""
        if [[ -n "${RING3_DEBUGCON_LOG:-}" ]]; then
            debugcon_log="${RING3_DEBUGCON_LOG}"
        fi
        local new_content=""
        local have_new=false

        if [[ -f "$stream_log" ]]; then
            local current_size=$(wc -c < "$stream_log" 2>/dev/null || echo 0)
            if (( current_size > last_output_size )); then
                new_content+=$(tail -c +$((last_output_size + 1)) "$stream_log" 2>/dev/null || echo "")
                last_output_size=$current_size
                have_new=true
            fi
        fi
        if [[ -n "$debugcon_log" && -f "$debugcon_log" ]]; then
            local current_dbg_size=$(wc -c < "$debugcon_log" 2>/dev/null || echo 0)
            if (( current_dbg_size > last_debugcon_size )); then
                if [[ "$have_new" == "true" ]]; then
                    new_content+=$'\n'
                fi
                new_content+=$(tail -c +$((last_debugcon_size + 1)) "$debugcon_log" 2>/dev/null || echo "")
                last_debugcon_size=$current_dbg_size
                have_new=true
            fi
        fi

        if [[ "$have_new" == "true" ]]; then
            # Check Ring3 initialization patterns
            for pattern in "${RING3_INIT_PATTERNS[@]}"; do
                if echo "$new_content" | grep -qE "$pattern"; then
                    match="$(echo "$new_content" | grep -m1 -E "$pattern" || true)"
                    write_log "Ring3 Init: $match" "SUCCESS"
                    ring3_stages+=("INIT: $match")
                    ((init_detected++))
                fi
            done

            # Check Ring3 process patterns
            for pattern in "${RING3_PROCESS_PATTERNS[@]}"; do
                if echo "$new_content" | grep -qE "$pattern"; then
                    match="$(echo "$new_content" | grep -m1 -E "$pattern" || true)"
                    write_log "Ring3 Process: $match" "SUCCESS"
                    ring3_stages+=("PROCESS: $match")
                    ((process_detected++))
                    if [[ "$pattern" == "\\[U\\]\\[RING3_OK\\]" ]]; then
                        ((marker_process_detected++))
                    fi
                fi
            done

            # Check Ring3 syscall patterns
            for pattern in "${RING3_SYSCALL_PATTERNS[@]}"; do
                if echo "$new_content" | grep -qE "$pattern"; then
                    match="$(echo "$new_content" | grep -m1 -E "$pattern" || true)"
                    write_log "Ring3 Syscall: $match" "SUCCESS"
                    ring3_stages+=("SYSCALL: $match")
                    ((syscall_detected++))
                    if [[ "$pattern" == "\\[U\\]\\[SYSCALL_OK\\]" ]]; then
                        ((marker_syscall_detected++))
                    fi
                fi
            done

            # Check Ring3 memory patterns
            for pattern in "${RING3_MEMORY_PATTERNS[@]}"; do
                if echo "$new_content" | grep -qE "$pattern"; then
                    match="$(echo "$new_content" | grep -m1 -E "$pattern" || true)"
                    write_log "Ring3 Memory: $match" "SUCCESS"
                    ring3_stages+=("MEMORY: $match")
                    ((memory_detected++))
                fi
            done

            # Check for errors
            if echo "$new_content" | grep -qE "PANIC|ERROR|FATAL|Triple fault|General Protection Fault"; then
                match="$(echo "$new_content" | grep -oE "PANIC|ERROR|FATAL|Triple fault|General Protection Fault" | head -n1)"
                write_log "Ring3 Error detected: $match" "ERROR"
                error_detected=1
                break
            fi

            # Verbose output
            if [[ "$VERBOSE" == "true" ]]; then
                while IFS= read -r line; do
                    if [[ -n "${line// }" ]]; then
                        echo -e "  ${GRAY}QEMU: $line${NC}"
                    fi
                done <<< "$new_content"
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
    
    # Ring3 validation criteria (marker-first):
    # - Canonical RING3_OK marker
    # - Canonical SYSCALL_OK marker
    # - At least 1 memory evidence
    # - No critical errors
    if (( marker_process_detected >= 1 && marker_syscall_detected >= 1 && memory_detected >= 1 && error_detected == 0 )); then
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
    "ring3_marker": $marker_process_detected,
    "syscall_marker": $marker_syscall_detected,
    "memory_patterns": $memory_detected,
    "total_detections": $total_detections,
    "error_detected": $([ "$error_detected" -eq 1 ] && echo "true" || echo "false"),
    "stages": [$(printf '"%s",' "${ring3_stages[@]:-}" | sed 's/,$//')]
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
    echo -e "  Canonical RING3_OK: $marker_process_detected (required: ≥1)"
    echo -e "  Canonical SYSCALL_OK: $marker_syscall_detected (required: ≥1)"
    echo -e "  Memory Management Patterns: $memory_detected"
    echo -e "  Errors Detected: $([ "$error_detected" -eq 1 ] && echo -e "${RED}Yes${NC}" || echo -e "${GREEN}No${NC}")"
    
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
    echo -e "  ✓ Canonical RING3_OK marker: $([ $marker_process_detected -ge 1 ] && echo -e "${GREEN}PASS${NC}" || echo -e "${RED}FAIL${NC}")"
    echo -e "  ✓ Canonical SYSCALL_OK marker: $([ $marker_syscall_detected -ge 1 ] && echo -e "${GREEN}PASS${NC}" || echo -e "${RED}FAIL${NC}")"
    echo -e "  ✓ User memory mapping evidence: $([ $memory_detected -ge 1 ] && echo -e "${GREEN}PASS${NC}" || echo -e "${RED}FAIL${NC}")"
    echo -e "  ✓ No critical errors: $([ "$error_detected" -eq 0 ] && echo -e "${GREEN}PASS${NC}" || echo -e "${RED}FAIL${NC}")"
    
    if [[ "$ring3_success" != "true" ]]; then
        SAVE_LOGS=true
    fi

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
    
    if [[ -n "$ovmf_vars_copy" ]]; then
        rm -f "$ovmf_vars_copy"
    fi
    
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
