# Phase 10 Closure Checklist - Rule Authority First / Broad Closure Second

**Status:** ACTIVE (authority-layer checklist)
**Scope:** Current worktree/runtime authority split for Phase 10-A2 / 10-B follow-through
**Last Updated:** 2026-03-28

## 1. Purpose

This checklist separates three different authority questions that must not be
collapsed:

1. Is the executable user-leaf rule live and fail-closed in the current tree?
2. Is the broader historical `Phase10-A2` strict authority surface closed?
3. Is the global/freeze authority package closed?

The first question now has a live local deterministic answer. The latter two
still require their own evidence.

## 2. Authority Order

For current runtime-regression triage, the binding truth surface is:

1. [Makefile](/Users/asel/Desktop/AykenOS/Makefile)
2. [scripts/ci/gate_ring3_user_leaf_rule.sh](/Users/asel/Desktop/AykenOS/scripts/ci/gate_ring3_user_leaf_rule.sh)
3. [scripts/ci/gate_ring3_execution_phase10a2.sh](/Users/asel/Desktop/AykenOS/scripts/ci/gate_ring3_execution_phase10a2.sh)
4. [RING3_RUNTIME_CLOSURE_NOTE.md](/Users/asel/Desktop/AykenOS/docs/governance/RING3_RUNTIME_CLOSURE_NOTE.md)
5. [RUNTIME_INTEGRATION_GUARDRAILS.md](/Users/asel/Desktop/AykenOS/docs/operations/RUNTIME_INTEGRATION_GUARDRAILS.md)

Historical closure snapshots such as [overview.md](/Users/asel/Desktop/AykenOS/docs/roadmap/overview.md) and [PROJECT_STATUS_REPORT.md](/Users/asel/Desktop/AykenOS/docs/development/PROJECT_STATUS_REPORT.md) remain important records, but they do not override an active fail-closed regression in the current worktree.

## 3. Layer 1 - Executable Leaf Rule Checklist

The executable user-leaf rule is live only if all items below are true:

- [ ] `ci-gate-ring3-user-leaf-rule` returns `PASS`.
- [ ] The enforced lane is `USER_MINIMAL_MODE=phase10a2-text-witness-bp`.
- [ ] The enforced knobs are `AYKEN_RING3_POST_CR3_TEXT_PROBE=1`, `AYKEN_RING3_MASK_IRQ0_FIRST_ENTRY=1`, and `AYKEN_CR3_PCID=0`.
- [ ] Source guard still requires executable Ring3 image leaves to flow through `proc_alloc_user_image_frame() -> phys_alloc_frame_high()`.
- [ ] The authoritative runtime chain appears in the same run:
  `P10_TEXT_FRAME_WITNESS -> P10_POST_CR3_TEXT_PROBE -> P10_RING3_USER_CODE`
- [ ] Runtime evidence comes from one same-run evidence directory.
- [ ] Software walk output is not used as authority unless the walker is kernel-CR3-safe.

Interpretation:

1. This layer closes the first-user-fetch / executable-leaf rule only.
2. A pass here is real and fail-closed.
3. A pass here does **not** by itself restate broader `Phase10-A2` strict/global closure.

## 4. Layer 2 - Broad Phase10-A2 Strict Checklist

Broader `Phase10-A2` strict authority is not re-established until all items below are true:

- [ ] `ci-gate-ring3-execution-phase10a2` returns strict `PASS`.
- [ ] The passing run is a canonical build: `AYKEN_RING3_FETCH_PROBE=0`, `AYKEN_RING3_SECOND_CANONICAL_PROBE=0`, `AYKEN_RING3_FRESH_FRAME_PROBE=0`, `AYKEN_RING3_IRETQ_DIAG_PROBE=0`.
- [ ] `violations.txt` does not contain `missing_marker:P10_RING3_USER_CODE`.
- [ ] The canonical marker chain reaches the user-proof boundary:
  `P10_RING3_ATTEMPT -> P10_RFLAGS_IF_ON -> P10_RING3_COMMIT -> P10_CR3_SWITCH -> P10_RING3_ENTER -> P10_RING3_USER_CODE`
- [ ] `trace_cut_before_user:*` is absent from the A2 evidence.
- [ ] The authoritative marker source is the same-run `qemu_debugcon.log`, not a fallback/probe-only stream.
- [ ] Evidence files (`qemu_debugcon.log`, `qemu_serial.log`, `events.jsonl`, `report.json`) all belong to the same run and can be tied back to the same evidence directory.

Interpretation:

1. This is the broader historical strict surface.
2. Layer 1 passing without Layer 2 passing is valid, but narrower.
3. Layer 2 cannot be inferred from Layer 1 alone.

## 5. Layer 3 - Global / Freeze Authority Checklist

After Layers 1 and 2 are both green, global/freeze authority still requires:

- [ ] `ci-gate-performance` passes on the configured authority environment.
- [ ] Performance baseline drift, if intentional, is regenerated only through the authorized `perf-baseline-init` workflow.
- [ ] `ci-freeze` is green on the intended closure branch/PR.
- [ ] The closure claim is backed by a single commit / single run / same-evidence closure proof, not by mixed local and remote fragments.
- [ ] Closure-facing docs are synchronized so active truth surfaces do not contradict the closure claim.

Interpretation:

1. Local runtime rule closure without freeze authority is not a global close.
2. Broad strict closure without freeze authority is not a global close.
3. Global/freeze authority is the last layer, not the first.

## 6. Operational Rule

Use this sequence:

1. Keep Layer 1 fail-closed and green.
2. Close Layer 2 on the broader historical `Phase10-A2` strict surface.
3. Then finish Layer 3: performance authority, CI freeze, and full document sync.

## 7. Current Reading

For the present tree, the repo-grounded reading is:

1. The executable user-leaf rule is now a live local deterministic authority surface.
2. Broader `Phase10-A2` strict/global authority remains separate and pending.
3. Global/freeze authority also remains pending until primary CI full-suite evidence exists.
4. The next hardening wall after Layer 1 is the transition paging contract described in `docs/specs/phase10b-execution-path-hardening/ring3-transition-minimal-secure-paging-contract.md`.
