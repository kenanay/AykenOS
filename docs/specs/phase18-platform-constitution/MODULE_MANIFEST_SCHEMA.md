# Phase-18 Module Manifest Schema

This document is subordinate to PHASE 0 - FOUNDATIONAL OATH,
`ARCHITECTURE_FREEZE.md`, and `PHASE18_TRANSITION_DECISION.md`. In case of
conflict, those documents prevail.

**Status:** RFC-DRAFT / PRE-ACTIVATION SPEC
**Schema id:** `ayken.platform.module.manifest.v1`
**Canonical file name:** `ayken.module.json`
**Authority boundary:** Documentation/specification only; not an installer,
runtime loader, capability grant, trust grant, or Phase-18 activation.

## Purpose

The module manifest answers the first Phase-18 Platform Constitution question:

How does a module declare what it is, what it exposes, and what platform
resources it requests without expanding the kernel or receiving implicit
authority?

This schema defines the minimum declarative module contract. It is intentionally
smaller than a package format, registry entry, signing envelope, or workspace
lifecycle contract.

## Non-Goals

This schema does not define:

1. Package distribution format.
2. Publisher identity or signature trust.
3. Capability token issuance.
4. Workspace mount lifecycle.
5. Plugin host execution protocol.
6. Semantic CLI authority.
7. AI Runtime authority.
8. Kernel ABI or syscall changes.

## Core Rule

A manifest may request platform resources. It must never grant them.

The following invariants are mandatory:

1. `capability_requests` are requests only.
2. Trust classification is external to the manifest.
3. Capability tokens must not appear in the manifest.
4. Kernel ABI expansion must not be expressible in the manifest.
5. Unknown top-level fields fail validation.
6. Manifest validation must fail closed.

## Manifest Encoding

`ayken.module.json` must be:

1. UTF-8 JSON.
2. A single JSON object.
3. Free of comments and trailing commas.
4. Free of duplicate keys.
5. Parsed with unknown top-level fields rejected.
6. Canonicalized by the future validator before digest computation.

The manifest must not contain a self-declared manifest digest. A validator or
package layer may compute and store the digest externally.

## Required Top-Level Fields

| Field | Type | Requirement |
|---|---|---|
| `manifest_version` | integer | Must be `1` for this RFC |
| `schema_id` | string | Must be `ayken.platform.module.manifest.v1` |
| `module_id` | string | Stable globally unique module id |
| `name` | string | Human-readable short name |
| `version` | string | Semver-compatible module version |
| `module_kind` | string | One of the allowed module kinds |
| `summary` | string | Short human-readable summary |
| `entrypoints` | array | Declared module entrypoints |
| `platform` | object | Platform ABI compatibility declaration |
| `capability_requests` | array | Requested capabilities; may be empty |
| `workspace` | object | Workspace requirement declaration |
| `integrity` | object | Artifact hash declarations |

Optional top-level fields are limited to:

| Field | Type | Requirement |
|---|---|---|
| `description` | string | Longer description |
| `authors` | array | Human-readable author metadata |
| `license` | string | License identifier or proprietary marker |
| `links` | object | Non-authoritative links |
| `dependencies` | object | Declarative dependency requirements |
| `plugin_boundary` | object | Plugin host/export declaration |
| `semantic_surface` | object | Advisory-only semantic metadata |
| `extensions` | object | Non-authoritative extension metadata |

All other top-level fields must fail validation.

## Field Rules

### `module_id`

`module_id` must be a lower-case, reverse-DNS-like identifier:

```text
^[a-z][a-z0-9]*(\.[a-z][a-z0-9-]*)+$
```

Examples:

```text
org.ayken.examples.echo
com.example.workspace-indexer
```

The identifier must not include `kernel`, `ring0`, `syscall`, `driver`, or
`ai-runtime` as a segment in Phase-18.

### `version`

`version` must use a semver-compatible string:

```text
MAJOR.MINOR.PATCH
```

Pre-release and build metadata may be allowed by a future validator, but this
RFC only requires the base form.

### `module_kind`

Allowed values:

| Value | Meaning |
|---|---|
| `tool` | User-invoked platform tool |
| `service` | Userspace platform service |
| `plugin` | Module loaded through a plugin boundary |
| `library` | Shared userspace library surface |
| `workflow` | Declarative workflow surface |

Forbidden values include `kernel`, `driver`, `syscall`, `ring0`,
`scheduler`, `interrupt`, `semantic-verdict`, and `ai-runtime`.

### `entrypoints`

Each entrypoint must be an object:

| Field | Type | Requirement |
|---|---|---|
| `id` | string | Stable entrypoint id unique within the module |
| `kind` | string | `cli`, `service`, `plugin`, `worker`, or `library` |
| `path` | string | Relative artifact path |
| `abi` | string | Entrypoint ABI id |
| `default` | boolean | Optional default marker |

`path` must be relative, must not contain `..`, and must not start with `/`.
The manifest may declare an entrypoint, but it does not authorize execution.

The initial entrypoint ABI id is:

```text
ayken.platform.entrypoint.v1
```

### `platform`

The `platform` object must declare compatibility with the Platform ABI above the
frozen kernel:

```json
{
  "platform_abi": "ayken.platform.module.v1",
  "kernel_abi_floor": "syscall-v2-1000-1011"
}
```

`kernel_abi_floor` is a compatibility floor only. It must not be interpreted as
direct syscall access or kernel ABI expansion.

### `capability_requests`

Each capability request must be an object:

| Field | Type | Requirement |
|---|---|---|
| `id` | string | Capability id from a future capability registry |
| `access` | array | Requested access verbs |
| `scope` | object | Requested resource scope |
| `required` | boolean | Whether missing capability blocks enablement |
| `reason` | string | Human-readable justification |

