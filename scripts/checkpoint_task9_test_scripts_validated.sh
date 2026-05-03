#!/usr/bin/env bash
# Checkpoint 9 — Test Scripts Validated
#
# Purpose:
#   Validate that test scripts produce checkpoint-grade evidence:
#   - no false PASS
#   - deterministic marker order
#   - non-empty debug log
#   - expected PASS markers
#   - expected negative-path behavior
#
# Maintainer: Kenan AY — System Architect

set -euo pipefail

EVIDENCE_DIR="${EVIDENCE_DIR:-out/evidence/task9}"
LOG_PATH="${LOG_PATH:-out/logs/debug_run.log}"
QEMU_TIMEOUT_SECONDS="${QEMU_TIMEOUT_SECONDS:-45}"

mkdir -p "$EVIDENCE_DIR"

RESULT_JSON="$EVIDENCE_DIR/result.json"
MARKERS_TXT="$EVIDENCE_DIR/markers.txt"
MARKER_HASH="$EVIDENCE_DIR/marker_hash.txt"

find_timeout_bin() {
    if [ -n "${TIMEOUT_BIN:-}" ]; then
        printf '%s\n' "$TIMEOUT_BIN"
        return 0
    fi

    if command -v timeout >/dev/null 2>&1; then
        command -v timeout
        return 0
    fi

    if command -v gtimeout >/dev/null 2>&1; then
        command -v gtimeout
        return 0
    fi

    if [ -x /opt/homebrew/bin/timeout ]; then
        printf '%s\n' /opt/homebrew/bin/timeout
        return 0
    fi

    return 1
}

fail() {
    reason="$1"

    cat > "$RESULT_JSON" <<EOF
{
  "checkpoint": "task9_test_scripts_validated",
  "status": "FAIL",
  "reason": "$reason",
  "maintainer": "Kenan AY"
}
EOF

    echo "❌ FAIL: $reason" >&2
    exit 1
}

log_has() {
    local pattern="$1"

    if [ ! -f "$LOG_PATH" ]; then
        return 1
    fi

    if command -v rg >/dev/null 2>&1; then
        rg -q "$pattern" "$LOG_PATH"
    else
        grep -Eq "$pattern" "$LOG_PATH"
    fi
}

line_of() {
    local pattern="$1"

    if [ ! -f "$LOG_PATH" ]; then
        echo 0
        return 0
    fi

    if command -v rg >/dev/null 2>&1; then
        rg -n "$pattern" "$LOG_PATH" | head -1 | cut -d: -f1 || echo 0
    else
        grep -En "$pattern" "$LOG_PATH" | head -1 | cut -d: -f1 || echo 0
    fi
}

assert_log_integrity() {
    [ -f "$LOG_PATH" ] || fail "debug log missing"
    [ -s "$LOG_PATH" ] || fail "debug log empty"

    local bytes
    bytes="$(wc -c < "$LOG_PATH" | tr -d ' ')"

    if [ "$bytes" -lt 128 ]; then
        fail "debug log too small: ${bytes} bytes"
    fi
}

assert_marker_present() {
    local marker="$1"

    if ! log_has "$marker"; then
        fail "required marker missing: $marker"
    fi
}

assert_marker_order() {
    local early late pass

    early="$(line_of "\\[K\\]\\[EARLY_BOOT_OK\\]")"
    late="$(line_of "\\[K\\]\\[LATE\\].*VCP_EVIDENCE_TESTS")"
    pass="$(line_of "VCP_EVIDENCE_TESTS PASSED")"

    [ "$early" -gt 0 ] || fail "EARLY_BOOT_OK marker missing"
    [ "$late" -gt 0 ] || fail "VCP late-init marker missing"
    [ "$pass" -gt 0 ] || fail "VCP PASS marker missing"

    if [ "$early" -ge "$late" ]; then
        fail "marker order invalid: EARLY_BOOT_OK must precede VCP late marker"
    fi

    if [ "$late" -ge "$pass" ]; then
        fail "marker order invalid: VCP late marker must precede PASS marker"
    fi
}

