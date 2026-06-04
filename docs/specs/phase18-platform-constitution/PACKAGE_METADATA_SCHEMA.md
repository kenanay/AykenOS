# Phase-18 Package Metadata Schema

This document is subordinate to PHASE 0 - FOUNDATIONAL OATH,
`ARCHITECTURE_FREEZE.md`, `PHASE18_TRANSITION_DECISION.md`,
`MODULE_MANIFEST_SCHEMA.md`, `CAPABILITY_CONTRACT_SPECIFICATION.md`, and
`WORKSPACE_LIFECYCLE_SPECIFICATION.md`. In case of conflict, those documents
prevail.

**Status:** RFC-DRAFT / PRE-ACTIVATION SPEC
**Schema id:** `ayken.platform.package.metadata.v1`
**Canonical file name:** `ayken.package.json`
**Authority boundary:** Documentation/specification only; not an installer,
registry, runtime loader, capability grant, trust grant, workspace admission,
mount authority, plugin host, execution right, or Phase-18 activation.

## Purpose

The package metadata schema answers the fourth Phase-18 Platform Constitution
question:

How does a package declare identity, version, publisher, content hashes,
signature evidence, dependency metadata, and Platform ABI compatibility without
becoming an authority source?

This first package metadata RFC is intentionally narrow. It records package
evidence inputs only. It does not define package installation, enablement,
execution, workspace admission, capability decisions, trust classification, or
runtime loading.

## Positive Scope

Version 1 package metadata is limited to:

1. Identity.
2. Version.
3. Publisher declaration.
4. Hash and integrity evidence.
5. Signature evidence references.
6. Dependency declarations.
7. Platform ABI compatibility.

No other constitutional domain is expressible in this RFC.

## Non-Goals

This schema does not define:

1. A new syscall.
2. Kernel ABI expansion.
3. A package installer implementation.
4. A package registry implementation.
5. A runtime package loader.
6. Capability requests, decisions, receipts, tokens, or grants.
7. Trust classification policy.
8. Workspace admission, workspace state, or mount creation.
9. Entrypoint execution rights.
10. Plugin host execution.
11. Semantic CLI execution verdict authority.
12. AI Runtime authority.

## Core Rule

Package metadata may declare package evidence. It must never grant authority.

The following invariants are mandatory:

1. Package metadata may identify a package and its content.
2. Package metadata may declare publisher identity claims and signature
   evidence references.
3. Package metadata may declare dependencies and compatibility floors.
4. Package metadata must not contain capability fields.
5. Package metadata must not contain trust classification fields.
6. Package metadata must not contain workspace or mount fields.
7. Package metadata must not contain loader, execution, or runtime handle
   fields.
8. Package metadata must not express kernel ABI expansion.
9. Unknown top-level fields fail validation.
10. Package metadata validation must fail closed.

## Metadata Encoding

`ayken.package.json` must be:

1. UTF-8 JSON.
2. A single JSON object.
3. Free of comments and trailing commas.
4. Free of duplicate keys.
5. Parsed with unknown top-level fields rejected.
6. Canonicalized by the future validator before digest computation.

The metadata file must not contain a self-declared package digest. A future
package envelope, registry, or validation gate may compute and store package
digests externally.

## Required Top-Level Fields

| Field | Type | Requirement |
|---|---|---|
| `package_metadata_version` | integer | Must be `1` for this RFC |
| `schema_id` | string | Must be `ayken.platform.package.metadata.v1` |
| `package_id` | string | Stable globally unique package id |
| `name` | string | Human-readable short name |
| `package_version` | string | Semver-compatible package version |
| `package_kind` | string | One of the allowed package kinds |
| `publisher` | object | Non-authoritative publisher declaration |
| `hashes` | object | Content hash declarations |
| `signatures` | object | External signature evidence references |
| `dependencies` | array | Declarative dependency requirements |
| `compatibility` | object | Platform ABI compatibility declaration |

Optional top-level fields are limited to:

| Field | Type | Requirement |
|---|---|---|
| `summary` | string | Short human-readable summary |
| `description` | string | Longer description |
| `links` | object | Non-authoritative links |

All other top-level fields must fail validation.

## Field Rules

### `package_id`

`package_id` must be a lower-case, reverse-DNS-like identifier:

```text
^[a-z][a-z0-9]*(\.[a-z][a-z0-9-]*)+$
```

Examples:

```text
org.ayken.examples.echo-package
com.example.workspace-indexer-package
```

The identifier must not include reserved authority segments:

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
14. `capability`
15. `workspace`
16. `mount`
17. `loader`
18. `execution`
19. `ai-runtime`
20. `semantic-verdict`

### `package_version`

`package_version` must use a semver-compatible string:

```text
MAJOR.MINOR.PATCH
```

Pre-release and build metadata may be allowed by a future validator, but this
RFC only requires the base form.

### `package_kind`

Allowed values:

| Value | Meaning |
|---|---|
| `module_package` | Package containing userspace module metadata and artifacts |
| `library_package` | Package containing userspace library artifacts |
| `asset_package` | Package containing static assets |
| `bundle_package` | Package containing multiple package-local resources |

