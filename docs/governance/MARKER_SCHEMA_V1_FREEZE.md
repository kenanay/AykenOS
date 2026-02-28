# Marker Schema v1 Freeze
This document is normative. If implementation and this document diverge, governance and CI must enforce this document.

## 1. Purpose
`constitution/markers_schema_v1.json` is the single source of truth for runtime proof marker contracts.

Gates must derive marker semantics only from:
1. the schema file, and
2. the schema-aware extractor pipeline.

Hardcoded marker regex/pattern additions inside gate scripts are forbidden.

## 2. Normative Scope
Freeze scope includes:
1. profile names
2. marker names
3. token/pattern matching rules
4. `required_count` semantics
5. `ordering` pairs
6. `profile_flags` semantics

Behavior-changing edits to these surfaces are governance changes.

## 3. Change Classes
### 3.1 Metadata-Only
Behavior-neutral edits:
1. descriptions
2. comments
3. cosmetic formatting with no contract effect

### 3.2 Compatible
Backward-compatible extension:
1. add new profiles
2. add markers to existing profiles without changing active contract behavior
3. add backward-compatible `profile_flags`

Constraint: active profile PASS/FAIL/SKIP contract must remain unchanged.

### 3.3 Breaking
Breaking changes include:
1. marker name changes
2. token/pattern changes
3. `required_count` changes
4. `ordering` relation changes
5. `profile_flags` semantic changes
6. any change that alters active profile PASS/FAIL/SKIP behavior

Breaking changes must not modify `markers_schema_v1.json`.
They must introduce `markers_schema_v2.json`.

## 4. Version Bump Rules
After v1 freeze, `markers_schema_v1.json` accepts only:
1. metadata-only edits
2. explicitly compatible extensions

Any breaking contract change requires v2.

Introducing `markers_schema_v2.json` requires a major `constitution/version.json` bump in the same change set.
CI must fail if v2 is introduced without a major version bump.

## 5. PR Process Requirements
Every schema-changing PR must include:
1. impact analysis
: affected gates, affected profiles, PASS/FAIL/SKIP behavior impact
2. mandatory diffs
: schema diff, extractor diff (if any), gate diff (if any)
3. evidence
: at least one run-id with relevant gate reports
4. approvals
: governance owner and kernel/CI owner

Single-party merge is forbidden for schema contract changes.

## 6. CI Enforcement
CI must enforce:
1. schema structural lint
: profile reference integrity, marker reference integrity, ordering integrity
2. extractor smoke checks
: every active profile must be extractable and contract-checkable
3. mandatory gate chain
: `make ci-gate-policy-proof-regression` and Phase10 A2/B chain
4. v1 breaking protection
: breaking v1 edits fail CI unless moved to v2

## 7. C1/C2 Policy Coupling
For profiles with:

```json
"profile_flags": {
  "requires_multi_owner": true
}
```

Normative behavior:
1. `SCHED_MULTI_OWNER=0` -> verdict `SKIP` (exit 0)
2. `SCHED_MULTI_OWNER=1` -> `SKIP` forbidden, strict PASS/FAIL required

This behavior is schema-driven and cannot be script-overridden ad hoc.

## 8. Constitutional Principle
Marker schema is not test configuration.
It is a runtime proof contract binding kernel, CI, and governance layers.

Schema contract modification is a constitutional change.
