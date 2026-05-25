# VCP Diagnostic Evidence Stub Tests

Task 5 implements diagnostic-only evidence emission. These stubs are useful for
boot proof and debugging, but they are not signed, durable, verified, or
authoritative.

## Scope

- Property 8: validation, contract, and boundary events emit diagnostic entries.
- Property 11: fail-closed paths emit complete diagnostic failure context.

## Non-Authority Boundary

- Diagnostic entries may use an in-memory ring buffer.
- Diagnostic marker emission does not authorize execution.
- Diagnostic emission failure semantics are not authoritative in Task 5.
- Signed, verified, durable-before-proceed evidence is reserved for Tasks 20-23.

## Required Markers

- `VCP_EVIDENCE_TESTS PASSED`
- `[VCP_EVIDENCE][VALIDATION_CHECK]`
- `[VCP_EVIDENCE][CONTRACT_EXECUTION]`
- `[VCP_EVIDENCE][BOUNDARY_CROSSING]`
- `[VCP_FAIL_CLOSED][BLOCK]`
- `[VCP_EVIDENCE][COMPREHENSIVE]`
- `[VCP_EVIDENCE][FAIL_CLOSED_COMPLETE]`

## Runner

```sh
./scripts/test_vcp_evidence.sh
```
