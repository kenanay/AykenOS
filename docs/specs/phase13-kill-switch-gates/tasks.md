# Implementation Plan: Phase-13 Kill-Switch Gates

## Overview

Add four architectural kill-switch CI gates as Rust tests inside `userspace/proofd/src/lib.rs`.
All tests validate that `proofd` diagnostics surfaces remain artifact-backed, read-only, and
non-authoritative. No new files, no new crate dependencies.

Two new test modules are added at the bottom of `lib.rs`:
- `mod tests_kill_switch_gates` — concrete unit tests (deterministic)
- `mod proptest_kill_switch_gates` — property-based tests (randomized, 100 cases each)

Run with: `cargo test --manifest-path userspace/proofd/Cargo.toml`

## Tasks

- [x] 1. Add shared test helpers to `lib.rs`
  - Add `response_contains_forbidden_field(body: &Value, fields: &[&str]) -> bool` helper
  - Add `json_contains_key(value: &Value, key: &str) -> bool` recursive helper (checks nested objects and arrays)
  - Place both helpers inside `#[cfg(test)]` scope, accessible to both test modules
  - _Requirements: 1.3, 1.4, 3.1, 3.2, 4.1_

- [x] 2. Implement Gate 1 — `ci-gate-proofd-observability-boundary`
  - [x] 2.1 Write unit tests for Gate 1 in `mod tests_kill_switch_gates`
  - [x] 2.2 Write property test for Gate 1 — Property 1: POST observability paths always return 405
    - **Validates: Requirements 1.1**
  - [x] 2.3 Write property test for Gate 1 — Property 2: Unsupported query always returns 400
    - **Validates: Requirements 1.2, 1.5**
  - [x] 2.4 Write property test for Gate 1 — Property 3: No forbidden fields in observability responses
    - **Validates: Requirements 1.3, 1.4**

- [x] 3. Implement Gate 2 — `ci-gate-observability-routing-separation`
  - [x] 3.1 Write deterministic source-scan unit test in `mod tests_kill_switch_gates`

- [x] 4. Checkpoint — ensure Gates 1 and 2 pass

- [x] 5. Implement Gate 3 — `ci-gate-convergence-non-election-boundary`
  - [x] 5.1 Write unit tests for Gate 3 in `mod tests_kill_switch_gates`
  - [x] 5.2 Write property test for Gate 3 — Property 4: No forbidden election fields in convergence responses
    - **Validates: Requirements 3.1, 3.2**

- [x] 6. Implement Gate 4 — `ci-gate-verifier-reputation-prohibition`
  - [x] 6.1 Write unit tests for Gate 4 in `mod tests_kill_switch_gates`
  - [x] 6.2 Write property test for Gate 4 — Property 5: No forbidden reputation fields in parity responses
    - **Validates: Requirements 4.1, 4.4**

- [x] 7. Final checkpoint — ensure all four gates pass

- [x] 8. Add Property 6: Artifact passthrough integrity
  - Write property test in `mod proptest_kill_switch_gates`
  - Generate random safe artifact, write to temp dir, serve via `GET /diagnostics/convergence`,
    assert response body deserializes to the same JSON value as the written artifact
  - proofd must not modify, interpret, aggregate, vote, or rank artifact content
  - **Validates: Requirements 1.3, 1.4, 3.1, 4.1**
  - [x] 8.1 Write `prop6_artifact_passthrough_integrity` in `mod proptest_kill_switch_gates`
    - **PBT Status: passed**

- [x] 9. Add Property 7: Diagnostics read-only surface
  - Write property test in `mod proptest_kill_switch_gates`
  - Generate random diagnostics path suffix + non-GET method (POST/PUT/PATCH/DELETE)
  - Assert all non-GET requests to `/diagnostics/*` return 405 with `method_not_allowed`
  - **Validates: Requirements 1.1**
  - [x] 9.1 Write `prop7_diagnostics_read_only_surface` in `mod proptest_kill_switch_gates`
    - **PBT:** prop7_diagnostics_read_only_surface — passed ✓

## Notes

- Tasks marked with `*` are optional and can be skipped for a faster MVP (unit tests alone satisfy the gate contract)
- Gate 2 (source scan) is a deterministic unit test — no property test needed
- `json_contains_key` must recurse into nested objects and arrays to catch fields at any depth
- Synthetic artifacts are plain JSON objects written to `temp_dir()` — no real verification run needed
- The source scan in task 3.1 reads the file relative to `CARGO_MANIFEST_DIR` at test time
