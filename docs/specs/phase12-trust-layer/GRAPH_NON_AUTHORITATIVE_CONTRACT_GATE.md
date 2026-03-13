# `ci-gate-graph-non-authoritative-contract`

This gate freezes the Phase-13 graph and topology surfaces as:

- structural
- descriptive
- non-authoritative
- non-inferential

It exists to enforce:

`graph explains verification state; graph does not determine truth`

## Scope

The gate validates the derived diagnostics artifacts that are most likely to drift into hidden truth inference:

- `parity_convergence_report.json`
- `parity_authority_drift_topology.json`
- `parity_incident_graph.json`
- `parity_consistency_report.json`

## Contract

The graph/topology layer MAY expose:

- structure
- topology
- partitions
- clusters
- incident relationships
- descriptive dominance metadata

The graph/topology layer MUST NOT expose:

- truth selection
- winner election
- authority arbitration
- consensus strength
- statistical truth estimation

## Allowed Descriptive Fields

The following fields are explicitly descriptive-only and remain allowed:

- `dominant_authority_chain_id`
- `dominant_authority_cluster_key`
- `surface_consistency_ratio`
- `outcome_convergence_ratio`

Those fields are not truth signals.

They are only graph/topology diagnostics.

## Forbidden Fields

The gate fails closed if graph/topology payloads expose fields such as:

- `selected_truth`
- `winning_verdict`
- `cluster_truth`
- `statistical_truth`
- `truth_estimate`
- `selected_authority`
- `authority_winner`
- `consensus_strength`
- `cluster_consensus_strength`
- `majority_accept`

## Negative Matrix Coverage

This gate primarily hardens:

- `P13-NEG-05`
- `P13-NEG-06`
- `P13-NEG-08`
- `P13-NEG-09`

## Outputs

The gate writes:

- `graph_non_authoritative_report.json`
- `report.json`
- `violations.txt`
- `meta.txt`

## Execution

Local:

```bash
make ci-gate-graph-non-authoritative-contract
```

Fixture mode:

```bash
bash scripts/ci/gate_graph_non_authoritative_contract.sh \
  --evidence-dir /tmp/graph-gate \
  --artifact-root /tmp/parity-artifacts
```

## Failure Meaning

If this gate fails, the graph layer has started to emit truth-inference or authority-selection signals.

That means Phase-13 has drifted from observability into hidden consensus.
