# Phase-16: Ayken Orchestration Layer

## Purpose

Provide a controlled orchestration surface for build, verification, and closure without redefining runtime or closure authority.

## Scope (Phase-16)

- `ayken gate all`
- `ayken closure status --json`
- `ayken bcib verify`
- `ayken bcib hash`
- `ayken bcib inspect`

## Authority Model

- `ci-freeze` is authoritative for official closure and closure artifact mutation
- local `ayken` commands are advisory and may not override CI-confirmed truth

## Constraints

- fail-closed policy enforcement on all commands
- no authority override from local tools
- no mutation of closure artifacts without CI confirmation
- reuse existing `proof-verifier`, `semantic-cli`, and `bcib-runtime` surfaces instead of copying logic

## Out of Scope

- DSL compiler redesign
- `ayken` backend compiler promotion
- automatic evidence mutation
- any local workflow that claims official closure without remote confirmation
