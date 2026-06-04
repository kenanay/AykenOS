# Phase-18 Plugin Boundary Contract

This document is subordinate to PHASE 0 - FOUNDATIONAL OATH,
`ARCHITECTURE_FREEZE.md`, `PHASE18_TRANSITION_DECISION.md`,
`MODULE_MANIFEST_SCHEMA.md`, `CAPABILITY_CONTRACT_SPECIFICATION.md`,
`WORKSPACE_LIFECYCLE_SPECIFICATION.md`, `PACKAGE_METADATA_SCHEMA.md`, and
`TRUST_CLASSIFICATION_MODEL.md`. In case of conflict, those documents prevail.

**Status:** RFC-DRAFT / PRE-ACTIVATION SPEC
**Contract id:** `ayken.platform.plugin.boundary.v1`
**Authority boundary:** Documentation/specification only; not a plugin loader,
installer, runtime host, capability grant, trust grant, workspace admission,
mount authority, execution right, semantic verdict source, AI Runtime
authority, kernel ABI expansion, or Phase-18 activation.

## Purpose

The plugin boundary contract answers the sixth Phase-18 Platform Constitution
question:

How can one module declare that it may extend another module without turning
that declaration into loading, execution, capability, trust, workspace, or
kernel authority?

This RFC defines the fail-closed vocabulary and record shape for plugin host
interfaces, extension points, compatibility checks, and external binding
decisions. It does not load plugins. It does not execute plugins. It does not
authorize plugin access by itself.

## Core Rule

**Plugin boundary is not authority.**

The following invariants are mandatory:

1. A plugin declaration must never load a plugin.
2. A plugin declaration must never auto-activate a plugin.
3. A plugin declaration must never grant capability.
4. A plugin declaration must never inherit trust.
5. A plugin declaration must never create workspace mounts.
6. A plugin declaration must never bypass package, manifest, trust,
   capability, or workspace validation.
7. A plugin declaration must never create execution verdict authority.
8. A plugin declaration must never require kernel ABI expansion.
9. A plugin host must remain userspace policy, not Ring0 policy.
10. Unknown plugin boundary fields, roles, states, interfaces, or effects fail
    closed.

## Positive Scope

Version 1 plugin boundary is limited to:

1. Plugin boundary vocabulary.
2. Host extension point declarations.
3. Plugin host interface requests.
4. Interface compatibility inputs.
5. External plugin binding decision records.
6. Binding lifecycle states.
7. Fail-closed validation rules.

No runtime authority is expressible in this RFC.

## Non-Goals

This contract does not define:

1. A new syscall.
2. Kernel ABI expansion.
3. Kernel-resident plugin systems.
4. A runtime plugin loader.
5. Automatic plugin activation.
6. Plugin execution scheduling.
7. Capability request, decision, receipt, token, or grant formats.
8. Workspace admission, workspace state, or mount creation.
9. Trust classification assignment.
10. Package installation or package registry behavior.
11. Semantic CLI execution verdict authority.
12. AI Runtime authority.

## Terms

| Term | Meaning | Authority boundary |
|---|---|---|
| Host module | Module that declares extension points | Does not auto-load plugins |
| Plugin module | Module that requests host interfaces | Does not receive capability by being a plugin |
| Extension point | Named host surface that may accept a compatible plugin | Not an execution right |
| Host interface | Versioned interface contract exposed by a host | Not a runtime handle |
| Plugin export | Versioned interface implementation declared by a plugin | Not a trust or capability grant |
| Binding candidate | Digest-bound host/plugin compatibility candidate | Not an active runtime binding |
| Binding decision | External review result for a binding candidate | Not a bearer token |

## Relationship To Existing Contracts

