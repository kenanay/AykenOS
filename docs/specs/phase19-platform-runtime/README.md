# Phase-19 Platform Runtime RFC Set

This directory is subordinate to PHASE 0 - FOUNDATIONAL OATH,
`ARCHITECTURE_FREEZE.md`, `PHASE18_TRANSITION_DECISION.md`,
`PHASE18_ACTIVATION_DECISION.md`, the Phase-18 Platform Constitution
reference set, `AUTHORITY_DRIFT_GUARD.md`, `TERMINOLOGY_AUDIT.md`,
`PHASE19_RUNTIME_DECISION.md`, and
`../../../PHASE19_POINTER_TRANSITION_DECISION.md`. In case of conflict, those
documents prevail.

**Status:** ACTIVE PHASE-19 RFC SET / RUNTIME IMPLEMENTATION NOT AUTHORIZED
**Authority basis:** `PHASE19_RUNTIME_DECISION.md` and
`../../../PHASE19_POINTER_TRANSITION_DECISION.md`
**Attribution:** Documentation metadata only; not runtime, merge, or execution
authority.

## Purpose

This directory defines the active Phase-19 Platform Runtime MVP RFC set.

The set narrows the future runtime implementation question before any runtime
source code exists. It defines lifecycle, static input bundle, validation
integration, workspace admission record, runtime receipt, evidence plan,
evidence matrix, and non-goal boundaries.

`CURRENT_PHASE=19` activates this planning/admission/receipt boundary only.
It does not authorize package installation, module loading, workspace
creation, real filesystem mounts, plugin loading, capability issuance, trust
assignment, Semantic CLI authority, AI Runtime authority, new syscalls, kernel
ABI expansion, or runtime implementation.

## Current RFCs

1. `RUNTIME_LIFECYCLE_SPECIFICATION.md` - deterministic runtime MVP lifecycle
   states and transitions.
2. `RUNTIME_INPUT_BUNDLE_SPECIFICATION.md` - static test-owned input bundle
   boundary.
3. `PLATFORM_VALIDATION_INTEGRATION_SPECIFICATION.md` - integration with the
   Phase-18 Platform ABI Validation Gate.
4. `WORKSPACE_ADMISSION_RUNTIME_SPECIFICATION.md` - workspace admission record
   boundary without workspace creation or mount authority.
5. `RUNTIME_RECEIPT_SPECIFICATION.md` - deterministic runtime receipt schema
   and digest binding.
6. `RUNTIME_EVIDENCE_PLAN.md` - required local/remote evidence surfaces for a
   later implementation.
7. `RUNTIME_EVIDENCE_MATRIX.md` - artifact-to-evidence mapping for positive,
   negative, deterministic, remote, and production-default proof obligations.
8. `RUNTIME_NON_GOALS_AND_DENIALS.md` - explicit denial list for installer,
   loader, issuer, trust, Semantic CLI, AI Runtime, registry, and agent drift.

## Current Review Record

`CROSS_CONSISTENCY_REVIEW.md` records the accepted cross-document review for
the Phase-19 Runtime RFC set. The review PASS is documentation evidence only.
It does not authorize runtime implementation.

`../../../PHASE19_POINTER_TRANSITION_CANDIDATE.md` records the later pointer
transition preconditions. It is a candidate record only; it does not authorize
runtime implementation.

`../../../PHASE19_ACTIVATION_PRECONDITIONS_REVIEW.md` records the review of
documented activation preconditions. It is not an activation decision; it does
not authorize runtime implementation.

`../../../PHASE19_POINTER_TRANSITION_DECISION.md` records the phase pointer
transition to `CURRENT_PHASE=19`. That transition activates only this
documented Runtime MVP planning, validation-integration, admission-record, and
receipt-boundary phase. It does not authorize runtime implementation.

`../../../PHASE19_RUNTIME_IMPLEMENTATION_DECISION_CANDIDATE.md` records the
later implementation decision candidate boundary. It does not authorize
runtime source code or implementation.

`../../../PHASE19_RUNTIME_IMPLEMENTATION_DECISION_PACKAGE_CANDIDATE.md`
records the later implementation decision package candidate boundary. It is
not an implementation decision and does not authorize runtime source code.

`../../../PHASE19_RUNTIME_IMPLEMENTATION_DECISION_PACKAGE_DRAFT.md` records
the later implementation decision package draft boundary. It is not the
implementation decision package and does not authorize runtime source code.