run_positive_path() {
    echo "== Task 9 Positive Path =="
    echo "Building validation image..."

    make clean >/dev/null 2>&1 || true

    make efi-img \
        KERNEL_PROFILE=validation \
        AYKEN_VALIDATION=1 \
        AYKEN_VCP_EVIDENCE_TEST=1 \
        AYKEN_VCP_FAIL_CLOSED_TEST=0 \
        AYKEN_VCP_RUNTIME_HOOK_TEST=0 \
        AYKEN_VCP_TRUST_VERIFICATION_TEST=0 \
        AYKEN_MB_SELFTEST=0 \
        AYKEN_GATE4_POLICY_TEST=0

    mkdir -p "$(dirname "$LOG_PATH")"
    : > "$LOG_PATH"

    local timeout_bin
    timeout_bin="$(find_timeout_bin)" || fail "timeout command not found"

    echo "Booting QEMU positive path..."

    set +e
    "$timeout_bin" "$QEMU_TIMEOUT_SECONDS" make run-qemu \
        KERNEL_PROFILE=validation \
        AYKEN_VALIDATION=1 \
        AYKEN_VCP_EVIDENCE_TEST=1 \
        AYKEN_VCP_FAIL_CLOSED_TEST=0 \
        AYKEN_VCP_RUNTIME_HOOK_TEST=0 \
        AYKEN_VCP_TRUST_VERIFICATION_TEST=0 \
        AYKEN_MB_SELFTEST=0 \
        AYKEN_GATE4_POLICY_TEST=0
    local run_status=$?
    set -e

    # Temporary rule:
    # 124 is tolerated only if PASS marker exists.
    # Task 9 hard closure should later replace this with clean kernel/QEMU exit.
    if [ "$run_status" -ne 0 ] && [ "$run_status" -ne 124 ]; then
        fail "QEMU failed before producing trusted evidence, exit=$run_status"
    fi

    assert_log_integrity

    assert_marker_present "\\[\\[AYKEN_BOOT_OK\\]\\]"
    assert_marker_present "\\[K\\]\\[EARLY_BOOT_OK\\]"
    assert_marker_present "VCP_EVIDENCE_TESTS PASSED"

    for marker in \
        "\\[VCP_EVIDENCE\\]\\[VALIDATION_CHECK\\]" \
        "\\[VCP_EVIDENCE\\]\\[CONTRACT_EXECUTION\\]" \
        "\\[VCP_EVIDENCE\\]\\[BOUNDARY_CROSSING\\]" \
        "\\[VCP_FAIL_CLOSED\\]\\[BLOCK\\]" \
        "\\[VCP_EVIDENCE\\]\\[COMPREHENSIVE\\]" \
        "\\[VCP_EVIDENCE\\]\\[FAIL_CLOSED_COMPLETE\\]"
    do
        assert_marker_present "$marker"
    done

    assert_marker_order

    grep -E "AYKEN_BOOT_OK|EARLY_BOOT_OK|VCP_EVIDENCE|VCP_FAIL_CLOSED|VCP_EVIDENCE_TESTS" "$LOG_PATH" > "$MARKERS_TXT"

    if command -v sha256sum >/dev/null 2>&1; then
        sha256sum "$MARKERS_TXT" | awk '{print $1}' > "$MARKER_HASH"
    else
        shasum -a 256 "$MARKERS_TXT" | awk '{print $1}' > "$MARKER_HASH"
    fi

    cp "$LOG_PATH" "$EVIDENCE_DIR/debug_run_positive.log"

    echo "✅ Positive path passed"
}

run_negative_path() {
    echo "== Task 9 Negative Path =="
    echo "Building validation image with VCP evidence test disabled..."

    make clean >/dev/null 2>&1 || true

    make efi-img \
        KERNEL_PROFILE=validation \
        AYKEN_VALIDATION=1 \
        AYKEN_VCP_EVIDENCE_TEST=0 \
        AYKEN_VCP_FAIL_CLOSED_TEST=0 \
        AYKEN_VCP_RUNTIME_HOOK_TEST=0 \
        AYKEN_VCP_TRUST_VERIFICATION_TEST=0 \
        AYKEN_MB_SELFTEST=0 \
        AYKEN_GATE4_POLICY_TEST=0

    mkdir -p "$(dirname "$LOG_PATH")"
    : > "$LOG_PATH"

    local timeout_bin
    timeout_bin="$(find_timeout_bin)" || fail "timeout command not found"

    echo "Booting QEMU negative path..."

    set +e
    "$timeout_bin" "$QEMU_TIMEOUT_SECONDS" make run-qemu \
        KERNEL_PROFILE=validation \
        AYKEN_VALIDATION=1 \
        AYKEN_VCP_EVIDENCE_TEST=0 \
        AYKEN_VCP_FAIL_CLOSED_TEST=0 \
        AYKEN_VCP_RUNTIME_HOOK_TEST=0 \
        AYKEN_VCP_TRUST_VERIFICATION_TEST=0 \
        AYKEN_MB_SELFTEST=0 \
        AYKEN_GATE4_POLICY_TEST=0
    local run_status=$?
    set -e

    if [ "$run_status" -ne 0 ] && [ "$run_status" -ne 124 ]; then
        fail "negative path QEMU failed unexpectedly, exit=$run_status"
    fi

    assert_log_integrity

    if log_has "VCP_EVIDENCE_TESTS PASSED"; then
        fail "false positive: PASS marker emitted while AYKEN_VCP_EVIDENCE_TEST=0"
    fi

    cp "$LOG_PATH" "$EVIDENCE_DIR/debug_run_negative.log"

    echo "✅ Negative path passed"
}

main() {
    echo "=========================================="
    echo "Checkpoint 9 — Test Scripts Validated"
    echo "=========================================="
    echo ""

    run_positive_path
    echo ""
    run_negative_path
    echo ""

    local hash_value
    hash_value="$(cat "$MARKER_HASH")"

    cat > "$RESULT_JSON" <<EOF
{
  "checkpoint": "task9_test_scripts_validated",
  "status": "PASS",
  "positive_path": true,
  "negative_path": true,
  "log_integrity": true,
  "marker_order": true,
  "marker_hash": "$hash_value",
  "clean_exit_required": false,
  "timeout_exit_tolerated": true,
  "note": "Timeout exit is tolerated only until controlled kernel/QEMU termination is implemented.",
  "maintainer": "Kenan AY"
}
EOF

    echo "✅ Checkpoint 9 PASS"
    echo ""
    echo "Evidence:"
    echo "  $EVIDENCE_DIR/debug_run_positive.log"
    echo "  $EVIDENCE_DIR/debug_run_negative.log"
    echo "  $MARKERS_TXT"
    echo "  $MARKER_HASH"
    echo "  $RESULT_JSON"
}

main "$@"
