#!/usr/bin/env bash
# Phase 16 Performance Regression RCA - Feature Isolation Measurement Script
# 
# This script measures boot_time with different Phase 16 feature configurations
# to identify which feature(s) contribute to the +1,506ms regression.
#
# Usage: ./measure_phase16_overhead.sh [--iterations N] [--output-dir DIR]

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
ITERATIONS="${1:-3}"
OUTPUT_DIR="${2:-${ROOT}/.kiro/specs/phase16-performance-regression-rca/measurements}"
KERNEL_PROFILE="validation"
QEMU_TIMEOUT="30"

# Create output directory
mkdir -p "${OUTPUT_DIR}"

# Timestamp for this measurement run
TIMESTAMP="$(date +%Y%m%d_%H%M%S)"
RUN_DIR="${OUTPUT_DIR}/run_${TIMESTAMP}"
mkdir -p "${RUN_DIR}"

echo "========================================="
echo "Phase 16 Performance Regression RCA"
echo "Feature Isolation Measurement"
echo "========================================="
echo "Iterations: ${ITERATIONS}"
echo "Output: ${RUN_DIR}"
echo ""

# Helper function to measure boot_time
measure_boot_time() {
    local config_name="$1"
    local bcib_worker="$2"
    local boundary_enforcement="$3"
    local probe_validation="$4"
    local diagnostic_markers="$5"
    
    echo "----------------------------------------"
    echo "Configuration: ${config_name}"
    echo "  BCIB_WORKER_ENABLE=${bcib_worker}"
    echo "  BOUNDARY_ENFORCEMENT_ENABLE=${boundary_enforcement}"
    echo "  PROBE_VALIDATION_ENABLE=${probe_validation}"
    echo "  DIAGNOSTIC_MARKERS_ENABLE=${diagnostic_markers}"
    echo "----------------------------------------"
    
    local config_dir="${RUN_DIR}/${config_name}"
    mkdir -p "${config_dir}"
    
    # Clean and rebuild with specific feature toggles
    echo "[${config_name}] Cleaning build artifacts..."
    make -C "${ROOT}" clean > "${config_dir}/clean.log" 2>&1 || true
    
    echo "[${config_name}] Building kernel with feature toggles..."
    if ! make -C "${ROOT}" \
        KERNEL_PROFILE="${KERNEL_PROFILE}" \
        USER_MINIMAL_MODE="syscall-v2-runtime" \
        AYKEN_PHASE16_BCIB_WORKER_ENABLE="${bcib_worker}" \
        AYKEN_PHASE16_BOUNDARY_ENFORCEMENT_ENABLE="${boundary_enforcement}" \
        AYKEN_PHASE16_PROBE_VALIDATION_ENABLE="${probe_validation}" \
        AYKEN_PHASE16_DIAGNOSTIC_MARKERS_ENABLE="${diagnostic_markers}" \
        efi-img > "${config_dir}/build.log" 2>&1; then
        echo "[${config_name}] BUILD FAILED - see ${config_dir}/build.log"
        return 1
    fi
    
    # Run multiple iterations
    local total_time=0
    local iteration_times=()
    
    for i in $(seq 1 "${ITERATIONS}"); do
        echo "[${config_name}] Iteration ${i}/${ITERATIONS}..."
        
        local iter_dir="${config_dir}/iter_${i}"
        mkdir -p "${iter_dir}"
        
        # Measure boot time using phase_4_4_qemu_boot_audit.sh
        local start_ms=$(date +%s%3N)
        
        if (cd "${ROOT}" && tools/validation/phase_4_4_qemu_boot_audit.sh \
            --timeout "${QEMU_TIMEOUT}" \
            --marker "[K][BOOT_OK] Phase 4.4 minimal boot reached" \
            --out-dir "${iter_dir}") > "${iter_dir}/boot_audit.log" 2>&1; then
            
            local end_ms=$(date +%s%3N)
            local boot_time=$((end_ms - start_ms))
            
            echo "[${config_name}] Iteration ${i}: ${boot_time}ms"
            echo "${boot_time}" >> "${config_dir}/boot_times.txt"
            iteration_times+=("${boot_time}")
            total_time=$((total_time + boot_time))
        else
            echo "[${config_name}] Iteration ${i}: BOOT FAILED"
            echo "FAILED" >> "${config_dir}/boot_times.txt"
        fi
    done
    
    # Calculate mean and stddev
    if [ ${#iteration_times[@]} -gt 0 ]; then
        local mean=$((total_time / ${#iteration_times[@]}))
        
        # Calculate standard deviation
        local sum_sq_diff=0
        for time in "${iteration_times[@]}"; do
            local diff=$((time - mean))
            sum_sq_diff=$((sum_sq_diff + diff * diff))
        done
        local variance=$((sum_sq_diff / ${#iteration_times[@]}))
        local stddev=$(echo "scale=2; sqrt(${variance})" | bc)
        
        echo "[${config_name}] Mean: ${mean}ms, StdDev: ${stddev}ms"
        echo "mean=${mean}" >> "${config_dir}/summary.txt"
        echo "stddev=${stddev}" >> "${config_dir}/summary.txt"
        echo "iterations=${#iteration_times[@]}" >> "${config_dir}/summary.txt"
    else
        echo "[${config_name}] All iterations failed"
        echo "mean=FAILED" >> "${config_dir}/summary.txt"
    fi
    
    echo ""
}

# Configuration 1: All Phase 16 features enabled (baseline regression measurement)
measure_boot_time "config1_all_enabled" 1 1 1 1

# Configuration 2: BCIB worker disabled only
measure_boot_time "config2_bcib_worker_disabled" 0 1 1 1

# Configuration 3: Boundary enforcement disabled only
measure_boot_time "config3_boundary_disabled" 1 0 1 1

# Configuration 4: Probe validation disabled only
measure_boot_time "config4_probe_disabled" 1 1 0 1

# Configuration 5: Diagnostic markers disabled only
measure_boot_time "config5_markers_disabled" 1 1 1 0

# Configuration 6: All Phase 16 features disabled (verify return to baseline)
measure_boot_time "config6_all_disabled" 0 0 0 0

# Generate summary report
echo "========================================="
echo "Measurement Summary"
echo "========================================="
echo ""

SUMMARY_FILE="${RUN_DIR}/SUMMARY.md"
cat > "${SUMMARY_FILE}" <<EOF
# Phase 16 Performance Regression RCA - Measurement Results

**Timestamp**: ${TIMESTAMP}
**Iterations per configuration**: ${ITERATIONS}
**Kernel Profile**: ${KERNEL_PROFILE}

## Configuration Results

| Configuration | BCIB Worker | Boundary | Probe | Markers | Mean Boot Time (ms) | StdDev (ms) |
|--------------|-------------|----------|-------|---------|---------------------|-------------|
EOF

for config_dir in "${RUN_DIR}"/config*; do
    if [ -f "${config_dir}/summary.txt" ]; then
        config_name=$(basename "${config_dir}")
        mean=$(grep "^mean=" "${config_dir}/summary.txt" | cut -d= -f2)
        stddev=$(grep "^stddev=" "${config_dir}/summary.txt" | cut -d= -f2)
        
        # Extract feature flags from config name
        case "${config_name}" in
            *all_enabled*)
                flags="✓|✓|✓|✓"
                ;;
            *bcib_worker_disabled*)
                flags="✗|✓|✓|✓"
                ;;
            *boundary_disabled*)
                flags="✓|✗|✓|✓"
                ;;
            *probe_disabled*)
                flags="✓|✓|✗|✓"
                ;;
            *markers_disabled*)
                flags="✓|✓|✓|✗"
                ;;
            *all_disabled*)
                flags="✗|✗|✗|✗"
                ;;
            *)
                flags="?|?|?|?"
                ;;
        esac
        
        echo "| ${config_name} | ${flags} | ${mean} | ${stddev} |" >> "${SUMMARY_FILE}"
    fi
done

cat >> "${SUMMARY_FILE}" <<EOF

## Analysis

Compare the mean boot times to identify which feature(s) contribute to the regression:

1. **Baseline (all enabled)**: Expected ~12,190ms (current regression)
2. **All disabled**: Expected ~10,684ms (return to baseline)
3. **Individual feature disabled**: Reduction indicates that feature's overhead

### Overhead Calculation

For each feature, calculate overhead as:
\`\`\`
Overhead = (All Enabled Boot Time) - (Feature Disabled Boot Time)
\`\`\`

### Expected Outcome

The measurements will identify which Phase 16 feature(s) cause the +1,506ms regression.
This will guide targeted optimization efforts in Phase 2 of the RCA.

## Raw Data

- Full logs: \`${RUN_DIR}\`
- Individual iteration times: \`config*/boot_times.txt\`
- Build logs: \`config*/build.log\`
- Boot audit logs: \`config*/iter_*/boot_audit.log\`
EOF

cat "${SUMMARY_FILE}"

echo ""
echo "========================================="
echo "Measurement complete!"
echo "Results saved to: ${RUN_DIR}"
echo "Summary: ${SUMMARY_FILE}"
echo "========================================="
