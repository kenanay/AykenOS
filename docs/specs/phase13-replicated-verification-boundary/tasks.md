# Implementation Plan: Phase 13 Replicated Verification Boundary

## Overview

Implement the `GET /diagnostics/runs/{run_id}/boundary` endpoint in `userspace/proofd`.
All changes are confined to `userspace/proofd/src/lib.rs`. The verify path is unchanged —
`proofd_run_manifest.json` already contains `request_fingerprint` and `verdict`.

## Tasks

- [x] 1. Add response types for boundary diagnostics
  - Add `BoundaryDiagnosticsResponseBody`, `VerdictConsistency`, `ContextHashConsistency`,
    `RegistryHashConsistency`, `RunVerdictEntry`, and `RunHashEntry` structs with
    `#[derive(Debug, Clone, Serialize)]`
  - `all_context_hashes_match` and `all_registry_hashes_match` are `Option<bool>` (serializes
    as JSON `null` when `None`)
  - _Requirements: 4.1, 4.2, 4.3, 4.4, 4.5, 4.6_

- [x] 2. Implement `build_run_boundary_diagnostics`
  - [x] 2.1 Implement the core function body
    - Check `run_dir.is_dir()` — return `NotFound("run_dir_not_found")` if absent
    - Load `proofd_run_manifest.json` via `load_required_run_json_artifact::<Value>` with
      error code `"invalid_run_manifest"`; the `NotFound` path from that helper becomes
      `"artifact_not_found"` (manifest absent)
    - Extract `request_fingerprint` as `&str` from the manifest value — return
      `MalformedArtifact("invalid_run_manifest")` if absent or not a string
    - Extract `verdict` as `String` from the primary manifest — return
      `MalformedArtifact("invalid_run_manifest")` if absent or not a string
    - Scan `evidence_dir` subdirectories for peers: skip the primary `run_id`, skip
      non-directories, skip unsafe path segments; for each candidate attempt to read its
      `proofd_run_manifest.json` as `Value` — silently skip on any error; compare
      `request_fingerprint` fields and collect matches into a peer list
    - Build `observed_verdicts`: one `RunVerdictEntry` per run (primary + peers); sort
      lexicographically by `run_id`
    - For each run (primary + peers), attempt to load
      `context/verification_context_object.json` as `Value` and extract
      `verification_context_id`; collect into `observed_context_hashes` (skip runs where
      artifact is absent or unparseable); sort by `run_id`
    - For each run (primary + peers), attempt to load `context/registry_snapshot.json` as
      `RegistrySnapshot` and call `compute_registry_snapshot_hash`; collect into
      `observed_registry_hashes` (skip runs where artifact is absent, unparseable, or hash
      fails); sort by `run_id`; always use the recomputed hash, never the self-declared field
    - Compute `all_verdicts_match`: `true` iff all entries in `observed_verdicts` carry the
      same verdict string
    - Compute `all_context_hashes_match`: `None` if `observed_context_hashes` is empty,
      `Some(true)` if all hashes are equal, `Some(false)` otherwise
    - Compute `all_registry_hashes_match`: same logic as context
    - Serialize via `serde_json::to_value` — return `Runtime("response_serialize_failed")`
      on error
    - _Requirements: 1.1, 1.2, 1.3, 1.4, 2.1, 2.2, 2.3, 2.4, 3.1, 3.4, 4.1, 4.2, 4.3,
      4.4, 4.5, 4.6, 4.7, 4.8, 5.1, 5.2, 5.3, 5.4, 5.5, 5.6_

  - [x] 2.2 Write unit tests for `build_run_boundary_diagnostics`
    - Single run, no peers → `peer_run_count: 0`, `observed_verdicts` has one entry
    - Primary + 2 peers with same fingerprint → `peer_run_count: 2`, three entries in
      `observed_verdicts`
    - Sibling with different fingerprint → not included as peer
    - Verdict mismatch across peers → `all_verdicts_match: false`
    - All runs have context objects with same `verification_context_id` →
      `all_context_hashes_match: true`
    - Context hash mismatch → `all_context_hashes_match: false`
    - No context objects present → `all_context_hashes_match: null`,
      `observed_context_hashes: []`
    - Registry hash consistency and mismatch cases
    - No registry snapshots present → `all_registry_hashes_match: null`,
      `observed_registry_hashes: []`
    - Sibling with missing manifest → silently skipped
    - Sibling with malformed manifest → silently skipped
    - Missing run directory → 404 `run_dir_not_found`
    - Missing `proofd_run_manifest.json` → 404 `artifact_not_found`
    - Malformed `proofd_run_manifest.json` → 500 `invalid_run_manifest`
    - Manifest missing `request_fingerprint` → 500 `invalid_run_manifest`
    - Observation arrays sorted by `run_id` (primary not necessarily first alphabetically)
    - Query string → 400 `unsupported_query_parameter`
    - POST method → 405 `method_not_allowed`
    - _Requirements: 1.2, 1.3, 1.4, 2.2, 2.3, 3.2, 3.3, 4.4, 4.5, 4.6, 4.7, 4.8, 5.1,
      5.6_