Forbidden values include `kernel`, `driver`, `syscall`, `ring0`,
`scheduler`, `interrupt`, `plugin`, `semantic-verdict`, and `ai-runtime`.

`PLUGIN_BOUNDARY_CONTRACT.md` defines plugin boundary semantics separately.
Package metadata still does not create plugin package, loader, or execution
authority.

### `publisher`

The `publisher` object is a declaration, not trust classification.

| Field | Type | Requirement |
|---|---|---|
| `declared_id` | string | Publisher identity claim |
| `display_name` | string | Optional human-readable name |
| `contact_refs` | array | Optional contact or profile references |

Unknown fields in `publisher` fail validation.

Publisher declarations may be inputs to `TRUST_CLASSIFICATION_MODEL.md`.
They do not create trust by themselves.

### `hashes`

The `hashes` object declares package content hashes:

| Field | Type | Requirement |
|---|---|---|
| `algorithm` | string | Must be `sha256` for this RFC |
| `content_set_digest` | string | Digest over the canonical content descriptor set |
| `content` | array | Hashed package content descriptors |

Each `content` entry must include:

| Field | Type | Requirement |
|---|---|---|
| `id` | string | Stable content id unique within the package |
| `path` | string | Relative package-local path |
| `kind` | string | Content kind |
| `sha256` | string | 64-character lower-case hex SHA-256 |
| `size_bytes` | integer | Non-negative byte size |

Allowed content kinds:

| Kind | Meaning |
|---|---|
| `manifest` | Declarative manifest or metadata file |
| `executable` | Userspace executable artifact |
| `library` | Userspace library artifact |
| `asset` | Static package asset |
| `config` | Declarative configuration artifact |
| `metadata` | Non-authoritative metadata artifact |

Paths must be relative, must not contain `..`, and must not start with `/`.
Raw host paths, device paths, kernel paths, and URLs are forbidden.

Content hashes are evidence inputs. They are not trust classification, install
authority, or execution authority.

The following fields are forbidden under `hashes`:

1. `package_digest`
2. `envelope_digest`
3. `trust_digest`
4. `capability_digest`
5. `workspace_digest`
6. `runtime_handle`

Package or envelope digests must be computed externally by a future package
validation gate or registry layer.

### `signatures`

The `signatures` object declares external signature evidence references:

| Field | Type | Requirement |
|---|---|---|
| `signature_policy` | string | Must be `evidence_only` |
| `signature_refs` | array | External signature references; may be empty |

Each signature reference must include:

| Field | Type | Requirement |
|---|---|---|
| `id` | string | Stable reference id |
| `kind` | string | `external`, `minisign`, `cosign`, `pgp`, or `x509` |
| `ref` | string | External evidence reference |
| `signed_digest` | string | Digest claimed by the signature reference |

Signature references are evidence inputs only. They do not imply `signed`,
`verified`, `trusted`, installation eligibility, or capability approval.

The `signatures` object must not contain private keys, bearer tokens, trust
levels, review verdicts, or registry approval status.

### `dependencies`

Each dependency entry must include:

| Field | Type | Requirement |
|---|---|---|
| `package_id` | string | Required package id |
| `version_constraint` | string | Version constraint expression |
| `required` | boolean | Whether absence blocks dependency resolution |
| `reason` | string | Human-readable reason |

Dependencies are review and resolution inputs only. They do not install,
enable, trust, execute, or authorize another package.

Unknown fields in a dependency entry fail validation.

### `compatibility`

The `compatibility` object must declare compatibility above the frozen kernel:

```json
{
  "platform_package_abi": "ayken.platform.package.v1",
  "kernel_abi_floor": "syscall-v2-1000-1011"
}
```

`kernel_abi_floor` is a compatibility floor only. It must not be interpreted as
direct syscall access or kernel ABI expansion.

Unknown fields in `compatibility` fail validation.

## Forbidden Domain Fields

The following top-level or nested domains are forbidden in Version 1 package
metadata:

1. `trust`
2. `trust_level`
3. `trusted`
4. `verified`
5. `review_status`
6. `distribution_trust`
7. `capability`
8. `capabilities`
9. `capability_requests`
10. `capabilities_granted`
11. `capability_decisions`
12. `capability_token`
13. `token`
14. `workspace`
15. `workspace_binding`
16. `mount`
17. `mounts`
18. `execution`
19. `entrypoints`
20. `loader`
21. `autoload`
22. `runtime_handle`
23. `plugin_boundary`
24. `semantic_verdict`
25. `ai_runtime`

## Forbidden Authority Claims

The package metadata must fail validation if any top-level or nested field
attempts to declare:

1. New syscall IDs.
2. Kernel ABI changes.
3. Ring0 policy.
4. Scheduler policy.
5. IRQ/interrupt control.
6. Capability requests, grants, decisions, receipts, or tokens.
7. Runtime handles.
8. Trust classification.
9. Workspace admission or mounts.
10. Plugin loading authority.
11. Entrypoint execution rights.
12. Semantic execution verdict authority.
13. AI Runtime authority.

## Minimal Valid Example

