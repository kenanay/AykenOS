# VCP Trust Verification Property Tests

This suite covers Task 18.8 through 18.13 and the additional hardening checks
called out during review. It validates the runtime contract that
`validation_result` is never authoritative until capability binding, context
hash, signature, and nonce checks all pass.

## Scope

- Property 25: fake validation state rejection
- Property 26: replayed validation state rejection
- Property 27: signature verification enforcement
- Property 28: trust verification before enforcement
- Property 24: complete validation state trust verification
- Property 29: deterministic trust-check order
- Property 30: validation_result never checked before trust
- Property 31: fail-closed paths leave no partial nonce mutation
- Property 32: replay ledger rejects seen nonce

## Test-Mode Trust Issuer

The tests use deterministic issuer hooks exposed only when
`AYKEN_VCP_TRUST_VERIFICATION_TEST=1`:

- capability_id must match the slot/context capability binding
- context_hash must match `vcp_compute_context_hash(slot)`
- signature must match the deterministic validation-state signature
- nonce must be unique in the append-only test ledger

This closes the old stub gap where any non-zero capability, signature, or nonce
could satisfy the verifier. It is still a deterministic test verifier, not a
claim that production cryptographic trust-root integration is complete.

## Build

```bash
make kernel.elf \
  KERNEL_PROFILE=validation \
  AYKEN_VALIDATION=1 \
  AYKEN_VCP_TRUST_VERIFICATION_TEST=1 \
  AYKEN_MB_SELFTEST=0 \
  AYKEN_GATE4_POLICY_TEST=0
```

The Makefile links `kernel/tests/validation/vcp_trust_verification_test.c` only
when `AYKEN_VCP_TRUST_VERIFICATION_TEST=1`.

## Run

```bash
./scripts/test_vcp_trust_verification.sh
```

The script performs both steps required for Task 18.13:

- builds the validation kernel with `AYKEN_VCP_TRUST_VERIFICATION_TEST=1`
- boots QEMU and requires the late-init marker
  `VCP_TRUST_VERIFICATION_TESTS PASSED` in `out/logs/debug_run.log`

For a manual boot run:

```bash
make run \
  KERNEL_PROFILE=validation \
  AYKEN_VALIDATION=1 \
  AYKEN_VCP_TRUST_VERIFICATION_TEST=1
```

Failure returns non-zero from the suite and the validation boot path halts
fail-closed.
