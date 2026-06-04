# Phase-18 Platform ABI Validation Gate

This document is subordinate to PHASE 0 - FOUNDATIONAL OATH,
`ARCHITECTURE_FREEZE.md`, `PHASE18_TRANSITION_DECISION.md`,
`MODULE_MANIFEST_SCHEMA.md`, `CAPABILITY_CONTRACT_SPECIFICATION.md`,
`WORKSPACE_LIFECYCLE_SPECIFICATION.md`, `PACKAGE_METADATA_SCHEMA.md`,
`TRUST_CLASSIFICATION_MODEL.md`, and `PLUGIN_BOUNDARY_CONTRACT.md`. In case of
conflict, those documents prevail.

**Status:** RFC-DRAFT / PRE-ACTIVATION SPEC
**Contract id:** `ayken.platform.abi.validation.gate.v1`
**Authority boundary:** Documentation/specification only; not a validator
implementation, installer, registry, runtime loader, capability grant, trust
grant, workspace admission, mount authority, plugin host, execution right,
semantic verdict source, AI Runtime authority, kernel ABI expansion, or
Phase-18 activation.

## Purpose

The Platform ABI Validation Gate answers the seventh Phase-18 Platform
Constitution question:

How does the platform evaluate manifest, package, trust, capability,
workspace, and plugin boundary inputs in a deterministic fail-closed order
without turning validation into runtime authority?

This RFC defines the fail-closed validation order, input bundle shape, stage
result shape, validation receipt shape, and rejection rules for the Platform ABI
layer above the frozen kernel execution substrate.

It does not implement a validator. It does not install, enable, load, execute,
mount, authorize, or trust anything by itself.

## Core Rule

**Validation is a denial boundary, not an authority grant.**

The following invariants are mandatory:

1. A validation PASS must never grant capability.
2. A validation PASS must never assign trust.
3. A validation PASS must never create workspace mounts.
4. A validation PASS must never install, enable, load, or execute a module.
5. A validation PASS must never load or auto-activate a plugin.
6. A validation PASS must never create bearer tokens or runtime handles.
7. A validation PASS must never bypass package, manifest, trust, capability,
   workspace, or plugin checks.
8. A validation PASS must never bypass the frozen kernel ABI.
9. A validation PASS must never use Semantic CLI or AI Runtime output as
   authority.
10. Unknown stages, inputs, states, verdicts, effects, or fields fail closed.

## Positive Scope

Version 1 Platform ABI validation is limited to:

1. Validation vocabulary.
2. Validation input bundle shape.
3. Deterministic validation stage order.
4. Stage result records.
5. Validation receipt records.
6. Validation lifecycle states.
7. Bounded policy effects.
8. Fail-closed rejection rules.
9. Cross-contract separation checks.

No runtime authority is expressible in this RFC.

## Non-Goals

This gate does not define:

1. A new syscall.
2. Kernel ABI expansion.
3. Ring0 policy.
4. A validator binary or runtime implementation.
5. A package installer.
6. A package registry.
7. A runtime loader.
8. Capability token issuance.
9. Trust root implementation.
10. Workspace mount implementation.
11. Plugin host execution.
12. Semantic CLI execution verdict authority.
13. AI Runtime authority.

## Terms

| Term | Meaning | Authority boundary |
|---|---|---|
| Validation gate | Deterministic evaluation boundary for Platform ABI inputs | Not an installer or loader |
| Validation input bundle | Digest-bound set of manifest, package, trust, capability, workspace, and plugin references | Not a runtime request |
| Validation stage | Ordered check against one contract boundary | Not an authority layer |
| Stage result | Evidence that one validation stage passed, failed, or was blocked | Not a bearer token |
| Validation receipt | Digest-bound record of the full gate result | Not a capability receipt |
| Validation PASS | Input set is internally consistent for later review | Not install, enable, execution, or trust |
| Validation FAIL | Input set is rejected or blocked | Must stop evaluation |

## Required Validation Order

The gate must evaluate Platform ABI inputs in this order:

