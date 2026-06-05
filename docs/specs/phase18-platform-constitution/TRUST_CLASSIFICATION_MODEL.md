# Phase-18 Trust Classification Model

This document is subordinate to PHASE 0 - FOUNDATIONAL OATH,
`ARCHITECTURE_FREEZE.md`, `PHASE18_TRANSITION_DECISION.md`,
`MODULE_MANIFEST_SCHEMA.md`, `CAPABILITY_CONTRACT_SPECIFICATION.md`,
`WORKSPACE_LIFECYCLE_SPECIFICATION.md`, and `PACKAGE_METADATA_SCHEMA.md`. In
case of conflict, those documents prevail.

**Status:** ACTIVE CONSTITUTION SPEC / RUNTIME NOT AUTHORIZED
**Contract id:** `ayken.platform.trust.classification.v1`
**Authority boundary:** Documentation/specification only; not a trust issuer,
installer, registry, runtime loader, capability grant, workspace admission,
mount authority, plugin host, execution right, semantic verdict source, AI
Runtime authority, or Phase-18 activation.

## Purpose

The trust classification model answers the fifth Phase-18 Platform
Constitution question:

How does the platform classify package, publisher, signature, validation, and
review evidence without turning trust into permission?

This RFC defines a fail-closed vocabulary and record shape for trust
classification. It does not assign trust to any package or module. It does not
install, enable, execute, authorize, or admit anything by itself.

## Core Rule

**Trust level does not grant capability.**

The following invariants are mandatory:

1. Trust classification is a policy input only.
2. Trust classification must never grant capability.
3. Trust classification must never widen capability scope.
4. Trust classification must never create bearer tokens.
5. Trust classification must never create workspace mounts.
6. Trust classification must never install, enable, load, or execute a module.
7. Trust classification must never bypass package metadata validation.
8. Trust classification must never bypass manifest validation.
9. Trust classification must never bypass the frozen kernel ABI.
10. Unknown classifications, inputs, states, or effects fail closed.

## Positive Scope

Version 1 trust classification is limited to:

1. Trust classification vocabulary.
2. External evidence input references.
3. Classification decision records.
4. Classification lifecycle states.
5. Review, install, update, distribution, quarantine, and revocation policy
   effects.
6. Fail-closed validation rules.

No runtime authority is expressible in this RFC.

## Non-Goals

This model does not define:

1. A new syscall.
2. Kernel ABI expansion.
3. A package installer implementation.
4. A package registry implementation.
5. A runtime package loader.
6. Capability request, decision, receipt, token, or grant formats.
7. Workspace admission, workspace state, or mount creation.
8. Plugin host execution.
9. Semantic CLI execution verdict authority.
10. AI Runtime authority.
11. A cryptographic signature format.
12. A global trust root implementation.

## Classification Vocabulary

Allowed trust classifications:

| Classification | Meaning | Authority boundary |
|---|---|---|
| `local` | Locally developed or manually introduced subject; no distribution trust implied | May require local/manual review only |
| `experimental` | Development or test subject accepted only under explicit development policy | Must not be production-default |
| `signed` | External signature evidence is present and accepted as evidence input | Signature does not imply validation or permission |
| `verified` | Required platform validation gates passed for the exact subject | Verification does not imply publisher trust or permission |
| `trusted` | Subject is accepted by a defined review/distribution policy | Trust still does not grant capability |
| `revoked` | Subject must not install, enable, update, execute, or distribute | Deny by default |

Unknown classifications fail closed.

### Classification Ordering

The classifications are not a privilege hierarchy.

`trusted` is not "more permitted" than `verified`. `signed` is not "less
capable" than `trusted`. Capability is external to this model and can be
approved only by the capability contract.

The classification values are policy labels. Policy may require multiple labels
as inputs, but labels must not be combined into authority.

## Evidence Inputs

A trust classification record may reference external evidence inputs:

| Input | Source | Boundary |
|---|---|---|
| Package metadata digest | `PACKAGE_METADATA_SCHEMA.md` or `PLATFORM_ABI_VALIDATION_GATE.md` | Evidence only |
| Publisher declaration | `PACKAGE_METADATA_SCHEMA.md` | Identity claim only |
| Signature reference | `PACKAGE_METADATA_SCHEMA.md` or external signature evidence | Evidence only |
| Manifest digest | `MODULE_MANIFEST_SCHEMA.md` canonical digest | Evidence only |
| Platform validation receipt | `PLATFORM_ABI_VALIDATION_GATE.md` | Evidence only |
| Review decision digest | Future review workflow | Evidence only |
| Registry policy digest | Future registry policy | Evidence only |
| Workspace policy digest | Future workspace policy | Admission input only |

