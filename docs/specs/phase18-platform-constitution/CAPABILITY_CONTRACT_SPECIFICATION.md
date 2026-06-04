# Phase-18 Capability Contract Specification

This document is subordinate to PHASE 0 - FOUNDATIONAL OATH,
`ARCHITECTURE_FREEZE.md`, `PHASE18_TRANSITION_DECISION.md`, and
`MODULE_MANIFEST_SCHEMA.md`. In case of conflict, those documents prevail.

**Status:** RFC-DRAFT / PRE-ACTIVATION SPEC
**Contract id:** `ayken.platform.capability.contract.v1`
**Authority boundary:** Documentation/specification only; not a runtime
capability issuer, not a kernel token format, not a trust grant, and not
Phase-18 activation.

## Purpose

The capability contract answers the second Phase-18 Platform Constitution
question:

How does a module request platform access, receive an explicit authorization
decision, and remain revocable without self-granting authority or expanding
the frozen kernel ABI?

The module manifest declares `capability_requests`. This specification defines
what those requests mean, how they must be evaluated, what a decision may
record, and which authority surfaces remain out of scope.

## Non-Goals

This contract does not define:

1. A new syscall.
2. A new kernel capability token layout.
3. A runtime token issuer implementation.
4. A package registry.
5. A workspace mount implementation.
6. Trust classification policy.
7. Semantic CLI execution verdict authority.
8. AI Runtime authority.

## Core Rule

A capability request is not authority. A trust level is not authority. A
receipt is not authority. Runtime access requires an explicit capability
authorization decision and a future runtime binding that remains subordinate to
the frozen kernel ABI.

The following invariants are mandatory:

1. Modules may request capabilities only through the manifest contract.
2. A manifest must never contain granted capabilities, bearer tokens, or
   trust status.
3. Unknown capability ids fail closed.
4. Unknown access verbs fail closed.
5. Missing or ambiguous scope fails closed.
6. Trust classification may affect review path only; it must not grant
   capability.
7. Platform capability decisions must not require kernel ABI expansion.
8. Revocation must be representable for every positive authorization decision.

## Relationship To Existing Kernel Capability Mechanism

The existing syscall v2 kernel capability path is the frozen mechanism layer.
The Platform Constitution capability contract is the userspace policy and
admission layer above it.

| Layer | Role | Phase-18 rule |
|---|---|---|
| Kernel ABI | Frozen syscall v2 mechanism, including capability bind/revoke | Must not expand |
| Platform capability request | Manifest-declared access request | Must not grant access |
| Platform authorization decision | Reviewed policy outcome | Must remain external to manifest |
| Runtime token or binding | Future runtime implementation detail | Must use existing ABI or a later separate phase decision |
| Receipt | Evidence that a decision was made | Must not be a bearer token |

This contract may reference the existing mechanism. It must not redefine the
kernel ABI, add syscalls, or turn a platform decision record into a direct
kernel handle.

## Capability Request Object

The `capability_requests` entries in `ayken.module.json` use this shape:

| Field | Type | Requirement |
|---|---|---|
| `id` | string | Capability id from an accepted registry |
| `access` | array | Requested access verbs |
| `scope` | object | Requested resource scope |
| `required` | boolean | Whether denial blocks enablement |
| `reason` | string | Human-readable justification |

Unknown fields in a capability request fail validation.

### `id`

The capability id must be lower-case and namespace-qualified:

```text
^[a-z][a-z0-9]*(\.[a-z][a-z0-9-]*){2,}$
```

Examples:

```text
ayken.platform.workspace.read
ayken.platform.workspace.write
ayken.platform.plugin.invoke
ayken.platform.observation.read
```

Phase-18 reserves the following segments. They must not appear in capability
ids unless a later reviewed phase explicitly reopens one of them:

1. `kernel`
2. `ring0`
3. `syscall`
4. `driver`
5. `root`
6. `admin`
7. `authority`
8. `verdict`
9. `trust`
10. `trusted`
11. `verified`
12. `token`
13. `grant`
14. `ai-runtime`
15. `semantic-verdict`

### `access`

`access` is an array of lower-case verbs. The initial common verb set is:

| Verb | Meaning |
|---|---|
| `read` | Inspect an allowed resource |
| `write` | Modify an allowed resource |
| `create` | Create a resource inside an allowed scope |
| `delete` | Remove a resource inside an allowed scope |
| `execute` | Run an allowed userspace entrypoint |
| `invoke` | Call an allowed plugin or service interface |
| `observe` | Read non-authoritative diagnostic state |

The accepted capability registry may restrict verbs per capability id. A verb
not accepted for the requested id fails closed.

The following verbs are forbidden in Phase-18:

1. `kernel`
2. `ring0`
3. `syscall`
4. `admin`
5. `root`
6. `authority`
7. `verdict`
8. `grant`
9. `trust`
10. `token`

### `scope`

