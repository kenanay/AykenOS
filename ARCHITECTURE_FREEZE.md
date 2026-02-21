# ARCHITECTURE_FREEZE.md

**Project:** AykenOS  
**Version:** 1.1  
**Status:** ACTIVE FREEZE  
**Effective Date:** 2026-02-13  
**Owner:** AykenOS Core Architecture Team  
**Authority:** Kenan AY

---

## 1. Purpose

Bu belge, AykenOS execution-centric mimarisini mimari borç üretmeden kalıcı olarak sabitlemek amacıyla hazırlanmıştır.

### Freeze Süresince:
- ✅ Mimari sözleşmeler değişmez
- ✅ Ring0/Ring3 sınırı delinmez
- ✅ Syscall ABI tek kaynaktan yönetilir
- ✅ Constitutional governance zorunlu merge kapısıdır
- ⛔ Yeni özellik geliştirme mainline'a merge edilmez
- ⛔ Paralel experimentation izole branch'lerde yapılır

**Bu belge bağlayıcıdır.**

---

## 2. Freeze Scope

### 2.1 Dondurulan Alanlar (IMMUTABLE)

#### Syscall v2 Interface
- **ID Range:** 1000-1010 (11 syscalls, fixed)
- **ABI Definition:** `kernel/include/ayken_abi.h` (single source of truth)
- **Register Mapping:** RDI, RSI, RDX, R10 (no alternatives)
- **Generation:** `make generate-abi` (deterministic)

#### Ring0/Ring3 Boundary
- **Ring0:** Mechanism only (memory, context, interrupt, syscall)
- **Ring3:** Policy only (scheduler, VFS, DevFS, AI runtime)
- **Enforcement:** `make ci-gate-boundary` (symbol-scan + evidence report)
- **Linker Export Enforcement:** `KERNEL_EXPORT_POLICY=1` + generated `kernel/include/generated/ring0.exports.map` (`local: *;` fail-closed export surface)
- **Link Contract:** `kernel.elf` excludes `userspace/libayken/*.o`
- **VFS Mechanism Surface:** `kernel/include/vfs_mech.h` + `kernel/fs/vfs_mech.c` (Ring0 only)

#### Scheduler Policy Separation
- **Mechanism:** wake/block, IRQ-tail reschedule (Ring0)
- **Policy:** run-queue decisions, scheduling logic (Ring3)
- **Fallback:** Isolated with feature flag or removed
- **Arbitration Contract (Yol A):** Ring3 `stage_next` = hint, Ring0 = final arbiter (accept/veto + fail-closed)
- **Decision Record:** `docs/architecture-board/decisions/20260214-scheduler-arbitration-contract.md`

#### Capability-Based Security Model
- **Binding:** syscall-only (no kernel bypass)
- **Enforcement:** Test coverage mandatory
- **Validation:** CI gate enforced

#### BCIB Execution Submission Contract
- **Interface:** `sys_v2_submit_execution(1003)`
- **Format:** BCIB v0.2 (frozen)
- **Changes:** Require full RFC + freeze re-evaluation

#### Kernel ↔ Userspace ABI
- **Source:** `kernel/include/ayken_abi.h`
- **Offsets:** Static assert protected
- **Drift:** CI fail trigger

#### Constitutional Compliance Gates
- **AHS Threshold:** ≥ 95 (mandatory)
- **NON_OVERRIDABLE:** Zero violations
- **Waiver Duration:** ≤ 90 days
- **Justification:** Mandatory for Allow/Waiver

#### CI Enforcement Pipeline
- **Gates:** ABI, Boundary, Ring0 Exports, Hygiene, Tooling Isolation, Constitutional, Workspace, Syscall v2 Runtime, Performance
- **Bypass:** Prohibited (no exceptions)
- **Repo Truth (2026-02-14):**
  - Implemented: `ci-gate-abi`, `ci-gate-boundary`, `ci-gate-ring0-exports`, `ci-gate-hygiene`, `ci-gate-tooling-isolation`, `ci-gate-constitutional`, `ci-gate-workspace`, `ci-gate-syscall-v2-runtime`, `ci-gate-performance`, `ci-summarize`
  - Planned (hard-fail stubs): none
  - Strict suite entrypoint: `make ci-freeze`

