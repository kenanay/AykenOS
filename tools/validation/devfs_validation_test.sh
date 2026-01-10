#!/usr/bin/env bash
# AykenOS DevFS Device I/O Operations Validation
# Author: Kenan AY
# Purpose: Specialized testing for DevFS device registration and I/O operations

set -e

# Configuration
TIMEOUT=45
VERBOSE=false
SAVE_LOGS=false

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
GRAY='\033[0;37m'
NC='\033[0m'

# DevFS specific test patterns
DEVFS_INIT_PATTERNS=(
    "devfs.*Initializing device filesystem"
    "VFS.*init"
    "devfs.*init"
)

DEVFS_STANDARD_DEVICES=(
    "devfs.*Registered.*null"
    "devfs.*Registered.*zero"
    "devfs.*Registered.*console"
)

DEVFS_EXTENDED_DEVICES=(
    "devfs.*Registered.*kbd"
    "devfs.*Registered.*ttyS0"
    "devfs.*Registered.*sda"
)

DEVFS_METADATA_PATTERNS=(
    "char.*device"
    "block.*device"
    "special.*device"
    "Primary storage device"
    "Keyboard input device"
    "Serial port"
)

DEVFS_VFS_INTEGRATION=(
    "VFS.*DevFS"
    "mount.*point"
    "device.*node"
    "file.*operations"
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
            echo "AykenOS DevFS Validation Test"
            echo ""
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --timeout N        Set timeout in seconds (default: 45)"
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

check_devfs_prerequisites() {
    write_log "Checking DevFS test prerequisites..." "INFO"
    
    # Check for DevFS source files
    if [[ ! -f "kernel/fs/devfs.c" ]]; then
        write_log "DevFS implementation file not found" "ERROR"
        return 1
    fi
    
    # Check for device registration functions
    if ! grep -q "devfs_register_device\|devfs_init" kernel/fs/devfs.c; then
        write_log "DevFS registration functions not found" "ERROR"
        return 1
    fi
    
    # Check for VFS integration
    if [[ ! -f "kernel/fs/vfs.c" ]]; then
        write_log "VFS implementation file not found" "ERROR"
        return 1
    fi
    
    # Check for device metadata structures
    if ! grep -q "device_metadata_t\|device_ops_t" kernel/include/devfs.h 2>/dev/null; then
        write_log "DevFS header structures not found" "WARNING"
    fi
    
    write_log "DevFS prerequisites check passed" "SUCCESS"
    return 0
}

analyze_device_coverage() {
    local output_content="$1"
    local device_coverage=()
    
    # Check for each required device
    local required_devices=("null" "zero" "console" "kbd" "ttyS0" "sda")
    local detected_devices=0
    
    for device in "${required_devices[@]}"; do
        if echo "$output_content" | grep -qE "devfs.*Registered.*$device"; then
            device_coverage+=("$device")
            ((detected_devices++))
            write_log "Device detected: /dev/$device" "SUCCESS"
        fi
    done
    
    # Check for device metadata
    local metadata_count=0
    local metadata_types=("char" "block" "special")
    
    for type in "${metadata_types[@]}"; do
        if echo "$output_content" | grep -qE "$type.*device"; then
            ((metadata_count++))
            write_log "Device type detected: $type" "DEBUG"
        fi
    done
    
    echo "$detected_devices:$metadata_count:${device_coverage[*]}"
}

run_devfs_validation() {
    local test_name="devfs_comprehensive"
    local output_log="${test_name}_output.log"
    local error_log="${test_name}_error.log"
    local analysis_log="${test_name}_analysis.log"
    
    write_log "Starting comprehensive DevFS validation..." "INFO"
    
    # Clean old logs
    rm -f "$output_log" "$error_log" "$analysis_log"
    
    # QEMU arguments optimized for DevFS testing
    local qemu_args=(
        "-drive" "format=raw,file=EFI.img"
        "-serial" "stdio"
        "-m" "256M"
        "-no-reboot"
        "-no-shutdown"
        "-display" "none"
    )
    
    write_log "QEMU command: qemu-system-x86_64 ${qemu_args[*]}" "DEBUG"
    
    # Start QEMU
    qemu-system-x86_64 "${qemu_args[@]}" > "$output_log" 2> "$error_log" &
    local qemu_pid=$!
    
    write_log "QEMU process started (PID: $qemu_pid)" "DEBUG"
    
    # Monitor execution with DevFS-specific analysis
    local start_time=$(date +%s)
    local last_output_size=0
    local devfs_stages=()
    local init_detected=0
    local standard_devices=0
    local extended_devices=0
    local metadata_detected=0
    local vfs_integration=0
    local error_detected=false
    local full_output=""
    
    write_log "Monitoring DevFS initialization and device registration..." "INFO"
    
    while kill -0 "$qemu_pid" 2>/dev/null; do
        local current_time=$(date +%s)
        if (( current_time - start_time > TIMEOUT )); then
            write_log "DevFS test timeout reached" "WARNING"
            break
        fi
        
        # Analyze output
        if [[ -f "$output_log" ]]; then
            local current_size=$(wc -c < "$output_log" 2>/dev/null || echo 0)
            if (( current_size > last_output_size )); then
                local new_content=$(tail -c +$((last_output_size + 1)) "$output_log" 2>/dev/null || echo "")
                last_output_size=$current_size
                full_output+="$new_content"
                
                # Check DevFS initialization patterns
                for pattern in "${DEVFS_INIT_PATTERNS[@]}"; do
                    if echo "$new_content" | grep -qE "$pattern"; then
                        local match=$(echo "$new_content" | grep -oE "$pattern" | head -n1)
                        write_log "DevFS Init: $match" "SUCCESS"
                        devfs_stages+=("INIT: $match")
                        ((init_detected++))
                    fi
                done
                
                # Check standard device patterns
                for pattern in "${DEVFS_STANDARD_DEVICES[@]}"; do
                    if echo "$new_content" | grep -qE "$pattern"; then
                        local match=$(echo "$new_content" | grep -oE "$pattern" | head -n1)
                        write_log "Standard Device: $match" "SUCCESS"
                        devfs_stages+=("STANDARD: $match")
                        ((standard_devices++))
                    fi
                done
                
                # Check extended device patterns
                for pattern in "${DEVFS_EXTENDED_DEVICES[@]}"; do
                    if echo "$new_content" | grep -qE "$pattern"; then
                        local match=$(echo "$new_content" | grep -oE "$pattern" | head -n1)
                        write_log "Extended Device: $match" "SUCCESS"
                        devfs_stages+=("EXTENDED: $match")
                        ((extended_devices++))
                    fi
                done
                
                # Check metadata patterns
                for pattern in "${DEVFS_METADATA_PATTERNS[@]}"; do
                    if echo "$new_content" | grep -qE "$pattern"; then
                        local match=$(echo "$new_content" | grep -oE "$pattern" | head -n1)
                        write_log "Device Metadata: $match" "SUCCESS"
                        devfs_stages+=("METADATA: $match")
                        ((metadata_detected++))
                    fi
                done
                
                # Check VFS integration patterns
                for pattern in "${DEVFS_VFS_INTEGRATION[@]}"; do
                    if echo "$new_content" | grep -qE "$pattern"; then
                        local match=$(echo "$new_content" | grep -oE "$pattern" | head -n1)
                        write_log "VFS Integration: $match" "SUCCESS"
                        devfs_stages+=("VFS: $match")
                        ((vfs_integration++))
                    fi
                done
                
                # Check for errors
                if echo "$new_content" | grep -qE "PANIC|ERROR|FATAL|devfs.*ERROR|Cannot register device"; then
                    local match=$(echo "$new_content" | grep -oE "PANIC|ERROR|FATAL|devfs.*ERROR|Cannot register device" | head -n1)
                    write_log "DevFS Error detected: $match" "ERROR"
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
    
    # Analyze device coverage
    local coverage_result=$(analyze_device_coverage "$full_output")
    IFS=':' read -r detected_devices metadata_count device_list <<< "$coverage_result"
    
    # DevFS validation criteria:
    # - DevFS initialization successful
    # - All 3 standard devices registered (/dev/null, /dev/zero, /dev/console)
    # - All 3 extended devices registered (/dev/kbd, /dev/ttyS0, /dev/sda)
    # - Device metadata present
    # - No critical errors
    local devfs_success=false
    if (( init_detected >= 1 && standard_devices >= 3 && extended_devices >= 3 && !error_detected )); then
        devfs_success=true
    fi
    
    # Generate analysis report
    cat > "$analysis_log" << EOF
{
    "test_name": "$test_name",
    "duration": $duration,
    "success": $([ "$devfs_success" == "true" ] && echo "true" || echo "false"),
    "init_detected": $init_detected,
    "standard_devices": $standard_devices,
    "extended_devices": $extended_devices,
    "metadata_detected": $metadata_detected,
    "vfs_integration": $vfs_integration,
    "total_devices_detected": $detected_devices,
    "device_coverage": "$device_list",
    "error_detected": $([ "$error_detected" == "true" ] && echo "true" || echo "false"),
    "stages": [$(printf '"%s",' "${devfs_stages[@]}" | sed 's/,$//')]
}
EOF
    
    # Generate detailed report
    echo ""
    echo -e "${CYAN}============================================================${NC}"
    echo -e "${CYAN}DEVFS VALIDATION TEST REPORT${NC}"
    echo -e "${CYAN}============================================================${NC}"
    echo ""
    echo -e "${NC}Test Results:${NC}"
    echo -e "  Status: $([ "$devfs_success" == "true" ] && echo -e "${GREEN}✓ PASS${NC}" || echo -e "${RED}✗ FAIL${NC}")"
    echo -e "  Duration: ${duration} seconds"
    echo -e "  Total Device Registrations: $detected_devices/6"
    echo ""
    echo -e "${NC}DevFS Component Analysis:${NC}"
    echo -e "  Initialization: $init_detected (required: ≥1)"
    echo -e "  Standard Devices: $standard_devices/3 (null, zero, console)"
    echo -e "  Extended Devices: $extended_devices/3 (kbd, ttyS0, sda)"
    echo -e "  Device Metadata: $metadata_detected"
    echo -e "  VFS Integration: $vfs_integration"
    echo -e "  Errors Detected: $([ "$error_detected" == "true" ] && echo -e "${RED}Yes${NC}" || echo -e "${GREEN}No${NC}")"
    
    if [[ -n "$device_list" ]]; then
        echo ""
        echo -e "${GREEN}Registered Devices:${NC}"
        for device in $device_list; do
            echo -e "  ${GREEN}/dev/$device${NC}"
        done
    fi
    
    if (( ${#devfs_stages[@]} > 0 )); then
        echo ""
        echo -e "${GREEN}DevFS Initialization Stages:${NC}"
        for stage in "${devfs_stages[@]}"; do
            echo -e "  ${GREEN}$stage${NC}"
        done
    fi
    
    echo ""
    echo -e "${CYAN}DevFS Validation Criteria:${NC}"
    echo -e "  ✓ DevFS initialization: $([ $init_detected -ge 1 ] && echo -e "${GREEN}PASS${NC}" || echo -e "${RED}FAIL${NC}")"
    echo -e "  ✓ Standard devices (3/3): $([ $standard_devices -ge 3 ] && echo -e "${GREEN}PASS${NC}" || echo -e "${RED}FAIL${NC}")"
    echo -e "  ✓ Extended devices (3/3): $([ $extended_devices -ge 3 ] && echo -e "${GREEN}PASS${NC}" || echo -e "${RED}FAIL${NC}")"
    echo -e "  ✓ No critical errors: $([ "$error_detected" == "false" ] && echo -e "${GREEN}PASS${NC}" || echo -e "${RED}FAIL${NC}")"
    
    echo ""
    echo -e "${CYAN}Device Requirements Coverage:${NC}"
    echo -e "  • /dev/null (data sink): $(echo "$device_list" | grep -q "null" && echo -e "${GREEN}✓${NC}" || echo -e "${RED}✗${NC}")"
    echo -e "  • /dev/zero (zero source): $(echo "$device_list" | grep -q "zero" && echo -e "${GREEN}✓${NC}" || echo -e "${RED}✗${NC}")"
    echo -e "  • /dev/console (system console): $(echo "$device_list" | grep -q "console" && echo -e "${GREEN}✓${NC}" || echo -e "${RED}✗${NC}")"
    echo -e "  • /dev/kbd (keyboard input): $(echo "$device_list" | grep -q "kbd" && echo -e "${GREEN}✓${NC}" || echo -e "${RED}✗${NC}")"
    echo -e "  • /dev/ttyS0 (serial port): $(echo "$device_list" | grep -q "ttyS0" && echo -e "${GREEN}✓${NC}" || echo -e "${RED}✗${NC}")"
    echo -e "  • /dev/sda (block storage): $(echo "$device_list" | grep -q "sda" && echo -e "${GREEN}✓${NC}" || echo -e "${RED}✗${NC}")"
    
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
    
    return $([ "$devfs_success" == "true" ] && echo 0 || echo 1)
}

# Main execution
echo -e "${GREEN}AykenOS DevFS Device I/O Operations Validation${NC}"
echo -e "${GRAY}Author: Kenan AY${NC}"
echo -e "${GRAY}Specialized DevFS Device Registration Test${NC}"
echo ""

if ! check_devfs_prerequisites; then
    exit 1
fi

if run_devfs_validation; then
    write_log "DevFS validation completed successfully" "SUCCESS"
    exit 0
else
    write_log "DevFS validation failed" "ERROR"
    exit 1
fi