# Phase-19 Reference Integrity Validation Merge Decision Rebind

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
`PHASE19_REFERENCE_INTEGRITY_VALIDATION_ACCEPTANCE_REVIEW.md`, and
`PHASE19_REFERENCE_INTEGRITY_VALIDATION_MERGE_DECISION.md`. In case of
conflict, this document is the narrower rebind record only for the final
PR #187 head, the intervening baseline renewal, and the post-merge sync
identified below.

**Status:** MERGE DECISION REBIND / FINAL PR HEAD RECORDED / BASELINE
RENEWAL PREREQUISITE RECORDED / POST-MERGE SYNC MAY RELY ON THIS REBIND /
NO NEW IMPLEMENTATION SUBJECT / RUNTIME ACTIVATION NOT AUTHORIZED /
PHASE-19 NOT CLOSED
**Rebind date:** 2026-06-27
**Rebind id:** `ayken.phase19.reference_integrity_validation_merge_decision_rebind.v1`
**Accepted implementation subject SHA:** `e3028fee36d06efa23401184f21a4e4815f7757e`
**Final PR #187 head SHA:** `9c928d8c77997c9127dc9769f65d003f44a7c0d8`
**PR #187 merge commit / main subject SHA:** `c82fe5f6a154f6d78708a5b94fa9b5dc367c02de`
**Baseline renewal merge commit:** `8f7bf671ab42a205f1b66f9f3dbff5d9c454de03`
**Authority boundary:** Merge-decision rebind only; not a new source
implementation decision, not a new implementation subject, not a source
acceptance review for another subject, not baseline inflation authority, not
runtime activation, not general runtime authority, and not Phase-19 closure.

## Core Rule

```text
branch-update merge commit != new implementation subject
governed baseline renewal != PR #187 source scope expansion
post-merge sync != retroactive runtime activation
rebind != broad merge authority
```

This document narrows how
`PHASE19_REFERENCE_INTEGRITY_VALIDATION_MERGE_DECISION.md` is read for the
actual PR #187 live merge path after two external governance facts changed:

1. The source PR had to be brought up to date with `main` before GitHub would
   permit normal protected-branch merge.
2. GitHub Actions runner image drift required a separate governed Phase-17
   performance baseline renewal before PR #187 could obtain current locked
   performance evidence.

This rebind does not change the accepted implementation subject:

```text
e3028fee36d06efa23401184f21a4e4815f7757e
```

## Rebind Trigger

The original merge decision conditionally authorized bounded merge while the
source PR head remained exactly `e3028fee...` and while no baseline change was
introduced before merge.

Those conditions failed as literal global statements once:

1. The PR #187 branch received maintainer branch-update merge commits, ending
   at head `9c928d8c77997c9127dc9769f65d003f44a7c0d8`.
2. PR #191 renewed `scripts/ci/perf-baseline.lock.json` to the current
   GitHub-hosted runner image digest before PR #187 merged.

Failing the literal conditions did not create a new implementation subject,
but it did require this explicit rebind before any main exact-SHA sync could
claim the merge-completion requirement is closed.

## Final Head Rebind

The original source-head condition is re-bound from:

```text
PR #187 head == e3028fee36d06efa23401184f21a4e4815f7757e
```

to the narrower live-merge condition:

```text
PR #187 final head may be a maintainer branch-update descendant of
e3028fee36d06efa23401184f21a4e4815f7757e only if:

1. the accepted implementation commit remains present in the PR history;
2. the PR file delta at merge remains exactly
   userspace/phase19-admission-receipt/src/lib.rs;
3. no later commit changes that source file beyond the accepted implementation
   subject;
4. the final PR head receives current strict freeze, full Dev Loop, locked
   performance, governance, spec, and evidence PASS; and
5. the post-merge main subject receives exact-SHA remote PASS.
```

The final PR #187 head satisfied those conditions:

| Item | Recorded result |
|---|---|
| Accepted implementation commit present | `e3028fee36d06efa23401184f21a4e4815f7757e` |
| Final PR head | `9c928d8c77997c9127dc9769f65d003f44a7c0d8` |
| PR #187 file delta at merge | `userspace/phase19-admission-receipt/src/lib.rs` |
| Final PR #187 state before merge | ready, mergeable, non-draft |
| Merge commit / main subject | `c82fe5f6a154f6d78708a5b94fa9b5dc367c02de` |

This rebind preserves the accepted implementation subject while allowing the
source PR head to be the protected-branch mergeable descendant GitHub
required.

## Baseline Renewal Rebind

The original no-baseline-change condition is re-bound from a global
repository statement to the source-PR boundary:

```text
PR #187 must not change baselines, workflows, thresholds, dependencies,
syscalls, ABI metadata, or production runtime wiring.
```

PR #187 satisfied that narrowed condition. Its file delta at merge remained
only:

```text
userspace/phase19-admission-receipt/src/lib.rs
```

The repository baseline did change before PR #187 merged, but only through a
separate governed renewal PR:

| Item | Recorded result |
|---|---|
| Renewal trigger | GitHub Actions runner image digest drift |
| Old expected digest | `gha-ubuntu24-20260615.205.1-X64` |
| New current digest | `gha-ubuntu24-20260622.220.1-X64` |
| Authorized init workflow | `perf-baseline-init.yml` run `28274941926` |
| Imported artifact SHA-256 | `f5798af16b0710221c5c821ce72176f887c6479ea3f3177f8330846c99e20046` |
| Baseline renewal PR | PR #191 |
| Baseline renewal head | `71dddce0e640b2c03570f22e2722f4e3bab5c3ed` |
| Baseline renewal merge commit | `8f7bf671ab42a205f1b66f9f3dbff5d9c454de03` |
| Changed file | `scripts/ci/perf-baseline.lock.json` |

This baseline renewal was a prerequisite for current locked performance
authority. It did not change PR #187 source scope, did not alter thresholds,
and did not authorize performance baseline inflation.

## Final Pre-Merge Evidence Accepted By This Rebind

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

These runs are current final-head evidence. They supersede stale readings of
the earlier PR #187 head only for merge-readiness purposes. They do not
create a new implementation subject and do not replace the accepted source
evidence for `e3028fee...`.

## Post-Merge Evidence Accepted By This Rebind

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

## Rebind Decision

For PR #187 only, the original merge decision is re-bound as follows:

1. The accepted implementation subject remains exactly `e3028fee...`.
2. The final PR head `9c928d8c...` is accepted as a protected-branch
   mergeable descendant of the accepted implementation subject.
3. PR #187 is confirmed not to introduce baseline, workflow, dependency,
   syscall, ABI, or production runtime wiring changes.
4. PR #191 is accepted as a separate governed baseline-renewal prerequisite
   for current runner image authority.
5. Main exact-SHA evidence sync may rely on final PR head `9c928d8c...` and
   main subject `c82fe5f6...`.

This rebind does not authorize another source PR, another implementation
subject, or any runtime activation.

## Non-Authority Boundary

This rebind does not authorize:

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
11. Workflow authority or threshold changes.
12. Runtime activation, general runtime authority, or Phase-19 closure.

Unknown authority readings fail closed.

## Rebind Conclusion

The Phase-19 reference-integrity validation merge decision is re-bound for
the actual protected-branch merge path:

```text
e3028fee... accepted implementation subject
  -> 9c928d8c... final PR #187 head
  -> c82fe5f6... merged main subject
```

The intervening PR #191 baseline renewal is recorded as a separate governed
CI authority prerequisite, not as PR #187 source scope expansion.

This rebind permits the main exact-SHA evidence sync to close the
merge-completion evidence gap for PR #187. Runtime activation, general
runtime authority, and Phase-19 closure remain unauthorized.