#### CI Mode: Constitutional Default + Provisional Compatibility
**Status:** ACTIVE (2026-02-21)  
**Default freeze mode:** `PERF_BASELINE_MODE=constitutional` (`.github/workflows/ci-freeze.yml`)  
**Compatibility path:** Provisional mode baseline-init ve diagnostik run'lar icin korunur

**Constitutional Default Behavior (freeze path):**
- ✅ Functional gates merge-blocking olarak calisir
- ✅ Performance gate baseline/env mismatch/regression icin fail-closed calisir
- ✅ Tooling isolation gate strict path'te aktif kalir

**Provisional Compatibility Behavior (limited scope):**
- ⚠️ Runtime/performance icin gevsek esik veya warning/skip yolu kullanilabilir
- ⚠️ Bu yol freeze/merge icin tek basina yeterli kabul edilmez
- ⚠️ Kullanimi baseline-init ve diagnostik senaryolarla sinirlidir

See:
- `docs/operations/CONSTITUTIONAL_CI_MODE.md`
- `docs/operations/PROVISIONAL_CI_MODE.md`

#### Repository Hygiene Rules
- **Tracking:** No `target/`, `obj/`, `*.o`, `*.elf` in git
- **Clean Tree:** `git diff --exit-code HEAD` in CI
- **Artifacts:** Build artifacts must be reproducible

### 2.2 Serbest Alanlar (ALLOWED)

- ✅ Performance optimizations (ABI-preserving)
- ✅ Refactoring (contract-preserving)
- ✅ Test expansion and improvement
- ✅ Benchmark enhancements
- ✅ Documentation updates
- ✅ Bug fixes (non-architectural)
- ✅ Isolated experimentation (non-mainline branches)

---

## 3. Architectural Invariants (Frozen Rules)

### 3.1 Syscall Contract Invariants

#### ID Range (FIXED)
```c
#define SYS_V2_BASE       1000
#define SYS_V2_MAX_INDEX  10
#define SYS_V2_NR         (SYS_V2_MAX_INDEX + 1)
#define SYS_V2_LAST       (SYS_V2_BASE + SYS_V2_MAX_INDEX)
```

**Debug syscall:** Included in-range as index 10 (`SYS_V2_DEBUG_PUTCHAR` → public 1010)

#### Single Source of Truth
```
Canonical ABI Definition: kernel/include/ayken_abi.h
Generation Command:       make generate-abi
Kernel Dispatcher:        Auto-generated from ayken_abi.h
Userspace Wrapper:        Auto-generated from ayken_abi.h
```

#### Register ABI (IMMUTABLE)
| Arg   | Register |
|-------|----------|
| arg0  | RDI      |
| arg1  | RSI      |
| arg2  | RDX      |
| arg3  | R10      |

**Alternative mappings (RBX/RCX/R8/R9) are PROHIBITED.**

#### ABI Offset Guards
```c
_Static_assert(offsetof(struct context, rdi) == CTX_RDI, "ABI drift");
_Static_assert(offsetof(struct context, rsi) == CTX_RSI, "ABI drift");
// ... all registers
```

**ABI drift = CI FAIL**

### 3.2 Ring0/Ring3 Boundary Invariants

#### Ring0 (Mechanism Only)
- Memory primitives (map, unmap, protect)
- Context switch mechanism
- Interrupt handling (entry, dispatch, exit)
- Syscall dispatch (no policy decisions)

#### Ring3 (Policy Only)
- Scheduler policy (which task runs when)
- VFS policy (file access decisions)
- DevFS policy (device access control)
- AI runtime (inference, agents, decisions)
- Execution decision logic (BCIB interpretation)

