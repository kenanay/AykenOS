#!/usr/bin/env bash
# AykenOS Toolchain and QEMU Validation Script
# Author: Kenan AY
# Purpose: Automated toolchain detection and QEMU boot validation for Linux/WSL

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
source "${ROOT}/tools/lib/ayken_path_contract.sh"
cd "${ROOT}"
ayken_prepare_out_dirs

# Default parameters
SKIP_QEMU=false
VERBOSE=false
QEMU_TIMEOUT=30

# Color output functions
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
GRAY='\033[0;37m'
NC='\033[0m' # No Color

success() { echo -e "${GREEN}✓ $1${NC}"; }
error() { echo -e "${RED}✗ $1${NC}"; }
warning() { echo -e "${YELLOW}⚠ $1${NC}"; }
info() { echo -e "${CYAN}ℹ $1${NC}"; }

# Validation results
TOOLCHAIN_VALID=false
QEMU_VALID=false
BUILD_VALID=false
ERRORS=()
WARNINGS=()

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --skip-qemu)
            SKIP_QEMU=true
            shift
            ;;
        --verbose)
            VERBOSE=true
            shift
            ;;
        --qemu-timeout)
            QEMU_TIMEOUT="$2"
            shift 2
            ;;
        --help)
            echo "AykenOS Toolchain & QEMU Validation Script"
            echo ""
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --skip-qemu        Skip QEMU boot validation"
            echo "  --verbose          Enable verbose output"
            echo "  --qemu-timeout N   Set QEMU timeout in seconds (default: 30)"
            echo "  --help             Show this help message"
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            exit 1
            ;;
    esac
done

command_exists() {
    command -v "$1" >/dev/null 2>&1
}

detect_timeout_cmd() {
    if command_exists "timeout"; then
        echo "timeout"
    elif command_exists "gtimeout"; then
        echo "gtimeout"
    else
        echo ""
    fi
}

get_command_version() {
    local cmd="$1"
    local version_arg="${2:---version}"
    
    if command_exists "$cmd"; then
        $cmd $version_arg 2>/dev/null | head -n1 || echo "Version unknown"
    else
        echo "Not found"
    fi
}

test_toolchain() {
    info "Validating toolchain components..."
    
    local tools=(
        "x86_64-elf-gcc:true:Cross-compiler for kernel"
        "x86_64-elf-ld:true:Cross-linker for kernel"
        "clang:true:UEFI bootloader compiler"
        "nasm:true:Assembly compiler"
        "make:true:Build system"
        "qemu-system-x86_64:false:Emulator for testing"
    )
    
    local all_required=true
    
    for tool_info in "${tools[@]}"; do
        IFS=':' read -r tool_name required description <<< "$tool_info"
        
        if command_exists "$tool_name"; then
            local version=$(get_command_version "$tool_name")
            success "$tool_name found - $version"
            if [[ "$VERBOSE" == "true" ]]; then
                echo -e "  ${GRAY}Description: $description${NC}"
            fi
        else
            if [[ "$required" == "true" ]]; then
                error "$tool_name not found - $description"
                ERRORS+=("Missing required tool: $tool_name")
                all_required=false
            else
                warning "$tool_name not found - $description"
                WARNINGS+=("Missing optional tool: $tool_name")
            fi
        fi
    done
    
    # Check package manager suggestions
    if [[ "$all_required" == "false" ]]; then
        info "Installation suggestions:"
        
        if command_exists "apt"; then
            echo -e "  ${GRAY}Ubuntu/Debian: sudo apt install gcc-multilib nasm clang make qemu-system-x86${NC}"
        fi
        
        if command_exists "yum"; then
            echo -e "  ${GRAY}RHEL/CentOS: sudo yum install gcc nasm clang make qemu-system-x86${NC}"
        fi
        
        if command_exists "pacman"; then
            echo -e "  ${GRAY}Arch Linux: sudo pacman -S gcc nasm clang make qemu${NC}"
        fi
        
        echo -e "  ${GRAY}Or build cross-compiler from source (see BUILD_FIXES.md)${NC}"
    fi
    
    TOOLCHAIN_VALID=$all_required
    return $([ "$all_required" == "true" ] && echo 0 || echo 1)
}

