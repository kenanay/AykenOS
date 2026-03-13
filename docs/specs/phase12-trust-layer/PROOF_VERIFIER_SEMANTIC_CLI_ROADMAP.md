# Proof Verifier Semantic CLI Roadmap

**Version:** 1.0
**Status:** Draft
**Date:** 2026-03-08
**Phase:** Kernel Phase 12 - Trusted Proof Transport and Distributed Verification
**Type:** Non-normative implementation roadmap
**Related Spec:** `requirements.md`, `tasks.md`, `PROOF_VERIFIER_CRATE_ARCHITECTURE.md`, `GENERIC_DETERMINISTIC_TRUTH_VERIFICATION_ARCHITECTURE.md`, `VERIFICATION_CONTEXT_PORTABILITY_AND_DISTRIBUTION_PROTOCOL.md`, `PROOF_BUNDLE_ATTACK_SURFACE_SECURITY_MODEL.md`

---

## 1. Purpose

This document evaluates the current Semantic CLI direction for AykenOS Phase-12 and turns it into an implementation roadmap that is technically compatible with:

- the current verifier-core architecture
- the Phase-12 closure criteria
- the AykenOS separation between subject, context, authority, and verdict surfaces

The goal is not to maximize command count.

The goal is to expose the existing truth surfaces through a thin, deterministic, offline-first operator interface.

---

## 2. Current Repo State

As of 2026-03-08:

- the verifier core exists as a library-first crate at `ayken-core/crates/proof-verifier/`
- `P12-07`, `P12-08`, and `P12-09` are locally gated
- signed receipt, audit ledger, verifier authority resolution, and cross-node parity local gates already exist
- a dedicated thin CLI binary now exists at `ayken-core/crates/proof-verifier/src/bin/proof-verifier.rs`
- local `ci-gate-proof-verifier-cli` evidence now validates the Stage-1 offline command surface
- no `proofd` service surface exists yet

This means the system now has a closure-minimum CLI and must keep later semantic growth staged behind verifier-core and `proofd` boundaries.

---

## 3. Consistency Assessment

The current Semantic CLI direction is mostly correct.

The following statements are architecturally aligned:

- the primary operator entrypoint should remain `verify`
- evidence generation should be explicit rather than implicit
- audit append should not happen by default
- subject/context/authority surfaces should be inspectable
- ABDF / BCIB should be consumed and explained from the verifier side, not generated as the CLI's primary job

However, three scope corrections are necessary.

### 3.1 `P12-10` Closure Minimum Is Smaller Than The Full Semantic Vocabulary

The current Phase-12 normative requirement for CLI is still narrow:

- offline bundle verification
- external policy and registry inputs
- human-readable output
- machine-readable JSON output
- verdict subject binding fields
- thin wrapper behavior

Therefore the following are architecturally coherent, but not closure-blocking for `P12-10`:

- `verify receipt`
- `inspect subject|context|authority`
- `parity compare`
- `gate <name>`

These belong in a staged rollout, not in the strict closure minimum.

### 3.2 `parity compare` Is Valid Only As A Local Artifact Comparison In Phase-12

`parity compare` is acceptable in Phase-12 only when it compares local artifacts such as:

- two receipts
- two parity reports
- two local verification outputs

Remote query, discovery, or network-backed parity behavior belongs to:

- `P12-13` exchange protocol
- `P12-16` `proofd`

### 3.3 ABDF / BCIB Generation Is Out Of Scope For The CLI Closure Minimum

The verifier CLI MAY inspect or summarize ABDF / BCIB bindings.

It SHOULD NOT make ABDF / BCIB production the center of `P12-10`.

The correct initial role is:

- read
- verify
- explain

not:

- generate
- orchestrate build production
- replace producer-side tooling

---

## 4. Design Guardrails

The CLI MUST follow these guardrails:

- offline-first
- deterministic
- explicit input and output
- no implicit persistence
- no implicit ledger mutation
- no hidden network behavior
- thin wrapper over `verify_bundle`