```text
0. Kernel Freeze Guard
1. Manifest Validation
2. Package Metadata Validation
3. Package-Manifest Binding
4. Trust Classification Validation
5. Capability Contract Validation
6. Workspace Lifecycle Validation
7. Plugin Boundary Validation
8. Cross-Contract Separation
9. Validation Receipt Emission
```

The rule is strict:

```text
FAIL => STOP => DENY => FAIL-CLOSED
```

A later stage must not run as authoritative validation after an earlier stage
fails. A later stage may emit `blocked_by_prior_failure` evidence only.

## Stage 0: Kernel Freeze Guard

The gate starts by preserving the existing frozen mechanism boundary.

Required checks:

1. The Kernel ABI remains `1000-1011` / 12 syscall / syscall v2.
2. No manifest, package, trust, capability, workspace, plugin, or validation
   record declares kernel ABI expansion.
3. No input declares `kernel`, `ring0`, `syscall`, `driver`, `scheduler`, or
   `interrupt` authority.
4. No input treats a Platform ABI PASS as a kernel verdict.
5. No input treats Semantic CLI or AI Runtime output as execution authority.

If any input requires new syscalls, Ring0 policy, kernel loader behavior, or
kernel ABI expansion, the gate must fail before manifest validation.

## Stage 1: Manifest Validation

The manifest stage validates `MODULE_MANIFEST_SCHEMA.md`.

Required checks:

1. `schema_id` is `ayken.platform.module.manifest.v1`.
2. Required manifest fields are present.
3. Unknown top-level fields are absent.
4. Duplicate JSON keys are absent.
5. `module_id`, `version`, `module_kind`, entrypoints, platform declaration,
   capability requests, workspace declaration, integrity entries, plugin
   boundary, semantic surface, and extensions obey their RFC rules.
6. Manifest digest is computed externally by the gate or a subordinate
   canonicalization input.
7. The manifest does not self-declare trust, grants, tokens, mounts, loaders,
   execution verdicts, Semantic CLI authority, AI authority, or kernel
   authority.

Manifest validation PASS means only that the manifest is structurally valid.
It does not authorize execution or access.

## Stage 2: Package Metadata Validation

The package stage validates `PACKAGE_METADATA_SCHEMA.md` when package metadata
is present.

Required checks:

1. `schema_id` is `ayken.platform.package.metadata.v1`.
2. Required package metadata fields are present.
3. Unknown top-level fields are absent.
4. Duplicate JSON keys are absent.
5. Package identity, version, publisher declaration, hashes, signature
   references, dependencies, and Platform ABI compatibility obey their RFC
   rules.
6. Package digest is computed externally by the gate or a subordinate
   canonicalization input.
7. Package metadata does not self-declare package digest, trust, grants,
   tokens, workspace mounts, loaders, execution rights, plugin loading,
   Semantic CLI authority, AI authority, or kernel authority.

Package metadata may be absent only when the validation policy explicitly
allows a manifest-only local subject. Unknown absence policy fails closed.

## Stage 3: Package-Manifest Binding

The binding stage checks that package metadata and module manifest inputs refer
to the same exact subject.

Required checks:

1. Manifest subject id and package subject id are explicitly related by policy.
2. Manifest version and package version compatibility is explicit.
3. Package hash declarations cover the manifest path or manifest digest input.
4. Manifest artifact hashes are consistent with package hash evidence when
   both are present.
5. Dependency declarations do not bypass capability, workspace, trust, plugin,
   or runtime review.
6. No package can bind to an unverifiable or stale manifest digest.

If package metadata and manifest evidence cannot be bound to the same exact
subject, validation must fail closed.

## Stage 4: Trust Classification Validation

The trust stage validates `TRUST_CLASSIFICATION_MODEL.md` records that are
provided as policy inputs.

Required checks:

1. Each record uses contract id `ayken.platform.trust.classification.v1`.
2. Subject kind, subject id, subject version, and subject digest match the
   manifest, package, publisher, signature, or distribution subject being
   evaluated.
3. Classification value is known.
4. Lifecycle state is known and usable by policy.
5. Evidence references are immutable references or digests.
6. Expired, quarantined, revoked, rejected, or superseded classifications do
   not pass as active inputs.
