# Verifier Cartel Correlation Gate

**Version:** 0.1  
**Status:** Implemented (Phase-13 Stage-1 collapse-horizon harness)  
**Date:** 2026-03-14  
**Phase:** Phase-13 distributed verification observability  
**Type:** Gate contract note  
**Target:** `ci-gate-verifier-cartel-correlation`  
**Related Spec:** `VERIFICATION_DIVERSITY_LEDGER_SPEC.md`, `VERIFICATION_DIVERSITY_FLOOR_GATE.md`, `PHASE13_COLLAPSE_SCENARIOS.md`, `VERIFICATION_INVARIANTS.md`, `GATE_REGISTRY.md`

---

## 1. Purpose

This gate detects cartel-style verifier correlation even when nominal diversity remains above the current floor.

Its target failure mode is:

`entropy illusion`

That means the system still appears behaviorally diverse by simple count or entropy metrics, while practical verifier independence has already collapsed.

The shortest correct reading is:

`high diversity counts do not prove verifier independence`

---

## 2. Protected Risk

Primary risk class:

- `cartel-formation-drift`

Protected failure meaning:

- formally distinct verifiers have become correlated enough to behave like one practical trust bloc

This is a behavioral correlation harness, not a schema gate.

---

## 3. Entropy Illusion

The core illusion is:

- many `verifier_id` values
- acceptable entropy
- acceptable dominance ratio
- but low practical independence

Typical causes:

- same `lineage_id`
- same `authority_chain_id`
- same `execution_cluster_id`
- repeated pairwise verdict correlation
- shared operator or deployment cadence where available

So the dangerous system shape is:

`nominal diversity = high`

but:

`effective independence = low`

---

## 4. Why Diversity Floor Can Still Pass

The diversity floor gate may still pass because it mainly answers:

- how many verifiers appeared?
- how concentrated is the largest verifier share?
- how diverse is the lineage distribution?

It does not yet fully answer:

- are those verifiers independent?
- are they repeatedly moving as one bloc?
- does one lineage keep producing many nominally separate identities?

So diversity floor is the first horizon.

Cartel correlation is the next one.

---

## 5. Required Inputs

The expected inputs are:

- Verification Diversity Ledger slices
- lineage distribution
- authority-chain distribution
- execution-cluster distribution where present
- optional authority-topology companion artifacts

Recommended evidence set:

- `vdl_window.json`
- `diversity_metrics.json`
- `lineage_distribution.json`
- `cluster_distribution.json`
- `dominance_analysis.json`
- `entropy_report.json`
- authority-topology companion artifact where available

These remain observability artifacts only.

They MUST NOT become routing or authority inputs.

---

## 6. Metric Evolution Strategy

The cartel correlation gate must evolve incrementally.

The first implementation should prioritize:

- interpretability
- operational stability
- low false positive rate

More advanced statistical signals may be added later once baseline observability stabilizes.

The shortest rule is:

`start with explainable correlation metrics, then add structural bloc detection, then only later add advanced statistical signals`

---

## 7. V0 Metrics (Initial Gate)

The initial implementation should detect clear verifier-bloc behavior while remaining easy to explain in CI.

### 7.1 Pairwise Verdict Correlation

This metric measures agreement between verifier verdict sequences inside a bounded window.

Example:

- `corr(verifier_a, verifier_b) = 0.99`

High sustained values indicate possible cartel behavior.

Suggested early threshold:

- `pairwise_verdict_correlation > 0.98`

### 7.2 Lineage-Conditioned Pairwise Correlation

This is the same correlation metric evaluated under the condition:

- same `lineage_id`

It exists to detect many nominal verifiers produced by one lineage moving identically.

The characteristic signal is:

- multiple `verifier_id`
- same `lineage_id`
- correlation above threshold

### 7.3 Authority-Chain Conditioned Correlation

This applies the same logic to:

- `authority_chain_id`

It detects trust-root concentration masked by multiple verifier identities.

### 7.4 Execution-Cluster Overlap Ratio

This measures how much of the verifier population originates from one execution environment.

Example:

- one `execution_cluster_id` supplies most observed verifiers

This captures infrastructure-level cartel formation.

