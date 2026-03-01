# Policy-Sensitive Knobs

This document defines governance rules for policy-sensitive build knobs.

## AYKEN_CR3_PCID

- Allowed values: `0` or `1`.
- Constitutional freeze discipline:
  - `ci-freeze-guard` must enforce `AYKEN_CR3_PCID=0`.
  - `ci-gate-ring3-execution-phase10a2` must pass `AYKEN_CR3_PCID=0` explicitly.
  - `scripts/ci/gate_ring3_execution_phase10a2.sh` must keep `ENFORCED_AYKEN_CR3_PCID="0"` and fail-closed guards.
- Build determinism:
  - `Makefile` must propagate `AYKEN_CR3_PCID` to both `KERNEL_CFLAGS` and `KERNEL_ASMFLAGS`.
- Evidence contract:
  - Ring3 execution gate report must include both `enforced_ayken_cr3_pcid` and `observed_ayken_cr3_pcid`.

## PHASE10C C2 Strict Knobs

- `PHASE10C_ENFORCE`:
  - `ci-freeze` target sets default `1` (gate included in freeze chain).
  - `ci-freeze-local` also keeps phase10c gate active.
- `PHASE10C_C2_STRICT`:
  - `ci-freeze` target sets default `1`.
  - `ci-freeze-local` sets default `0` (development-friendly baseline).
- `PHASE10C_C2_OWNER_SET`:
  - CSV owner identity set used by strict validator (default `2`).
- `PHASE10C_C2_REQUIRE_CURSOR_MARKER`:
  - strict validator cursor/apply coupling check (default `1`).
- Enforcement contract:
  - `scripts/ci/gate_scheduler_mailbox_phase10c.sh` must forward these knobs to
    `tools/ci/validate_scheduler_mailbox_phase10c.py`.
  - Freeze workflow (`.github/workflows/ci-freeze.yml`) must pin strict values
    explicitly.

## Change Control

Any policy-sensitive knob change above must update all enforcement points in the
same change-set and keep fail-closed behavior.