The CLI MUST NOT:

- mutate bundle identity
- redefine context semantics
- redefine authority semantics
- inline service-discovery behavior before `proofd`
- collapse verifier-core logic into CLI formatting code

---

## 5. Recommended Command Model

The long-term Semantic CLI model remains:

`proof-verifier <surface-or-pipeline-domain> <operation>`

But the practical operator path SHOULD remain verify-centric:

- `proof-verifier verify bundle ...`
- later: `proof-verifier verify receipt ...`

The reason is simple:

the user-facing center of gravity is still:

`verify(subject, context, authority) -> verdict`

Debug and introspection commands should remain secondary.

---

## 6. Stage 1: Phase-12 Closure Minimum

This stage is the smallest correct implementation that satisfies `P12-10` without leaking into later-phase service behavior.

### 6.1 Required Command

The required initial command is:

```text
proof-verifier verify bundle <bundle_path> --policy <policy.json> --registry <registry.json>
```

### 6.2 Required Output

The command SHALL provide:

- human-readable verdict output by default
- machine-readable JSON via `--json`
- explicit verdict binding fields:
  - `bundle_id`
  - `trust_overlay_hash`
  - `policy_hash`
  - `registry_snapshot_hash`

### 6.3 Closure-Minimum Flags

The Stage-1 closure-minimum flag set is:

- `--json`

No broader flag surface is required for local `P12-10` closure.

### 6.4 Post-Minimum Optional Flags

The following flags remain architecturally valid, but are post-minimum Stage-1 or later additions:

- `--evidence-dir <dir>`
- `--run-id <id>`
- `--trace`
- `--explain`

`--trace` and `--explain` are especially compatible with Stage-1 because they expose verifier-core reasoning without changing trust semantics.

### 6.5 Stage-1 Exit Contract

Default exit semantics SHOULD remain thin:

- `0` = CLI executed successfully and emitted a deterministic verification result
- non-zero = usage, parsing, config, or runtime error

Verification verdict itself SHOULD remain in stdout / JSON output, not overloaded into shell semantics by default.

If verdict-sensitive exit behavior is later needed for CI convenience, it SHOULD be explicit, for example through a later `--strict-exit` mode.

### 6.6 Stage-1 Non-Goals

Stage-1 MUST NOT include:

- remote parity query
- network fetch
- exchange protocol behavior
- default audit append
- ABDF generation
- BCIB generation

### 6.7 Stage-1 Evidence Layout

If `--evidence-dir` is supplied, the CLI SHOULD emit:

```text
<evidence-dir>/
  verification/
    subject.json
    verdict.json
  trace/
    verification_trace.json
```

This Stage-1 evidence layout is intentionally limited to verifier-core outputs that already exist or can be derived without introducing new authority or distributed-context semantics into the CLI closure minimum.

Stage-1 SHOULD additionally permit lightweight local summaries such as:

- `verification/subject_hashes.json`
- `verification/policy_registry_summary.json`

Stage-1 SHOULD NOT require a full:

- `context/verification_context.json`
- `authority/authority_resolution.json`

because the current `P12-10` closure minimum is still defined around offline bundle verification with external policy and registry inputs, not full context portability or verifier-authority inspection surfaces.

Those richer files become appropriate in Stage-2 or later once the CLI explicitly exposes:

- context inspection
- authority inspection
- receipt verification
- local parity comparison

No files should be written unless `--evidence-dir` is explicitly supplied.

---

## 7. Stage 2: Post-Closure Semantic Expansion

This stage remains offline and local, but exposes more of the truth surfaces for debugging and inspection.

Recommended additions:

- `proof-verifier verify receipt <receipt_path> ...`
- `proof-verifier inspect subject <bundle_path>`
- `proof-verifier inspect context <context_path>`
- `proof-verifier inspect authority <authority_input>`
- `proof-verifier parity compare <left.json> <right.json>`
- `proof-verifier gate <name>`

