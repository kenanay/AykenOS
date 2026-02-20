#!/usr/bin/env bash
# AykenOS Syscall Roundtrip Testing
# Author: Kenan AY
# Purpose: Specialized testing for syscall interface and kernel-user transitions

set -e

# Configuration
TIMEOUT=50
VERBOSE=false
SAVE_LOGS=false
SYSCALL_CANONICAL_MARKER_USER="[U][SYSCALL_OK]"
SYSCALL_CANONICAL_MARKER_KERNEL="[[AYKEN_SYSCALL_V2_OK]]"

resolve_ovmf_firmware() {
    # Honor explicit overrides first.
    if [[ -n "${SYSCALL_OVMF_CODE:-}" && -n "${SYSCALL_OVMF_VARS:-}" \
          && -f "${SYSCALL_OVMF_CODE}" && -f "${SYSCALL_OVMF_VARS}" ]]; then
        printf "%s\n%s\n" "${SYSCALL_OVMF_CODE}" "${SYSCALL_OVMF_VARS}"
        return 0
    fi

    # Known firmware locations across macOS/Linux runners.
    local candidates=(
        # Prefer distro-managed OVMF on Linux/CI runners.
        "/usr/share/OVMF/OVMF_CODE_4M.fd|/usr/share/OVMF/OVMF_VARS_4M.fd"
        "/usr/share/OVMF/OVMF_CODE.fd|/usr/share/OVMF/OVMF_VARS.fd"
        "/usr/share/edk2/ovmf/OVMF_CODE.fd|/usr/share/edk2/ovmf/OVMF_VARS.fd"
        "/usr/share/qemu/OVMF_CODE.fd|/usr/share/qemu/OVMF_VARS.fd"
        # Fallbacks for repo-bundled/local firmware.
        "OVMF_CODE.fd|OVMF_VARS.fd"
        "firmware/ovmf/OVMF_CODE.fd|firmware/ovmf/OVMF_VARS.fd"
        "/opt/homebrew/share/qemu/edk2-x86_64-code.fd|/opt/homebrew/share/qemu/edk2-x86_64-vars.fd"
    )

    local entry code vars
    for entry in "${candidates[@]}"; do
        code="${entry%%|*}"
        vars="${entry##*|}"
        if [[ -f "${code}" && -f "${vars}" ]]; then
            printf "%s\n%s\n" "${code}" "${vars}"
            return 0
        fi
    done

    return 1
}

safe_count_re() {
    local pattern="$1"
    local content="$2"
    local count
    count=$(printf "%s" "$content" | grep -a -cE "$pattern" 2>/dev/null || true)
    count=$(printf "%s" "$count" | tr -dc '0-9')
    if [[ -z "$count" ]]; then
        count=0
    fi
    echo "$count"
}

safe_count_file() {
    local pattern="$1"
    local file="$2"
    local count
    count=$(grep -a -cE "$pattern" "$file" 2>/dev/null || true)
    count=$(printf "%s" "$count" | tr -dc '0-9')
    if [[ -z "$count" ]]; then
        count=0
    fi
    echo "$count"
}

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
    "\\[\\[AYKEN_SYSCALL_V2_OK\\]\\]"
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

