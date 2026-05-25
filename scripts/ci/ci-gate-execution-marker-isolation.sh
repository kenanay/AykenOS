#!/usr/bin/env bash
# CI Gate: Execution Marker Validation Isolation
# Author: Kenan AY
# Role: Developer / Architect / Designer / Implementer
#
# PURPOSE:
# Enforce the Phase-17 feature-flag and test-harness isolation contract.
#
# RULES:
# - Production marker capture/guard integration is allowed only behind a
#   default-off AYKEN_EXECUTION_MARKER_VALIDATION_ENABLE switch.
# - Injection harness code is test-only and excluded from default builds.
# - Phase-specific coupling is allowed only in the guarded injection bridge.
# - Marker timestamps remain deterministic; no rdtsc() execution input.
# - Production code protection maintained

set -euo pipefail

RUN_ID="${RUN_ID:-$(date -u +%Y%m%dT%H%M%SZ)-$(git rev-parse --short HEAD)-$$}"
EVIDENCE_DIR="${EVIDENCE_DIR:-out/evidence/run-$RUN_ID/gates/execution-marker-isolation}"

mkdir -p "$EVIDENCE_DIR"

echo "== CI GATE EXECUTION MARKER ISOLATION =="
echo "run_id: $RUN_ID"

VIOLATIONS=()

# Check 1: production marker integration is guarded and default-off
echo ""
echo "Check 1: Production feature-flag guard"
if ! grep -Eq '^AYKEN_EXECUTION_MARKER_VALIDATION_ENABLE[[:space:]]*\?=[[:space:]]*0([[:space:]]|$)' Makefile; then
    echo "  ❌ FAIL: Marker validation default is not locked OFF"
    VIOLATIONS+=("feature_flag:default_not_off")
elif ! grep -q "#if AYKEN_EXECUTION_MARKER_VALIDATION_ENABLE" kernel/sys/execution_slot.c ||
     ! grep -q "#if AYKEN_EXECUTION_MARKER_VALIDATION_ENABLE" kernel/include/execution_slot.h; then
    echo "  ❌ FAIL: Production marker state is not feature-flag guarded"
    VIOLATIONS+=("feature_flag:guard_missing")
else
    echo "  ✅ PASS: Marker integration is guarded and default-off"
fi

# Check 2: injection harness remains test-only
echo ""
echo "Check 2: Injection harness build isolation"
if ! grep -q "kernel/sys/execution_marker_injection.c" Makefile ||
   ! grep -q 'ifeq ($(AYKEN_PHASE17_MARKER_INJECTION_TEST),1)' Makefile ||
   ! grep -Eq '^AYKEN_PHASE17_MARKER_INJECTION_TEST[[:space:]]*\?=[[:space:]]*0([[:space:]]|$)' Makefile ||
   ! grep -Eq '^AYKEN_MARKER_INJECT_INVALID_ORDER[[:space:]]*\?=[[:space:]]*0([[:space:]]|$)' Makefile ||
   ! grep -Eq '^AYKEN_EXECUTION_MARKER_NEGATIVE_EXPECT_REJECT[[:space:]]*\?=[[:space:]]*0([[:space:]]|$)' Makefile ||
   ! grep -q 'KERNEL_CFLAGS += -DAYKEN_PHASE17_MARKER_INJECTION_TEST=$(AYKEN_PHASE17_MARKER_INJECTION_TEST)' Makefile ||
   ! grep -q 'defined(AYKEN_PHASE17_MARKER_INJECTION_TEST).*== 1' kernel/sys/execution_slot.c; then
    echo "  ❌ FAIL: Injection harness default-build isolation contract missing"
    VIOLATIONS+=("injection_harness:isolation_missing")
else
    echo "  ✅ PASS: Injection/negative harness flags are compiled explicitly and default-off"
fi

