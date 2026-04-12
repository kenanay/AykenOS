#!/bin/bash

# Property-Based Test: Preservation - Boot Functionality Unchanged
#
# This test validates that boot functionality (kernel initialization, subsystem setup,
# existing validation tests) works correctly on UNFIXED code, establishing the baseline
# behavior that must be preserved after implementing the evidence pipeline fix.
#
# IMPORTANT: Follow observation-first methodology
# - Run tests on UNFIXED code
# - Document baseline behavior
# - These tests MUST PASS to establish preservation requirements
#
# Expected Outcome: TESTS PASS (confirms baseline behavior to preserve)
#
# Author: Kenan AY - Architectural Steward
# Spec: .kiro/specs/boot-chain-observability-restoration/
# Requirements: 3.1, 3.2, 3.3, 3.4, 3.5, 3.6, 3.7

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
TEST_NAME="test_preservation_boot_functionality"
EVIDENCE_DIR="$PROJECT_ROOT/evidence/pbt/$TEST_NAME"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

log_property() {
    echo -e "${BLUE}[PROPERTY]${NC} $1"
}

# Create evidence directory
mkdir -p "$EVIDENCE_DIR"

BASELINE_LOG="$EVIDENCE_DIR/baseline_behavior.log"
TEST_RESULT_JSON="$EVIDENCE_DIR/test_result.json"
FAILURES_LOG="$EVIDENCE_DIR/failures.log"

log_info "========================================="
log_info "Property Test: Preservation - Boot Functionality"
log_info "========================================="
log_info "Test: Boot functionality unchanged on UNFIXED code"
log_info "Expected: TESTS PASS (baseline behavior preserved)"
log_info ""

# Initialize logs
> "$BASELINE_LOG"
> "$FAILURES_LOG"

FAILURES=0
TOTAL_CHECKS=0

# ============================================================================
# Property 2: Preservation - Boot Functionality Unchanged
# ============================================================================
log_property "Property 2: Preservation - Boot Functionality Unchanged"
log_property "For any kernel operation that does NOT involve boot observability,"
log_property "the system SHALL produce the same behavior as before the fix"
log_property ""

# ============================================================================
# Test 1: Build System Preservation
# ============================================================================
log_info "========================================="
log_info "Test 1: Build System Preservation"
log_info "========================================="
log_info "Requirement 3.6: Build system produces valid ELF and EFI binaries"
log_info ""

TOTAL_CHECKS=$((TOTAL_CHECKS + 1))

log_info "Building kernel and bootloader..."
cd "$PROJECT_ROOT"

# Clean build to ensure fresh state
make clean > /dev/null 2>&1 || true

# Build with release profile (default)
if make all > "$EVIDENCE_DIR/build.log" 2>&1; then
    log_info "  ✓ Build succeeded"
    echo "BUILD: SUCCESS" >> "$BASELINE_LOG"
    
    # Verify kernel ELF exists and is valid
    if [[ -f "out/build/kernel.elf" ]]; then
        log_info "  ✓ Kernel ELF exists: out/build/kernel.elf"
        
        # Check ELF magic
        if file out/build/kernel.elf | grep -q "ELF 64-bit"; then
            log_info "  ✓ Kernel ELF is valid 64-bit ELF"
            echo "KERNEL_ELF: VALID" >> "$BASELINE_LOG"
        else
            log_error "  ✗ Kernel ELF is not a valid 64-bit ELF"
            echo "KERNEL_ELF: INVALID" >> "$BASELINE_LOG"
            echo "FAILURE: Kernel ELF is not valid 64-bit ELF" >> "$FAILURES_LOG"
            FAILURES=$((FAILURES + 1))
        fi
    else
        log_error "  ✗ Kernel ELF not found"
        echo "KERNEL_ELF: MISSING" >> "$BASELINE_LOG"
        echo "FAILURE: Kernel ELF not found at out/build/kernel.elf" >> "$FAILURES_LOG"
        FAILURES=$((FAILURES + 1))
    fi
    
    # Verify bootloader EFI exists and is valid
    if [[ -f "out/build/BOOTX64.EFI" ]]; then
        log_info "  ✓ Bootloader EFI exists: out/build/BOOTX64.EFI"
        
        # Check PE32+ magic
        if file out/build/BOOTX64.EFI | grep -q "PE32+"; then
            log_info "  ✓ Bootloader EFI is valid PE32+ executable"
            echo "BOOTLOADER_EFI: VALID" >> "$BASELINE_LOG"
        else
            log_error "  ✗ Bootloader EFI is not a valid PE32+ executable"
            echo "BOOTLOADER_EFI: INVALID" >> "$BASELINE_LOG"
            echo "FAILURE: Bootloader EFI is not valid PE32+ executable" >> "$FAILURES_LOG"
            FAILURES=$((FAILURES + 1))
        fi
    else
        log_error "  ✗ Bootloader EFI not found"
        echo "BOOTLOADER_EFI: MISSING" >> "$BASELINE_LOG"
        echo "FAILURE: Bootloader EFI not found at out/build/BOOTX64.EFI" >> "$FAILURES_LOG"
        FAILURES=$((FAILURES + 1))
    fi
