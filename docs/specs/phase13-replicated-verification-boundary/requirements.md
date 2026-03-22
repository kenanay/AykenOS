# Requirements Document

## Introduction

Phase 13 replicated verification boundary extends the `proofd` verification service to expose a read-only cross-run consistency projection via a new `GET /diagnostics/runs/{run_id}/boundary` endpoint.

Multiple runs may verify the same bundle with the same policy and registry inputs. The replicated verification boundary is the observable surface that shows whether two or more such runs produced consistent verdicts, consistent context hashes, and consistent registry hashes. This dilim is diagnostics-only: it reads existing run artifacts and projects a cross-run consistency view without introducing consensus, authority election, or any write semantics.

This follows the established Phase 13 dilim pattern: `POST /verify/bundle` materializes artifacts, and `GET /diagnostics/runs/{run_id}/...` endpoints expose descriptive projections. No authority resolution, trust election, or consensus semantics are introduced.

## Glossary

- **Proofd**: The `userspace/proofd` Rust service that executes bundle verification and exposes read-only diagnostics.
- **Run**: A single invocation of `POST /verify/bundle` identified by a `run_id`, producing artifacts under `evidence/{run_id}/`.
- **Run_Manifest**: The `proofd_run_manifest.json` artifact written to the run directory during `POST /verify/bundle`, containing the `request_fingerprint` and `verdict` fields for the run.
- **Request_Fingerprint**: The SHA-256 hash of the canonical `VerifyBundleRequestBody` for a run, stored as `request_fingerprint` in the `Run_Manifest`. Two runs share the same fingerprint if and only if they were invoked with identical request parameters.
- **Peer_Run**: Any run directory under the same evidence root whose `Run_Manifest` carries the same `request_fingerprint` as the primary run, excluding the primary run itself.
- **Boundary_Diagnostics_Endpoint**: The new `GET /diagnostics/runs/{run_id}/boundary` endpoint.
- **Verdict_Consistency**: A descriptive projection of whether all peer runs and the primary run produced the same verdict string.
- **Context_Hash_Consistency**: A descriptive projection of whether all runs that have a `context/verification_context_object.json` artifact carry the same `verification_context_id` value.
- **Registry_Hash_Consistency**: A descriptive projection of whether all runs that have a `context/registry_snapshot.json` artifact carry the same recomputed registry snapshot hash.
- **Evidence_Root**: The directory passed to `proofd` as its evidence base, containing one subdirectory per run.

## Requirements

### Requirement 1: Run Manifest as Boundary Anchor

**User Story:** As a verification operator, I want each run's manifest to serve as the anchor for cross-run boundary discovery, so that peer runs can be identified by their shared request fingerprint without any authority resolution.

#### Acceptance Criteria

1. WHEN `GET /diagnostics/runs/{run_id}/boundary` is called and `proofd_run_manifest.json` is present in the run directory, THE Boundary_Diagnostics_Endpoint SHALL read the `request_fingerprint` field from the manifest to identify peer runs.
2. WHEN `GET /diagnostics/runs/{run_id}/boundary` is called and `proofd_run_manifest.json` is absent from the run directory, THE Boundary_Diagnostics_Endpoint SHALL return HTTP 404 with `{"error": "artifact_not_found"}`.
3. WHEN `GET /diagnostics/runs/{run_id}/boundary` is called and the run directory does not exist, THE Boundary_Diagnostics_Endpoint SHALL return HTTP 404 with `{"error": "run_dir_not_found"}`.
4. WHEN `proofd_run_manifest.json` is present but cannot be parsed as valid JSON or is missing the `request_fingerprint` field, THE Boundary_Diagnostics_Endpoint SHALL return HTTP 500 with `{"error": "invalid_run_manifest"}`.

### Requirement 2: Peer Run Discovery

**User Story:** As a verification operator, I want the boundary endpoint to discover all sibling runs that share the same request fingerprint, so that I can see the full replication surface for a given verification request.

#### Acceptance Criteria

1. WHEN `GET /diagnostics/runs/{run_id}/boundary` is called, THE Boundary_Diagnostics_Endpoint SHALL scan all subdirectories of the evidence root and identify those whose `proofd_run_manifest.json` carries the same `request_fingerprint` as the primary run, excluding the primary run itself.
2. WHEN a sibling run directory has no `proofd_run_manifest.json`, THE Boundary_Diagnostics_Endpoint SHALL silently skip that directory and continue peer discovery.
3. WHEN a sibling run directory has a `proofd_run_manifest.json` that cannot be parsed or is missing the `request_fingerprint` field, THE Boundary_Diagnostics_Endpoint SHALL silently skip that directory and continue peer discovery.
4. THE Boundary_Diagnostics_Endpoint SHALL include `peer_run_count` as a non-negative integer in the response body equal to the number of discovered peer runs.
5. THE Boundary_Diagnostics_Endpoint SHALL NOT perform authority resolution, trust election, or consensus semantics during peer discovery.

### Requirement 3: Boundary Diagnostics Endpoint

