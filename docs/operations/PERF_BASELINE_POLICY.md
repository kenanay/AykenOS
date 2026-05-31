# Performance Baseline Governance Policy

**Effective date:** 2026-05-25
**Current authority:** `github-hosted-ubuntu-24.04-x64`
**Current phase:** Phase-17 officially closed; Phase-18 transition not activated
**Authority boundary:** A baseline renewal is an artifact-import procedure
recorded by Kenan AY and followed by constitutional remote checks. It does
not by itself grant merge authority or establish Phase-17 closure.
**Duzenleyen / Gelistiren / Olusturan / Mimari Sorumlu:** Kenan AY
**Attribution boundary:** Documentation metadata only; not runtime, evidence,
baseline, merge, or closure authority.

## Purpose

This document defines the strict governance rules for performance baseline management in AykenOS. The baseline is a constitutional lock that ensures system performance never regresses unintentionally.

## Core Principle

**The performance baseline is immutable except under explicitly approved conditions.**

Any attempt to modify the baseline without meeting these conditions will be rejected.

## Terminology

`ci_image_digest` is a runner image fingerprint in AykenOS governance, not an OCI/container digest.

- Source fields: `ImageOS`, `ImageVersion`, `RUNNER_ARCH`
- Format: `gha-<ImageOS>-<ImageVersion>-<RUNNER_ARCH>`
- Used together with `env_hash` for strict environment matching

## Baseline Renewal Conditions

The performance baseline (`scripts/ci/perf-baseline.lock.json`) may ONLY be renewed when ONE of the following conditions is met:

### 1. CI Environment Change (Mandatory Renewal)

**Trigger:** GitHub Actions runner image update

**Examples:**
- `ImageVersion: 20260201.15.1` → `20260310.2`
- `ubuntu-24.04` → `ubuntu-26.04`
- Major toolchain version change (clang 18 → 19, qemu 8.2 → 9.0)

**Detection:**
- Performance gate fails with `env_hash_mismatch`
- `ci_image_digest` mismatch in gate report

**Action:**
1. Verify the environment change is legitimate (check GitHub Actions changelog)
2. Run baseline init workflow with new digest
3. Create PR with title: `ci(perf): renew baseline for runner image update`
4. Include old vs new env_hash in PR description
5. Merge after CI passes

**Authority:** Authorized workflow artifact, Kenan AY recorded maintainer
decision, and required remote constitutional PASS.

### 2. Intentional Performance Improvement (Conditional Renewal)

**Trigger:** Code change that improves performance metrics

**Examples:**
- Scheduler optimization reduces context switch latency
- Boot path optimization reduces boot time
- Syscall path optimization reduces syscall latency

**Detection:**
- Performance evidence shows actual metrics are lower than baseline across repeated runs
- Improvement is reproducible under the same baseline authority and `ci_image_digest`

**Action:**
1. Verify the improvement is real and reproducible
2. Document the optimization in commit message
3. Run baseline init workflow
4. Create PR with title: `perf: [optimization description] + baseline renewal`
5. Include before/after metrics in PR description
6. Record Kenan AY maintainer decision
7. Merge after CI passes

**Authority:** Core maintainer approval required

**Critical Rule:** The same PR MUST contain both:
- The performance improvement code
- The baseline renewal

Splitting these into separate PRs is PROHIBITED.

### 3. Toolchain Major Version Upgrade (Mandatory Renewal)

**Trigger:** Major version upgrade of build toolchain

**Examples:**
- clang 18.x → 19.x
- qemu 8.x → 9.x
- nasm 2.16 → 2.17

**Detection:**
- Performance gate fails with `env_hash_mismatch`
- Toolchain version mismatch in gate report

**Action:**
1. Verify the toolchain upgrade is intentional
2. Run baseline init workflow
3. Create PR with title: `ci(toolchain): upgrade [tool] to [version] + baseline renewal`
4. Include toolchain version change in PR description
5. Merge after CI passes

**Authority:** Authorized workflow artifact, Kenan AY recorded maintainer
decision, and required remote constitutional PASS.

## Prohibited Baseline Modifications

The following actions are STRICTLY PROHIBITED and will result in PR rejection:

### ❌ Baseline Inflation

**Definition:** Renewing baseline to hide performance regression

**Example:**
- Code change causes boot time to increase from 10719ms to 12000ms
- Developer renews baseline to accept the regression
- No legitimate reason (no env change, no toolchain change)

**Detection:**
- Baseline renewal PR without corresponding env/toolchain change
- Metrics in new baseline are WORSE than old baseline
- No performance improvement code in the same PR

**Action:** PR rejected, code must be fixed to restore performance

### ❌ Split Baseline Renewal

**Definition:** Separating performance code change from baseline renewal

**Example:**
- PR #1: Performance optimization code
- PR #2: Baseline renewal

**Detection:**
- Baseline renewal PR without code changes
- Performance improvement PR without baseline renewal

**Action:** Both PRs rejected, must be combined into single PR

### ❌ Manual Baseline Editing

**Definition:** Hand-editing baseline lock file without running init workflow