#### Kernel Fallback Policy
**PROHIBITED.** Temporary fallback must be:
- Feature flag isolated (`AYKEN_SCHED_FALLBACK`)
- Default OFF in all standard builds (`AYKEN_SCHED_FALLBACK ?= 0`)
- Validation-only explicit enable (`AYKEN_SCHED_FALLBACK=1` requires `KERNEL_PROFILE=validation`)
- `make ci-freeze` hard-fail guard enforces fallback disabled
- Removal plan documented with timeline

#### Enforcement
```bash
# CI Gate: Boundary enforcement (symbol-level, deterministic evidence)
make ci-gate-boundary

# Rule sources
# - Deny list:  tools/ci/deny.symbols
# - Allow list: tools/ci/allow.symbols

# Evidence outputs
# - Gate report: evidence/run-<RUN_ID>/gates/symbol-scan/report.json
# - Run summary: evidence/run-<RUN_ID>/reports/summary.json
```

Linker-level export policy is mandatory in freeze mode:
- `KERNEL_EXPORT_POLICY ?= 1` (default)
- kernel link includes `--version-script=$(RING0_EXPORT_MAP)`
- generated map enforces `local: *;` so non-whitelisted globals are not exported

### 3.3 Scheduler Isolation Invariants

#### Mechanism (Ring0)
- `wake()` / `block()` primitives
- IRQ-tail reschedule trigger
- Context switch execution

#### Policy (Ring3)
- Run-queue management
- Priority decisions
- Time slice allocation
- Load balancing
- Next-task staging for Ring0 via scheduler mailbox (Ring0 consumes, Ring3 decides)

#### Scheduler Arbitration Contract (Yol A)
- Ring3 `scheduler_stage_next(...)` çağrısı öneri/hint üretir; seçim emri üretmez.
- Ring0 staged adayı doğrular (registered/state/context sanity) ve son kararı verir.
- Ring0 aday veto edebilir; veto edilen aday context switch'e taşınmaz.
- Scheduler armed olduktan sonra kabul edilebilir aday yoksa fail-closed semantiği uygulanır: `cli; hlt;`.
- Bridge syscall penceresi `0x90..0x9F` aralığında tutulur; `SYS_V2` freeze aralığına dokunulmaz.
- Bridge window (`0x90..0x9F`) scheduler/policy bridge için reserved'dır ve execution-centric `SYS_V2` sözleşmesinin parçası değildir.

**Scheduling decision logic in kernel = VIOLATION**

### 3.4 Capability Model Invariants

- Capability bind: **syscall-only** (`sys_v2_capability_bind`)
- Kernel bypass: **PROHIBITED**
- Enforcement: **Test coverage mandatory** (>95%)
- Validation: **CI gate enforced**

### 3.5 Constitutional Governance Invariants

- **AHS ≥ 95:** Mandatory for merge
- **NON_OVERRIDABLE violation:** Merge reject (no exceptions)
- **Waiver duration:** ≤ 90 days (hard limit)
- **Allow/Waiver justification:** Mandatory (minimum 10 chars + technical content)
- **CI constitutional gate:** Cannot be bypassed

---

## 4. Mandatory Technical Gates (CI Enforcement)

**Implementation state is enforced explicitly:**
- Implemented gates produce evidence and can PASS.
- Baseline-backed gates fail-closed until baseline lock is initialized and committed.
- This prevents silent PASS with missing enforcement.

### 4.1 ABI Gate

**Checks:**
```bash
make ci-gate-abi
```

**Current state:** Implemented (deterministic evidence + baseline lock compare).

**Validations:**
- Syscall header hash verification
- Kernel dispatcher auto-verify
- Userspace wrapper parity check
- ABI offset guard compile-time assert
- Register mapping consistency

**Failure → Merge REJECT**

### 4.2 Boundary Gate

**Checks:**
```bash
make ci-gate-boundary
```

