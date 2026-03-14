# `ci-gate-proofd-observability-boundary`

This gate freezes the `proofd` observability namespace as:

- artifact-backed
- read-only
- non-authoritative

It exists to enforce the `Phase-13` rule:

`proofd` diagnostics surfaces explain verification state but do not mutate it, elect truth, or expose hidden control-plane affordances.

## Scope

The gate validates the `GET /diagnostics/*` family and the run-scoped `GET /diagnostics/runs/{run_id}/*` family.

It does not replace `ci-gate-proofd-service`.

`POST /verify/bundle` remains part of the execution surface outside the read-only observability namespace.

## Boundary Checks

The gate enforces:

- diagnostics endpoints remain artifact passthrough surfaces
- observability namespace rejects `POST`
- truth-election query smuggling fails closed
- cluster-commit query smuggling fails closed
- incident filters remain allow-listed and read-only
- diagnostics payloads do not expose truth-selection or authority-arbitration fields
- diagnostics payloads do not expose control-plane or mutation hints

## Negative Matrix Coverage

The gate executes these `Phase-13` negatives:

- `P13-NEG-01`
  - `POST /diagnostics/graph`
- `P13-NEG-02`
  - `POST /diagnostics/authority-topology`
- `P13-NEG-03`
  - `GET /diagnostics/graph?select_winner=true`
- `P13-NEG-04`
  - `GET /diagnostics/convergence?commit=true`
- `P13-NEG-13`
  - forbidden truth/arbitration fields such as:
    - `selected_truth`
    - `winning_verdict`
    - `committed_cluster`
    - `accepted_authority`
- `P13-NEG-14`
  - forbidden control-plane fields such as:
    - `retry`
    - `override`
    - `promote`
    - `commit`
    - `recommended_action`
    - `mitigation`
    - `routing_hint`
    - `node_priority`
    - `verification_weight`
    - `execution_override`

## Inputs

The gate expects an evidence root containing:

- `parity_determinism_incidents.json`
- `parity_report.json`
- `parity_drift_attribution_report.json`
- `parity_convergence_report.json`
- `failure_matrix.json`
- `parity_authority_drift_topology.json`
- `parity_authority_suppression_report.json`
- `parity_incident_graph.json`

It also expects a run-scoped directory with the same artifacts under:

- `<evidence_root>/<run_id>/`

## Outputs

The gate writes:

- `proofd_observability_boundary_report.json`
- `proofd_observability_negative_matrix.json`
- `report.json`
- `violations.txt`
- `meta.txt`

## Execution

Local:

```bash
make ci-gate-proofd-observability-boundary
```

Direct:

```bash
bash scripts/ci/gate_proofd_observability_boundary.sh \
  --evidence-dir evidence/run-<id>/gates/proofd-observability-boundary
```

Fixture mode:

```bash
bash scripts/ci/gate_proofd_observability_boundary.sh \
  --evidence-dir /tmp/proofd-boundary \
  --artifact-root /tmp/proofd-fixture \
  --run-id run-proofd-local-r1
```

## Failure Meaning

If this gate fails, `proofd` has drifted out of the AykenOS observability contract.

Typical failure classes:

- diagnostics namespace mutation
- query-parameter truth election
- hidden authority or consensus fields in payloads
- control-plane affordances embedded in diagnostics responses
- actionable remediation or routing signals embedded in diagnostics responses
