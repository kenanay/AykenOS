# Result Hash Minimal Spec

**Status:** Active minimal contract  
**Scope:** Minimal integrity anchor on top of the landed raw+structured result plane

## 1. Purpose

The landed output/result model now gives the execution system:

- bounded frozen result bytes
- deterministic same-VA replay
- read-only/NX owner publication
- raw v1 and structured v2 header support

What it does not yet provide is a stable integrity anchor over those frozen
published bytes.

This spec defines the smallest additive result-hash layer without widening the
current execution syscall surface.

## 2. Non-Negotiable Rules

The existing boundary remains:

- kernel = mechanism + enforcement
- userland = meaning + interpretation

Therefore:

- the kernel hashes published bytes, not semantic meaning
- the kernel MUST NOT parse payload content while computing the hash
- the kernel MUST NOT treat the digest as a semantic verifier verdict

The result hash is an integrity anchor, not a semantic interpretation layer.

## 3. Hash Subject

The hash subject for the first landing MUST be:

- the exact logical published result bytes
- starting at byte 0 of the owner-visible frozen result surface
- spanning exactly `result_size`

This means:

- the hash covers `header + payload`
- the hash does **not** cover zero-sealed trailing padding beyond `result_size`
- the hash does **not** cover pre-publication worker scratch bytes

The same rule applies regardless of whether the result was produced via:

- raw output v1
- structured output v2

## 4. Fixed Algorithm

The first landing MUST use one fixed algorithm:

- `SHA-256`

The first landing MUST NOT introduce:

- algorithm negotiation
- per-result algorithm selection
- dynamic digest sizing

The goal is a single deterministic integrity anchor.

## 5. Owner-Visible Surface

The first landing MUST preserve the existing `wait_result()` return contract:

- `wait_result()` still returns the result base VA

The hash therefore becomes an additive owner-visible sidecar, not a replacement
for the current result mapping.

Minimal landed surface:

- a deterministic read-only/NX per-result hash page at a fixed hash-sidecar VA
- the hash page is slot-owned until mapped
- repeated successful waits replay the same hash VA exactly as they replay the
  same result VA

## 6. Hash Header

The first sidecar header SHOULD be fixed and self-describing:

```c
typedef struct ayken_execution_result_hash_v1 {
    uint32_t magic;
    uint32_t abi_version;
    uint32_t algorithm;
    uint32_t flags;
    uint64_t hashed_size;
    uint8_t digest[32];
    uint8_t reserved[16];
} ayken_execution_result_hash_v1_t;
```

Rules:

- `algorithm` is fixed to the SHA-256 identifier for the first landing
- `hashed_size` MUST equal the logical `result_size`
- the digest is computed over exactly those `hashed_size` bytes

## 7. Runtime Contract

The first result-hash landing MUST:

- preserve the existing `submit_execution`, `complete_execution`, and
  `wait_result` syscall surface
- compute the digest over frozen published bytes only
- compute that digest during successful completion-time freeze rather than on
  demand during `wait_result()`
- keep the hash backing kernel-owned and slot-owned
- keep the owner-visible hash mapping read-only and NX
- keep repeated successful waits deterministic for both result VA and hash VA
- revoke the hash sidecar on timeout/abort/exit alongside the result mapping

## 8. Backward Compatibility

Result hashing is additive.

It MUST NOT:

- mutate raw v1 result bytes
- mutate structured v2 result bytes
- require a new output format
- break the current result publication ABI

The digest is attached to the existing result contract; it does not redefine
that contract.

## 9. Explicit Non-Goals

The first landing does not authorize:

- semantic payload hashing rules by `kind`
- kernel-side parsing of structured payloads
- signed receipts
- proof verdicts
- multi-algorithm support
- streamed/incremental owner-visible hash APIs

## 10. Validation Minimum

The first result-hash landing MUST prove:

- the digest is stable across repeated waits
- the digest matches the exact frozen published bytes, not padded bytes
- raw v1 and structured v2 results both produce hashes under the same rule
- timeout/abort/exit revoke the hash sidecar with the result mapping
- foreign callers still fail closed
