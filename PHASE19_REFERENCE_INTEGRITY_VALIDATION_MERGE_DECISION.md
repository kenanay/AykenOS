# Phase-19 Reference Integrity Validation Merge Decision

This document is subordinate to PHASE 0 - FOUNDATIONAL OATH,
`ARCHITECTURE_FREEZE.md`, the Phase-18 Platform Constitution reference set,
`docs/specs/phase18-platform-constitution/AUTHORITY_DRIFT_GUARD.md`,
`docs/specs/phase18-platform-constitution/TERMINOLOGY_AUDIT.md`,
`PHASE19_RUNTIME_DECISION.md`, the Phase-19 Runtime RFC set,
`docs/specs/phase19-platform-runtime/RUNTIME_EVIDENCE_MATRIX.md`,
`PHASE19_RUNTIME_IMPLEMENTATION_POST_MERGE_CONSISTENCY_REVIEW.md`,
`PHASE19_REFERENCE_INTEGRITY_VALIDATION_DECISION_CANDIDATE.md`,
`PHASE19_REFERENCE_INTEGRITY_VALIDATION_IMPLEMENTATION_DECISION.md`,
`PHASE19_REFERENCE_INTEGRITY_VALIDATION_EVIDENCE_PACKAGE.md`, and
`PHASE19_REFERENCE_INTEGRITY_VALIDATION_ACCEPTANCE_REVIEW.md`. In case of
conflict, those documents prevail unless this decision is the narrower merge
decision for the exact implementation subject and PR identified below.

**Status:** MERGE DECISION / BOUNDED SOURCE PR MERGE CONDITIONALLY AUTHORIZED / DECISION-RECORD HEAD REMOTE PASS AND LIVE MAINTAINER ACTION REQUIRED / PR #187 NOT MERGED / RUNTIME ACTIVATION NOT AUTHORIZED
**Decision date:** 2026-06-21
**Decision id:** `ayken.phase19.reference_integrity_validation_merge_decision.v1`
**Implementation subject SHA:** `e3028fee36d06efa23401184f21a4e4815f7757e`
**Evidence package subject SHA:** `2f4e51e3aba757fc989be42e2eeed0f11d559fe8`
**Acceptance review subject SHA:** `3db66b8062c5664d0a20c09828c4428e23f21e55`
**Reviewed base SHA:** `62d736cbb8d859beaaa5a5960ff53ca480d5cd38`
**Implementation PR:** PR #187, draft and not merged at decision creation
**Evidence package PR:** PR #188, draft and not merged at decision creation
**Acceptance review PR:** PR #189, draft and not merged at decision creation
**Authority boundary:** Conditional bounded merge decision only; not merge
completion, not parser or resolver authority, not authenticity, not loader,
installer, mount, workspace runtime, execution, capability or trust issuance,
Semantic CLI or AI Runtime authority, syscall or kernel ABI change, runtime
activation, general runtime authority, or Phase-19 closure.

## Core Rule

```text
merge decision != merge completion
merge != runtime activation
merge != parser, resolver, loader, installer, or executor authority
remote PASS != live maintainer action
bounded merge != Phase-19 closure
```

This document decides only whether exact source subject `e3028fee...` may
proceed to a later live maintainer merge action when every condition below is
simultaneously true. It does not merge any PR by itself.

## Decision Record Subject Rule

The commit adding this decision is a new documentation subject. It must
receive its own strict freeze and full Dev Loop PASS before this conditional
merge authorization can be exercised.

The accepted implementation subject remains:

```text
e3028fee36d06efa23401184f21a4e4815f7757e
```

Documentation commits after the implementation subject do not create a new
implementation subject. Any later change to
`userspace/phase19-admission-receipt/src/lib.rs` invalidates this decision
until evidence and acceptance are regenerated for the new source SHA.

## Decision Inputs

| Input | Exact subject or run | Decision result |
|---|---|---|
| Implementation decision | `PHASE19_REFERENCE_INTEGRITY_VALIDATION_IMPLEMENTATION_DECISION.md` | One-file source boundary authorized |
| Implementation subject | `e3028fee36d06efa23401184f21a4e4815f7757e` | Accepted bounded subject |
| Implementation PR | PR #187 | Open, draft, mergeable, one commit, one changed file |
| Implementation strict freeze | Run `27898387751`, job `82554084377` | PASS |
| Implementation full Dev Loop | Run `27898387721` | PASS |
| Evidence package subject | `2f4e51e3aba757fc989be42e2eeed0f11d559fe8` | Matrix evidence bound |
| Evidence-package strict freeze | Run `27898814206`, job `82555228948` | PASS |
| Evidence-package full Dev Loop | Run `27898814186` | PASS |
| Acceptance review subject | `3db66b8062c5664d0a20c09828c4428e23f21e55` | Bounded acceptance granted |
| Acceptance-review strict freeze | Run `27899133489`, job `82556147125` | PASS |
| Acceptance-review full Dev Loop | Run `27899133510` | PASS |
| Acceptance-review full job | Job `82556471257` | PASS |
| Acceptance-review isolation job | Job `82556641450` | PASS |
| Acceptance-review locked performance job | Job `82556788608` | PASS |

