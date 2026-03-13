# `ci-gate-convergence-non-election-boundary`

This gate freezes the `Phase-13` convergence surface as:

- artifact-backed
- descriptive
- non-elective
- non-authoritative

It exists to enforce:

`convergence explains divergence state; convergence does not elect truth`

## Scope

The gate validates the convergence-specific diagnostics artifacts that are most likely to drift from observability into hidden selection semantics:

- `parity_convergence_report.json`
- `parity_drift_attribution_report.json`

It does not replace:

- `ci-gate-graph-non-authoritative-contract`
- `ci-gate-proofd-observability-boundary`

This gate is narrower. It hardens the producer contract for convergence partitions, clusters, ratios, and island diagnostics.

## Contract

The convergence layer MAY expose:

- partition counts
- cluster counts
- partition sizes
- cluster sizes
- descriptive ratios
- explicit derivation metadata
- diagnostic `global_status`
- historical-only and insufficient-evidence islands

The convergence layer MUST NOT expose:

- winning cluster selection
- preferred partition selection
- admission or replay routing hints
- execution priority or weighting
- truth finality inferred from convergence
- silent collapse of historical or insufficient-evidence islands

## Semantic Contract Checks

The gate enforces both field-level and value-level contracts:

- `global_status` must remain inside the descriptive parity status enum
- `cluster_derivation` must remain `node_parity_outcome_dk_partitions`
- `edge_match_cluster_derivation` must remain `pairwise_match_graph_connected_components`

This matters because convergence drift can hide inside value semantics even when field names still look harmless.

## Negative Matrix Coverage

This gate primarily hardens:

- `P13-NEG-07`
  - largest partition or cluster metadata must remain descriptive only
- `P13-NEG-08`
  - convergence must not imply admission, execution, or truth finality
- `P13-NEG-09`
  - convergence artifacts must not resolve a winning verdict or cluster
- `P13-NEG-10`
  - historical and insufficient-evidence islands must remain explicit diagnostics

## Forbidden Fields

The gate fails closed if convergence artifacts expose fields such as:

- `winning_cluster`
- `selected_partition`
- `preferred_cluster`
- `cluster_policy_input`
- `partition_replay_admission`
- `verification_weight`
- `execution_route`
- `committed_cluster`

It also rejects pattern-based drift such as:

- cluster or partition selection fields
- ratio or size metrics repurposed as routing or policy inputs
- convergence finality or authority fields
- island-collapse metadata

## Outputs

The gate writes:

- `convergence_non_election_report.json`
- `report.json`
- `violations.txt`
- `meta.txt`

## Execution

Local:

```bash
make ci-gate-convergence-non-election-boundary
```

Fixture mode:

```bash
bash scripts/ci/gate_convergence_non_election_boundary.sh \
  --evidence-dir /tmp/convergence-gate \
  --artifact-root /tmp/parity-artifacts
```

## Failure Meaning

If this gate fails, convergence diagnostics have started to behave like a selection surface.

That means Phase-13 is drifting from:

`diagnostics about partitions`

toward:

`policy or election decisions from partitions`
