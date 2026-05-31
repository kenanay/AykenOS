# Constitutional CI Gate Inventory and Debt Control - 2026-05-25

This inventory is subordinate to `ARCHITECTURE_FREEZE.md` and the active
freeze ordering in `Makefile`. It records existing enforcement; it does not
promote a diagnostic lane, change runtime behavior, or grant closure.

**Status:** S2 OPERATIONAL INVENTORY / REVIEW INPUT
**Effective date:** 2026-05-25
**Current phase:** Phase-17 officially closed; Phase-18 transition not activated
**Accepted-main evidence subject:** `e0286c7b64c15e27f810e634713a07652def169c`
(`ci-gate-phase17-performance-acceptance` run `26421686338` PASS and full
`ci-freeze` run `26421295459` PASS)
**Authority alignment:** GitHub issue #145 is resolved through the accepted
single-maintainer decision and matching live `main` protection; PR #144 and
workflow authority repair PR #148 are merged. Exact-SHA PASS supports
closure-candidate review only, not official closure.
**Duzenleyen / Gelistiren / Olusturan / Mimari Sorumlu:** Kenan AY
**Attribution boundary:** Documentation metadata only; not runtime, evidence,
merge, baseline, or closure authority.

## 1. Purpose

The strict chain has reached a size where a gate must be justified by the
invariant it protects, the evidence it emits, and the execution cost it adds.
This document supplies the S2 inventory required by the stabilization
roadmap.

It separates three surfaces:

1. `ci-freeze` authority targets: the strict remote acceptance chain.
2. Composite targets expanded behind that chain: hidden cost and ownership
   within an apparent single gate.
3. Phase-17 candidate/diagnostic lanes: useful evidence that is intentionally
   outside strict closure authority unless separately reviewed and promoted.

## 2. Inventory Rules

| Rule | Requirement |
|---|---|
| Gate admission | A new gate is allowed only for a new protected invariant or materially new failure mode. |
| Duplication | Same risk class plus same protected surface means extend or compose an existing gate, not add a parallel verdict. |
| Evidence boundary | Evidence describes a verdict; it is not scheduler, execution, policy, or baseline input. |
| Performance boundary | Validation-only payload latency does not enter the locked timer/preemption performance authority. |
| Ownership boundary | `CODEOWNERS` maps accountability to `@kenanay`; required remote `freeze` and live protection enforce the accepted single-maintainer model, without claiming independent self-review. |
| Change protocol | Gate order, promotion into `ci-freeze`, baseline policy, or ownership enforcement requires reviewed governance change and new remote CI. |

## 3. Cost Classes and Ownership

Cost classes are qualitative execution classes, not timing claims:

| Class | Meaning |
|---|---|
| `S` | Source, metadata, hash, policy, or static contract scan. |
| `B` | Build, generated artifact, symbol, binary, or workspace compile/fixture work. |
| `Q1` | One bounded QEMU/runtime witness path. |
| `QN` | Multiple runtime samples/boots or performance sampling. |
| `R` | Rust/package test execution. |
| `C` | Composite target that expands to multiple gates and may contain `S`, `R`, or runtime work. |

Unless noted otherwise, a change to the Make target, workflow or script has
accountable ownership `@kenanay` through `.github/CODEOWNERS`. Source
changes retain the same documented maintainer accountability. Under the
single-maintainer decision, this mapping is not an independent approval;
required remote `freeze` and recorded maintainer decisions remain the merge
boundary.

## 4. Strict Freeze Preconditions

These are mandatory guards before the 40 strict gate/cluster targets; they
are not counted as gates.

| Target | Protected invariant | Cost | Failure meaning | Evidence/output |
|---|---|---:|---|---|
| `ci-freeze-guard` | Strict configuration cannot run with fallback policy, PCID drift, disabled Phase10-C enforcement, or bootstrap policy enabled. | `S` | The run is not a valid strict authority configuration. | Fail-fast console verdict. |
| `preflight-mode-guard` | `USER_MINIMAL_MODE` must be scoped at gate call sites, not ambient globally exported state. | `S` | Runtime evidence could be contaminated by external mode state. | Fail-fast console verdict. |

## 5. Strict `ci-freeze` Inventory

The order below matches `Makefile` and
`docs/roadmap/freeze-enforcement-workflow.md`. `PHASE10C_FREEZE_GATE`
resolves to `ci-gate-scheduler-mailbox-phase10c` in strict mode.