No historical PASS from superseded implementation subject `b9b0f7b7...` is
used as current merge evidence.

## Merge Decision

PR #187 is conditionally authorized for bounded merge only when all of the
following remain simultaneously true:

1. The source PR head remains exactly
   `e3028fee36d06efa23401184f21a4e4815f7757e`.
2. The cumulative documentation head containing this decision receives
   strict freeze and full Dev Loop PASS.
3. Evidence package subject `2f4e51e3...` and acceptance subject
   `3db66b80...` remain the current reviewed inputs.
4. Current source and documentation checks remain PASS and are not stale,
   cancelled, or superseded.
5. All actionable source review threads are resolved against the exact source
   subject and no newer technical finding remains open.
6. PR #187 is changed from draft only after the current decision-record
   conditions are verified.
7. A live maintainer review/merge action is recorded for the current source
   PR head.
8. No source, dependency, workflow, baseline, syscall, ABI, production
   wiring, or authority expansion is introduced before merge.

Until every condition is true, the safe result is no source merge.

## Accepted Merge Scope

The conditionally accepted merge scope is only:

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

The accepted implementation includes:

1. The closed five-class canonical reference map.
2. Required local envelope, subject, digest algorithm, digest shape, content
   cardinality, and exact-byte digest checks.
3. The exact ten-entry `0..9` validation stage sequence.
4. Sixteen stable reference-integrity denial reasons.
5. Decision-compliant denial precedence.
6. Two-run complete outcome parity for every negative fixture.

## Documentation Integration Boundary

The evidence package, acceptance review, and this merge decision are kept on
a cumulative documentation branch separate from source PR #187. Integrating
that documentation branch records governance state; it does not merge or
activate the implementation.

The cumulative documentation PR may be integrated as the current decision
record after its own remote PASS. Earlier draft documentation PRs may then be
closed only as superseded by that cumulative record or merged in strict
ancestry order. Neither path changes source authority.

The source PR remains a one-file `lib.rs` PR and requires its own later live
maintainer action.

## Non-Authority Boundary

This decision and any later bounded source merge do not authorize:

1. General parsing or `Deserialize`-driven input acceptance.
2. Filesystem, URI, package, registry, or network resolution.
3. Publisher identity, signature validation, authenticity, or trust proof.
4. Phase-18 stage semantic evaluation or validation authority issuance.
5. Package installation or execution.
6. Module or plugin loading.
7. Workspace creation, runtime, or real mounts.
8. Capability or trust issuance.
9. Semantic CLI, AI Runtime, or agent authority.
10. New syscalls, kernel ABI expansion, or Ring0 policy.
11. Workflow authority, threshold changes, or baseline changes.
12. Runtime activation, general runtime authority, or Phase-19 closure.

## Post-Merge Requirement

If PR #187 is later merged, the next required artifact is a main exact-SHA
evidence/status synchronization record. It must bind:

1. PR #187 merge commit SHA.
2. Resulting exact `main` SHA.
3. Post-merge strict freeze and full Dev Loop results for that main SHA.
4. Confirmation that the merged source remains exact accepted subject
   `e3028fee...` or a mechanically identical merge descendant.
5. Confirmation that dependencies, workflows, baselines, syscall surface,
   ABI metadata, and production runtime wiring remain unchanged.
6. Confirmation that no prohibited authority was activated.

Post-merge synchronization is evidence/status maintenance. It is not runtime
activation, general runtime authority, or Phase-19 closure.

## Fail-Closed Conditions

Conditional merge authorization fails closed if:

1. This decision-record subject lacks required remote PASS.
2. Source PR #187 no longer points to `e3028fee...`.
3. Implementation source changes without new evidence and acceptance.
4. Evidence or acceptance subjects change without explicit re-binding.
5. A review thread or technical finding remains unresolved.
6. Required checks fail, become stale, or are cancelled.
7. Live maintainer action is absent or refers to an older source head.
8. Any accepted denial, precedence, determinism, ABI, or production-boundary
   property is removed or weakened.
9. Documentation integration is treated as source merge.
10. Source merge is treated as parser, resolver, authenticity, loader,
    installer, executor, runtime activation, general runtime authority, or
    Phase-19 closure.

Unknown authority readings fail closed.

## Decision Conclusion

PR #187 has conditional bounded merge authorization for exact implementation
subject `e3028fee36d06efa23401184f21a4e4815f7757e`.

The authorization cannot be exercised until this decision-record subject has
its own strict freeze and full Dev Loop PASS, all current review conditions
remain satisfied, and a live maintainer action is recorded for the exact
source head.

PR #187 is not merged by this document. Runtime activation, parser or
resolver authority, authenticity, general runtime authority, and Phase-19
closure remain unauthorized.
