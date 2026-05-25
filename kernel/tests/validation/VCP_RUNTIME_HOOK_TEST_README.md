# VCP Runtime Hook Property Tests

This suite covers Task 2.3 through 2.5. It validates that
`vcp_runtime_validate()` is the runtime enforcement hook and blocks execution
unless the slot carries trusted, valid VCP state.

## Scope

- Property 2: fail-closed on missing validation state
- Property 3: invalid validation state blocks execution
- Property 4: valid validation state permits execution

The runtime hook tests keep trust-token inputs valid for Properties 3 and 4.
They vary only `validation_result` after producing a deterministic trusted
state. Capability, context, signature, and nonce failure cases remain isolated
in the Task 18 trust verification suite.

## Build And Run

```bash
./scripts/test_vcp_runtime_hook.sh
```

The script builds the validation kernel with `AYKEN_VCP_RUNTIME_HOOK_TEST=1`,
boots QEMU, and requires the late-init marker
`VCP_RUNTIME_HOOK_TESTS PASSED` in `out/logs/debug_run.log`.

It also requires per-path boot markers:

- `[VCP_HOOK][FAIL_MISSING_STATE]`
- `[VCP_HOOK][FAIL_INVALID_STATE]`
- `[VCP_HOOK][ALLOW_VALID]`

Before booting, the script runs `scripts/check_vcp_runtime_contract.sh`. That
guard fails if production kernel code reads `validation_result` directly outside
`kernel/sys/vcp_runtime.c`; all enforcement must go through the runtime hook.

These tests use the deterministic test issuer hooks from `vcp_runtime.c`. They
prove the runtime hook contract in test mode; production evidence and trust-root
authority are implemented by Tasks 20-23.