- [x] 3. Wire `build_run_boundary_diagnostics` into the router
  - Add `"boundary" if parts.len() == 4 =>` arm to `handle_run_endpoint` match block,
    alongside the existing `"registry"`, `"context"`, and `"federation"` arms
  - Pass `evidence_dir` through to `build_run_boundary_diagnostics` (it is already available
    in `handle_run_endpoint`)
  - _Requirements: 3.1, 3.2, 3.3_

- [x] 4. Checkpoint — ensure all tests pass
  - Run `cargo test --manifest-path userspace/proofd/Cargo.toml` and confirm all existing
    tests still pass alongside the new ones
  - Ensure all tests pass, ask the user if questions arise.

- [x] 5. Write property-based tests
  - [x]* 5.1 Write property test for response structure completeness
    - **Property 1: Response structure completeness**
    - Generate a random `run_id` and `request_fingerprint`; write a minimal manifest; call
      the boundary endpoint; assert HTTP 200 and all required top-level fields are present
      with correct types
    - **Validates: Requirements 1.1, 3.1, 4.1, 4.2, 4.3**

  - [x]* 5.2 Write property test for peer discovery accuracy
    - **Property 2: Peer discovery accuracy**
    - Generate an evidence root with N runs (1–8) sharing a fingerprint and M runs (0–4)
      with different fingerprints; for each of the N runs assert `peer_run_count == N - 1`
      and `observed_verdicts.len() == N`; assert runs with different fingerprints are never
      included
    - **Validates: Requirements 2.1, 2.4, 4.3, 4.4**

  - [x]* 5.3 Write property test for verdict consistency semantics
    - **Property 3: Verdict consistency semantics**
    - Generate a set of runs with random verdict strings; call the boundary endpoint; assert
      `all_verdicts_match` equals whether all verdict strings in `observed_verdicts` are
      identical
    - **Validates: Requirements 4.4, 5.1**

  - [x]* 5.4 Write property test for context hash consistency semantics
    - **Property 4: Context hash consistency semantics**
    - Generate runs where each may or may not have a context object with a random
      `verification_context_id`; assert `observed_context_hashes` contains exactly one entry
      per run with the artifact, each `hash` equals the `verification_context_id`, and
      `all_context_hashes_match` reflects actual equality (null when empty)
    - **Validates: Requirements 4.5, 4.7, 5.2, 5.5**

  - [x]* 5.5 Write property test for registry hash consistency semantics
    - **Property 5: Registry hash consistency semantics**
    - Generate runs where each may or may not have a `RegistrySnapshot`; assert
      `observed_registry_hashes` contains exactly one entry per run with the artifact, each
      `hash` equals `compute_registry_snapshot_hash` of the deserialized snapshot (not the
      self-declared field), and `all_registry_hashes_match` reflects actual equality (null
      when empty)
    - **Validates: Requirements 4.6, 4.8, 5.3, 5.4**

  - [x]* 5.6 Write property test for observation array sort order
    - **Property 6: Observation arrays are sorted by run_id**
    - Generate a set of runs with random `run_id` strings; call the boundary endpoint; assert
      `observed_verdicts`, `observed_context_hashes`, and `observed_registry_hashes` are each
      in strict lexicographic order by `run_id`
    - **Validates: Requirements 5.6**

  - [x]* 5.7 Write property test for endpoint read-only invariant
    - **Property 7: Endpoint is read-only**
    - Snapshot the file set of an evidence root; call
      `GET /diagnostics/runs/{run_id}/boundary` one or more times; assert the file set is
      unchanged
    - **Validates: Requirements 3.4, 5.7**

- [x] 6. Final checkpoint — ensure all tests pass
  - Run `cargo test --manifest-path userspace/proofd/Cargo.toml` and confirm all tests pass
  - Ensure all tests pass, ask the user if questions arise.

## Notes

- Tasks marked with `*` are optional and can be skipped for a faster MVP
- All changes are confined to `userspace/proofd/src/lib.rs` — no new files, no new crate
  dependencies for the core implementation
- `evidence_dir` is already threaded through `handle_run_endpoint`; the function signature
  change to `build_run_boundary_diagnostics` is the only routing-layer delta
- The verify path (`POST /verify/bundle`) requires no changes — `proofd_run_manifest.json`
  already contains `request_fingerprint` and `verdict`
- For context hash observation, use `verification_context_id` from
  `context/verification_context_object.json` loaded as `Value` (no typed struct needed)
- For registry hash observation, load as `RegistrySnapshot` and call
  `compute_registry_snapshot_hash` — never trust the self-declared `registry_snapshot_hash`
  field inside the artifact