test_build_system() {
    info "Testing build system..."
    
    # Check required files
    local required_files=("Makefile" "linker.ld" "kernel/kernel.c" "bootloader/efi/efi_main.c")
    local files_ok=true
    
    for file in "${required_files[@]}"; do
        if [[ -f "$file" ]]; then
            success "Found: $file"
        else
            error "Missing: $file"
            ERRORS+=("Missing required file: $file")
            files_ok=false
        fi
    done
    
    if [[ "$files_ok" == "false" ]]; then
        return 1
    fi
    
    # Test make clean (do not delete EFI.img)
    info "Testing make clean (no EFI.img)..."
    if make clean-noimg >/dev/null 2>&1; then
        success "make clean-noimg successful"
    else
        warning "make clean-noimg returned non-zero exit code"
    fi
    
    # Test make all
    info "Testing make all..."
    local build_output
    if build_output=$(make all 2>&1); then
        success "make all successful"
        
        # Check output files
        if [[ -f "${AYKEN_KERNEL_ELF}" && -f "${AYKEN_BOOT_EFI}" ]]; then
            success "Build artifacts created successfully"
            BUILD_VALID=true
            return 0
        else
            error "Build completed but artifacts missing"
            ERRORS+=("Build artifacts not created")
            return 1
        fi
    else
        error "make all failed"
        ERRORS+=("Build failed")
        if [[ "$VERBOSE" == "true" ]]; then
            echo -e "${GRAY}Build output:${NC}"
            echo -e "${GRAY}$build_output${NC}"
        fi
        return 1
    fi
}

test_qemu_boot() {
    if [[ "$SKIP_QEMU" == "true" ]]; then
        info "Skipping QEMU validation (--skip-qemu specified)"
        return 0
    fi
    
    if ! command_exists "qemu-system-x86_64"; then
        warning "QEMU not found, skipping boot validation"
        WARNINGS+=("QEMU not available for boot testing")
        return 0
    fi
    
    info "Testing QEMU boot validation..."
    
    # Create EFI image if needed (deterministic via Makefile guard)
    if [[ ! -f "${AYKEN_EFI_IMG}" ]]; then
        info "Creating EFI image..."
        if command_exists "make"; then
            make efi-img
        else
            error "Cannot create EFI image - 'make' not available"
            return 1
        fi
    fi
    
    # Run QEMU with timeout
    info "Starting QEMU boot test (timeout: ${QEMU_TIMEOUT}s)..."
    
    mkdir -p "${AYKEN_LOG_DIR}"
    local qemu_output="${AYKEN_LOG_DIR}/validate_qemu_output.log"
    local qemu_error="${AYKEN_LOG_DIR}/validate_qemu_error.log"
    local dbgcon_log="${AYKEN_LOG_DIR}/validate_qemu_debugcon.log"
    local qemu_trace="${AYKEN_LOG_DIR}/validate_qemu_trace.log"
    rm -f "$qemu_output" "$qemu_error" "$dbgcon_log" "$qemu_trace"
    local timeout_cmd
    timeout_cmd="$(detect_timeout_cmd)"
    if [[ -z "$timeout_cmd" ]]; then
        warning "No timeout command found; using manual time limit for QEMU"
        WARNINGS+=("Missing timeout command (using manual QEMU time limit)")
    fi

    # Optional UEFI firmware (OVMF) for deterministic boot
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

    local ovmf_args=()
    if [[ -n "$ovmf_code" && -n "$ovmf_vars" ]]; then
        ovmf_vars_copy="${AYKEN_OVMF_VARS_RUN}"
        mkdir -p "$(dirname "${ovmf_vars_copy}")"
        cp -f "$ovmf_vars" "$ovmf_vars_copy"
        ovmf_args+=(
            -machine pc
            -drive "if=pflash,format=raw,readonly=on,file=${ovmf_code}"
            -drive "if=pflash,format=raw,file=${ovmf_vars_copy}"
        )
    fi
    
    # Start QEMU in background
    if [[ -n "$timeout_cmd" ]]; then
        "$timeout_cmd" "$QEMU_TIMEOUT" qemu-system-x86_64 \
            "${ovmf_args[@]}" \
            -drive format=raw,file="${AYKEN_EFI_IMG}" \
            -serial file:"$qemu_output" \
            -chardev file,id=dbgcon,path="$dbgcon_log" \
            -device isa-debugcon,iobase=0xe9,chardev=dbgcon \
            -display none \
            -no-reboot \
            -no-shutdown \
            -d guest_errors -D "$qemu_trace" \
            > /dev/null 2> "$qemu_error" &
    else
        qemu-system-x86_64 \
            "${ovmf_args[@]}" \
            -drive format=raw,file="${AYKEN_EFI_IMG}" \
            -serial file:"$qemu_output" \
            -chardev file,id=dbgcon,path="$dbgcon_log" \
            -device isa-debugcon,iobase=0xe9,chardev=dbgcon \
            -display none \
            -no-reboot \
            -no-shutdown \
            -d guest_errors -D "$qemu_trace" \
            > /dev/null 2> "$qemu_error" &
    fi
    
    local qemu_pid=$!
    local boot_success=false
    local start_time=$(date +%s)
    
    # Monitor for boot success
    while kill -0 $qemu_pid 2>/dev/null; do
        local current_time=$(date +%s)
        if (( current_time - start_time > QEMU_TIMEOUT )); then
            break
        fi
        
        # Check for boot success indicators
        if [[ -f "$qemu_output" || -f "$dbgcon_log" ]]; then
            if grep -q -E "\\[K\\]\\[QEMU_BOOT_OK\\]" "$qemu_output" 2>/dev/null \
               || grep -q -E "\\[K\\]\\[QEMU_BOOT_OK\\]" "$dbgcon_log" 2>/dev/null; then
                boot_success=true
                success "Boot success detected in QEMU output"
                break
            fi
        fi
        
        sleep 0.5
    done
    
    # Clean shutdown
    if kill -0 $qemu_pid 2>/dev/null; then
        kill $qemu_pid 2>/dev/null || true
        wait $qemu_pid 2>/dev/null || true
    fi
    
    if [[ "$boot_success" == "true" ]]; then
        success "QEMU boot validation passed"
        QEMU_VALID=true
        
        if [[ "$VERBOSE" == "true" && -f "$qemu_output" ]]; then
            info "QEMU output:"
            sed 's/^/  /' "$qemu_output"
        fi
    else
        warning "QEMU boot validation inconclusive (no clear success indicators)"
        WARNINGS+=("QEMU boot validation inconclusive")
        
        if [[ -f "$qemu_error" && -s "$qemu_error" ]]; then
            warning "QEMU errors detected:"
            sed 's/^/  /' "$qemu_error"
        fi
        if [[ -f "$dbgcon_log" && -s "$dbgcon_log" ]]; then
            warning "Debugcon output present but boot marker not found"
            WARNINGS+=("Debugcon output present; missing boot marker")
        fi
    fi
    
    # Cleanup (preserve logs when verbose for audit/debug)
    if [[ "$VERBOSE" != "true" ]]; then
        rm -f "$qemu_output" "$qemu_error" "$dbgcon_log" "$qemu_trace"
        if [[ -n "$ovmf_vars_copy" ]]; then
            rm -f "$ovmf_vars_copy"
        fi
    fi
    
    return $([ "$boot_success" == "true" ] && echo 0 || echo 1)
}