```json
{
  "package_metadata_version": 1,
  "schema_id": "ayken.platform.package.metadata.v1",
  "package_id": "org.ayken.examples.echo-package",
  "name": "echo-package",
  "package_version": "1.0.0",
  "package_kind": "module_package",
  "publisher": {
    "declared_id": "org.ayken.examples",
    "display_name": "Example Publisher",
    "contact_refs": []
  },
  "hashes": {
    "algorithm": "sha256",
    "content_set_digest": "2222222222222222222222222222222222222222222222222222222222222222",
    "content": [
      {
        "id": "module-manifest",
        "path": "ayken.module.json",
        "kind": "manifest",
        "sha256": "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
        "size_bytes": 2048
      },
      {
        "id": "echo-cli",
        "path": "bin/echo",
        "kind": "executable",
        "sha256": "abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789",
        "size_bytes": 4096
      }
    ]
  },
  "signatures": {
    "signature_policy": "evidence_only",
    "signature_refs": []
  },
  "dependencies": [],
  "compatibility": {
    "platform_package_abi": "ayken.platform.package.v1",
    "kernel_abi_floor": "syscall-v2-1000-1011"
  },
  "summary": "Package metadata for the echo example module."
}
```

This example declares package evidence only. It does not install, enable,
execute, trust, admit, mount, or authorize the package.

## Invalid Examples

### Self-Declared Trust

```json
{
  "trust_level": "trusted"
}
```

Invalid because trust classification is external to package metadata.

### Capability Field

```json
{
  "capability_requests": [
    "ayken.platform.workspace.read"
  ]
}
```

Invalid because package metadata cannot request, grant, decide, or bind
capabilities.

### Workspace Field

```json
{
  "workspace": {
    "mode": "required"
  }
}
```

Invalid because package metadata cannot declare workspace admission or mounts.

### Embedded Package Digest

```json
{
  "hashes": {
    "package_digest": "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
  }
}
```

Invalid because package or envelope digest must be computed externally.

### Runtime Loader Authority

```json
{
  "loader": {
    "autoload": true,
    "runtime_handle": "active"
  }
}
```

Invalid because package metadata cannot load or enable runtime surfaces.

### Kernel Package

```json
{
  "package_kind": "kernel",
  "syscalls": [1012]
}
```

Invalid because Phase-18 cannot add syscalls or package kernel authority.

### Absolute Content Path

```json
{
  "hashes": {
    "algorithm": "sha256",
    "content_set_digest": "2222222222222222222222222222222222222222222222222222222222222222",
    "content": [
      {
        "id": "host-root",
        "path": "/bin/sh",
        "kind": "executable",
        "sha256": "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
        "size_bytes": 1
      }
    ]
  }
}
```

Invalid because absolute host paths are forbidden.

## Fail-Closed Validation Matrix

| Condition | Required result |
|---|---|
| Missing required field | Reject package metadata |
| Unknown top-level field | Reject package metadata |
| Duplicate JSON key | Reject package metadata |
| Unsupported `package_metadata_version` | Reject package metadata |
| Package id uses reserved authority segment | Reject package metadata |
| Absolute or parent-relative content path | Reject package metadata |
| URL, device, host, or kernel path in content path | Reject package metadata |
| Content set digest mismatch | Reject package metadata |
| Capability field present | Reject package metadata |
| Trust classification field present | Reject package metadata |
| Workspace or mount field present | Reject package metadata |
| Runtime loader/handle field present | Reject package metadata |
| Package digest self-declared inside metadata | Reject package metadata |
| Kernel/Ring0/syscall expansion field present | Reject package metadata |
| Semantic verdict authority present | Reject package metadata |
| AI Runtime authority present | Reject package metadata |

## Relationship To Existing Phase-18 Specs

1. `MODULE_MANIFEST_SCHEMA.md` may be shipped as a hashed package content file;
   package metadata does not interpret manifest capability or workspace fields.
2. `CAPABILITY_CONTRACT_SPECIFICATION.md` remains external; package metadata
   must not contain capability requests, decisions, receipts, tokens, or
   grants.
3. `WORKSPACE_LIFECYCLE_SPECIFICATION.md` remains external; package metadata
   must not contain workspace admission, lifecycle state, or mount fields.
4. `TRUST_CLASSIFICATION_MODEL.md` defines how publisher and signature evidence
   influence review without granting capability.
5. `PLUGIN_BOUNDARY_CONTRACT.md` defines plugin boundary compatibility
   semantics; package metadata still does not grant plugin package, loader, or
   execution authority.
6. `PLATFORM_ABI_VALIDATION_GATE.md` defines validation order and enforces
   package metadata, manifest, capability, workspace, trust, and plugin
   invariants without installing packages.

## Activation Boundary

This RFC is not sufficient to activate Phase-18. Phase-18 activation still
requires an explicit `CURRENT_PHASE` pointer transition and reviewed acceptance
of the required Platform Constitution set.

This RFC does not authorize implementation work that widens the kernel ABI,
adds syscalls, places policy in Ring0, creates runtime loaders, admits
workspaces, grants trust, grants capabilities, or makes semantic/AI systems
execution authority.