| Contract | Relationship |
|---|---|
| `MODULE_MANIFEST_SCHEMA.md` | Declares `plugin_boundary.host_interfaces` and `plugin_boundary.exports` only |
| `PACKAGE_METADATA_SCHEMA.md` | Declares package evidence only; no plugin loader or plugin package authority |
| `TRUST_CLASSIFICATION_MODEL.md` | May influence review path; trust does not inherit across plugin boundaries |
| `CAPABILITY_CONTRACT_SPECIFICATION.md` | Remains the only capability request, decision, receipt, and revocation boundary |
| `WORKSPACE_LIFECYCLE_SPECIFICATION.md` | Remains the only workspace admission and logical mount lifecycle boundary |
| Platform ABI Validation Gate | Must enforce ordering and separation across manifest, package, trust, capability, workspace, and plugin inputs |

Plugin boundary compatibility may consume evidence from these contracts. It
must not replace any of them.

## Manifest Surface

The module manifest optional `plugin_boundary` object remains declarative.

Allowed top-level fields inside `plugin_boundary`:

| Field | Type | Requirement |
|---|---|---|
| `host_interfaces` | array | Interfaces requested by a plugin module |
| `exports` | array | Extension points or plugin interfaces exported by a module |

Unknown fields in `plugin_boundary` fail validation.

The following fields are forbidden anywhere under `plugin_boundary`:

1. `autoload`
2. `auto_enable`
3. `load`
4. `execute`
5. `capabilities`
6. `capability_requests`
7. `grant`
8. `token`
9. `trust`
10. `trusted`
11. `verified`
12. `workspace`
13. `mount`
14. `kernel`
15. `ring0`
16. `syscall`
17. `semantic_verdict`
18. `ai_verdict`

## Host Interface Requests

`host_interfaces` declares the host interfaces a plugin module wants to attach
to. It does not attach the plugin.

Each entry must include:

| Field | Type | Requirement |
|---|---|---|
| `id` | string | Stable request id unique within the module |
| `host_module_id` | string | Expected host module id or `any_compatible_host` |
| `interface_id` | string | Requested interface id |
| `interface_version` | string | Requested interface semver base version |
| `extension_point_id` | string | Requested extension point id or `any` |
| `required` | boolean | Whether missing compatible host blocks the plugin feature |
| `reason` | string | Human-readable justification |

Unknown fields in a host interface request fail validation.

`host_module_id` must be either `any_compatible_host` or a lower-case
reverse-DNS-like module id. `any_compatible_host` is a compatibility selector,
not authority to attach to arbitrary hosts.

`interface_id` and `extension_point_id` must be lower-case, namespace-qualified
ids:

```text
^[a-z][a-z0-9]*(\.[a-z][a-z0-9-]*){2,}$
```

The value `any` is allowed only for `extension_point_id`, and it means that the
future validation gate must select a concrete host extension point before a
binding candidate can become compatible.

`interface_version` must use:

```text
MAJOR.MINOR.PATCH
```

Pre-release and build metadata are deferred to a future validator.

## Host Exports

`exports` declares extension points or plugin interfaces exported by a host
module. It does not publish a runtime loader.

Each export must include:

| Field | Type | Requirement |
|---|---|---|
| `id` | string | Stable export id unique within the module |
| `kind` | string | `extension_point` or `plugin_interface` |
| `interface_id` | string | Exported interface id |
| `interface_version` | string | Exported interface semver base version |
| `admission_policy` | string | `explicit_review` or `blocked` |
| `multiplicity` | string | `single` or `multiple` |
| `stability` | string | `experimental`, `stable`, or `deprecated` |

Unknown fields in an export fail validation.

`admission_policy` must not be `automatic` in Phase-18. The safe default is
`blocked`.

`multiplicity` is a compatibility hint only. It does not load one or many
plugins.

`stability` is metadata only. It does not grant trust or capability.

Forbidden export kinds include `kernel`, `ring0`, `syscall`, `driver`,
`scheduler`, `interrupt`, `capability`, `workspace_mount`, `runtime_loader`,
`semantic_verdict`, and `ai_runtime`.

## Plugin Capability Boundary

Plugins must not request platform capabilities directly through the plugin
boundary.

For Phase-18 plugin compatibility:

