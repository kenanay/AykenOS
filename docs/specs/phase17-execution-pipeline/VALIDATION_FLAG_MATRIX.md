# Phase-17 Validation Flag and Evidence Matrix

**Status:** BINDING REVIEW INPUT / CLOSURE-CANDIDATE SUPPORT
**Effective date:** 2026-05-25
**Candidate evidence SHA:** `f129d4aaa37edd34b06e2f89dea57f20de57f691`
**Evidence status:** PR #144 candidate remote runtime, locked performance and
full `ci-freeze` PASS; review/merge and official closure remain pending.
**Duzenleyen / Gelistiren / Olusturan / Mimari Sorumlu:** Kenan AY
**Yetki siniri:** Bu belge metadata ve review girdisidir; runtime karari,
CI verdict'i, merge approval veya Phase-17 closure otoritesi degildir.

## 1. Purpose

This matrix prevents validation-only behavior from becoming an implicit
production contract. It records each Phase-17 execution acceptance lane,
the permitted flag configuration, the measured surface and the condition
under which the lane must be retained, consolidated or removed.

The governing rules are:

1. All listed test activation flags default to `0` in `Makefile`.
2. Validation-only execution lanes require `KERNEL_PROFILE=validation`.
3. Evidence output never becomes scheduler, verification or execution input.
4. Performance acceptance measures the existing timer/preemption hot path
   with Phase-17 validation-only flags disabled.
5. A diagnostic PASS never replaces locked-baseline acceptance or closure.
6. Any new validation flag or lane must be added here before review.

## 2. Flag Inventory

| Flag | Default | Permitted activation | Evidence purpose | Performance surface | Owner / review surface | Closure or removal condition |
|---|---:|---|---|---|---|---|
| `AYKEN_EXECUTION_MARKER_VALIDATION_ENABLE` | `0` | Validation execution gates only | Capture and validate ordered lifecycle markers | Disabled in locked performance build | Kernel execution lifecycle / architecture review | Retain only as reviewed regression instrumentation or remove after closure decision |
| `AYKEN_EXECUTION_MARKER_LIFECYCLE_SELFTEST` | `0` | Lifecycle and determinism lanes with marker validation enabled | Drive bounded single-slot lifecycle | Unmeasured validation payload | Kernel execution lifecycle | Reassess after closure manifest binds accepted evidence |
| `AYKEN_PHASE17_MARKER_INJECTION_TEST` | `0` | Negative determinism lane only | Enable test-only marker corruption bridge | Unmeasured; forbidden in performance acceptance | Failure-path validation | Retain only for fail-closed regression evidence |
| `AYKEN_MARKER_INJECT_INVALID_ORDER` | `0` | Negative lane with injection bridge enabled | Create invalid marker prefix | Unmeasured | Failure-path validation | Must remain coupled to negative rejection test |
| `AYKEN_EXECUTION_MARKER_NEGATIVE_EXPECT_REJECT` | `0` | Negative lane with invalid-order injection | Require pre-publication rejection | Unmeasured | Failure-path validation | Must remain fail-closed or be removed with injection lane |
| `AYKEN_BCIB_STUB_RESULT_ENABLE` | `0` | Public E2E stub lane uses `1`; worker/race lanes require `0` | Separate mapped-result ABI proof from worker completion proof | Disabled in locked performance build | Ring3/public ABI validation | Never represents production completion authority |
| `AYKEN_BCIB_PUBLIC_E2E_SELFTEST` | `0` | Public Ring3 submit/wait lane only | Prove public `1003 -> 1004` result publication with deterministic stub | Unmeasured validation payload | Public ABI boundary | Retain only as bounded ABI regression lane |
| `AYKEN_BCIB_WORKER_COMPLETION_SELFTEST` | `0` | Worker completion lane only, stub off | Prove bounded fixture `1003 -> 1011 -> 1004` | Unmeasured validation payload | Ring3 worker/public completion | Does not expand to general interpreter without separate review |
| `AYKEN_EXECUTION_RACE_SELFTEST` | `0` | Timeout-race lane only, stub off | Prove timer IRQ timeout wins over delayed `1011` | Unmeasured validation payload | IRQ/execution failure boundary | Broader race coverage requires separate bounded lane |
| `AYKEN_RING3_ENTRY_GUARD` | `0` | `1` in public/worker/race evidence and existing measured preemption contract | Preserve first-entry return-frame safety | Included only where existing performance harness explicitly sets it | Scheduler/Ring3 boundary | Any behavior change requires legacy-gate regression review |
| `AYKEN_RING3_MASK_IRQ0_FIRST_ENTRY` | `0` | `0` in Phase-17 public/worker/race lanes; legacy lanes may require their own value | Keep timer ordering owned by each gate contract | Not a Phase-17 measured payload feature | Scheduler/IRQ integration | Changes require low-half/timer witness and public/race regression proof |

