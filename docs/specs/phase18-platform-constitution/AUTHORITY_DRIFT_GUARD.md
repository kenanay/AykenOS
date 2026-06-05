# Phase-18 Authority Drift Guard

This document is subordinate to PHASE 0 - FOUNDATIONAL OATH,
`ARCHITECTURE_FREEZE.md`, `PHASE18_TRANSITION_DECISION.md`,
`PHASE18_ACTIVATION_DECISION.md`, and the Phase-18 Platform Constitution
reference set. In case of conflict, those documents prevail.

**Status:** ACTIVE REVIEW GUARD / RUNTIME NOT AUTHORIZED
**Review guard id:** `ayken.platform.phase18.authority_drift_guard.v1`
**Authority boundary:** Documentation/review guard only; not a CI gate, merge
authority, closure authority, runtime validator, manifest parser, package
installer, registry, runtime loader, workspace runtime, mount authority,
plugin host, capability issuer, trust issuer, execution authority, Semantic
CLI authority, AI Runtime authority, syscall, or kernel ABI expansion.

## Purpose

Phase-18 is active only as Platform Constitution. This guard defines how
future Phase-18 edits are reviewed so constitutional contracts do not drift
into runtime authority.

The guard does not authorize implementation. It exists to keep the active
specification set from becoming an implicit package installer, loader,
workspace runtime, capability engine, trust service, plugin host, Semantic CLI
authority, or AI Runtime authority.

## Core Rule

```text
Phase-18 authority is constitutional, not operational.
```

The mandatory invariant remains:

```text
Constitution != Runtime
```

A schema is not a parser. A contract is not an engine. A lifecycle state is
not a runtime object. A validation receipt is not an authority grant. An
accepted Platform Constitution does not install, load, mount, issue, trust,
execute, or run anything.

## Protected Separations

Future Phase-18 edits must preserve these separations:

1. Constitution != Runtime.
2. Manifest != Parser.
3. Package metadata != Installer.
4. Package compatibility != Install permission.
5. Trust classification != Capability.
6. Trust classification != Execution permission.
7. Capability contract != Token issuer.
8. Capability receipt != Bearer token.
9. Workspace lifecycle != Workspace runtime.
10. Workspace admission != Mount creation.
11. Logical mount record != Real filesystem mount.
12. Plugin boundary != Plugin loader.
13. Plugin compatibility != Loading.
14. Plugin binding decision != Execution.
15. Platform ABI validation != Authority grant.
16. Validation receipt != Runtime handle.
17. Review PASS != Activation by itself.
18. CI PASS != Runtime authority.
19. Semantic CLI output != Execution verdict authority.
20. AI Runtime output != Execution verdict authority.
21. Platform ABI != Kernel ABI expansion.
22. Phase-18 active pointer != Phase-19 runtime permission.

Unknown, ambiguous, or mixed authority interpretations fail closed.

## Allowed Phase-18 Maintenance

Phase-18 may continue to accept documentation-only maintenance that preserves
the active Platform Constitution boundary:

1. Clarifying accepted RFC text without changing authority.
2. Adding non-runtime examples that explicitly deny install, load, mount,
   execute, issue, or trust effects.
3. Maintaining glossary, terminology, review checklist, and cross-consistency
   records.
4. Recording future-facing references as non-authoritative placeholders.
5. Tightening fail-closed wording for ambiguous terms.
6. Updating roadmap and index references to point at accepted documents.
7. Preparing Phase-19 decision inputs without implementing Phase-19 runtime.

Allowed maintenance must remain documentation-only unless a separate reviewed
phase decision explicitly authorizes implementation.

## Forbidden Phase-18 Drift

The following work is forbidden in Phase-18 unless a separate reviewed phase
decision, implementation RFC, evidence plan, and acceptance boundary exists:

1. Manifest parser implementation.
2. Package installer implementation.
3. Package registry publication service.
4. Runtime package loader.
5. Package install, enable, update, or execution path.
6. Workspace runtime implementation.
7. Real mount creation or runtime mount binding.
8. Plugin loader, autoload, host execution, or plugin runtime.
9. Capability token issuer.
10. Capability runtime binding engine.
11. Trust issuer or trust assignment service.
12. Platform ABI runtime validator binary with authority effects.
13. Semantic CLI execution authority.
14. AI Runtime authority.
15. New syscall.
16. Kernel ABI expansion.
17. Ring0 policy.
18. Kernel plugin system.
19. Loader handles, runtime handles, or bearer tokens in constitution records.
20. Treating evidence, diagnostics, review, or CI output as runtime control
    input.