1. `plugin_boundary` must not contain `capabilities` or `capability_requests`.
2. A module with `module_kind=plugin` must declare an empty top-level
   `capability_requests` array for Phase-18 plugin-boundary compatibility.
3. A module with `module_kind=plugin` must not gain new platform capability
   because it is compatible with a host.
4. Host-owned platform capabilities remain governed only by
   `CAPABILITY_CONTRACT_SPECIFICATION.md`.
5. If a host invokes a plugin in a later runtime phase, the host must remain
   accountable for its own capability boundary.
6. Plugin compatibility must fail closed if a plugin requires a capability that
   is not explicitly authorized through the capability contract for the
   responsible module boundary.

Plugin compatibility is not a shortcut around capability review.

## Plugin Trust Boundary

Trust does not inherit across plugin boundaries.

The following are forbidden:

1. Host trust implies plugin trust.
2. Plugin trust implies host trust.
3. Signed package implies trusted plugin.
4. Verified host implies verified plugin.
5. Trusted package implies compatible plugin.

Each subject must be classified independently by
`TRUST_CLASSIFICATION_MODEL.md`. Trust classification may influence review
path only; it must not load the plugin or grant capability.

## Plugin Workspace Boundary

Plugins must not create workspace surfaces.

The following are forbidden under plugin boundary declarations and binding
records:

1. Workspace creation.
2. Workspace mount creation.
3. Workspace selector widening.
4. Workspace capability inheritance from host to plugin.
5. Plugin-local workspace authority.

If a later runtime phase allows a host to expose workspace-derived data to a
plugin, that exposure must remain bounded by the host's approved capability
decision and workspace lifecycle state. This RFC does not authorize that
runtime exposure.

## Plugin Execution Boundary

Plugin compatibility is not execution.

The following are forbidden:

1. Plugin execution verdicts.
2. Plugin approval verdicts.
3. Plugin loader handles.
4. Plugin autoload decisions.
5. Plugin runtime tokens.
6. Plugin scheduler decisions.
7. Semantic or AI verdicts used as plugin authority.

A compatible plugin binding candidate may become an input to a later runtime
loader only after a separate reviewed phase defines that runtime layer.

## Compatibility Rules

A host/plugin pair may be considered compatible only when all of the following
are true:

1. Host manifest validation passed.
2. Plugin manifest validation passed.
3. Host package metadata validation passed when package metadata exists.
4. Plugin package metadata validation passed when package metadata exists.
5. Host and plugin subject digests are explicit.
6. Requested `interface_id` equals exported `interface_id`.
7. Requested and exported interface major versions match.
8. Concrete `extension_point_id` is selected.
9. Host export `admission_policy` is `explicit_review`.
10. No forbidden plugin boundary fields are present.
11. No capability, trust, workspace, package, semantic, AI, syscall, or kernel
    authority is embedded in the plugin boundary.

If any input is missing, stale, ambiguous, or unverifiable, compatibility must
fail closed.

Minor and patch version compatibility policy is deferred to Platform ABI
Validation Gate. The safe default is exact version match.

## Binding Decision Record

A plugin binding decision is external to manifests and package metadata. It may
be stored by a future registry, workspace, review, or Platform ABI validation
layer.

Required fields:

| Field | Type | Requirement |
|---|---|---|
| `contract_id` | string | Must be `ayken.platform.plugin.boundary.v1` |
| `host_module_id` | string | Host module id |
| `host_module_version` | string | Host module version |
| `host_digest` | string | 64-character lower-case hex SHA-256 |
| `plugin_module_id` | string | Plugin module id |
| `plugin_module_version` | string | Plugin module version |
| `plugin_digest` | string | 64-character lower-case hex SHA-256 |
| `extension_point_id` | string | Concrete extension point id |
| `interface_id` | string | Interface id |
| `interface_version` | string | Interface version accepted by policy |
| `decision` | string | `compatible`, `incompatible`, `blocked`, `quarantined`, or `revoked` |
| `state` | string | Binding lifecycle state |
| `evidence_refs` | array | External evidence references |
| `policy_digest` | string | Digest of policy inputs |
| `review_mode` | string | `manual`, `automatic`, or `blocked` |
| `issued_at` | string | Record creation timestamp |
| `expires_at` | string | Timestamp or `never` |
| `revocation_epoch` | integer | Monotonic revocation epoch |