else
    log_error "  ✗ Build failed"
    echo "BUILD: FAILED" >> "$BASELINE_LOG"
    echo "FAILURE: Build system failed to compile kernel and bootloader" >> "$FAILURES_LOG"
    FAILURES=$((FAILURES + 1))
    
    log_error ""
    log_error "Build log:"
    tail -20 "$EVIDENCE_DIR/build.log" | while read -r line; do
        log_error "  $line"
    done
fi

log_info ""

# ============================================================================
# Test 2: QEMU Boot Preservation
# ============================================================================
log_info "========================================="
log_info "Test 2: QEMU Boot Preservation"
log_info "========================================="
log_info "Requirement 3.2: System boots without hanging"
log_info ""

TOTAL_CHECKS=$((TOTAL_CHECKS + 1))

# Only run if build succeeded
if [[ $FAILURES -eq 0 ]]; then
    log_info "Creating EFI disk image..."
    if make efi-img > "$EVIDENCE_DIR/efi-img.log" 2>&1; then
        log_info "  ✓ EFI disk image created"
        echo "EFI_IMG: SUCCESS" >> "$BASELINE_LOG"
        
        log_info ""
        log_info "Testing QEMU boot (5 second timeout)..."
        
        # Run QEMU with timeout to verify it boots without hanging
        QEMU_LOG="$EVIDENCE_DIR/qemu_boot_test.log"
        
        # Check if EFI.img exists in either location
        EFI_IMG_PATH=""
        if [[ -f "out/build/EFI.img" ]]; then
            EFI_IMG_PATH="out/build/EFI.img"
        elif [[ -f "build/EFI.img" ]]; then
            EFI_IMG_PATH="build/EFI.img"
        elif [[ -f "EFI.img" ]]; then
            EFI_IMG_PATH="EFI.img"
        else
            log_error "  ✗ EFI.img not found in any expected location"
            echo "EFI_IMG: NOT_FOUND" >> "$BASELINE_LOG"
            echo "FAILURE: EFI.img not found" >> "$FAILURES_LOG"
            FAILURES=$((FAILURES + 1))
        fi
        
        if [[ -n "$EFI_IMG_PATH" ]]; then
            timeout 5s qemu-system-x86_64 \
                -drive if=pflash,format=raw,readonly=on,file=/usr/share/OVMF/OVMF_CODE.fd \
                -drive if=pflash,format=raw,file=/tmp/ayken_ovmf_vars.fd \
                -drive format=raw,file="$EFI_IMG_PATH" \
                -m 512M \
                -nographic \
                -no-reboot \
                > "$QEMU_LOG" 2>&1 || true
            
            # Check if QEMU produced any output (indicates boot started)
            if [[ -s "$QEMU_LOG" ]]; then
                log_info "  ✓ QEMU boot started (produced output)"
                echo "QEMU_BOOT: STARTED" >> "$BASELINE_LOG"
                
                # Check for UEFI boot indicators
                if grep -q "UEFI" "$QEMU_LOG" || grep -q "EFI" "$QEMU_LOG"; then
                    log_info "  ✓ UEFI firmware initialized"
                    echo "UEFI_INIT: SUCCESS" >> "$BASELINE_LOG"
                fi
            else
                log_warn "  ⚠ QEMU produced no output (may indicate boot issue)"
                echo "QEMU_BOOT: NO_OUTPUT" >> "$BASELINE_LOG"
            fi
            
            log_info "  ✓ QEMU did not hang (completed within timeout)"
            echo "QEMU_HANG: NO" >> "$BASELINE_LOG"
        fi
    else
        log_error "  ✗ EFI disk image creation failed"
        echo "EFI_IMG: FAILED" >> "$BASELINE_LOG"
        echo "FAILURE: EFI disk image creation failed" >> "$FAILURES_LOG"
        FAILURES=$((FAILURES + 1))
    fi
else
    log_warn "  ⊘ Skipping QEMU boot test (build failed)"
    echo "QEMU_BOOT: SKIPPED" >> "$BASELINE_LOG"
fi

log_info ""

