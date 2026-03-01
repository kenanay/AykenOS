# Phase 10: Determinism Achieved - Baseline Ready

**Date:** 2026-03-01  
**Status:** BASELINE-READY  
**Authority:** Authoritative CI regeneration required  

---

## Executive Summary

System has achieved deterministic execution with 100% reproducible behavior across 3 consecutive runs. The runtime is now measurable, predictable, and ready for authoritative baseline lock.

---

## Determinism Verification

### 3 Consecutive Runs (Local macOS arm64)

```
Run 1: SW=62 IRET=62 Exit=1 Timeout=0 Proof=1 Time=11163ms
Run 2: SW=62 IRET=62 Exit=1 Timeout=0 Proof=1 Time=11220ms
Run 3: SW=62 IRET=62 Exit=1 Timeout=0 Proof=1 Time=11230ms
```

**Determinism Metrics:**
- ✅ Cadence: 100% deterministic (62/62 constant)
- ✅ Exit: Deterministic (rc=1 constant)
- ✅ Timeout: None (0 constant)
- ✅ Proof: Deterministic (proof_done=1 constant)
- ✅ QEMU Jitter: ~67ms (acceptable, kernel-external)

**Evidence Locations:**
- `evidence/run-20260301T151444Z-030ed1d2-7646/`
- `evidence/run-20260301T151519Z-030ed1d2-8858/`
- `evidence/run-20260301T151554Z-030ed1d2-10056/`

---

## Contract Evolution (Intentional)

### Old Baseline (2026-02-26)
```json
{
  "measurement_contract": null,
  "preempt_sw_count": 39408,
  "preempt_iret_count": 39408,
  "preempt_qemu_run_time_ms": 30027,
  "preempt_deterministic_exit": null
}
```

### New Contract (2026-03-01)
```json
{
  "measurement_contract": "deterministic_preempt_harness",
  "mark_sw_count": 62,
  "mark_iret_count": 62,
  "qemu_run_time_ms": ~11200,
  "preempt_deterministic_exit": 1,
  "preempt_bootstrap_policy": 1,
  "preempt_mb_selftest": 0,
  "preempt_user_minimal_mode": "syscall-v2-runtime"
}
```

**Contract Change Rationale:**
- **Not a regression**: Intentional semantic change
- **Reason**: Deterministic early exit implementation
- **Impact**: SW/IRET count reduced from 39408 → 62 (expected)
- **Benefit**: Faster, deterministic validation runs (~11s vs ~30s)

---

## Pre-CI Discipline Status

**Local Gates (2026-03-01T15:21Z):**
```
✅ ABI Gate: PASS (no ABI-affecting changes)
✅ Boundary Gate: PASS (symbol-scan clean)
✅ Hygiene Gate: PASS (working tree clean)
✅ Constitutional Gate: PASS (compliance verified)
```

**Run IDs:**
- ABI: `20260301T152015Z-030ed1d2-13724`
- Boundary: `20260301T152017Z-030ed1d2-14016`
- Hygiene: `20260301T152111Z-030ed1d2-41800`
- Constitutional: `20260301T152116Z-030ed1d2-41917`

---

## Authoritative Baseline Regeneration Protocol

### Current Environment Mismatch

**Local (Development):**
```
host_os: Darwin
host_arch: arm64
ci_image_digest: unknown
env_hash: 7f333677cde632595fe639476f44e0ad47ca11d6b55d1b372010956dd661b4fd
```

**Authority (Required):**
```
baseline_authority: github-hosted-ubuntu-24.04-x64
ci_image_digest: gha-ubuntu24-20260224.36.1-X64
env_hash: b198f0cd6195143696ede006a7141a995e4fbe05ed8177f495ed0d7f503c5aa8
```

**Constitutional Requirement:**
- ❌ Local baseline init: PROHIBITED (env mismatch)
- ✅ CI baseline init: REQUIRED (authoritative)

### Required Action

**GitHub Actions Workflow: `perf-baseline-init`**

1. Navigate to: `https://github.com/kenanay/AykenOS/actions/workflows/perf-baseline-init.yml`
2. Click: "Run workflow"
3. Select branch: `pr/main-updates-20260301`
4. Input: `ci_image_digest: gha-ubuntu24-20260224.36.1-X64`
5. Execute workflow (expected: exit code 2, baseline written)
6. Download artifact: `perf-baseline.lock.json`
7. **CRITICAL: Validate CI determinism matches local (see Risk Assessment)**
8. Commit with message (ONLY if validation passes):

```
perf: regenerate baseline after deterministic-preempt contract update [authorized]

Contract evolution:
- Old: preempt_sw_count=39408 (30s timeout run)
- New: mark_sw_count=62 (deterministic early exit)
- Reason: measurement_contract="deterministic_preempt_harness"

Determinism verified:
- 3 consecutive runs: SW=62, IRET=62, Exit=1, Proof=1
- QEMU jitter: ~67ms (acceptable, kernel-external)
- Cadence: 100% deterministic

Authority: github-hosted-ubuntu-24.04-x64
CI Image: gha-ubuntu24-20260224.36.1-X64
Evidence: evidence/run-20260301T1514*/
```

---

## Engineering Assessment

### System Maturity: Phase 10 Level

