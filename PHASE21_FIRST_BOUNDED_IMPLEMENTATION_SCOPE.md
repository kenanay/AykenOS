# Phase-21 First Bounded Implementation Scope

This document is subordinate to PHASE 0 - FOUNDATIONAL OATH,
`ARCHITECTURE_FREEZE.md`, the Phase-18 Platform Constitution reference set,
`docs/specs/phase18-platform-constitution/AUTHORITY_DRIFT_GUARD.md`,
`docs/specs/phase18-platform-constitution/TERMINOLOGY_AUDIT.md`,
`PHASE19_RUNTIME_DECISION.md`, the Phase-19 Runtime RFC set,
`docs/specs/phase19-platform-runtime/RUNTIME_EVIDENCE_MATRIX.md`,
`PHASE19_CLOSURE_DECISION.md`,
`PHASE20_POINTER_TRANSITION_CANDIDATE.md`,
`PHASE20_POINTER_TRANSITION_DECISION.md`,
`PHASE20_GOVERNANCE_OVERVIEW.md`,
`PHASE20_CAPABILITY_MODEL.md`,
`PHASE20_CAPABILITY_IDENTITY.md`,
`PHASE20_CAPABILITY_MANIFEST_SCHEMA.md`,
`PHASE20_CAPABILITY_LIFECYCLE.md`,
`PHASE20_REGISTRY_MODEL.md`,
`PHASE20_REGISTRY_GOVERNANCE.md`,
`PHASE20_TRUST_MODEL.md`,
`PHASE20_DISTRIBUTION_POLICY.md`,
`PHASE20_CAPABILITY_EVIDENCE_MODEL.md`,
`PHASE20_CAPABILITY_ACCEPTANCE_WORKFLOW.md`,
`PHASE20_IMPLEMENTATION_DECISION.md`,
`PHASE20_IMPLEMENTATION_SLICE.md`,
`PHASE20_IMPLEMENTATION_REVIEW.md`,
`PHASE20_IMPLEMENTATION_ACCEPTANCE_DECISION.md`,
`PHASE20_RUNTIME_DECISION.md`,
`PHASE20_RUNTIME_DECISION_REVIEW.md`,
`PHASE20_RUNTIME_ACCEPTANCE_DECISION.md`,
`PHASE20_RUNTIME_ACTIVATION_DECISION.md`,
`PHASE20_RUNTIME_IMPLEMENTATION_DECISION.md`,
`PHASE20_RUNTIME_IMPLEMENTATION_REVIEW.md`,
`PHASE20_RUNTIME_IMPLEMENTATION_ACCEPTANCE_DECISION.md`,
`PHASE20_CLOSURE_DECISION.md`,
`PHASE21_POINTER_TRANSITION_CANDIDATE.md`,
`PHASE21_POINTER_TRANSITION_DECISION.md`, and
`PHASE21_GOVERNANCE_OVERVIEW.md`. In case of conflict, those documents
prevail unless this scope RFC is the narrower Phase-21 first bounded
implementation scope record for the exact governance subject identified
below.

**Status:** PHASE-21 FIRST BOUNDED IMPLEMENTATION SCOPE RFC / SCOPE
BOUNDARY ONLY / USERSPACE-ONLY / NON-EXECUTING / VALIDATOR, RECEIPT,
FIXTURE, AND CI GATE ORIENTED / NO IMPLEMENTATION PACKAGE / NO RUNTIME
IMPLEMENTATION PROCEDURE / NO SOURCE MODIFICATION BY THIS RFC / NO CODE
IMPLEMENTATION BY THIS RFC / NO CODE EXECUTION / NO PROCESS START / NO
RUNTIME STATE CREATION / NO PACKAGE AUTHORITY / NO PACKAGE INSTALLATION /
NO PACKAGE LOADING / NO PACKAGE EXECUTION / NO DEPLOYMENT / NO CAPABILITY
ISSUANCE / NO TRUST ASSIGNMENT / NO REGISTRY PUBLICATION / NO DISTRIBUTION
AUTHORITY / NO SOURCE MERGE AUTHORITY / NO SOURCE ACCEPTANCE
**Scope date:** 2026-07-01
**Scope id:** `ayken.phase21.first_bounded_implementation_scope.v1`
**Scope base main SHA:** `ae3f9f05cad36451e49a81e4ccfe593d7a9f9ec6`
**Reviewed Phase-21 governance overview SHA:**
`ae3f9f05cad36451e49a81e4ccfe593d7a9f9ec6`
**Phase-21 pointer transition decision exact-main SHA:**
`a30c34bd141b2fc3c655bdc0a483f418674365f1`
**Current phase pointer:** `CURRENT_PHASE=21`
**Scope theme:** First Bounded Implementation
**Authority boundary:** Scope boundary only; not an implementation package,
not runtime implementation procedure, not source modification, not code
implementation, not code execution, not process start, not runtime state
creation, not general runtime authority, not unbounded execution authority,
not package authority, not package installation, not package loading, not
package execution, not deployment, not source acceptance, not source merge
authority, not source repository authority, not module loading, not workspace
runtime, not plugin loading, not capability token minting, not capability
issuance, not trust assignment, not trust issuer authority, not registry
authority, not registry publication, not publication authority, not
distribution authority, not distribution execution, not Semantic CLI
authority, not AI Runtime authority, not agent authority, not syscall
expansion, not kernel ABI expansion, not workflow-threshold, baseline,
dependency, or Ring0 authority.

