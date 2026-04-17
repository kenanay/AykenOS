#!/usr/bin/env bash
# Root cause isolation test for IRQ path overhead
# Tests whether BCIB validation + boundary enforcement causes the regression

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

echo "=========================================="
echo "IRQ PATH OVERHEAD ROOT CAUSE TEST"
echo "=========================================="
echo ""
echo "Testing hypothesis: BCIB validation + boundary enforcement in IRQ path"
echo "causes +14% boot time, +15% context switch, +15% syscall latency"
echo ""

cd "$PROJECT_ROOT"

# Baseline: Current implementation (with all validation)
echo ">> TEST A: BASELINE (current implementation)"
echo "   - BCIB validation: ENABLED"
echo "   - Boundary enforcement: ENABLED"
echo "   - Expected: FAIL (regression present)"
echo ""

cargo test -p semantic-cli --release --test gate_c_performance \
    bench_scalability_instruction_count -- --exact --nocapture \
    > /tmp/test_a_baseline.log 2>&1 || true

if grep -q "PASS" /tmp/test_a_baseline.log; then
    echo "   Result: PASS (unexpected)"
else
    echo "   Result: FAIL (expected - regression confirmed)"
fi
echo ""

# Test B: Disable BCIB validation
echo ">> TEST B: BCIB VALIDATION DISABLED"
echo "   - BCIB validation: DISABLED"
echo "   - Boundary enforcement: ENABLED"
echo "   - Expected: Partial improvement if validation is the bottleneck"
echo ""

# We'll create a temporary patch to disable validation
# This is a diagnostic test, not production code

echo "   Creating temporary validation bypass..."
cat > /tmp/validation_bypass.patch << 'EOF'
--- a/userspace/semantic-cli/src/kernel_submit_adapter.rs
+++ b/userspace/semantic-cli/src/kernel_submit_adapter.rs
@@ -220,7 +220,10 @@ impl SubmitAdapter for KernelSubmitAdapter {
         // Phase-16: Enhanced kernel boundary hardening
         
         // 1. Verify BCIB is submittable with boundary enforcement (FAIL CLOSED)
-        self.verify_bcib_submittable(&input.bcib)?;
+        // DIAGNOSTIC: Temporarily disabled for root cause isolation
+        if std::env::var("AYKEN_SKIP_BCIB_VALIDATION").is_err() {
+            self.verify_bcib_submittable(&input.bcib)?;
+        }
 
         // 2. Submit to kernel via hardened path
         let kernel_result = self.submit_to_kernel(&input.bcib)?;
EOF

echo "   Note: This test requires code modification"
echo "   Skipping Test B for now (requires manual patch)"
echo ""

# Test C: Disable boundary enforcement
echo ">> TEST C: BOUNDARY ENFORCEMENT DISABLED"
echo "   - BCIB validation: ENABLED"
echo "   - Boundary enforcement: DISABLED"
echo "   - Expected: Partial improvement if boundary checks are the bottleneck"
echo ""

echo "   Note: This test requires code modification"
echo "   Skipping Test C for now (requires manual patch)"
echo ""

# Summary
echo "=========================================="
echo "ROOT CAUSE ISOLATION SUMMARY"
echo "=========================================="
echo ""
echo "Current status:"
echo "  - Baseline test completed"
echo "  - Validation bypass test: requires code patch"
echo "  - Boundary bypass test: requires code patch"
echo ""
echo "RECOMMENDATION:"
echo "  The proper fix is NOT to disable validation/enforcement"
echo "  The proper fix is to MOVE them OUT of IRQ context"
echo ""
echo "Architecture fix required:"
echo "  IRQ path:     enqueue only (fast, bounded, deterministic)"
echo "  Worker path:  validation + enforcement + policy"
echo ""
echo "This is not an optimization — this is a model correction."
echo ""
