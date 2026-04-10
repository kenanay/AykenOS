# Phase-16A Requirements

## Purpose

Phase-16A closes one real, deterministic, evidence-bound execution path without
redefining runtime authority, closure authority, or semantic scope.

The first slice is intentionally narrow:

- read-only query path only
- canonical IR frozen from the existing execution-plan subset
- BCIB lowering limited to the minimal runtime subset
- submit-only orchestration with fail-closed behavior

## Definitions

- **Phase-16A**: The first implementation slice of Phase-16.
- **Canonical IR**: The frozen register-based contract between semantic parsing
  and BCIB lowering.
- **Production Path**: The path used for the real Phase-16A command flow, not
  examples, mocks, or historical compatibility shims.
- **Submit-Only Orchestration**: A stateless translation/submission layer that
  may package and submit work but may not reinterpret authority, optimize policy,
  or execute semantic shortcuts.
- **Canonical Query Binding**: The deterministic lowering record that binds a
  supported query command to `context_path`, `predicate_kind`, and optional
  `predicate_fingerprint` before BCIB emission.
- **Implementation Naming Rule**: Production code artifacts are named by stable
  responsibility, not by transient phase label. Phase names live in spec and
  roadmap documents, not in long-lived module or file names.

## Requirement 1: Locked Command Surface

Phase-16A SHALL support only:

- `list <context>`
- `show <context> <id>`
- `query <context> where <predicate>`

All other semantic/orchestration commands in the production path SHALL fail
closed with explicit errors.

## Requirement 1A: Purpose-Based Implementation Naming

Production implementation files, modules, and exported code surfaces SHALL use
purpose-based names.

Examples of acceptable naming:

- `canonical_query`
- `submission_bridge`
- `orchestration`

Examples of forbidden naming for long-lived implementation surfaces:

- `phase16a`
- `phase17_query`
- `phase_x_runtime`

## Requirement 2: Canonical IR Freeze

Phase-16A SHALL use the existing execution-plan IR subset as its canonical IR.
No second semantic IR may be introduced for the first slice.

The canonical IR subset is:

- `LoadContext`
- `LoadField`
- `LoadLiteral`
- `Compare`
- `LogicalOp`
- `ApplyFilter`
- `Return`

Register invariants:

- `r0` SHALL be the active context/result register
- every lowered plan SHALL end in `Return`
- hidden implicit state transitions are forbidden

## Requirement 3: Two-Stage Lowering

The production path SHALL be split into two explicit stages:

1. `DSL -> Canonical IR`
2. `Canonical IR -> BCIB`

Direct production-path `DSL -> BCIB` lowering is forbidden.

## Requirement 4: Minimal BCIB Subset

Phase-16A production lowering may emit only:

- `DataQuery`
- `End`
- optional `TraceEmit`

The following opcodes SHALL NOT appear in the Phase-16A production path:

- `Nop`
- `DataCreate`
- `DataAdd`
- `UiRender`
- `AiAsk`

## Requirement 5: Deterministic Query Lowering Semantics

Every supported Phase-16A command SHALL lower through a Canonical Query Binding
before BCIB emission.

The Canonical Query Binding fields are:

- `context_path`
- `predicate_kind`
- `predicate_fingerprint`

Lowering rules:

- `list <context>` SHALL lower with:
  - `context_path = <context>`
  - `predicate_kind = all`
  - `predicate_fingerprint = None`
- `show <context> <id>` SHALL lower with:
  - `context_path = <context>`
  - `predicate_kind = id_eq`
  - `predicate_fingerprint = SHA-256 hex of the canonical equality predicate`
- `query <context> where <predicate>` SHALL lower with:
  - `context_path = <context>`
  - `predicate_kind = filter`
  - `predicate_fingerprint = SHA-256 hex of the canonical predicate form`

The lowerer SHALL preserve the difference between `show` and `query`; `show`
is not a generic query alias in the production path.

The same Canonical Query Binding SHALL always produce the same BCIB bytes.

Until the runtime verifier distinguishes semantic operands from control-flow
indices, Phase-16A lowering SHALL keep every emitted operand within the current
instruction-count bound accepted by the verifier while preserving deterministic
binding.

## Requirement 6: Submit-Only Orchestration

The orchestration layer SHALL be:

- stateless
- deterministic
- side-effect free except for explicit submission
- non-authoritative

The orchestration layer SHALL NOT:

- make policy decisions
- optimize semantic intent
- infer missing semantic meaning
- create a second execution authority path

## Requirement 7: Submission Boundary

Phase-16A submission SHALL flow through the submit-only orchestration boundary
into the existing runtime submit surface.

Semantic parsing and canonical IR construction SHALL NOT call the kernel-facing
submit path directly.

## Requirement 8: Minimum Submission Validation

Phase-16A submission validation SHALL reject a request unless all of the
following are true:

- the command belongs to the locked Phase-16A surface
- the canonical plan is non-empty
- the canonical plan ends in exactly one terminal `Return`
- the Canonical Query Binding is present
- the lowered BCIB is NOP-free
- the lowered BCIB contains no forbidden Phase-16A opcode
- the target context identifier is explicit and valid
- the submission bridge is available
- the derived capability set is explicit and non-empty
- the derived capability set contains the context-read capability required by
  the `LoadContext` source

A validator that unconditionally allows all submissions does not satisfy
Phase-16A.

## Requirement 9: Fail-Closed Behavior

The following conditions SHALL return explicit errors and SHALL NOT silently
fall back to `Nop`, empty output, or partial lowering:

- unsupported DSL command
- unsupported predicate shape
- empty context
- canonical IR node without BCIB mapping
- unavailable submission bridge
- invalid target context

## Requirement 10: Proof and Replay Closure

Phase-16A SHALL expose one end-to-end proof/replay path for the supported query
surface:

- canonical command string
- canonical IR fingerprint
- Canonical Query Binding fingerprint
- lowered BCIB SHA-256
- submission result identifier
- deterministic replay result

A replay deviation for the same canonical BCIB input SHALL be treated as failure.
