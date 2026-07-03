# Phase-21 First Bounded Implementation Package Boundary

This document records the boundary for the Phase-21 first bounded
implementation actual skeleton.

The skeleton is limited to static, userspace-only, non-executing package
shape material. It is not package acceptance, runtime implementation
procedure, execution authority, process start authority, runtime state
authority, package loading authority, capability issuance authority, registry
publication authority, trust assignment authority, or source merge authority.

## Exact Skeleton File Set

The actual skeleton file set is limited to:

```text
docs/specs/phase21-first-bounded-implementation/PACKAGE_BOUNDARY.md
tools/phase21_first_bounded_validator/README.md
tools/phase21_first_bounded_validator/validator_skeleton.py
docs/specs/phase21-first-bounded-implementation/receipts/RECEIPT_SCHEMA.md
docs/specs/phase21-first-bounded-implementation/receipts/RECEIPT_TEMPLATE.md
docs/specs/phase21-first-bounded-implementation/fixtures/README.md
docs/specs/phase21-first-bounded-implementation/fixtures/minimal_valid_manifest.fixture.json
docs/specs/phase21-first-bounded-implementation/fixtures/denied_runtime_authority.fixture.json
tests/phase21_first_bounded_static/README.md
tests/phase21_first_bounded_static/test_validator_skeleton_static.py
docs/specs/phase21-first-bounded-implementation/CI_GATE_EXPECTATIONS.md
docs/specs/phase21-first-bounded-implementation/EVIDENCE_NOTES.md
```

Any file outside this list is outside the actual skeleton boundary.

## Category Mapping

| File | Category |
|---|---|
| `PACKAGE_BOUNDARY.md` | Package boundary document |
| `tools/phase21_first_bounded_validator/README.md` | Static validator skeleton |
| `tools/phase21_first_bounded_validator/validator_skeleton.py` | Static validator skeleton |
| `receipts/RECEIPT_SCHEMA.md` | Receipt schema/template |
| `receipts/RECEIPT_TEMPLATE.md` | Receipt schema/template |
| `fixtures/README.md` | Fixture input examples |
| `fixtures/minimal_valid_manifest.fixture.json` | Fixture input examples |
| `fixtures/denied_runtime_authority.fixture.json` | Fixture input examples |
| `tests/phase21_first_bounded_static/README.md` | Non-runtime tests |
| `tests/phase21_first_bounded_static/test_validator_skeleton_static.py` | Non-runtime tests |
| `CI_GATE_EXPECTATIONS.md` | CI gate expectation documentation |
| `EVIDENCE_NOTES.md` | Exact-SHA evidence notes |

## Non-Authorization Boundary

This skeleton does not authorize:

1. Package acceptance.
2. Package review result.
3. Runtime implementation procedure.
4. Code execution.
5. Process start.
6. Runtime state creation.
7. Package installation, loading, or execution.
8. Module loading.
9. Workspace runtime or real mounts.
10. Plugin loading or plugin instantiation.
11. Capability issuance.
12. Registry publication.
13. Trust assignment.
14. Distribution execution.
15. Deployment.
16. Source acceptance.
17. Source merge authority.
18. Syscall expansion.
19. Kernel ABI expansion.
20. Ring0 policy movement.

Unknown authority readings fail closed.

## Non-Executing Guarantee

The skeleton has no runtime entrypoint, package loader entrypoint, module
loader entrypoint, workspace mount behavior, plugin loader behavior, process
spawning hook, runtime state writer, capability issuer, registry publisher,
trust issuer, deployment hook, or distribution execution hook.

Static inspection of the skeleton is not execution authority.

Presence of the skeleton is not package acceptance.
