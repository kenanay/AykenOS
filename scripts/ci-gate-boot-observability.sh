#!/bin/bash

# CI Gate: Boot Chain Observability Evidence Pipeline Integrity
#
# This gate validates that the boot chain observability evidence pipeline
# maintains integrity and prevents regressions in evidence capture.
#
# CRITICAL REQUIREMENTS:
#   - At least one output channel (debugcon OR serial) must capture markers
#   - Marker order must be preserved (NO sort, reorder, or temporal tampering)
#   - Channel-local analysis only (NO cross-channel merge creating fake ordering)
#   - Forbidden operations detected → FAIL
#
# Author: Kenan AY - Architectural Steward
# Spec: .kiro/specs/boot-chain-observability-restoration/

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
EVIDENCE_DIR="$PROJECT_ROOT/evidence/boot-observability"
GATE_NAME="ci-gate-boot-observability"

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

log_gate() {
    echo -e "${BLUE}[CI-GATE]${NC} $1"
}

# Create evidence directory
mkdir -p "$EVIDENCE_DIR"

EVIDENCE_FILE="$EVIDENCE_DIR/boot_observability_evidence.json"
VIOLATIONS_LOG="$EVIDENCE_DIR/violations.log"

log_info "Starting $GATE_NAME validation..."
log_info "Evidence requirement: Boot chain markers in preserved append-order"

# Initialize violations counter
VIOLATIONS=0
> "$VIOLATIONS_LOG"

# Expected trace files from harness
DEBUGCON_LOG="$EVIDENCE_DIR/qemu_debugcon.log"
SERIAL_LOG="$EVIDENCE_DIR/qemu_serial.log"
TRACE_DEBUGCON="$EVIDENCE_DIR/debugcon.trace"
TRACE_SERIAL="$EVIDENCE_DIR/serial.trace"

# ============================================================================
# CI GATE 1: Channel Integrity (HARD FAIL)
# ============================================================================
log_gate "Gate 1: Channel Integrity Validation"

DEBUGCON_SIZE=0
SERIAL_SIZE=0

if [[ -f "$DEBUGCON_LOG" ]]; then
    DEBUGCON_SIZE=$(stat -c%s "$DEBUGCON_LOG" 2>/dev/null || stat -f%z "$DEBUGCON_LOG" 2>/dev/null || echo "0")
fi

if [[ -f "$SERIAL_LOG" ]]; then
    SERIAL_SIZE=$(stat -c%s "$SERIAL_LOG" 2>/dev/null || stat -f%z "$SERIAL_LOG" 2>/dev/null || echo "0")
fi

log_info "Channel sizes: debugcon=$DEBUGCON_SIZE bytes, serial=$SERIAL_SIZE bytes"

# HARD FAIL: All channels zero
if [[ $DEBUGCON_SIZE -eq 0 ]] && [[ $SERIAL_SIZE -eq 0 ]]; then
    log_error "HARD FAIL: All output channels are empty (debugcon AND serial)"
    echo "VIOLATION: OUTPUT_CHANNEL_FAILURE - All channels zero bytes" >> "$VIOLATIONS_LOG"
    VIOLATIONS=$((VIOLATIONS + 1))
    
    cat > "$EVIDENCE_FILE" << EOF
{
  "gate": "$GATE_NAME",
  "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "result": "FAIL",
  "failure_code": "OUTPUT_CHANNEL_FAILURE",
  "violations_detected": $VIOLATIONS,
  "channel_integrity": {
    "debugcon_size": $DEBUGCON_SIZE,
    "serial_size": $SERIAL_SIZE,
    "at_least_one_channel_working": false
  },
  "message": "HARD FAIL: All output channels empty. Cannot proceed with validation."
}
EOF
    
    log_error "Evidence: $EVIDENCE_FILE"
    log_error "Violations: $VIOLATIONS_LOG"
    exit 1
fi

log_info "✓ Gate 1 PASS: At least one channel has output"

