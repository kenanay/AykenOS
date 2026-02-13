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
- **ID Range:** 1000-1009 (10 syscalls, fixed)
- **ABI Definition:** `kernel/include/ayken_abi.h` (single source of truth)
- **Register Mapping:** RDI, RSI, RDX, R10 (no alternatives)
- **Generation:** `make generate-abi` (deterministic)

#### Ring0/Ring3 Boundary
- **Ring0:** Mechanism only (memory, context, interrupt, syscall)
- **Ring3:** Policy only (scheduler, VFS, DevFS, AI runtime)
- **Enforcement:** `make ci-gate-boundary` (symbol-scan + evidence report)

#### Scheduler Policy Separation
- **Mechanism:** wake/block, IRQ-tail reschedule (Ring0)
- **Policy:** run-queue decisions, scheduling logic (Ring3)
- **Fallback:** Isolated with feature flag or removed

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
- **Gates:** ABI, Boundary, Workspace, Hygiene, Performance
- **Bypass:** Prohibited (no exceptions)
- **Repo Truth (2026-02-13):**
  - Implemented: `ci-gate-boundary`, `ci-summarize`
  - Planned (hard-fail stubs): `ci-gate-abi`, `ci-gate-workspace`, `ci-gate-hygiene`, `ci-gate-performance`
  - Strict suite entrypoint: `make ci-freeze`

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
#define SYS_V2_BASE  1000
#define SYS_V2_LAST  1009
#define SYS_V2_COUNT 10
```

**Debug syscalls:** Separate namespace or removed (not in 1000-1009 range)

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
- Feature flag isolated (`#ifdef FALLBACK_POLICY`)
- CI boundary violation test enforced
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
- Planned gates exist as hard-fail stubs (`exit 2`) until fully implemented.
- This prevents silent PASS with missing enforcement.

### 4.1 ABI Gate

**Checks:**
```bash
make ci-gate-abi
```

**Current state:** Planned hard-fail stub (not implemented yet).

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
- Scheduler isolation test (no decision logic in kernel)
- Capability bypass test (no kernel direct access)
- Ring boundary matrix validation

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

**Current state:** Planned hard-fail stub (not implemented yet).

**Validations:**
- `cargo test --workspace` full green
- Clippy warnings = 0
- Kernel build warnings = 0
- Reproducible build check

**Partial green = REJECT**  
**Failure → Merge REJECT**

### 4.4 Repo Hygiene Gate

**Checks:**
```bash
make ci-gate-hygiene
```

**Current state:** Planned hard-fail stub (not implemented yet).

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
Host CPU:         [Specific model]
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

**Current state:** Planned hard-fail stub (not implemented yet).

**Validations:**
- Syscall latency baseline comparison
- Context switch latency comparison
- Preempt determinism test passing
- Boot time regression check

**Failure → Manual architecture review**

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
- `docs/rfc/0001-template.md`
- `docs/development/PR_FREEZE_TEMPLATE.md`
- `.github/pull_request_template.md`

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

1. ✅ Syscall ID range finalized (1000-1009)
2. ✅ Userspace syscall register mapping fixed
3. ✅ Scheduler fallback isolated or removed
4. ✅ Tracked build artifacts cleaned
5. ✅ Workspace fully green (all tests passing)
6. ✅ CI gates implemented and active
7. ✅ Performance baseline established
8. ✅ Repo clean baseline created

**Current Status (2026-02-13):**
- ✅ Boundary gate implementation active (`make ci-gate-boundary`)
- ✅ Summary gate active (`make ci-summarize`, auto-discovery)
- ✅ Evidence schema active (`evidence/run-<RUN_ID>/reports/summary.json`)
- 🔄 ABI/Workspace/Hygiene/Performance gates tracked as planned hard-fail stubs
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
   - ID range: 1000-1009 (fixed)
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
