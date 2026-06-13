# Phase-19 Runtime Implementation Decision Package

This document is subordinate to PHASE 0 - FOUNDATIONAL OATH,
`ARCHITECTURE_FREEZE.md`, `PHASE18_TRANSITION_DECISION.md`,
`PHASE18_ACTIVATION_DECISION.md`, the Phase-18 Platform Constitution
reference set, `AUTHORITY_DRIFT_GUARD.md`, `TERMINOLOGY_AUDIT.md`,
`PHASE19_RUNTIME_DECISION.md`, the Phase-19 Runtime RFC set,
`docs/specs/phase19-platform-runtime/CROSS_CONSISTENCY_REVIEW.md`,
`PHASE19_POINTER_TRANSITION_CANDIDATE.md`,
`PHASE19_ACTIVATION_PRECONDITIONS_REVIEW.md`,
`PHASE19_POINTER_TRANSITION_DECISION.md`,
`PHASE19_RUNTIME_IMPLEMENTATION_DECISION_CANDIDATE.md`,
`docs/specs/phase19-platform-runtime/RUNTIME_EVIDENCE_MATRIX.md`,
`PHASE19_RUNTIME_IMPLEMENTATION_DECISION_PACKAGE_CANDIDATE.md`, and
`PHASE19_RUNTIME_IMPLEMENTATION_DECISION_PACKAGE_DRAFT.md`. In case of
conflict, those documents prevail unless this package is the narrower
authority for the later Phase-19 implementation decision package boundary.

**Status:** IMPLEMENTATION DECISION PACKAGE / IMPLEMENTATION PR NOT INCLUDED / RUNTIME SOURCE CODE NOT AUTHORIZED
**Decision package date:** 2026-06-13
**Decision id:** `ayken.phase19.runtime_implementation_decision_package.v1`
**Authority boundary:** Decision package only; not an implementation PR, not
an evidence package, not an acceptance review, not runtime source code, not a
manifest parser, not a general parser, not harness implementation, not a CI
workflow, not a test script, not a performance threshold, not package
installation, not package execution, not module loading, not workspace
runtime, not workspace creation, not real mount authority, not plugin host, not
plugin loading, not capability token minting, not capability issuance, not
trust assignment, not registry publication, not Semantic CLI authority, not AI
Runtime authority, not agent authority, not a syscall, not kernel ABI
expansion, not Ring0 policy, not merge authority, and not closure authority.

## Decision

The Phase-19 Runtime MVP implementation decision package boundary is accepted
only for the minimum inert admission/receipt behavior and evidence obligations
defined in this document.

This package does not include runtime source code.

This package does not authorize a runtime implementation.

This package does not accept an implementation PR, evidence package, remote
PASS result, or acceptance review.

## Core Rule

```text
implementation decision package != implementation PR
implementation PR != evidence package
evidence package != acceptance review
runtime source code remains unauthorized
```

The safe default remains no runtime behavior unless a later separate
implementation PR and evidence package pass their own exact-SHA review.

## Minimum Allowed Behavior

The only implementation behavior that a later separate PR may propose under
this package is:

```text
static input bundle
  -> Phase-18 validation integration record
  -> workspace admission record
  -> deterministic runtime receipt
```

This chain is an inert admission/receipt boundary only.

It must not install, load, mount, execute, issue, trust, publish, schedule,
bind, grant, tokenize, instantiate, or create anything.

Any behavior outside this chain is outside this package and must fail closed
or move to a later reviewed phase.

## Evidence Binding

A later separate evidence package must close every accepted
`RUNTIME_EVIDENCE_MATRIX.md` row that applies to the proposed implementation.

The required binding classes are:

| Matrix row class | Required evidence class | Forbidden reading |
|---|---|---|
| Positive input bundle | Positive transcript and input digest | Parser, installer request, loader request, execution request |
| Positive validation integration | Validation integration record and subject digest | Authorization, trust grant, capability grant, workspace grant |
| Positive workspace admission | Workspace admission record transcript and digest | Workspace creation, real mount, namespace, handle, access grant |
| Positive runtime receipt | Runtime receipt transcript and digest binding | Token, capability, handle, execution right |
| Negative denial rows | Negative transcript ending before receipt success | Partial accept, fallback validation, authority-bearing artifact |
| Deterministic repeat rows | Repeated transcript, record, and receipt digest proof | Wall-clock, host timing, debug output ordering |
| Remote exact-SHA rows | Remote proof for the implementation decision subject SHA | Historical PASS inheritance |
| Production-default rows | Default profile proof | Production runtime authority |
| ABI freeze rows | ABI preservation proof | New syscall, ABI widening, Ring0 policy |

