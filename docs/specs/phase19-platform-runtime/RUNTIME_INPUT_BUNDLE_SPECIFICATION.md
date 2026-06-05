# Phase-19 Runtime Input Bundle Specification

This document is subordinate to `PHASE19_RUNTIME_DECISION.md`,
`RUNTIME_LIFECYCLE_SPECIFICATION.md`, and the Phase-18 Platform Constitution
reference set. In case of conflict, those documents prevail.

**Status:** PRE-IMPLEMENTATION RFC / PHASE-19 NOT ACTIVE / RUNTIME NOT AUTHORIZED
**Contract id:** `ayken.phase19.runtime.input_bundle.v1`
**Authority boundary:** Documentation/specification only; not a parser,
installer, loader, workspace creator, mount engine, plugin host, issuer,
trust assignment, Semantic CLI authority, AI Runtime authority, syscall, or
kernel ABI expansion.

## Purpose

This RFC defines the static input bundle shape for the first possible
Phase-19 Runtime MVP.

The bundle is a deterministic collection of references to Phase-18
constitutional inputs. It is not a runtime request and does not authorize any
action by itself.

## Core Rule

```text
Input bundle != execution request
```

An input bundle may be validated and recorded. It must not install, load,
mount, execute, issue, trust, publish, or schedule anything.

## Bundle Shape

Version 1 bundles are declarative records with these required fields:

| Field | Required | Meaning | Authority boundary |
|---|---|---|---|
| `schema_id` | yes | Must be `ayken.phase19.runtime.input_bundle.v1` | Not runtime authority |
| `bundle_id` | yes | Stable bundle identifier | Not package id authority |
| `bundle_version` | yes | Bundle schema version | Not module version authority |
| `subject` | yes | Exact subject being evaluated | Not install target |
| `manifest_ref` | yes | Digest-bound manifest reference | Not parser authority |
| `package_ref` | optional | Digest-bound package metadata reference | Not installer authority |
| `platform_validation_policy_ref` | yes | Validation policy reference | Not policy execution |
| `workspace_admission_request` | yes | Declarative workspace admission request | Not workspace creation |
| `expected_receipt_profile` | yes | Receipt profile name and version | Not token request |
| `evidence_refs` | optional | Immutable input evidence references | Not authority inheritance |

Unknown fields fail closed.

## Subject Binding

The `subject` field must bind:

1. Subject kind.
2. Subject id.
3. Subject version.
4. Subject digest.
5. Optional publisher reference.
6. Optional package reference.

The subject must match all referenced Phase-18 documents. If manifest,
package, validation, workspace, or evidence references disagree, the bundle
must be rejected.

## Reference Rules

All references must be immutable or digest-bound.

Required reference properties:

1. Stable path or URI.
2. Digest algorithm.
3. Digest value.
4. Declared contract id.
5. Expected schema version.

Mutable references, branch names, unpinned URLs, ambient local paths, or
implicit latest values fail closed.

## Canonicalization Rules

A later implementation must define a canonical serialization before computing
bundle digests.

Until then, this RFC requires only these constraints:

1. Duplicate keys are invalid.
2. Unknown top-level fields are invalid.
3. Unknown enum values are invalid.
4. Missing required references are invalid.
5. Non-deterministic ordering is invalid unless canonicalized before use.
6. Wall-clock timestamps are not authoritative.

## Negative Cases

The bundle must be denied if it:

1. Requests installation.
2. Requests execution.
3. Requests module loading.
4. Requests plugin loading.
5. Requests real filesystem mounts.
6. Requests capability issuance.
7. Requests trust assignment.
8. Treats validation PASS as authority.
9. Treats a receipt as a bearer token.
10. Requires new syscalls or kernel ABI expansion.

## Acceptance Boundary

This RFC does not implement bundle parsing. A later implementation must prove
deterministic positive and negative bundle handling before Phase-19 runtime
activation can be considered.