write_validation_report() {
    echo ""
    echo -e "${CYAN}============================================================${NC}"
    echo -e "${CYAN}AykenOS Validation Report${NC}"
    echo -e "${CYAN}============================================================${NC}"
    
    echo ""
    echo -e "${NC}Validation Results:${NC}"
    echo -e "  Toolchain: $([ "$TOOLCHAIN_VALID" == "true" ] && echo -e "${GREEN}✓ PASS${NC}" || echo -e "${RED}✗ FAIL${NC}")"
    echo -e "  Build System: $([ "$BUILD_VALID" == "true" ] && echo -e "${GREEN}✓ PASS${NC}" || echo -e "${RED}✗ FAIL${NC}")"
    echo -e "  QEMU Boot: $([ "$QEMU_VALID" == "true" ] && echo -e "${GREEN}✓ PASS${NC}" || echo -e "${YELLOW}⚠ SKIP/WARN${NC}")"
    
    if [[ ${#ERRORS[@]} -gt 0 ]]; then
        echo ""
        echo -e "${RED}Errors:${NC}"
        for error in "${ERRORS[@]}"; do
            echo -e "  ${RED}• $error${NC}"
        done
    fi
    
    if [[ ${#WARNINGS[@]} -gt 0 ]]; then
        echo ""
        echo -e "${YELLOW}Warnings:${NC}"
        for warning in "${WARNINGS[@]}"; do
            echo -e "  ${YELLOW}• $warning${NC}"
        done
    fi
    
    local overall_success=false
    if [[ "$TOOLCHAIN_VALID" == "true" && "$BUILD_VALID" == "true" ]]; then
        overall_success=true
    fi
    
    echo ""
    echo -e "Overall Status: $([ "$overall_success" == "true" ] && echo -e "${GREEN}✓ READY FOR DEVELOPMENT${NC}" || echo -e "${RED}✗ SETUP REQUIRED${NC}")"
    
    if [[ "$overall_success" == "false" ]]; then
        echo ""
        echo -e "${CYAN}Next Steps:${NC}"
        echo -e "  ${NC}1. Install missing tools (see BUILD_FIXES.md)${NC}"
        echo -e "  ${NC}2. Build cross-compiler if needed${NC}"
        echo -e "  ${NC}3. Run validation again: ./validate_toolchain.sh${NC}"
    else
        echo ""
        echo -e "${GREEN}Ready to develop! Try:${NC}"
        echo -e "  ${NC}make clean && make all && make run${NC}"
    fi
    
    echo -e "${CYAN}============================================================${NC}"
}

# Main execution
echo -e "${GREEN}AykenOS Toolchain & QEMU Validation${NC}"
echo -e "${GRAY}Author: Kenan AY${NC}"
echo ""

if test_toolchain; then
    if test_build_system; then
        test_qemu_boot || true
    fi
fi

write_validation_report

# Exit with appropriate code
if [[ "$TOOLCHAIN_VALID" == "true" && "$BUILD_VALID" == "true" ]]; then
    exit 0
else
    exit 1
fi