# Check 3: public Ring3 E2E witness remains validation-only and isolated
echo ""
echo "Check 3: Public Ring3 E2E witness isolation"
if ! grep -Eq '^AYKEN_BCIB_PUBLIC_E2E_SELFTEST[[:space:]]*\?=[[:space:]]*0([[:space:]]|$)' Makefile ||
   ! grep -q 'AYKEN_BCIB_PUBLIC_E2E_SELFTEST=1 is only allowed with KERNEL_PROFILE=validation' Makefile ||
   ! grep -q 'AYKEN_BCIB_PUBLIC_E2E_SELFTEST=1 requires AYKEN_EXECUTION_MARKER_LIFECYCLE_SELFTEST=0' Makefile ||
   ! grep -q 'AYKEN_BCIB_PUBLIC_E2E_SELFTEST=1 requires AYKEN_PHASE17_MARKER_INJECTION_TEST=0' Makefile ||
   ! grep -q 'AYKEN_BCIB_PUBLIC_E2E_SELFTEST=1 requires AYKEN_RING3_ENTRY_GUARD=1' Makefile ||
   ! grep -q 'AYKEN_BCIB_PUBLIC_E2E_SELFTEST=1 requires AYKEN_RING3_MASK_IRQ0_FIRST_ENTRY=0' Makefile ||
   ! grep -q 'AYKEN_RING3_ENTRY_GUARD=1' scripts/ci/gate_execution_public_e2e.sh ||
   ! grep -q 'AYKEN_RING3_MASK_IRQ0_FIRST_ENTRY=0' scripts/ci/gate_execution_public_e2e.sh ||
   ! grep -q 'AYKEN_DEBUG_SCHED=0' scripts/ci/gate_execution_public_e2e.sh ||
   ! grep -q 'AYKEN_DEBUG_IRQ=0' scripts/ci/gate_execution_public_e2e.sh ||
   ! grep -q 'KERNEL_CFLAGS += -DAYKEN_BCIB_PUBLIC_E2E_SELFTEST=$(AYKEN_BCIB_PUBLIC_E2E_SELFTEST)' Makefile ||
   ! grep -q 'AYKEN_BCIB_PUBLIC_E2E_SELFTEST == 1' kernel/ring3_jump.c ||
   ! grep -q 'AYKEN_BCIB_PUBLIC_E2E_SELFTEST == 1' kernel/sys/syscall_v2.c; then
    echo "  ❌ FAIL: Public E2E witness is not isolated behind validation-only default-off guards"
    VIOLATIONS+=("public_e2e:isolation_missing")
else
    echo "  ✅ PASS: Public E2E witness is validation-only, default-off and separated from injection modes"
fi

# Check 4: Ring3 worker completion witness remains validation-only and stub-free
echo ""
echo "Check 4: Ring3 worker completion witness isolation"
if ! grep -Eq '^AYKEN_BCIB_WORKER_COMPLETION_SELFTEST[[:space:]]*\?=[[:space:]]*0([[:space:]]|$)' Makefile ||
   ! grep -q 'AYKEN_BCIB_WORKER_COMPLETION_SELFTEST=1 is only allowed with KERNEL_PROFILE=validation' Makefile ||
   ! grep -q 'AYKEN_BCIB_WORKER_COMPLETION_SELFTEST=1 requires AYKEN_BCIB_STUB_RESULT_ENABLE=0' Makefile ||
   ! grep -q 'AYKEN_BCIB_WORKER_COMPLETION_SELFTEST=1 requires AYKEN_BCIB_PUBLIC_E2E_SELFTEST=0' Makefile ||
   ! grep -q 'AYKEN_BCIB_WORKER_COMPLETION_SELFTEST=1 requires AYKEN_PHASE17_MARKER_INJECTION_TEST=0' Makefile ||
   ! grep -q 'AYKEN_BCIB_WORKER_COMPLETION_SELFTEST=1 requires AYKEN_RING3_ENTRY_GUARD=1' Makefile ||
   ! grep -q 'AYKEN_BCIB_WORKER_COMPLETION_SELFTEST=1 requires AYKEN_RING3_MASK_IRQ0_FIRST_ENTRY=0' Makefile ||
   ! grep -q 'AYKEN_BCIB_STUB_RESULT_ENABLE=0' scripts/ci/gate_execution_worker_completion.sh ||
   ! grep -q 'AYKEN_BCIB_WORKER_COMPLETION_SELFTEST=1' scripts/ci/gate_execution_worker_completion.sh ||
   ! grep -q 'KERNEL_CFLAGS += -DAYKEN_BCIB_WORKER_COMPLETION_SELFTEST=$(AYKEN_BCIB_WORKER_COMPLETION_SELFTEST)' Makefile ||
   ! grep -q 'AYKEN_BCIB_WORKER_COMPLETION_SELFTEST == 1' kernel/ring3_jump.c ||
   ! grep -q 'AYKEN_BCIB_WORKER_COMPLETION_SELFTEST == 1' kernel/sys/syscall_v2.c; then
    echo "  ❌ FAIL: Ring3 worker completion witness is not isolated behind validation-only stub-free guards"
    VIOLATIONS+=("worker_completion:isolation_missing")