7. Trust is not treated as capability, install, enable, execution, workspace,
   plugin loading, Semantic CLI, AI, or kernel authority.

Trust records are optional only when policy explicitly accepts an
unclassified subject. Unknown trust policy fails closed.

## Stage 5: Capability Contract Validation

The capability stage validates capability request, decision, receipt, and
revocation boundaries from `CAPABILITY_CONTRACT_SPECIFICATION.md`.

Required checks:

1. Manifest `capability_requests` are request-only.
2. Capability ids, access verbs, scopes, and required flags obey the
   capability contract.
3. External capability decision records, when present, are digest-bound to the
   exact module subject.
4. Receipts are evidence of decisions only and are not bearer tokens.
5. Revocation epoch does not regress.
6. Required capability denial blocks later enablement review.
7. Optional capability denial is recorded and does not silently widen scope.
8. Trust classification does not grant or widen capability.
9. Capability decisions do not require kernel ABI expansion.

Capability validation PASS does not issue runtime tokens. A future runtime
binding must remain separate and subordinate to the frozen kernel ABI.

## Stage 6: Workspace Lifecycle Validation

The workspace stage validates `WORKSPACE_LIFECYCLE_SPECIFICATION.md` inputs.

Required checks:

1. Manifest workspace declaration is declarative only.
2. Workspace admission records, logical mount records, disable records,
   quarantine records, revocation records, and removal records obey the
   workspace lifecycle contract when present.
3. Workspace subject digests match the module/package subject being evaluated.
4. Logical mounts are policy records only and are not kernel mounts.
5. Workspace records do not grant capability by themselves.
6. Workspace state does not bypass package, manifest, trust, or capability
   validation.
7. Quarantined, revoked, disabled, or removed workspace states block later
   enablement review as required by policy.

Workspace validation PASS does not create mounts or filesystem access.

## Stage 7: Plugin Boundary Validation

The plugin stage validates `PLUGIN_BOUNDARY_CONTRACT.md` inputs when a manifest
declares `plugin_boundary`.

Required checks:

1. `plugin_boundary.host_interfaces` and `plugin_boundary.exports` contain
   only allowed fields.
2. Host interface request ids, interface ids, interface versions, extension
   point ids, and host selectors obey the plugin boundary contract.
3. Export ids, kinds, interface ids, versions, admission policies,
   multiplicity, and stability obey the plugin boundary contract.
4. Plugin binding decision records, when present, are digest-bound to the
   exact host and plugin subjects.
5. Host and plugin trust are evaluated independently.
6. Plugin compatibility does not grant capability.
7. Plugin compatibility does not create workspace mounts.
8. Plugin compatibility does not load, auto-load, enable, or execute a plugin.
9. Plugin compatibility does not use Semantic CLI or AI Runtime output as
   authority.

Plugin validation PASS means only that plugin boundary inputs are structurally
compatible for later review.

## Stage 8: Cross-Contract Separation

The separation stage checks that no contract is smuggling authority from
another contract.

The gate must reject any input set where:

1. Manifest grants capability.
2. Package metadata declares trust.
3. Trust grants capability.
4. Trust installs, enables, loads, or executes.
5. Capability creates workspace mounts.
6. Workspace grants capability.
7. Plugin inherits host trust.
8. Plugin inherits host capability.
9. Plugin creates workspace authority.
10. Package dependencies imply install, enable, or execution.
11. Semantic metadata creates execution verdict authority.
12. AI output creates execution, trust, capability, workspace, or plugin
    authority.
13. Any Platform ABI input requires new syscalls or kernel ABI expansion.

This stage is mandatory even when all earlier stages pass.

## Stage 9: Validation Receipt Emission

The receipt stage records the final gate outcome.

Allowed final verdicts:

| Verdict | Meaning | Authority boundary |
|---|---|---|
| `validated` | All required gate stages passed for the exact input bundle | Not install, enable, execution, trust, capability, mount, or plugin load |
| `rejected` | One or more required stages failed | Deny by default |
| `blocked` | Validation could not continue because a prior required input was missing or failed | Deny by default |
| `quarantined` | Policy requires quarantine before further review | Not execution authority |
| `stale` | One or more evidence references are stale, expired, superseded, or digest-mismatched | Deny by default |