## Purpose

This document defines the Phase-21 first bounded implementation scope
boundary after the accepted Phase-21 Governance Overview.

It defines only the allowed boundary for a possible later first bounded
implementation package.

The scope is limited to:

1. Userspace-only.
2. Non-executing.
3. Validator, receipt, fixture, and CI gate oriented.
4. Fail-closed.
5. Exact-SHA evidence oriented.

Phase-21 may later host a first bounded implementation package, but only
inside a userspace-only, non-executing validation and receipt evidence
boundary.

This scope RFC does not define runtime implementation procedure.

This scope RFC does not modify source.

This scope RFC does not implement code.

This scope RFC does not execute code.

This scope RFC does not start a process.

This scope RFC does not create runtime state.

This scope RFC does not install, load, or execute packages.

This scope RFC does not issue capabilities, publish registry entries,
assign trust, accept source, or merge source.

## Exact Subject

This first bounded implementation scope RFC is bound to the Phase-21
Governance Overview published at exact main SHA:

```text
ae3f9f05cad36451e49a81e4ccfe593d7a9f9ec6
```

The exact subject records:

1. Phase-21 open as pointer-governance only.
2. First Bounded Implementation as a governance theme only.
3. No Phase-21 implementation scope before a separate reviewed scope
   decision.
4. No runtime implementation procedure.
5. No source modification, code implementation, code execution, process
   start, or runtime state creation.
6. No package loading, package execution, capability issuance, registry
   publication, trust assignment, or source merge authority.

This scope RFC consumes that exact subject as governance input only. It does
not replace, broaden, reinterpret, or supersede the Phase-21 Governance
Overview.

Missing, ambiguous, stale, inherited, aliased, superseded, or differently
scoped subject readings fail closed.

## Scope Decision

The Phase-21 first bounded implementation scope decision is:

```text
userspace-only
non-executing
validator / receipt / fixture / CI gate oriented
fail-closed
exact-SHA evidence oriented
```

This is a scope boundary only.

It does not create an implementation package.

It does not authorize runtime implementation procedure.

It does not modify source.

It does not implement code.

It does not execute code.

It does not start a process.

It does not create runtime state.

It does not install, load, or execute packages.

It does not issue capabilities, publish registry entries, assign trust,
authorize distribution, deploy artifacts, accept source, or merge source.

Unknown authority readings fail closed.

## Core Rule

```text
scope decision != implementation package
scope decision != runtime implementation procedure
scope decision != source modification
scope decision != code implementation
scope decision != code execution
scope decision != process start
scope decision != runtime state creation
scope decision != package loading
scope decision != package execution
scope decision != capability issuance
scope decision != registry publication
scope decision != trust assignment
scope decision != source merge
userspace-only != runtime authority
non-executing != execution authority
validator oriented != validator implementation
receipt oriented != receipt implementation
fixture oriented != fixture files created
CI gate oriented != CI workflow change
exact-SHA evidence oriented != evidence accepted
```

