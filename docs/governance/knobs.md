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

## Change Control

Any `AYKEN_CR3_PCID` policy change must update all enforcement points above in the same change-set and keep fail-closed behavior.