This stage is where ABDF / BCIB binding summaries become appropriate.

Examples:

- `inspect subject` may display `abdf_snapshot_hash`
- `inspect context` may display `bcib_plan_hash`
- `verify bundle --json` may include ABDF / BCIB binding summaries

This stage still SHOULD NOT add remote service semantics.

---

## 8. Stage 3: `proofd`-Adjacent Extension

This stage begins only after:

- `P12-13` exchange protocol
- `P12-16` `proofd`

Possible additions:

- remote context resolution orchestration
- exchange import/export helpers
- remote parity query
- distributed authority lookup
- service-backed context fetch

These commands are valid only when the service and transport contracts already exist.

They MUST NOT be backported into Stage-1 or Stage-2 as ad hoc CLI behavior.

---

## 9. ABDF / BCIB Integration Model

The correct verifier-side integration is:

- inspect binding
- verify binding
- explain binding

The incorrect closure-minimum integration is:

- generate ABDF
- generate BCIB
- replace producer pipeline

Therefore the right early integrations are:

- expose `abdf_snapshot_hash` in subject inspection
- expose `bcib_plan_hash` in context inspection or verification summaries
- include ABDF / BCIB binding status in JSON output

This keeps the CLI aligned with verifier responsibility rather than producer responsibility.

---

## 10. Implementation Mapping

The cleanest Rust implementation path is:

- keep `ayken-core/crates/proof-verifier/src/lib.rs` as the engine
- add a thin binary entrypoint at:
  - `ayken-core/crates/proof-verifier/src/bin/proof-verifier.rs`

The binary SHOULD:

- parse arguments
- load policy and registry inputs
- call `verify_bundle`
- render text or JSON output
- optionally emit explicit evidence files

The binary SHOULD NOT:

- duplicate verification logic
- compute alternative verdicts
- reimplement receipt or authority rules outside the library

---

## 11. PR-by-PR Roadmap

The safest execution order is:

### PR1: CLI Skeleton

Invariant:

`proof-verifier` exists as a thin offline binary over the existing library.

Deliver:

- binary entrypoint
- `verify bundle`
- `--policy`
- `--registry`
- human-readable output

Local status:

- implemented

### PR2: JSON Output Contract

Invariant:

JSON output exposes the same verdict subject tuple as verifier-core.

Deliver:

- `--json`
- machine-readable verdict output
- `bundle_id`, `trust_overlay_hash`, `policy_hash`, `registry_snapshot_hash`

Local status:

- implemented

### PR3: Explicit Evidence Emission

Invariant:

CLI writes nothing unless explicitly asked.

Deliver:

- `--evidence-dir`
- `--run-id`
- `--trace`
- `--explain`
- `ci-gate-proof-verifier-cli`

Local status:

- partially implemented
- local `ci-gate-proof-verifier-cli` is active
- explicit CLI-side evidence emission flags remain deferred so Stage-1 stays thin

### PR4: Offline Semantic Introspection

Invariant:

debug surfaces remain local and read-only.

Deliver:

- `inspect subject`
- optional `inspect context`
- optional `inspect authority`
- ABDF / BCIB binding summaries

### PR5: Local Semantic Comparison

Invariant:

parity comparison remains artifact-local until `proofd`.

Deliver:

- `verify receipt`
- `parity compare`
- optional `gate <name>` wrapper

---

## 12. Final Recommendation

The current Semantic CLI direction is correct, but only if it is staged.

The right Phase-12 interpretation is:

- build a thin offline CLI first
- keep `verify` as the primary UX
- expose truth surfaces through explicit inspect and JSON contracts
- make evidence generation opt-in
- keep audit append opt-in
- defer network/service behavior until `proofd`

So the correct roadmap is not:

`build the full semantic CLI now`

It is:

`build the closure-minimum CLI now, then expand semantically without breaking Phase-12 boundaries`