The safe default remains no source modification, no runtime implementation
procedure, no code execution, no process start, no runtime state, and no
package, capability, registry, trust, distribution, deployment, or source
merge authority unless a later reviewed Phase-21 decision grants a specific
bounded authority with its own exact-SHA evidence.

## First Bounded Implementation Boundary

First Bounded Implementation Scope is the narrow governance boundary for a
possible later implementation package.

The boundary is limited to defining what a later package may be allowed to
propose.

A later first bounded implementation package may be considered only if it is:

1. Userspace-only.
2. Non-executing.
3. Static validation oriented.
4. Receipt evidence oriented.
5. Fixture category oriented.
6. CI gate expectation oriented.
7. Fail-closed.
8. Exact-SHA evidence oriented.
9. Explicitly non-runtime.
10. Explicitly non-loader.
11. Explicitly non-package-executing.
12. Explicitly non-process-starting.
13. Explicitly non-runtime-state-creating.

This boundary is not a package.

This boundary is not source modification.

This boundary is not code implementation.

This boundary is not runtime implementation procedure.

This boundary is not execution authority.

Any later package must receive its own reviewed decision, exact subject,
allowed and denied readings, validation model, evidence requirements, and
post-merge verification.

## Allowed Scope

This scope RFC may define only the allowed boundary for a possible later
first bounded implementation package.

Allowed scope is limited to:

1. Defining a later userspace-only static validator package boundary.
2. Defining receipt evidence expectations.
3. Defining fixture categories.
4. Defining CI gate expectations.
5. Defining fail-closed validation behavior.
6. Defining exact-SHA evidence requirements.
7. Defining non-execution requirements.
8. Defining denied runtime, package, process, and state readings.
9. Defining relationship boundaries to Phase-21 Governance Overview.
10. Defining relationship boundaries to Phase-20 Closure.
11. Defining relationship boundaries to Phase-19 runtime authority.
12. Defining later implementation package dependency requirements.

Allowed scope is governance text only.

Allowed scope is not source modification.

Allowed scope is not code implementation.

Allowed scope is not validator implementation.

Allowed scope is not receipt file creation.

Allowed scope is not fixture file creation.

Allowed scope is not CI workflow creation or modification.

Allowed scope is not runtime behavior.

Allowed scope is not package loading or execution.

## Explicitly Denied Scope

This scope RFC does not authorize:

1. Runtime implementation procedure.
2. Source modification by this RFC.
3. Code implementation by this RFC.
4. Code execution.
5. Process start.
6. Runtime state creation.
7. General runtime authority.
8. Unbounded execution authority.
9. Package installation, loading, execution, scheduling, or publication.
10. Module loading.
11. Workspace creation, workspace runtime, or real mounts.
12. Plugin host, plugin loading, or plugin instantiation.
13. Deployment behavior.
14. Capability token minting or capability issuance.
15. Trust assignment.
16. Trust issuer authority.
17. Registry authority.
18. Registry publication.
19. Publication authority.
20. Distribution authority.
21. Distribution execution.
22. Source acceptance or source merge authority.
23. Source repository authority.
24. Semantic CLI execution or verdict authority.
25. AI Runtime authority.
26. Agent behavior.
27. New syscalls.
28. Kernel ABI expansion.
29. Workflow-threshold, baseline, dependency, or Ring0 policy changes.
30. Observability-as-authority.

Denied scope includes source code, tests, fixtures, validators, receipts,
CI workflows, baselines, and dependency files in this RFC.

Any scope reading that crosses this denied boundary fails closed.

## Validator / Receipt / Fixture / CI Gate Orientation

Validator, receipt, fixture, and CI gate orientation is a governance
orientation only.

Validator orientation may describe later static validation expectations. It
does not create a validator, modify validator code, execute a validator, or
accept validator output.

Receipt orientation may describe later evidence receipt expectations. It
does not create receipt files, accept receipt evidence, issue proof, or grant
authority.

Fixture orientation may describe later fixture categories. It does not create
fixtures, load fixtures, execute fixtures, or treat fixture presence as
authority.

CI gate orientation may describe later CI gate expectations. It does not
create, modify, enable, bypass, or weaken CI workflows, workflow thresholds,
baselines, dependency locks, or enforcement policy.

Each later validator, receipt, fixture, or CI gate change requires a separate
reviewed decision path and exact-SHA evidence.

