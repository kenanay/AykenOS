# AykenOS Gate Architecture

**Version:** 1.0  
**Status:** Draft (Phase-13 architecture preparation)  
**Date:** 2026-03-13  
**Type:** Normative gate architecture guide

---

## 1. Purpose

This document defines the intended gate architecture for AykenOS as the system grows from:

`trusted deterministic verification`

toward:

`distributed verification observability`

Its purpose is to prevent:

- gate explosion
- duplicated semantic checks
- CI noise without architectural meaning
- drift between technical validators and architectural invariants

The core rule is:

`gates exist to preserve invariants, not to multiply scripts`

---

## 2. Design Goal

AykenOS should operate as:

`verification architecture with executable governance`

That means the gate system must preserve:

- architectural identity
- service boundaries
- artifact semantics
- consumer safety
- verification determinism

while still remaining readable enough that CI answers:

`which invariant failed?`

instead of:

`which subset of scripts turned red?`

---

## 3. The Five Layers

AykenOS uses a five-layer gate model.

Each layer exists to close a different risk class.

### 3.1 Layer 1: Invariant Gates

Purpose:

- preserve AykenOS core architectural identity

Primary risks:

- truth-election drift
- authority drift
- consensus drift
- reputation drift
- determinism drift

Representative examples:

- `ci-gate-graph-non-authoritative-contract`
- `ci-gate-convergence-non-election-boundary`
- `ci-gate-verifier-reputation-prohibition`
- `ci-gate-verification-determinism-contract`

These gates answer:

`did AykenOS stop being AykenOS?`

### 3.2 Layer 2: Service Boundary Gates

Purpose:

- preserve API, namespace, and method boundaries

Primary risks:

- diagnostics becoming control plane
- service namespaces becoming mutation surfaces
- query smuggling of forbidden semantics

Representative example:

- `ci-gate-proofd-observability-boundary`

These gates answer:

`did a service surface drift from read-only observability into control?`

### 3.3 Layer 3: Artifact Contract Gates

Purpose:

- preserve artifact schemas as non-authoritative truth surfaces

Primary risks:

- hidden consensus fields
- authority arbitration fields
- reputation or scoring fields
- semantic drift hidden in descriptive artifacts

Representative examples:

- `ci-gate-graph-non-authoritative-contract`
- `ci-gate-convergence-non-election-boundary`
- `ci-gate-verifier-reputation-prohibition`

These gates answer:

`did an artifact start carrying decision semantics?`

### 3.4 Layer 4: Consumer Safety Gates

Purpose:

- preserve the usage boundary of diagnostics outputs

Primary risks:

- descriptive diagnostics reused as policy input
- replay or execution decisions derived from observability
- routing or priority computed from convergence or topology metadata
- verifier ordering or scheduling biased by observability artifacts
- renamed diagnostics aliases hiding consumer drift

Representative examples:

- `ci-gate-diagnostics-consumer-non-authoritative-contract`
- `ci-gate-diagnostics-callsite-correlation`
- `ci-gate-observability-routing-separation`

These gates answer:

`did a runtime consumer start treating observability as authority?`

### 3.5 Layer 5: Determinism and Execution Purity Gates

Purpose:

- preserve verifier purity and environment independence

Primary risks:

- time dependency
- randomness dependency
- network-visible context
- filesystem-visible context
- ambient host-state dependency

Representative example:

- `ci-gate-verification-determinism-contract`

These gates answer:

`can the same verification input drift across environments?`

---

## 4. Risk Classes

Gate design should normalize around risk classes, not around files.

| Risk Class | Meaning | Typical Drift |
|---|---|---|
| `truth-election-drift` | observability or convergence becoming winner selection | `majority -> truth`, `cluster -> winner` |
| `authority-drift` | descriptive or derived surfaces becoming trust authority | topology or diagnostics promoted into authority |
| `control-plane-drift` | read-only observability becoming mutation or action surface | diagnostics -> retry / override / promote |
| `consumer-misuse-drift` | valid artifacts consumed as decision input | `global_status -> routing`, `largest_cluster -> replay` |
| `topology-feedback-drift` | observability artifacts bias verification routing or scheduling over time | `dominant cluster -> preferred route`, `topology -> verifier ordering` |
| `reputation-drift` | verification history becoming trust scoring | correctness ranking, reliability score |
| `determinism-drift` | verification depending on ambient execution state | time, env, network, fs |
| `replay-boundary-drift` | verification success silently granting replay or execution admission | `verified proof -> replay admission` |

