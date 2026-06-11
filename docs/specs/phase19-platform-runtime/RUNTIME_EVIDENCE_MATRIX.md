# Phase-19 Runtime Evidence Matrix

This document is subordinate to PHASE 0 - FOUNDATIONAL OATH,
`ARCHITECTURE_FREEZE.md`, `PHASE18_TRANSITION_DECISION.md`,
`PHASE18_ACTIVATION_DECISION.md`, the Phase-18 Platform Constitution
reference set, `AUTHORITY_DRIFT_GUARD.md`, `TERMINOLOGY_AUDIT.md`,
`PHASE19_RUNTIME_DECISION.md`, `../../../PHASE19_POINTER_TRANSITION_DECISION.md`,
the Phase-19 Runtime RFC set,
`docs/specs/phase19-platform-runtime/RUNTIME_EVIDENCE_PLAN.md`, and
`../../../PHASE19_RUNTIME_IMPLEMENTATION_DECISION_CANDIDATE.md`. In case of
conflict, those documents prevail.

**Status:** ACTIVE RFC / EVIDENCE MATRIX ONLY / RUNTIME IMPLEMENTATION NOT AUTHORIZED
**Contract id:** `ayken.phase19.runtime.evidence_matrix.v1`
**Authority boundary:** Documentation/specification only; not a CI gate
implementation, not evidence PASS, not runtime implementation, not a manifest
parser, not a package installer, not a module loader, not workspace runtime,
not real mount authority, not plugin loading, not capability issuance, not
trust assignment, not Semantic CLI authority, not AI Runtime authority, not a
syscall, not kernel ABI expansion, not merge authority, and not closure
authority.

## Purpose

This matrix maps the Phase-19 Runtime MVP artifacts to the evidence that a
later implementation decision must require before runtime source code can be
accepted.

It does not implement the evidence. It does not accept any implementation
decision. It does not authorize runtime source code.

## Core Rule

```text
evidence matrix != evidence PASS
evidence row != CI gate
artifact evidence != authority grant
```

The safe default remains no runtime behavior unless a later exact-SHA
implementation decision grants a specific bounded behavior and its own
evidence package passes.

## Matrix Scope

The matrix covers only the accepted Phase-19 Runtime MVP shape:

```text
static input bundle
  -> Phase-18 Platform ABI validation integration record
  -> workspace admission record
  -> deterministic runtime receipt
```

This flow must remain inert. It must not install, load, mount, execute, issue,
trust, publish, schedule, bind, or grant anything.

## Artifact Evidence Matrix

| Artifact | Positive evidence required later | Determinism evidence required later | Mandatory denial evidence | Forbidden reading |
|---|---|---|---|---|
| Static input bundle | One static, test-owned bundle is digest-bound and accepted as input | Same bundle produces the same input bundle digest | Unknown field, duplicate key, missing reference, stale digest, contradictory subject | Parser, installer request, loader request, execution request, token request, workspace creation request |
| Validation integration record | Phase-18 Platform ABI validation evidence is bound to the same input subject | Same validation input produces the same validation-integration digest | Missing validation receipt, validation FAIL, subject mismatch, validation-as-authority | Authorization, install permission, load permission, trust grant, capability grant |
| Workspace admission record | Admission record is emitted only after validation integration succeeds | Same accepted input produces the same admission record digest | Real mount request, workspace handle claim, namespace creation claim, access grant claim | Workspace creation, filesystem mount, namespace creation, handle, access grant |
| Runtime receipt | Receipt binds lifecycle, input, validation, and admission digests | Same accepted input produces the same receipt digest | Receipt-as-token, receipt-as-handle, receipt-as-execution-right | Bearer token, capability token, workspace handle, plugin binding, execution right |
| Lifecycle transcript | Transcript shows the bounded inert order ending at receipt emission | Same accepted input produces the same lifecycle transcript digest | Any `LOADED`, `RUNNING`, `EXECUTING`, `MOUNTED`, issuer, trust, registry, Semantic CLI, AI, or agent authority phase | Scheduler state, loader state, execution state, workspace runtime state |

## Positive Evidence Matrix

| ID | Evidence target | Required later proof | Does not prove |
|---|---|---|---|
| P19-M-P1 | Input binding | Static test-owned input bundle is accepted with canonical digest and subject binding | General parsing, installability, loadability, execution |
| P19-M-P2 | Validation integration | Validation-integration record references the accepted Phase-18 declarative contract metadata required for admission/receipt evidence | Authorization, trust, capability, workspace, plugin, Semantic CLI, AI authority |
| P19-M-P3 | Admission record | Workspace admission record is emitted as an inert record after validation integration succeeds | Workspace creation, real mount, namespace creation, handle, access grant |
| P19-M-P4 | Runtime receipt | Receipt binds lifecycle, input, validation, and admission digests | Token minting, capability issuance, execution right, plugin binding |
| P19-M-P5 | Bounded transcript | Lifecycle transcript follows the accepted inert order without loader, executor, issuer, trust, registry, Semantic CLI, AI, or agent phases | General runtime execution |