| # | Target | Protected invariant / risk | Cost | Primary evidence surface | Overlap decision |
|---:|---|---|---:|---|---|
| 1 | `ci-gate-abi` | Frozen canonical ABI and baseline parity; no unreviewed syscall/layout drift. | `S` | `reports/abi.json` | Keep primary ABI source gate. |
| 2 | `ci-gate-boundary` | Ring0 mechanism / Ring3 policy boundary and symbol discipline. | `B` | symbol/boundary report | Keep primary boundary gate. |
| 3 | `ci-gate-ring0-exports` | Ring0 export ceiling and whitelist stability. | `B` | `reports/ring0-exports.json` | Keep separate export-surface verdict. |
| 4 | `ci-gate-hygiene` | Repository input hygiene for authoritative CI. | `S` | `reports/hygiene.json` | Keep early fail-fast guard. |
| 5 | `ci-gate-execution-slot-integrity` | Production execution-slot implementation cannot regress to prototype/stub damage. | `S` | `reports/execution-slot-integrity.json` | Keep until lifecycle ownership is closed. |
| 6 | `ci-gate-execution-marker-isolation` | Marker/test injection remains default-off and isolated from production authority. | `S` | `reports/execution-marker-isolation.json` | Keep; complements runtime evidence lanes. |
| 7 | `ci-gate-tooling-isolation` | Tooling/evidence cannot become authority input. | `S` | `reports/tooling-isolation.json` | Review with observation checks after closure. |
| 8 | `ci-gate-constitutional` | Frozen constitutional contract surface remains intact. | `S` | `reports/constitutional.json` | Keep primary hard lock. |
| 9 | `ci-gate-governance-policy` | Policy, waiver and architecture-health rules remain fail-closed. | `S/B` | `reports/governance-policy.json` | Keep policy tier separate from hard lock. |
| 10 | `ci-gate-naming-convention` | Forward-only naming contract prevents new authority ambiguity. | `S` | naming convention report | Keep; assess scope clarity only. |
| 11 | `ci-gate-spec-purity` | Normative specifications do not absorb implementation syntax. | `S` | script/workflow verdict | Keep static pre-runtime guard. |
| 12 | `ci-gate-drift-activation` | Constitutional drift blocking inputs remain activated and consistent. | `S` | drift activation report | Keep; feeds no runtime decision. |
| 13 | `ci-gate-structural-abi` | Generated/build-consumed ABI surface matches frozen structure. | `S/B` | structural ABI report | Retain as build-surface complement to #1. |
| 14 | `ci-gate-runtime-marker-contract` | Runtime markers retain ratified format and enforcement setting. | `S` | marker contract report | Retain; differs from Phase-17 sequence evidence. |
| 15 | `ci-gate-user-bin-lock` | Embedded user payload binary hash is immutable for the profile. | `B` | user binary lock report | Retain fixture identity check. |
| 16 | `ci-gate-embedded-elf-hash` | Embedded ELF payload matches locked user artifact. | `B` | embedded ELF hash report | Review jointly with #15 for shared primitive only. |
| 17 | `ci-gate-performance` | Locked environment and timer/preemption baseline do not regress. | `QN` | `reports/performance.json` | Keep unique performance authority. |
| 18 | `ci-gate-ring3-user-leaf-rule` | Ring3 executable user-leaf allocation/entry rule holds at runtime. | `Q1` | ring3 user-leaf report/log | Keep boundary runtime witness. |
| 19 | `ci-gate-ring3-execution-phase10a2` | First Ring3 execution/runtime transition remains operational. | `Q1` | Phase10-A2 runtime report/log | Shared prerequisite; retain. |
| 20 | `ci-gate-syscall-semantics-phase10b` | Invalid syscall transition fails closed with proof evidence. | `Q1` | syscall/proof reports | Retain negative semantic coverage. |
| 21 | `ci-gate-low-half-kheap-scaffold` | Legacy low-half runtime witness includes timer IRQ sequence. | `Q1` | low-half proof report/log | Retain regression witness after PR-4D. |
| 22 | `ci-gate-scheduler-mailbox-phase10c` | Strict scheduler mailbox metadata/ownership contract holds. | `Q1` | scheduler mailbox report/log | Strict-only resolved conditional target. |
| 23 | `ci-gate-mailbox-capability-negative` | Capability bypass/invalid mailbox actions reject. | `Q1` | mailbox negative report | Retain fail-closed complement to #22. |
| 24 | `ci-gate-workspace` | Workspace authority and strict multi-component validation remain intact. | `B/R` | `reports/workspace.json` | Retain cross-workspace boundary. |
| 25 | `ci-gate-syscall-v2-runtime` | Public frozen syscall v2 runtime path remains executable. | `Q1` | syscall-v2 runtime report/log | Retain ABI runtime witness. |
| 26 | `ci-gate-sched-bridge-runtime` | Scheduler bridge markers and epoch transitions remain valid. | `Q1` | sched bridge report/log | Retain bridge-specific coverage. |
| 27 | `ci-gate-behavioral-suite` | Phase behavioral acceptance remains fail-closed. | `Q1` | behavioral suite report | Inventory internal overlap in later S2 pass. |
| 28 | `ci-gate-policy-accept` | Ring3 policy acceptance contract holds without kernel policy drift. | `Q1` | policy acceptance report | Retain policy boundary witness. |
| 29 | `ci-gate-alias-proof` | Address-space alias proof prevents hidden mapping dependency. | `Q1` | alias-proof report/log | Retain strict-only memory boundary. |
| 30 | `ci-kill-switch-phase13` | Distributed diagnostics/proof surfaces do not become authority/control. | `C` | `reports/kill_switch_summary.json` plus child reports | Expand below; keep composite authority summary. |
| 31 | `ci-gate-determinism-replay-consistency` | Verification replay remains deterministic and artifact-bound. | `C` | determinism reports/artifacts | Retain; separate from kernel result fingerprint lane. |
| 32 | `ci-gate-bcib-v3-core` | BCIB core determinism, fail-closed and memory model contracts hold. | `R` | `reports/bcib-v3-core.json` | Keep userspace core contract. |
| 33 | `ci-gate-toolchain-opcode-registry` | Opcode IDs and compatibility fixtures stay locked. | `R` | opcode registry report/log | Keep toolchain ABI complement. |
| 34 | `ci-gate-capability-manager` | Token capability enforcement has no bypass. | `R` | `reports/capability-manager.json` | Keep userspace security contract. |
| 35 | `ci-gate-proofd-observability-boundary` | Observability endpoint remains descriptive, never control plane. | `S/R` | proofd boundary reports | Also child of #30; dedup candidate. |
| 36 | `ci-gate-dsl-bcib-contract` | DSL to BCIB IR golden mapping remains compatible. | `R` | `reports/dsl-bcib-contract.json` | Keep contract witness. |
| 37 | `ci-gate-semantic-cli-contract` | CLI to DSL regression boundary remains stable. | `R` | `reports/semantic-cli-contract.json` | Keep tooling contract witness. |
| 38 | `ci-gate-data-runtime-bcib` | Data query path remains mediated through BCIB contract. | `R` | `reports/data-runtime-bcib.json` | Keep data/runtime boundary. |
| 39 | `ci-gate-ai-runtime-boundary` | AI runtime remains suggestion-only and capability-gated. | `R` | `reports/ai-runtime-boundary.json` | Keep Ring3 authority boundary. |
| 40 | `ci-gate-bcib-stub-determinism` | Stub-mode kernel result remains deterministic pending wider worker authority. | `Q1` | `reports/bcib-stub-determinism.json` | Review retirement after Phase-17 closure. |