### 7.5 Correlation Stability Across Windows

One high-correlation window may be noise.

Cartel behavior usually persists across windows.

Example:

- `window_1 = 0.96`
- `window_2 = 0.97`
- `window_3 = 0.98`

Persistent high correlation indicates bloc coordination rather than transient coincidence.

The operational goal of V0 is:

`detect correlated verifier motion despite acceptable nominal diversity`

---

## 8. V1 Metrics (Advanced Correlation Layer)

The next layer should detect cartel behavior that evades simple pairwise checks.

These metrics should be added only after the V0 gate stabilizes operationally.

### 8.1 Triadic Verifier Correlation

This detects coordination among groups of three verifiers.

It matters because pairwise signals can remain only moderate while three verifiers still move as one bloc together.

### 8.2 Multi-Lineage Bloc Formation

This detects coordinated behavior across distinct lineages.

It exists to prevent cartel evasion through lineage diversification.

### 8.3 Correlation Network Density

This constructs a verifier-correlation graph:

- nodes = verifiers
- edges = high-correlation relationships

The resulting density measures whether many verifiers are collapsing into one coordinated cluster.

### 8.4 Dominance Slope

This measures whether verifier-share concentration is rising across successive windows even before the system crosses a hard dominance threshold.

Representative pattern:

- `share_window_1 = 0.18`
- `share_window_2 = 0.24`
- `share_window_3 = 0.31`

This matters because slow cartel emergence may remain below the diversity-floor threshold for a long time while still becoming progressively harder to reverse.

---

## 9. V2 Metrics (Optional Statistical Layer)

These metrics may improve detection power, but they are less interpretable.

They should remain optional unless strong operational value appears.

### 9.1 Mutual Information

Mutual information measures predictive dependence between verifier outputs even when linear correlation is weak.

### 9.2 Information Flow Analysis

This tries to detect directional influence between verifier outputs over time.

It may expose hidden coordination or shared upstream inputs, but it should be treated as an advanced research metric rather than an early CI default.

---

## 10. Example Detection Shapes

Representative suspicious cases:

1. two or more distinct `verifier_id` values show near-identical verdict behavior over a bounded window
2. many verifier identities collapse into one `lineage_id`
3. many verifier identities collapse into one `authority_chain_id`
4. many verifier identities come from one `execution_cluster_id`
5. the same lineage repeatedly supplies the dominant verifier subset across windows

Typical example:

- `pairwise_verdict_correlation > 0.98`
- same `lineage_id`
- repeated over bounded windows

This should raise cartel suspicion even if:

- `unique_verifier_count >= floor`
- `dominance_ratio <= max`

---

## 11. CI Implementation Order

The recommended rollout order is:

### Stage 1

- pairwise verdict correlation
- lineage-conditioned correlation
- authority-chain-conditioned correlation
- execution-cluster overlap
- correlation stability across windows

### Stage 2

- triadic verifier correlation
- correlation graph density
- multi-lineage bloc formation
- dominance slope

### Stage 3

- mutual information
- information flow analysis

The shortest operational rule is:

`few clear metrics first, advanced statistics only after the baseline becomes stable`

---

## 12. Expected Outputs

The current Stage-1 gate exports:

- `report.json`
- `verifier_cartel_correlation_report.json`
- `cartel_correlation_metrics.json`
- `pairwise_correlation_report.json`
- `lineage_correlation_report.json`
- `authority_chain_correlation_report.json`
- `cluster_overlap_report.json`
- `correlation_stability_report.json`
- `violations.txt`

`report.json` remains the CI verdict surface.

The others are behavioral forensic evidence.

---

## 13. Non-Goals

This gate does not:

- elect authority
- rank verifiers by trust
- recommend preferred verifiers
- produce routing hints
- replace diversity-floor checks

It only answers:

`does nominal diversity conceal practical verifier correlation?`

---

## 14. Short System Model

The larger verification-health sequence is:

`diversity floor -> cartel correlation -> gravity or basin formation`

Those correspond to three distinct failure horizons:

- distribution health
- independence health
- temporal drift

---

## 15. Short Rule

The shortest correct reading is:

`diversity is necessary, but independence must also remain measurable`
