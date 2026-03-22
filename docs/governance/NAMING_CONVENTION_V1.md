# Naming Convention V1

Status: ACTIVE (governance guidance + scoped CI enforcement)

## Purpose

This document freezes execution-path naming policy without forcing a mass rename
of already-landed kernel/runtime surfaces.

The goal is:

- preserve backward stability for existing names
- force new execution-path work to use execution-centric terminology
- prevent classical OS naming from silently re-entering new execution-path code

This document does not redefine ABI ownership or runtime semantics. It governs
future naming choices only.

## Core Rule

Two rules apply together:

1. Existing names remain stable unless an explicit refactor plan is ratified.
2. New execution-path additions SHOULD prefer the execution-centric terms
   defined below and MUST avoid the forbidden terms when they describe
   execution-model concepts.

This is intentionally a forward-only naming freeze.

## Frozen Existing Terms

The following categories are treated as frozen existing terms:

- already-landed ABI names such as `execution_inbox`, `EXECUTION_INBOX_VA`, and
  `ayken_execution_inbox_v1_t`
- already-landed execution-path identifiers such as `active_execution_id`
- legacy scheduler/process names that are already embedded in stable files

Frozen existing terms are not retroactively renamed by this policy.

## Preferred New Terms

New execution-path code and nearby kernel/runtime documentation SHOULD prefer
the following terminology:

- `execution_slot`
- `execution_id`
- `execution_context`
- `executor`
- `dispatch`
- `delivery surface`
- `bcib_frame`
- `bcib_window`

These are preferred guidance terms, not a compile-time ontology mandate.

## Forbidden New Terms

In scoped execution-path additions, the following terms MUST NOT be introduced
for execution-path concepts:

- `worker`
- `thread`
- `task`
- `job`

This prohibition applies to identifiers, comments, and user-facing strings when
they describe new execution-path surfaces.

## Golden Rule

If the concept belongs to the execution model, use execution-centric naming.

If the concept belongs to actual OS or runtime primitives, classical naming is
allowed.

Naming should guide architecture, not force architecture into misleading terms.

## Naming Decision Table

### Execution-Centric Areas

| Situation | Preferred Term | Avoid |
| --- | --- | --- |
| Kernel execution dispatch | `execution_slot` | `task`, `job` |
| Execution identity | `execution_id` | `thread_id` |
| Active execution model context | `execution_context` | `process context` |
| Runtime execution entity | `executor` | `worker` |
| Scheduler-side delivery action | `dispatch` | `schedule task` |
| Worker-visible delivery surface | `delivery surface` | `worker inbox` |

In these areas, classical OS naming is misleading and should not be introduced
for new execution-path concepts.

### Legacy / Frozen Areas

| Situation | Rule |
| --- | --- |
| Existing struct or ABI names | Leave unchanged |
| Existing scheduler naming | Leave unchanged |
| Existing `proc` / `sched` terminology | Preserve for stability |

In these areas, stability matters more than terminology cleanup.

### Real OS / Runtime Primitive Areas

| Situation | Allowed Classical Naming |
| --- | --- |
| Real CPU or host thread primitive | `thread` |
| Host runtime or thread-pool worker | `worker` |
| Parallel processing pool | `thread pool` |
| Actual OS scheduling primitive | `task` |

If the term is semantically correct for a real OS/runtime primitive, it MAY be
used.

### Borderline Cases

| Scenario | Preferred Resolution |
| --- | --- |
| Execution model mixed with scheduler logic | Name the execution-side concept separately |
| Runtime entity executes BCIB payloads | Prefer `executor` |
| Queueing mixed with execution ownership | Keep queue terminology distinct from execution terminology |
| Async or host-runtime internals | Choose names based on actual primitive semantics |

## Exceptions

The following cases are explicitly allowed:

- frozen existing symbols and ABI names
- allowlisted legacy files that still carry historical terminology
- comment lines marked with `LEGACY:` when the old term must be referenced for
  backward clarity
- non-execution domains where the term is semantically correct, such as host
  thread-pool tooling outside the scoped gate

## Enforcement

`scripts/ci/check_naming_convention.sh` is the enforcement surface for this
policy.

The gate is intentionally narrow:

- it scans only diff-added lines
- it scans only scoped execution-path files
- it skips allowlisted legacy files
- it fails closed when new forbidden terms appear outside those exemptions

The gate is intentionally negative-only:

- it blocks bad execution-path naming patterns
- it does not force one exact preferred replacement
- final semantic judgment still belongs to the developer

This keeps the rule enforceable without producing repo-wide false positives.

## Scope Files

The active scope is defined in:

- `scripts/ci/naming-convention-scope.regex`

Legacy execution-path files that remain exempt are defined in:

- `scripts/ci/naming-convention-legacy-allow.regex`

Forbidden term patterns are defined in:

- `scripts/ci/naming-convention-deny.regex`

## Guidance For New Work

When adding new execution-path code:

- use `executor`, not `worker`
- use `dispatch`, not `task scheduling` for execution delivery concepts
- use `delivery surface`, not `worker inbox` in new abstractions
- keep existing frozen names untouched unless a dedicated rename plan exists
- if a classical term is semantically correct for a real OS/runtime primitive,
  it may still be used

Violating this rule creates two risks:

- non-deterministic architectural language across adjacent modules
- accidental drift back toward classical OS semantics in the execution path