`capability_requests` do not grant capabilities. They are admission inputs for
future platform validation and user/admin review.

The following fields are forbidden anywhere under `capability_requests`:

1. `grant`
2. `token`
3. `capability_token`
4. `trusted`
5. `verified`
6. `ring0`
7. `syscall`

### `workspace`

The `workspace` object declares workspace dependency only:

| Field | Type | Requirement |
|---|---|---|
| `mode` | string | `none`, `optional`, or `required` |
| `declared_mounts` | array | Requested logical mounts; may be empty |

Workspace declarations do not create mounts. The future Workspace Lifecycle
Specification defines install, enable, mount, disable, and removal behavior.

### `integrity`

The `integrity` object declares module artifact hashes:

| Field | Type | Requirement |
|---|---|---|
| `artifacts` | array | Hashes for shipped module artifacts |

Each artifact entry must include:

| Field | Type | Requirement |
|---|---|---|
| `path` | string | Relative artifact path |
| `kind` | string | `executable`, `library`, `asset`, or `metadata` |
| `sha256` | string | 64-character lower-case hex SHA-256 |

Artifact hashes are package evidence inputs. They are not trust classification.

### `plugin_boundary`

`plugin_boundary` is optional. If present, it must be declarative:

| Field | Type | Requirement |
|---|---|---|
| `host_interfaces` | array | Host interfaces requested by the plugin |
| `exports` | array | Interfaces exported by the module |

This field does not load a plugin. The future Plugin Boundary Contract defines
host admission and runtime behavior.

### `semantic_surface`

`semantic_surface` is optional. If present, it must explicitly remain
advisory-only:

```json
{
  "authority": "advisory_only",
  "declares_execution_verdict": false
}
```

Any semantic field that claims execution verdict authority must fail
validation.

### `extensions`

`extensions` may carry non-authoritative metadata for experiments. It must not
contain fields that affect install, enable, capability grant, trust
classification, workspace access, plugin loading, semantic verdicts, or AI
authority.

## Trust Boundary

Trust is intentionally not a manifest field.

The following top-level fields are forbidden:

1. `trust`
2. `trust_level`
3. `trusted`
4. `verified`
5. `review_status`
6. `distribution_trust`

Trust classification belongs to the future Trust Classification Model and to
external registry/install receipts. A module must not self-certify trust.

## Forbidden Authority Fields

The manifest must fail validation if any top-level or nested field attempts to
declare:

1. New syscall IDs.
2. Kernel ABI changes.
3. Ring0 policy.
4. Scheduler policy.
5. IRQ/interrupt control.
6. Capability grants or tokens.
7. Trust classification.
8. AI Runtime authority.
9. Semantic execution verdict authority.

## Minimal Valid Example

```json
{
  "manifest_version": 1,
  "schema_id": "ayken.platform.module.manifest.v1",
  "module_id": "org.ayken.examples.echo",
  "name": "echo",
  "version": "0.1.0",
  "module_kind": "tool",
  "summary": "Echo text through a userspace module boundary.",
  "entrypoints": [
    {
      "id": "echo-cli",
      "kind": "cli",
      "path": "bin/echo",
      "abi": "ayken.platform.entrypoint.v1",
      "default": true
    }
  ],
  "platform": {
    "platform_abi": "ayken.platform.module.v1",
    "kernel_abi_floor": "syscall-v2-1000-1011"
  },
  "capability_requests": [
    {
      "id": "workspace.read",
      "access": ["read"],
      "scope": {
        "workspace": "current"
      },
      "required": false,
      "reason": "Read user-selected workspace input files."
    }
  ],
  "workspace": {
    "mode": "optional",
    "declared_mounts": []
  },
  "integrity": {
    "artifacts": [
      {
        "path": "bin/echo",
        "kind": "executable",
        "sha256": "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
      }
    ]
  },
  "plugin_boundary": {
    "host_interfaces": [],
    "exports": []
  }
}
```

## Invalid Examples

### Self-Granted Capability

```json
{
  "capability_requests": [
    {
      "id": "workspace.write",
      "grant": true
    }
  ]
}
```

Reason: `grant` attempts to convert a request into authority.

### Self-Declared Trust

```json
{
  "trust_level": "trusted"
}
```

Reason: trust classification is external to the manifest.

### Kernel Expansion

```json
{
  "syscalls": [1012]
}
```

Reason: Phase-18 does not add syscalls.

## Fail-Closed Validation Matrix

| Condition | Validator result |
|---|---|
| Missing required field | FAIL |
| Unknown top-level field | FAIL |
| Duplicate JSON key | FAIL |
| Unsupported `manifest_version` | FAIL |
| Absolute or parent-relative path | FAIL |
| Capability grant/token field present | FAIL |
| Trust classification field present | FAIL |
| Kernel/Ring0/syscall expansion field present | FAIL |
| Semantic verdict authority present | FAIL |
| AI Runtime authority present | FAIL |

## Relationship To Future Specs

| Future spec | Relationship |
|---|---|
| Package Metadata Schema | Owns publisher, signature, package digest, distribution channel |
| Capability Contract Specification | Defines capability ids, grants, tokens, receipts, revocation |
| Workspace Lifecycle Specification | Defines workspace states, mounts, enable/disable/remove behavior |
| Trust Classification Model | Defines trusted/verified/signed/local/revoked classification |
| Plugin Boundary Contract | Defines host admission, plugin loading, interface binding |
| Platform ABI Validation Gate | Implements fail-closed schema and authority checks |

## Activation Boundary

This RFC draft is not sufficient to activate Phase-18. Phase-18 activation still
requires an explicit `CURRENT_PHASE` pointer transition and acceptance of the
remaining Platform Constitution contracts required by
`PHASE18_TRANSITION_DECISION.md`.