Unknown fields in a binding decision record fail validation.

Binding decision records must not include:

1. Runtime bearer tokens.
2. Kernel handles.
3. Raw syscall arguments.
4. Capability grants.
5. Trust grants.
6. Workspace mounts.
7. Loader handles.
8. Execution verdicts.
9. AI or semantic authority claims.

## Binding Lifecycle States

Allowed binding lifecycle states:

```text
unbound -> candidate -> compatible
candidate -> rejected
compatible -> suspended
compatible -> revoked
suspended -> compatible
suspended -> revoked
rejected -> tombstoned
revoked -> tombstoned
```

State meanings:

| State | Meaning |
|---|---|
| `unbound` | No binding candidate exists |
| `candidate` | Host/plugin compatibility is under review |
| `compatible` | Binding may be used as a policy input by a later runtime phase |
| `rejected` | Candidate binding was denied |
| `suspended` | Binding is temporarily blocked pending review |
| `revoked` | Binding is withdrawn and denies by default |
| `tombstoned` | Historical terminal record retained for evidence |

Unknown states fail closed.

`compatible` is not an active runtime state. It is a policy input only.

## Policy Effects

Plugin boundary decisions may affect only bounded policy effects:

| Effect | Meaning |
|---|---|
| `allow_binding_review` | Host/plugin pair may enter review |
| `require_manual_review` | Manual review is required |
| `block_binding_review` | Pair must not proceed to binding review |
| `require_quarantine` | Pair must be quarantined before further review |
| `trigger_binding_revocation_review` | Existing compatible records must be reviewed |

These effects are not authority grants.

Forbidden effects include:

1. `load_plugin`
2. `autoload`
3. `install`
4. `enable`
5. `execute`
6. `grant_capability`
7. `grant_token`
8. `inherit_trust`
9. `create_mount`
10. `widen_scope`
11. `bypass_review`
12. `bypass_kernel_abi`
13. `semantic_verdict`
14. `ai_verdict`

## Valid Host Export Example

```json
{
  "plugin_boundary": {
    "host_interfaces": [],
    "exports": [
      {
        "id": "workspace-indexer.extension.echo",
        "kind": "extension_point",
        "interface_id": "org.ayken.examples.echo.extension",
        "interface_version": "1.0.0",
        "admission_policy": "explicit_review",
        "multiplicity": "multiple",
        "stability": "experimental"
      }
    ]
  }
}
```

This example declares an extension point only. It does not load plugins.

## Valid Plugin Request Example

```json
{
  "module_kind": "plugin",
  "capability_requests": [],
  "plugin_boundary": {
    "host_interfaces": [
      {
        "id": "echo-plugin.host-request",
        "host_module_id": "org.ayken.examples.workspace-indexer",
        "interface_id": "org.ayken.examples.echo.extension",
        "interface_version": "1.0.0",
        "extension_point_id": "workspace-indexer.extension.echo",
        "required": true,
        "reason": "Expose echo formatting through the host extension point"
      }
    ],
    "exports": []
  }
}
```

This example requests host compatibility only. It does not grant plugin
execution or capability.

## Valid Binding Decision Example

