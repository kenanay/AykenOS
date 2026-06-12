# Phase-19 Runtime Implementation Decision Package Draft

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
`docs/specs/phase19-platform-runtime/RUNTIME_EVIDENCE_MATRIX.md`, and
`PHASE19_RUNTIME_IMPLEMENTATION_DECISION_PACKAGE_CANDIDATE.md`. In case of
conflict, those documents prevail.

**Status:** DRAFT / IMPLEMENTATION DECISION PACKAGE NOT ACCEPTED / RUNTIME SOURCE CODE NOT AUTHORIZED
**Draft date:** 2026-06-13
**Draft id:** `ayken.phase19.runtime_implementation_decision_package_draft.v1`
**Authority boundary:** Draft documentation only; not an implementation
decision package, not an implementation decision, not runtime source code, not
a manifest parser, not a general parser, not a harness implementation, not a
CI workflow, not a test script, not a performance threshold, not package
installation, not package execution, not module loading, not workspace
runtime, not workspace creation, not real mount authority, not plugin host, not
plugin loading, not capability token minting, not capability issuance, not
trust assignment, not registry publication, not Semantic CLI authority, not AI
Runtime authority, not agent authority, not a syscall, not kernel ABI
expansion, not Ring0 policy, not merge authority, and not closure authority.

## Purpose

This draft narrows what a later exact-SHA Phase-19 Runtime MVP implementation
decision package would need to contain.

This draft is not the implementation decision package and does not authorize
runtime source code.

It exists to keep the next package small, evidence-bound, and fail-closed
before any runtime implementation PR can be reviewed.

## Core Rule

```text
draft != implementation decision package
implementation decision package != implementation decision
implementation decision != runtime implementation
runtime implementation != authority source
```

The safe default remains no runtime source code and no runtime behavior.

## Draft Scope

A later implementation decision package may be drafted only around these four
questions:

1. What exact minimum behavior is being considered?
2. How will the accepted evidence matrix rows be closed?
3. Which exact-SHA preconditions must pass?
4. Which cases fail closed before authority can exist?

This draft must not include parser design, harness design, CI workflow files,
test scripts, performance threshold values, or runtime implementation details.

## Minimum Accepted Behavior

The only behavior that may be discussed by a later implementation decision
package is:

```text
static input bundle
  -> Phase-18 validation integration record
  -> workspace admission record
  -> deterministic runtime receipt
```

This chain is an inert admission/receipt boundary only.

It must not install, load, mount, execute, issue, trust, publish, schedule,
bind, grant, tokenize, instantiate, or create anything.

Anything outside this chain is a non-goal for the first Phase-19 Runtime MVP
implementation decision package.

## Evidence Binding

A later implementation decision package must bind every accepted
`RUNTIME_EVIDENCE_MATRIX.md` row to a future evidence class without defining
workflow or script implementation details.

The required binding classes are:

| Matrix row class | Required future evidence class | Forbidden reading |
|---|---|---|
| Positive input bundle | Positive transcript and input digest | Parser, installer request, loader request, execution request |
| Positive validation integration | Validation integration record and subject digest | Authorization, trust grant, capability grant, workspace grant |
| Positive workspace admission | Workspace admission record transcript and digest | Workspace creation, real mount, namespace, handle, access grant |
| Positive runtime receipt | Runtime receipt transcript and digest binding | Token, capability, handle, execution right |
| Negative denial rows | Negative transcript ending before receipt success | Partial accept, fallback validation, authority-bearing artifact |
| Deterministic repeat rows | Repeated digest proof | Wall-clock, host timing, debug output ordering |
| Remote exact-SHA rows | Remote proof for the decision subject SHA | Historical PASS inheritance |
| Production-default rows | Default profile proof | Production runtime authority |
| ABI freeze rows | ABI preservation proof | New syscall, ABI widening, Ring0 policy |

The implementation decision package may name concrete checks only when those
checks are reviewed as part of that later package. This draft does not define
or authorize any CI workflow, test script, runtime gate, or performance
threshold.

## Exact-SHA Preconditions

A later implementation decision package must require exact-SHA proof for the
decision subject.

At minimum, it must require:

1. Strict `ci-freeze` PASS.
2. Dev Loop PASS.
3. Runtime-specific gates PASS if they are separately proposed in that later
   package.
4. Positive evidence PASS.
5. Negative evidence PASS.
6. Deterministic repeat evidence PASS.
7. Production-default evidence PASS.
8. ABI freeze PASS.
9. No runtime source code bundled into the decision package.
10. No parser, loader, installer, issuer, workspace runtime, Semantic CLI, AI
    Runtime, registry, or agent authority bundled into the decision package.

Historical PASS results are context only. Authority cannot be inherited.

If the subject SHA changes, all exact-SHA evidence for that decision subject
must be regenerated.

## Fail-Closed Denials

A later implementation decision package must fail closed on:

1. Missing evidence.
2. Partial evidence.
3. Unmapped matrix rows.
4. Nondeterministic lifecycle, admission, receipt, or denial digests.
5. Receipt-as-token.
6. Trust-as-capability.
7. Plugin-as-loading.
8. Semantic CLI output-as-authority.
9. AI output-as-authority.
10. Workspace-as-real-mount.
11. Input bundle as install, load, or execute request.
12. Kernel ABI drift.
13. Ring0 policy.
14. Evidence used as runtime control input.
15. Runtime source code included in the decision package.
16. Parser, harness, workflow, script, or threshold detail included in this
    draft layer.

Unknown authority readings fail closed.

## Prohibited Draft Contents

This draft must not be read to include or authorize:

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
  -> separate exact-SHA implementation decision package
  -> separate implementation PR
  -> evidence package
  -> remote PASS
  -> acceptance review
```

No step in that chain can inherit implementation authority from this draft.

## Draft Conclusion

This file records a narrow draft shape for a later Phase-19 Runtime MVP
implementation decision package.

The draft is limited to minimum inert behavior, evidence binding, exact-SHA
preconditions, and fail-closed denials.

Runtime implementation remains unauthorized.
