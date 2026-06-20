# Phase-19 Runtime Implementation Review Findings Update

This document is subordinate to PHASE 0 - FOUNDATIONAL OATH,
`ARCHITECTURE_FREEZE.md`, the Phase-18 Platform Constitution reference set,
`PHASE19_RUNTIME_DECISION.md`, the Phase-19 Runtime RFC set,
`PHASE19_RUNTIME_IMPLEMENTATION_DECISION_PACKAGE.md`,
`PHASE19_RUNTIME_IMPLEMENTATION_EVIDENCE_PACKAGE_REBIND.md`,
`PHASE19_RUNTIME_IMPLEMENTATION_ACCEPTANCE_REVIEW_FINAL.md`,
`PHASE19_RUNTIME_IMPLEMENTATION_MERGE_REVIEW.md`, and
`PHASE19_RUNTIME_IMPLEMENTATION_MERGE_DECISION.md`. In case of conflict,
those documents prevail unless this record is the narrower implementation
update for the subject identified below.

**Status:** IMPLEMENTATION SUBJECT UPDATE / PREVIOUS MERGE DECISION FAIL-CLOSED / EVIDENCE RE-BIND REQUIRED / ACCEPTANCE NOT GRANTED FOR UPDATED SUBJECT / PR #181 NOT MERGED
**Update date:** 2026-06-20
**Update id:** `ayken.phase19.runtime_implementation_review_findings_update.v1`
**Previous accepted implementation subject SHA:** `64fa476256e5572f91661f717f1312abcc6daf0d`
**Updated implementation subject SHA:** `0a067dbaa230838e2c14e1e1f0bd91494092713e`
**Implementation PR:** PR #181
**Authority boundary:** Bounded review-finding implementation update only;
not evidence acceptance, not acceptance review, not merge authority, not
runtime activation, not general runtime authority, and not Phase-19 closure.

## Core Rule

```text
review finding != merge waiver
source change != inherited acceptance
updated subject != runtime expansion
remote PASS != acceptance
```

The attempted PR #181 merge was blocked by unresolved review conversations.
Both findings were reviewed against the accepted Phase-19 RFC set and were
confirmed as valid fail-closed gaps.

The previous merge decision cannot be exercised for the updated source. It
remains a historical decision for its reviewed subject and fails closed after
the implementation change recorded here.

## Confirmed Findings

| Finding | Normative requirement | Previous behavior | Updated behavior |
|---|---|---|---|
| Workspace declaration stale or subject-mismatched | Workspace admission must deny stale declarations and subject-binding mismatch | Presence was checked, but stale and subject fields were not evaluated | Denied before input binding with distinct stable reason classes |
| Validation receipt schema version unknown | Validation integration must check a known receipt schema version | Contract and subject were checked, but schema version was not evaluated | Denied after input binding and before validation/admission/receipt emission |

These findings concern the bounded admission/receipt harness only. They do
not authorize parser, loader, installer, executor, workspace runtime, plugin
host, issuer, Semantic CLI, AI Runtime, syscall, or kernel ABI work.

## Updated Subject Scope

The updated implementation subject changes only:

```text
userspace/phase19-admission-receipt/src/lib.rs
```

The update adds:

1. Known validation receipt schema version constant `1`.
2. `stale_workspace_declaration` denial.
3. `workspace_declaration_subject_mismatch` denial.
4. `unknown_validation_schema_version` denial.
5. Pre-input workspace declaration stale and subject-binding checks.
6. Input-bound validation schema version check.
7. Targeted unit assertions for all three denial classes.

The update does not add:

1. General manifest parsing.
2. Package installation or execution.
3. Module or plugin loading.
4. Workspace creation, runtime, or real mounts.
5. Capability or trust issuance.
6. Registry behavior.
7. Semantic CLI or AI Runtime authority.
8. New syscalls or kernel ABI expansion.
9. Ring0 policy.
10. Workflow, threshold, or baseline changes.

## Fail-Closed Placement

Workspace declaration reference checks occur before input binding. A stale or
subject-mismatched declaration therefore emits no input digest, validation
integration record, workspace admission record, or successful runtime
receipt.

Validation schema version checking occurs after deterministic input binding
and before validation recordability. An unknown schema version emits the
input-bound denial transcript but no validation integration record, workspace
admission record, or successful runtime receipt.

## Local Validation

For updated implementation subject `0a067dba`:

1. `cargo test --manifest-path userspace/Cargo.toml -p phase19-admission-receipt -- --test-threads=1` - PASS, 7 tests.
2. `make ci-gate-abi RUN_ID=local-phase19-review-fix-abi-20260620 EVIDENCE_ROOT=evidence` - PASS.
3. `make ci-gate-hygiene RUN_ID=local-phase19-review-fix-hygiene-clean-20260620 EVIDENCE_ROOT=evidence` - PASS.
4. `make ci-gate-governance RUN_ID=local-phase19-review-fix-governance-20260620 EVIDENCE_ROOT=evidence` - PASS.
5. `make ci-gate-workspace RUN_ID=local-phase19-review-fix-workspace-clean-20260620 EVIDENCE_ROOT=evidence` - PASS.
6. `git diff --check` - PASS.

Local PASS is evidence input only.

## Remote Subject Validation

Remote checks for exact implementation subject
`0a067dbaa230838e2c14e1e1f0bd91494092713e`:

1. `ci-freeze` run `27868634546` - PASS.
2. Locked Phase-17 performance acceptance run `27868634553` - PASS.
3. Dev Loop CI run `27868634530` - smoke, contract, full, isolation, and performance PASS.
4. Dev Loop Optimized run `27868634515` - PASS.
5. Dev Loop Validation run `27868634535` - PASS.
6. Governance, evidence isolation, naming, spec purity, observation, Phase-17 runtime, and WS 3.x checks - PASS.

Remote PASS does not restore acceptance or merge authority by itself.

## Prior Decision Invalidation

`PHASE19_RUNTIME_IMPLEMENTATION_MERGE_DECISION.md` required no implementation
source changes after subject `64fa4762`. Subject `0a067dba` intentionally
changes that source to close confirmed review findings. Therefore:

```text
previous merge decision != authority for subject 0a067dba
```

Evidence must be re-bound and separately reviewed before a new merge decision
can be considered.

## Update Conclusion

The two confirmed review findings are addressed in bounded implementation
subject `0a067dba`.

Acceptance and merge authority are not granted. The next layer is an exact-SHA
evidence re-bind for the updated subject.
