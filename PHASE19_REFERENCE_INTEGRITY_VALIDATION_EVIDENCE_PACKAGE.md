# Phase-19 Reference Integrity Validation Evidence Package

This document is subordinate to PHASE 0 - FOUNDATIONAL OATH,
`ARCHITECTURE_FREEZE.md`, the Phase-18 Platform Constitution reference set,
`docs/specs/phase18-platform-constitution/AUTHORITY_DRIFT_GUARD.md`,
`docs/specs/phase18-platform-constitution/TERMINOLOGY_AUDIT.md`,
`PHASE19_RUNTIME_DECISION.md`, the Phase-19 Runtime RFC set,
`docs/specs/phase19-platform-runtime/RUNTIME_EVIDENCE_MATRIX.md`,
`PHASE19_RUNTIME_IMPLEMENTATION_POST_MERGE_CONSISTENCY_REVIEW.md`,
`PHASE19_REFERENCE_INTEGRITY_VALIDATION_DECISION_CANDIDATE.md`, and
`PHASE19_REFERENCE_INTEGRITY_VALIDATION_IMPLEMENTATION_DECISION.md`. In case
of conflict, those documents prevail unless this package is the narrower
evidence record for the implementation subject identified below.

**Status:** EVIDENCE PACKAGE / IMPLEMENTATION SUBJECT RECORDED / MATRIX ROWS BOUND FOR ACCEPTANCE REVIEW / ACCEPTANCE AND MERGE AUTHORITY PENDING / NO RUNTIME ACTIVATION
**Evidence package date:** 2026-06-21
**Evidence id:** `ayken.phase19.reference_integrity_validation_evidence_package.v1`
**Implementation subject SHA:** `e3028fee36d06efa23401184f21a4e4815f7757e`
**Implementation PR:** PR #187, draft at evidence capture time
**Implementation base SHA:** `62d736cbb8d859beaaa5a5960ff53ca480d5cd38`
**Superseded implementation subject:** `b9b0f7b7a42714016b06551b1edaaa2f986542b8`
**Authority boundary:** Evidence package only; not source acceptance, not an
acceptance review, not merge authority, not a parser, not filesystem or
network resolution, not authenticity or signature validation, not a
Phase-18 validator implementation, not loader, installer, mount, workspace,
execution, capability, trust, Semantic CLI, AI Runtime, syscall, kernel ABI,
runtime activation, general runtime authority, or Phase-19 closure.

## Core Rule

```text
evidence package != acceptance review
exact-head PASS != merge authority
reference byte binding != authenticity
stage-reference order validation != stage semantic execution
```

This package binds evidence only to implementation subject
`e3028fee36d06efa23401184f21a4e4815f7757e`.

## Evidence Subject Rule

PR #187 remains the one-file source PR. This evidence package is recorded on
a separate documentation branch so that the source PR boundary remains
exactly:

```text
userspace/phase19-admission-receipt/src/lib.rs
```

The evidence-package commit is not a new implementation subject. Any later
change to `lib.rs` invalidates the implementation evidence in this package
and requires a new exact implementation SHA plus regenerated local and remote
evidence.

## Implementation Scope Proof

The implementation subject differs from base SHA
`62d736cbb8d859beaaa5a5960ff53ca480d5cd38` in exactly one file:

```text
userspace/phase19-admission-receipt/src/lib.rs
```

The subject does not change:

1. `Cargo.toml` or `Cargo.lock`.
2. Existing `serde`, `serde_json`, or `sha2` dependency declarations.
3. CI workflows, Make targets, thresholds, or performance baselines.
4. Kernel, syscall, scheduler, loader, workspace, Semantic CLI, or AI source.
5. ABI declarations, syscall range `1000-1011`, syscall count `12`, or ABI
   version `0x00010001`.
6. Phase pointer, status, roadmap, or closure records.

Repository search finds no production call site for `run_harness`; all direct
calls remain inline test-owned calls in the same library file.

## Corrected Denial Precedence Evidence

The implementation validates reference failures in the accepted order:

1. Canonical contract and content classification.
2. Local typed-envelope version.
3. Required declared-reference subject presence.
4. Declared-reference and supplied-content subject equality.
5. Digest algorithm.
6. Digest textual shape.
7. Missing, duplicate, then unexpected content cardinality.
8. Reference content digest recomputation.

`reference_integrity_denial_precedence_is_stable` supplies combined-defect
fixtures proving that:

1. Contract failure precedes version, subject, algorithm, and digest shape.
2. Envelope-version failure precedes missing subject and algorithm failure.
3. Content classification failure precedes version, subject, algorithm, and
   missing-content failure.
4. Content subject mismatch precedes algorithm and missing-content failure.
5. Unknown validation stage id precedes index, digest, and stale-evidence
   failures.

All reference-integrity failures occur before input binding and therefore do
not publish an input-bundle digest. Validation contract and stage failures
occur after input binding but before validation-integration, admission, or
receipt success record emission.

## Canonical Reference Map Evidence

