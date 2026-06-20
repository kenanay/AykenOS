# Phase-19 Reference Integrity Validation Decision Candidate

This document is subordinate to PHASE 0 - FOUNDATIONAL OATH,
`ARCHITECTURE_FREEZE.md`, the Phase-18 Platform Constitution reference set,
`docs/specs/phase18-platform-constitution/AUTHORITY_DRIFT_GUARD.md`,
`docs/specs/phase18-platform-constitution/TERMINOLOGY_AUDIT.md`,
`PHASE19_RUNTIME_DECISION.md`, the Phase-19 Runtime RFC set,
`docs/specs/phase19-platform-runtime/RUNTIME_EVIDENCE_MATRIX.md`, and
`PHASE19_RUNTIME_IMPLEMENTATION_POST_MERGE_CONSISTENCY_REVIEW.md`.

**Status:** DECISION CANDIDATE / EVIDENCE MATRIX DELTA CANDIDATE / IMPLEMENTATION NOT AUTHORIZED / NO RUNTIME EXPANSION
**Candidate date:** 2026-06-20
**Candidate id:** `ayken.phase19.reference_integrity_validation_decision_candidate.v1`
**Proposal base main SHA:** `eb33776d76b5c5ef2234b8d5d5b53d7f7c561f65`
**Prior bounded implementation subject SHA:** `0a067dbaa230838e2c14e1e1f0bd91494092713e`
**Authority boundary:** Candidate documentation and candidate evidence-matrix
delta only; not source code authority, not an implementation decision, not an
evidence PASS, not acceptance, not merge authority, not runtime activation,
not general runtime authority, and not Phase-19 closure.

## Core Rule

```text
reference integrity validation != general parsing
digest recomputation != package installation or loading
stage-order verification != Platform ABI validator authority
candidate matrix rows != evidence PASS
```

This candidate narrows the next possible Phase-19 implementation slice. It
does not authorize that slice.

## Decision Question

A later reviewed decision may consider one bounded extension to the existing
typed, test-owned admission/receipt harness:

```text
typed static bundle
  -> typed test-owned reference content binding
  -> canonical contract/schema/subject checks
  -> SHA-256 content digest recomputation
  -> structured Phase-18 validation stage-order checks
  -> existing inert admission/receipt record emission
```

The extension must remain in the existing userspace crate and must not read
untrusted serialized input or access filesystem, network, process, syscall,
loader, installer, mount, or execution APIs.

## Candidate Scope

The narrow candidate may validate only the following typed reference classes:

| Reference class | Required canonical contract id | Local typed-envelope version |
|---|---|---|
| Module manifest | `ayken.platform.module.manifest.v1` | `1` |
| Package metadata | `ayken.platform.package.metadata.v1` | `1` |
| Platform validation policy | `ayken.platform.abi.validation.gate.v1` | `1` |
| Workspace declaration | `ayken.platform.workspace.lifecycle.v1` | `1` |
| Runtime evidence matrix | `ayken.phase19.runtime.evidence_matrix.v1` | `1` |
| Platform validation receipt | `ayken.platform.abi.validation.gate.v1` | `1` |

The local typed-envelope version is Phase-19 harness metadata. It must not be
represented as a new field in a Phase-18 record.

Legacy aliases such as `ayken.phase18.module_manifest.schema.v1`,
`ayken.phase18.package_metadata.schema.v1`,
`ayken.phase18.platform_abi_validation_gate.v1`, and
`ayken.phase18.workspace_lifecycle.specification.v1` are not canonical
Phase-18 contract identifiers and must fail closed in the later proposal.

## Candidate Data Boundary

The later implementation proposal may introduce an explicit test-owned
content table supplied directly to the harness. Each entry must contain:

1. Stable reference key matching one bundle reference.
2. Exact test-owned bytes used for digest recomputation.
3. One canonical reference class.
4. One required subject equal to the bundle subject.

The content table must satisfy all of the following:

1. Exactly one content entry exists for every declared reference.
2. No undeclared, duplicate, or ambiguous content entry exists.
3. `digest_algorithm` is exactly `sha256`.
4. `digest_value` is `sha256:` followed by 64 lower-case hexadecimal digits.
5. SHA-256 recomputation over the supplied bytes equals `digest_value`.
6. Contract id and local envelope version match the reference class table.
7. Subject is present and exactly equals the static bundle subject.

This is content binding for explicit test fixtures. It is not filesystem
resolution, URI fetching, package retrieval, registry access, or a parser.

## Validation Stage-Order Boundary

The later proposal may replace the unstructured validation-stage digest list
with typed stage references containing only `stage_id`, `stage_index`, and a
digest-bound test-owned stage record reference.

The required order is exactly:

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

The candidate may verify exact count, id, index, order, digest format, and
test-owned content binding. It must not implement the Phase-18 validation
stages, reinterpret their verdicts, issue validation authority, or treat a
stage reference as proof of install, load, mount, execution, trust, or
capability authority.

## Candidate Failure Classes