If any Phase-18 PR contains one of these surfaces, the review outcome must be
deny or move the work to a separate phase package.

## Drift Trigger Vocabulary

The following terms require extra review whenever added or changed in
Phase-18 documents:

| Term | Safe Phase-18 meaning | Drift risk |
|---|---|---|
| `validated` | All required validation stages passed for an exact input bundle | May be misread as install, trust, enable, or execute |
| `trusted` | Evidence classification accepted by policy | May be misread as capability or privilege hierarchy |
| `approved` | Decision record for a requested scope | May be misread as bearer token issuance |
| `admitted` | Workspace policy state | May be misread as mount creation |
| `enabled` | Lifecycle state after review | May be misread as loader or execution state |
| `compatible` | Compatibility input for later review | May be misread as plugin loading |
| `binding` | Review record or declared relationship | May be misread as runtime handle |
| `receipt` | Evidence record | May be misread as bearer token |
| `issuer` | Future authority placeholder only | May be misread as active service |
| `loader` | Future implementation placeholder only | May be misread as Phase-18 runtime |
| `runtime` | Out-of-scope future phase unless explicitly denied | May be misread as active implementation |
| `execute` | Forbidden as Phase-18 authority | May be misread as permission |

Use of these terms is not automatically invalid. It is valid only when the
surrounding text keeps the term non-authoritative.

## Review Checklist

Every Phase-18 documentation PR should answer:

1. Does the change preserve `Constitution != Runtime`?
2. Does the change avoid new syscalls and kernel ABI expansion?
3. Does the change avoid Ring0 policy?
4. Does the change keep trust separate from capability?
5. Does the change keep validation separate from authority?
6. Does the change keep plugin compatibility separate from loading?
7. Does the change keep workspace lifecycle separate from mount creation?
8. Does the change keep capability receipts separate from tokens?
9. Does the change keep package metadata separate from install or execution?
10. Does the change keep Semantic CLI and AI Runtime outputs
    non-authoritative?
11. Does every example deny install, load, mount, execute, issue, and trust
    effects unless a later phase is explicitly referenced?
12. Does the change avoid adding runtime source code, workflow authority, or
    gate authority under Phase-18?

If any answer is no or unclear, the change fails this guard.

## Fail-Closed Review Matrix

| Condition | Required result |
|---|---|
| Constitution text implies parser implementation | Reject or move to Phase-19 decision package |
| Package metadata text implies install or execution | Reject |
| Trust text implies capability or execution authority | Reject |
| Capability receipt text implies bearer token | Reject |
| Workspace lifecycle text implies real mount creation | Reject |
| Plugin compatibility text implies load, autoload, or execution | Reject |
| Validation PASS implies authority grant | Reject |
| CI PASS implies runtime authority | Reject |
| Semantic CLI or AI output implies execution authority | Reject |
| New syscall or kernel ABI expansion appears | Reject before Phase-18 review |
| Runtime code is bundled with Phase-18 constitution maintenance | Reject or split |
| Node or CI maintenance is bundled with Phase-18 authority change | Split into separate maintenance PR |
| Phase-19 runtime planning is bundled with Phase-18 guard maintenance | Split into separate decision package |

## Relationship To Phase-19

Phase-19 is the earliest planned place for Platform Runtime MVP work. This
guard does not start Phase-19.

Phase-19 requires a separate Runtime MVP Decision Package before any runtime
implementation is considered. That package must define its own scope,
non-goals, evidence plan, validation boundaries, and fail-closed acceptance
rules.

Until then, Phase-18 may describe the constitutional rules a future runtime
must obey, but it must not implement the runtime or grant operational
authority.

## Non-Authority Conclusion

This guard is an active Phase-18 review surface. It protects against authority
drift after activation.

It does not authorize runtime implementation, package installation, workspace
creation, plugin loading, capability issuance, trust assignment, Semantic CLI
execution authority, AI Runtime authority, new syscalls, kernel ABI expansion,
or Ring0 policy.
