# Phase-19 Reference Integrity Validation Main Exact-SHA Evidence Sync

This document is subordinate to PHASE 0 - FOUNDATIONAL OATH,
`ARCHITECTURE_FREEZE.md`, the Phase-18 Platform Constitution reference set,
`docs/specs/phase18-platform-constitution/AUTHORITY_DRIFT_GUARD.md`,
`docs/specs/phase18-platform-constitution/TERMINOLOGY_AUDIT.md`,
`PHASE19_RUNTIME_DECISION.md`, the Phase-19 Runtime RFC set,
`docs/specs/phase19-platform-runtime/RUNTIME_EVIDENCE_MATRIX.md`,
`PHASE19_RUNTIME_IMPLEMENTATION_POST_MERGE_CONSISTENCY_REVIEW.md`,
`PHASE19_REFERENCE_INTEGRITY_VALIDATION_DECISION_CANDIDATE.md`,
`PHASE19_REFERENCE_INTEGRITY_VALIDATION_IMPLEMENTATION_DECISION.md`,
`PHASE19_REFERENCE_INTEGRITY_VALIDATION_EVIDENCE_PACKAGE.md`,
`PHASE19_REFERENCE_INTEGRITY_VALIDATION_ACCEPTANCE_REVIEW.md`,
`PHASE19_REFERENCE_INTEGRITY_VALIDATION_MERGE_DECISION.md`, and
`PHASE19_REFERENCE_INTEGRITY_VALIDATION_MERGE_DECISION_REBIND.md`.

**Status:** MAIN EXACT-SHA EVIDENCE SYNC / MERGE DECISION REBIND RECORDED /
PR #187 MERGED / POST-MERGE REMOTE PASS / BOUNDED IMPLEMENTATION ONLY /
RUNTIME ACTIVATION NOT AUTHORIZED / PHASE-19 NOT CLOSED
**Sync date:** 2026-06-27
**Sync id:** `ayken.phase19.reference_integrity_validation_main_exact_sha_evidence_sync.v1`
**Accepted implementation subject SHA:** `e3028fee36d06efa23401184f21a4e4815f7757e`
**Merged PR head SHA:** `9c928d8c77997c9127dc9769f65d003f44a7c0d8`
**Merge commit / main subject SHA:** `c82fe5f6a154f6d78708a5b94fa9b5dc367c02de`
**Implementation PR:** PR #187, merged 2026-06-27
**Authority boundary:** Post-merge exact-SHA evidence and status
synchronization only; not a new implementation decision, not a new evidence
package, not acceptance for another subject, not runtime activation, not
general runtime authority, and not Phase-19 closure.

## Core Rule

```text
merge completion != runtime activation
post-merge PASS != general runtime authority
main evidence sync != Phase-19 closure
baseline renewal prerequisite != implementation scope expansion
```

This record closes the merge-completion and post-merge exact-SHA evidence gap
required by `PHASE19_REFERENCE_INTEGRITY_VALIDATION_MERGE_DECISION.md` only
as re-bound by
`PHASE19_REFERENCE_INTEGRITY_VALIDATION_MERGE_DECISION_REBIND.md`.

It does not expand the accepted bounded implementation and does not rebind
the evidence package to a different implementation subject.

## Rebind Dependency

Two live-merge facts required an explicit merge-decision rebind before this
sync could be treated as closed:

1. PR #187 final head was `9c928d8c...`, a protected-branch mergeable
   descendant of accepted implementation subject `e3028fee...`, not the
   original single-commit source head itself.
2. PR #191 renewed `scripts/ci/perf-baseline.lock.json` before PR #187 merged
   so current locked performance evidence could run under the updated
   GitHub-hosted runner image digest.

`PHASE19_REFERENCE_INTEGRITY_VALIDATION_MERGE_DECISION_REBIND.md` records
both facts and narrows the original merge-decision conditions accordingly.
Without that rebind, this sync would not be sufficient to close the
merge-completion evidence requirement.

## Merge Record

| Input | Recorded result |
|---|---|
| Accepted implementation subject | `e3028fee36d06efa23401184f21a4e4815f7757e` |
| Final PR head at merge | `9c928d8c77997c9127dc9769f65d003f44a7c0d8` |
| PR result | PR #187 merged |
| Merge commit | `c82fe5f6a154f6d78708a5b94fa9b5dc367c02de` |
| Resulting main subject | `c82fe5f6a154f6d78708a5b94fa9b5dc367c02de` |
| Merge timestamp | 2026-06-27T02:15:42Z |

The final PR head is accepted through the merge-decision rebind. It includes
the accepted source commit
`e3028fee36d06efa23401184f21a4e4815f7757e` and two maintainer branch-update
merge commits. The pull-request file delta at merge remained exactly:

```text
userspace/phase19-admission-receipt/src/lib.rs
```

## Baseline Renewal Prerequisite

Before PR #187 could be merged, GitHub Actions runner image authority drift
caused the Phase-17 locked baseline acceptance check to fail with:

```text
source_ci_image_digest:
expected=gha-ubuntu24-20260615.205.1-X64
actual=gha-ubuntu24-20260622.220.1-X64
```