| Reference class | Canonical contract id | Local envelope version |
|---|---|---|
| `ModuleManifest` | `ayken.platform.module.manifest.v1` | `1` |
| `PackageMetadata` | `ayken.platform.package.metadata.v1` | `1` |
| `PlatformValidationPolicy` | `ayken.platform.abi.validation.gate.v1` | `1` |
| `WorkspaceDeclaration` | `ayken.platform.workspace.lifecycle.v1` | `1` |
| `RuntimeEvidenceMatrix` | `ayken.phase19.runtime.evidence_matrix.v1` | `1` |

The Platform validation receipt contract is also
`ayken.platform.abi.validation.gate.v1` with local typed-envelope version
`1`. `canonical_reference_map_and_stage_order_are_exact` verifies the closed
five-class map and exact stage sequence.

## Positive Receipt And Stage Transcript

`positive_flow_emits_inert_deterministic_records` runs the canonical fixture
twice and asserts equality of the complete outcomes. The accepted lifecycle
transcript is:

```text
UNINITIALIZED
  -> INPUT_BOUND
  -> VALIDATING
  -> VALIDATED_RECORDABLE
  -> ADMISSION_RECORDED
  -> RECEIPT_EMITTED
```

The canonical lifecycle transcript digest is:

```text
sha256:9c25d061b39761189cd5e6268166d52639d4bb6531e17eaaa94e697fab5296ef
```

The positive fixture emits the existing inert validation-integration record,
workspace admission record, and runtime receipt. It does not install, load,
mount, execute, issue authority, or activate runtime behavior.

Validation evidence contains exactly these ten typed stage references:

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

Every stage fixture carries `sha256`, a lower-case 64-hex digest, and exact
test-owned bytes whose SHA-256 value is recomputed before success.

## Negative Evidence Binding

| Stable denial reason | Focused test witness | Record boundary |
|---|---|---|
| `unknown_reference_contract` | `reference_contract_and_version_fail_closed` | Before input binding |
| `unknown_reference_schema_version` | `reference_contract_and_version_fail_closed` | Before input binding |
| `missing_reference_subject` | `reference_subject_binding_fail_closed` | Before input binding |
| `reference_subject_mismatch` | `reference_subject_binding_fail_closed` | Before input binding |
| `unsupported_reference_digest_algorithm` | `reference_digest_shape_and_content_fail_closed` | Before input binding |
| `malformed_reference_digest` | `reference_digest_shape_and_content_fail_closed` | Before input binding |
| `missing_reference_content` | `reference_content_cardinality_fail_closed` | Before input binding |
| `duplicate_reference_content` | `reference_content_cardinality_fail_closed` | Before input binding |
| `unexpected_reference_content` | `reference_content_cardinality_fail_closed` | Before input binding |
| `reference_digest_mismatch` | `reference_digest_shape_and_content_fail_closed` | Before input binding |
| `validation_contract_mismatch` | `validation_contract_and_stage_structure_fail_closed` | Input bound; no success records |
| `validation_stage_count_mismatch` | `validation_contract_and_stage_structure_fail_closed` | Input bound; no success records |
| `unknown_validation_stage_id` | `validation_contract_and_stage_structure_fail_closed` | Input bound; no success records |
| `validation_stage_index_mismatch` | `validation_contract_and_stage_structure_fail_closed` | Input bound; no success records |
| `validation_stage_order_mismatch` | `validation_contract_and_stage_structure_fail_closed` | Input bound; no success records |
| `validation_stage_digest_mismatch` | `validation_stage_digest_fail_closed` | Input bound; no success records |

The before-input denial transcript is:

```text
UNINITIALIZED -> ABORTED
sha256:3b5b60fac26532ec44f8c5105458e32d566bd99ffa9d0acbc6e29b400d722f9c
```

The input-bound validation denial transcript is:

```text
UNINITIALIZED -> INPUT_BOUND -> ABORTED
sha256:1730763be7c6c4e365f2a655043931a23ec87260bf862bdb4f25d53b91c8d1f4
```

No negative fixture emits a validation-integration record, workspace
admission record, or runtime receipt.

## Determinism And Repeat Parity

`positive_flow_emits_inert_deterministic_records` executes the positive
fixture twice and asserts complete outcome equality.

The shared `assert_denied_with_contents` helper executes every negative
fixture twice and asserts complete `HarnessOutcome` equality before checking
the expected reason. Complete equality includes:

1. Stable denial reason.
2. Stable lifecycle transcript.
3. Stable transcript digest.
4. Stable optional input-bundle digest.
5. Absence of all success records.

This closes repeat parity for all negative fixtures, not only one selected
denial class. `reference_integrity_denial_is_deterministic` retains an
additional focused repeat witness for `reference_digest_mismatch`.

## Evidence Matrix Binding