The later implementation decision must define stable, distinct denial reasons
for at least:

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

All failures must occur before validation-integration success, workspace
admission record emission, or runtime receipt emission. Distinct semantic
failures must not collapse into a generic mismatch class.

## Candidate Evidence Matrix Delta

This section is the only evidence-matrix delta for this candidate. It does
not modify the accepted runtime evidence matrix and does not record PASS.

| ID | Candidate evidence obligation | Required witness | Forbidden reading |
|---|---|---|---|
| P19-RI-A1 | Canonical reference map | Exact class/contract/version table in code and tests | General schema registry |
| P19-RI-A2 | Test-owned content binding | One declared content entry per typed reference | Filesystem or network resolver |
| P19-RI-A3 | Structured stage sequence | Exact ten-entry `0..9` stage id/index sequence | Phase-18 validator implementation |
| P19-RI-P1 | Positive reference integrity | Canonical references, subjects, and recomputed digests permit existing inert receipt | Install, load, mount, or execute authority |
| P19-RI-P2 | Positive stage order | Exact known order permits validation-integration record emission | Validation authority grant |
| P19-RI-N1 | Unknown contract | `unknown_reference_contract` before success record emission | Parser evidence |
| P19-RI-N2 | Unknown envelope version | `unknown_reference_schema_version` before success record emission | General version negotiation |
| P19-RI-N3 | Missing or mismatched subject | Distinct subject denial reason | Identity or trust issuance |
| P19-RI-N4 | Unsupported or malformed digest | Distinct algorithm/format denial reason | Cryptographic policy expansion |
| P19-RI-N5 | Missing, duplicate, or extra content | Distinct content-cardinality denial reason | Repository or registry resolution |
| P19-RI-N6 | Recomputed digest mismatch | `reference_digest_mismatch` | Package authenticity or signature validation |
| P19-RI-N7 | Validation contract mismatch | `validation_contract_mismatch` | Platform ABI validator authority |
| P19-RI-N8 | Invalid stage count/id/index/order | Distinct stable stage denial reason | Stage semantic execution |
| P19-RI-N9 | Stage content digest mismatch | `validation_stage_digest_mismatch` | Validation verdict authority |
| P19-RI-D1 | Positive determinism | Two runs produce identical admission/receipt digests | Wall-clock authority |
| P19-RI-D2 | Denial determinism | Each denial fixture repeats with identical reason and transcript digest | Acceptance by repetition alone |
| P19-RI-R1 | Exact-SHA local evidence | Targeted crate tests and denial transcripts | Merge authority |
| P19-RI-R2 | Exact-SHA remote evidence | Strict `ci-freeze`, full Dev Loop, ABI and governance PASS | Runtime activation |
| P19-RI-B1 | Production boundary | No runtime wiring, new dependency, syscall, kernel, baseline, or workflow authority change | General runtime readiness |

Every row must be bound to one later implementation subject SHA. Missing,
stale, aggregated, or differently scoped evidence fails closed.

## Candidate Implementation Limits

A later proposal must remain limited to:

1. `userspace/phase19-admission-receipt/`.
2. Typed test-owned structures and deterministic pure validation functions.
3. Existing `serde`, `serde_json`, and `sha2` dependencies unless a separate
   reviewed dependency decision is accepted.
4. Additive tests and deterministic transcript evidence.

It must not add:

1. `Deserialize`-driven general input parsing.
2. Filesystem or URI resolution.
3. Package, module, workspace, plugin, capability, trust, Semantic CLI, AI,
   or agent semantics.
4. Installation, loading, mounting, execution, issuance, publication, or
   scheduling.
5. New syscalls, kernel ABI changes, Ring0 policy, workflow authority,
   performance baseline changes, or threshold changes.

## Preconditions For A Later Implementation Decision

Source changes remain denied until a separate reviewed decision package:

1. Accepts or narrows this candidate scope.
2. Freezes the canonical reference map and stage sequence.
3. Binds every candidate matrix row to a planned test or transcript.
4. Defines exact production-default and dependency boundaries.
5. Defines stable denial precedence when multiple failures are present.
6. Records the exact implementation subject and remote evidence requirements.
7. Reconfirms that the safe default is no runtime expansion.

This candidate must not share a commit with implementation source code.

## Governance Complexity Bound

This candidate must not create an unbounded review-of-review chain.

For one later implementation subject, the maximum normal authority sequence
is:

```text
implementation decision
  -> evidence package
  -> acceptance review
  -> merge decision
  -> main exact-SHA sync
```

Status and index synchronization are not new authority documents. Evidence
re-binding is required only when the implementation subject changes.
Post-merge review is opened only for a new technical finding, claim drift, or
explicit contract-consistency question.

## Candidate Conclusion

Reference-integrity validation is the narrowest reasonable Phase-19 technical
candidate after the first bounded admission/receipt milestone.

This document records only its decision boundary and candidate evidence
matrix delta. It does not authorize implementation, runtime activation,
general RFC conformance, general runtime authority, module admission,
loading, installation, execution, or Phase-19 closure.