That failure was not caused by the Phase-19 reference-integrity source
change. It was resolved by a separate governed baseline renewal, and that
intervening baseline renewal is authorized for this sync only through
`PHASE19_REFERENCE_INTEGRITY_VALIDATION_MERGE_DECISION_REBIND.md`:

| Item | Recorded result |
|---|---|
| Baseline init workflow | `perf-baseline-init.yml` run `28274941926` |
| Imported artifact SHA-256 | `f5798af16b0710221c5c821ce72176f887c6479ea3f3177f8330846c99e20046` |
| Baseline renewal PR | PR #191 |
| Baseline renewal head | `71dddce0e640b2c03570f22e2722f4e3bab5c3ed` |
| Baseline renewal merge commit | `8f7bf671ab42a205f1b66f9f3dbff5d9c454de03` |
| Renewed `ci_image_digest` | `gha-ubuntu24-20260622.220.1-X64` |

The baseline renewal changed only
`scripts/ci/perf-baseline.lock.json`. It is not part of the reference
integrity implementation subject and does not expand PR #187 scope. PR #187
itself still introduced no baseline, workflow, dependency, syscall, ABI, or
production wiring change.

## Final Pre-Merge Exact-Head Evidence

All rows below are bound to final PR #187 head:

```text
9c928d8c77997c9127dc9769f65d003f44a7c0d8
```

| Evidence | Run | Result |
|---|---|---|
| Strict `ci-freeze` | `28275306612` | PASS |
| AykenOS Dev Loop CI smoke / contract / full / isolation / performance | `28275306654` | PASS |
| Dev Loop optimized | `28275306614` | PASS |
| Dev Loop validation | `28275306627` | PASS |
| Phase-17 locked baseline performance acceptance | `28275306617` | PASS |
| Governance Summary | `28275306591` | PASS |
| Spec Purity | `28275306588` | PASS |
| Evidence Isolation | `28275306580`, `28275306629` | PASS |

The final exact-head PR evidence restored the required locked baseline
performance PASS after the governed baseline renewal landed.

## Post-Merge Exact-SHA Evidence

All rows below are bound to main subject:

```text
c82fe5f6a154f6d78708a5b94fa9b5dc367c02de
```

| Evidence | Run / job | Result |
|---|---|---|
| Strict `ci-freeze` | run `28275597374`, job `83781542616` | PASS |
| AykenOS Dev Loop CI smoke | run `28275597383`, job `83781542702` | PASS |
| AykenOS Dev Loop CI contract | run `28275597383`, job `83781602350` | PASS |
| AykenOS Dev Loop CI full | run `28275597383`, job `83781737756` | PASS |
| AykenOS Dev Loop CI isolation | run `28275597383`, job `83781899973` | PASS |
| AykenOS Dev Loop CI performance | run `28275597383`, job `83782048436` | PASS |
| Dev Loop optimized | run `28275597393` | PASS |
| Dev Loop validation | run `28275597395` | PASS |
| Governance Summary | run `28275597376` | PASS |
| Spec Purity | run `28275597397` | PASS |
| Evidence Isolation | runs `28275597388`, `28275597390` | PASS |
| Observation Boundary | run `28275597414` | PASS |
| Naming Compliance | runs `28275597365`, `28275597372` | PASS |

Supporting workspace and boundary push workflows for the same main subject
also completed successfully.

## Merged Scope

The merged implementation remains limited to the accepted userspace
reference-integrity validation slice:

```text
typed static bundle
  -> typed test-owned reference content binding
  -> canonical contract/schema/subject checks
  -> SHA-256 content digest recomputation
  -> structured Phase-18 validation stage-order checks
  -> existing inert admission/receipt record emission
```

It remains implemented only in:

```text
userspace/phase19-admission-receipt/src/lib.rs
```

The accepted implementation does not add parser authority, filesystem or
network resolution, package retrieval, registry access, installation,
loading, mounting, execution, capability or trust issuance, workflow
authority, kernel ABI changes, syscall changes, runtime activation, or
Phase-19 closure.

## Non-Authority Boundary

PR #187 merge and this evidence sync do not authorize:

1. General manifest parsing.
2. Filesystem, URI, registry, or package resolution.
3. Package installation or execution.
4. Module or plugin loading.
5. Workspace runtime, creation, or real mounts.
6. Capability or trust issuance.
7. Semantic CLI authority.
8. AI Runtime or agent authority.
9. New syscalls, kernel ABI expansion, or Ring0 policy.
10. Runtime activation.
11. General runtime authority.
12. Phase-19 closure.

## Status Consequence

PR #187 is merged and, after the merge-decision rebind, its post-merge
exact-SHA evidence requirement is satisfied on `c82fe5f6`.

The decision candidate, implementation decision, evidence package,
acceptance review, and merge decision remain historical authority inputs for
this bounded merge only. They do not authorize another implementation
subject or a broader runtime surface.

Any later Phase-19 implementation expansion requires a separate reviewed
decision, evidence package, acceptance boundary, and merge decision.

## Sync Conclusion

The Phase-19 reference-integrity validation implementation is merged and
post-merge verified on exact main SHA
`c82fe5f6a154f6d78708a5b94fa9b5dc367c02de`.

Runtime activation, general runtime authority, and Phase-19 closure remain
unauthorized.