Risk classes are the stable top-level units.

Gates are supporting machinery.

---

## 5. Gate Kinds

AykenOS gates should be modeled in three kinds.

### 5.1 Primitive Checks

Primitive checks are the smallest reusable validators.

Examples:

- forbidden field scan
- forbidden pattern scan
- allow-listed enum validation
- derivation-value validation
- namespace method boundary check
- source consumer scanner
- source-to-sink correlation scanner

Primitive checks should be:

- small
- composable
- reusable across multiple gates

Primitive checks are not the preferred user-facing CI language.

### 5.2 Composite Gates

Composite gates combine primitive checks into an architectural sentence.

Examples:

- `ci-gate-proofd-observability-boundary`
- `ci-gate-convergence-non-election-boundary`
- `ci-gate-diagnostics-consumer-non-authoritative-contract`

Composite gates are the preferred repo targets because they answer:

`which boundary did we preserve or lose?`

### 5.3 Invariant Summaries

Invariant summaries reduce multiple composite gates into a small number of architectural outcomes.

Examples:

- `observability != control`
- `graph != truth inference`
- `convergence != election`
- `descriptive diagnostics != execution input`
- `observability != scheduling`
- `verification != environment dependent`

CI should present invariant summaries before raw gate lists.

---

## 6. Compilation Model

The intended authoring flow is:

1. define invariant
2. define risk class
3. define protected surface
4. define forbidden semantics
5. define evidence source
6. define primitive checks
7. compose gate target
8. reduce results into invariant summary

The author should primarily write:

- invariant
- risk class
- protected surfaces
- forbidden semantics
- authoritative failure meaning

The rest should be derivable.

---

## 7. Authority Levels

Not every gate has the same authority.

AykenOS should track gate authority explicitly.

| Authority Level | Meaning | Typical Use |
|---|---|---|
| `freeze_authoritative` | gate participates in strict freeze truth | current official closure or constitutional freeze |
| `closure_authoritative` | gate participates in dedicated phase closure workflow | phase closure candidate or official closure execution |
| `boundary_authoritative` | gate is authoritative for boundary preservation but not yet part of freeze | current Phase-13 boundary gates |
| `research_boundary` | gate exists to prove non-goals or boundary separation | replicated verification / research bridge |

This distinction matters because:

- not every useful gate belongs in `ci-freeze`
- not every boundary gate should block official closure
- not every research gate should appear as mainline authority

---

## 8. Current Phase-13 Mapping

The current boundary set can already be read as a layered model:

| Gate | Layer | Primary Risk Class | Current Authority |
|---|---|---|---|
| `ci-gate-proofd-observability-boundary` | service boundary | `control-plane-drift` | `boundary_authoritative` |
| `ci-gate-graph-non-authoritative-contract` | invariant + artifact | `truth-election-drift` | `boundary_authoritative` |
| `ci-gate-convergence-non-election-boundary` | artifact | `truth-election-drift` | `boundary_authoritative` |
| `ci-gate-verifier-reputation-prohibition` | invariant + artifact | `reputation-drift` | `boundary_authoritative` |
| `ci-gate-verification-determinism-contract` | determinism | `determinism-drift` | `boundary_authoritative` |
| `ci-gate-diagnostics-consumer-non-authoritative-contract` | consumer safety | `consumer-misuse-drift` | `boundary_authoritative` |
| `ci-gate-diagnostics-callsite-correlation` | consumer safety | `consumer-misuse-drift` | `boundary_authoritative` |
| `ci-gate-observability-routing-separation` | consumer safety | `topology-feedback-drift` | `boundary_authoritative` |
| `ci-gate-verification-diversity-floor` | collapse-horizon harness | `verification-gravity-drift` | `research_boundary` |
| `ci-gate-verifier-cartel-correlation` | collapse-horizon harness | `cartel-formation-drift` | `research_boundary` |
| `ci-gate-authority-sinkhole-absorption` | collapse-horizon harness | `authority-sinkhole-drift` | `research_boundary` |
| `ci-gate-proof-replay-admission-boundary` | invariant + boundary | `replay-boundary-drift` | `closure_authoritative` |