This package does not define CI workflow files, test scripts, runtime gate
implementations, performance thresholds, or benchmark values. Those details
must be reviewed separately if a later implementation PR requires them.

## Exact-SHA Preconditions

A later implementation PR and evidence package must be evaluated only for the
exact subject SHA that contains the proposed implementation.

At minimum, the later implementation decision chain must require:

1. Strict `ci-freeze` PASS.
2. Dev Loop PASS.
3. Runtime-specific gates PASS if separately proposed and reviewed.
4. Positive evidence PASS.
5. Negative evidence PASS.
6. Deterministic repeat evidence PASS.
7. Production-default evidence PASS.
8. ABI freeze PASS.
9. No unreviewed runtime source outside the bounded admission/receipt harness.
10. No parser, loader, installer, issuer, workspace runtime, Semantic CLI, AI
    Runtime, registry, or agent authority.

Historical PASS results are context only. Authority cannot be inherited.

If the subject SHA changes, all exact-SHA evidence for that subject must be
regenerated.

## Separation Rules

The later chain must remain separated:

1. This implementation decision package is documentation authority only.
2. A separate implementation PR is required before any runtime source code can
   be reviewed.
3. A separate evidence package is required before any implementation can be
   accepted.
4. Remote PASS results must be tied to the exact implementation subject SHA.
5. Acceptance review is separate from the implementation PR and the evidence
   package.

No step can inherit authority from a prior step unless the exact-SHA evidence
for that step is present and reviewed.

## Evidence Rule

This package becomes an accepted documentation record only for the exact
subject SHA that contains it after required remote checks pass.

Required evidence for the package subject:

1. Strict `ci-freeze` PASS.
2. Dev Loop PASS.
3. Dev Loop Validation PASS when required by the branch protection profile.
4. No runtime source code changes.
5. No kernel code changes.
6. No syscall declaration or ABI layout changes.
7. No performance baseline changes.
8. No CI workflow authority widening.
9. Status, roadmap, README, documentation index, and Phase-19 RFC set
   synchronization.

Historical PASS results may be cited as context only. They cannot be
inherited as authority for this package subject SHA.

If the subject SHA changes after evidence is recorded, the evidence must be
regenerated for the new subject SHA.

## Fail-Closed Denials

The later implementation decision chain must fail closed on:

1. Missing evidence.
2. Partial evidence.
3. Unmapped matrix rows.
4. Nondeterministic lifecycle, admission, receipt, or denial digests.
5. Receipt-as-token.
6. Trust-as-capability.
7. Plugin-as-loading.
8. Workspace-as-real-mount.
9. Semantic CLI output-as-authority.
10. AI output-as-authority.
11. Evidence-as-control-input.
12. Input bundle as install, load, or execute request.
13. Kernel ABI drift.
14. Ring0 policy.
15. Runtime source code bundled into this package.
16. Parser design, harness implementation design, workflow design, test
    scripts, thresholds, or benchmark values bundled into this package.

Unknown authority readings fail closed.

## Prohibited Package Contents

This package must not be read to include or authorize:

1. Runtime source code.
2. Manifest parser design.
3. General parser design.
4. Harness implementation details.
5. CI workflow files.
6. Test scripts.
7. Performance threshold values.
8. `CURRENT_PHASE` changes.
9. Closure language.
10. Runtime activation beyond the already active planning/admission/receipt
    boundary.
11. Loader, installer, package executor, or workspace runtime.
12. Plugin host or plugin loading.
13. Capability issuer or token minting.
14. Trust issuer or trust assignment.
15. Semantic CLI authority.
16. AI Runtime authority.
17. Agent authority.
18. New syscalls or kernel ABI expansion.

## Relationship To Later Steps

The order remains:

```text
implementation decision candidate
  -> implementation decision package candidate
  -> implementation decision package draft
  -> exact-SHA implementation decision package
  -> separate implementation PR
  -> separate evidence package
  -> remote PASS
  -> acceptance review
```

This package occupies the exact-SHA implementation decision package step. It
does not collapse the later implementation PR, evidence package, remote PASS,
or acceptance review steps.

## Decision Package Conclusion

This file accepts the narrow Phase-19 Runtime MVP implementation decision
package boundary.

The accepted boundary is limited to minimum inert admission/receipt behavior,
evidence binding, exact-SHA preconditions, separation rules, and fail-closed
denials.

Runtime implementation remains unauthorized.