**Validations:**
- Symbol-level deny/allow scan over build artifacts (`tools/ci/symbol-scan.sh`)
- Filtered symbol evidence output (`symbols.filtered.txt`)
- Gate report + run summary evidence (`report.json`, `summary.json`)
- Link map evidence (`evidence/run-<RUN_ID>/artifacts/kernel.map`) for kernel-only link set audit
- Scheduler isolation test (no decision logic in kernel)
- Capability bypass test (no kernel direct access)
- Ring boundary matrix validation

**Failure → Merge REJECT**

### 4.2.1 Ring0 Exports Gate

**Checks:**
```bash
make ci-gate-ring0-exports
```

**Validations:**
- Policy-on deterministic build (`KERNEL_EXPORT_POLICY=1`)
- `nm -g --defined-only kernel.elf` global export evidence
- Whitelist conformance (`constitutional-ring0-symbol-whitelist.regex`)
- Export surface ceiling (`RING0_EXPORT_MAX`, default 165)

**Failure → Merge REJECT**

### 4.3 Workspace Gate

**Scope:**
```
Mandatory Green Workspaces:
- root workspace (kernel + tools)
- userspace/ayken (constitutional system)
- userspace/ayken-core (ABDF/BCIB)
```

**Checks:**
```bash
make ci-gate-workspace
```

**Current state:** Implemented (clean-state + determinism + link-set evidence gate).

**Validations:**
- Git clean state checks (`git diff`, `git diff --cached`, strict untracked handling)
- ABI generated include determinism (`make generate-abi` + drift check)
- ABI baseline lock tracked/clean checks (`scripts/ci/abi-baseline.lock.json`)
- Lightweight reproducibility signal (double clean build + `kernel.elf` hash compare)
- Kernel link-set discipline via map evidence (`userspace/libayken/*.o` and `*_test.o` must not link)

**Partial green = REJECT**  
**Failure → Merge REJECT**

### 4.4 Repo Hygiene Gate

**Checks:**
```bash
make ci-gate-hygiene
```

**Current state:** Implemented (evidence-producing gate).

**Validations:**
- `target/` not tracked
- `obj/` not tracked
- `*.o`, `*.elf` not tracked
- Build artifact diff detection
- Git clean tree enforcement: `git diff --exit-code HEAD` (PR branch only)

**Failure → Merge REJECT**

### 4.5 Performance Regression Gate

**Baseline Definition:**
```
Baseline Commit:  [SHA to be set at freeze start]
Compiler:         rustc 1.76.0 / gcc 14.2.0
Target:           x86_64-unknown-none
QEMU:             8.2.0
Host CPU:         GitHub-hosted ephemeral x86_64 (non-pinned microarchitecture)
Authority:        github-hosted-ubuntu-latest-x64
CI Image Digest:  gha-${ImageOS}-${ImageVersion}-${RUNNER_ARCH}
```

**Thresholds:**
```
x86_64 Stabilized Baseline:
- Syscall latency:      ±5%
- Context switch:       ±5%
- Boot time:            ±10%

New Platform Baseline Establishment:
- All metrics:          ±15% (temporary)

Compiler Upgrade:
- Temporary waiver with justification
```

**Checks:**
```bash
make ci-gate-performance
```

**Current state:** Implemented (baseline lock + env hash + regression compare + evidence).

**Validations:**
- Environment manifest + env hash generation (`env.json`)
- Baseline lock compare (`scripts/ci/perf-baseline.lock.json`)
- Baseline authority lock (`PERF_BASELINE_AUTHORITY=github-hosted-ubuntu-latest-x64`)
- CI image/build digest lock (`PERF_CI_IMAGE_DIGEST`)
- Baseline init with `PERF_CI_IMAGE_DIGEST=unknown` is prohibited (must be pinned)
- CI freeze workflow resolves pinned digest from GitHub hosted image metadata (`ImageOS`, `ImageVersion`, `RUNNER_ARCH`)
- Marker contract lock (`boot_ok_marker`, `preempt_sw_count_pattern`, `preempt_iret_count_pattern`)
- Baseline init authority lock (default CI-only; local init requires explicit override)
- QEMU boot audit timing proxy (`boot_time_ms`)
- Preempt marker timing proxy (`context_switch_latency_ms_proxy`)
- Preempt IRET timing proxy (`syscall_latency_ms_proxy`)
- Threshold policy enforcement (`±5%` syscall/context-switch proxy, `±10%` boot)
- Proxy model disclosure: metrics are wall-time proxies, not cycle-accurate guest counters

