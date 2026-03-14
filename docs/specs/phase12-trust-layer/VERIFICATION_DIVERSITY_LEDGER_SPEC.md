# Verification Diversity Ledger

**Version:** 0.1  
**Status:** Draft (Phase-13 diversity observability artifact)  
**Date:** 2026-03-14  
**Phase:** Phase-13 distributed verification observability  
**Type:** Normative artifact specification

**Related Spec:** `VERIFICATION_DIVERSITY_LEDGER_PRODUCER_SPEC.md`

---

## 1. Artifact Purpose

The Verification Diversity Ledger (VDL) is an append-only artifact for observing distributed verification behavior across multiple runs.

Its purpose is to:

- measure how verification behavior is distributed over time
- observe verifier diversity
- detect authority-basin formation early
- support cartel-correlation analysis
- provide stable inputs to future diversity harnesses

The ledger is not an authority surface.

It is only:

`behavioral observability surface`

It MUST NOT be used for:

- verifier ranking
- verifier trust scoring
- routing preference
- authority election

---

## 2. Canonical Fields

Each ledger entry represents one verification event.

The minimum canonical schema is:

```json
{
  "ledger_version": 1,
  "entry_id": "<content-addressed-id>",
  "run_id": "<verification-run-id>",
  "timestamp_unix_ns": "<unix-ns>",
  "subject_bundle_id": "<bundle_id>",
  "verification_context_id": "<context_id>",
  "verification_node_id": "<node_identity>",
  "verifier_id": "<verifier_identity>",
  "authority_chain_id": "<authority_chain_id>",
  "lineage_id": "<verifier_lineage_id>",
  "execution_cluster_id": "<optional_cluster_identity>",
  "verdict": "PASS | FAIL | INSUFFICIENT_EVIDENCE",
  "receipt_hash": "<receipt_hash>"
}
```

### 2.1 Field Roles

| Field | Meaning |
|---|---|
| `run_id` | verification execution instance |
| `timestamp_unix_ns` | event ordering for diversity windows |
| `subject_bundle_id` | verified artifact |
| `verification_context_id` | verification policy context |
| `verification_node_id` | physical or execution-origin node identity |
| `verifier_id` | concrete verifier instance |
| `authority_chain_id` | trust lineage chain |
| `lineage_id` | verifier family or registry lineage |
| `execution_cluster_id` | optional deployment, cluster, or region grouping hint |
| `verdict` | deterministic verification result |
| `receipt_hash` | receipt binding |

`verification_node_id` and `verifier_id` MUST remain distinct.

The first names the concrete execution origin.

The second names the verifier identity surface.

`execution_cluster_id` is optional.

If present, it remains descriptive only and MUST NOT become routing, authority, or preference input.

---

## 3. Append Rules

The VDL is an append-only artifact.

Rules:

1. existing entries MUST NOT be modified
2. new entries are appended only when new verification runs occur
3. canonical ordering MAY be:
   - `timestamp_unix_ns` ascending
   - content-addressed ordering
4. ledger snapshots MAY be produced:
   - `ledger_snapshot_hash`
   - snapshot artifacts are derived outputs, not canonical entries

---

## 4. Update Rules

The ledger is not updated in place.

Only new events are appended.

Allowed operations:

- `append_entry`
- `create_snapshot`
- `query_window`

Forbidden operations:

- entry deletion
- entry mutation
- verifier metadata rewrite

---

## 5. Subject / Context Binding

Every ledger entry is bound to:

- `subject_bundle_id`
- `verification_context_id`

The ledger is not itself a proof surface.

It is a behavioral verification trace.

So the correct reading is:

`ledger entry = verification event witness`

not:

`ledger entry = proof of truth`

---

## 6. Forbidden Semantics

The VDL MUST NOT be used for the following purposes.

### 6.1 Routing Input

Ledger data MUST NOT be used for:

- verifier selection
- verification scheduling
- preferred-verifier routing
- fallback suppression

### 6.2 Authority Ranking

Ledger data MUST NOT become:

- verifier score
- verifier reliability ranking
- trust score
- dominant verifier list

### 6.3 Implicit Reputation System

The following are forbidden:

- success-rate ranking
- agreement ranking
- failure-rate scoring
- historical reliability scoring

These prohibitions preserve:

- `verification history != verifier reputation`
- `observability != scheduling`

The shortest rule is:

`VDL = diversity observability, not reputation or routing input`

---

## 7. Diversity Metrics

The VDL MAY support descriptive metrics.

Allowed metric classes include:

- unique verifier count
- unique verification-node count
- unique authority-chain count
- unique lineage count
- dominance ratio
- entropy score
- diversity index
- pairwise verdict correlation

Examples:

- `dominance_ratio = max(verifier_share)`
- `lineage_entropy = shannon_entropy(lineage_distribution)`
- `pairwise_verdict_correlation(verifier_a, verifier_b)`

These metrics remain:

`observability artifacts`

They MUST NOT become direct policy or routing outputs.

---

## 8. Window Model

Diversity analysis SHOULD be window-based.

Example windows:

- last `N` verification runs
- last `T` time window
- subject-scoped window
- context-scoped window
- dual window

Examples:

- `window_size = 100 runs`
- `window_time = 24h`
- `window_size = 200 runs` and `window_time = 24h`

Windowing exists so that diversity can be evaluated behaviorally instead of by isolated events.

Dual-window evaluation is preferred for Phase-13 because it can expose:

- short burst concentration
- long horizon concentration

---

## 9. Threshold Policy Separation

The VDL itself MUST NOT encode thresholds.

It only produces data.

Thresholds belong in a separate policy surface.

Example:

- `diversity_policy_v1`

Possible policy sentences:

- `min_unique_verifiers >= 3`
- `max_dominance_ratio <= 0.40`
- `min_lineage_entropy >= 1.2`

This separation preserves:

`artifact != policy`

---

## 10. Future Gate Binding

The VDL is intended to feed future collapse-horizon harnesses.

### 10.1 Diversity Floor Gate

Reserved gate:

- `ci-gate-verification-diversity-floor`

Intended check:

- verifier diversity remains above a declared threshold

Primary inputs:

- VDL window metrics

### 10.2 Cartel Correlation Gate

Reserved gate:

- `ci-gate-verifier-cartel-correlation`

Intended check:

- lineage correlation
- authority-chain concentration
- verdict-correlation patterns

Primary inputs:

- VDL
- authority topology

### 10.3 Authority Sinkhole Gate

Reserved gate:

- `ci-gate-authority-sinkhole-absorption`

Intended check:

- authority-basin absorption detection

Primary inputs:

- VDL
- authority-chain distribution

---

## 11. Relationship to Existing Artifacts

The VDL is related to:

- `PROOFD_DIAGNOSTICS_SERVICE_SURFACE.md`
- `VERIFICATION_OBSERVABILITY_MODEL.md`
- `GLOBAL_VERIFICATION_GRAPH_MODEL.md`
- `AUTHORITY_TOPOLOGY_FORMAL_MODEL.md`

It is distinct from:

| Artifact | Role |
|---|---|
| receipt | proof artifact |
| parity report | cross-node diagnostic |
| incident graph | topology diagnostic |
| verification audit ledger | verifier-local append-only audit evidence |
| VDL | multi-run diversity behavior trace |

The VDL is not a global consensus log.

It is a derived observability ledger for behavioral concentration analysis.

---

## 12. Short Rule

The shortest correct reading is:

`distributed verification correctness requires behavioral diversity observability`

So the VDL exists to:

`detect concentration before consensus-style failure appears`
