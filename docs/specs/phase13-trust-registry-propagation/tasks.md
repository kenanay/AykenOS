# Implementation Plan: Phase 13 Trust Registry Propagation

## Overview

Implement the `GET /diagnostics/runs/{run_id}/registry` endpoint in `userspace/proofd` and promote `context/registry_snapshot.json` to a first-class named artifact. All changes are confined to `userspace/proofd/src/lib.rs`.

## Tasks

- [x] 1. Add response types for registry diagnostics
  - Add `RegistryDiagnosticsResponseBody`, `RegistryContextBindingStatus`, and `RegistryObservationSource` structs with `#[derive(Debug, Clone, Serialize)]`
  - `registry_snapshot_hash_matches_declared_context` field is `Option<bool>` (serializes as `null` when absent)
  - `source_artifact_path` on `RegistryObservationSource` uses `#[serde(skip_serializing_if = "Option::is_none")]`
  - _Requirements: 3.1, 3.2, 3.3, 3.4, 3.5, 3.6, 3.7_

- [x] 2. Implement `build_run_registry_diagnostics`
  - [x] 2.1 Implement the core function body
    - Load `context/registry_snapshot.json` via `load_required_run_json_artifact::<RegistrySnapshot>` — return `MalformedArtifact("invalid_context_registry_snapshot")` on parse failure
    - Call `compute_registry_snapshot_hash(&registry)` — return `MalformedArtifact("invalid_context_registry_snapshot")` on error; always use the recomputed hash as `declared_registry_snapshot_hash`, never the self-declared `snapshot.registry_snapshot_hash` field
    - Set `declared_registry_entry_count = registry.producers.len()`
    - Load `context/verification_context_object.json` via `load_optional_run_json_artifact::<VerificationContextObject>` — return `MalformedArtifact("invalid_verification_context_object")` if present but unparseable
    - Load `receipts/verification_receipt.json` via `load_optional_run_json_artifact::<proof_verifier::VerificationReceipt>` — return `MalformedArtifact("invalid_receipt_artifact")` if present but unparseable
    - Build `context_binding_status`: `Some(recomputed == context_obj.registry_snapshot_hash)` when context object present, `None` when absent
    - Build `observed_registry_hash_sources` in fixed order: context object first, receipt second; call `unique_sorted_strings` on each values vec; drop entries with empty values
    - Serialize via `serde_json::to_value` — return `Runtime("response_serialize_failed")` on error
    - _Requirements: 3.1, 3.2, 3.3, 3.4, 3.5, 3.6, 3.7, 3.8, 4.1, 4.2, 4.3, 5.1, 5.3, 5.4_

  - [x]* 2.2 Write unit tests for `build_run_registry_diagnostics`
    - Happy path: registry + context object + receipt present, hashes match → 200, correct fields
    - Happy path: registry only (no context object, no receipt) → `null` binding status, empty sources array
    - Hash mismatch: context object has different `registry_snapshot_hash` → `false` binding status
    - Missing run directory → 404 `run_dir_not_found`
    - Missing `context/registry_snapshot.json` → 404 `artifact_not_found`
    - Malformed `context/registry_snapshot.json` → 500 `invalid_context_registry_snapshot`
    - `declared_registry_entry_count` equals `producers.len()` for a snapshot with N producers
    - Query string on registry endpoint → 400 `unsupported_query_parameter`
    - POST to registry endpoint → 405 `method_not_allowed`
    - _Requirements: 2.1, 2.2, 2.3, 2.4, 2.5, 3.3, 3.4, 3.5, 3.6, 5.3_

- [x] 3. Wire `build_run_registry_diagnostics` into the router
  - Add `"registry" if parts.len() == 4 =>` arm to `handle_run_endpoint` match block, alongside the existing `"federation"` and `"context"` arms
  - _Requirements: 2.1, 2.2, 2.3, 2.4, 2.5_

- [x] 4. Checkpoint — ensure all tests pass
  - Run `cargo test -p proofd` and confirm all existing tests still pass alongside the new ones
  - Ensure all tests pass, ask the user if questions arise.

- [x] 5. Write property-based tests
  - [x]* 5.1 Write property test for registry artifact write idempotence
    - **Property 1: Registry artifact write idempotence**
    - Generate a random `RegistrySnapshot`; call `write_canonical_json_file_if_absent_or_same` twice; assert both succeed and file bytes are identical
    - **Validates: Requirements 1.2**

  - [x]* 5.2 Write property test for hash consistency
    - **Property 2: Registry diagnostics hash consistency**
    - Generate a random `RegistrySnapshot`; write to temp run dir; call registry endpoint; assert `declared_registry_snapshot_hash` equals `compute_registry_snapshot_hash(&snapshot)`
    - **Validates: Requirements 3.3, 5.3, 5.4**

  - [x]* 5.3 Write property test for entry count
    - **Property 6: Entry count matches producers map**
    - Generate a random `RegistrySnapshot` with 0–20 producers; write and call endpoint; assert `declared_registry_entry_count == producers.len()`
    - **Validates: Requirements 3.4**

  - [x]* 5.4 Write property test for context binding status correctness
    - **Property 3: Context binding status correctness**
    - Generate a random `RegistrySnapshot` and `VerificationContextObject`; vary whether the context object's `registry_snapshot_hash` matches the recomputed hash; assert the boolean reflects actual equality; also test absent context object → `null`
    - **Validates: Requirements 3.5, 3.6**

  - [x]* 5.5 Write property test for source observation completeness
    - **Property 5: Source observation completeness**
    - Generate a run with any combination of present/absent context object and receipt; assert every present artifact with a non-empty hash appears in `observed_registry_hash_sources` with the correct `source` label
    - **Validates: Requirements 4.1, 4.2**

  - [x]* 5.6 Write property test for observed sources values uniqueness and sort order
    - **Property 4: Observed sources values are unique and sorted**
    - Generate a run with source surfaces; call endpoint; for every entry in `observed_registry_hash_sources` assert `values` has no duplicates and is lexicographically sorted
    - **Validates: Requirements 3.7**

  - [x]* 5.7 Write property test for empty sources omission
    - **Property 5 (edge): Empty sources are omitted**
    - Generate a run where one or more source surfaces are absent; assert no entry with an empty `values` array appears in `observed_registry_hash_sources`
    - **Validates: Requirements 3.8, 4.3**

  - [x]* 5.8 Write property test for endpoint read-only invariant
    - **Property 7: Endpoint is read-only**
    - Snapshot the file set of a run directory; call `GET /diagnostics/runs/{run_id}/registry` one or more times; assert the file set is unchanged
    - **Validates: Requirements 5.1, 5.2**

- [x] 6. Final checkpoint — ensure all tests pass
  - Run `cargo test -p proofd` and confirm all tests pass
  - Ensure all tests pass, ask the user if questions arise.

## Notes

- Tasks marked with `*` are optional and can be skipped for a faster MVP
- All changes are confined to `userspace/proofd/src/lib.rs` — no new files, no new crate dependencies for the core implementation
- Property tests may require adding `proptest` to `[dev-dependencies]` in `userspace/proofd/Cargo.toml`
- The `context/registry_snapshot.json` artifact is already written by `write_verification_context_package`; no changes to the verify path are needed
- `NESTED_RUN_LEVEL_ARTIFACTS` already contains `CONTEXT_REGISTRY_SNAPSHOT_RELATIVE_PATH`; requirement 1.3 and 1.4 are already satisfied
