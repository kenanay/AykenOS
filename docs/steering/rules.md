# AykenOS Constitutional Rules

**Version:** 1.0  
**Authority:** ARCHITECTURE_FREEZE.md  
**Enforcement:** CI Gates + Branch Protection  
**Status:** ACTIVE

This document defines non-negotiable rules that MUST be followed by all contributors.

---

## Rule 1: Ring0 Policy Prohibition

**Statement:** Ring0 code SHALL NOT contain policy decisions.

**Rationale:** Mechanism/policy separation is foundational to AykenOS architecture.

**Enforcement:**
- CI Gate: `make ci-gate-boundary`
- Symbol scanning: `tools/ci/symbol-scan.sh`
- Deny list: `tools/ci/deny.symbols`

**Violations:**
- Scheduler logic in kernel → **PR AUTO-REJECT**
- VFS access control in kernel → **PR AUTO-REJECT**
- AI inference in kernel → **PR AUTO-REJECT**
- File access decisions in kernel → **PR AUTO-REJECT**

**Exceptions:** NONE (non-overridable)

---

## Rule 2: ABI Stability

**Statement:** Syscall ABI (1000-1011) is FROZEN. Changes require RFC + version bump.

**Rationale:** ABI stability is critical for Ring3 compatibility.

**Enforcement:**
- CI Gate: `make ci-gate-abi`
- Single source: `kernel/include/ayken_abi.h`
- Baseline lock: `scripts/ci/abi-baseline.lock.json`

**Requirements:**
- ABI change → `AYKEN_ABI_VERSION` MUST increment
- New syscall → RFC approval required
- Register mapping change → **PROHIBITED**
- Syscall ID range expansion → **PROHIBITED**

**Exceptions:** Security vulnerabilities only (with ADR)

---

## Rule 3: Ring0 Export Surface

**Statement:** Ring0 exports are constitutional surface. New exports require ADR.

**Rationale:** Export surface defines kernel API contract.

**Enforcement:**
- CI Gate: `make ci-gate-ring0-exports`
- Whitelist: `scripts/ci/constitutional-ring0-symbol-whitelist.regex`
- Ceiling: 165 symbols (hard limit)

**Requirements:**
- New export → ADR required
- Export removal → version bump required
- Ceiling breach → **CI FAIL**

**Exceptions:** NONE (non-overridable)

---

## Rule 4: Evidence Integrity

**Statement:** Evidence directory is immutable. Manual modification is prohibited.

**Rationale:** Evidence-based governance requires tamper-proof records.

**Enforcement:**
- CI Gate: `make ci-gate-hygiene`
- Directory: `evidence/run-<RUN_ID>/`
- Append-only policy

**Requirements:**
- Evidence MUST be committed
- Evidence MUST NOT be modified after creation
- Baseline locks require authorized workflow
- Manual evidence edit → **VIOLATION**

**Exceptions:** NONE (non-overridable)

---

## Rule 5: Determinism Requirement

**Statement:** All behavior MUST be deterministic and reproducible.

**Rationale:** Determinism enables evidence-based validation.

**Enforcement:**
- CI Gate: `make ci-gate-performance`
- Baseline: `scripts/ci/perf-baseline.lock.json`
- Authority: GitHub-hosted runner environment

**Requirements:**
- No busy-loop timing hacks
- Tick-based regression injection only
- CI reproducibility mandatory
- Performance regression requires evidence

**Exceptions:** Platform-specific errata (with waiver)

---

## Rule 6: Constitutional Compliance

**Statement:** All code MUST pass constitutional checks.

**Rationale:** Constitutional governance ensures architectural health.

**Enforcement:**
- CI Gate: `make ci-gate-constitutional`
- Tool: `ayken check`
- AHS threshold: ≥ 95

**Requirements:**
- NON_OVERRIDABLE violations → **PR REJECT**
- Waiver duration ≤ 90 days
- Justification mandatory
- Locked modules immutable

**Exceptions:** Emergency security fixes (with tracking issue)

---

## Rule 7: Documentation Synchronization

**Statement:** Architectural changes MUST update documentation.

**Rationale:** Undocumented behavior is undefined behavior.

**Enforcement:**
- Hook: `doc-sync-mandatory.kiro.hook`
- Manual review required

**Requirements:**
- ABI change → update syscall guide
- Boot change → update setup guides
- Build change → update tech.md
- Freeze change → update roadmap

**Exceptions:** NONE (non-overridable)

---

## Rule 8: Clean Git State

**Statement:** Tracked files MUST be clean before merge.

**Rationale:** Dirty state indicates incomplete work.

**Enforcement:**
- CI Gate: `make ci-gate-hygiene`
- Check: `git diff --exit-code HEAD`

**Requirements:**
- No modified tracked files
- No tracked build artifacts
- No tracked binaries (unless whitelisted)

**Exceptions:** NONE (non-overridable)

---

## Rule 9: Syscall Interface Stability

**Statement:** Syscall interface (1000-1011) is FROZEN.

**Rationale:** Ring3 depends on stable syscall contract.

**Enforcement:**
- CI Gate: `make ci-gate-syscall-v2-runtime`
- Runtime validation required

**Requirements:**
- Syscall count: 12 (fixed)
- Register mapping: RDI/RSI/RDX/R10 (fixed)
- Return convention: RAX (fixed)
- Error convention: negative errno (fixed)

**Exceptions:** NONE (non-overridable)

---

## Rule 10: Baseline Lock Authority

**Statement:** Baseline locks MUST be updated via authorized workflow only.

**Rationale:** Baseline integrity prevents silent regressions.

**Enforcement:**
- CI Gate: `make ci-gate-performance`
- Authority: `PERF_BASELINE_AUTHORITY` environment variable

**Requirements:**
- Baseline init requires CI environment
- Local baseline init → **PROHIBITED** (unless override)
- Baseline change requires evidence
- Unauthorized baseline change → **CI FAIL**

**Exceptions:** Authorized maintainers only (with ADR)

---

## Violation Response Protocol

### Severity Levels

**Critical (PR Auto-Reject):**
- Ring0 policy code
- ABI breaking change without RFC
- Evidence tampering
- Baseline lock bypass

**High (CI Fail):**
- Export surface breach
- Constitutional violation
- Dirty git state
- Performance regression without evidence

**Medium (Manual Review):**
- Documentation out of sync
- Test coverage below threshold
- Waiver expiry approaching

### Response Actions

**For Critical Violations:**
1. PR automatically rejected
2. Violation logged in audit trail
3. Architecture Board notification
4. Remediation plan required

**For High Violations:**
1. CI fails with evidence
2. Manual review required
3. Fix required before merge
4. No bypass allowed

**For Medium Violations:**
1. Warning issued
2. Tracking issue created
3. Timeline for fix established
4. Waiver may be granted (≤90 days)

---

## Rule Amendment Process

Constitutional rules can only be amended via:

1. RFC submission
2. Architecture Board review
3. Unanimous approval
4. ADR documentation
5. Version bump

**Emergency amendments** (security only):
- Temporary waiver (≤7 days)
- Tracking issue mandatory
- Retroactive ADR required

---

## Enforcement Hierarchy

```
1. CI Gates (automated, fail-closed)
2. Branch Protection (GitHub, mandatory)
3. CODEOWNERS (manual review, required)
4. Architecture Board (final authority)
```

All levels MUST pass for merge.

---

**Maintained by:** AykenOS Architecture Board  
**Last Updated:** 2026-02-22  
**Next Review:** Bi-weekly during freeze

**This document is binding. Violations result in PR rejection.**
