# Requirements Document

## Introduction

Phase 13 trust registry propagation extends the `proofd` verification service to materialize run-local registry propagation material as canonical artifacts during bundle verification, and to expose a read-only descriptive projection of that material via a new `GET /diagnostics/runs/{run_id}/registry` endpoint.

This follows the established Phase 13 dilim pattern: `POST /verify/bundle` materializes artifacts, and `GET /diagnostics/runs/{run_id}/...` endpoints expose descriptive projections. No authority resolution, trust election, or consensus semantics are introduced.

## Glossary

- **Proofd**: The `userspace/proofd` Rust service that executes bundle verification and exposes read-only diagnostics.
- **Run**: A single invocation of `POST /verify/bundle` identified by a `run_id`, producing artifacts under `evidence/{run_id}/`.
- **Registry_Snapshot**: The `RegistrySnapshot` value loaded from `registry_path` during verification; the declared registry input to the verifier.
- **Registry_Propagation_Material**: The canonical artifact written to `context/registry_snapshot.json` during verification, representing the run-local registry snapshot used.
- **Registry_Snapshot_Hash**: The SHA-256 hash of the canonicalized `RegistrySnapshot`, as computed by `compute_registry_snapshot_hash`.
- **Registry_Diagnostics_Endpoint**: The new `GET /diagnostics/runs/{run_id}/registry` endpoint.
- **Verification_Context_Object**: The artifact at `context/verification_context_object.json` that binds `registry_snapshot_hash` to the run's verification context.
- **Authority_Chain_Registry_Binding**: The descriptive projection of how the registry snapshot hash is referenced across the verifier authority chain surfaces present in the run.
- **Source_Surface**: Any run artifact (receipt, diversity ledger, flow source documents, trust reuse runtime surface) that references a registry snapshot hash or registry-related field.

## Requirements

### Requirement 1: Registry Propagation Material Artifact

**User Story:** As a verification operator, I want the registry snapshot used during verification to be materialized as a canonical run-local artifact, so that the exact registry input is preserved and traceable for each run.

#### Acceptance Criteria

1. WHEN `POST /verify/bundle` is called with a valid request, THE Proofd SHALL write the registry snapshot to `context/registry_snapshot.json` within the run directory using canonical JSON encoding.
2. WHEN `POST /verify/bundle` is called and `context/registry_snapshot.json` already exists for the run, THE Proofd SHALL verify the existing file's bytes match the new canonical encoding and return an error if they conflict.
3. THE Proofd SHALL include `context/registry_snapshot.json` in the set of nested run-level artifact paths enumerated by `GET /diagnostics/runs/{run_id}/artifacts`.
4. WHEN `context/registry_snapshot.json` is present, THE Proofd SHALL make it accessible via `GET /diagnostics/runs/{run_id}/artifacts/context/registry_snapshot.json`.

### Requirement 2: Registry Diagnostics Endpoint

**User Story:** As a verification operator, I want a read-only diagnostics endpoint that shows the declared registry snapshot and its observed hash/source surfaces for a run, so that I can inspect registry binding without any authority resolution or trust election.

#### Acceptance Criteria

1. WHEN `GET /diagnostics/runs/{run_id}/registry` is called and the run directory exists with `context/registry_snapshot.json` present, THE Registry_Diagnostics_Endpoint SHALL return HTTP 200 with a JSON body containing the run's registry diagnostics projection.
2. WHEN `GET /diagnostics/runs/{run_id}/registry` is called and `context/registry_snapshot.json` is absent, THE Registry_Diagnostics_Endpoint SHALL return HTTP 404 with `{"error": "artifact_not_found"}`.
3. WHEN `GET /diagnostics/runs/{run_id}/registry` is called and the run directory does not exist, THE Registry_Diagnostics_Endpoint SHALL return HTTP 404 with `{"error": "run_dir_not_found"}`.
4. WHEN `GET /diagnostics/runs/{run_id}/registry` is called with a query string, THE Registry_Diagnostics_Endpoint SHALL return HTTP 400 with `{"error": "unsupported_query_parameter"}`.
5. WHEN `POST /diagnostics/runs/{run_id}/registry` is called, THE Proofd SHALL return HTTP 405 with `{"error": "method_not_allowed"}`.