# ============================================================================
# CI GATE 2: Forbidden Operations Detection
# ============================================================================
log_gate "Gate 2: Forbidden Operations Detection"

# Check harness scripts for forbidden operations
HARNESS_SCRIPTS=(
    "$SCRIPT_DIR/qemu-fail-closed-proof-harness.sh"
    "$SCRIPT_DIR/qemu-runtime-bridge-proof-harness.sh"
)

FORBIDDEN_OPS_FOUND=0

for script in "${HARNESS_SCRIPTS[@]}"; do
    if [[ ! -f "$script" ]]; then
        continue
    fi
    
    log_info "Checking $script for forbidden operations..."
    
    # Check for 'sort' command (destroys temporal order)
    if grep -q "| sort" "$script" || grep -q "sort " "$script"; then
        log_error "FORBIDDEN: 'sort' operation detected in $script"
        echo "VIOLATION: FORBIDDEN_OPERATION - sort detected in $script" >> "$VIOLATIONS_LOG"
        VIOLATIONS=$((VIOLATIONS + 1))
        FORBIDDEN_OPS_FOUND=1
    fi
    
    # Check for 'uniq' command (can reorder or drop lines)
    if grep -q "| uniq" "$script" || grep -q "uniq " "$script"; then
        log_error "FORBIDDEN: 'uniq' operation detected in $script"
        echo "VIOLATION: FORBIDDEN_OPERATION - uniq detected in $script" >> "$VIOLATIONS_LOG"
        VIOLATIONS=$((VIOLATIONS + 1))
        FORBIDDEN_OPS_FOUND=1
    fi
    
    # Check for 'grep -o' (loses line context and order)
    if grep -q "grep -o" "$script"; then
        log_error "FORBIDDEN: 'grep -o' operation detected in $script"
        echo "VIOLATION: FORBIDDEN_OPERATION - grep -o detected in $script" >> "$VIOLATIONS_LOG"
        VIOLATIONS=$((VIOLATIONS + 1))
        FORBIDDEN_OPS_FOUND=1
    fi
    
    # Check for cross-channel concatenation creating fake temporal ordering
    # Pattern: cat debugcon serial (without clear separation or NON-AUTHORITATIVE warning)
    if grep -q "cat.*debugcon.*serial" "$script" && ! grep -q "NON-AUTHORITATIVE" "$script"; then
        log_warn "WARNING: Cross-channel concatenation detected in $script"
        log_warn "Ensure this is marked NON-AUTHORITATIVE and not used for temporal ordering"
    fi
done

if [[ $FORBIDDEN_OPS_FOUND -eq 0 ]]; then
    log_info "✓ Gate 2 PASS: No forbidden operations detected"
else
    log_error "✗ Gate 2 FAIL: Forbidden operations detected"
fi

# ============================================================================
# CI GATE 3: Required Boot Markers Present
# ============================================================================
log_gate "Gate 3: Required Boot Markers Validation"

# Required markers (at least one channel must have them)
# MANDATORY markers only - optional markers ([B][KERNEL_ELF_LOADED], [B][JUMP_NOW]) 
# are not enforced to allow flexibility in bootloader implementation
REQUIRED_MARKERS=(
    "\[B\]\[UEFI_BOOT_START\]"
    "\[\[AYKEN_BOOT_OK\]\]"
    "\[K\]\[EARLY_BOOT_OK\]"
)

MARKER_RESULTS=()