else
    echo "  ✅ PASS: Ring3 worker completion witness is validation-only, default-off and stub-free"
fi

# Check 5: IRQ timeout race witness remains validation-only and stub-free
echo ""
echo "Check 5: IRQ timeout race witness isolation"
if ! grep -Eq '^AYKEN_EXECUTION_RACE_SELFTEST[[:space:]]*\?=[[:space:]]*0([[:space:]]|$)' Makefile ||
   ! grep -q 'AYKEN_EXECUTION_RACE_SELFTEST=1 is only allowed with KERNEL_PROFILE=validation' Makefile ||
   ! grep -q 'AYKEN_EXECUTION_RACE_SELFTEST=1 requires AYKEN_BCIB_STUB_RESULT_ENABLE=0' Makefile ||
   ! grep -q 'AYKEN_EXECUTION_RACE_SELFTEST=1 requires AYKEN_BCIB_WORKER_COMPLETION_SELFTEST=0' Makefile ||
   ! grep -q 'AYKEN_EXECUTION_RACE_SELFTEST=1 requires AYKEN_BCIB_PUBLIC_E2E_SELFTEST=0' Makefile ||
   ! grep -q 'AYKEN_EXECUTION_RACE_SELFTEST=1 requires AYKEN_PHASE17_MARKER_INJECTION_TEST=0' Makefile ||
   ! grep -q 'AYKEN_EXECUTION_RACE_SELFTEST=1 requires AYKEN_RING3_ENTRY_GUARD=1' Makefile ||
   ! grep -q 'AYKEN_EXECUTION_RACE_SELFTEST=1 requires AYKEN_RING3_MASK_IRQ0_FIRST_ENTRY=0' Makefile ||
   ! grep -q 'AYKEN_BCIB_STUB_RESULT_ENABLE=0' scripts/ci/gate_execution_timeout_race.sh ||
   ! grep -q 'AYKEN_EXECUTION_RACE_SELFTEST=1' scripts/ci/gate_execution_timeout_race.sh ||
   ! grep -q 'KERNEL_CFLAGS += -DAYKEN_EXECUTION_RACE_SELFTEST=$(AYKEN_EXECUTION_RACE_SELFTEST)' Makefile ||
   ! grep -q 'AYKEN_EXECUTION_RACE_SELFTEST == 1' kernel/ring3_jump.c ||
   ! grep -q 'AYKEN_EXECUTION_RACE_SELFTEST.*== 1' kernel/sched/sched.c ||
   ! grep -q 'AYKEN_EXEC_RACE_DEADLINE_ARMED' kernel/sched/sched.c ||
   ! grep -q 'AYKEN_EXECUTION_RACE_SELFTEST.*== 1' kernel/arch/x86_64/timer.c ||
   ! grep -q 'AYKEN_EXECUTION_RACE_SELFTEST.*== 1' kernel/sys/syscall_v2.c; then
    echo "  ❌ FAIL: IRQ timeout race witness is not isolated behind validation-only stub-free guards"
    VIOLATIONS+=("timeout_race:isolation_missing")
else
    echo "  ✅ PASS: IRQ timeout race witness is validation-only, default-off and stub-free"
