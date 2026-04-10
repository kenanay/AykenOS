# Phase-16: Ayken Orchestration Layer

## Purpose

Provide a controlled orchestration surface for build, verification, and closure without redefining runtime or closure authority.

## Scope (Phase-16)

- `ayken status`
- `ayken risk`
- `ayken gate all`
- `ayken closure status --json`
- `ayken closure verify`
- `ayken head verify`
- `ayken head lineage`
- `ayken bcib verify`
- `ayken bcib hash`
- `ayken bcib inspect`

## Active First Slice (Phase-16A)

Phase-16A does not widen the command surface first. It closes one real,
end-to-end execution path on top of the already landed runtime and authority
surfaces.

Locked first-slice command surface:

- `list <context>`
- `show <context> <id>`
- `query <context> where <predicate>`

Everything else in the semantic/orchestration path is explicitly unsupported in
Phase-16A and must fail closed.

Locked Phase-16A pipeline:

`DSL -> semantic parse -> canonical IR -> canonical IR validation -> BCIB lowering -> submission bridge -> runtime submit -> proof/replay`

## Implementation Naming

Implementation file and module names must be purpose-based, not phase-based.
Phase labels are allowed in roadmap/spec text, but production code should carry
stable responsibility names such as `canonical_query`, `submission_bridge`, or
`orchestration`.

## Canonical IR Rule

Phase-16A does not invent a new semantic IR. It freezes the existing
register-based execution-plan subset as the canonical contract between semantic
parsing and BCIB lowering.

Canonical IR subset:

- `LoadContext`
- `LoadField`
- `LoadLiteral`
- `Compare`
- `LogicalOp`
- `ApplyFilter`
- `Return`

Phase-16A register discipline:

- `r0` is the active context/result register
- no implicit mutation of hidden state
- no direct production-path `DSL -> BCIB` shortcut

## Phase-16A BCIB Subset

Production lowering for the first slice may emit only:

- `DataQuery`
- `End`
- optional `TraceEmit`

The following opcodes are out of scope for Phase-16A production lowering:

- `Nop`
- `DataCreate`
- `DataAdd`
- `UiRender`
- `AiAsk`

## Authority Model

- official closure authority is phase-tagged and immutable
- verified head authority is CI-backed and SHA-scoped
- verified head records use full SHA filenames plus binding-hash integrity
- verified head records are local CI projections; only an exact current-SHA record may satisfy `head verify`
- authority lineage, when added, is advisory only and MUST NOT inherit verified authority across SHAs
- local `ayken` commands are advisory and may not override CI-confirmed truth
- `ayken closure verify` validates official closure only
- `ayken head verify` validates CI-backed development head records only
- `ayken head lineage` reports nearest verified ancestors without changing authority
- `ayken risk` reports advisory interpretation only and must not affect authority
- `ayken bcib inspect` reports BCIB structure/decode signals together with advisory authority, lineage, and risk context only
- a verified head is not an official closure

## Constraints

- `closure verify` and `gate all` are fail-closed
- `head verify` is fail-closed against `reports/verified_heads/<FULL_SHA>.json`
- `head lineage` is advisory-only and must not modify `effective_authority`
- `risk` is advisory-only and must not modify `effective_authority`
- `bcib inspect` is advisory-only and must not emit execution-safety or authority claims
- `gate all --json` emits normalized per-gate result summaries plus advisory risk suitable for pipeline input
- `status`, `closure status`, `bcib hash`, and `bcib inspect` are advisory observation surfaces
- no authority override from local tools
- no mutation of closure artifacts without CI confirmation
- reuse existing `proof-verifier`, `semantic-cli`, and `bcib-runtime` surfaces instead of copying logic
- Phase-16A orchestration is a pure translation/submission layer; it does not make policy decisions
- Phase-16A orchestration may reject or package work; it may not reinterpret semantic intent
- Phase-16A submission is submit-only; it does not grow a parallel execution authority plane
- Phase-16A production lowering must be NOP-free
- unsupported DSL commands or predicates must return explicit errors, never silent fallback

## Out of Scope

- DSL compiler redesign
- `ayken` backend compiler promotion
- automatic evidence mutation
- any local workflow that claims official closure without remote confirmation
- mutation, AI, UI, debug, system, loop, or control-flow widening inside the Phase-16A production path

## Related Specs

- `docs/specs/authority-lineage-v1/README.md`
- `docs/specs/phase16-ayken-orchestration/requirements.md`
- `docs/specs/phase16-ayken-orchestration/tasks.md`
