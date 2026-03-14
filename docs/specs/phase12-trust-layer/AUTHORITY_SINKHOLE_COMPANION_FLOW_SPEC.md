# Authority Sinkhole Companion Flow Specification

**Version:** 0.1  
**Status:** Implemented (Phase-13 Stage-2 companion producer contract)  
**Date:** 2026-03-14  
**Phase:** Phase-13 distributed verification observability  
**Type:** Normative companion-flow artifact specification

**Related Spec:** `AUTHORITY_SINKHOLE_ABSORPTION_GATE.md`, `CROSS_SURFACE_BASIN_ALIGNMENT_METRICS.md`, `VERIFICATION_DIVERSITY_LEDGER_SPEC.md`, `TRUST_REUSE_RUNTIME_SURFACE_SPEC.md`, `GATE_REGISTRY.md`

---

## 1. Purpose

This specification defines the Stage-2 companion observability surfaces for authority sinkhole analysis.

Its purpose is to:

- expose replay-boundary flow as a canonical observability artifact
- expose trust-reuse flow as a canonical observability artifact
- allow cross-surface basin alignment analysis
- keep replay and trust-reuse behavior descriptive rather than authoritative

The shortest rule is:

`verification basin analysis needs replay-boundary and trust-reuse companion flow evidence`

These artifacts are not routing surfaces.

They are not authority surfaces.

They are sinkhole-analysis companion surfaces only.

---

## 2. Companion Outputs

Stage-2 companion producers should export:

- `replay_boundary_flow_report.json`
- `trust_reuse_flow_report.json`

The sinkhole harness may then derive:

- `cross_surface_basin_alignment_report.json`

The Stage-2 metric model for that derived report is:

- `CROSS_SURFACE_BASIN_ALIGNMENT_METRICS.md`

Producer surfaces MUST NOT skip directly to alignment or basin verdicts.

They must first materialize canonical replay-boundary and trust-reuse flow events.

The current producer implementation defaults to the following source and output artifact names:

- `replay_boundary_flow_source.json`
- `trust_reuse_flow_source.json`
- `replay_boundary_flow_report.json`
- `trust_reuse_flow_report.json`

For trust-reuse specifically, a native runtime surface may be valid while still yielding no reusable Stage-2 events.

In that case the preferred source remains native, the source status becomes `NO_REUSABLE_EVENTS`, and the companion producer must materialize `trust_reuse_flow_report.json` with status `NOT_EVALUATED` rather than falling back to request-bound inference.

Current local status:

- `replay_boundary_flow_source.json` may be emitted from native bundle replay runtime evidence
- `trust_reuse_flow_source.json` currently remains request-bound fallback unless a native trust-reuse runtime surface exists

The native target contract for trust-reuse runtime evidence is:

- `TRUST_REUSE_RUNTIME_SURFACE_SPEC.md`

---

## 3. Shared Event Model

Both companion reports must use a shared minimal event model.

The minimum canonical event schema is:

```json
{
  "event_schema_version": 1,
  "event_id": "<content-addressed-id>",
  "flow_surface": "replay_boundary | trust_reuse",
  "run_id": "<run_id>",
  "timestamp_unix_ns": "<unix-ns>",
  "subject_bundle_id": "<bundle_id>",
  "verification_context_id": "<context_id>",
  "authority_chain_id": "<authority_chain_id>",
  "terminal": true,
  "reused": true
}
```

The minimum event set intentionally stays small.

Stage-2 only needs enough structure to answer:

`which practical basin is absorbing replay-boundary or trust-reuse flow for the same subjects over time?`

---

## 4. Canonical Field Roles

| Field | Meaning |
|---|---|
| `event_schema_version` | versioned event-schema handle for forward-compatible parsing |
| `event_id` | content-addressed identity for the companion flow event |
| `flow_surface` | whether the event belongs to replay-boundary or trust-reuse flow |
| `run_id` | local run handle that emitted the event |
| `timestamp_unix_ns` | event ordering for windowed analysis |
| `subject_bundle_id` | subject under evaluation |
| `verification_context_id` | explicit verification-context binding |
| `authority_chain_id` | practical authority basin attached to this event |
| `terminal` | whether the event is the terminal practical path for this surface |
| `reused` | whether the event represents reuse rather than a fresh isolated path |

These fields are the Stage-2 minimum.

Additional optional fields may appear, but the sinkhole gate must not require them for baseline operation.

---

## 5. Optional Extended Fields

Companion producers may additionally include:

- `verification_node_id`
- `verifier_id`
- `lineage_id`
- `execution_cluster_id`
- `source_run_id`
- `replay_contract_id`
- `trust_reuse_source`
- `reuse_group_id`
- `surface_local_path_id`