**Example:**
- Developer manually changes `env_hash` in lock file
- Developer manually adjusts metrics in lock file

**Detection:**
- Baseline lock file modified without corresponding workflow run
- `git_sha` in lock file doesn't match workflow run commit

**Action:** PR rejected, baseline must be generated by init workflow

### ❌ Threshold Manipulation

**Definition:** Increasing threshold percentages to hide regression

**Example:**
- Changing `boot_time_ms` threshold from 10% to 20%
- Changing `syscall_latency_ms_proxy` threshold from 5% to 10%

**Detection:**
- `thresholds_percent` values changed in lock file
- No corresponding policy document update

**Action:** PR rejected, thresholds are constitutional constants

### Performance Learning Review

Split-metric learning is explicitly non-authoritative.

- Use `make ci-gate-performance-learning-review PERF_LEARNING_SOURCE_GLOB='<glob>'`
- Input reports must already be clean `performance` gate `PASS` runs from a single authority surface
- Output is evidence-only:
  `history.json`, `summary.json`, `recommendations.json`
- If `sample_count < 5`, recommendations must stay `enforcement=none` and `status=insufficient_samples`
- The learning review must not mutate `scripts/ci/perf-baseline.lock.json`
- The learning review must not auto-waive regressions or auto-commit new thresholds

## Baseline Renewal Workflow

### Step 1: Verify Renewal Condition

Before initiating baseline renewal, verify ONE of the approved conditions is met:
- CI environment changed (check GitHub Actions changelog)
- Performance improvement implemented (check code diff)
- Toolchain upgraded (check toolchain versions)

If no condition is met, DO NOT proceed.

### Step 2: Run Baseline Init Workflow

1. Go to: https://github.com/kenanay/AykenOS/actions/workflows/perf-baseline-init.yml
2. Click "Run workflow"
3. Select branch: `main` (or feature branch for testing)
4. Enter `ci_image_digest`:
   - Find current digest from recent CI run logs
   - Format: `gha-ubuntu24-YYYYMMDD.X.Y-X64`
   - Example: `gha-ubuntu24-20260201.15.1-X64`
5. Click "Run workflow"

### Step 3: Download Generated Baseline

1. Wait for workflow to complete (exit code 2 expected)
2. Download artifact: `perf-baseline-evidence`
3. Extract `scripts/ci/perf-baseline.lock.json`
4. Verify critical fields:
   - `env_hash`: 64-char hex string
   - `ci_image_digest`: matches input
   - `preempt_sw_count`: >0
   - `preempt_iret_count`: >0
5. Confirm the workflow generated an artifact only; it must not push a
   baseline lock directly to `main` or another protected branch.

### Step 4: Create PR

1. Commit baseline lock file
2. Create PR with appropriate title (see conditions above)
3. Apply the governed `baseline-update` label so CI can evaluate the imported
   workflow artifact while ordinary PR lock mutations remain rejected.
4. Include in PR description:
   - Renewal condition (env change / perf improvement / toolchain upgrade)
   - Old vs new env_hash
   - Old vs new metrics (if applicable)
   - Justification for renewal
5. Record Kenan AY's maintainer decision and the artifact/workflow run that
   generated the imported lock. CODEOWNERS is accountability metadata under
   the accepted single-maintainer authority model.

### Step 5: Merge After CI Passes

1. Wait for constitutional CI to pass (all required gates green)
2. Verify performance gate uses new baseline
3. Merge only after required remote CI and the recorded maintainer decision
   are satisfied
4. Monitor next few PRs for false positives

## Baseline Lock Immutability

**CRITICAL RULE:** Baseline lock files cannot be modified in an ordinary pull
request. A renewal pull request may import an artifact generated by the
authorized init workflow, with the required review and mutation authorization.

### Protected Files

The following files are baseline locks and are immutable in PRs:
- `scripts/ci/perf-baseline.lock.json` (performance baseline) - **enforced**
- `scripts/ci/abi-baseline.lock.json` (ABI baseline) - **future enforcement**

### Enforcement

The performance gate enforces baseline lock immutability for `perf-baseline.lock.json`:
- On PR events (`pull_request`, `pull_request_target`), the gate checks if the performance baseline lock was modified
- If mutation detected → gate fails with `baseline lock immutability violation`
- A reviewed renewal PR must carry the governed baseline-update
  authorization before CI permits the generated lock mutation

**Note:** ABI baseline lock enforcement will be added in a future phase.

### Authorized Mutation Paths

Only the following workflows are authorized to generate baseline lock artifacts:
- `perf-baseline-init.yml` (performance baseline renewal)
- Future: `abi-baseline-init.yml` (ABI baseline renewal)

`perf-baseline-init.yml` validates and uploads the generated lock. It must not
commit or push the lock to a protected branch. Repository mutation happens
only through a reviewed renewal PR importing that artifact.

### Rationale

Baseline lock immutability prevents:
- Accidental baseline drift in feature PRs
- Baseline inflation to hide regressions
- Manual baseline editing without proper workflow
- Split baseline renewal (code change in one PR, baseline in another)

