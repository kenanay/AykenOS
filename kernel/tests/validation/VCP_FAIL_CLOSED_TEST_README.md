# VCP Fail-Closed Property Tests

Task 4 validates the fail-closed enforcement mechanism after Task 18 trust
verification and Task 2 runtime hook enforcement are in place.

## Scope

- Property 9: once VCP fail-closed is triggered, the execution slot cannot
  recover by attaching a later valid validation state.
- Property 10: fail-closed preserves slot identity and validation-state fields,
  while only applying the permanent blocked state.

## Contract

- `vcp_fail_closed()` returns `VCP_FAIL_CLOSED`; it does not panic.
- A VCP-blocked slot is represented by:
  - `slot->state == EXEC_SLOT_ABORTED`
  - `slot->error_code == VCP_FAIL_CLOSED_SLOT_ERROR_CODE`
- `vcp_runtime_validate()` short-circuits when that permanent state is active.
- Fail-closed paths must not commit nonce ledger entries.

## Boot Markers

The integration runner requires these debugcon markers:

- `VCP_FAIL_CLOSED_TESTS PASSED`
- `[VCP_FAIL_CLOSED][BLOCK]`
- `[VCP_FAIL_CLOSED][PERMANENCE]`
- `[VCP_FAIL_CLOSED][STATE_INTEGRITY]`

## Runner

```sh
./scripts/test_vcp_fail_closed.sh
```

The runner also executes `scripts/check_vcp_runtime_contract.sh` so direct
production reads of `validation_result` remain blocked outside the VCP runtime
module.