**Failure → Manual architecture review**

### 4.6 Constitutional Gate

**Checks:**
```bash
make ci-gate-constitutional
```
Default mode: strict (`CONSTITUTIONAL_STRICT=1`, fail-closed).

**Current state:** Implemented (evidence-producing gate).

**Validations:**
- Ring0 tracked-path whitelist integrity (`scripts/ci/constitutional-ring0-whitelist.regex`)
- Ring0 exported symbol whitelist integrity (`scripts/ci/constitutional-ring0-symbol-whitelist.regex`)
- Kernel source deny/allow scan (`scripts/ci/constitutional-source-deny.regex`, `scripts/ci/constitutional-source-allow.regex`)
- Syscall freeze contract lock (`SYS_V2_BASE/MAX_INDEX/NR/LAST` invariants)
- Scheduler fallback contract lock (`AYKEN_SCHED_FALLBACK` strict-mode=0 + Makefile/header default checks)
- AHS threshold floor checks from `_ayken/steering/AHS_CONFIG.toml` (`P5_minimum >= 95`)
- NON_OVERRIDABLE registry integrity checks from `_ayken/steering/NON_OVERRIDABLE.md`
- Waiver metadata/expiry/duration policy checks under `docs/waivers/`

**Failure → Merge REJECT**

---

## 5. Claim Freeze Rule

**"Completed" or "Production-ready" claims require:**

1. ✅ CI full green (all gates passing)
2. ✅ Test evidence committed
3. ✅ Benchmark results committed
4. ✅ Documentation updated
5. ✅ Architecture review approval

**No evidence = No claim.**

---

## 6. Change Control Procedure

### 6.1 Architectural Changes During Freeze

**Process:**
1. Open RFC (Request for Comments)
2. Provide impact analysis:
   - ABI impact
   - Boundary impact
   - Performance impact
   - Security impact
3. Include regression plan
4. Include rollback plan
5. Obtain Architecture Board approval

**No approval = No merge.**

**Operational Artifacts (repo):**
- `docs/roadmap/freeze-enforcement-workflow.md`
- `docs/operations/SELF_HOSTED_RUNNER_HARDENING.md`
- `docs/rfc/0001-template.md`
- `docs/development/PR_FREEZE_TEMPLATE.md`
- `.github/pull_request_template.md`
- `.github/workflows/ci-freeze.yml`

### 6.2 RFC Template

```markdown
# RFC: [Title]

## Motivation
[Why is this change needed?]

## Impact Analysis
- ABI Impact: [None/Breaking/Compatible]
- Boundary Impact: [Ring0/Ring3 changes]
- Performance Impact: [Measured delta]
- Security Impact: [Risk assessment]

## Regression Plan
[How to detect if this breaks existing functionality]

## Rollback Plan
[How to revert if issues arise]

## Timeline
[Estimated implementation time]
```

---

## 7. Exception Protocol (Waiver)

### 7.1 Critical Exceptions (Immediate Action)

**Allowed only for:**
- Critical security vulnerability
- Kernel crash/blocker
- Data corruption
- Platform-breaking bug

**Requirements:**
- Time-limited (≤ 7 days)
- Tracking issue mandatory
- Fix plan mandatory
- CI note mandatory

**Waivers not closed within 90 days = automatic VIOLATION**

### 7.2 Architectural Exceptions (RFC Required)

**Allowed for:**
- Hardware errata workaround
- Platform bring-up architectural deviation
- Upstream toolchain breaking change
- Performance critical optimization (with trade-off analysis)