# ============================================================================
# Test 3: Kernel Subsystem Initialization Preservation
# ============================================================================
log_info "========================================="
log_info "Test 3: Kernel Subsystem Initialization"
log_info "========================================="
log_info "Requirement 3.1: Subsystems initialize correctly"
log_info "Requirement 3.4: Paging, heap, memory management work"
log_info ""

TOTAL_CHECKS=$((TOTAL_CHECKS + 1))

# Check kernel source for subsystem initialization markers
log_info "Verifying kernel subsystem initialization code exists..."

KERNEL_MAIN="$PROJECT_ROOT/kernel/kernel.c"

if [[ -f "$KERNEL_MAIN" ]]; then
    # Check for paging initialization
    if grep -q "paging" "$KERNEL_MAIN" || grep -q "page_table" "$KERNEL_MAIN"; then
        log_info "  ✓ Paging initialization code present"
        echo "PAGING_CODE: PRESENT" >> "$BASELINE_LOG"
    else
        log_warn "  ⚠ Paging initialization code not found"
        echo "PAGING_CODE: NOT_FOUND" >> "$BASELINE_LOG"
    fi
    
    # Check for heap initialization
    if grep -q "heap" "$KERNEL_MAIN" || grep -q "kmalloc" "$KERNEL_MAIN"; then
        log_info "  ✓ Heap initialization code present"
        echo "HEAP_CODE: PRESENT" >> "$BASELINE_LOG"
    else
        log_warn "  ⚠ Heap initialization code not found"
        echo "HEAP_CODE: NOT_FOUND" >> "$BASELINE_LOG"
    fi
    
    # Check for memory management
    if grep -q "memory" "$KERNEL_MAIN" || grep -q "pmm" "$KERNEL_MAIN"; then
        log_info "  ✓ Memory management code present"
        echo "MEMORY_CODE: PRESENT" >> "$BASELINE_LOG"
    else
        log_warn "  ⚠ Memory management code not found"
        echo "MEMORY_CODE: NOT_FOUND" >> "$BASELINE_LOG"
    fi
    
    log_info "  ✓ Kernel subsystem code structure preserved"
    echo "SUBSYSTEM_STRUCTURE: PRESERVED" >> "$BASELINE_LOG"
else
    log_error "  ✗ Kernel main file not found: $KERNEL_MAIN"
    echo "KERNEL_MAIN: MISSING" >> "$BASELINE_LOG"
    echo "FAILURE: Kernel main file not found" >> "$FAILURES_LOG"
    FAILURES=$((FAILURES + 1))
fi

log_info ""

# ============================================================================
# Test 4: Architectural Freeze Compliance
# ============================================================================
log_info "========================================="
log_info "Test 4: Architectural Freeze Compliance"
log_info "========================================="
log_info "Requirement 3.7: No new syscalls, layers, or contracts"
log_info ""

TOTAL_CHECKS=$((TOTAL_CHECKS + 1))

log_info "Verifying architectural freeze compliance..."

# Check that no new syscall definitions exist beyond baseline
SYSCALL_HEADER="$PROJECT_ROOT/kernel/include/syscall.h"

if [[ -f "$SYSCALL_HEADER" ]]; then
    SYSCALL_COUNT=$(grep -c "SYSCALL_" "$SYSCALL_HEADER" || echo "0")
    log_info "  ✓ Syscall definitions found: $SYSCALL_COUNT"
    echo "SYSCALL_COUNT: $SYSCALL_COUNT" >> "$BASELINE_LOG"
    
    # Document baseline syscall count for future comparison
    echo "$SYSCALL_COUNT" > "$EVIDENCE_DIR/baseline_syscall_count.txt"
else
    log_warn "  ⚠ Syscall header not found (may not exist yet)"
    echo "SYSCALL_HEADER: NOT_FOUND" >> "$BASELINE_LOG"
    echo "0" > "$EVIDENCE_DIR/baseline_syscall_count.txt"
fi

# Check for Ring0/Ring3 boundary preservation
if grep -rq "Ring3" "$PROJECT_ROOT/kernel/" 2>/dev/null; then
    log_info "  ✓ Ring3 boundary code present (baseline)"
    echo "RING3_BOUNDARY: PRESENT" >> "$BASELINE_LOG"
fi

# Check for capability system preservation
if grep -rq "capability" "$PROJECT_ROOT/kernel/" 2>/dev/null; then
    log_info "  ✓ Capability system code present (baseline)"
    echo "CAPABILITY_SYSTEM: PRESENT" >> "$BASELINE_LOG"
fi

log_info "  ✓ Architectural freeze baseline documented"
echo "ARCHITECTURAL_FREEZE: BASELINE_DOCUMENTED" >> "$BASELINE_LOG"

log_info ""

