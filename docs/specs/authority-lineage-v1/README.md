# Authority Lineage v1

## Purpose

Define an advisory lineage model for locating the nearest CI-verified ancestor of a development HEAD without expanding binding authority.

## Status

- draft
- advisory only
- not a closure mechanism
- not a verified-head override

## Core Rule

Exact authority and lineage diagnostics are separate surfaces.

- `ayken closure verify` may only confirm official closure authority.
- `ayken head verify` may only confirm exact-SHA verified-head authority.
- lineage resolution may describe ancestry relative to verified heads, but it MUST NOT produce `head_verified=true`.

## Definitions

- **Official Closure**
  - Phase-tagged, immutable, CI-confirmed closure package
- **Verified Head**
  - Exact current SHA backed by remote `ci-freeze` evidence and a valid binding hash
- **Authority Lineage**
  - Advisory traversal from the current SHA toward its ancestors to locate the nearest verified head

## Scope

This spec covers:

- nearest verified ancestor discovery
- lineage diagnostics for `ayken status` or a future `ayken head lineage`
- trust-boundary rules for advisory ancestry reporting

This spec does not cover:

- closure mutation
- verified-head promotion
- authority inheritance
- majority voting or truth election

## Resolution Model

Given a current `HEAD`:

1. Resolve exact authority first.
2. If exact verified-head authority is absent, traverse ancestry in advisory mode only.
3. Stop at the first ancestor whose full SHA has a valid record under `reports/verified_heads/<FULL_SHA>.json`.
4. Report lineage diagnostics without changing effective authority.

If no verified ancestor is found within the traversal limit, the lineage result is empty.

## Traversal Rules

- first-parent traversal only
- maximum depth: `32`
- no merge-graph authority resolution
- no branch name heuristics
- no remote lookup during local lineage evaluation
- dirty worktree marks lineage as tainted advisory output

## Advisory Output Contract

Lineage-aware status surfaces may expose:

- `nearest_verified_ancestor`
- `ancestor_distance`
- `lineage_resolved`
- `lineage_tainted`
- `lineage_note`

These fields are diagnostic only.

The following fields remain binding and exact:

- `closure_authority_confirmed`
- `head_verified`
- `effective_authority`

## Trust Boundary

Lineage MUST NOT:

- promote an unverified SHA into verified authority
- reinterpret a verified ancestor as proof for the current SHA
- override official closure state
- suppress fail-closed behavior in `closure verify` or `head verify`

Lineage MAY:

- explain that the current SHA descends from a verified ancestor
- help operators understand CI recency and local drift
- support future dashboards and audit tooling

## Failure Semantics

- exact authority commands remain fail-closed
- lineage diagnostics remain advisory
- missing git ancestry data yields `lineage_resolved=false`
- malformed verified-head records are ignored for lineage promotion and surfaced as advisory diagnostics

## Future Hardening

- add `tree_sha` to verified-head bindings
- add `binding_version`
- adopt canonical JSON when verified-head records need cross-implementation hashing
- define a future `ayken head lineage` command as a read-only diagnostic surface