## 6. Composite Expansion: `ci-kill-switch-phase13`

The strict list counts this as one gate/cluster, but its cost and evidence
surface contain thirteen child gates. The composite summary remains useful;
the hidden fan-out must be visible for maintenance decisions.

| Child gate | Protected invariant | Cost class | Consolidation note |
|---|---|---:|---|
| `ci-gate-proof-receipt` | Receipt remains artifact-bound. | `C` | Keep as proof chain prerequisite. |
| `ci-gate-proof-verdict-binding` | Verdict cannot detach from proof subject. | `C` | Keep primary verdict binding. |
| `ci-gate-verifier-authority-resolution` | Verifier validity does not silently confer authority. | `C` | Keep authority boundary. |
| `ci-gate-cross-node-parity` | Parity remains diagnostics, not truth election. | `C` | Keep distributed artifact check. |
| `ci-gate-proofd-service` | Verification/diagnostics service contract remains deterministic. | `C` | Shared service prerequisite. |
| `ci-gate-proofd-schema-coverage` | Service schema coverage remains complete. | `S/R` | Keep as service contract complement. |
| `ci-gate-proofd-observability-boundary` | Diagnostics namespace remains read-only. | `S/R` | Duplicate invocation candidate with strict row #35. |
| `ci-gate-graph-non-authoritative-contract` | Graph does not infer truth. | `S/R` | Keep risk-specific verdict. |
| `ci-gate-convergence-non-election-boundary` | Convergence does not elect authority. | `S/R` | Keep risk-specific verdict. |
| `ci-gate-diagnostics-consumer-non-authoritative-contract` | Diagnostics are not execution input. | `S` | Candidate shared scan primitive only. |
| `ci-gate-diagnostics-callsite-correlation` | Diagnostics do not flow into decision sinks. | `S` | Candidate shared correlation primitive only. |
| `ci-gate-observability-routing-separation` | Observability does not steer routing/scheduling. | `S/R` | Keep boundary-specific verdict. |
| `ci-gate-verifier-reputation-prohibition` | History does not become trust score. | `S/R` | Keep risk-specific verdict. |