**Achieved:**
- ✅ Deterministic runtime (100% reproducible)
- ✅ Deterministic exit path (no timeout dependency)
- ✅ Deterministic proof marker (canonical source)
- ✅ Timeout chain closed (no busy-loop timing)
- ✅ Contract explicit (measurement_contract defined)
- ✅ Self-echo eliminated (no marker duplication)
- ✅ Local discipline clean (4/4 gates PASS)
- ✅ Authority enforcement (CI-only baseline)

**System State:**
```
Runtime: Deterministic ✅
Exit: Deterministic ✅
Proof: Deterministic ✅
Timeout: Closed ✅
Contract: Explicit ✅
Discipline: Clean ✅
Authority: Enforced ✅
```

### Architectural Significance

This is no longer "does it work?" level. This is "is it authoritatively locked?" level.

The system has transitioned from:
- **Experimental** → **Measurable**
- **Variable** → **Deterministic**
- **Timeout-dependent** → **Exit-driven**
- **Implicit** → **Contract-explicit**

**Phase 10 Maturity: ACHIEVED**

---

## CI Baseline Validation Checklist

After `perf-baseline-init` workflow completes, verify the generated `perf-baseline.lock.json`:

### ✅ Required Values (MUST match local evidence)

```bash
# Extract from CI baseline artifact
jq -r '.raw_metrics | {
  preempt_sw_count,
  preempt_iret_count,
  preempt_qemu_exit_rc,
  preempt_timeout_hit,
  proof_done_seen
}' perf-baseline.lock.json

# Expected output:
{
  "preempt_sw_count": 62,
  "preempt_iret_count": 62,
  "preempt_qemu_exit_rc": 1,
  "preempt_timeout_hit": 0,
  "proof_done_seen": 1
}
```

### ✅ Required Contract (MUST be explicit)

```bash
jq -r '.env.marker_contract.measurement_contract' perf-baseline.lock.json
# Expected: "deterministic_preempt_harness"

jq -r '.env.marker_contract.preempt_deterministic_exit' perf-baseline.lock.json
# Expected: 1
```

### ✅ Authority Metadata (MUST be CI environment)

```bash
jq -r '.env | {
  baseline_authority,
  ci_image_digest,
  host_os,
  host_arch
}' perf-baseline.lock.json

# Expected:
{
  "baseline_authority": "github-hosted-ubuntu-24.04-x64",
  "ci_image_digest": "gha-ubuntu24-20260224.36.1-X64",
  "host_os": "Linux",
  "host_arch": "x86_64"
}
```

### ❌ Failure Conditions (DO NOT COMMIT)

- `preempt_sw_count != 62` → Architecture-dependent behavior
- `measurement_contract != "deterministic_preempt_harness"` → Contract mismatch
- `preempt_deterministic_exit != 1` → Exit path not deterministic
- `ci_image_digest == "unknown"` → Not authoritative environment
- `host_os != "Linux"` → Wrong platform

**If ANY failure condition is met: STOP. Investigate. Do not proceed.**

---

## Commit Protocol (Post-Validation Only)

**ONLY commit if ALL validation checks pass.**

### Single Commit (Baseline Only)

```bash
# After validation passes
git add scripts/ci/perf-baseline.lock.json
git commit -m "perf: regenerate baseline after deterministic-preempt contract update [authorized]

Contract evolution:
- Old: preempt_sw_count=39408 (30s timeout run)
- New: preempt_sw_count=62 (deterministic early exit)
- Reason: measurement_contract=\"deterministic_preempt_harness\"

Determinism verified (CI):
- preempt_sw_count: 62 (matches local)
- preempt_iret_count: 62 (matches local)
- qemu_exit_rc: 1 (deterministic)
- timeout_hit: 0 (no timeout)
- proof_done: 1 (deterministic)

Authority: github-hosted-ubuntu-24.04-x64
CI Image: gha-ubuntu24-20260224.36.1-X64
Workflow: perf-baseline-init
Run ID: [INSERT_RUN_ID]
Evidence: [INSERT_EVIDENCE_PATH]"
```

### Verification After Commit

```bash
# Verify performance gate passes with new baseline
make ci-gate-performance

# Expected: PASS (no violations)
```

---

## Critical Risk Assessment

### CI vs Local Determinism Alignment

**Risk:** Local determinism (macOS arm64) may differ from CI determinism (Ubuntu x64)

**Validation Required:**
After CI baseline regeneration, verify:
```
CI observed values MUST match local:
- preempt_sw_count = 62 (not 39408, not other)
- preempt_iret_count = 62
- measurement_contract = deterministic_preempt_harness
- preempt_deterministic_exit = 1
- qemu_exit_rc = 1
- qemu_timeout_hit = 0
- proof_done_seen = 1
```

**If CI differs:**
- Local determinism ≠ CI determinism
- Architecture-dependent behavior detected
- Baseline regeneration MUST be investigated
- Do NOT proceed with freeze until aligned

**If CI matches:**
- Determinism is architecture-independent ✅
- Baseline lock is valid ✅
- Proceed with freeze ✅

---

## Next Steps

1. **Trigger authoritative baseline regeneration** (GitHub Actions)
2. **Validate CI determinism** (compare with local evidence)
3. **Commit new baseline** (single commit, explicit message, ONLY if validated)
4. **Verify CI freeze** (all gates PASS)
5. **Tag release** (freeze milestone)

**No rollback after validation. Lock it down. 🔐**

---

**Maintained by:** AykenOS Architecture Board  
**Last Updated:** 2026-03-01T15:23Z  
**Git SHA:** 030ed1d2132132af4f03f51428090f41e68cee40  
**Branch:** pr/main-updates-20260301