This is a coherent architecture.

The next problem is not missing gates.

The next problem is summary, registry, and deduplication.

The next threat horizon after that is collapse drift:

- verification gravity
- verifier cartel formation
- authority sinkhole absorption

Those are not first solved by more schema checks.

They require behavior-measuring harnesses over time.

The first collapse-horizon harnesses are now executable:

- `ci-gate-verification-diversity-floor`
  - invariant: `distributed verification must remain behaviorally diverse`
  - risk class: `verification-gravity-drift`
- `ci-gate-verifier-cartel-correlation`
  - invariant: `diversity != independence`
  - risk class: `cartel-formation-drift`
  - enforcement shape: dual-window diversity-floor analysis over Verification Diversity Ledger evidence
- `ci-gate-authority-sinkhole-absorption`
  - invariant: `verification reuse != authority basin collapse`
  - risk class: `authority-sinkhole-drift`
  - enforcement shape: VDL-backed authority-basin share, repeated-capture, alternate-path decay, and basin-slope analysis

The current routing-blindness boundary candidate is now executable:

- `ci-gate-observability-routing-separation`
  - invariant: `observability != scheduling`
  - risk class: `topology-feedback-drift`
  - enforcement shape: routing or scheduling harness over verification-facing Rust surfaces

---

## 9. Phase-13 Kill-Switch Profile

Phase-13 should preserve a small kill-switch profile above the full gate set.

The primary kill switches are:

1. `observability -> control plane`
2. `authority election`
3. `verification artifact integrity`
4. `verifier authority drift`

Those kill switches are mapped in detail in:

- `PHASE13_KILL_SWITCH_GATES.md`
- `GATE_REGISTRY.md`

The architectural rule is:

`few primary kill switches, many supporting checks`

The next horizon after kill-switch protection is documented in:

- `PHASE13_COLLAPSE_SCENARIOS.md`

---

## 10. Summary Reduction Rules

CI should reduce technical gate output using these rules:

1. show invariant summaries first
2. group failing gates by risk class
3. identify one primary explanatory gate per invariant
4. mark secondary failures as supporting evidence
5. avoid presenting duplicated semantic failures as independent root causes

Target CI language:

- `FAIL: observability != control`
- `FAIL: descriptive diagnostics != execution input`

not:

- `5 gates red, investigate manually`

---

## 11. Deduplication Rules

AykenOS should use the following rules before adding a new gate:

1. a new gate must declare its invariant class
2. a new gate must declare its risk class
3. a new gate must declare which protected surface is new
4. if the candidate gate does not introduce a new risk class or a new protected surface, it must not be added as a new gate
5. if the candidate gate targets the same risk class and the same protected surface, extend an existing gate instead
6. if overlap is mostly technical, factor shared logic into primitives instead of cloning validators
7. every gate must define its authoritative failure meaning in one sentence

Gate creation without those fields is architectural debt.

The default presumption is:

`new gate denied unless it adds new architectural coverage`

---

## 12. Negative-Test Growth Rule

Negative-test growth should not be managed by hand-written case explosion alone.

The preferred direction is:

- define forbidden field classes
- define forbidden action classes
- define forbidden query or method classes
- generate constrained cross-products where architectural meaning stays clear

So the intended long-term model is:

`negative matrix -> constrained generator`

This prevents:

- duplicated case writing
- drift between near-identical negative tests
- large matrices that stop mapping cleanly to invariants

Generated cases remain subordinate to invariant summaries.

They do not replace them.

---

## 13. Freeze and Closure Guidance

Before any Phase-13 boundary gate is promoted into strict freeze authority, AykenOS should first define:

- which invariants are phase-prep only
- which invariants are closure-authoritative
- which invariants become official freeze requirements

Until then, the current boundary set should remain:

- executable
- authoritative for local architecture preservation
- visible in the corpus
- outside the strict official freeze truth chain

This keeps Phase-13 discipline strong without confusing governance status.

---

## 14. Short Rule Set

The shortest correct AykenOS gate sentence is:

`primitive checks -> composite gates -> invariant summaries`

If gate growth stops preserving that structure, AykenOS has entered gate explosion.
