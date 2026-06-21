# Phase-19 Reference Integrity Validation Acceptance Review

This document is subordinate to PHASE 0 - FOUNDATIONAL OATH,
`ARCHITECTURE_FREEZE.md`, the Phase-18 Platform Constitution reference set,
`docs/specs/phase18-platform-constitution/AUTHORITY_DRIFT_GUARD.md`,
`docs/specs/phase18-platform-constitution/TERMINOLOGY_AUDIT.md`,
`PHASE19_RUNTIME_DECISION.md`, the Phase-19 Runtime RFC set,
`docs/specs/phase19-platform-runtime/RUNTIME_EVIDENCE_MATRIX.md`,
`PHASE19_RUNTIME_IMPLEMENTATION_POST_MERGE_CONSISTENCY_REVIEW.md`,
`PHASE19_REFERENCE_INTEGRITY_VALIDATION_DECISION_CANDIDATE.md`,
`PHASE19_REFERENCE_INTEGRITY_VALIDATION_IMPLEMENTATION_DECISION.md`, and
`PHASE19_REFERENCE_INTEGRITY_VALIDATION_EVIDENCE_PACKAGE.md`. In case of
conflict, those documents prevail unless this review is the narrower
acceptance decision for the exact subjects identified below.

**Status:** ACCEPTANCE REVIEW / BOUNDED IMPLEMENTATION ACCEPTANCE GRANTED / MERGE NOT AUTHORIZED / RUNTIME ACTIVATION NOT AUTHORIZED / PHASE-19 NOT CLOSED
**Review date:** 2026-06-21
**Review id:** `ayken.phase19.reference_integrity_validation_acceptance_review.v1`
**Implementation subject SHA:** `e3028fee36d06efa23401184f21a4e4815f7757e`
**Evidence package subject SHA:** `2f4e51e3aba757fc989be42e2eeed0f11d559fe8`
**Implementation PR:** PR #187, draft and unmerged at review time
**Evidence package PR:** PR #188, draft and unmerged at review time
**Authority boundary:** Bounded acceptance for the exact implementation
subject only; not merge authority, not merge completion, not parser, resolver,
authenticity, signature, loader, installer, mount, workspace runtime,
execution, capability, trust, Semantic CLI, AI Runtime, syscall, kernel ABI,
runtime activation, general runtime authority, or Phase-19 closure.

## Core Rule

```text
bounded acceptance != merge authority
reference integrity acceptance != parser or resolver authority
digest recomputation acceptance != authenticity
stage-order acceptance != stage semantic execution
remote PASS != runtime activation
```

## Review Subject Rule

This review evaluates immutable remote subjects. Any change to
`userspace/phase19-admission-receipt/src/lib.rs` after implementation subject
`e3028fee36d06efa23401184f21a4e4815f7757e` invalidates this acceptance and
requires regenerated evidence plus a new acceptance review.

Any change to the evidence package after subject
`2f4e51e3aba757fc989be42e2eeed0f11d559fe8` requires review re-binding before
that changed package can support merge consideration.

This acceptance-review document must receive its own exact-head remote checks
before it can be used as the current accepted documentation record. Those
checks validate this document subject only and do not expand its authority.

## Review Inputs

| Review input | Exact subject or run | Result |
|---|---|---|
| Implementation decision | `PHASE19_REFERENCE_INTEGRITY_VALIDATION_IMPLEMENTATION_DECISION.md` | Bounded source behavior authorized |
| Implementation subject | `e3028fee36d06efa23401184f21a4e4815f7757e` | One source file |
| Source PR | PR #187 | Open, draft, mergeable, one commit, one changed file |
| Implementation strict freeze | Run `27898387751`, job `82554084377` | PASS |
| Implementation full Dev Loop | Run `27898387721` | PASS |
| Evidence package subject | `2f4e51e3aba757fc989be42e2eeed0f11d559fe8` | One documentation file |
| Evidence package PR | PR #188 | Open, draft, mergeable, one commit, one changed file |
| Evidence-package strict freeze | Run `27898814206`, job `82555228948` | PASS |
| Evidence-package full Dev Loop | Run `27898814186` | PASS |
| Evidence-package Dev Loop full | Job `82555440946` | PASS |
| Evidence-package isolation | Job `82555592270` | PASS |
| Evidence-package locked performance | Job `82555749046` | PASS |

Historical results for superseded implementation subject `b9b0f7b7...` are
not used to grant this acceptance.

## Acceptance Decision

Bounded implementation acceptance is granted for exactly:

```text
e3028fee36d06efa23401184f21a4e4815f7757e
```

The accepted behavior is limited to:

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

No parsing, file or URI access, network access, semantic evaluation,
installation, loading, mounting, execution, issuance, or runtime activation
is accepted.

## Architecture And Positive Evidence Review

| Matrix row | Review result |
|---|---|
| `P19-RI-A1` canonical map | SATISFIED: exact five-class map, contract ids, and envelope version |
| `P19-RI-A2` content binding | SATISFIED: exact declared/content cardinality with test-owned bytes |
| `P19-RI-A3` stage sequence | SATISFIED: exact ten-entry `0..9` id/index sequence |
| `P19-RI-P1` positive integrity | SATISFIED: canonical references and recomputed bytes permit only inert receipt flow |
| `P19-RI-P2` positive stage order | SATISFIED: exact stage sequence permits validation-integration record emission |