## Negative Evidence Matrix

Every negative case must fail closed before receipt success emission.

| ID | Negative case | Required denial point | Required stable reason class |
|---|---|---|---|
| P19-M-N1 | Unknown input bundle field | Before input binding success | `unknown_input_field` |
| P19-M-N2 | Duplicate input bundle key | Before input binding success | `duplicate_input_key` |
| P19-M-N3 | Missing manifest reference | Before validation integration success | `missing_manifest_reference` |
| P19-M-N4 | Stale manifest digest | Before validation integration success | `stale_manifest_digest` |
| P19-M-N5 | Package and manifest subject mismatch | Before validation integration success | `subject_mismatch` |
| P19-M-N6 | Missing Platform ABI validation receipt | Before validation integration success | `missing_platform_validation` |
| P19-M-N7 | Platform ABI validation FAIL | Before admission record success | `platform_validation_failed` |
| P19-M-N8 | Workspace declaration requests real mount | Before admission record success | `real_mount_denied` |
| P19-M-N9 | Admission record claims workspace handle | Before admission record success | `workspace_handle_denied` |
| P19-M-N10 | Receipt declares token authority | Before receipt success | `receipt_token_denied` |
| P19-M-N11 | Trust classification treated as capability grant | Before receipt success | `trust_capability_denied` |
| P19-M-N12 | Plugin compatibility treated as loading | Before receipt success | `plugin_loading_denied` |
| P19-M-N13 | Semantic CLI output treated as runtime authority | Before receipt success | `semantic_cli_authority_denied` |
| P19-M-N14 | AI output treated as runtime authority | Before receipt success | `ai_authority_denied` |
| P19-M-N15 | New syscall or kernel ABI expansion request | Before input binding success | `kernel_abi_expansion_denied` |

## Determinism Matrix

| ID | Deterministic surface | Required later repeat proof |
|---|---|---|
| P19-M-D1 | Lifecycle transcript digest | Same accepted input produces the same transcript digest across repeated runs |
| P19-M-D2 | Input bundle digest | Same bundle produces the same canonical input digest |
| P19-M-D3 | Validation integration digest | Same validation evidence and subject binding produce the same integration digest |
| P19-M-D4 | Admission record digest | Same accepted input and validation integration produce the same admission digest |
| P19-M-D5 | Runtime receipt digest | Same accepted input, validation, admission, and lifecycle produce the same receipt digest |
| P19-M-D6 | Denial reason digest | Same negative input produces the same denial reason class and digest |

Wall-clock, host timing, runner identity, debug log ordering, or advisory text
must not affect authoritative verdicts.

## Remote Evidence Matrix

| ID | Remote requirement | Required later result |
|---|---|---|
| P19-M-R1 | Strict freeze | `ci-freeze` PASS on the exact implementation decision subject SHA |
| P19-M-R2 | Dev Loop | Dev Loop PASS on the exact implementation decision subject SHA |
| P19-M-R3 | Runtime-specific gate | Any new admission/receipt runtime gate PASS on the exact subject SHA |
| P19-M-R4 | Kernel ABI preservation | Syscall IDs `1000-1011`, count `12`, and ABI version `0x00010001` remain unchanged |
| P19-M-R5 | Authority drift guard | Loader, installer, mount, execution, issuer, trust, registry, Semantic CLI, AI, and agent readings remain denied |
| P19-M-R6 | Production default | Any validation-only path is default-off and documented with owner, measured surface, and closure condition |

## Performance Evidence Boundary

This matrix does not create a performance claim.

If a later implementation touches runtime hot paths, that implementation
decision must define:

1. Measured surface.
2. Baseline authority.
3. Threshold policy.
4. Local diagnostic boundary.
5. Remote acceptance boundary.
6. Evidence artifact location.

Docs-only evidence-matrix changes do not require baseline renewal and do not
authorize performance acceptance.

## Later Decision Acceptance Boundary

A later Phase-19 implementation decision is not ready unless it maps every
accepted positive, negative, deterministic, remote, production-default, and
performance-relevant matrix row to concrete evidence paths.

Missing, ambiguous, stale, or partially mapped rows fail closed.

## Non-Authority Rule

This matrix must not be read to authorize:

1. Runtime source code.
2. General manifest parser implementation.
3. Package installation.
4. Package execution.
5. Module loading.
6. Plugin host, plugin loading, or plugin instantiation.
7. Workspace creation, workspace runtime, or real mounts.
8. Capability token minting.
9. Capability issuance.
10. Trust assignment or trust issuer behavior.
11. Registry publication or marketplace behavior.
12. Semantic CLI execution authority.
13. AI Runtime authority.
14. Agent behavior.
15. New syscalls.
16. Kernel ABI expansion.
17. Ring0 policy.
18. Observability-as-authority.

Unknown authority readings fail closed.

## Matrix Conclusion

This matrix converts the Phase-19 evidence plan into a reviewable evidence
mapping for a later implementation decision.

It is still documentation only.

Runtime implementation remains unauthorized.
