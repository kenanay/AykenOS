# Phase-19 Runtime Implementation Decision Package Candidate

This document is subordinate to PHASE 0 - FOUNDATIONAL OATH,
`ARCHITECTURE_FREEZE.md`, `PHASE18_TRANSITION_DECISION.md`,
`PHASE18_ACTIVATION_DECISION.md`, the Phase-18 Platform Constitution
reference set, `AUTHORITY_DRIFT_GUARD.md`, `TERMINOLOGY_AUDIT.md`,
`PHASE19_RUNTIME_DECISION.md`, the Phase-19 Runtime RFC set,
`docs/specs/phase19-platform-runtime/CROSS_CONSISTENCY_REVIEW.md`,
`PHASE19_POINTER_TRANSITION_CANDIDATE.md`,
`PHASE19_ACTIVATION_PRECONDITIONS_REVIEW.md`,
`PHASE19_POINTER_TRANSITION_DECISION.md`,
`PHASE19_RUNTIME_IMPLEMENTATION_DECISION_CANDIDATE.md`, and
`docs/specs/phase19-platform-runtime/RUNTIME_EVIDENCE_MATRIX.md`. In case of
conflict, those documents prevail.

**Status:** CANDIDATE / DECISION PACKAGE NOT ACCEPTED / RUNTIME SOURCE CODE NOT AUTHORIZED
**Candidate date:** 2026-06-12
**Candidate id:** `ayken.phase19.runtime_implementation_decision_package_candidate.v1`
**Authority boundary:** Candidate documentation only; not an implementation
decision, not runtime source code, not a manifest parser, not a general
parser, not a userspace harness implementation, not a CI workflow, not a test
script, not a performance threshold, not package installation, not package
execution, not module loading, not workspace runtime, not workspace creation,
not real mount authority, not plugin host, not plugin loading, not capability
token minting, not capability issuance, not trust assignment, not registry
publication, not Semantic CLI authority, not AI Runtime authority, not agent
authority, not a syscall, not kernel ABI expansion, not Ring0 policy, not
merge authority, and not closure authority.

## Purpose

This package candidate defines what a later exact-SHA Phase-19 Runtime MVP
implementation decision package must contain before runtime source code can be
reviewed.

It does not accept the implementation decision.

It does not authorize runtime source code.

It exists to keep the future decision package narrow: minimum inert
admission/receipt behavior, complete evidence mapping, exact-SHA acceptance
preconditions, and fail-closed denial rules only.

## Core Rule

```text
decision package candidate != implementation decision package
decision package candidate != implementation decision
implementation decision != runtime implementation
bounded implementation != loader / installer / issuer / executor authority
```

The safe default remains no runtime source code and no runtime behavior.

## Candidate Package Scope

A later implementation decision package may be considered only if it answers
these four questions:

1. What is the exact minimum behavior?
2. Which evidence path closes each required matrix row?
3. What exact-SHA remote checks must pass before the decision can be accepted?
4. Which conditions fail closed before implementation authority can exist?

The package candidate must not include implementation design beyond those
questions.

## Minimum Behavior Boundary

The only behavior a later implementation decision package may discuss is:

```text
static input bundle
  -> Phase-18 Platform ABI validation integration record
  -> workspace admission record
  -> deterministic runtime receipt
```

The future behavior must remain inert. It must not install, load, mount,
execute, issue, trust, publish, schedule, bind, grant, tokenize, instantiate,
or create anything.

## Matrix Row Mapping Requirement

A later implementation decision package must map every accepted
`RUNTIME_EVIDENCE_MATRIX.md` row to a future evidence path.

This candidate records the expected mapping shape only:

| Matrix area | Future evidence path class | Acceptance meaning | Forbidden reading |
|---|---|---|---|
| Positive input binding | Positive transcript and input digest evidence | Static test-owned bundle was accepted as inert input | General parser, installer request, loader request |
| Positive validation integration | Validation integration record and subject-binding evidence | Required Phase-18 declarative metadata was checked for admission/receipt evidence | Authorization, trust, capability, workspace grant |
| Positive admission record | Workspace admission record evidence | Inert record was emitted after validation integration | Workspace creation, real mount, namespace, handle |
| Positive runtime receipt | Receipt digest and transcript binding evidence | Receipt binds lifecycle, input, validation, and admission digests | Token, capability, workspace handle, execution right |
| Unknown or duplicate input denial | Negative denial transcript | Rejected before input binding success | Lenient parse or partial accept |
| Missing or stale validation denial | Negative denial transcript | Rejected before validation integration success | Validation fallback or assumed compatibility |
| Workspace mount or handle denial | Negative denial transcript | Rejected before admission record success | Workspace runtime, real mount, access grant |
| Receipt-as-token denial | Negative denial transcript | Rejected before receipt success | Bearer token or capability token |
| Trust-as-capability denial | Negative denial transcript | Rejected before receipt success | Trust issuer or capability grant |
| Plugin-as-loading denial | Negative denial transcript | Rejected before receipt success | Plugin loading, binding, instantiation |
| Semantic CLI output-as-authority denial | Negative denial transcript | Rejected before receipt success | Semantic CLI execution authority |
| AI output-as-authority denial | Negative denial transcript | Rejected before receipt success | AI Runtime authority |
| Kernel ABI expansion denial | Negative denial transcript and ABI freeze proof | Rejected before input binding success | New syscall or ABI widening |
| Deterministic digest rows | Repeated digest evidence | Same input produces same transcript, record, receipt, and denial digests | Wall-clock, host timing, debug log ordering |
| Remote proof rows | Exact-SHA remote run evidence | Required checks passed on the decision subject SHA | Historical PASS inheritance |
| Production-default row | Default profile evidence | Validation-only behavior remains default-off unless separately accepted | Production runtime authority |

Rows may be refined by a later accepted matrix update, but missing,
ambiguous, stale, or partially mapped rows fail closed.

## Exact-SHA Acceptance Preconditions

A later implementation decision package is not acceptable unless it requires
all of the following for the exact decision subject SHA:

1. Strict `ci-freeze` PASS.
2. Dev Loop PASS.
3. Runtime-specific admission/receipt gates PASS if those gates are proposed.
4. Positive admission/receipt evidence PASS.
5. Required negative denial evidence PASS.
6. Deterministic digest repeat evidence PASS.
7. Production-default evidence PASS.
8. Kernel ABI freeze proof PASS.
9. No CI workflow authority widening unless separately reviewed.
10. No performance threshold or baseline change unless separately reviewed.

Historical PASS results may be cited as context only. They cannot be inherited
as authority for the later decision subject SHA.

## Fail-Closed Conditions

A later implementation decision package must fail closed if any of these
conditions exist:

1. A matrix row has no mapped evidence path.
2. Positive receipt evidence is missing.
3. A negative denial case reaches receipt success.
4. Lifecycle, admission, receipt, or denial digest repeat is nondeterministic.
5. A receipt is treated as a token, handle, right, or capability.
6. Trust classification is treated as capability grant.
7. Plugin compatibility is treated as loading, binding, or instantiation.
8. Semantic CLI output is treated as runtime authority.
9. AI output is treated as runtime authority.
10. Workspace admission is treated as workspace creation or real mount.
11. Static input bundle acceptance is treated as install, load, or execute
    permission.
12. Kernel ABI `1000-1011` / 12 syscall / `0x00010001` changes.
13. Ring0 policy is introduced.
14. Evidence becomes runtime control input.
15. The package includes runtime source code, parser design, CI workflow,
    test script, or performance threshold details.

Unknown authority readings fail closed.

## Prohibited Package Contents

This package candidate must not include, and a later decision package must not
smuggle in:

1. Runtime source code.
2. Manifest parser design.
3. General parser design.
4. Harness implementation details.
5. CI workflow files.
6. Test scripts.
7. Performance threshold values.
8. `CURRENT_PHASE` changes.
9. Closure language.
10. Runtime activation language beyond the bounded decision subject.
11. Loader, installer, package executor, or workspace runtime.
12. Plugin host or plugin loading.
13. Capability issuer or token minting.
14. Trust issuer or trust assignment.
15. Semantic CLI authority.
16. AI Runtime authority.
17. Agent authority.
18. New syscalls or kernel ABI expansion.

## Relationship To Later Decision And Code

This file is one step before a possible implementation decision package.

The order remains:

```text
decision package candidate
  -> implementation decision package draft
  -> separate exact-SHA implementation decision package
  -> separate implementation PR
  -> evidence package
  -> remote PASS
  -> acceptance review
```

No step in that chain can inherit implementation authority from this
candidate.

`PHASE19_RUNTIME_IMPLEMENTATION_DECISION_PACKAGE_DRAFT.md` may narrow the
draft shape of that later package, but it is not the implementation decision
package and does not authorize runtime source code.

## Candidate Conclusion

This file records the candidate shape for a later Phase-19 Runtime MVP
implementation decision package.

The candidate narrows the decision package to minimum inert behavior,
evidence-row mapping, exact-SHA acceptance preconditions, and fail-closed
conditions.

Runtime implementation remains unauthorized.
