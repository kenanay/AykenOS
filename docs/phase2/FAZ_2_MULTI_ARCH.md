# Faz 2 - Multi-Arch Plan
This document is subordinate to PHASE 0 – FOUNDATIONAL OATH. In case of conflict, Phase 0 prevails.

Targets:
- Primary: x86_64 (host/UEFI), ARM64 (Raspberry Pi or aarch64 host)
- Future: riscv64 (note only)

Builds:
- Host demo: cargo build (CLI/runtime)
- Cross: cargo build --target x86_64-unknown-none / aarch64-unknown-none (or host OS targets for demo)
- Optional: QEMU smoke for x86_64; note for ARM64

Portability rules:
- All on-disk formats little-endian; use from_le_bytes
- Avoid pointer-based layouts; use offsets/indices
- Alignment: respect ABDF-defined padding; avoid unaligned loads
- Conditional code: cfg(target_arch) for any asm/backend specifics

CI ideas:
- Matrix build x86_64 + aarch64
- Lint/tests on host; binary size note optional