Unknown verdicts fail closed.

## Validation Input Bundle

A validation input bundle must be external to manifest and package metadata.

Required fields:

| Field | Type | Requirement |
|---|---|---|
| `contract_id` | string | Must be `ayken.platform.abi.validation.gate.v1` |
| `bundle_id` | string | Stable validation bundle id |
| `subject_kind` | string | `module`, `package`, `module_package`, or `plugin_binding` |
| `subject_id` | string | Stable subject id |
| `subject_version` | string | Subject version |
| `manifest_ref` | object | Manifest evidence reference or `none` |
| `package_ref` | object | Package evidence reference or `none` |
| `trust_refs` | array | Trust classification references; may be empty only by policy |
| `capability_refs` | array | Capability decision/receipt references; may be empty |
| `workspace_refs` | array | Workspace lifecycle references; may be empty |
| `plugin_refs` | array | Plugin boundary/binding references; may be empty |
| `policy_digest` | string | Digest of validation policy inputs |
| `created_at` | string | Bundle creation timestamp |
| `validation_epoch` | integer | Monotonic validation epoch |

Unknown fields in a validation input bundle fail validation.

Input bundles must not contain:

1. Raw secrets.
2. Private keys.
3. Bearer tokens.
4. Kernel handles.
5. Runtime handles.
6. Loader handles.
7. Raw syscall arguments.
8. Workspace mount handles.
9. Capability grants embedded outside the capability contract.
10. Trust assignments embedded outside the trust contract.
11. Execution verdicts.
12. AI or semantic authority claims.

## Evidence Reference Shape

Every evidence reference object must include:

| Field | Type | Requirement |
|---|---|---|
| `kind` | string | Known evidence kind |
| `ref` | string | Stable external reference |
| `digest` | string | 64-character lower-case hex SHA-256 or `none` |

Allowed evidence kinds:

1. `module_manifest`
2. `package_metadata`
3. `trust_classification`
4. `capability_decision`
5. `capability_receipt`
6. `capability_revocation`
7. `workspace_lifecycle`
8. `plugin_boundary`
9. `plugin_binding_decision`
10. `review_policy`
11. `validation_policy`

Unknown evidence kinds fail closed.

## Stage Result Record

Each validation stage must produce a stage result record.

Required fields:

| Field | Type | Requirement |
|---|---|---|
| `contract_id` | string | Must be `ayken.platform.abi.validation.gate.v1` |
| `stage_id` | string | Known stage id |
| `stage_index` | integer | Stage index from `0` through `9` |
| `status` | string | `pass`, `fail`, `blocked`, or `skipped` |
| `reason_code` | string | Known reason code |
| `input_digests` | array | Digests evaluated by the stage |
| `evidence_refs` | array | Evidence references used by the stage |
| `issued_at` | string | Stage result timestamp |

Unknown fields in a stage result fail validation.

Allowed stage ids:

1. `kernel_freeze_guard`
2. `manifest_validation`
3. `package_metadata_validation`
4. `package_manifest_binding`
5. `trust_classification_validation`
6. `capability_contract_validation`
7. `workspace_lifecycle_validation`
8. `plugin_boundary_validation`
9. `cross_contract_separation`
10. `validation_receipt_emission`

Allowed reason codes:

1. `ok`
2. `missing_input`
3. `unknown_field`
4. `duplicate_key`
5. `digest_mismatch`
6. `stale_evidence`
7. `policy_missing`
8. `policy_denied`
9. `forbidden_authority`
10. `kernel_abi_violation`
11. `semantic_authority_violation`
12. `ai_authority_violation`
13. `blocked_by_prior_failure`
14. `quarantine_required`
15. `revocation_required`

Unknown reason codes fail closed.

## Validation Receipt Record

A validation receipt must be external to manifests, package metadata, trust
records, capability records, workspace records, and plugin records.

Required fields:

| Field | Type | Requirement |
|---|---|---|
| `contract_id` | string | Must be `ayken.platform.abi.validation.gate.v1` |
| `bundle_id` | string | Input bundle id |
| `subject_kind` | string | Subject kind from the bundle |
| `subject_id` | string | Subject id |
| `subject_version` | string | Subject version |
| `subject_digest` | string | 64-character lower-case hex SHA-256 or `none` |
| `verdict` | string | Allowed final verdict |
| `state` | string | Validation lifecycle state |
| `stage_results` | array | Ordered stage result references or embedded records |
| `evidence_refs` | array | External evidence references |
| `policy_digest` | string | Digest of validation policy inputs |
| `issued_at` | string | Receipt creation timestamp |
| `expires_at` | string | Timestamp or `never` |
| `validation_epoch` | integer | Monotonic validation epoch |
| `revocation_epoch` | integer | Monotonic revocation epoch |

Unknown fields in a validation receipt fail validation.

Validation receipts must not include:

1. Capability grants.
2. Capability tokens.
3. Trust grants.
4. Runtime bearer tokens.
5. Kernel handles.
6. Raw syscall arguments.
7. Workspace mounts.
8. Loader handles.
9. Plugin runtime handles.
10. Execution verdicts.
11. AI or semantic authority claims.

## Validation Lifecycle States

Allowed validation lifecycle states:

```text
unvalidated -> candidate -> validated
candidate -> rejected
candidate -> blocked
candidate -> quarantined
validated -> superseded
validated -> quarantined
validated -> revoked
quarantined -> candidate
quarantined -> revoked
rejected -> tombstoned
blocked -> tombstoned
revoked -> tombstoned
superseded -> tombstoned
```

State meanings:

| State | Meaning |
|---|---|
| `unvalidated` | No accepted validation receipt exists |
| `candidate` | Input bundle is under validation |
| `validated` | Gate checks passed and may be used as a policy input |
| `rejected` | Gate checks failed |
| `blocked` | Validation could not continue due to missing or failed prior input |
| `quarantined` | Policy requires quarantine before further review |
| `superseded` | Receipt was replaced by a newer validation record |
| `revoked` | Receipt is withdrawn and denies by default |
| `tombstoned` | Historical terminal record retained for evidence |

Unknown states fail closed.

`validated` is not an active runtime state. It is a policy input only.

## Policy Effects

Validation gate results may affect only bounded policy effects:

| Effect | Meaning |
|---|---|
| `allow_next_review` | Subject may enter a later review workflow |
| `require_manual_review` | Manual review is required before later policy use |
| `deny_next_review` | Subject must not proceed to later review |
| `require_quarantine` | Subject must be quarantined before later review |
| `trigger_revocation_review` | Related capability/workspace/plugin decisions must be reviewed |
| `record_evidence` | Store the validation receipt as evidence |

These effects are policy gates, not authority grants.

Forbidden effects include:

1. `grant_capability`
2. `grant_token`
3. `assign_trust`
4. `inherit_trust`
5. `widen_scope`
6. `create_mount`
7. `install`
8. `enable`
9. `execute`
10. `autoload`
11. `load_plugin`
12. `issue_runtime_handle`
13. `bypass_review`
14. `bypass_kernel_abi`
15. `semantic_verdict`
16. `ai_verdict`

## Valid Minimal Receipt Example

```json
{
  "contract_id": "ayken.platform.abi.validation.gate.v1",
  "bundle_id": "validation:org.ayken.examples.echo:1.0.0",
  "subject_kind": "module_package",
  "subject_id": "org.ayken.examples.echo",
  "subject_version": "1.0.0",
  "subject_digest": "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
  "verdict": "validated",
  "state": "validated",
  "stage_results": [
    {
      "contract_id": "ayken.platform.abi.validation.gate.v1",
      "stage_id": "kernel_freeze_guard",
      "stage_index": 0,
      "status": "pass",
      "reason_code": "ok",
      "input_digests": [
        "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
      ],
      "evidence_refs": [],
      "issued_at": "2026-06-04T00:00:00Z"
    }
  ],
  "evidence_refs": [
    {
      "kind": "module_manifest",
      "ref": "manifest:org.ayken.examples.echo:1.0.0",
      "digest": "abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789"
    }
  ],
  "policy_digest": "1111111111111111111111111111111111111111111111111111111111111111",
  "issued_at": "2026-06-04T00:00:00Z",
  "expires_at": "never",
  "validation_epoch": 1,
  "revocation_epoch": 0
}
```