**Requirements:**
- Full RFC process
- Architecture Board review
- Documented rollback plan
- Timeline commitment

**Waiver Registry (repo):**
- `docs/waivers/README.md`
- `docs/waivers/WAIVER_TEMPLATE.md`

---

## 8. Freeze Entry Criteria

**Freeze CANNOT start until:**

1. ✅ Syscall ID range finalized (1000-1010)
2. ✅ Userspace syscall register mapping fixed
3. ✅ Scheduler fallback isolated or removed
4. ✅ Tracked build artifacts cleaned
5. ✅ Workspace fully green (all tests passing)
6. ✅ CI gates implemented and active
7. ✅ Performance baseline established
8. ✅ Repo clean baseline created

**Current Status (2026-02-14):**
- ✅ Boundary gate implementation active (`make ci-gate-boundary`)
- ✅ Hygiene gate implementation active (`make ci-gate-hygiene`)
- ✅ Tooling isolation gate implementation active (`make ci-gate-tooling-isolation`)
- ✅ ABI gate implementation active (`make ci-gate-abi`)
- ✅ Constitutional gate implementation active (`make ci-gate-constitutional`)
- ✅ Workspace gate implementation active (`make ci-gate-workspace`)
- ✅ Syscall v2 runtime gate implementation active (`make ci-gate-syscall-v2-runtime`)
- ✅ Performance gate implementation active (`make ci-gate-performance`)
- ✅ Summary gate active (`make ci-summarize`, auto-discovery)
- ✅ Evidence schema active (`evidence/run-<RUN_ID>/reports/summary.json`)
- 🔄 Performance baseline initialization/lock commit required (`PERF_INIT_BASELINE=1`)
- 🔄 Remaining entry criteria tracked in roadmap and CI backlog

---

## 9. Freeze Timeline and Milestones

### 9.1 Duration

- **Target Duration:** 4-8 weeks
- **Maximum Duration:** 12 weeks
- **Review Cadence:** Bi-weekly architecture board meeting

### 9.2 Milestones

**Week 1-2: Baseline Establishment**
- CI gates activation
- Metric collection
- Documentation freeze
- Performance baseline commit

**Week 3-4: Stabilization**
- Ring3 policy hardening
- Scheduler fallback isolation/removal
- Test expansion
- Boundary enforcement validation

**Week 5-6: Validation**
- 30-day CI stability window start
- Performance regression analysis
- Security audit
- Constitutional compliance review

**Week 7-8: Exit Preparation**
- Architecture review
- Freeze exit decision
- Phase 4.5 kickoff planning
- Post-freeze roadmap finalization

**Week 9-12: Extended Validation (if needed)**
- Additional stability testing
- Edge case validation
- Community feedback integration

### 9.3 Progress Tracking

**Daily:**
- CI gate status dashboard
- Automated metric collection

**Weekly:**
- Freeze progress report
- Blocker identification
- Timeline adjustment (if needed)

**Bi-weekly:**
- Architecture board review
- Exit criteria evaluation
- Risk assessment update

---

## 10. AI Integration Compatibility

### 10.1 During Freeze

**ALLOWED:**
- Ring3 AI runtime development (userspace only)
- TinyLLM integration (userspace)
- Shell agent prototyping (userspace)
- AI service experimentation (isolated branches)

**REQUIRES RFC:**
- New syscall for AI operations
- BCIB contract changes
- Capability model extensions
- Kernel-level AI infrastructure

**STRICTLY PROHIBITED:**
- Ring0 AI logic
- Kernel-side inference
- Policy decisions in kernel
- Mainline merge of AI features

### 10.2 Post-Freeze AI Integration

After freeze exit:
- Full AI integration allowed
- Semantic CLI implementation
- Multi-agent orchestration
- AI-native features deployment

---

## 11. Freeze Exit Criteria

**Freeze can ONLY be lifted when:**

