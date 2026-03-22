# Structured Output Minimal Spec

**Status:** Active minimal contract for the first structured semantic layer  
**Scope:** Minimal semantic layer on top of the landed output plane

## 1. Purpose

The landed output plane gives the execution model a safe byte-oriented result
surface:

- input plane = immutable kernel-owned BCIB backing
- output plane = executor-written bounded output window
- result plane = frozen kernel-owned publication of validated output bytes

That is sufficient for byte-level correctness, safety, and deterministic
publication. It does not yet define semantic result types.

This spec defines the smallest landed structured-output layer without changing
the current raw output contract.

## 2. Non-Negotiable Rules

The core boundary remains:

- kernel = mechanism + enforcement
- userland = meaning + interpretation

Therefore:

- the kernel MUST treat the payload as opaque bytes, regardless of the declared
  kind
- the kernel MUST NOT interpret payload semantics
- the kernel MUST NOT parse payload content
- the kernel MUST NOT perform field-level semantic validation

The kernel only enforces:

- header validity
- bounds
- known version
- known kind

All semantic interpretation, including parsing and validation of payload
content, is explicitly delegated to userland.

## 3. Layering Model

Structured output is an additive semantic layer on top of the landed raw
output plane. The current runtime now accepts both:

- raw v1 output headers
- structured v2 output headers

This means:

- raw output remains valid and backward-compatible
- structured output does not replace raw output
- structured output reuses the same bounded output/result publication model

The first structured-output landing MUST preserve:

- the existing `submit_execution`, `complete_execution`, and `wait_result`
  syscall surface
- the existing fixed output window and frozen result publication model

## 4. Header Shape

The first structured-output header is a distinct v2 struct:

```c
typedef struct ayken_execution_output_v2 {
    uint32_t magic;
    uint32_t abi_version;
    uint32_t kind;
    uint32_t flags;
    uint64_t bytes_written;
    uint64_t reserved[3];
} ayken_execution_output_v2_t;
```

This separates the semantic layer from the current minimal raw header rather
than retrofitting semantics into the old surface.

## 5. Kind Model

The first structured-output kind set MUST remain closed and minimal.

Initial allowed kinds:

```c
#define AYKEN_OUTPUT_KIND_RAW   0u
#define AYKEN_OUTPUT_KIND_BLOB  1u
```

Rules:

- the kernel only performs equality checks against the allowed set
- unknown `kind` MUST fail closed
- kind-specific parsing remains outside the kernel

## 6. Validation Contract

For structured output, the kernel MUST validate:

- `magic`
- `abi_version`
- `kind` in the allowed set
- `bytes_written` bounded by the fixed output window

The kernel MUST fail closed for:

- unknown `kind`
- version mismatch
- invalid bounds
- malformed header

There is no best-effort structured output mode.

## 7. Publication Contract

Structured publication still follows the landed result model:

- publish `header + payload`
- keep the result immutable after successful completion
- keep owner-visible mapping read-only and NX
- keep repeated successful waits deterministic
- keep tail zero-sealing for bytes beyond the declared logical result size

The first structured-output landing does not authorize a new owner-visible
publication surface.

## 8. Backward Compatibility

Backward compatibility is mandatory.

The minimum compatibility rule is:

- the current raw output header remains valid
- structured output is introduced as a distinct typed header
- raw publication remains available as the fallback contract

Structured output therefore extends the current result model; it does not break
or replace it.

## 9. Explicit Non-Goals

This first semantic layer does not authorize:

- JSON or text-schema payloads
- nested structures
- dynamic schema registration
- user-defined kinds
- streaming or chunked output
- kernel-side payload parsing
- kernel-side semantic validation

## 10. Validation Minimum

The first structured-output landing MUST eventually prove:

- v2 structured header validation is fail-closed
- unknown `kind` is fail-closed
- backward-compatible raw publication still works
- repeated waits remain deterministic
- timeout/abort/exit revoke the published structured result exactly as they do
  for raw output
