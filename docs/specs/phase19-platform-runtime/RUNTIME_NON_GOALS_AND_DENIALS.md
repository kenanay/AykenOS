# Phase-19 Runtime Non-Goals And Denials

This document is subordinate to `PHASE19_RUNTIME_DECISION.md`,
`../../../PHASE19_POINTER_TRANSITION_DECISION.md`,
`../phase18-platform-constitution/AUTHORITY_DRIFT_GUARD.md`,
`../phase18-platform-constitution/TERMINOLOGY_AUDIT.md`, and the Phase-19
Runtime RFC set. In case of conflict, those documents prevail.

**Status:** ACTIVE RFC / RUNTIME IMPLEMENTATION NOT AUTHORIZED
**Contract id:** `ayken.phase19.runtime.non_goals.denials.v1`
**Authority boundary:** Documentation/specification only; not runtime source
code, installer, loader, workspace runtime, plugin host, capability issuer,
trust issuer, Semantic CLI authority, AI Runtime authority, registry, agent,
syscall, kernel ABI expansion, merge authority, or closure authority.

## Purpose

This RFC records what Phase-19 Runtime MVP planning must not do.

The denial list exists to prevent Phase-19 from absorbing Phase-20, Phase-21,
Phase-22, or Phase-23+ work by terminology or convenience.

## Core Rule

```text
MVP boundary denial is mandatory
```

If a future Phase-19 proposal conflicts with this file, the proposal must fail
closed or move to a later reviewed phase decision.

## Denied In Phase-19 MVP

The Phase-19 Runtime MVP must not include:

1. Package installation.
2. Package execution.
3. Module loading.
4. Loader handles.
5. Plugin loading.
6. Plugin instantiation.
7. Workspace creation.
8. Real filesystem mounts.
9. Capability token minting.
10. Capability issuance.
11. Trust assignment.
12. Registry publication.
13. Marketplace behavior.
14. Semantic CLI execution authority.
15. AI Runtime authority.
16. Agent systems.
17. Kernel loader behavior.
18. New syscalls.
19. Kernel ABI expansion.
20. Ring0 policy.
21. Observability-as-authority.

## High-Risk Term Denials

| Term | Safe Phase-19 meaning | Forbidden reading |
|---|---|---|
| `runtime` | Future userspace admission/receipt MVP boundary | Full platform executor |
| `admission` | Evidence record | Workspace creation or mount |
| `receipt` | Digest-bound evidence output | Token, handle, or capability |
| `validated` | Consistent with Platform ABI inputs | Install, load, execute, trust, or issue |
| `enabled` | Not allowed for MVP behavior | Autoload or execution |
| `binding` | Digest/reference relation | Runtime link or loader binding |
| `workspace` | Declarative admission subject | Real filesystem namespace |
| `loader` | Forbidden in Phase-19 MVP | Module or plugin loading |
| `trusted` | Phase-18 classification input only | Capability grant |

Unknown high-risk terms fail closed until audited.

## Later-Phase Ownership

The following work remains outside Phase-19:

1. Phase-20: Capability Ecosystem / Module Registry.
2. Phase-21: Semantic CLI Integration.
3. Phase-22: AI Runtime Foundation.
4. Phase-23+: Agent Systems.

These later phases require their own decision packages, RFC sets, evidence
plans, pointer transitions, and acceptance boundaries.

## Rejection Conditions

A future Phase-19 proposal must be rejected if it:

1. Updates `CURRENT_PHASE` without separate pointer transition.
2. Adds runtime source code before RFC/evidence acceptance.
3. Adds loader, installer, registry, workspace runtime, plugin host, issuer,
   trust, Semantic CLI, AI Runtime, or agent code.
4. Adds new syscalls or changes the frozen kernel ABI.
5. Treats a receipt as a bearer token.
6. Treats validation as authority.
7. Treats trust as capability.
8. Treats admission as mount.
9. Treats plugin compatibility as loading.
10. Omits negative evidence for authority drift.

## Acceptance Boundary

This file is a denial contract. It authorizes no runtime behavior.
