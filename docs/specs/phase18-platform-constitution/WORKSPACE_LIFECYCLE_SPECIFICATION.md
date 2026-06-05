# Phase-18 Workspace Lifecycle Specification

This document is subordinate to PHASE 0 - FOUNDATIONAL OATH,
`ARCHITECTURE_FREEZE.md`, `PHASE18_TRANSITION_DECISION.md`,
`MODULE_MANIFEST_SCHEMA.md`, and `CAPABILITY_CONTRACT_SPECIFICATION.md`. In
case of conflict, those documents prevail.

**Status:** ACTIVE CONSTITUTION SPEC / RUNTIME NOT AUTHORIZED
**Contract id:** `ayken.platform.workspace.lifecycle.v1`
**Authority boundary:** Documentation/specification only; not an installer,
runtime mount implementation, capability grant, trust grant, kernel mapping
contract, or Phase-18 activation.

## Purpose

The workspace lifecycle contract answers the third Phase-18 Platform
Constitution question:

How does a module declare workspace dependency, become installable and
enableable inside a workspace, receive bounded logical workspace surfaces, and
remain disableable/removable without granting authority by declaration?

The module manifest declares `workspace`. The capability contract defines
whether requested workspace access may be authorized. This specification
defines the workspace states and transitions that must happen around those
contracts.

## Non-Goals

This contract does not define:

1. A new syscall.
2. Kernel filesystem or VFS policy.
3. Runtime mount implementation.
4. Package distribution format.
5. Capability token issuance.
6. Trust classification policy.
7. Plugin host execution.
8. Semantic CLI execution verdict authority.
9. AI Runtime authority.

## Core Rule

A workspace declaration is not a mount. A mount declaration is not capability.
A capability decision is not a mount. A trust level is not workspace authority.

The following invariants are mandatory:

1. Workspace lifecycle state is external to the manifest.
2. A manifest may declare workspace dependency only.
3. A module must not create workspace mounts by declaration.
4. A module must not receive workspace access without an approved capability
   decision.
5. Workspace selectors must be bounded and reviewable.
6. Unknown lifecycle states fail closed.
7. Disable, quarantine, revocation, and removal must deny future access by
   default.
8. Workspace lifecycle must not require kernel ABI expansion.

## Relationship To Existing Contracts

| Contract | Relationship |
|---|---|
| `MODULE_MANIFEST_SCHEMA.md` | Declares `workspace.mode` and `workspace.declared_mounts` |
| `CAPABILITY_CONTRACT_SPECIFICATION.md` | Authorizes or denies requested workspace capability access |
| `PACKAGE_METADATA_SCHEMA.md` | Declares package identity, content hashes, signatures, dependencies, and compatibility only; no workspace admission or mount fields |
| `TRUST_CLASSIFICATION_MODEL.md` | May influence review path, not workspace authority |
| `PLUGIN_BOUNDARY_CONTRACT.md` | May reference workspace-derived host surfaces later; cannot create mounts or workspace authority |
| Platform ABI Validation Gate | Must reject stale, ambiguous, or authority-expanding workspace states |

Workspace lifecycle is the admission state machine around a module in a
workspace. It is not a direct execution, loader, filesystem, or kernel mapping
surface.

## Workspace Declaration In Manifest

The module manifest `workspace` object remains declarative:

| Field | Type | Requirement |
|---|---|---|
| `mode` | string | `none`, `optional`, or `required` |
| `declared_mounts` | array | Requested logical mount declarations; may be empty |

Unknown fields in `workspace` fail validation.

### `mode`

| Value | Meaning |
|---|---|
| `none` | Module does not require workspace lifecycle admission |
| `optional` | Module may enable without workspace surfaces, but dependent paths remain disabled |
| `required` | Module cannot enable unless workspace admission succeeds |

If `mode` is `none`, `declared_mounts` must be empty.

If `mode` is `required`, at least one workspace capability request must exist
unless a future registry marks the module kind as workspace-admitted without
resource access. The safe default is rejection.

### `declared_mounts`

Each declared mount is a request for a logical workspace surface. It does not
create a mount.

| Field | Type | Requirement |
|---|---|---|
| `id` | string | Mount declaration id unique within the module |
| `kind` | string | Mount kind |
| `selector` | string | Workspace selector |
| `access` | array | Requested access verbs |
| `required` | boolean | Whether missing mount blocks enablement |
| `reason` | string | Human-readable justification |

Unknown fields in a declared mount fail validation.

Initial mount kinds:

| Kind | Meaning |
|---|---|
| `workspace_view` | Logical view into a workspace-defined resource set |
| `workspace_cache` | Module-local cache namespace inside workspace policy |
| `workspace_output` | Module output namespace inside workspace policy |
| `workspace_config` | Workspace-scoped configuration surface |

