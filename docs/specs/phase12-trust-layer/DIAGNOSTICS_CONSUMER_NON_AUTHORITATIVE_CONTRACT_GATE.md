# `ci-gate-diagnostics-consumer-non-authoritative-contract`

This gate freezes the consumer side of `Phase-13` diagnostics as:

- descriptive-only
- read-only
- non-authoritative
- non-executable

It exists to enforce:

`descriptive diagnostics artifacts MUST NOT become policy, authority, replay, routing, or execution input`

## Scope

This is a source-contract gate.

It scans runtime-bearing Rust sources under:

- `ayken-core/crates`
- `userspace`

and fails if descriptive diagnostics fields or diagnostics artifact names are referenced outside approved producer and passthrough surfaces.

## Approved Surfaces

The gate currently allows these runtime references:

- `ayken-core/crates/proof-verifier/examples/phase12_gate_harness.rs`
- `ayken-core/crates/proof-verifier/src/authority/authority_drift_topology.rs`
- `ayken-core/crates/proof-verifier/src/authority/drift_attribution.rs`
- `userspace/proofd/src/lib.rs`
- `userspace/proofd/examples/proofd_gate_harness.rs`

These are allowed because they either:

- produce canonical diagnostics artifacts
- expose raw artifact passthrough behavior
- enforce the diagnostics boundary itself

## Protected Diagnostics Fields

The gate protects descriptive-only fields such as:

- `dominant_authority_chain_id`
- `largest_outcome_cluster_size`
- `outcome_convergence_ratio`
- `historical_authority_island_count`
- `historical_authority_islands`
- `insufficient_evidence_island_count`
- `insufficient_evidence_islands`
- `global_status`
- `cluster_derivation`
- `edge_match_cluster_derivation`

It also protects diagnostics artifact identities such as:

- `parity_convergence_report.json`
- `parity_authority_drift_topology.json`
- `parity_drift_attribution_report.json`

## Contract

These diagnostics surfaces MAY be:

- produced
- served
- tested
- boundary-validated

They MUST NOT be:

- imported into policy engines
- reused as authority decisions
- reused as replay admission input
- reused as routing or priority input
- reused as execution overrides

## Negative Matrix Coverage

This gate primarily hardens:

- `P13-CONS-01`
  - diagnostics fields must not be imported into non-observability runtime code
- `P13-CONS-02`
  - convergence and topology artifacts must not become execution or routing inputs
- `P13-CONS-03`
  - diagnostic `global_status` must not become admission, policy, or priority input
- `P13-CONS-04`
  - historical or insufficient-evidence islands must not drive suppression or trust promotion

## Outputs

The gate writes:

- `diagnostics_consumer_contract_report.json`
- `report.json`
- `violations.txt`
- `meta.txt`

## Execution

Local:

```bash
make ci-gate-diagnostics-consumer-non-authoritative-contract
```

Fixture mode:

```bash
bash scripts/ci/gate_diagnostics_consumer_non_authoritative_contract.sh \
  --evidence-dir /tmp/diagnostics-consumer-gate \
  --source-root /tmp/fixture-root \
  --scan-root runtime
```

## Failure Meaning

If this gate fails, AykenOS diagnostics are no longer merely descriptive.

That means a runtime-bearing consumer has started to treat observability metadata as execution-bearing input.