The positive lifecycle remains exactly:

```text
UNINITIALIZED
  -> INPUT_BOUND
  -> VALIDATING
  -> VALIDATED_RECORDABLE
  -> ADMISSION_RECORDED
  -> RECEIPT_EMITTED
```

Its bound transcript digest is
`sha256:9c25d061b39761189cd5e6268166d52639d4bb6531e17eaaa94e697fab5296ef`.

## Negative Evidence Review

| Matrix row | Reviewed denial coverage | Result |
|---|---|---|
| `P19-RI-N1` | Unknown contract and content classification | SATISFIED |
| `P19-RI-N2` | Unknown local typed-envelope version | SATISFIED |
| `P19-RI-N3` | Missing and mismatched declared/content subject | SATISFIED |
| `P19-RI-N4` | Unsupported algorithm and malformed digest | SATISFIED |
| `P19-RI-N5` | Missing, duplicate, and unexpected content | SATISFIED |
| `P19-RI-N6` | Reference content digest recomputation mismatch | SATISFIED |
| `P19-RI-N7` | Validation receipt contract mismatch | SATISFIED |
| `P19-RI-N8` | Stage count, id, index, and order mismatch | SATISFIED |
| `P19-RI-N9` | Stage algorithm, shape, and byte-recomputation mismatch | SATISFIED |

All sixteen stable denial reasons are distinct and bound to focused fixtures.
No reviewed denial emits a validation-integration success record, workspace
admission record, or runtime receipt.

## Denial Precedence Review

The amended subject closes the prior precedence finding.

The implementation checks content classification with canonical contract ids
before envelope version, subject, digest, and content-cardinality failures.
It checks supplied-content subject equality before algorithm, shape, and
cardinality failures. Combined-defect fixtures verify the externally visible
first-denial result.

The accepted precedence remains:

```text
contract/classification
  -> envelope version
  -> subject presence/equality
  -> digest algorithm/shape
  -> missing/duplicate/unexpected content
  -> content digest recomputation
  -> input binding
  -> validation presence/contract/version/subject
  -> stage count/id/index/order/digest
  -> existing stale/authority/status/admission guards
```

No separate `reference_class_mismatch` reason is required by the accepted
decision. A content classification mismatch remains correctly represented by
`unknown_reference_contract` at the contract/classification precedence layer.

## Determinism Review

| Matrix row | Review result |
|---|---|
| `P19-RI-D1` positive determinism | SATISFIED: complete positive outcome compared across two runs |
| `P19-RI-D2` denial determinism | SATISFIED: every negative fixture executes twice and compares complete outcomes |

Complete denial-outcome equality includes reason, transcript, transcript
digest, optional input digest, and absence of success records. The shared
two-run helper applies to all denial fixtures, not only one denial class.

## Exact-SHA And Production Boundary Review

| Matrix row | Review result |
|---|---|
| `P19-RI-R1` local evidence | SATISFIED: format, targeted 16-test suite, final full userspace workspace suite, and diff checks PASS |
| `P19-RI-R2` remote evidence | SATISFIED: implementation and evidence-package strict freeze plus full Dev Loop PASS |
| `P19-RI-B1` production boundary | SATISFIED: source diff is one `lib.rs`; production unwired; dependencies, workflows, baselines, syscall surface, and ABI metadata unchanged |

The implementation retains only existing `serde`, `serde_json`, and `sha2`
dependencies. Repository search shows no production call site for
`run_harness`.

## Merge Boundary

This review does not authorize merging PR #187 or PR #188.

A separate merge decision must verify at least:

1. Implementation subject `e3028fee...` remains the exact source PR subject.
2. Evidence package subject `2f4e51e3...` remains the reviewed evidence input.
3. This acceptance-review subject has its own exact-head remote PASS.
4. PR review threads are resolved and no newer source finding exists.
5. Required checks remain PASS at merge consideration time.
6. Merge is not interpreted as runtime activation or Phase-19 closure.

PR #187 and PR #188 remain draft pending that separate merge decision.

## Fail-Closed Conditions

This acceptance fails closed if:

1. Implementation source changes after `e3028fee...`.
2. Any canonical contract, envelope, subject, digest, cardinality, or stage
   check is removed or weakened.
3. Denial precedence no longer matches the accepted decision.
4. Any negative fixture stops providing two-run complete parity.
5. A denial emits a success record or runtime receipt.
6. Production wiring, dependencies, workflow authority, baselines, syscall
   surface, or ABI metadata change.
7. Exact-head remote evidence becomes stale or failing.
8. Acceptance is interpreted as merge authority, parser or resolver
   authority, authenticity, runtime activation, general runtime authority, or
   Phase-19 closure.

Unknown authority readings fail closed.

## Review Conclusion

All required reference-integrity architecture, positive, negative,
determinism, exact-SHA, and production-boundary rows are satisfied for exact
implementation subject `e3028fee36d06efa23401184f21a4e4815f7757e`.

Bounded implementation acceptance is granted for that subject only.

Merge authority, merge completion, parser or resolver authority, runtime
activation, general runtime authority, and Phase-19 closure remain separate
and unauthorized by this review.