1. ✅ Ring3 policy fully hardened (no kernel fallback)
2. ✅ Scheduler fallback removed or isolated with feature flag
3. ✅ Syscall drift = 0 (ABI gate passing 30 days)
4. ✅ CI gates stable for 30 consecutive days
5. ✅ AHS trend not declining (≥ 95 maintained)
6. ✅ Performance regression = 0 (all baselines met)
7. ✅ All freeze-blocking issues resolved
8. ✅ Architecture Board approval

**Exit decision requires:**
- Unanimous Architecture Board vote
- Post-freeze roadmap approval
- Phase 4.5 readiness confirmation
- Decision record in `docs/architecture-board/decisions/`

---

## 12. Freeze Enforcement

### 12.1 Parallel Work Policy

**During freeze:**
- ⛔ **Mainline merge:** Prohibited for new features
- ✅ **Isolated branches:** Allowed for experimentation
- ✅ **Bug fixes:** Allowed (non-architectural)
- ✅ **Documentation:** Allowed and encouraged
- ✅ **Testing:** Allowed and encouraged

**"Freeze bitmeden yeni faz açılmaz" = ABSOLUTE RULE**

### 12.2 Boundary Gate Enforcement

**Automated Detection:**
```bash
# CI gate execution
make ci-gate-boundary

# Deterministic evidence
cat evidence/run-<RUN_ID>/reports/summary.json
```

**Manual Review:**
- Every PR touching `kernel/` requires architecture review
- Ring0/Ring3 boundary changes require RFC
- `tools/ci/deny.symbols` and `tools/ci/allow.symbols` changes require architecture sign-off

---

## 13. Strategic Intent

**This freeze is designed to:**

1. ✅ Stabilize execution-centric architecture before AI integration
2. ✅ Harden multi-platform foundation before ARM64/RISC-V expansion
3. ✅ Validate execution-centric claims with technical evidence
4. ✅ Transform constitutional governance from paper to CI enforcement
5. ✅ Establish AykenOS as reference architecture (not experimental kernel)

**This freeze is NOT:**
- ❌ A slowdown (it's a foundation solidification)
- ❌ A feature freeze (isolated experimentation continues)
- ❌ A permanent state (target: 4-8 weeks)

**Analogy:**
> "Bir binanın temelini dökerken, duvarları örmeye başlamazsınız. Freeze, temel kuruduktan sonra güvenle inşaata devam etmek içindir."

---

## 14. Non-Negotiable Rules

### 14.1 The Three Absolutes

1. **Ring0 CANNOT contain policy logic**
   - No scheduler decisions
   - No VFS access control
   - No AI inference
   - Violation = Immediate merge reject

2. **Syscall ABI CANNOT drift**
   - ID range: 1000-1010 (fixed)
   - Register mapping: RDI/RSI/RDX/R10 (fixed)
   - Single source: `kernel/include/ayken_abi.h`
   - Violation = CI fail + merge reject

3. **CI gates CANNOT be bypassed**
   - No "temporary" gate disabling
   - No "emergency" merge without gates
   - No "we'll fix it later" exceptions
   - Violation = Immediate rollback

**These three rules are NOT open for discussion during freeze.**

---

## 15. Final Statement

AykenOS is not an experimental kernel. It aims to be an **execution-centric reference architecture**.

This freeze is a decision to **solidify the foundation before scaling**.

**Freeze bitmeden yeni faz açılmaz.**

This document is **binding** and **enforceable** through CI gates.

---

## 16. Document Control

**Version:** 1.1  
**Status:** ACTIVE  
**Effective Date:** 2026-02-13  
**Review Date:** Bi-weekly  
**Next Review:** [To be scheduled]  
**Approval Authority:** AykenOS Architecture Board  
**Document Owner:** Kenan AY

**Revision History:**
| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.1 | 2026-02-13 | Kenan AY | Boundary enforcement updated to symbol-scan + deterministic evidence schema |
| 1.0 | 2026-02-13 | Kenan AY | Initial freeze document |

---

**© 2026 Kenan AY - AykenOS Project**

**This document is the law of the land during freeze. Enforce it.**