terminate_process() {
    local pid="$1"
    local grace_seconds="${2:-5}"

    if ! kill -0 "$pid" 2>/dev/null; then
        return 0
    fi

    kill "$pid" 2>/dev/null || true
    local deadline=$(( $(date +%s) + grace_seconds ))
    while kill -0 "$pid" 2>/dev/null; do
        if (( $(date +%s) >= deadline )); then
            write_log "Process $pid did not terminate in ${grace_seconds}s; forcing kill" "WARNING"
            kill -9 "$pid" 2>/dev/null || true
            break
        fi
        sleep 0.1
    done

    wait "$pid" 2>/dev/null || true
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
    local syscall_count
    syscall_count=$(safe_count_re "syscall.*handler|SYS_.*called" "$output_content")
    if (( syscall_count > 0 )); then
        syscall_flow+=("HANDLER_INVOKED:$syscall_count")
        write_log "Syscall handler invoked $syscall_count times" "SUCCESS"
    fi
    
    # Track specific syscall types
    local write_syscalls
    local read_syscalls
    local exit_syscalls
    write_syscalls=$(safe_count_re "SYS_write" "$output_content")
    read_syscalls=$(safe_count_re "SYS_read" "$output_content")
    exit_syscalls=$(safe_count_re "SYS_exit|Process exit requested" "$output_content")
    
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
    local run_tmp_dir=""
    local qemu_int_trace="${SYSCALL_QEMU_INT_TRACE:-}"
    local qemu_accel="${SYSCALL_QEMU_ACCEL:-}"
    local qemu_smp="${SYSCALL_QEMU_SMP:-}"
    local qemu_debug_export="qemu_syscall_debug.log"
    local qemu_debug_log=""

    write_log "Starting comprehensive syscall roundtrip validation..." "INFO"

    # Keep INT trace enabled by default because runtime gate currently uses it
    # as fallback evidence source for syscall index verification.
    if [[ -z "$qemu_int_trace" ]]; then
        qemu_int_trace="1"
    fi
    if [[ "$qemu_int_trace" != "0" && "$qemu_int_trace" != "1" ]]; then
        write_log "SYSCALL_QEMU_INT_TRACE must be 0 or 1 (got: $qemu_int_trace)" "ERROR"
        return 1
    fi

    # Force deterministic software emulation in hosted CI unless overridden.
    if [[ -z "$qemu_accel" && "${CI:-}" == "true" ]]; then
        qemu_accel="tcg,thread=single"
    fi
    if [[ -z "$qemu_smp" && "${CI:-}" == "true" ]]; then
        qemu_smp="1"
    fi
    if [[ -n "$qemu_smp" ]] && ! [[ "$qemu_smp" =~ ^[1-9][0-9]*$ ]]; then
        write_log "SYSCALL_QEMU_SMP must be a positive integer (got: $qemu_smp)" "ERROR"
        return 1
    fi

    # Clean old logs
    rm -f "$output_log" "$error_log" "$analysis_log"
    
    # QEMU arguments optimized for syscall testing
    local serial_arg="stdio"
    if [[ -n "${SYSCALL_SERIAL_LOG:-}" ]]; then
        serial_arg="file:${SYSCALL_SERIAL_LOG}"
    fi
    local debugcon_args=()
    if [[ -n "${SYSCALL_DEBUGCON_LOG:-}" ]]; then
        debugcon_args=(
            "-chardev" "file,id=dbgcon,path=${SYSCALL_DEBUGCON_LOG}"
            "-device" "isa-debugcon,iobase=0xe9,chardev=dbgcon"
        )
    fi


    # Use a dedicated temp dir per run to avoid mktemp/template collisions.
    run_tmp_dir="$(mktemp -d "${TMPDIR:-/tmp}/syscall_run.XXXXXX" 2>/dev/null || mktemp -d -t syscall_run 2>/dev/null || true)"
    if [[ -z "$run_tmp_dir" || ! -d "$run_tmp_dir" ]]; then
        write_log "Failed to allocate temporary run directory" "ERROR"
        return 1
    fi
    qemu_debug_log="${run_tmp_dir}/qemu_syscall_debug.log"

    # UEFI firmware (OVMF) is required for EFI.img boot in CI/local validation.
    local ovmf_args=()
    local ovmf_code=""
    local ovmf_vars=""
    local ovmf_vars_copy=""
    local ovmf_pair=""
    ovmf_pair="$(resolve_ovmf_firmware || true)"
    if [[ -n "${ovmf_pair}" ]]; then
        ovmf_code="$(printf "%s\n" "${ovmf_pair}" | sed -n '1p')"
        ovmf_vars="$(printf "%s\n" "${ovmf_pair}" | sed -n '2p')"
    fi
    if [[ -z "$ovmf_code" || -z "$ovmf_vars" ]]; then
        if [[ "${CI:-}" == "true" ]]; then
            write_log "OVMF firmware not found in CI. Set SYSCALL_OVMF_CODE/SYSCALL_OVMF_VARS or install ovmf package." "ERROR"
            return 1
        fi
        write_log "OVMF firmware not found; falling back to non-UEFI boot path (local only)." "WARNING"
    else
        ovmf_vars_copy="${run_tmp_dir}/syscall_ovmf_vars.fd"
        cp -f "$ovmf_vars" "$ovmf_vars_copy"
        ovmf_args=(
            "-machine" "q35"
            "-drive" "if=pflash,format=raw,readonly=on,file=${ovmf_code}"
            "-drive" "if=pflash,format=raw,file=${ovmf_vars_copy}"
        )

        write_log "Using OVMF CODE: ${ovmf_code}" "INFO"
        write_log "Using OVMF VARS: ${ovmf_vars}" "INFO"
    fi
    local efi_img_source="EFI.img"
    if [[ ! -f "$efi_img_source" ]]; then
        write_log "EFI image missing, building via make efi-img" "WARNING"
        if ! make efi-img; then
            write_log "Failed to build EFI image (make efi-img)" "ERROR"
            return 1
        fi
    fi
    local efi_img_run="${run_tmp_dir}/EFI.img"
    cp -f "$efi_img_source" "$efi_img_run"
    if [[ ! -f "$efi_img_run" ]]; then
        write_log "Failed to prepare temporary EFI image copy" "ERROR"
        return 1
    fi
    rm -f "$qemu_debug_export"

    local qemu_args=()
    if (( ${#ovmf_args[@]} > 0 )); then
        qemu_args+=("${ovmf_args[@]}")
    fi
    qemu_args+=(
        # Use a per-run writable copy to avoid write-lock contention on EFI.img.
        "-drive" "format=raw,file=${efi_img_run}"
        "-serial" "$serial_arg"
        "-m" "256M"
        "-no-reboot"
        "-no-shutdown"
        "-display" "none"
    )
    if [[ -n "$qemu_accel" ]]; then
        qemu_args+=("-accel" "$qemu_accel")
    fi
    if [[ -n "$qemu_smp" ]]; then
        qemu_args+=("-smp" "$qemu_smp")
    fi
    if [[ "$qemu_int_trace" == "1" ]]; then
        qemu_args+=(
            "-d" "int"
            "-D" "$qemu_debug_log"
        )
    fi
    if (( ${#debugcon_args[@]} > 0 )); then
        qemu_args+=("${debugcon_args[@]}")
    fi

    write_log "QEMU INT trace: $([ "$qemu_int_trace" == "1" ] && echo "enabled" || echo "disabled")" "INFO"
    if [[ -n "$qemu_accel" ]]; then
        write_log "QEMU accel: ${qemu_accel}" "INFO"
    fi
    if [[ -n "$qemu_smp" ]]; then
        write_log "QEMU SMP: ${qemu_smp}" "INFO"
    fi

    write_log "QEMU command: qemu-system-x86_64 ${qemu_args[*]}" "INFO"
    
    # Start QEMU
    qemu-system-x86_64 "${qemu_args[@]}" > "$output_log" 2> "$error_log" &
    local qemu_pid=$!
    
    write_log "QEMU process started (PID: $qemu_pid)" "DEBUG"
    
    # Monitor execution with syscall-specific analysis
    local start_time=$(date +%s)
    local last_output_size=0
    local last_debugcon_size=0
    local syscall_stages=()
    local init_detected=0
    local handler_detected=0
    local execution_detected=0
    local user_detected=0
    local canonical_marker_detected=false
    local timed_out=false
    local error_detected=false
    local full_output=""
    local marker_tail=""
    
    write_log "Monitoring syscall interface and roundtrip execution..." "INFO"
    
    while kill -0 "$qemu_pid" 2>/dev/null; do
        local current_time=$(date +%s)
        if (( current_time - start_time > TIMEOUT )); then
            write_log "Syscall test timeout reached" "WARNING"
            timed_out=true
            break
        fi
        
        # Analyze output (serial + debugcon)
        local stream_log="$output_log"
        if [[ -n "${SYSCALL_SERIAL_LOG:-}" ]]; then
            stream_log="${SYSCALL_SERIAL_LOG}"
        fi
        local debugcon_log=""
        if [[ -n "${SYSCALL_DEBUGCON_LOG:-}" ]]; then
            debugcon_log="${SYSCALL_DEBUGCON_LOG}"
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
            full_output+="$new_content"
            local marker_probe="${marker_tail}${new_content}"
            if (( ${#marker_probe} > 64 )); then
                marker_tail="${marker_probe: -64}"
            else
                marker_tail="${marker_probe}"
            fi

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

            # Deterministic hosted-CI behavior: once canonical marker appears,
            # terminate QEMU instead of waiting for non-deterministic guest exit.
            if [[ "$canonical_marker_detected" == "false" ]] && \
               printf "%s" "$marker_probe" | grep -F -q -e "${SYSCALL_CANONICAL_MARKER_USER}" -e "${SYSCALL_CANONICAL_MARKER_KERNEL}"; then
                canonical_marker_detected=true
                write_log "Canonical marker detected; terminating QEMU deterministically" "SUCCESS"
                terminate_process "$qemu_pid" 5
                break
            fi
            
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
        
        sleep 0.5
    done
    
    # Cleanup QEMU
    if kill -0 "$qemu_pid" 2>/dev/null; then
        write_log "Terminating QEMU process..." "DEBUG"
        terminate_process "$qemu_pid" 5
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
    if [[ -f "$qemu_debug_log" ]]; then
        local int80_count
        int80_count=$(safe_count_file "int.*0x80|interrupt.*128" "$qemu_debug_log")
        if (( int80_count > 0 )); then
            interrupt_analysis="INT_0x80_TRIGGERED:$int80_count"
            write_log "INT 0x80 interrupts detected: $int80_count" "SUCCESS"
        fi
    fi

    # Export with canonical filename expected by audit wrappers.
    if [[ -f "$qemu_debug_log" ]]; then
        cp -f "$qemu_debug_log" "$qemu_debug_export"
    fi
    
    # Generate analysis report
    cat > "$analysis_log" << EOF
{
    "test_name": "$test_name",
    "duration": $duration,
    "success": $([ "$syscall_success" == "true" ] && echo "true" || echo "false"),
    "canonical_marker_detected": $([ "$canonical_marker_detected" == "true" ] && echo "true" || echo "false"),
    "timed_out": $([ "$timed_out" == "true" ] && echo "true" || echo "false"),
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
        if [[ -f "$qemu_debug_export" ]]; then
            echo -e "  QEMU Debug: $qemu_debug_export"
        fi
    else
        echo -e "  ${GRAY}Logs cleaned up (use --save-logs to preserve)${NC}"
        rm -f "$output_log" "$error_log" "$analysis_log" "$qemu_debug_export"
    fi
    
    echo -e "${CYAN}============================================================${NC}"
    
    rm -rf "$run_tmp_dir"
    
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