## 3. Authorized Lane Configurations

| Lane / gate | Required enabled values | Required disabled values | Proves | Does not prove |
|---|---|---|---|---|
| Production default build | None of the Phase-17 validation flags | All flags in Section 2 default to `0` | No validation-only path leaks into default build | Runtime acceptance or closure |
| `ci-gate-execution-marker-lifecycle` | `MARKER_VALIDATION_ENABLE=1`, `MARKER_LIFECYCLE_SELFTEST=1` | Injection and public/worker/race paths `0` | One QEMU single-slot canonical lifecycle | Public Ring3 completion, performance, closure |
| `ci-gate-execution-marker-determinism` positive | `MARKER_VALIDATION_ENABLE=1`, `MARKER_LIFECYCLE_SELFTEST=1` | Injection values `0` | Two-boot result fingerprint parity | Race/performance/closure |
| `ci-gate-execution-marker-determinism` negative | Positive values plus `PHASE17_MARKER_INJECTION_TEST=1`, `MARKER_INJECT_INVALID_ORDER=1`, `MARKER_NEGATIVE_EXPECT_REJECT=1` | Public/worker/race paths `0` | Invalid order rejected before publication | General rollback, performance, closure |
| `ci-gate-execution-public-e2e` | `BCIB_STUB_RESULT_ENABLE=1`, `BCIB_PUBLIC_E2E_SELFTEST=1`, `MARKER_VALIDATION_ENABLE=1`, `RING3_ENTRY_GUARD=1` | `RING3_MASK_IRQ0_FIRST_ENTRY=0`, worker/race/injection `0` | Public `1003 -> 1004` mapped result witness | Real worker completion or payload latency |
| `ci-gate-execution-worker-completion` | `BCIB_WORKER_COMPLETION_SELFTEST=1`, `MARKER_VALIDATION_ENABLE=1`, `RING3_ENTRY_GUARD=1` | Stub/public/race/injection `0`, `RING3_MASK_IRQ0_FIRST_ENTRY=0` | Bounded fixture public `1003 -> 1011 -> 1004` | General interpreter or latency acceptance |
| `ci-gate-execution-timeout-race` | `EXECUTION_RACE_SELFTEST=1`, `MARKER_VALIDATION_ENABLE=1`, `RING3_ENTRY_GUARD=1` | Stub/public/worker/injection `0`, `RING3_MASK_IRQ0_FIRST_ENTRY=0` | One real timer IRQ timeout-wins interleaving | Exhaustive/SMP race or latency acceptance |
| `ci-gate-phase17-performance-acceptance` | Existing governed performance harness only | Phase-17 lifecycle/public/worker/race/injection flags `0` | Locked timer/preemption hot-path acceptance | Validation payload latency or closure alone |
| Phase10 low-half timer witness integration | Gate-owned Phase10 configuration | Phase-17 additions may not pre-mask the required timer witness | Legacy `create -> syscall_entry -> timer_irq` proof remains intact | Phase-17 closure |

In the table, abbreviated flag names retain the `AYKEN_` prefix defined in
Section 2. The executable source of truth remains `Makefile` plus the
corresponding `scripts/ci/gate_execution_*.sh` scripts.

## 4. Accepted Candidate Evidence

| Evidence surface | Candidate result | Authority limit |
|---|---|---|
| Lifecycle, determinism/negative, public E2E, worker completion, timeout-race | PR #144 remote PASS on `f129d4aa` | Candidate evidence; merge/review pending |
| Locked-baseline performance | Run `26370895287` PASS | Hot-path component only |
| Full strict freeze integration | Run `26370895297` PASS | Candidate SHA verified; not official closure |
| Baseline lineage | Generated by authorized run `26370359958` and imported under `baseline-update` review label | Manual inflation is not authorized |

## 5. Closure Review Checklist

Before an official Phase-17 manifest or tag may be prepared:

1. PR #142 is reviewed and accepted into `main`.
2. PR #144 is restacked or retargeted against accepted `main`, with required
   checks rerun if its base or candidate SHA changes.
3. Review confirms baseline lock import lineage from the authorized artifact.
4. Review confirms production defaults remain off and measured performance
   excludes validation-only payload latency.
5. This matrix is either retained as a regression ownership record or replaced
   by a reviewed S2 gate/flag inventory decision.

Phase-17 remains active and formally unclosed until that sequence completes
and an official closure manifest and tag are issued.

---

**Dijital imza / attribution:** Kenan AY - Duzenleyen, Gelistiren,
Olusturan ve Mimari Sorumlu
**Yetki notu:** Belgesel metadata; sistem otoritesi, CI verdict'i veya
runtime karari degildir.