This example records validation evidence only. It does not install, enable,
execute, mount, trust-grant, capability-grant, load a plugin, or authorize a
module.

## Invalid Examples

### Validation Grants Capability

```json
{
  "verdict": "validated",
  "grant_capability": "ayken.platform.workspace.write"
}
```

Invalid because validation cannot grant capability.

### Validation Assigns Trust

```json
{
  "verdict": "validated",
  "classification": "trusted"
}
```

Invalid because trust classification must remain external to the validation
receipt.

### Validation Creates Workspace Mount

```json
{
  "verdict": "validated",
  "mount": "/workspace/project"
}
```

Invalid because validation cannot create workspace mounts.

### Plugin Autoload From Validation

```json
{
  "verdict": "validated",
  "load_plugin": true
}
```

Invalid because validation cannot load or auto-activate plugins.

### AI Validation Verdict

```json
{
  "verdict": "validated",
  "ai_verdict": "safe"
}
```

Invalid because AI Runtime is not a Phase-18 authority source.

## Fail-Closed Validation Matrix

| Condition | Required result |
|---|---|
| Missing validation input bundle | Reject |
| Unknown bundle field | Reject |
| Unknown evidence kind | Reject |
| Unknown stage id | Reject |
| Unknown stage result status | Reject |
| Unknown reason code | Reject |
| Unknown final verdict | Reject |
| Unknown lifecycle state | Reject |
| Duplicate JSON key in any input | Reject |
| Manifest validation fails | Stop and reject |
| Package metadata validation fails | Stop and reject |
| Package-manifest binding fails | Stop and reject |
| Trust classification record is expired, revoked, quarantined, rejected, or stale | Deny policy use |
| Capability decision digest does not match subject | Reject |
| Required capability denied | Block later enablement review |
| Workspace state is revoked, removed, disabled, or quarantined | Block later enablement review |
| Plugin binding is revoked, rejected, blocked, or stale | Block plugin binding review |
| Any digest mismatch | Reject |
| Any stale evidence reference | Reject |
| Any embedded bearer token | Reject |
| Any embedded kernel or runtime handle | Reject |
| Any embedded raw syscall argument | Reject |
| Validation treated as capability | Deny |
| Validation assigns trust | Deny |
| Validation creates workspace mount | Deny |
| Validation installs, enables, loads, or executes | Deny |
| Validation loads or autoloads plugin | Deny |
| Validation bypasses package, manifest, trust, capability, workspace, or plugin checks | Deny |
| Validation bypasses kernel ABI boundary | Deny |
| Semantic or AI verdict used as authority | Deny |

## Relationship To Existing Phase-18 Specs

1. `MODULE_MANIFEST_SCHEMA.md` provides declarative manifest input. The gate
   validates it but does not execute it.
2. `PACKAGE_METADATA_SCHEMA.md` provides package evidence input. The gate
   validates it but does not install it.
3. `TRUST_CLASSIFICATION_MODEL.md` provides trust policy input. The gate
   validates it but does not assign trust.
4. `CAPABILITY_CONTRACT_SPECIFICATION.md` provides capability request,
   decision, receipt, and revocation boundaries. The gate validates them but
   does not issue tokens.
5. `WORKSPACE_LIFECYCLE_SPECIFICATION.md` provides workspace lifecycle input.
   The gate validates it but does not create mounts.
6. `PLUGIN_BOUNDARY_CONTRACT.md` provides plugin boundary compatibility input.
   The gate validates it but does not load plugins.

## Activation Boundary

This RFC is not sufficient to activate Phase-18. Phase-18 activation still
requires an explicit `CURRENT_PHASE` pointer transition and reviewed acceptance
of the required Platform Constitution set.

This RFC does not authorize implementation work that widens the kernel ABI,
adds syscalls, places policy in Ring0, creates runtime loaders, admits
workspaces, grants capabilities, loads plugins, installs packages, or makes
semantic/AI systems execution authority.