fi

# Check 6: no unexpected phase-specific coupling outside approved guarded files
echo ""
echo "Check 6: Phase-specific coupling boundary"
PHASE17_LEAKS=$(grep -rE "phase17|PHASE17|Phase-17" kernel/ 2>/dev/null | \
    cut -d: -f1 | grep -E '\.(c|h)$' | sort -u | \
    grep -Ev '^kernel/+((sys/execution_slot\.c)|(sys/execution_marker_injection\.(c|h))|(include/execution_marker_validation\.h))$' || true)

if [ -n "$PHASE17_LEAKS" ]; then
    echo "  ❌ FAIL: Phase-specific coupling found outside approved guard files:"
    while IFS= read -r file; do
        [ -n "$file" ] || continue
        echo "    - $file"
        VIOLATIONS+=("phase17_naming:$file")
    done <<< "$PHASE17_LEAKS"
else
    echo "  ✅ PASS: Phase-specific references remain in approved guard files"
fi

# Check 7: execution_slot.c exists and protected
echo ""
echo "Check 7: Production code protection"
if [ ! -f "kernel/sys/execution_slot.c" ]; then
    echo "  ❌ FAIL: execution_slot.c missing"
    VIOLATIONS+=("execution_slot.c:missing")
else
    LINES=$(wc -l < kernel/sys/execution_slot.c)
    if [ "$LINES" -lt 1500 ]; then
        echo "  ❌ FAIL: execution_slot.c too small ($LINES lines)"
        VIOLATIONS+=("execution_slot.c:line_count_low:$LINES")
    else
        echo "  ✅ PASS: execution_slot.c protected ($LINES lines)"
    fi
fi

# Check 8: Marker validation contract files exist
echo ""
echo "Check 8: Marker contract files present"
if [ ! -f "kernel/include/execution_marker_validation.h" ]; then
    echo "  ❌ FAIL: Header missing"
    VIOLATIONS+=("header:missing")
else
    echo "  ✅ PASS: Header present"
fi

if [ ! -f "kernel/sys/execution_marker_validation.c" ]; then
    echo "  ❌ FAIL: Implementation missing"
    VIOLATIONS+=("implementation:missing")
else
    echo "  ✅ PASS: Implementation present"
fi

if [ ! -f "tests/unit/execution_marker_validation_test.c" ]; then
    echo "  ❌ FAIL: Test missing"
    VIOLATIONS+=("test:missing")
else
    echo "  ✅ PASS: Test present"
fi

# Check 9: marker evidence uses deterministic clock source only
echo ""
echo "Check 9: Marker timestamp determinism"
if grep -nE 'rdtsc[[:space:]]*\(' kernel/sys/execution_slot.c >/dev/null 2>&1; then
    echo "  ❌ FAIL: Nondeterministic rdtsc() use found in execution-slot path"
    VIOLATIONS+=("determinism:rdtsc_call")
elif ! grep -q "return timer_ticks();" kernel/sys/execution_slot.c; then
    echo "  ❌ FAIL: Logical timer source missing from execution-slot marker contract"
    VIOLATIONS+=("determinism:logical_tick_missing")
else
    echo "  ✅ PASS: Marker evidence uses logical ticks"
fi

