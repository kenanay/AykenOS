# Phase-19 Reference Integrity Validation Implementation Decision

This document is subordinate to PHASE 0 - FOUNDATIONAL OATH,
`ARCHITECTURE_FREEZE.md`, the Phase-18 Platform Constitution reference set,
`docs/specs/phase18-platform-constitution/AUTHORITY_DRIFT_GUARD.md`,
`docs/specs/phase18-platform-constitution/TERMINOLOGY_AUDIT.md`,
`PHASE19_RUNTIME_DECISION.md`, the Phase-19 Runtime RFC set,
`docs/specs/phase19-platform-runtime/RUNTIME_EVIDENCE_MATRIX.md`,
`PHASE19_RUNTIME_IMPLEMENTATION_POST_MERGE_CONSISTENCY_REVIEW.md`, and
`PHASE19_REFERENCE_INTEGRITY_VALIDATION_DECISION_CANDIDATE.md`.

**Status:** IMPLEMENTATION DECISION / SEPARATE SOURCE PR AUTHORIZED WITHIN BOUNDED SCOPE / SOURCE NOT INCLUDED / EVIDENCE AND ACCEPTANCE PENDING / NO RUNTIME ACTIVATION
**Decision date:** 2026-06-21
**Decision id:** `ayken.phase19.reference_integrity_validation_implementation_decision.v1`
**Candidate subject SHA:** `8d0a17c266ca988ecc5b31859ab319ccf9220c08`
**Candidate merge/main SHA:** `194d5e3e102ebb93cc9f04cc2798b22f466b3baa`
**Prior bounded implementation subject SHA:** `0a067dbaa230838e2c14e1e1f0bd91494092713e`
**Authority boundary:** Documentation decision authorizing one separate draft
source PR within the exact limits below; not source code, not evidence PASS,
not acceptance, not merge authority, not general parsing, not loader or
installer authority, not execution, not runtime activation, not general
runtime authority, and not Phase-19 closure.

## Decision

The reference-integrity validation candidate is accepted as the boundary for
one later, separate implementation PR.

That PR may extend only the existing typed, test-owned admission/receipt
harness. It may not include this decision document as new source authority and
must receive a new exact implementation subject SHA.

This decision does not accept an implementation, evidence package,
acceptance review, or merge decision.

## Core Rule

```text
implementation decision != implementation
reference integrity != parser
digest recomputation != authenticity or signature validation
stage-order verification != Platform ABI validator implementation
source PR PASS != acceptance or merge authority
```

The safe default remains the already merged inert admission/receipt harness.

## Accepted Implementation Behavior

The separate source PR may implement only:

```text
typed static test bundle
  -> canonical typed reference validation
  -> explicit test-owned content binding
  -> SHA-256 content digest recomputation
  -> exact Phase-18 validation stage-reference order verification
  -> existing validation-integration record
  -> existing inert workspace admission record
  -> existing deterministic runtime receipt
```

Every new check must fail closed before validation-integration success,
workspace admission record emission, or runtime receipt emission.

## Exact Source Boundary

The implementation PR may change only:

```text
userspace/phase19-admission-receipt/src/lib.rs
```

Tests must remain inline in that file. The PR must not change:

1. `Cargo.toml` or `Cargo.lock`.
2. CI workflows, Make targets, baselines, or thresholds.
3. Kernel, syscall, ABI, scheduler, loader, workspace, Semantic CLI, or AI
   source.
4. Phase pointer or closure records.

The crate must retain only its existing `serde`, `serde_json`, and `sha2`
dependencies and must remain unwired from production runtime paths.

## Accepted Typed Data Shape

The source PR must add one closed reference classification enum with exactly:

1. `ModuleManifest`
2. `PackageMetadata`
3. `PlatformValidationPolicy`
4. `WorkspaceDeclaration`
5. `RuntimeEvidenceMatrix`

It must add a test-owned content entry containing only:

1. Stable `path_or_uri` reference key.
2. One reference classification value.
3. Required subject equal to the bundle subject.
4. Exact test-owned bytes used for digest recomputation.

The harness entrypoint may accept an immutable slice of these entries. It
must not resolve paths, open files, fetch URIs, read environment state, or
deserialize untrusted input.

The validation evidence must replace its unstructured stage digest vector with
typed stage references containing only:

1. `stage_id`
2. `stage_index`
3. `digest_algorithm`
4. `digest_value`
5. Exact test-owned stage record bytes

No stage semantic evaluator, policy interpreter, or Phase-18 validator may be
implemented.

## Canonical Reference Map

The implementation must use this closed map:

| Reference class | Canonical contract id | Local typed-envelope version |
|---|---|---|
| Module manifest | `ayken.platform.module.manifest.v1` | `1` |
| Package metadata | `ayken.platform.package.metadata.v1` | `1` |
| Platform validation policy | `ayken.platform.abi.validation.gate.v1` | `1` |
| Workspace declaration | `ayken.platform.workspace.lifecycle.v1` | `1` |
| Runtime evidence matrix | `ayken.phase19.runtime.evidence_matrix.v1` | `1` |

The Platform validation receipt contract must also be
`ayken.platform.abi.validation.gate.v1`. Its Phase-19 typed-envelope version
must be `1` and must not be represented as a new Phase-18 receipt field.

Any legacy alias, unknown id, or unknown local envelope version must deny.

## Digest And Content Rules

The source PR must enforce:

1. `digest_algorithm == "sha256"`.
2. `digest_value` is `sha256:` followed by exactly 64 lower-case hexadecimal
   digits.
3. Exactly one test-owned content entry exists for every declared reference.
4. No duplicate or undeclared content entry exists.
5. Every content entry carries the bundle subject.
6. SHA-256 over the exact supplied bytes equals the declared digest.
7. Stage reference digests follow the same algorithm, shape, and content
   recomputation rules.

Digest equality proves only byte binding for the supplied test fixture. It
does not prove publisher identity, signature validity, package authenticity,
trust, capability, installability, loadability, or executability.

## Exact Validation Stage Order

Validation evidence must contain exactly ten stage references in this order:

```text
0 kernel_freeze_guard
1 manifest_validation
2 package_metadata_validation
3 package_manifest_binding
4 trust_classification_validation
5 capability_contract_validation
6 workspace_lifecycle_validation
7 plugin_boundary_validation
8 cross_contract_separation
9 validation_receipt_emission
```

Count, id, index, order, digest format, and test-owned byte binding must be
checked. Stage status, reason semantics, input semantics, and policy behavior
remain external and unimplemented.

The stage denial classes are distinct:

1. `unknown_validation_stage_id` means a stage id is outside the closed list.
2. `validation_stage_index_mismatch` means a known stage id declares an index
   other than its canonical index.
3. `validation_stage_order_mismatch` means all ids and declared indices are
   individually valid but list position does not match canonical order.

## Stable Denial Reasons

The separate source PR must add distinct stable denial reasons for:

1. `unknown_reference_contract`
2. `unknown_reference_schema_version`
3. `missing_reference_subject`
4. `reference_subject_mismatch`
5. `unsupported_reference_digest_algorithm`
6. `malformed_reference_digest`
7. `missing_reference_content`
8. `duplicate_reference_content`
9. `unexpected_reference_content`
10. `reference_digest_mismatch`
11. `validation_contract_mismatch`
12. `validation_stage_count_mismatch`
13. `unknown_validation_stage_id`
14. `validation_stage_index_mismatch`
15. `validation_stage_order_mismatch`
16. `validation_stage_digest_mismatch`

Existing denial reasons must retain their meanings. Distinct failures must
not be collapsed into `subject_mismatch`, `stale_manifest_digest`, or another
generic class.

## Denial Precedence

When multiple defects are present, the first denial must follow this order:

1. Existing input schema, unknown-field, duplicate-key, and kernel ABI guards.
2. Existing required-reference presence guards.
3. Reference classification and canonical contract id.
4. Local typed-envelope version.
5. Required reference subject presence.
6. Reference subject equality.
7. Digest algorithm.
8. Digest textual shape.
9. Missing, duplicate, then unexpected content cardinality.
10. Reference content digest recomputation.
11. Existing input-bundle binding.
12. Platform validation evidence presence.
13. Validation receipt contract, local version, and subject binding.
14. Validation stage count.
15. Stage id.
16. Stage index.
17. Stage order.
18. Stage digest algorithm and shape.
19. Stage content digest recomputation.
20. Existing validation stale, authority, status, and admission authority
    guards.

