# PROOFD External Diagnostics Contract v1

## Purpose

This document defines the canonical external diagnostics contract for `proofd`.
It exists so the public query surface, router behavior, and CI harness
expectations derive from a single truth surface.

## Scope

This contract covers:

- public root diagnostics endpoints
- public run-scoped diagnostics endpoints
- allowed HTTP methods
- allowed query parameter keys
- forbidden response-field classes for observability payloads

This contract does not cover:

- `POST /verify/bundle`
- `POST /internal/replay`
- replay execution semantics
- baseline authority or performance authority

## Invariants

- `proofd` is a verification and diagnostics service, not an authority service.
- Public diagnostics are read-only.
- Public diagnostics do not imply replay admission, consensus, routing, or trust arbitration.
- Internal service routes are explicitly outside the external diagnostics contract.

## Public Endpoints

### Service

- `GET /healthz`
- `GET /diagnostics/version`

### Root Diagnostics

- `GET /diagnostics/runs`
- `GET /diagnostics/federation`
- `GET /diagnostics/context`
- `GET /diagnostics/trust`
- `GET /diagnostics/parity`
- `GET /diagnostics/parity/context-relation`
- `GET /diagnostics/incidents`
- `GET /diagnostics/incidents/{incident_id}`
- `GET /diagnostics/fingerprints/{fp}`
- `GET /diagnostics/replicated-boundary`
- `GET /diagnostics/authority-suppression`
- `GET /diagnostics/authority-topology`
- `GET /diagnostics/graph`
- `GET /diagnostics/drift`
- `GET /diagnostics/convergence`
- `GET /diagnostics/failure-matrix`

### Run-Scoped Diagnostics

- `GET /diagnostics/runs/{run_id}`
- `GET /diagnostics/runs/{run_id}/artifacts`
- `GET /diagnostics/runs/{run_id}/artifacts/{artifact_path}`
- `GET /diagnostics/runs/{run_id}/federation`
- `GET /diagnostics/runs/{run_id}/context`
- `GET /diagnostics/runs/{run_id}/registry`
- `GET /diagnostics/runs/{run_id}/boundary`
- `GET /diagnostics/runs/{run_id}/incidents`
- `GET /diagnostics/runs/{run_id}/parity`
- `GET /diagnostics/runs/{run_id}/authority-suppression`
- `GET /diagnostics/runs/{run_id}/authority-topology`
- `GET /diagnostics/runs/{run_id}/graph`
- `GET /diagnostics/runs/{run_id}/drift`
- `GET /diagnostics/runs/{run_id}/convergence`
- `GET /diagnostics/runs/{run_id}/failure-matrix`

## Query Rules

- `GET /diagnostics/incidents`
  - allowed query keys: `severity`, `surface_key`, `node_id`
- all other public diagnostics endpoints
  - query parameters are forbidden
  - unsupported keys MUST fail closed with `400 unsupported_query_parameter`

## Method Rules

- Public diagnostics endpoints are `GET` only.
- `POST`, `PUT`, `PATCH`, and `DELETE` against `/diagnostics/*` MUST return `405 method_not_allowed`.

## Forbidden Response Fields

The public diagnostics surface MUST NOT expose truth-election or control-plane
affordances. Forbidden normalized field classes include:

- truth-election style fields
  - `selectedtruth`
  - `winningverdict`
  - `committedcluster`
  - `acceptedauthority`
  - `acceptauthority`
  - `resolvetruth`
  - `selectwinner`
  - `elect`
- control-plane style fields
  - `retry`
  - `override`
  - `promote`
  - `commit`
  - `forceaccept`
  - `recommendedaction`
  - `recommendedactions`
  - `mitigation`
  - `routinghint`
  - `nodepriority`
  - `verificationweight`
  - `executionoverride`
  - `quarantine`
  - `autoquarantine`
  - `autorecovery`
  - `suppressnode`
  - `triggerreplayadmission`
  - `commitclusterstate`

## Runtime Enforcement

- public diagnostics responses MUST be checked at runtime for forbidden normalized fields
- detected forbidden fields MUST fail closed
- the current fail-closed error is `forbidden_observability_field_exposed`
- artifact-backed passthrough does not bypass this rule

## Canonical Source

The canonical implementation source for this contract is:

- `userspace/proofd/src/api_contract.rs`

The following surfaces MUST derive from that module:

- `/diagnostics/version` endpoint declarations
- GET query validation for public diagnostics
- artifact-backed public route lookup for eligible passthrough endpoints
- `proofd_gate_harness` endpoint expectations
- forbidden observability field scan mapping

## Fail-Closed Rules

- public query mismatch => `400 unsupported_query_parameter`
- unsupported public write method => `405 method_not_allowed`
- unknown path => `404 not_found`
- internal routes MUST NOT appear in external endpoint declarations

## Authority Boundary

- external diagnostics contract != verification authority
- external diagnostics contract != replay admission authority
- external diagnostics contract != baseline authority

This contract only defines the read-only public diagnostics surface.
