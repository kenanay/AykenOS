# `proofd` Service Final Hardening Checklist

**Version:** 1.0
**Status:** Draft (executed locally; Phase-13 preparation)
**Date:** 2026-03-11
**Phase:** Kernel Phase 12 - Trusted Proof Transport and Distributed Verification
**Type:** Non-normative gate-hardening note
**Related Spec:** `requirements.md`, `tasks.md`, `PROOFD_SERVICE_CLOSURE_PLAN.md`, `PROOFD_DIAGNOSTICS_SERVICE_SURFACE.md`, `PROOF_VERIFIER_CRATE_ARCHITECTURE.md`

---

## 1. Purpose

This document defines the final checklist that separated the earlier `proofd` signed-receipt execution slice from a closure-ready `P12-16` gate.

It exists to freeze one distinction:

`execution slice PASS != final T16 closure PASS`

That distinction is now closed locally in `run-local-phase12c-closure-2026-03-11`.

Current local reality now proves:

- `proofd` can delegate to verifier-core
- `proofd` can bind explicit `bundle_path`, `policy_path`, and `registry_path`
- `proofd` can emit a signed receipt
- `proofd` can preserve diagnostics passthrough purity
- repeated identical requests preserve request-bound timestamp semantics
- repeated identical requests rewrite identical receipt and run-manifest artifacts under the current contract

---

## 2. Current Closure Baseline

The current local `ci-gate-proofd-service` closure gate proves:

- `POST /verify/bundle` exists
- explicit policy binding is active
- explicit registry binding is active
- signed receipt emission is active
- signed receipt verification is active
- authority-aware signed receipt verification is active
- root and run-scoped diagnostics endpoints remain read-only artifact passthrough
- repeated identical `POST /verify/bundle` requests return identical JSON
- repeated identical `POST /verify/bundle` requests rewrite identical signed receipt bytes
- repeated identical `POST /verify/bundle` requests rewrite identical run manifest bytes
- execution requests do not mutate parity diagnostics or merge run artifacts

So the current local state is:

`P12-16 = COMPLETED_LOCAL`

and:

`proofd = local closure-ready verification execution service + read-only diagnostics surface`

---

## 3. Closure Conditions Frozen By The Local Gate

`P12-16` is treated as closure-ready locally because all items below are now true in the normative gate.

### 3.1 Signed Receipt Determinism Contract

- the signed receipt path MUST be the normative gate path
- repeated identical requests MUST yield the same verdict
- repeated identical requests MUST yield the same `verdict_subject`
- repeated identical requests MUST yield the same receipt payload semantics
- repeated identical requests MUST yield the same detached receipt signature bytes under the current request contract

### 3.2 Request-Bound Timestamp Contract

The current service contract should freeze:

- `verified_at_utc` is request-bound input, not server-generated time

So the current closure rule is:

- `verified_at_utc` MUST be explicitly present in `receipt_signer`
- `proofd` MUST NOT replace it with `now()`
- repeated execution determinism MUST include the emitted signed receipt path under that request-bound timestamp contract

If a future service contract introduces server-generated timestamps, the determinism gate must be versioned and reduced to identity-bearing receipt fields only. That is not the current contract.

### 3.3 Receipt Boundary Contract

- emitted receipts MUST remain derived verification artifacts
- `proofd` MUST NOT reinterpret receipts as authority objects
- `proofd` MUST NOT rewrite receipt payload fields after verifier-core emission
- receipt boundary preservation MUST be checked against emitted `verdict_subject`

### 3.4 Diagnostics Purity Contract

- all `GET /diagnostics/*` endpoints MUST remain passthrough-only
- execution requests MUST NOT mutate existing parity artifacts
- execution requests MUST NOT cause run merging or cross-run artifact synthesis

### 3.5 Service Contract Stability

- request schema drift MUST fail the gate
- response schema drift MUST fail the gate
- run manifest drift MUST fail the gate
- signed receipt verification drift MUST fail the gate
- authority-aware signed receipt verification drift MUST fail the gate

---

## 4. Exact Gate Assertions

The final `ci-gate-proofd-service` asserts at least the following.

### 4.1 Endpoint Assertions

- `GET /healthz` returns `status=ok`
- `GET /diagnostics/runs` returns the expected run index without merging runs
- `GET /diagnostics/runs/{run_id}` returns the expected run summary
- root diagnostics endpoints equal their underlying artifact files byte-for-byte at JSON value level
- run-scoped diagnostics endpoints equal their run-local artifact files byte-for-byte at JSON value level
- `POST /verify/bundle` returns `status=ok`

### 4.2 Verification Assertions

- `POST /verify/bundle` delegates to verifier-core semantics
- request requires explicit absolute `bundle_path`
- request requires explicit absolute `policy_path`
- request requires explicit absolute `registry_path`
- `emit_signed` requires `receipt_signer`
- missing signer MUST fail as `receipt_signer_missing`

### 4.3 Receipt Assertions

- `receipt_mode = emit_signed`
- `receipt_emitted = true`
- `receipt_path = receipts/verification_receipt.json`
- receipt payload subject fields equal response `verdict_subject`
- signed receipt verification produces no error findings
- authority-aware signed receipt verification produces no error findings
- emitted run manifest records `receipt_mode = emit_signed`

### 4.4 Determinism Assertions

- repeated `GET /diagnostics/parity` returns identical JSON
- repeated identical `POST /verify/bundle` returns identical JSON
- repeated identical `POST /verify/bundle` rewrites an identical signed receipt artifact under the current request-bound timestamp contract
- repeated identical `POST /verify/bundle` rewrites an identical run manifest under the current contract

### 4.5 Non-Goals Assertions

- `proofd` does not perform authority arbitration
- `proofd` does not recompute parity artifacts
- `proofd` does not synthesize incident classes
- `proofd` does not merge run evidence

---

## 5. Evidence Layout

The final hardening gate exports at least:

- `proofd_service_report.json`
- `proofd_receipt_report.json`
- `proofd_endpoint_contract.json`
- `proofd_verify_request.json`
- `proofd_verify_response.json`
- `proofd_run_manifest.json`
- `proofd_receipt_verification_report.json`
- `proofd_repeated_execution_report.json`
- `report.json`
- `violations.txt`

---

## 6. Failure Labels

The final hardening gate fails closed on at least:

- `verify_endpoint_missing`
- `policy_binding_missing`
- `registry_binding_missing`
- `receipt_signer_missing`
- `signed_receipt_verification_failed`
- `receipt_authority_verification_failed`
- `receipt_boundary_preserved_failed`
- `run_manifest_receipt_mode_mismatch`
- `repeated_execution_determinism_failed`
- `diagnostics_passthrough_drift`
- `run_artifact_merge_detected`

---

## 7. Closure Decision Rule

`P12-16` is `COMPLETED_LOCAL` because:

- signed receipt execution slice remains green
- exact gate assertions remain green
- repeated execution determinism remains green under request-bound `verified_at_utc`
- diagnostics purity remains green
- no service-side semantic reinterpretation is introduced

The remaining work after this local closure decision is not service implementation.

The remaining work is:

- remote / official confirmation of the same contract
- governance-level status updates that rely on remote confirmation

---

## 8. Summary

The current local service is beyond bootstrap and beyond execution-slice-only status.

The signed-path determinism contract is now frozen by executable evidence.

So the correct next order is:

1. preserve the current gate assertions against drift
2. confirm the same contract in remote / official evidence
3. carry the result into the formal Phase-12 closure decision
