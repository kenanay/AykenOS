#!/usr/bin/env bash
# CI Gate: BCIB Stub Determinism (v1 - Phase-16)
#
# Scope: Validates BCIB stub infrastructure is buildable and ready
# - Kernel builds successfully with AYKEN_BCIB_STUB_RESULT_ENABLE=1
# - BCIB stub code compiles without errors
# - Stub completion path exists in kernel binary
#
# Does NOT validate:
# - Runtime determinism (requires bcib_worker, Phase-17)
# - Full BCIB pipeline markers (Phase-17 backlog)
# - Userspace worker flow
#
# Authority: Phase-16 closure requirement
# Next: Phase-17 will extend to runtime validation

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

# Default configuration
BCIB_KERNEL_PROFILE="${BCIB_KERNEL_PROFILE:-validation}"
EVIDENCE_DIR=""

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --evidence-dir)
            EVIDENCE_DIR="$2"
            shift 2
            ;;
        *)
            echo "ERROR: Unknown argument: $1" >&2
            exit 1
            ;;
    esac
done

if [[ -z "$EVIDENCE_DIR" ]]; then
    echo "ERROR: --evidence-dir required" >&2
    exit 1
fi

mkdir -p "$EVIDENCE_DIR"

# Validate kernel profile
if [[ "$BCIB_KERNEL_PROFILE" != "validation" ]]; then
    echo "ERROR: BCIB_KERNEL_PROFILE must be 'validation' (got: $BCIB_KERNEL_PROFILE)" >&2
    exit 1
fi

echo "bcib-stub-determinism: kernel_profile=$BCIB_KERNEL_PROFILE mode=build_validation"

# Clean and build kernel with stub mode enabled
echo "bcib-stub-determinism: clean build with AYKEN_BCIB_STUB_RESULT_ENABLE=1..."
make -C "$PROJECT_ROOT" clean > "$EVIDENCE_DIR/clean.log" 2>&1 || {
    echo "ERROR: make clean failed" >&2
    tail -50 "$EVIDENCE_DIR/clean.log" >&2
    exit 1
}

make -C "$PROJECT_ROOT" kernel \
    KERNEL_PROFILE="$BCIB_KERNEL_PROFILE" \
    AYKEN_BCIB_STUB_RESULT_ENABLE=1 \
    AYKEN_BCIB_STUB_RESULT_VALUE_U64=0xDEADBEEFCAFEBABE \
    > "$EVIDENCE_DIR/build.log" 2>&1 || {
    echo "ERROR: kernel build failed" >&2
    tail -100 "$EVIDENCE_DIR/build.log" >&2
    exit 1
}

KERNEL_ELF="$PROJECT_ROOT/out/build/kernel.elf"
if [[ ! -f "$KERNEL_ELF" ]]; then
    echo "ERROR: kernel.elf not found at $KERNEL_ELF after build" >&2
    exit 1
fi

# Ensure file system sync
sync

# Verify stub code is compiled in by checking for marker strings
# (The function is static so won't appear in symbol tables)
echo "bcib-stub-determinism: verifying stub markers in binary..."

# Extract strings to a temp file to avoid pipe issues
STRINGS_FILE="$EVIDENCE_DIR/kernel_strings.txt"
strings "$KERNEL_ELF" > "$STRINGS_FILE" 2>&1 || {
    echo "ERROR: strings command failed on $KERNEL_ELF" >&2
    exit 1
}

if ! grep -q "EXEC_OUTPUT_WRITTEN" "$STRINGS_FILE"; then
    echo "ERROR: EXEC_OUTPUT_WRITTEN marker not found in kernel binary" >&2
    echo "DEBUG: Strings file has $(wc -l < "$STRINGS_FILE") lines" >&2
    exit 1
fi

if ! grep -q "EXEC_COMPLETE_OK" "$STRINGS_FILE"; then
    echo "ERROR: EXEC_COMPLETE_OK marker not found in kernel binary" >&2
    exit 1
fi

echo "bcib-stub-determinism: stub markers verified in binary"

# Generate report
REPORT_JSON="$EVIDENCE_DIR/report.json"
cat > "$REPORT_JSON" <<EOF
{
  "gate": "bcib-stub-determinism",
  "version": "v1",
  "scope": "Phase-16 stub infrastructure build validation",
  "kernel_profile": "$BCIB_KERNEL_PROFILE",
  "stub_mode": true,
  "build_successful": true,
  "markers_present": ["EXEC_OUTPUT_WRITTEN", "EXEC_COMPLETE_OK"],
  "runtime_validation": false,
  "runtime_validation_note": "Requires bcib_worker (Phase-17)",
  "phase17_backlog": true,
  "status": "PASS"
}
EOF

echo "bcib-stub-determinism: PASS (build validation)"
echo "bcib-stub-determinism: stub infrastructure ready for Phase-17 runtime validation"
exit 0