```json
{
  "contract_id": "ayken.platform.plugin.boundary.v1",
  "host_module_id": "org.ayken.examples.workspace-indexer",
  "host_module_version": "1.0.0",
  "host_digest": "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
  "plugin_module_id": "org.ayken.examples.echo-plugin",
  "plugin_module_version": "1.0.0",
  "plugin_digest": "abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789",
  "extension_point_id": "workspace-indexer.extension.echo",
  "interface_id": "org.ayken.examples.echo.extension",
  "interface_version": "1.0.0",
  "decision": "compatible",
  "state": "compatible",
  "evidence_refs": [
    {
      "kind": "manifest_digest_pair",
      "ref": "plugin-binding:workspace-indexer:echo-plugin:1.0.0",
      "digest": "1111111111111111111111111111111111111111111111111111111111111111"
    }
  ],
  "policy_digest": "2222222222222222222222222222222222222222222222222222222222222222",
  "review_mode": "manual",
  "issued_at": "2026-06-04T00:00:00Z",
  "expires_at": "never",
  "revocation_epoch": 0
}
```

This example records compatibility only. It does not load, execute, mount,
trust-grant, capability-grant, or authorize the plugin.

## Invalid Examples

### Plugin Autoload

```json
{
  "plugin_boundary": {
    "autoload": true
  }
}
```

Invalid because plugin boundary cannot load or auto-activate plugins.

### Plugin Requests Capability Through Boundary

```json
{
  "plugin_boundary": {
    "capabilities": [
      "ayken.platform.workspace.read"
    ]
  }
}
```

Invalid because plugin boundary cannot request or grant capability.

### Plugin Module Requests Capability

```json
{
  "module_kind": "plugin",
  "capability_requests": [
    {
      "id": "ayken.platform.workspace.read"
    }
  ]
}
```

Invalid because Phase-18 plugin-boundary compatibility does not allow a plugin
module to request direct platform capability. The host boundary remains
responsible for capability review.

### Plugin Inherits Trust

```json
{
  "plugin_boundary": {
    "inherit_trust": "host"
  }
}
```

Invalid because trust does not inherit across plugin boundaries.

### Plugin Creates Workspace

```json
{
  "plugin_boundary": {
    "workspace": {
      "create": true
    }
  }
}
```

Invalid because plugin boundary cannot create workspace surfaces.

### Plugin Execution Verdict

```json
{
  "plugin_boundary": {
    "plugin_verdict": "approved",
    "execute": true
  }
}
```

Invalid because plugin compatibility is not execution authority.

### AI Plugin Verdict

```json
{
  "plugin_boundary": {
    "ai_verdict": "safe_to_load"
  }
}
```

Invalid because AI Runtime is not a Phase-18 authority source.

## Fail-Closed Validation Matrix

| Condition | Required result |
|---|---|
| Unknown `plugin_boundary` field | Reject manifest |
| Unknown host interface request field | Reject manifest |
| Unknown export field | Reject manifest |
| Missing required request/export field | Reject manifest |
| Unknown interface id | Reject binding candidate |
| Unknown extension point id | Reject binding candidate |
| Missing concrete extension point | Reject binding candidate |
| Interface major version mismatch | Reject binding candidate |
| Stale host digest | Reject binding candidate |
| Stale plugin digest | Reject binding candidate |
| Missing host or plugin package evidence where required | Reject binding candidate |
| Host export admission policy is not `explicit_review` | Reject binding candidate |
| Plugin declares autoload/load/execute | Reject |
| Plugin declares capability grant or token | Reject |
| Plugin module declares direct capability requests | Reject binding candidate |
| Plugin inherits trust from host or package | Reject |
| Plugin creates workspace or mount | Reject |
| Plugin bypasses package, manifest, trust, capability, or workspace validation | Reject |
| Plugin uses Semantic CLI or AI verdict as authority | Reject |
| Plugin requires kernel ABI expansion | Reject |

## Activation Boundary

This RFC is not sufficient to activate Phase-18. Phase-18 activation still
requires an explicit `CURRENT_PHASE` pointer transition and reviewed acceptance
of the required Platform Constitution set.

This RFC does not authorize implementation work that widens the kernel ABI,
adds syscalls, places policy in Ring0, creates runtime loaders, installs or
enables packages, admits workspaces, grants capabilities, loads plugins, or
makes semantic/AI systems execution authority.