## 7. Phase-17 Lanes Outside Strict Freeze

These targets produce closure-candidate evidence in separate workflows. They
must not be presented as strict-chain members merely because their latest
remote run is green.

| Target | Status at accepted subject `e0286c7b` | Cost | Proves | Does not prove |
|---|---|---:|---|---|
| `ci-gate-execution-marker-lifecycle` | Remote PASS (`26421686302`) | `Q1` | One marker-enabled kernel lifecycle. | Public ABI, worker completion or closure. |
| `ci-gate-execution-marker-determinism` | Remote PASS (`26421686320`) | `QN` | Repeat fingerprint and invalid-order rejection. | General race/performance or closure. |
| `ci-gate-execution-public-e2e` | Remote PASS (`26421686322`) | `Q1` | Public `1003 -> 1004` mapped result path with bounded stub. | Real worker completion. |
| `ci-gate-execution-worker-completion` | Remote PASS (`26421686303`) | `Q1` | Bounded fixture public `1003 -> 1011 -> 1004`. | General BCIB interpreter coverage. |
| `ci-gate-execution-timeout-race` | Remote PASS (`26421686331`) | `Q1` | One timeout-wins/late-completion rejection interleaving. | Exhaustive or SMP race safety. |
| `ci-gate-phase17-performance-acceptance` | Remote PASS (`26421686338`) | `QN` | Locked timer/preemption hot-path acceptance. | Validation payload latency or closure alone. |
| `ci-gate-phase17-performance-readiness-local` | Diagnostic only | `QN` | Local median/stability signal. | Remote baseline authority. |
| `ci-gate-phase17-performance-variance-diagnostic` | Diagnostic only | `S` | Existing-evidence outlier classification. | Root cause or acceptance. |
| `ci-gate-phase17-performance-variance-isolation` | Diagnostic only | `QN` | Bounded reproduction attempt. | Root cause or acceptance. |

## 8. Debt Findings and Decisions

