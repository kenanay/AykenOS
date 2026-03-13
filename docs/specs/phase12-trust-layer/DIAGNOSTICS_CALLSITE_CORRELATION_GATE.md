# `ci-gate-diagnostics-callsite-correlation`

This gate hardens the second consumer-side layer of `Phase-13`.

It exists to enforce:

`descriptive diagnostics sources MUST NOT correlate with policy, replay, routing, or override sinks`

## Scope

This is a source-contract gate over the approved diagnostics producer and passthrough surfaces:

- `ayken-core/crates/proof-verifier/examples/phase12_gate_harness.rs`
- `ayken-core/crates/proof-verifier/src/authority/authority_drift_topology.rs`
- `ayken-core/crates/proof-verifier/src/authority/drift_attribution.rs`
- `userspace/proofd/src/lib.rs`
- `userspace/proofd/examples/proofd_gate_harness.rs`

It complements `ci-gate-diagnostics-consumer-non-authoritative-contract`.

The previous gate blocks unapproved consumers from referencing diagnostics fields at all.

This gate goes one level deeper:

- it watches approved diagnostics files
- it tracks aliasing from protected diagnostics tokens
- it fails if those aliases reach decision sinks

## Protected Sources

The gate treats fields and artifact names such as these as descriptive-only sources:

- `global_status`
- `dominant_authority_chain_id`
- `largest_outcome_cluster_size`
- `outcome_convergence_ratio`
- `historical_authority_islands`
- `insufficient_evidence_islands`
- `cluster_derivation`
- `edge_match_cluster_derivation`
- `parity_convergence_report.json`
- `parity_authority_drift_topology.json`
- `parity_drift_attribution_report.json`

## Protected Sinks

The gate fails if those sources flow into call sites associated with:

- policy evaluation
- verification execution
- replay admission
- execution admission
- routing hints
- priority or override channels
- action or promotion channels

## Contract

The gate does not fail on simple co-occurrence.

It fails when a diagnostics source:

- appears on the same sink line
- is assigned to an alias
- is renamed through intermediate aliases
- and that alias later reaches a protected sink

This is the first repo-level guard against:

`descriptive diagnostics -> renamed local state -> decision sink`

## Negative Matrix Coverage

This gate primarily hardens:

- `P13-CORR-01`
  - direct diagnostics-to-policy or verification correlation
- `P13-CORR-02`
  - renamed diagnostics state flowing into replay or routing sinks
- `P13-CORR-03`
  - diagnostics artifact aliases flowing into override or priority sinks

## Outputs

The gate writes:

- `diagnostics_callsite_correlation_report.json`
- `report.json`
- `violations.txt`
- `meta.txt`

## Execution

Local:

```bash
make ci-gate-diagnostics-callsite-correlation
```

Fixture mode:

```bash
bash scripts/ci/gate_diagnostics_callsite_correlation.sh \
  --evidence-dir /tmp/diagnostics-callsite-correlation \
  --source-root /tmp/fixture-root \
  --source-path approved/flow.rs
```

## Failure Meaning

If this gate fails, an approved diagnostics surface has started to use descriptive metadata as decision input.

That means the system is drifting from:

`diagnostics producer or passthrough`

toward:

`hidden consumer of diagnostics semantics`