### Requirement 3: Registry Diagnostics Response Body

**User Story:** As a verification operator, I want the registry diagnostics response to show the declared snapshot, its computed hash, and all source surfaces that reference the registry, so that I can verify registry binding consistency across the run.

#### Acceptance Criteria

1. THE Registry_Diagnostics_Endpoint SHALL include `run_id` as a string field in the response body.
2. THE Registry_Diagnostics_Endpoint SHALL include `source_artifact_path` as a string field set to `"context/registry_snapshot.json"` in the response body.
3. THE Registry_Diagnostics_Endpoint SHALL include `declared_registry_snapshot_hash` as a string field containing the recomputed hash of the artifact at `context/registry_snapshot.json`.
4. THE Registry_Diagnostics_Endpoint SHALL include `declared_registry_entry_count` as a non-negative integer field equal to the number of entries in the `producers` map of the registry snapshot.
5. THE Registry_Diagnostics_Endpoint SHALL include `context_binding_status` as an object with a boolean field `registry_snapshot_hash_matches_declared_context` indicating whether the recomputed hash matches the `registry_snapshot_hash` field in the `Verification_Context_Object`.
6. WHEN the `Verification_Context_Object` is absent, THE Registry_Diagnostics_Endpoint SHALL set `context_binding_status.registry_snapshot_hash_matches_declared_context` to `null`.
7. THE Registry_Diagnostics_Endpoint SHALL include `observed_registry_hash_sources` as an array of source surface objects, each with `source` (string), `source_artifact_path` (optional string), and `values` (array of strings) fields, enumerating every run artifact that references a registry snapshot hash. The `values` array for each source MUST contain only unique strings in lexicographic order.
8. WHEN no source surfaces reference a registry snapshot hash, THE Registry_Diagnostics_Endpoint SHALL return `observed_registry_hash_sources` as an empty array.

### Requirement 4: Registry Hash Source Surface Observation

**User Story:** As a verification operator, I want the registry diagnostics to enumerate all run artifacts that carry a registry snapshot hash reference, so that I can confirm registry binding consistency across the full run surface.

#### Acceptance Criteria

1. WHEN a receipt artifact is present at `receipts/verification_receipt.json`, THE Registry_Diagnostics_Endpoint SHALL include the receipt's `registry_snapshot_hash` field value as an observed source with `source` set to `"receipt"`.
2. WHEN a `Verification_Context_Object` is present at `context/verification_context_object.json`, THE Registry_Diagnostics_Endpoint SHALL include its `registry_snapshot_hash` field value as an observed source with `source` set to `"verification_context_object"`.
3. THE Registry_Diagnostics_Endpoint SHALL omit source surface entries whose `values` array is empty.
4. THE Registry_Diagnostics_Endpoint SHALL NOT perform authority resolution, trust election, or consensus semantics when building the observed sources list.

### Requirement 5: Fail-Closed and Read-Only Constraints

**User Story:** As a system architect, I want the registry propagation feature to remain strictly fail-closed and read-only on the diagnostics side, so that it cannot be used to influence verification outcomes or introduce policy decisions.

#### Acceptance Criteria

1. THE Registry_Diagnostics_Endpoint SHALL NOT modify any artifact on disk.
2. THE Registry_Diagnostics_Endpoint SHALL NOT accept request bodies.
3. IF `context/registry_snapshot.json` cannot be parsed as a valid `RegistrySnapshot`, THEN THE Registry_Diagnostics_Endpoint SHALL return HTTP 500 with `{"error": "invalid_context_registry_snapshot"}`.
4. IF `compute_registry_snapshot_hash` fails for the loaded snapshot, THEN THE Registry_Diagnostics_Endpoint SHALL return HTTP 500 with `{"error": "invalid_context_registry_snapshot"}`.
5. THE Proofd SHALL NOT add authority resolution, trust election, or consensus semantics to any endpoint as part of this feature.