Forbidden mount kinds include `kernel`, `ring0`, `syscall`, `driver`,
`device`, `root`, `authority`, `trust`, `token`, `semantic-verdict`, and
`ai-runtime`.

The selector `*` is forbidden. Absolute paths, parent-relative paths, raw host
paths, device paths, and kernel paths are forbidden.

## Workspace Lifecycle States

The lifecycle state machine is:

```text
absent -> discovered -> installed -> admitted -> enabled
enabled -> disabled -> enabled
enabled -> quarantined
enabled -> revoked
disabled -> removed
quarantined -> disabled
quarantined -> removed
revoked -> removed
installed -> removed
admitted -> removed
```

State meanings:

| State | Meaning |
|---|---|
| `absent` | Module is not known to the workspace |
| `discovered` | Manifest/package candidate is visible, not installed |
| `installed` | Package artifacts are present, not workspace-admitted |
| `admitted` | Manifest, package, trust, and capability decisions allow workspace admission |
| `enabled` | Module is allowed to expose declared entrypoints in this workspace |
| `disabled` | Module remains installed but entrypoints and mounts are inactive |
| `quarantined` | Module is isolated pending review; enablement denied |
| `revoked` | Workspace authority has been withdrawn; future access denied |
| `removed` | Module artifacts and workspace lifecycle records are removed or tombstoned |

Unknown states fail closed.

## Transition Requirements

### Discover

`absent -> discovered` requires:

1. Candidate manifest is parseable.
2. Unknown manifest fields are rejected.
3. No capability, trust, semantic, AI, or kernel authority is claimed.

Discovery does not install or enable the module.

### Install

`discovered -> installed` requires:

1. Manifest validation PASS.
2. Package metadata validation PASS when package metadata exists.
3. Artifact integrity input available.
4. No capability decision embedded in the manifest.
5. No trust self-declaration embedded in the manifest.

Install does not enable entrypoints and does not create mounts.

### Admit

`installed -> admitted` requires:

1. Workspace declaration validation PASS.
2. Capability request validation PASS.
3. Required workspace capability decisions are approved.
4. Effective scopes are equal to or narrower than requested scopes.
5. Trust classification permits the review path.
6. Workspace policy accepts the package and module subject.

Admission does not mint runtime tokens and does not execute the module.

### Enable

`admitted -> enabled` requires:

1. Current manifest digest matches the admitted subject.
2. Current package digest matches the admitted subject when package metadata
   exists.
3. Required capability decisions are not expired, suspended, quarantined, or
   revoked.
4. Workspace policy is still current.
5. Declared mount ids resolve to bounded logical surfaces.

Enablement allows future platform runtime to expose entrypoints and logical
workspace surfaces. It does not bypass capability checks.

### Disable

`enabled -> disabled` requires:

1. Future access is denied by default.
2. Logical mounts are inactive.
3. Runtime bindings, if any exist in a later phase, are stopped or marked
   inactive.
4. Receipts remain historical evidence only.

Disablement must be reversible only through a fresh enable check.

### Quarantine

`enabled -> quarantined` or `admitted -> quarantined` may be triggered by:

1. Trust downgrade.
2. Policy conflict.
3. Evidence invalidation.
4. Package digest mismatch.
5. Manifest digest mismatch.
6. Capability registry withdrawal.
7. Manual review action.

Quarantine denies enablement and future access. Quarantine does not prove a
capability violation by itself.

### Revoke

`enabled -> revoked` or `admitted -> revoked` requires:

1. Capability decisions are no longer usable.
2. Future workspace access is denied.
3. Runtime bindings, if any exist in a later phase, must be revoked by that
   runtime layer.
4. Receipts remain historical evidence only.

Revocation must fail closed if cleanup status is unknown.

### Remove

`disabled -> removed`, `quarantined -> removed`, `revoked -> removed`,
`installed -> removed`, or `admitted -> removed` requires:

1. Entry points are inactive.
2. Logical mounts are inactive.
3. Future capability evaluation for the removed subject denies by default.
4. Workspace-local module state is deleted or tombstoned according to package
   policy.
5. Removal receipt records what was removed without creating new authority.

Removal must not remove unrelated modules, packages, workspace data, or
evidence records.

## Workspace Admission Record

A workspace admission record is external to the manifest. It may be stored by a
future workspace or package layer.

Required fields:

| Field | Type | Requirement |
|---|---|---|
| `contract_id` | string | Must be `ayken.platform.workspace.lifecycle.v1` |
| `workspace_id` | string | Workspace subject |
| `module_id` | string | Module subject |
| `module_version` | string | Module version subject |
| `manifest_digest` | string | Canonical manifest digest |
| `package_digest` | string | Package digest or `none` |
| `state` | string | Lifecycle state |
| `capability_decision_refs` | array | References to external capability decisions |
| `policy_digest` | string | Workspace policy input digest |
| `trust_ref` | string | External trust classification reference or `none` |
| `revocation_epoch` | integer | Monotonic revocation epoch |