for marker in "${REQUIRED_MARKERS[@]}"; do
    FOUND_IN_DEBUGCON=0
    FOUND_IN_SERIAL=0
    
    if [[ -f "$TRACE_DEBUGCON" ]] && grep -q "$marker" "$TRACE_DEBUGCON"; then
        FOUND_IN_DEBUGCON=1
    fi
    
    if [[ -f "$TRACE_SERIAL" ]] && grep -q "$marker" "$TRACE_SERIAL"; then
        FOUND_IN_SERIAL=1
    fi
    
    if [[ $FOUND_IN_DEBUGCON -eq 1 ]] || [[ $FOUND_IN_SERIAL -eq 1 ]]; then
        log_info "✓ Marker found: $marker"
        MARKER_RESULTS+=("\"$marker\": true")
    else
        log_error "✗ Marker missing: $marker"
        echo "VIOLATION: MARKER_ABSENT - $marker not found in any channel" >> "$VIOLATIONS_LOG"
        VIOLATIONS=$((VIOLATIONS + 1))
        MARKER_RESULTS+=("\"$marker\": false")
    fi
done

# ============================================================================
# CI GATE 4: Marker Order Preservation
# ============================================================================
log_gate "Gate 4: Marker Order Preservation"

# Verify markers appear in correct order within each channel
# Expected canonical order (boot flow):
#   1. [B][UEFI_BOOT_START]     - Bootloader entry
#   2. [B][KERNEL_ELF_LOADED]   - ELF load complete (optional)
#   3. [B][JUMP_NOW]            - Before kernel jump (optional)
#   4. [[AYKEN_BOOT_OK]]        - Kernel entry stub (first kernel marker)
#   5. [K][EARLY_BOOT_OK]       - Early boot complete (kmain_real)
#
# CRITICAL: [[AYKEN_BOOT_OK]] appears BEFORE [K][EARLY_BOOT_OK]
# because [[AYKEN_BOOT_OK]] is emitted at entry stub (actual entry point)
# and [K][EARLY_BOOT_OK] is emitted later in kmain_real (C function)

ORDER_VALID=1

for trace_file in "$TRACE_DEBUGCON" "$TRACE_SERIAL"; do
    if [[ ! -f "$trace_file" ]]; then
        continue
    fi
    
    # Extract line numbers of all markers
    BOOT_START_LINE=$(grep -n "\[B\]\[UEFI_BOOT_START\]" "$trace_file" | head -1 | cut -d: -f1 || echo "0")
    ELF_LOADED_LINE=$(grep -n "\[B\]\[KERNEL_ELF_LOADED\]" "$trace_file" | head -1 | cut -d: -f1 || echo "0")
    JUMP_NOW_LINE=$(grep -n "\[B\]\[JUMP_NOW\]" "$trace_file" | head -1 | cut -d: -f1 || echo "0")
    BOOT_OK_LINE=$(grep -n "\[\[AYKEN_BOOT_OK\]\]" "$trace_file" | head -1 | cut -d: -f1 || echo "0")
    EARLY_BOOT_LINE=$(grep -n "\[K\]\[EARLY_BOOT_OK\]" "$trace_file" | head -1 | cut -d: -f1 || echo "0")
    
    # Verify mandatory order: [B][UEFI_BOOT_START] before [[AYKEN_BOOT_OK]]
    if [[ $BOOT_START_LINE -gt 0 ]] && [[ $BOOT_OK_LINE -gt 0 ]]; then
        if [[ $BOOT_START_LINE -lt $BOOT_OK_LINE ]]; then
            log_info "✓ Bootloader -> Kernel order preserved in $(basename "$trace_file")"
        else
            log_error "✗ Bootloader -> Kernel order broken in $(basename "$trace_file")"
            log_error "  [B][UEFI_BOOT_START] at line $BOOT_START_LINE"
            log_error "  [[AYKEN_BOOT_OK]] at line $BOOT_OK_LINE"
            echo "VIOLATION: MARKER_ORDER_BROKEN - Bootloader->Kernel order violated in $(basename "$trace_file")" >> "$VIOLATIONS_LOG"
            VIOLATIONS=$((VIOLATIONS + 1))
            ORDER_VALID=0
        fi
    fi
    
    # Verify kernel marker order: [[AYKEN_BOOT_OK]] before [K][EARLY_BOOT_OK]
    if [[ $BOOT_OK_LINE -gt 0 ]] && [[ $EARLY_BOOT_LINE -gt 0 ]]; then
        if [[ $BOOT_OK_LINE -lt $EARLY_BOOT_LINE ]]; then
            log_info "✓ Kernel entry -> Early boot order preserved in $(basename "$trace_file")"
        else
            log_error "✗ Kernel entry -> Early boot order broken in $(basename "$trace_file")"
            log_error "  [[AYKEN_BOOT_OK]] at line $BOOT_OK_LINE"
            log_error "  [K][EARLY_BOOT_OK] at line $EARLY_BOOT_LINE"
            echo "VIOLATION: MARKER_ORDER_BROKEN - Kernel marker order violated in $(basename "$trace_file")" >> "$VIOLATIONS_LOG"
            VIOLATIONS=$((VIOLATIONS + 1))
            ORDER_VALID=0
        fi
    fi
    
    # Verify optional bootloader markers if present
    if [[ $ELF_LOADED_LINE -gt 0 ]] && [[ $BOOT_START_LINE -gt 0 ]]; then
        if [[ $BOOT_START_LINE -lt $ELF_LOADED_LINE ]]; then
            log_info "✓ Optional: [B][UEFI_BOOT_START] before [B][KERNEL_ELF_LOADED]"
        else
            log_warn "⚠ Optional marker order issue: ELF_LOADED before BOOT_START"
        fi
    fi
    
    if [[ $JUMP_NOW_LINE -gt 0 ]] && [[ $ELF_LOADED_LINE -gt 0 ]]; then
        if [[ $ELF_LOADED_LINE -lt $JUMP_NOW_LINE ]]; then
            log_info "✓ Optional: [B][KERNEL_ELF_LOADED] before [B][JUMP_NOW]"
        else
            log_warn "⚠ Optional marker order issue: JUMP_NOW before ELF_LOADED"
        fi
    fi
