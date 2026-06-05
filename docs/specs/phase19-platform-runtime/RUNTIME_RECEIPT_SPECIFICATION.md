# Phase-19 Runtime Receipt Specification

This document is subordinate to `PHASE19_RUNTIME_DECISION.md`,
`RUNTIME_LIFECYCLE_SPECIFICATION.md`,
`WORKSPACE_ADMISSION_RUNTIME_SPECIFICATION.md`, and
`../phase18-platform-constitution/TERMINOLOGY_AUDIT.md`. In case of conflict,
those documents prevail.

**Status:** PRE-IMPLEMENTATION RFC / PHASE-19 NOT ACTIVE / RUNTIME NOT AUTHORIZED
**Contract id:** `ayken.phase19.runtime.receipt.v1`
**Authority boundary:** Documentation/specification only; not a token,
capability, runtime handle, loader handle, workspace handle, plugin instance,
trust assignment, Semantic CLI authority, AI Runtime authority, syscall,
kernel ABI expansion, or closure authority.

## Purpose

This RFC defines the deterministic runtime receipt for the first possible
Phase-19 Runtime MVP.

The receipt records the outcome of the static input bundle admission flow. It
does not grant access or execution.

## Core Rule

```text
Receipt != token
```

A receipt is evidence. It must not be accepted as a bearer credential,
capability token, workspace handle, plugin binding, execution right, trust
assignment, or loader result.

## Receipt Fields

Version 1 receipts require:

| Field | Required | Meaning | Authority boundary |
|---|---|---|---|
| `schema_id` | yes | Must be `ayken.phase19.runtime.receipt.v1` | Not authority |
| `receipt_id` | yes | Stable deterministic id | Not token id |
| `subject` | yes | Bound subject | Not execution target |
| `input_bundle_digest` | yes | Bound input bundle | Not replay permission |
| `lifecycle_digest` | yes | Ordered lifecycle digest | Not lifecycle authority |
| `validation_result_digest` | yes | Bound validation integration digest | Not validation grant |
| `admission_record_digest` | conditional | Required for admitted records | Not workspace handle |
| `receipt_status` | yes | `admitted_recorded`, `denied`, or `aborted` | Not runtime state |
| `denial_reason` | conditional | Required for denied or aborted receipts | Not override authority |

Unknown fields fail closed.

## Receipt Status Values

| Status | Meaning | Boundary |
|---|---|---|
| `admitted_recorded` | Admission record and receipt were emitted | No execution or workspace |
| `denied` | Input failed deterministic checks | Terminal denial |
| `aborted` | Flow stopped due to ambiguity or internal failure | Terminal denial |

Unknown statuses fail closed.

## Digest Binding

The receipt digest must bind:

1. Receipt schema id.
2. Subject.
3. Input bundle digest.
4. Lifecycle digest.
5. Validation result digest.
6. Admission record digest when present.
7. Receipt status.
8. Denial reason when present.

The receipt must not bind wall-clock time as an authoritative verdict input.

## Denial Requirements

A receipt must be denied or aborted if:

1. Input bundle is missing.
2. Validation integration is missing.
3. Workspace admission record is stale or mismatched.
4. Any digest is stale or inconsistent.
5. Receipt content declares capability, token, loader, execution, trust,
   plugin, workspace mount, Semantic CLI, AI, registry, or agent authority.
6. Kernel ABI expansion is required.

## Evidence Requirements

Future implementation evidence must prove:

1. Same accepted input emits identical receipt digest.
2. Changed input digest changes or denies the receipt.
3. Missing validation denies the receipt.
4. Authority-granting receipt fields deny the receipt.
5. Denial receipts cannot be reused as success receipts.

## Acceptance Boundary

This RFC defines receipt semantics only. It does not implement receipt
generation and does not grant runtime authority.