# Generate report
VIOLATIONS_COUNT=${#VIOLATIONS[@]}

if [ "$VIOLATIONS_COUNT" -eq 0 ]; then
    VERDICT="PASS"
    VIOLATIONS_JSON="[]"
else
    VERDICT="FAIL"
    VIOLATIONS_JSON="[$(printf '"%s",' "${VIOLATIONS[@]}" | sed 's/,$//')]"
fi

cat > "$EVIDENCE_DIR/report.json" <<EOF
{
  "gate": "execution-marker-isolation",
  "verdict": "$VERDICT",
  "run_id": "$RUN_ID",
  "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "commit": "$(git rev-parse HEAD)",
  "violations": $VIOLATIONS_JSON,
  "violations_count": $VIOLATIONS_COUNT,
  "checks": {
    "feature_flag_default_off": "$(grep -Eq '^AYKEN_EXECUTION_MARKER_VALIDATION_ENABLE[[:space:]]*\?=[[:space:]]*0([[:space:]]|$)' Makefile && echo "pass" || echo "fail")",
    "injection_harness_isolation": "$(grep -q 'ifeq ($(AYKEN_PHASE17_MARKER_INJECTION_TEST),1)' Makefile && echo "pass" || echo "fail")",
    "public_e2e_witness_isolation": "$(grep -Eq '^AYKEN_BCIB_PUBLIC_E2E_SELFTEST[[:space:]]*\?=[[:space:]]*0([[:space:]]|$)' Makefile && grep -q 'AYKEN_BCIB_PUBLIC_E2E_SELFTEST=1 requires AYKEN_PHASE17_MARKER_INJECTION_TEST=0' Makefile && grep -q 'AYKEN_BCIB_PUBLIC_E2E_SELFTEST=1 requires AYKEN_RING3_ENTRY_GUARD=1' Makefile && grep -q 'AYKEN_RING3_ENTRY_GUARD=1' scripts/ci/gate_execution_public_e2e.sh && echo "pass" || echo "fail")",
    "worker_completion_witness_isolation": "$(grep -Eq '^AYKEN_BCIB_WORKER_COMPLETION_SELFTEST[[:space:]]*\?=[[:space:]]*0([[:space:]]|$)' Makefile && grep -q 'AYKEN_BCIB_WORKER_COMPLETION_SELFTEST=1 requires AYKEN_BCIB_STUB_RESULT_ENABLE=0' Makefile && grep -q 'AYKEN_BCIB_WORKER_COMPLETION_SELFTEST=1 requires AYKEN_BCIB_PUBLIC_E2E_SELFTEST=0' Makefile && grep -q 'AYKEN_BCIB_STUB_RESULT_ENABLE=0' scripts/ci/gate_execution_worker_completion.sh && echo "pass" || echo "fail")",
    "timeout_race_witness_isolation": "$(grep -Eq '^AYKEN_EXECUTION_RACE_SELFTEST[[:space:]]*\?=[[:space:]]*0([[:space:]]|$)' Makefile && grep -q 'AYKEN_EXECUTION_RACE_SELFTEST=1 requires AYKEN_BCIB_STUB_RESULT_ENABLE=0' Makefile && grep -q 'AYKEN_EXECUTION_RACE_SELFTEST=1 requires AYKEN_BCIB_WORKER_COMPLETION_SELFTEST=0' Makefile && grep -q 'AYKEN_BCIB_STUB_RESULT_ENABLE=0' scripts/ci/gate_execution_timeout_race.sh && echo "pass" || echo "fail")",
    "phase_specific_coupling_boundary": "$([ -z "$PHASE17_LEAKS" ] && echo "pass" || echo "fail")",
    "deterministic_marker_timestamp": "$(grep -q "return timer_ticks();" kernel/sys/execution_slot.c && echo "pass" || echo "fail")",
    "production_protected": "$([ -f kernel/sys/execution_slot.c ] && [ $(wc -l < kernel/sys/execution_slot.c) -ge 1500 ] && echo "pass" || echo "fail")",
    "sandbox_files_present": "$([ -f kernel/include/execution_marker_validation.h ] && [ -f kernel/sys/execution_marker_validation.c ] && [ -f tests/unit/execution_marker_validation_test.c ] && echo "pass" || echo "fail")"
  },
  "meta": {
    "purpose": "enforce_feature_flag_and_test_harness_isolation",
    "scope": "execution_marker_validation_guard"
  }
}
EOF

echo ""
if [ "$VERDICT" = "PASS" ]; then
    echo "✅ PASS: Execution Marker Isolation Gate"
    echo "Evidence: $EVIDENCE_DIR/report.json"
    exit 0
else
    echo "❌ FAIL: Execution Marker Isolation Gate"
    echo "Violations:"
    printf '  - %s\n' "${VIOLATIONS[@]}"
    echo "Evidence: $EVIDENCE_DIR/report.json"
    exit 1
fi