## Relationship To Phase-21 Governance Overview

This scope RFC consumes the Phase-21 Governance Overview as its exact
governance prerequisite.

The Phase-21 Governance Overview remains bound to:

```text
ae3f9f05cad36451e49a81e4ccfe593d7a9f9ec6
```

This scope RFC preserves:

1. Phase-21 as pointer-governance before narrower decisions.
2. First Bounded Implementation as governance theme.
3. No runtime implementation procedure.
4. No source modification by implication.
5. No code execution.
6. No process start.
7. No runtime state creation.
8. No package loading or execution.
9. No capability issuance.
10. No registry publication.
11. No trust assignment.
12. No source merge authority.

This scope RFC narrows the governance theme into an allowed scope boundary
for a possible later implementation package. It does not implement that
package.

Any reading that treats this scope as runtime implementation procedure or
implementation package authority fails closed.

## Relationship To Phase-20 Closure

Phase-20 remains closed for exact subject:

```text
ee1f1c7f43fe478c8cbdab3fbeb2844365c9c5bc
```

This scope RFC does not reopen Phase-20.

This scope RFC does not reinterpret Phase-20 closure.

This scope RFC does not extend Phase-20 closure into Phase-21
implementation package authority.

This scope RFC does not convert Phase-20 Runtime Implementation Acceptance
Decision, Phase-20 Closure Decision, or any Phase-20 governance record into
runtime implementation procedure, source modification, code implementation,
code execution, process start, runtime state creation, package loading,
capability issuance, registry publication, trust assignment, or source merge
authority.

Any reading that treats Phase-20 closure as Phase-21 implementation package
authority fails closed.

## Relationship To Phase-19 Runtime Authority

This scope RFC remains subordinate to Phase-19 runtime authority records.

Phase-19 runtime records may be read as boundary context for:

1. Runtime MVP planning boundaries.
2. Runtime evidence expectations.
3. Runtime non-goals and denials.
4. Platform runtime constitutional constraints.
5. Userspace-only runtime constraints.
6. Frozen syscall and kernel ABI boundaries.
7. Denied package, module, workspace, plugin, trust, capability, AI Runtime,
   Semantic CLI, and agent authority readings.

This scope RFC must not broaden, replace, supersede, weaken, or reinterpret
Phase-19 runtime authority records.

This scope RFC must not use Phase-21 first bounded implementation scope to
infer Phase-19 runtime authority.

This scope RFC must not use `CURRENT_PHASE=21` to infer runtime authority.

Any Phase-21 first bounded implementation scope reading that conflicts with
Phase-19 runtime authority records fails closed.

## Source / Execution / Runtime State Boundary

This scope RFC is non-mutating.

It does not modify source.

It does not create implementation files.

It does not create tests.

It does not create fixtures.

It does not create validators.

It does not create receipts.

It does not create CI workflows.

It does not execute code.

It does not start a process.

It does not create runtime state.

It does not depend on runtime-observed state.

It does not install, load, or execute packages.

It does not load modules.

It does not create workspace runtime or real mounts.

It does not load or instantiate plugins.

It does not issue capabilities.

It does not publish registry entries.

It does not assign trust.

It does not accept or merge source.

Any source, execution, process, runtime state, package, module, workspace,
plugin, capability, registry, trust, distribution, deployment, or source
merge reading fails closed.

## Kernel And ABI Boundary

The kernel ABI remains frozen.

This scope RFC does not authorize:

1. New syscalls.
2. Kernel ABI expansion.
3. Syscall ID changes.
4. Syscall count changes.
5. ABI version changes.
6. Ring0 policy movement.
7. Kernel source modification.
8. Runtime source modification.
9. Workflow-threshold changes.
10. Baseline changes.
11. Dependency changes.

Any kernel, ABI, workflow-threshold, baseline, dependency, or Ring0 policy
reading fails closed.

## Later Implementation Package Dependency

This scope RFC is a prerequisite governance input for a later first bounded
implementation package decision path only if such a path is separately
reviewed and authorized.

A later first bounded implementation package decision, if ever authorized,
must define:

1. Exact implementation package subject.
2. Exact source boundary.
3. Exact non-execution boundary.
4. Exact validator boundary.
5. Exact receipt boundary.
6. Exact fixture boundary.
7. Exact CI gate boundary.
8. Exact fail-closed validation behavior.
9. Exact evidence requirements.
10. Exact post-merge verification.
11. Exact denied readings for runtime implementation procedure, code
    execution, process start, runtime state creation, package loading,
    package execution, capability issuance, registry publication, trust
    assignment, distribution execution, deployment, source acceptance, and
    source merge authority.

Until such a reviewed decision exists, no implementation package authority is
granted.

Implementation package presence is not runtime implementation procedure.

Implementation package presence is not code execution.

Implementation package presence is not process start.

Implementation package presence is not runtime state creation.

Implementation package presence is not package loading or package execution.

## Scope Invariants

Every later Phase-21 RFC must preserve these first bounded implementation
scope invariants:

1. First Bounded Implementation Scope is userspace-only.
2. First Bounded Implementation Scope is non-executing.
3. First Bounded Implementation Scope is validator, receipt, fixture, and
   CI gate oriented.
4. First Bounded Implementation Scope is fail-closed.
5. First Bounded Implementation Scope is exact-SHA evidence oriented.
6. Scope decision is not implementation package.
7. Scope decision is not runtime implementation procedure.
8. Scope decision does not modify source.
9. Scope decision does not implement code.
10. Scope decision does not execute code.
11. Scope decision does not start a process.
12. Scope decision does not create runtime state.
13. Scope decision does not install packages.
14. Scope decision does not load packages.
15. Scope decision does not execute packages.
16. Scope decision does not issue capabilities.
17. Scope decision does not publish registry entries.
18. Scope decision does not assign trust.
19. Scope decision does not grant source merge authority.
20. Scope decision does not create validators.
21. Scope decision does not create receipts.
22. Scope decision does not create fixtures.
23. Scope decision does not create CI workflows.
24. Scope decision does not broaden Phase-19 runtime authority.
25. Scope decision does not reopen Phase-20.
26. Later implementation package authority requires a separate reviewed
    decision path, if ever authorized.
27. Ambiguity fails closed.

Violation of any invariant fails closed.

## Publication Boundary

If this scope RFC is merged, the landing SHA publishes only this scope
record. The landing SHA must not be read as implementation package authority,
runtime implementation procedure, source modification authority, code
implementation authority, code execution authority, process start authority,
runtime state authority, package loading authority, package execution
authority, capability issuance authority, registry publication authority,
trust assignment authority, source merge authority, implementation
authority, or general runtime authority.

The scope remains bound to:

```text
ae3f9f05cad36451e49a81e4ccfe593d7a9f9ec6
```

Any later technical change, authority expansion, implementation package
proposal, procedure proposal, source modification, execution authority,
runtime state, or package, capability, registry, trust, distribution,
deployment, Semantic CLI, AI Runtime, agent, or source merge behavior
requires a separate reviewed decision path.

## Architecture Signature

**Prepared by:** Kenan AY
**Role:** AykenOS Architecture Steward
**Document type:** Phase-21 RFC
**Architecture status:** Draft RFC / pending architectural review
**Authority notice:** This signature identifies the architectural authorship
of this RFC. It grants no implementation package authority, runtime
implementation procedure authority, source modification authority, code
implementation authority, code execution authority, process start authority,
general runtime authority, unbounded execution authority, runtime state
authority, implementation authority, implementation approval authority,
source merge authority, trust authority, evidence authority, acceptance
authority, proof authority, constitutional authority, registry authority,
distribution authority, publication authority, capability issuance authority,
package authority, deployment authority, module authority, plugin authority,
Semantic CLI authority, AI Runtime authority, agent authority, or Ring0
authority.

## Conclusion

Phase-21 First Bounded Implementation Scope is defined only as:

```text
userspace-only
non-executing
validator / receipt / fixture / CI gate oriented
fail-closed
exact-SHA evidence oriented
```

This scope RFC does not create an implementation package, define runtime
implementation procedure, modify source, implement code, execute code, start
a process, create runtime state, install packages, load packages, execute
packages, issue capabilities, publish registry entries, assign trust, accept
source, merge source, broaden Phase-19 runtime authority, reopen Phase-20, or
expand kernel ABI/syscall authority.

Any later first bounded implementation package requires a separate reviewed
decision path and exact-SHA evidence.