`scope` limits the requested resource. It must be a JSON object:

| Field | Type | Requirement |
|---|---|---|
| `kind` | string | Scope kind |
| `selector` | string | Stable selector within that scope |
| `constraints` | object | Additional least-privilege constraints |

Unknown fields in `scope` fail validation.

Initial scope kinds:

| Kind | Meaning |
|---|---|
| `workspace` | Workspace-local resource selected by the future workspace contract |
| `package` | Package metadata or package-local artifact surface |
| `module` | Same-module declared artifact or entrypoint surface |
| `plugin_host` | Future plugin host interface boundary |
| `platform_observation` | Non-authoritative diagnostic or status surface |

The scope must be specific enough for review. The selector `*` is forbidden in
Phase-18. Empty selectors fail validation.

`constraints` must be a JSON object. Empty constraints are allowed only when
the capability registry explicitly marks the capability as scope-complete
without extra constraints.

### `required`

If `required` is `true`, denial of the capability must block module enablement.
If `required` is `false`, denial must disable only the dependent feature path.

A module must not use `required=false` to hide an authority dependency. If an
entrypoint cannot run correctly without the capability, the request must be
required.

### `reason`

`reason` must describe why the module needs the capability. Empty, generic, or
misleading reasons fail review. The reason is review metadata only and does not
grant authority.

## Authorization Decision Record

A capability authorization decision is external to the manifest. It may be
stored by a future package/workspace/runtime layer, but the manifest must not
embed it.

A decision record must include at least:

| Field | Type | Requirement |
|---|---|---|
| `contract_id` | string | Must be `ayken.platform.capability.contract.v1` |
| `module_id` | string | Module subject |
| `module_version` | string | Module version subject |
| `request_index` | integer | Index of the manifest request |
| `request_digest` | string | Digest of the canonical request object |
| `decision` | string | `approved`, `denied`, `quarantined`, or `revoked` |
| `effective_scope` | object | Scope accepted by policy |
| `policy_digest` | string | Digest of the policy input set |
| `review_mode` | string | `automatic`, `manual`, or `blocked` |
| `expires_at` | string | Timestamp or `never` |
| `revocation_epoch` | integer | Monotonic revocation epoch |

The effective scope must be equal to or narrower than the requested scope. A
decision must fail closed if the policy layer cannot prove that narrowing.

Decision records must not include:

1. Runtime bearer tokens.
2. Kernel handles.
3. Raw syscall arguments.
4. Private signing keys.
5. Trust self-declarations.
6. AI or semantic execution verdicts.

## Decision Lifecycle

The decision lifecycle is:

```text
requested -> evaluated -> approved
                       -> denied
                       -> quarantined
approved -> active
approved -> expired
approved -> revoked
active -> suspended
active -> revoked
suspended -> active
suspended -> revoked
```

Rules:

1. `requested` comes only from a validated manifest request.
2. `evaluated` requires accepted registry, policy, trust, package, and
   workspace inputs.
3. `approved` records authorization only; it does not mint a runtime token.
4. `active` is a future runtime state and must not be claimed by the manifest.
5. `expired`, `suspended`, and `revoked` must deny future access by default.
6. Unknown lifecycle states fail closed.

## Receipt Boundary

A capability receipt is evidence that a decision occurred. It is not a bearer
token and must not authorize access by itself.

A receipt may include:

| Field | Type | Requirement |
|---|---|---|
| `receipt_id` | string | Stable receipt id |
| `decision_digest` | string | Digest of the authorization decision |
| `module_id` | string | Module subject |
| `capability_id` | string | Capability subject |
| `decision` | string | Decision result |
| `issued_at` | string | Receipt creation time |
| `evidence_refs` | array | Non-authoritative evidence references |

A receipt must not include:

1. Token material.
2. Secret material.
3. Direct kernel capability ids.
4. A claim that trust grants capability.
5. A semantic or AI-generated verdict as authority.

## Revocation Contract

Every approved capability must be revocable.

Revocation may be triggered by:

1. Explicit user or maintainer action.
2. Package update.
3. Manifest digest change.
4. Workspace removal or policy change.
5. Trust downgrade or quarantine.
6. Capability registry withdrawal.
7. Expiration.
8. Evidence invalidation.

Revocation effects:

1. Future evaluation must deny access.
2. Future runtime token issuance must stop.
3. Existing future runtime bindings must be revoked by the runtime layer.
4. Receipts remain historical evidence, but no longer imply active access.
5. Failure to apply revocation must fail closed.

Trust downgrade may trigger revocation review. Trust downgrade does not prove a
capability violation by itself, and trust upgrade does not restore capability
by itself.

## Trust Boundary

Trust level does not grant capability.

Trust classification may affect:

1. Whether review is automatic or manual.
2. Whether installation is allowed.
3. Whether enablement is allowed.
4. Whether updates are accepted.
5. Whether quarantine is required.
6. Whether revocation review is triggered.