The admission record must not include:

1. Runtime bearer tokens.
2. Kernel handles.
3. Raw syscall arguments.
4. Trust self-declarations.
5. Semantic or AI execution verdicts.

## Logical Mount Record

A logical mount record is external to the manifest and subordinate to workspace
admission.

Required fields:

| Field | Type | Requirement |
|---|---|---|
| `mount_id` | string | Stable workspace-local mount id |
| `module_id` | string | Module subject |
| `declaration_id` | string | Matching manifest declared mount id |
| `kind` | string | Accepted mount kind |
| `selector` | string | Accepted bounded selector |
| `effective_access` | array | Access verbs authorized for this mount |
| `capability_decision_ref` | string | Matching approved capability decision |
| `state` | string | `inactive`, `active`, `suspended`, or `revoked` |

`effective_access` must be equal to or narrower than both the declared mount
access and the approved capability decision. If this cannot be proven, the
mount remains inactive.

Logical mount records must not include raw host paths, kernel paths, device
paths, or bearer tokens.

## Valid Manifest Workspace Example

```json
{
  "mode": "required",
  "declared_mounts": [
    {
      "id": "input-documents",
      "kind": "workspace_view",
      "selector": "current.project.documents",
      "access": ["read"],
      "required": true,
      "reason": "Read selected project documents for the module entrypoint."
    }
  ]
}
```

This example declares dependency only. It does not create a mount and does not
grant workspace access.

## Valid Admission Example

```json
{
  "contract_id": "ayken.platform.workspace.lifecycle.v1",
  "workspace_id": "local.workspace.current",
  "module_id": "org.ayken.examples.echo",
  "module_version": "1.0.0",
  "manifest_digest": "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
  "package_digest": "none",
  "state": "admitted",
  "capability_decision_refs": [
    "capability-decision:workspace-read:0"
  ],
  "policy_digest": "abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789",
  "trust_ref": "none",
  "revocation_epoch": 0
}
```

This example records admission only. It is not runtime authority.

## Invalid Examples

### Direct Mount Grant

```json
{
  "mode": "required",
  "declared_mounts": [
    {
      "id": "all-files",
      "kind": "workspace_view",
      "selector": "*",
      "access": ["read", "write"],
      "grant": true,
      "required": true,
      "reason": "Need everything."
    }
  ]
}
```

Invalid because wildcard selector and `grant` attempt to create authority.

### Raw Host Path

```json
{
  "mode": "required",
  "declared_mounts": [
    {
      "id": "host-root",
      "kind": "workspace_view",
      "selector": "/",
      "access": ["read"],
      "required": true,
      "reason": "Read host root."
    }
  ]
}
```

Invalid because raw host paths are forbidden.

### Kernel Workspace

```json
{
  "mode": "required",
  "declared_mounts": [
    {
      "id": "kernel",
      "kind": "kernel",
      "selector": "ring0",
      "access": ["syscall"],
      "required": true,
      "reason": "Need kernel access."
    }
  ]
}
```

Invalid because workspace lifecycle cannot request kernel, Ring0, or syscall
authority.

## Fail-Closed Matrix

| Condition | Required result |
|---|---|
| Unknown workspace field | Reject manifest |
| `mode=none` with declared mounts | Reject manifest |
| Unknown mount kind | Reject manifest |
| Wildcard selector | Reject manifest |
| Raw host or kernel path | Reject manifest |
| Missing required capability decision | Deny admission |
| Effective mount wider than request | Keep mount inactive |
| Capability revoked | Disable or revoke workspace access |
| Trust treated as workspace authority | Deny |
| Receipt used as runtime authority | Deny |
| Unknown lifecycle state | Deny |
| Cleanup status unknown during removal | Deny future access |
| AI or semantic verdict used as authority | Deny |

## Relationship To Other Phase-18 Specs

1. `MODULE_MANIFEST_SCHEMA.md` defines workspace declaration placement.
2. `CAPABILITY_CONTRACT_SPECIFICATION.md` defines required authorization
   decisions for workspace access.
3. `PACKAGE_METADATA_SCHEMA.md` may provide package evidence inputs, but it
   must not contain workspace admission, lifecycle state, or mount fields.
4. `TRUST_CLASSIFICATION_MODEL.md` defines review inputs, not authority.
5. `PLUGIN_BOUNDARY_CONTRACT.md` defines plugin boundary review inputs, not
   workspace authority or mount creation.
6. `PLATFORM_ABI_VALIDATION_GATE.md` defines validation order and enforces
   workspace lifecycle invariants without creating mounts.

## Activation Boundary

This RFC is part of the active Phase-18 Platform Constitution set.

This RFC does not authorize implementation work that widens the kernel ABI,
adds syscalls, places policy in Ring0, creates runtime mounts, or makes
semantic/AI systems execution authority.