done

if [[ $ORDER_VALID -eq 1 ]]; then
    log_info "✓ Gate 4 PASS: Marker order preserved"
else
    log_error "✗ Gate 4 FAIL: Marker order broken"
fi

# ============================================================================
# Generate Evidence JSON
# ============================================================================

if [[ $VIOLATIONS -eq 0 ]]; then
    RESULT="PASS"
    FAILURE_CODE="NONE"
else
    RESULT="FAIL"
    FAILURE_CODE="EVIDENCE_PIPELINE_INTEGRITY_VIOLATION"
fi

cat > "$EVIDENCE_FILE" << EOF
{
  "gate": "$GATE_NAME",
  "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "result": "$RESULT",
  "failure_code": "$FAILURE_CODE",
  "violations_detected": $VIOLATIONS,
  "channel_integrity": {
    "debugcon_size": $DEBUGCON_SIZE,
    "serial_size": $SERIAL_SIZE,
    "at_least_one_channel_working": true
  },
  "forbidden_operations": {
    "detected": $FORBIDDEN_OPS_FOUND
  },
  "required_markers": {
    $(IFS=,; echo "${MARKER_RESULTS[*]}")
  },
  "marker_order": {
    "preserved": $ORDER_VALID
  },
  "violations_log": "$VIOLATIONS_LOG"
}
EOF

# ============================================================================
# Final Result
# ============================================================================

if [[ "$RESULT" == "PASS" ]]; then
    log_info ""
    log_info "========================================="
    log_info "$GATE_NAME: PASS"
    log_info "========================================="
    log_info "Evidence: $EVIDENCE_FILE"
    log_info "Boot chain observability evidence pipeline integrity validated"
    exit 0
else
    log_error ""
    log_error "========================================="
    log_error "$GATE_NAME: FAIL"
    log_error "========================================="
    log_error "Violations: $VIOLATIONS"
    log_error "Evidence: $EVIDENCE_FILE"
    log_error "Violations log: $VIOLATIONS_LOG"
    log_error ""
    log_error "CRITICAL: Evidence pipeline integrity compromised"
    log_error "Review violations and fix before proceeding"
    exit 1
fi
