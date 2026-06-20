# Phase-19 Runtime Implementation Reason-Class Update

This document is subordinate to PHASE 0 - FOUNDATIONAL OATH,
`ARCHITECTURE_FREEZE.md`, `PHASE18_TRANSITION_DECISION.md`,
`PHASE18_ACTIVATION_DECISION.md`, the Phase-18 Platform Constitution
reference set, `AUTHORITY_DRIFT_GUARD.md`, `TERMINOLOGY_AUDIT.md`,
`PHASE19_RUNTIME_DECISION.md`, the Phase-19 Runtime RFC set,
`docs/specs/phase19-platform-runtime/RUNTIME_EVIDENCE_MATRIX.md`,
`docs/specs/phase19-platform-runtime/CROSS_CONSISTENCY_REVIEW.md`,
`PHASE19_POINTER_TRANSITION_DECISION.md`,
`PHASE19_RUNTIME_IMPLEMENTATION_DECISION_PACKAGE.md`,
`PHASE19_RUNTIME_IMPLEMENTATION_EVIDENCE_PACKAGE.md`,
`PHASE19_RUNTIME_IMPLEMENTATION_ACCEPTANCE_REVIEW.md`,
`PHASE19_RUNTIME_IMPLEMENTATION_ADDITIONAL_TRANSCRIPT_EVIDENCE.md`, and
`PHASE19_RUNTIME_IMPLEMENTATION_ACCEPTANCE_REVIEW_UPDATE.md`, and
`PHASE19_RUNTIME_IMPLEMENTATION_ACCEPTANCE_REVIEW_FINAL.md`. In case of
conflict, those documents prevail unless this record is the narrower
implementation-update record for the subject identified below.

**Status:** IMPLEMENTATION SUBJECT UPDATE / EVIDENCE RE-BIND RECORDED SEPARATELY / FINAL ACCEPTANCE REVIEW RECORDED SEPARATELY / MERGE NOT AUTHORIZED
**Update date:** 2026-06-14
**Update id:** `ayken.phase19.runtime_implementation_reason_class_update.v1`
**Previous implementation subject SHA:** `22d5e86a1306f1d0cccc2cdf9772eac93003b372`
**Updated implementation subject SHA:** `64fa476256e5572f91661f717f1312abcc6daf0d`
**Implementation PR:** PR #181, draft at update time
**Authority boundary:** Bounded implementation subject update only; not
evidence package, not acceptance review, not acceptance, not merge authority,
not runtime activation, not a general runtime, not a manifest parser, not a
package installer, not a module loader, not package execution, not workspace
runtime, not workspace creation, not real mount authority, not plugin host,
not plugin loading, not capability token minting, not capability issuance, not
trust assignment, not registry publication, not Semantic CLI authority, not AI
Runtime authority, not agent authority, not a syscall, not kernel ABI
expansion, not Ring0 policy, and not closure authority.

## Core Rule

```text
reason-class update != evidence package
reason-class update != acceptance review
reason-class update != acceptance
new implementation subject != runtime expansion
remote PASS != acceptance
```

This update addresses only the reason-class granularity blocker identified by
`PHASE19_RUNTIME_IMPLEMENTATION_ACCEPTANCE_REVIEW_UPDATE.md`.

It does not grant acceptance.

## Update Decision

The previous implementation subject mapped two semantically distinct
validation receipt failures to `subject_mismatch`:

1. Validation receipt stale digest.
2. Validation receipt unknown stage.

The updated implementation subject separates them into stable denial reason
classes:

```text
validation_stale_digest
unknown_validation_stage
```

The bounded harness still denies both surfaces after input binding and before
validation integration record emission, workspace admission record emission,
or runtime receipt success.

## Scope

The updated subject changes only:

```text
userspace/phase19-admission-receipt/src/lib.rs
```

The update adds:

1. `ValidationStaleDigest`.
2. `UnknownValidationStage`.
3. Targeted fail-closed branches for `PlatformValidationEvidence.stale_digest`
   and `PlatformValidationEvidence.unknown_stage_observed`.
4. Unit assertions that those two inputs deny with distinct reason classes.

The update does not add:

1. General manifest parsing.
2. Package installation.
3. Package execution.
4. Module loading.
5. Workspace runtime or real mounts.
6. Plugin host or plugin loading.
7. Capability token minting or issuance.
8. Trust assignment.
9. Registry behavior.
10. Semantic CLI authority.
11. AI Runtime authority.
12. Agent behavior.
13. New syscalls.
14. Kernel ABI expansion.
15. Ring0 policy.
16. CI workflow authority changes.
17. Baseline changes.

## Evidence Impact

The old evidence package remains historical evidence for implementation
subject:

```text
22d5e86a1306f1d0cccc2cdf9772eac93003b372
```

It is no longer sufficient as acceptance evidence for the updated
implementation subject:

```text
64fa476256e5572f91661f717f1312abcc6daf0d
```

`PHASE19_RUNTIME_IMPLEMENTATION_EVIDENCE_PACKAGE_REBIND.md` later records
review input for the updated subject.

`PHASE19_RUNTIME_IMPLEMENTATION_ACCEPTANCE_REVIEW_FINAL.md` records the later
final bounded acceptance review for the updated subject. This update itself
does not grant acceptance or merge authority.

The updated subject must remain bound to:

1. Positive transcript evidence.
2. Negative denial transcript evidence.
3. Deterministic repeat digest evidence.
4. Denial-repeat digest evidence.
5. Production-default proof.
6. ABI freeze proof.
7. Remote exact-SHA PASS.

## Local Evidence

Local evidence recorded before this update record:

```text
cargo test --manifest-path userspace/Cargo.toml -p phase19-admission-receipt -- --test-threads=1
```

Result: PASS, 6 tests.

Local PASS is evidence input only. It is not acceptance, merge authority,
runtime activation, or closure authority.

## PR State

PR #181 must remain draft after this implementation update.

This update does not approve:

1. Marking PR #181 ready for review.
2. Merging PR #181.
3. Runtime activation.
4. Acceptance of the bounded implementation.
5. Closure of Phase-19.

The next acceptance layer must review the re-bound exact-SHA evidence before
acceptance can be reconsidered. The later final acceptance review records
that decision separately.

## Non-Authority Rule

This update must not be read to authorize:

1. General runtime behavior.
2. General manifest parsing.
3. Package installation.
4. Package execution.
5. Module loading.
6. Workspace runtime or real mounts.
7. Plugin host or plugin loading.
8. Capability token minting or issuance.
9. Trust assignment.
10. Registry behavior.
11. Semantic CLI authority.
12. AI Runtime authority.
13. Agent behavior.
14. New syscalls.
15. Kernel ABI expansion.
16. Ring0 policy.
17. Merge or closure authority.

Unknown authority readings fail closed.

## Update Conclusion

The reason-class granularity blocker identified by the acceptance review
update is addressed in a bounded implementation subject.

This update itself does not grant acceptance.

Evidence re-binding and final acceptance review are recorded separately.
Merge authority remains separate.
