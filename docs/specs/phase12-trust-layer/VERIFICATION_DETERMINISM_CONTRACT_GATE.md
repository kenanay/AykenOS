# `ci-gate-verification-determinism-contract`

This gate freezes verifier-critical code as:

- environment-independent
- time-free
- randomness-free
- ambient-state-free

It exists to enforce:

`same artifact -> same verification result`

## Scope

The gate scans the verdict-bearing and parity-bearing modules inside `proof-verifier`.

It intentionally targets the semantic core, not the CLI, test fixtures, or service wrappers.

## Contract

Verifier-critical code MUST NOT depend on:

- wall-clock time
- randomness
- ambient environment variables
- network-visible context
- filesystem I/O

Those dependencies turn:

`verify(proof) -> verdict`

into:

`verify(proof, environment) -> verdict`

which is not allowed.

## Default Source Set

The default scan covers the curated verifier-critical modules under:

- `src/authority/`
- `src/policy/`
- `src/registry/`
- `src/verdict/`
- `src/canonical/`
- `src/receipt/verify.rs`
- `src/receipt/schema.rs`
- `src/overlay/overlay_validator.rs`
- `src/portable_core/identity.rs`

## Violation Classes

The gate fails closed on patterns such as:

- `SystemTime`
- `Instant`
- `rand::`
- `thread_rng`
- `std::env`
- `env::var`
- `TcpStream`
- `reqwest`
- `std::fs`
- `fs::read`

## Outputs

The gate writes:

- `verification_determinism_contract_report.json`
- `report.json`
- `violations.txt`
- `meta.txt`

## Execution

Local:

```bash
make ci-gate-verification-determinism-contract
```

Focused fixture:

```bash
bash scripts/ci/gate_verification_determinism_contract.sh \
  --evidence-dir /tmp/verification-determinism \
  --source-root /tmp/source-root \
  --source-path critical/verifier.rs
```

## Failure Meaning

If this gate fails, verifier-critical code has gained an ambient dependency that can make verification results vary by node or runtime context.
