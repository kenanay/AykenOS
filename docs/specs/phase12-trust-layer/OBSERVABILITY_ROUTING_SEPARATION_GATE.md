# `ci-gate-observability-routing-separation`

This gate hardens the `Phase-13` routing-blindness boundary.

It exists to enforce:

`observability != scheduling`

and the operational reading:

`verification routing must be observability blind`

## Scope

This gate is a routing and scheduling contract gate over the verification-facing Rust surfaces:

- `ayken-core/crates/proof-verifier`
- `userspace/proofd`

It is intentionally narrower than general scheduler logic elsewhere in the repo.

It protects the future federated verification path, not the historical kernel scheduler path.

## Protected Observability Sources

The gate treats descriptive observability fields and artifacts such as these as routing-blindness sources:

- `dominant_authority_chain_id`
- `largest_outcome_cluster_size`
- `outcome_convergence_ratio`
- `global_status`
- `historical_authority_islands`
- `insufficient_evidence_islands`
- `suppressed_drift_count`
- `parity_convergence_report.json`
- `parity_authority_drift_topology.json`
- `parity_authority_suppression_report.json`
- `parity_drift_attribution_report.json`

## Protected Routing Context

The gate inspects functions or call sites that look like verification-routing or verifier-selection surfaces, including names such as:

- `route_verification`
- `verification_route`
- `schedule_verification`
- `schedule_next_verifier`
- `select_verifier`
- `choose_verifier`
- `prefer_verifier`
- `set_preferred_node`
- `set_verifier_order`

## Contract

Inside those routing or scheduling contexts:

- observability modules MUST NOT be imported directly
- descriptive observability fields MUST NOT be read
- aliases derived from descriptive observability MUST NOT be reused
- dominant-cluster, convergence, or suppression signals MUST NOT become ordering heuristics
- routing MUST NOT optimize for agreement likelihood
- scheduling MUST preserve diversity rather than prefer likely-agreeing nodes

The shortest correct reading is:

`diagnostics explain the system; diagnostics must never steer the system`

## Negative Matrix Coverage

This gate primarily hardens:

- `P13-FEED-01`
  - descriptive observability fields becoming routing or ordering input
- `P13-FEED-02`
  - topology or convergence analytics biasing routing order
- `P13-FEED-03`
  - suppression or island diagnostics becoming orchestration control
- `P13-FEED-04`
  - scheduling optimizing for agreement likelihood instead of diversity
- `P13-FEED-05`
  - routing or scheduling files importing observability modules directly

## Outputs

The gate writes:

- `observability_routing_separation_report.json`
- `observability_routing_negative_matrix.json`
- `report.json`
- `violations.txt`
- `meta.txt`

## Execution

Local:

```bash
make ci-gate-observability-routing-separation
```

Fixture mode:

```bash
bash scripts/ci/gate_observability_routing_separation.sh \
  --evidence-dir /tmp/observability-routing-separation \
  --source-root /tmp/fixture-root \
  --source-path approved/routing.rs
```

## Failure Meaning

If this gate fails, a verification-routing or scheduling surface has started to consume descriptive observability as behavior-shaping input.

That means AykenOS is drifting from:

`distributed verification observability`

toward:

`implicit authority injection through routing bias`