Evidence inputs must be immutable references or digests. Raw secrets, private
keys, bearer tokens, kernel handles, runtime handles, and raw syscall arguments
are forbidden.

## Classification Subject

A classification subject must be explicit and digest-bound where possible.

Allowed subject kinds:

| Subject kind | Meaning |
|---|---|
| `package` | Package metadata subject |
| `module` | Module manifest subject |
| `publisher` | Publisher identity claim subject |
| `signature` | External signature evidence subject |
| `distribution_channel` | Distribution policy subject |

Forbidden subject kinds include `kernel`, `ring0`, `syscall`, `driver`,
`root`, `admin`, `capability`, `token`, `workspace_mount`, `runtime_loader`,
`semantic_verdict`, and `ai_runtime`.

## Classification Record

A classification record must be external to the manifest and package metadata.
It may be stored by a future registry, workspace, review, or platform
validation layer.

Required fields:

| Field | Type | Requirement |
|---|---|---|
| `contract_id` | string | Must be `ayken.platform.trust.classification.v1` |
| `subject_kind` | string | Allowed subject kind |
| `subject_id` | string | Stable subject identifier |
| `subject_version` | string | Subject version or `none` |
| `subject_digest` | string | 64-character lower-case hex SHA-256 or `none` |
| `classification` | string | Allowed classification value |
| `state` | string | Lifecycle state |
| `evidence_refs` | array | External evidence references |
| `policy_digest` | string | Digest of policy inputs |
| `review_mode` | string | `automatic`, `manual`, or `blocked` |
| `issued_at` | string | Record creation timestamp |
| `expires_at` | string | Timestamp or `never` |
| `revocation_epoch` | integer | Monotonic revocation epoch |

Unknown fields in a classification record fail validation.

Classification records must not include:

1. Capability grants.
2. Capability tokens.
3. Runtime bearer tokens.
4. Kernel handles.
5. Raw syscall arguments.
6. Workspace mounts.
7. Loader handles.
8. Execution verdicts.
9. AI or semantic authority claims.

## Lifecycle States

Allowed classification lifecycle states:

```text
unclassified -> candidate -> active
candidate -> rejected
active -> superseded
active -> quarantined
active -> revoked
quarantined -> active
quarantined -> revoked
revoked -> tombstoned
rejected -> tombstoned
superseded -> tombstoned
```

State meanings:

| State | Meaning |
|---|---|
| `unclassified` | No accepted classification exists |
| `candidate` | Classification is under review |
| `active` | Classification may be used as a policy input |
| `rejected` | Candidate classification was denied |
| `superseded` | Classification was replaced by a newer record |
| `quarantined` | Classification is temporarily blocked pending review |
| `revoked` | Classification is withdrawn and denies by default |
| `tombstoned` | Historical terminal record retained for evidence |

Unknown states fail closed.

## Policy Effects

Trust classification may affect only bounded policy effects:

| Effect | Meaning |
|---|---|
| `allow_review` | Subject may enter a review workflow |
| `require_manual_review` | Manual review is required |
| `allow_distribution_review` | Subject may be considered for distribution |
| `block_install_review` | Subject must not proceed to install review |
| `block_enable_review` | Subject must not proceed to enable review |
| `block_update_review` | Subject must not proceed to update review |
| `require_quarantine` | Subject must be quarantined before further review |
| `trigger_revocation_review` | Capability/workspace/runtime layers must review existing decisions |

These effects are policy gates, not authority grants.

Forbidden effects include:

1. `grant_capability`
2. `grant_token`
3. `widen_scope`
4. `create_mount`
5. `install`
6. `enable`
7. `execute`
8. `autoload`
9. `load_plugin`
10. `bypass_review`
11. `bypass_kernel_abi`
12. `semantic_verdict`
13. `ai_verdict`

## Classification Requirements

### `local`

`local` requires:

1. Explicit local introduction path.
2. Subject id and digest when available.
3. No distribution trust claim.

`local` must not auto-install, auto-enable, or grant capabilities.

### `experimental`

`experimental` requires:

1. Explicit development policy.
2. Manual or development-mode review.
3. Clear non-production boundary.

`experimental` must not become `trusted` without a new classification record.

### `signed`

`signed` requires:

1. Signature evidence reference.
2. Signed digest.
3. Publisher or key identity reference.
4. Policy digest for accepted signature evidence.

`signed` proves only that accepted signature evidence exists. It does not prove
safe behavior, validation success, or permission.

### `verified`

`verified` requires:

1. Exact subject digest.
2. Platform validation receipt reference.
3. Validation policy digest.
4. Evidence that validation belongs to the same subject.

`verified` does not imply publisher trust and does not grant capability.

### `trusted`

`trusted` requires:

1. Explicit review or distribution policy decision.
2. Policy digest.
3. Subject digest.
4. Evidence references accepted by that policy.
5. Non-expired active classification record.

`trusted` must not be inferred solely from `signed` or `verified`. It must not
grant capability, workspace access, package install, plugin loading, or
execution rights.

### `revoked`

`revoked` requires:

1. Revocation reason reference.
2. Monotonic revocation epoch.
3. Subject id and digest when available.
4. Policy digest for revocation.

`revoked` denies install, enable, update, distribution, and execution review by
default. It may trigger capability and workspace revocation review, but it is
not itself a capability decision.

## Relationship To Existing Phase-18 Specs

1. `MODULE_MANIFEST_SCHEMA.md` must not contain trust classification fields.
2. `PACKAGE_METADATA_SCHEMA.md` may declare publisher and signature evidence
   inputs, but must not self-declare trust.
3. `CAPABILITY_CONTRACT_SPECIFICATION.md` remains the only Phase-18 contract
   for capability request, decision, receipt, and revocation boundaries.
4. `WORKSPACE_LIFECYCLE_SPECIFICATION.md` may use trust classification as an
   admission input, but trust must not create mounts or workspace access.
5. `PLUGIN_BOUNDARY_CONTRACT.md` defines how plugin hosts consume trust as a
   review input without inheriting trust or loading plugins automatically.
6. `PLATFORM_ABI_VALIDATION_GATE.md` defines validation order and enforces that
   trust records, manifests, packages, capabilities, workspaces, and plugin
   boundaries remain separated without assigning trust.

## Valid Classification Example

```json
{
  "contract_id": "ayken.platform.trust.classification.v1",
  "subject_kind": "package",
  "subject_id": "org.ayken.examples.echo-package",
  "subject_version": "1.0.0",
  "subject_digest": "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
  "classification": "verified",
  "state": "active",
  "evidence_refs": [
    {
      "kind": "platform_validation_receipt",
      "ref": "validation:package:echo-package:1.0.0",
      "digest": "abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789"
    }
  ],
  "policy_digest": "1111111111111111111111111111111111111111111111111111111111111111",
  "review_mode": "automatic",
  "issued_at": "2026-06-04T00:00:00Z",
  "expires_at": "never",
  "revocation_epoch": 0
}
```

This example records a classification only. It does not install, enable,
execute, mount, trust-grant, or authorize the package.

## Invalid Examples

### Trust Grants Capability

```json
{
  "classification": "trusted",
  "grant_capability": "ayken.platform.workspace.write"
}
```

Invalid because trust classification cannot grant capability.

### Package Self-Declares Trust

```json
{
  "package_id": "org.ayken.examples.echo-package",
  "trust_level": "trusted"
}
```

Invalid because package metadata cannot self-certify trust.

### Verified Means Execute

```json
{
  "classification": "verified",
  "execute": true
}
```

Invalid because validation success does not grant execution rights.

### AI Trust Verdict

```json
{
  "classification": "trusted",
  "ai_verdict": "safe"
}
```

Invalid because AI Runtime is not a Phase-18 authority source.

## Fail-Closed Validation Matrix

| Condition | Required result |
|---|---|
| Missing required field | Reject classification record |
| Unknown classification | Reject classification record |
| Unknown lifecycle state | Reject classification record |
| Unknown subject kind | Reject classification record |
| Missing subject digest where policy requires digest | Reject classification record |
| Signature evidence missing for `signed` | Reject classification record |
| Validation receipt missing for `verified` | Reject classification record |
| Policy decision missing for `trusted` | Reject classification record |
| Revocation epoch regression | Reject classification record |
| Expired classification used as active | Deny policy use |
| Quarantined classification used as active | Deny policy use |
| Revoked classification used as active | Deny policy use |
| Trust treated as capability | Deny |
| Trust widens capability scope | Deny |
| Trust creates workspace mount | Deny |
| Trust installs, enables, loads, or executes package/module | Deny |
| Trust bypasses package or manifest validation | Deny |
| Trust bypasses kernel ABI boundary | Deny |
| Semantic or AI verdict used as authority | Deny |

## Activation Boundary

This RFC is part of the active Phase-18 Platform Constitution set.

This RFC does not authorize implementation work that widens the kernel ABI,
adds syscalls, places policy in Ring0, creates runtime loaders, admits
workspaces, grants capabilities, loads plugins, or makes semantic/AI systems
execution authority.
