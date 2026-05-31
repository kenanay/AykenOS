# Phase-18 Transition Decision - Platform Constitution

This document is subordinate to PHASE 0 - FOUNDATIONAL OATH and
`ARCHITECTURE_FREEZE.md`. In case of conflict, the foundational oath and the
freeze contract prevail.

**Status:** TRANSITION DECISION PACKAGE / PHASE-18 NOT ACTIVATED
**Decision date:** 2026-05-31
**Authority basis:** `phase17-official-closure` at
`416a5392afbe217e16d26a59e2e1716fdfa9c8f6`
**Attribution:** Kenan AY - informational documentation metadata only; not
runtime, merge, or execution authority.

## Decision

Phase-18 does not expand the AykenOS kernel. Phase-18 defines the fail-closed
platform constitution for modules, packages, capabilities, workspaces, trust
classification, and plugin boundaries on top of the frozen execution substrate.

Phase-18 is therefore the **Platform Constitution** phase, not a continuation
of kernel runtime bring-up and not a new syscall phase.

This package records the intended Phase-18 direction. It does not by itself
change `docs/roadmap/CURRENT_PHASE` to `18`. Activation requires an explicit
transition update that points `CURRENT_PHASE` and the active roadmap surfaces to
this decision.

## Closure Preconditions Already Satisfied

Phase-17 is officially closed and no longer blocks a Phase-18 transition
decision:

| Evidence | Status |
|---|---|
| Official closure tag | `phase17-official-closure` |
| Tag subject | `416a5392afbe217e16d26a59e2e1716fdfa9c8f6` |
| Closure record | `reports/phase17_official_closure_candidate/closure_decision_record.json` |
| Closure manifest | `reports/phase17_official_closure_candidate/closure_manifest.json` |
| Bounded Phase-17 runtime/QEMU evidence | PASS on the exact closure subject |
| Strict freeze and performance evidence | PASS on the exact closure subject |

The Phase-17 closure proves the reviewed execution substrate evidence for the
closure subject. It does not grant Phase-18 activation, kernel ABI expansion,
or future runtime authority.

## Mandatory Decisions

| Decision | Verdict |
|---|---|
| `PHASE18_KERNEL_EXPANSION` | `FORBIDDEN` |
| `PHASE18_NEW_SYSCALLS` | `FORBIDDEN` |
| `PHASE18_RING0_POLICY` | `FORBIDDEN` |
| `PHASE18_AI_AUTHORITY` | `FORBIDDEN` |
| `PHASE18_PLATFORM_CONSTITUTION` | `REQUIRED` |

Any kernel ABI or syscall expansion requires a separate phase decision, RFC,
evidence plan, reviewed authority boundary, and closure process. Phase-18 must
not be used as an implicit exception path for kernel growth.

## Kernel ABI Versus Platform ABI

Phase-18 is built on a strict separation:

| Layer | Boundary |
|---|---|
| Kernel ABI | Frozen syscall v2 surface, syscall IDs `1000-1011`, 12 syscalls, Ring0 mechanism only |
| Platform ABI | Module manifest, package metadata, workspace lifecycle, capability contract, trust classification, plugin boundary, semantic contract |

Kernel ABI is the frozen execution substrate. Platform ABI is the governed
userspace contract that defines how AykenOS can accept modules, packages,
workspaces, plugins, and later semantic systems without widening Ring0.

## Phase-18 Scope

Phase-18 must answer one constitutional question:

How can a developer add something to AykenOS in a safe, verifiable,
fail-closed, and kernel-preserving way?

Required Platform Constitution outputs:

1. Module manifest schema.
2. Package metadata schema.
3. Workspace lifecycle contract.
4. Capability contract for platform resources.
5. Trust classification model.
6. Plugin boundary contract.
7. Platform ABI validation gate.
8. Minimal reference examples that do not create new runtime authority.

## Non-Goals

Phase-18 must not include:

1. New syscalls.
2. Kernel ABI expansion.
3. Ring0 policy engines.
4. Kernel-resident plugin systems.
5. AI Runtime authority.
6. Semantic CLI as an execution verdict source.
7. Scheduler, SMP, or BCIB expansion as the primary roadmap objective.

## Trust Classification Model

Trust classification is required, but it must not be confused with permission.

**Trust level does not grant capability.**

Trust may affect:

1. Install eligibility.
2. Enable/disable policy.
3. Update policy.
4. Distribution channel.
5. Review requirement.
6. Quarantine or revocation handling.

Actual access is granted only through an explicit capability contract. No trust
level may bypass the frozen kernel ABI, capability validation, workspace
boundary, or package policy.

Initial trust classifications may include:

| Classification | Meaning |
|---|---|
| `local` | Locally developed or manually installed; no distribution trust implied |
| `experimental` | Allowed only under explicit development policy |
| `signed` | Cryptographically signed by an accepted identity |
| `verified` | Passed required platform validation gates |
| `trusted` | Accepted for a defined distribution or workspace policy |
| `revoked` | Must not install, enable, update, or execute |

These classifications are policy inputs. They are not authority tokens.

## Validation Backlog

The following items remain important and visible, but they do not define the
main Phase-18 direction:

1. BCIB completeness.
2. SMP safety.
3. Exhaustive race coverage.
4. Advanced interrupt validation.
5. Broader scheduler stress evidence.

These belong to a deferred validation track unless a separate reviewed phase
decision makes one of them the active scope.

## Forward Phase Line

| Phase | Direction |
|---|---|
| Phase-18 | Platform Constitution |
| Phase-19 | Platform Runtime MVP |
| Phase-20 | Capability Ecosystem / Module Registry |
| Phase-21 | Semantic CLI Integration |
| Phase-22 | AI Runtime Foundation |
| Phase-23+ | Agent Systems |

AI Runtime is intentionally outside Phase-18. Its non-determinism, authority
ambiguity, hallucination risk, and verification difficulty require a later
foundation after the platform contracts are explicit.

## Fail-Closed Activation Rule

Phase-18 must remain inactive if any of the following are true:

1. `CURRENT_PHASE` has not been explicitly transitioned.
2. Platform Constitution scope is ambiguous.
3. Kernel ABI expansion is bundled into the transition.
4. Trust classification is treated as capability grant.
5. AI Runtime or Semantic CLI is made an execution authority.
6. Required closure references cannot be verified.

The safe default is no activation.

## Supersession Notice

`PHASE18_ROADMAP.md` is a historical pre-closure runtime-validation roadmap.
Its QEMU/runtime validation items are retained as historical context and as
deferred validation backlog, not as the active Phase-18 objective.