Trust classification must not:

1. Add access verbs.
2. Widen scope.
3. Create tokens.
4. Bypass registry policy.
5. Bypass workspace policy.
6. Bypass kernel capability checks.

## Registry Boundary

This specification defines the contract shape, not the final capability
registry. `PLATFORM_ABI_VALIDATION_GATE.md` must reject requests whose ids are
absent from the accepted registry.

Initial registry entries should define:

1. Capability id.
2. Allowed access verbs.
3. Allowed scope kinds.
4. Required scope constraints.
5. Whether manual review is required.
6. Whether the capability is workspace-bound.
7. Whether revocation requires runtime cleanup.

Registry entries must not authorize kernel ABI expansion.

## Valid Request Example

```json
{
  "id": "ayken.platform.workspace.read",
  "access": ["read"],
  "scope": {
    "kind": "workspace",
    "selector": "current.project.documents",
    "constraints": {
      "recursive": false,
      "write": false
    }
  },
  "required": true,
  "reason": "Read declared project documents for the module entrypoint."
}
```

This example is a request only. It does not grant workspace access.

## Valid Decision Example

```json
{
  "contract_id": "ayken.platform.capability.contract.v1",
  "module_id": "org.ayken.examples.echo",
  "module_version": "1.0.0",
  "request_index": 0,
  "request_digest": "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
  "decision": "approved",
  "effective_scope": {
    "kind": "workspace",
    "selector": "current.project.documents",
    "constraints": {
      "recursive": false,
      "write": false
    }
  },
  "policy_digest": "abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789",
  "review_mode": "manual",
  "expires_at": "never",
  "revocation_epoch": 0
}
```

This example records a decision only. It is not a runtime token.

## Invalid Examples

### Self-Granted Capability

```json
{
  "id": "ayken.platform.workspace.read",
  "access": ["read"],
  "scope": {
    "kind": "workspace",
    "selector": "current.project.documents",
    "constraints": {}
  },
  "required": true,
  "reason": "Needed by entrypoint.",
  "grant": true
}
```

Invalid because a manifest request contains `grant`.

### Trust As Capability

```json
{
  "id": "ayken.platform.workspace.write",
  "access": ["write"],
  "scope": {
    "kind": "workspace",
    "selector": "current.project",
    "constraints": {}
  },
  "required": true,
  "reason": "Trusted modules can write.",
  "trusted": true
}
```

Invalid because trust metadata appears in a capability request and is treated as
authority.

### Kernel Expansion Request

```json
{
  "id": "ayken.platform.kernel.syscall",
  "access": ["syscall"],
  "scope": {
    "kind": "kernel",
    "selector": "1000-1011",
    "constraints": {}
  },
  "required": true,
  "reason": "Need direct syscall authority."
}
```

Invalid because Phase-18 cannot request kernel, syscall, or Ring0 authority.

### Wildcard Scope

```json
{
  "id": "ayken.platform.workspace.read",
  "access": ["read"],
  "scope": {
    "kind": "workspace",
    "selector": "*",
    "constraints": {}
  },
  "required": true,
  "reason": "Read everything."
}
```

Invalid because wildcard scope is forbidden.

## Fail-Closed Matrix

| Condition | Required result |
|---|---|
| Unknown capability id | Deny |
| Unknown access verb | Deny |
| Forbidden reserved segment | Deny |
| Missing scope | Deny |
| Wildcard scope | Deny |
| Effective scope wider than request | Deny |
| Manifest contains token/grant/trust | Reject manifest |
| Decision references kernel ABI expansion | Reject decision |
| Receipt used as bearer token | Reject access |
| Revocation state unknown | Deny |
| Trust level treated as capability | Deny |
| AI or semantic verdict used as authority | Deny |

## Relationship To Other Phase-18 Specs

1. `MODULE_MANIFEST_SCHEMA.md` defines where requests are declared.
2. `PACKAGE_METADATA_SCHEMA.md` remains external to capability decisions and
   must not contain capability requests, decisions, receipts, tokens, or
   grants.
3. `WORKSPACE_LIFECYCLE_SPECIFICATION.md` defines workspace selectors,
   admission, logical mounts, and lifecycle.
4. `TRUST_CLASSIFICATION_MODEL.md` defines review inputs, not authority.
5. `PLUGIN_BOUNDARY_CONTRACT.md` defines plugin host interface boundaries; it
   must not request or grant capability.
6. `PLATFORM_ABI_VALIDATION_GATE.md` defines validation order and enforces
   registry, request, decision, and receipt invariants without issuing tokens.

## Activation Boundary

This RFC is not sufficient to activate Phase-18. Phase-18 activation still
requires an explicit `CURRENT_PHASE` pointer transition and reviewed acceptance
of the required Platform Constitution set.

This RFC does not authorize implementation work that widens the kernel ABI,
adds syscalls, places policy in Ring0, or makes semantic/AI systems execution
authority.