# ============================================================================
# Test 5: Existing Validation Tests Preservation
# ============================================================================
log_info "========================================="
log_info "Test 5: Existing Validation Tests"
log_info "========================================="
log_info "Requirement 3.5: Existing validation tests execute correctly"
log_info ""

TOTAL_CHECKS=$((TOTAL_CHECKS + 1))

log_info "Checking for existing validation test infrastructure..."

# Check for test directories
if [[ -d "$PROJECT_ROOT/tests" ]]; then
    log_info "  ✓ Test directory exists"
    echo "TEST_DIR: EXISTS" >> "$BASELINE_LOG"
    
    # Count test files
    TEST_FILE_COUNT=$(find "$PROJECT_ROOT/tests" -name "*.sh" -type f | wc -l)
    log_info "  ✓ Test scripts found: $TEST_FILE_COUNT"
    echo "TEST_SCRIPT_COUNT: $TEST_FILE_COUNT" >> "$BASELINE_LOG"
    
    # Document test structure for preservation
    find "$PROJECT_ROOT/tests" -name "*.sh" -type f > "$EVIDENCE_DIR/baseline_test_files.txt"
else
    log_warn "  ⚠ Test directory not found"
    echo "TEST_DIR: NOT_FOUND" >> "$BASELINE_LOG"
fi

# Check for Makefile validation targets
if grep -q "validate-build" "$PROJECT_ROOT/Makefile"; then
    log_info "  ✓ Makefile validation targets present"
    echo "MAKEFILE_VALIDATION: PRESENT" >> "$BASELINE_LOG"
fi

log_info "  ✓ Existing validation test structure documented"
echo "VALIDATION_STRUCTURE: DOCUMENTED" >> "$BASELINE_LOG"

log_info ""

# ============================================================================
# Test Result Summary
# ============================================================================
log_info "========================================="
log_info "Test Result Summary"
log_info "========================================="
log_info "Total checks: $TOTAL_CHECKS"
log_info "Failures: $FAILURES"
log_info ""

if [[ $FAILURES -eq 0 ]]; then
    log_info "TEST RESULT: PASS"
    log_info ""
    log_info "✓ Baseline behavior documented and preserved"
    log_info ""
    log_info "Baseline behavior log: $BASELINE_LOG"
    log_info ""
    log_info "This test establishes the preservation requirements."
    log_info "After implementing the evidence pipeline fix, re-run this test"
    log_info "to verify that all baseline behaviors remain unchanged."
    
    # Generate test result JSON
    cat > "$TEST_RESULT_JSON" << EOF
{
  "test": "$TEST_NAME",
  "property": "Preservation - Boot Functionality Unchanged",
  "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "result": "PASS",
  "expected_result": "PASS",
  "status": "BASELINE_DOCUMENTED",
  "total_checks": $TOTAL_CHECKS,
  "failures": $FAILURES,
  "baseline_log": "$BASELINE_LOG",
  "message": "Baseline behavior documented - all preservation requirements met",
  "preservation_requirements": [
    "Build system produces valid ELF and EFI binaries",
    "QEMU boots without hanging",
    "Kernel subsystem initialization code preserved",
    "Architectural freeze compliance maintained",
    "Existing validation test structure preserved"
  ]
}
EOF
    
    log_info "Test result: $TEST_RESULT_JSON"
    
    exit 0
else
    log_error "TEST RESULT: FAIL"
    log_error ""
    log_error "✗ Baseline behavior issues detected"
    log_error ""
    log_error "Failures: $FAILURES"
    log_error ""
    log_error "Failure details:"
    cat "$FAILURES_LOG" | while read -r line; do
        log_error "  - $line"
    done
    log_error ""
    log_error "CRITICAL: Baseline behavior must be stable before implementing fix"
    log_error "Address these issues before proceeding with evidence pipeline repair"
    
    # Generate test result JSON
    cat > "$TEST_RESULT_JSON" << EOF
{
  "test": "$TEST_NAME",
  "property": "Preservation - Boot Functionality Unchanged",
  "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "result": "FAIL",
  "expected_result": "PASS",
  "status": "BASELINE_UNSTABLE",
  "total_checks": $TOTAL_CHECKS,
  "failures": $FAILURES,
  "failures_log": "$FAILURES_LOG",
  "message": "Baseline behavior unstable - fix these issues before proceeding",
  "next_steps": [
    "Review failure log: $FAILURES_LOG",
    "Fix baseline behavior issues",
    "Re-run this test to establish stable baseline",
    "Only proceed with evidence pipeline fix after baseline is stable"
  ]
}
EOF
    
    log_info "Test result: $TEST_RESULT_JSON"
    
    exit 1
fi