These remain descriptive only.

They must not be interpreted as:

- preferred basin
- trusted lineage
- routing priority
- admission authority

---

## 6. Report JSON Shape

Each companion report should use the following canonical outer shape:

```json
{
  "report_version": 1,
  "flow_surface": "replay_boundary",
  "status": "PASS",
  "run_id": "<run_id>",
  "window_model": "append_only_event_stream",
  "event_count": 2,
  "terminal_event_count": 2,
  "reused_event_count": 2,
  "events": [
    {
      "event_schema_version": 1,
      "event_id": "sha256:<digest>",
      "flow_surface": "replay_boundary",
      "run_id": "run-42",
      "timestamp_unix_ns": 1710000000000000000,
      "subject_bundle_id": "bundle-a",
      "verification_context_id": "context-a",
      "authority_chain_id": "chain-a",
      "terminal": true,
      "reused": true
    }
  ]
}
```

The `status` field here is producer-materialization status, not authority status.

Correct readings:

- `PASS` = companion flow was materialized successfully
- `FAIL` = producer failed closed

Incorrect reading:

- `PASS` does not mean the flow is healthy

That health judgment remains the sinkhole gate’s job.

---

## 7. Event Identity

Every companion event MUST carry a content-addressed `event_id`.

The recommended rule is:

1. canonicalize the event without `event_id`
2. hash canonical bytes with SHA-256
3. encode as `sha256:<digest>`

This provides:

- duplicate-event guard
- append determinism
- cross-surface forensic reproducibility

---

## 8. Producer Rules

Companion producers MUST:

1. derive events from existing canonical runtime or gate evidence
2. keep the reports append-only or regenerable in stable order
3. preserve explicit subject and verification-context binding
4. preserve explicit `authority_chain_id`
5. distinguish terminal and reused flow events
6. fail closed on malformed or unbound events

For trust-reuse specifically, companion producers SHOULD prefer:

1. native runtime trust-reuse evidence
2. explicit request-bound fallback

and MUST NOT invent trust-reuse events from receipt presence alone.

Companion producers MUST NOT:

- infer authority preference
- compress multiple subjects into one synthetic basin verdict
- emit routing hints
- collapse replay-boundary and trust-reuse surfaces into a single opaque score

---

## 9. Stable Ordering

The canonical sort order is:

1. `timestamp_unix_ns` ascending
2. `subject_bundle_id`
3. `verification_context_id`
4. `event_id`

This keeps cross-surface windowing reproducible.

---

## 10. Minimum Event Semantics

Stage-2 minimum semantics are:

- `terminal = true` means the event records the practical terminal path for that surface
- `reused = true` means the event represents operational reuse rather than isolated first-pass evaluation

The sinkhole gate only needs these minimal booleans to compute:

- replay-boundary basin capture
- trust-reuse basin capture
- cross-surface basin alignment
- cross-surface alternate-path decay

If a producer cannot determine `terminal` or `reused` exactly, it MUST fail closed rather than guess.

`event_schema_version` MUST be present on every event so that future companion-flow parsers can evolve without ambiguous back-compat guesses.

---

## 11. Cross-Surface Alignment Model

The sinkhole gate’s Stage-2 cross-surface layer is expected to compare:

- verification flow from VDL
- replay-boundary flow
- trust-reuse flow

on the shared keys:

- `subject_bundle_id`
- `verification_context_id`

The minimum Stage-2 question is:

`does the same authority basin keep absorbing verification, replay-boundary, and trust-reuse flow for the same subjects over time?`

This is stronger than Stage-1.

Stage-1 only asks:

`is verification flow collapsing into one basin?`

Future phases may add:

- `delegation_flow`

to model cases where:

`authority_chain_A -> authority_chain_B -> authority_chain_B`

creates sinkhole pressure without obvious verifier-count collapse.

---

## 12. Gate Integration Order

The correct Stage-2 integration order is:

1. verification run completes
2. VDL producer materializes verification-flow evidence
3. replay-boundary companion producer materializes replay-boundary flow
4. trust-reuse companion producer materializes trust-reuse flow
5. sinkhole gate consumes all available surfaces
6. sinkhole gate derives cross-surface basin alignment and decay metrics

The shortest pipeline is:

`verification run -> VDL -> replay/trust companion flows -> sinkhole Stage-2`

---

## 13. Non-Goals

These companion flow artifacts do not:

- grant replay admission
- grant trust reuse
- recommend authority chain selection
- rank verifiers
- suppress alternate paths

They are descriptive flow evidence only.

---

## 14. Short Rule

The shortest correct reading is:

`Stage-2 sinkhole analysis needs canonical replay-boundary and trust-reuse flow events, not opaque authority scores`