**User Story:** As a verification operator, I want a read-only diagnostics endpoint that shows the cross-run consistency status for a given run, so that I can inspect replication consistency without any write side-effects.

#### Acceptance Criteria

1. WHEN `GET /diagnostics/runs/{run_id}/boundary` is called and the run directory exists with `proofd_run_manifest.json` present and parseable, THE Boundary_Diagnostics_Endpoint SHALL return HTTP 200 with a JSON body containing the boundary diagnostics projection.
2. WHEN `GET /diagnostics/runs/{run_id}/boundary` is called with a query string, THE Boundary_Diagnostics_Endpoint SHALL return HTTP 400 with `{"error": "unsupported_query_parameter"}`.
3. WHEN `POST /diagnostics/runs/{run_id}/boundary` is called, THE Proofd SHALL return HTTP 405 with `{"error": "method_not_allowed"}`.
4. THE Boundary_Diagnostics_Endpoint SHALL NOT modify any artifact on disk.
5. THE Boundary_Diagnostics_Endpoint SHALL NOT accept request bodies.

### Requirement 4: Boundary Diagnostics Response Body

**User Story:** As a verification operator, I want the boundary diagnostics response to show the run's fingerprint, peer count, and consistency projections for verdicts, context hashes, and registry hashes, so that I can assess cross-run replication consistency at a glance.

#### Acceptance Criteria

1. THE Boundary_Diagnostics_Endpoint SHALL include `run_id` as a string field in the response body.
2. THE Boundary_Diagnostics_Endpoint SHALL include `request_fingerprint` as a string field containing the value read from the primary run's `Run_Manifest`.
3. THE Boundary_Diagnostics_Endpoint SHALL include `peer_run_count` as a non-negative integer field equal to the number of discovered peer runs.
4. THE Boundary_Diagnostics_Endpoint SHALL include `verdict_consistency` as an object with a boolean field `all_verdicts_match` and an array field `observed_verdicts`, where each element has `run_id` (string) and `verdict` (string) fields, covering the primary run and all peer runs.
5. THE Boundary_Diagnostics_Endpoint SHALL include `context_hash_consistency` as an object with an `Option<bool>` field `all_context_hashes_match` (null when no run has a context object) and an array field `observed_context_hashes`, where each element has `run_id` (string) and `hash` (string) fields, covering only runs that have a `context/verification_context_object.json` artifact.
6. THE Boundary_Diagnostics_Endpoint SHALL include `registry_hash_consistency` as an object with an `Option<bool>` field `all_registry_hashes_match` (null when no run has a registry snapshot) and an array field `observed_registry_hashes`, where each element has `run_id` (string) and `hash` (string) fields, covering only runs that have a `context/registry_snapshot.json` artifact.
7. WHEN no runs have a `context/verification_context_object.json` artifact, THE Boundary_Diagnostics_Endpoint SHALL set `context_hash_consistency.all_context_hashes_match` to `null` and `observed_context_hashes` to an empty array.
8. WHEN no runs have a `context/registry_snapshot.json` artifact, THE Boundary_Diagnostics_Endpoint SHALL set `registry_hash_consistency.all_registry_hashes_match` to `null` and `observed_registry_hashes` to an empty array.
9. THE Boundary_Diagnostics_Endpoint SHALL include `peer_run_ids` as an array of strings in the response body, containing the `run_id` values of all discovered peer runs in lexicographic order.

### Requirement 5: Consistency Projection Semantics and Fail-Closed Constraints

**User Story:** As a system architect, I want the boundary consistency projections to be purely descriptive and strictly fail-closed, so that the endpoint cannot be used to influence verification outcomes or introduce policy decisions.

#### Acceptance Criteria

1. THE Boundary_Diagnostics_Endpoint SHALL compute `all_verdicts_match` as `true` if and only if all entries in `observed_verdicts` carry the same verdict string.
2. THE Boundary_Diagnostics_Endpoint SHALL compute `all_context_hashes_match` as `Some(true)` if and only if all entries in `observed_context_hashes` carry the same hash string, and as `Some(false)` if two or more entries carry different hash strings.
3. THE Boundary_Diagnostics_Endpoint SHALL compute `all_registry_hashes_match` as `Some(true)` if and only if all entries in `observed_registry_hashes` carry the same hash string, and as `Some(false)` if two or more entries carry different hash strings.
4. THE Boundary_Diagnostics_Endpoint SHALL recompute registry snapshot hashes from artifact bytes using `compute_registry_snapshot_hash` and SHALL NOT trust any self-declared hash field in the registry snapshot artifact.
5. THE Boundary_Diagnostics_Endpoint SHALL use the `verification_context_id` field from `context/verification_context_object.json` as the context hash for each run.
6. THE Boundary_Diagnostics_Endpoint SHALL enumerate `observed_verdicts`, `observed_context_hashes`, and `observed_registry_hashes` in deterministic lexicographic order by `run_id`.
7. THE Proofd SHALL NOT add authority resolution, trust election, or consensus semantics to any endpoint as part of this feature.