`../../../PHASE19_RUNTIME_IMPLEMENTATION_DECISION_PACKAGE.md` records the
exact-SHA implementation decision package boundary. It is not an
implementation PR, evidence package, acceptance review, or runtime source code
authority.

`../../../PHASE19_RUNTIME_IMPLEMENTATION_EVIDENCE_PACKAGE.md` records evidence
for the draft PR #181 bounded admission/receipt implementation subject. It is
not acceptance review, merge authority, general runtime authority, or runtime
activation.

`../../../PHASE19_RUNTIME_IMPLEMENTATION_ACCEPTANCE_REVIEW.md` records the
first acceptance review for that evidence package. It did not grant
acceptance for the original subject and required additional transcript
evidence.

`../../../PHASE19_RUNTIME_IMPLEMENTATION_ADDITIONAL_TRANSCRIPT_EVIDENCE.md`
records additional denial transcript evidence requested by the first
acceptance review. It is evidence only; it is not an acceptance review update,
acceptance, merge authority, general runtime authority, or runtime activation.

`../../../PHASE19_RUNTIME_IMPLEMENTATION_ACCEPTANCE_REVIEW_UPDATE.md` records
the acceptance review update for that additional evidence. It treats the
additional transcript evidence as sufficient review input, but it does not
grant acceptance because validation stale digest and validation unknown stage
remain collapsed into `subject_mismatch`. It requires a new implementation
subject before PR #181 can leave draft status.

`../../../PHASE19_RUNTIME_IMPLEMENTATION_REASON_CLASS_UPDATE.md` records the
bounded implementation subject update that separates validation stale digest
from unknown validation stage as distinct denial reason classes. It is not an
evidence package, acceptance review, acceptance, merge authority, or general
runtime authority. Evidence re-binding is recorded separately and acceptance
review is recorded separately for the updated subject.

`../../../PHASE19_RUNTIME_IMPLEMENTATION_EVIDENCE_PACKAGE_REBIND.md` records
evidence re-binding for updated implementation subject `64fa4762`. It is not
an acceptance review, acceptance, merge authority, runtime activation, or
general runtime authority. Final bounded acceptance review is recorded
separately.

`../../../PHASE19_RUNTIME_IMPLEMENTATION_ACCEPTANCE_REVIEW_FINAL.md` records
final bounded implementation acceptance for updated subject `64fa4762`. It is
not merge authority, runtime activation, parser authority, loader authority,
installer authority, workspace runtime authority, issuer authority, Semantic
CLI authority, AI Runtime authority, syscall authority, kernel ABI expansion,
or Phase-19 closure authority.

`RUNTIME_EVIDENCE_MATRIX.md` maps accepted evidence obligations to future
proof rows. It is not a CI gate, evidence PASS, implementation decision, or
runtime authority.

## Core Rule

```text
Runtime RFC set != runtime implementation
CURRENT_PHASE=19 != runtime implementation
evidence matrix != evidence PASS
decision package candidate != implementation decision
decision package draft != implementation decision package
implementation decision package != implementation PR
implementation PR != evidence package
evidence package != acceptance review
acceptance review != acceptance
additional transcript evidence != acceptance
reason-class update != evidence package
evidence re-bind != acceptance review
final acceptance review != merge authority
bounded acceptance != runtime activation
```

The existence of these RFCs and the `CURRENT_PHASE=19` pointer means only that
the Phase-19 Runtime MVP boundary is active as planning authority. The final
acceptance review grants only exact-SHA bounded admission/receipt
implementation acceptance; merge, runtime activation, general runtime source
code authority, loader, installer, workspace runtime, issuer, Semantic CLI
authority and AI Runtime authority remain separate and closed.

## MVP Boundary

The only future MVP shape allowed by this set is:

```text
static input bundle
  -> Phase-18 Platform ABI validation integration
  -> workspace admission record
  -> deterministic runtime receipt
```

This flow must not install, load, mount, execute, issue, trust, publish, or
schedule anything.

## Non-Authority Rule

No file in this directory may grant:

1. Runtime source code authority.
2. Package installation or execution.
3. Module loading.
4. Plugin loading or instantiation.
5. Workspace creation or real mounts.
6. Capability token minting or issuance.
7. Trust assignment.
8. Registry publication.
9. Semantic CLI execution authority.
10. AI Runtime authority.
11. Agent authority.
12. New syscalls.
13. Kernel ABI expansion.
14. Ring0 policy.

Unknown authority readings fail closed.
