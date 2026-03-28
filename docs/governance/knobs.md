# Policy-Sensitive Knobs

This document defines governance rules for policy-sensitive build knobs.

## AYKEN_CR3_PCID

- Allowed values: `0` or `1`.
- Constitutional freeze discipline:
  - `ci-freeze-guard` must enforce `AYKEN_CR3_PCID=0`.
  - `ci-gate-ring3-execution-phase10a2` must pass `AYKEN_CR3_PCID=0` explicitly.
  - `ci-gate-ring3-user-leaf-rule` must also pass `AYKEN_CR3_PCID=0` explicitly.
  - `scripts/ci/gate_ring3_execution_phase10a2.sh` must keep `ENFORCED_AYKEN_CR3_PCID="0"` and fail-closed guards.
  - `scripts/ci/gate_ring3_user_leaf_rule.sh` must keep `AYKEN_CR3_PCID=0` and fail closed otherwise.
- Build determinism:
  - `Makefile` must propagate `AYKEN_CR3_PCID` to both `KERNEL_CFLAGS` and `KERNEL_ASMFLAGS`.
- Evidence contract:
  - Ring3 execution gate report must include both `enforced_ayken_cr3_pcid` and `observed_ayken_cr3_pcid`.

## AYKEN_RING3_POST_CR3_TEXT_PROBE

- Allowed values: `0` or `1`.
- Dedicated executable user-leaf rule gate enforcement:
  - `ci-gate-ring3-user-leaf-rule` must run with `AYKEN_RING3_POST_CR3_TEXT_PROBE=1`.
  - `scripts/ci/gate_ring3_user_leaf_rule.sh` must fail closed if this knob is not `1`.
- Authority note:
  - this knob supports the local deterministic witness chain
  - by itself it does not restate broader Phase10-A2 strict/global closure

## AYKEN_RING3_MASK_IRQ0_FIRST_ENTRY

- Allowed values: `0` or `1`.
- Ring3 runtime gate enforcement:
  - `ci-gate-ring3-execution-phase10a2` must run with `AYKEN_RING3_MASK_IRQ0_FIRST_ENTRY=1`.
  - `scripts/ci/gate_ring3_execution_phase10a2.sh` must keep `ENFORCED_AYKEN_RING3_MASK_IRQ0_FIRST_ENTRY="1"` and fail closed otherwise.
- Dedicated executable user-leaf rule gate enforcement:
  - `ci-gate-ring3-user-leaf-rule` must run with `AYKEN_RING3_MASK_IRQ0_FIRST_ENTRY=1`.
  - `scripts/ci/gate_ring3_user_leaf_rule.sh` must fail closed if this knob is not `1`.
- Syscall runtime gate enforcement:
  - `ci-gate-syscall-v2-runtime` must run with `AYKEN_RING3_MASK_IRQ0_FIRST_ENTRY=1`.
  - `scripts/ci/gate_syscall_v2_runtime.sh` must fail closed if this knob is not `1`.
- Authority note:
  - this knob exists to stabilize first-entry user progress before the timer IRQ path can preempt the lane
  - by itself it does not restate broader Phase10-A2 strict/global closure

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
