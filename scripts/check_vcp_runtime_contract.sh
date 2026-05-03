#!/bin/bash
# Static guard for the VCP runtime hook contract.
#
# Production kernel code must not read vcp_validation_state.validation_result
# directly outside vcp_runtime.c. All enforcement must flow through
# vcp_runtime_validate() / vcp_verify_validation_state().

set -euo pipefail

if ! command -v rg >/dev/null 2>&1; then
    echo "ERROR: rg is required for VCP runtime contract checks." >&2
    exit 1
fi

pattern='->[[:space:]]*validation_result|\.[[:space:]]*validation_result'

set +e
violations="$(
    rg -n \
        --glob '*.[ch]' \
        --glob '!kernel/sys/vcp_runtime.c' \
        --glob '!kernel/include/execution_slot.h' \
        --glob '!kernel/include/vcp_runtime.h' \
        --glob '!kernel/tests/**' \
        -- "$pattern" kernel
)"
rg_status=$?
set -e

if [ "$rg_status" -gt 1 ]; then
    echo "ERROR: rg failed while checking VCP runtime contract." >&2
    exit "$rg_status"
fi

if [ -n "$violations" ]; then
    echo "ERROR: direct validation_result access outside VCP runtime verifier is forbidden." >&2
    echo "$violations" >&2
    exit 1
fi

echo "PASS: no production validation_result bypass found."