| Matrix row | Evidence for subject `e3028fee...` | Package result |
|---|---|---|
| `P19-RI-A1` | Closed enum, exact contract map, version checks, map test | Bound for acceptance review |
| `P19-RI-A2` | Declared/content cardinality checks and five-class positive fixture | Bound for acceptance review |
| `P19-RI-A3` | Exact ten-entry stage list and typed stage fixture | Bound for acceptance review |
| `P19-RI-P1` | Positive canonical reference/content run emits inert receipt | Bound for acceptance review |
| `P19-RI-P2` | Exact ordered stage run emits validation-integration record | Bound for acceptance review |
| `P19-RI-N1` | Unknown contract and content-class mismatch fixtures | Bound for acceptance review |
| `P19-RI-N2` | Unknown local envelope version fixture | Bound for acceptance review |
| `P19-RI-N3` | Missing and mismatched declared/content subject fixtures | Bound for acceptance review |
| `P19-RI-N4` | Unsupported algorithm and malformed digest fixtures | Bound for acceptance review |
| `P19-RI-N5` | Missing, duplicate, duplicate-unexpected, and unexpected fixtures | Bound for acceptance review |
| `P19-RI-N6` | Exact-byte recomputation mismatch fixture | Bound for acceptance review |
| `P19-RI-N7` | Validation receipt contract mismatch fixture | Bound for acceptance review |
| `P19-RI-N8` | Stage count, id, index, and order fixtures | Bound for acceptance review |
| `P19-RI-N9` | Stage algorithm, shape, and byte-recomputation fixtures | Bound for acceptance review |
| `P19-RI-D1` | Two-run complete positive outcome equality | Bound for acceptance review |
| `P19-RI-D2` | Shared two-run complete equality for every denial fixture | Bound for acceptance review |
| `P19-RI-R1` | Exact-SHA local targeted and workspace test PASS | Bound for acceptance review |
| `P19-RI-R2` | Exact-SHA strict freeze and full Dev Loop PASS | Bound for acceptance review |
| `P19-RI-B1` | One-file diff; production unwired; dependency/ABI/workflow/baseline unchanged | Bound for acceptance review |

No row in this table grants acceptance or merge authority.

## Local Exact-SHA Evidence

Local checks recorded with HEAD at
`e3028fee36d06efa23401184f21a4e4815f7757e`:

1. `cargo fmt --manifest-path userspace/phase19-admission-receipt/Cargo.toml --check` - PASS.
2. `cargo test --manifest-path userspace/Cargo.toml -p phase19-admission-receipt -- --test-threads=1` - PASS, 16 tests.
3. `cargo test --manifest-path userspace/Cargo.toml --workspace` - PASS on the final complete run.
4. `git diff --check 62d736cb..e3028fee` - PASS.
5. `git diff --name-only 62d736cb..e3028fee` - exactly
   `userspace/phase19-admission-receipt/src/lib.rs`.

An earlier local workspace attempt observed a timing-only
`semantic-cli` scalability-test outlier. The exact failing test passed on
immediate isolated retry, the final complete workspace rerun passed, and the
remote locked performance jobs passed. No baseline or threshold was changed.

Local PASS is an evidence input only. It is not acceptance or merge authority.

## Remote Exact-Head Evidence

Remote checks for implementation subject
`e3028fee36d06efa23401184f21a4e4815f7757e`:

1. PR #187 state at capture: open, draft, mergeable, one commit, one changed
   source file.
2. Strict `ci-freeze` run `27898387751`, job `82554084377` - PASS.
3. AykenOS Dev Loop CI run `27898387721` - PASS.
4. Dev Loop smoke job `82554084513` - PASS.
5. Dev Loop contract job `82554145703` - PASS.
6. Dev Loop full job `82554273133` - PASS.
7. Dev Loop isolation job `82554440300` - PASS.
8. Dev Loop locked performance job `82554577734` - PASS.
9. Dev Loop Validation run `27898387748` - PASS.
10. Dev Loop Optimized run `27898387713` - PASS.
11. Governance Summary run `27898387761` - PASS.
12. Spec Purity run `27898387789` - PASS.
13. Evidence Isolation runs `27898387726` and `27898387779` - PASS.
14. Workspace, Semantic CLI contract, execution-marker, ABI/freeze, and
    Phase-17 locked-performance gates in the exact-head check rollup - PASS.

Remote PASS is necessary evidence. It does not accept or merge PR #187.

## Acceptance Review Still Pending

The next governance layer must decide whether this package completely closes
`P19-RI-A1..A3`, `P19-RI-P1..P2`, `P19-RI-N1..N9`, `P19-RI-D1..D2`,
`P19-RI-R1..R2`, and `P19-RI-B1` for implementation subject `e3028fee...`.

Until that separate review is accepted:

1. PR #187 remains draft.
2. No merge authority exists.
3. Runtime activation and Phase-19 closure remain unauthorized.

## Evidence Package Conclusion

Implementation subject `e3028fee36d06efa23401184f21a4e4815f7757e`
preserves the one-file bounded source surface, corrects denial precedence,
and provides two-run complete parity for every negative fixture.

Local targeted and workspace validation pass. Exact-head strict freeze, full
Dev Loop, governance, isolation, spec-purity, workspace, ABI/freeze, and
locked-performance evidence pass.

The evidence matrix rows are bound for a separate acceptance review.
Acceptance, merge authority, runtime activation, general runtime authority,
and Phase-19 closure remain pending and unauthorized.
