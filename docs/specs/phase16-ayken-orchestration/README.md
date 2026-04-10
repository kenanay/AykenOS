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

## Out of Scope

- DSL compiler redesign
- `ayken` backend compiler promotion
- automatic evidence mutation
- any local workflow that claims official closure without remote confirmation

## Related Specs

- `docs/specs/authority-lineage-v1/README.md`