The only valid way to update a baseline is:
1. Trigger the authorized workflow (`perf-baseline-init`)
2. Download the generated baseline artifact
3. Commit the baseline in the same PR as the triggering condition (env change, perf improvement, toolchain upgrade)

## Enforcement Mechanisms

### 1. CI Gate Enforcement

The performance gate enforces baseline policy automatically:
- `PERF_ENV_MISMATCH_POLICY=fail`: Strict env_hash matching
- `PERF_REGRESSION_POLICY=fail`: Strict metric regression detection
- `PERF_BASELINE_MODE=constitutional`: Baseline-locked mode
- Baseline lock immutability check on PR events

Any violation results in CI failure.

### 2. Maintainer Decision Enforcement

Kenan AY must verify and record:
- Baseline renewal meets approved conditions
- Baseline was generated by init workflow (not hand-edited)
- Performance improvement code is in same PR (if applicable)
- Justification is documented in PR description

Issue #145 was resolved through the single-maintainer authority decision and
matching live protection configuration. This checklist still does not
establish Phase-17 closure.

### 3. Audit Trail

All baseline renewals are tracked:
- `git_sha` in lock file links to commit
- `created_at_utc` timestamp
- CI workflow run ID in artifact
- PR discussion and approval

## Baseline Stability Monitoring

### False Positive Detection

During stabilization period (first 2 weeks), monitor for:
- Legitimate code changes triggering false regressions
- Threshold calibration issues
- Non-deterministic metric variance

If false positives occur:
1. Document the issue in GitHub issue
2. Analyze root cause (code or threshold)
3. Adjust thresholds if justified (requires policy update)
4. Re-run baseline init if needed

### Threshold Calibration

Current thresholds:
- `boot_time_ms`: 10% (±1071ms)
- `context_switch_latency_ms_proxy`: 5% (±0.038ms)
- `syscall_latency_ms_proxy`: 5% (±0.038ms)

These thresholds are based on empirical variance observed during Phase 4.5 validation.

If calibration is needed:
1. Collect evidence from multiple CI runs
2. Calculate actual variance (mean, stddev, p95)
3. Propose new threshold in GitHub issue
4. Update this policy document
5. Update baseline lock file
6. Merge as policy change PR

## Runner Image Drift Response Plan

### Scenario: GitHub Updates Runner Image

**Detection:**
- Performance gate fails with `env_hash_mismatch`
- `ci_image_digest` shows new version

**Response:**
1. Check GitHub Actions changelog for runner update announcement
2. Verify new digest: `gha-ubuntu24-YYYYMMDD.X.Y-X64`
3. Run baseline init workflow with new digest
4. Create PR: `ci(perf): renew baseline for runner image update`
5. Include in PR description:
   - Old digest: `gha-ubuntu24-20260201.15.1-X64`
   - New digest: `gha-ubuntu24-YYYYMMDD.X.Y-X64`
   - Old env_hash: `777ca464...`
   - New env_hash: `[new hash]`
   - Link to GitHub Actions changelog
6. Merge only after CI passes and the maintainer decision is recorded

**Timeline:** Baseline renewal PR must be opened within 1 business day of first detection (`env_hash_mismatch` or `ci_image_digest` mismatch), target <= 24 hours.

**Authority:** Authorized workflow artifact, Kenan AY recorded maintainer
decision, and constitutional remote PASS; runner drift does not waive
baseline governance.

## Intentional Regression Validation Rule

Intentional regression tests are allowed only to verify that constitutional gates fail correctly.

- Delay injection must be deterministic (timer/tick based), not CPU busy-loop based
- Regression injection must be compile-time gated (`AYKEN_INTENTIONAL_PERF_REGRESSION_MS`) and default OFF
- Intentional regression commits must not be merged to `main`; use dedicated validation PRs and revert after evidence capture

## Tooling Isolation Future Work

### Phase 3: Container-Based CI

To reduce runner image drift risk:
- Use Docker container with pinned toolchain
- Pin exact versions: clang, lld, nasm, qemu
- Generate env_hash from container digest
- Reduce dependency on GitHub runner image

### Phase 4: Reproducible Builds

To achieve full determinism:
- Hermetic build environment
- Pinned toolchain binaries (checksums)
- Deterministic QEMU execution
- Eliminate non-deterministic variance

## References

- [Baseline Renewal Procedure](BASELINE_RENEWAL_PROCEDURE.md)
- [Constitutional CI Mode](CONSTITUTIONAL_CI_MODE.md)
- [Provisional CI Mode](PROVISIONAL_CI_MODE.md)
- [Performance Gate Implementation](../development/PERFORMANCE_GATE.md)
- [Single-Maintainer Authority Decision](../architecture-board/decisions/20260525-single-maintainer-authority-model.md)

## Revision History

| Date | Version | Changes | Author |
|------|---------|---------|--------|
| 2026-02-21 | 1.0 | Initial policy document | Constitutional Mode Team |
| 2026-05-25 | 1.1 | Synchronize baseline renewal, review and closure authority boundaries | Kenan AY |

---

**This is a constitutional document. Changes require Kenan AY's recorded
maintainer decision and evidence-based justification.**