| ID | Finding | Risk | Decision | Priority |
|---|---|---|---|---|
| `S2-D1` | `Makefile` declares the identical `ci-freeze` prerequisite chain twice on consecutive rules. | Maintenance drift and confusing order reviews; not a proven double-execution bug. | Remove duplication in a narrowly reviewed CI-maintenance PR with unchanged dry-run order and full remote recheck. | Next maintenance PR after current stacked review decision. |
| `S2-D2` | Existing `GATE_REGISTRY.md` explicitly covers only a partial Phase-12/13 surface while strict freeze now presents 40 upper-level targets. | Reviewers cannot map all enforcement to invariants from one surface. | Treat this inventory as the S2 operational complement; later consolidate into a versioned registry without changing gate authority. | Started here. |
| `S2-D3` | `ci-kill-switch-phase13` hides thirteen child targets behind one strict row. | Runtime/cost growth and duplicated invocation can be missed. | Preserve composite verdict; expose expansion and audit duplicate child execution. | Open. |
| `S2-D4` | `ci-gate-proofd-observability-boundary` is both a kill-switch child and a subsequent top-level strict prerequisite. | Potential repeated execution/evidence churn for the same protected surface. | Measure actual invocation/evidence behavior before changing dependencies; consolidate only if the same invariant and evidence are preserved. | Open measurement. |
| `S2-D5` | Phase-17 validation workflows run separately from strict freeze. | Additional CI cost and mistaken authority interpretation. | Preserve separation through closure review; promotion or consolidation requires an explicit post-closure decision. | Intentional. |
| `S2-D6` | Validation flags and conditional profiles remain manually coordinated in Makefile/workflows. | Invalid-state combinations and validation/production divergence. | Continue `VALIDATION_FLAG_MATRIX.md`; define a machine-checkable matrix only after current closure path and review enforcement are settled. | Queued. |
| `S2-D7` | Declared CODEOWNERS authority previously did not match live configuration. | A generic approval could be mistaken for architectural authority. | Resolved by the single-maintainer decision, `@kenanay` ownership map and aligned live protection; the result does not confer closure. | Resolved (#145). |
| `S2-D8` | Operational CI-mode and baseline runbook documentation predated the locked Phase-17 performance and 40-target strict-chain model. | Provisional diagnostics, baseline artifacts or smoke PASS could be confused with constitutional acceptance, merge authority or closure; an obsolete gate list could be reviewed as authority. | Synchronize `CONSTITUTIONAL_CI_MODE.md`, `PROVISIONAL_CI_MODE.md`, `PERF_BASELINE_POLICY.md`, `BASELINE_RENEWAL_PROCEDURE.md` and `POST_MERGE_SMOKE_TEST.md` to canonical Makefile/workflows and reference this inventory instead of duplicating authority claims. | Implemented in current documentation changeset; new-head CI required. |

## 9. Execution Order for S2 Remediation

1. Preserve issue #145 resolution through the single-maintainer decision,
   `@kenanay` ownership metadata and enforced remote `freeze`.
2. Preserve accepted-main exact-SHA Phase-17 evidence through
   `reports/phase17_official_closure_candidate/`; do not treat it as an
   official closure record.
3. In a separate CI-maintenance PR, remove only the duplicated `ci-freeze`
   declaration and demonstrate identical expanded target order.
4. Measure whether the repeated proofd observability boundary target produces
   duplicated execution/evidence within one strict run.
5. Convert confirmed duplication into a reviewed consolidation proposal; do
   not remove a verdict merely to reduce run time.
6. Treat the accepted operational CI-mode synchronization and workflow
   authority repair as evidence inputs only; neither is Phase-17 closure.
7. After Phase-17 closure authority is established, decide whether candidate
   S1 workflows remain regression lanes, are promoted, or are retired.

## 10. Acceptance for This Inventory

This S2 inventory is complete as a review input when:

1. all 40 strict targets, both preconditions and the Phase-13 composite
   expansion are visible;
2. separate Phase-17 evidence lanes are explicitly non-promoted;
3. debt findings do not weaken existing fail-closed behavior;
4. the active roadmap references this inventory;
5. required governance/spec/documentation checks pass for the documentation
   changeset.

It is not a Phase-17 closure manifest. Issue #145 resolution removes the
former review-configuration blocker only; PR #144/#148 acceptance and
exact-SHA evidence now support a closure-candidate record, while official
closure authority remains separate.

## References

- `ARCHITECTURE_FREEZE.md`
- `Makefile`
- `docs/roadmap/freeze-enforcement-workflow.md`
- `docs/roadmap/CONSTITUTIONAL_STABILIZATION_ROADMAP_2026_05_23.md`
- `docs/specs/phase12-trust-layer/GATE_REGISTRY.md`
- `docs/specs/phase17-execution-pipeline/VALIDATION_FLAG_MATRIX.md`
- `docs/operations/CONSTITUTIONAL_CI_MODE.md`
- `docs/operations/PROVISIONAL_CI_MODE.md`
- `docs/operations/PERF_BASELINE_POLICY.md`
- `docs/operations/BASELINE_RENEWAL_PROCEDURE.md`
- `docs/operations/POST_MERGE_SMOKE_TEST.md`
- `docs/architecture-board/decisions/20260525-single-maintainer-authority-model.md`
- GitHub issue #145 resolution record: `https://github.com/kenanay/AykenOS/issues/145`

---

**Dijital imza / attribution:** Kenan AY - Duzenleyen, Gelistiren,
Olusturan ve Mimari Sorumlu
**Yetki notu:** Belgesel metadata; sistem otoritesi, CI verdict'i veya
runtime karari degildir.
