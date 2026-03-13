# AykenOS Gate Registry

**Version:** 1.0  
**Status:** Draft (Phase-13 architecture preparation)  
**Date:** 2026-03-13  
**Type:** Operational registry

---

## 1. Purpose

This document is the registry surface for AykenOS gates.

It exists so that gate growth remains:

- risk-centered
- phase-readable
- de-duplicable
- summary-friendly

This registry is intentionally partial.

It currently prioritizes:

- Phase-13 boundary gates
- their immediate Phase-12 supporting gates

It should be expanded before any large freeze-chain promotion effort.

---

## 2. Registry Schema

Every gate should eventually declare at least the following fields:

| Field | Meaning |
|---|---|
| `gate_id` | repo target name |
| `layer` | five-layer gate architecture layer |
| `invariant_class` | architectural sentence being protected |
| `risk_class` | normalized drift class |
| `surface_type` | service, artifact, consumer, verifier-core, governance |
| `enforcement_type` | validator, harness, source-scan, source-correlation, namespace-boundary |
| `drift_detection_strategy` | how the gate detects the protected drift |
| `composition` | primitive or composite |
| `authority_level` | freeze, closure, boundary, research |
| `primary_inputs` | artifacts, source paths, or service surfaces scanned |
| `authoritative_failure_meaning` | one-sentence meaning of a red gate |

Gate admission rule:

`same risk class + same protected surface -> extend existing gate`

The default registry stance is:

`new gate denied unless it adds new architectural coverage`

---

## 3. Risk Class Registry

| Risk Class | Meaning |
|---|---|
| `truth-election-drift` | diagnostics drift into winner selection or truth inference |
| `artifact-truth-drift` | runtime, cache, or network state starts substituting for canonical evidence artifacts |
| `authority-drift` | descriptive surfaces drift into trust authority |
| `control-plane-drift` | observability or diagnostics drift into action surface |
| `consumer-misuse-drift` | descriptive outputs reused as decision input |
| `topology-feedback-drift` | observability artifacts bias routing, preferred-node selection, or verification scheduling |
| `reputation-drift` | verifier history becomes scoring or trust ranking |
| `determinism-drift` | verification depends on ambient environment |
| `replay-boundary-drift` | verification success drifts into replay admission |
| `verification-gravity-drift` | verification behavior quietly concentrates around a small verifier subset without explicit authority election |
| `cartel-formation-drift` | formally separate verifiers become operationally correlated enough to behave like one trust bloc |
| `authority-sinkhole-drift` | operational reuse or trust reuse absorbs flows into one practical authority basin |

---

## 4. Current Registered Gates