Reference-integrity failures before input binding must not publish an input
bundle digest. Later failures must not publish validation-integration,
admission, or receipt success records.

## Accepted Evidence Matrix Binding

The separate implementation and evidence package must close every candidate
row from `PHASE19_REFERENCE_INTEGRITY_VALIDATION_DECISION_CANDIDATE.md`:

| Row group | Required evidence |
|---|---|
| `P19-RI-A1..A3` | Exact canonical map, content-table cardinality, and ten-stage structure |
| `P19-RI-P1..P2` | Positive inert receipt and exact ordered stage transcript |
| `P19-RI-N1..N9` | One focused denial fixture and stable reason for each negative row |
| `P19-RI-D1..D2` | Positive and denial repeat digest parity |
| `P19-RI-R1..R2` | Local and remote exact-SHA evidence |
| `P19-RI-B1` | Production unwired, dependency/ABI/workflow/baseline unchanged proof |

Required targeted test clusters are:

1. Canonical reference map and positive content-binding success.
2. Contract and envelope-version denial.
3. Subject-presence and subject-mismatch denial.
4. Digest algorithm, format, and recomputation denial.
5. Missing, duplicate, and unexpected content denial.
6. Validation contract and exact stage count/id/index/order denial.
7. Stage digest recomputation denial.
8. Positive and denial determinism repeat proof.
9. Existing admission/receipt and authority-denial regression suite.

Tests are evidence inputs only. They are not acceptance or merge authority.

## Exact-SHA Evidence Requirements

The later implementation subject must receive:

1. Targeted crate test PASS.
2. Positive and negative transcript evidence bound to the implementation SHA.
3. Deterministic repeat evidence for positive and denial paths.
4. Strict `ci-freeze` PASS.
5. Full Dev Loop PASS.
6. ABI, governance, spec-purity, isolation, and locked performance PASS.
7. Proof that production wiring, dependencies, workflows, baseline, syscall
   surface, and ABI metadata are unchanged.

Historical candidate or prior implementation PASS results cannot be inherited.
Any implementation subject change requires evidence regeneration.

## Decision Record Evidence

This documentation decision is based on:

| Input | Result |
|---|---|
| Candidate subject | `8d0a17c266ca988ecc5b31859ab319ccf9220c08` |
| Candidate merge/main subject | `194d5e3e102ebb93cc9f04cc2798b22f466b3baa` |
| Candidate PR | PR #185, merged |
| Candidate PR strict freeze | Run `27877760939`, PASS |
| Candidate PR full Dev Loop | Run `27877760951`, PASS |
| Candidate main strict freeze | Run `27878009714`, PASS |
| Candidate main full Dev Loop | Run `27878009710`, PASS |

This decision record itself must also receive exact-head strict freeze and full
Dev Loop PASS before it can authorize opening the separate source PR.

## Prohibited Readings

This decision does not authorize:

1. General parsing or `Deserialize`-driven input acceptance.
2. Filesystem, URI, package, registry, or network resolution.
3. Signature, publisher, dependency, trust, capability, plugin, or module
   semantics.
4. Package installation or execution.
5. Module or plugin loading.
6. Workspace creation, runtime, or real mounts.
7. Capability or trust issuance.
8. Semantic CLI, AI Runtime, or agent authority.
9. New syscalls, kernel ABI expansion, or Ring0 policy.
10. Runtime activation, general runtime authority, or Phase-19 closure.

Unknown authority readings fail closed.

## Governance Bound

After this decision is accepted, the normal chain for one implementation
subject is limited to:

```text
separate implementation PR
  -> evidence package
  -> acceptance review
  -> merge decision
  -> main exact-SHA sync
```

Status synchronization is index maintenance, not a new authority layer.
Evidence re-binding is required only when source changes produce a new subject.
Post-merge review requires a new technical finding, claim drift, or explicit
contract-consistency question.

## Decision Conclusion

One separate draft source PR may be opened after this decision record receives
its own exact-head remote PASS.

That PR is limited to typed test-owned reference integrity in
`userspace/phase19-admission-receipt/src/lib.rs`. It remains subject to a new
evidence package, acceptance review, merge decision, and main exact-SHA sync.

Runtime activation, general parsing, loading, installation, execution,
general runtime authority, and Phase-19 closure remain unauthorized.
