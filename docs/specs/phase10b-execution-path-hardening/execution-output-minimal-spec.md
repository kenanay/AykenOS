# Execution Output Minimal Spec

**Status:** Active landed contract for the first distinct output-plane landing  
**Scope:** Minimal extension on top of the landed execution lifecycle

## 1. Purpose

The previous runtime re-exposed the completed slot's kernel-owned BCIB backing
as the result window. The current runtime now publishes frozen validated output
bytes through that same owner-visible result surface without widening the
syscall ABI.

This spec defines the minimum landed upgrade path.

## 2. Core Model

The first output-plane landing MUST preserve the current syscall contract:

- `submit_execution(..., context_id)` stays unchanged
- `complete_execution(execution_id, completion_code)` stays the commit point
- `wait_result(execution_id, timeout_ms)` stays the result publication surface

The semantic change is:

- input plane = immutable kernel-owned BCIB backing
- output plane = executor-written bounded output window
- result plane = frozen kernel-owned publication of validated output bytes

The first landing MUST NOT add a per-write output syscall.

## 3. Userspace Surface

The worker process gets a fixed output VA:

- `EXECUTION_OUTPUT_VA`

The output window size is bounded and fixed:

- `AYKEN_EXECUTION_OUTPUT_WINDOW_SIZE`

The worker mapping MUST be:

- writable
- user-accessible
- non-executable

The owner-facing mapped result remains:

- read-only
- user-accessible
- non-executable

## 4. Output Header

The executor MUST write a valid header at the beginning of the output window.

Shared ABI:

```c
typedef struct ayken_execution_output_v1 {
    uint32_t magic;
    uint32_t abi_version;
    uint32_t flags;
    uint32_t reserved0;
    uint64_t bytes_written;
    uint64_t reserved[3];
} ayken_execution_output_v1_t;
```

Validation rules:

- `magic` MUST equal `AYKEN_EXECUTION_OUTPUT_MAGIC`
- `abi_version` MUST equal `AYKEN_EXECUTION_OUTPUT_VERSION`
- `bytes_written` MUST be `<= output_window_size - sizeof(header)`
- `bytes_written == 0` is valid

If any check fails, completion MUST fail-closed.
If a worker requests `COMPLETED` but output validation fails, the kernel MUST:

- terminalize the slot as `FAILED`
- clear the active execution latch
- wake waiters
- return `ESYS_V2_INVALID_STATE`

The first published result size for a valid output-backed completion MUST be:

- `sizeof(ayken_execution_output_v1_t) + bytes_written`

## 5. Ownership Rules

The output backing MUST remain slot-owned, not worker-owned.

This means:

- the worker only receives a fixed-VA mapping into slot-owned backing
- completion freezes the slot-owned output backing as the future result source
- later worker mutation MUST NOT change the already committed result

Completion therefore creates one explicit boundary:

- before successful `complete_execution()`: executor may write
- after successful `complete_execution()`: output backing is immutable
- failed or timed-out executions MUST NOT publish partially written output as a
  successful result
- bytes beyond `sizeof(header) + bytes_written` inside the final mapped result
  frame span MUST be zero-sealed before publication

## 6. Runtime Sequence

Minimum landing sequence:

1. pickup maps slot-owned output backing at `EXECUTION_OUTPUT_VA`
2. scheduler zero-fills the full output window before user execution begins
3. executor writes `header + payload`
4. `complete_execution()` validates ownership, latch, header, and bounds
5. validated output backing becomes the frozen result source
6. `wait_result()` maps that frozen result backing into the owner
7. repeated successful waits replay the same VA
8. timeout/abort/exit revoke and cleanup the output/result backing

## 7. Explicit Non-Goals

This first landing does not authorize:

- a new `write_output` syscall
- user-provided shared output buffers
- post-completion result mutation
- streaming or chunked output semantics
- a second owner-visible output ABI distinct from the existing `wait_result()`
  publication model

## 8. Validation Minimum

The first output-plane landing MUST prove:

- output window backing is distinct from input payload backing
- valid completion maps the written output bytes, not the input BCIB bytes
- repeated waits replay the same VA
- bytes past the declared logical result size inside the mapped frame span are
  zero-sealed
- invalid header magic fails closed
- invalid ABI version fails closed
- overflowing `bytes_written` fails closed
- foreign wait still fails closed
- timeout/abort/exit cleanup revokes the frozen output publication