| gate_id | layer | invariant_class | risk_class | surface_type | enforcement_type | drift_detection_strategy | composition | authority_level | primary_inputs | authoritative_failure_meaning |
|---|---|---|---|---|---|---|---|---|---|---|
| `ci-gate-proof-bundle` | artifact + portability boundary | `artifacts = canonical interface` | `artifact-truth-drift` | artifact | harness | bundle portability and checksum integrity harness | composite | `closure_authoritative` | proof bundle, checksums, portable proof package | canonical proof artifacts are no longer portable or integrity-bound |
| `ci-gate-proof-verdict-binding` | invariant + artifact | `artifacts = canonical interface` | `artifact-truth-drift` | artifact + verifier boundary | harness | artifact-bound verdict-subject validation | composite | `closure_authoritative` | verdict subject tuple, repeated verification evidence, verdict binding report | verification verdict semantics are no longer artifact-bound |
| `ci-gate-cross-node-parity` | artifact | `parity = diagnostics` | `truth-election-drift` | artifact | harness | parity artifact harness | composite | `closure_authoritative` | parity matrix, convergence, drift, topology artifacts | distributed parity evidence no longer explains disagreement deterministically |
| `ci-gate-proofd-service` | service boundary | `proofd = verification and diagnostics service` | `control-plane-drift` | service | harness | service contract harness | composite | `closure_authoritative` | `proofd` service contract, request/response, receipt evidence | `proofd` service contract is no longer stable or deterministic |
| `ci-gate-proofd-observability-boundary` | service boundary | `observability != control` | `control-plane-drift` | service | namespace-boundary | namespace and payload boundary validation | composite | `boundary_authoritative` | `/diagnostics/*` namespace, methods, payloads | read-only diagnostics surface has drifted into action or mutation behavior |
| `ci-gate-graph-non-authoritative-contract` | invariant + artifact | `graph != truth inference` | `truth-election-drift` | artifact | validator | schema and payload semantic validation | composite | `boundary_authoritative` | graph, topology, convergence artifacts | graph or topology artifacts started carrying truth or authority selection semantics |
| `ci-gate-convergence-non-election-boundary` | artifact | `convergence != election` | `truth-election-drift` | artifact | validator | field and value semantic validation | composite | `boundary_authoritative` | convergence and drift artifacts | convergence diagnostics started carrying cluster-selection or finality semantics |
| `ci-gate-diagnostics-consumer-non-authoritative-contract` | consumer safety | `descriptive diagnostics != execution input` | `consumer-misuse-drift` | consumer | source-scan | static protected-token scan | composite | `boundary_authoritative` | runtime Rust sources outside approved producer/passthrough surfaces | descriptive diagnostics are being imported into runtime consumers |
| `ci-gate-diagnostics-callsite-correlation` | consumer safety | `descriptive diagnostics != decision flow` | `consumer-misuse-drift` | consumer | source-correlation | function-local source-to-sink correlation | composite | `boundary_authoritative` | approved diagnostics producer/passthrough Rust sources | diagnostics aliases are flowing into policy, replay, routing, or override sinks |
| `ci-gate-observability-routing-separation` | consumer safety | `observability != scheduling` | `topology-feedback-drift` | consumer + routing boundary | harness + source-correlation | routing-blindness contract scan over verification-facing routing contexts | composite | `boundary_authoritative` | proof-verifier and proofd Rust sources with routing or verifier-selection contexts | observability artifacts have started influencing verification routing or scheduling behavior |
| `ci-gate-verification-diversity-floor` | collapse-horizon harness | `distributed verification must remain behaviorally diverse` | `verification-gravity-drift` | behavioral ledger | harness | dual-window diversity, dominance, and entropy analysis over VDL evidence | composite | `research_boundary` | verification diversity ledger, diversity policy, dual-window VDL slice | verification activity has concentrated below an acceptable verifier-diversity floor |
| `ci-gate-verifier-cartel-correlation` | collapse-horizon harness | `diversity != independence` | `cartel-formation-drift` | behavioral ledger | harness | pairwise and conditioned verifier-correlation analysis over bounded VDL windows | composite | `research_boundary` | verification diversity ledger, cartel correlation policy, bounded correlation windows | nominal verifier diversity is concealing correlated verifier-bloc behavior |
| `ci-gate-verifier-reputation-prohibition` | invariant + artifact | `verification history != verifier reputation` | `reputation-drift` | artifact | validator | schema and pattern validation | composite | `boundary_authoritative` | graph, convergence, topology, incident artifacts | verification history has become scoring or implicit authority |
| `ci-gate-verification-determinism-contract` | determinism | `verification != environment dependent` | `determinism-drift` | verifier-core | source-scan | verifier purity source scan | composite | `boundary_authoritative` | verifier-critical Rust source set | verifier semantics now depend on ambient environment state |
| `ci-gate-proof-replay-admission-boundary` | invariant + boundary | `verified proof != replay admission` | `replay-boundary-drift` | artifact + policy boundary | harness | replay-boundary contract harness | composite | `closure_authoritative` | proof subject, receipt, replay boundary report | proof verification has started to imply replay admission authority |
| `ci-gate-proof-replicated-verification-boundary` | invariant + research boundary | `replicated verification != current closure authority` | `replay-boundary-drift` | governance + research boundary | harness | research-boundary harness | composite | `research_boundary` | bridge report, research note | replicated verification semantics have leaked into current closure authority |

## 5. Invariant Summary Mapping

CI should reduce the above gates into invariant summaries like this:

| Invariant Summary | Supporting Gates |
|---|---|
| `observability != control` | `ci-gate-proofd-observability-boundary`, `ci-gate-diagnostics-consumer-non-authoritative-contract`, `ci-gate-diagnostics-callsite-correlation` |
| `graph != truth inference` | `ci-gate-graph-non-authoritative-contract` |
| `convergence != election` | `ci-gate-convergence-non-election-boundary` |
| `verification history != verifier reputation` | `ci-gate-verifier-reputation-prohibition` |
| `descriptive diagnostics != execution input` | `ci-gate-diagnostics-consumer-non-authoritative-contract`, `ci-gate-diagnostics-callsite-correlation` |
| `observability != scheduling` | `ci-gate-observability-routing-separation`; supported in part by `ci-gate-diagnostics-consumer-non-authoritative-contract` and `ci-gate-diagnostics-callsite-correlation` |
| `diversity != independence` | `ci-gate-verifier-cartel-correlation`; supported by `ci-gate-verification-diversity-floor` |
| `verification != environment dependent` | `ci-gate-verification-determinism-contract` |
| `verified proof != replay admission` | `ci-gate-proof-replay-admission-boundary` |

This reduction layer should become the primary CI reading surface.

## 5A. Phase-13 Kill-Switch Profile

| Kill Switch | Primary Gate | Supporting Gates | Primary Meaning |
|---|---|---|---|
| `observability -> control plane` | `ci-gate-observability-routing-separation` | `ci-gate-proofd-observability-boundary`, `ci-gate-diagnostics-consumer-non-authoritative-contract`, `ci-gate-diagnostics-callsite-correlation` | observability has started steering verification behavior |
| `authority election` | `ci-gate-convergence-non-election-boundary` | `ci-gate-graph-non-authoritative-contract`, `ci-gate-cross-node-parity` | distributed agreement shape is being treated as truth selection |
| `verification artifact integrity` | `ci-gate-proof-verdict-binding` | `ci-gate-proof-bundle`, `ci-gate-proof-receipt`, `ci-gate-proofd-service` | verification truth is no longer artifact-bound |
| `verifier authority drift` | `ci-gate-verifier-authority-resolution` | `ci-gate-verifier-reputation-prohibition`, `ci-gate-observability-routing-separation`, `ci-gate-cross-node-parity` | valid receipt semantics are being confused with trusted verifier authority |

## 5B. Reserved Collapse-Horizon Harnesses

The following future harnesses are not current authority surfaces.

They exist as reserved responses to Phase-13 collapse scenarios that can emerge even when current gates pass.

| Reserved Gate | Risk Class | Intended Meaning |
|---|---|---|
| `ci-gate-authority-sinkhole-absorption` | `authority-sinkhole-drift` | repeated verification or replay-boundary flows are being absorbed into one practical authority basin |

Those future harnesses are expected to consume:

- `VERIFICATION_DIVERSITY_LEDGER_SPEC.md`
- `VERIFICATION_DIVERSITY_FLOOR_GATE.md`
- `AUTHORITY_SINKHOLE_ABSORPTION_GATE.md`
- authority-topology and lineage-distribution artifacts

---

## 6. Primitive Backlog

The current gate set already suggests a shared primitive backlog:

- `forbidden_key_scan`
- `forbidden_pattern_scan`
- `allowed_enum_validator`
- `allowed_derivation_validator`
- `namespace_method_boundary_check`
- `diagnostics_consumer_scan`
- `diagnostics_source_sink_correlation`
- `artifact_passthrough_integrity_check`
- `routing_blindness_harness`

Current repo targets are mostly composite gates.

Primitive extraction should happen only where:

- overlap is repeated
- failure meaning remains clear
- the primitive can be reused without hiding architectural intent

Negative-test growth should also normalize around generators where possible:

- field-class generators
- action-class generators
- query or method-class generators

The preferred long-term direction is:

`negative matrix -> constrained generator`

That keeps case growth aligned with risk classes instead of ad hoc duplication.

---

## 7. Freeze Guidance

Current status:

- the strict `ci-freeze` chain remains the authoritative freeze truth for already closed phases
- the current Phase-13 boundary gates are executable and authoritative for architecture preservation
- those boundary gates are not yet promoted into strict official freeze truth

Promotion into freeze authority should require:

1. explicit governance decision
2. registry update
3. invariant summary mapping update
4. dedup check against existing closure-authoritative gates

Without those four steps, adding a gate directly to `ci-freeze` is architectural drift.

---

## 8. Short Rule

The shortest correct registry sentence is:

`many technical checks, few architectural judgments`

If the registry stops making that relationship obvious, gate explosion has started.
