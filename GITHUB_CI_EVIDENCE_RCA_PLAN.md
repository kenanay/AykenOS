# GitHub CI Evidence-Based RCA Plan

## Critical Correction

**Previous hypothesis REJECTED:** 61/61 mailbox fallback is NOT the root cause.

**Evidence:** Bisect showed baseline commit (050332220d9a) already has 61/61 pattern.

**New approach:** Use ONLY GitHub CI evidence as authority, ignore local macOS results.

## 5-Step RCA Plan

### Step 1: Download GitHub CI Evidence (Authoritative)

**Objective:** Get actual.lock.json from the failing GitHub Actions run

**Actions:**
```bash
# From GitHub Actions run: https://github.com/kenanay/AykenOS/actions/runs/24535837417
# Download artifacts or extract from logs:
# - actual.lock.json (performance measurements)
# - report.json (gate violations)
# - env.json (environment details)
# - preempt.log (execution trace)

# Store in evidence/github-ci-failure/
mkdir -p evidence/github-ci-failure
# Download files to this directory
```

**Expected data:**
- `boot_time_ms`: 12197 (or 12720 from latest report)
- `context_switch_latency_ms_proxy`: 201.93
- `syscall_latency_ms_proxy`: 201.93
- `mailbox_phase_breakdown_ticks`: full diagnostics
- `env_hash`: GitHub CI environment signature

**Authority:** ONLY this data is authoritative for RCA.

---

### Step 2: Compare GitHub CI Evidence vs Baseline

**Objective:** Identify what changed between baseline and current in SAME environment

**Actions:**
```bash
# Baseline (050332220d9a) - from scripts/ci/perf-baseline.lock.json
BASELINE_BOOT=10684
BASELINE_CTX=175.081967
BASELINE_ENV="edbe5bed0e83075ace80b5d9aa09588613afceedd8304e30891a20aae2dc4a67"

# Current (9b3358e6) - from GitHub CI actual.lock.json
CURRENT_BOOT=12197  # or 12720
CURRENT_CTX=201.93
CURRENT_ENV="<extract from GitHub CI>"

# Compare:
# 1. Are env_hash values identical? (environment drift check)
# 2. Are mailbox stats identical? (61/61 in both?)
# 3. Are phase_breakdown_ticks different? (where is time spent?)
# 4. Are raw_markers different? (timing of boot phases)
```

**Key questions:**
- If env_hash differs → environment changed (runner image, QEMU, compiler)
- If mailbox stats identical → regression is NOT in mailbox path
- If phase_breakdown differs → regression is in specific boot phase
- If raw_markers differ → regression is in specific subsystem

---

### Step 3: Analyze Phase Breakdown Differences

**Objective:** Find which boot phase regressed

**Actions:**
```bash
# Extract phase durations from both:
jq '.raw_metrics.phase_breakdown_ticks.durations' scripts/ci/perf-baseline.lock.json > baseline_phases.json
jq '.raw_metrics.phase_breakdown_ticks.durations' evidence/github-ci-failure/actual.lock.json > current_phases.json

# Compare:
diff -u baseline_phases.json current_phases.json

# Calculate deltas:
# - boot_start_to_core_ready
# - core_ready_to_first_sched_activity
# - first_sched_activity_to_first_user_entry
# - first_user_entry_to_first_syscall_gate_entry
```

**Expected outcome:**
- Identify which phase(s) account for +1513ms (or +2036ms) regression
- Example: if "boot_start_to_core_ready" increased by 1500ms → boot phase regression
- Example: if "first_user_entry_to_first_syscall_gate_entry" increased → userspace entry regression

---

### Step 4: Analyze Mailbox Phase Breakdown Differences

**Objective:** Even if 61/61 pattern is same, check if timing changed

**Actions:**
```bash
# Extract mailbox durations:
jq '.raw_metrics.mailbox_phase_breakdown_ticks.durations' scripts/ci/perf-baseline.lock.json > baseline_mailbox.json
jq '.raw_metrics.mailbox_phase_breakdown_ticks.durations' evidence/github-ci-failure/actual.lock.json > current_mailbox.json

# Compare:
diff -u baseline_mailbox.json current_mailbox.json

# Check path_durations:
jq '.raw_metrics.mailbox_phase_breakdown_ticks.path_durations' scripts/ci/perf-baseline.lock.json > baseline_paths.json
jq '.raw_metrics.mailbox_phase_breakdown_ticks.path_durations' evidence/github-ci-failure/actual.lock.json > current_paths.json

diff -u baseline_paths.json current_paths.json
```

**Key metrics:**
- `fallback.mean_ticks`: baseline vs current (even if count=61 in both)
- `switch.total_ticks`: baseline vs current
- `arbiter.ticks`: baseline vs current
- `extract.ticks`: baseline vs current

**Hypothesis:**
- If fallback count is same (61) but mean_ticks increased → fallback path got slower
- If arbiter.ticks increased → arbiter decision logic got slower
- If extract.ticks increased → mailbox extract got slower

---

### Step 5: Targeted Bisect Based on Evidence

**Objective:** Bisect on the specific subsystem that regressed

**Actions:**

**If Step 3 shows boot phase regression:**
```bash
# Bisect with boot phase timing threshold
git bisect start
git bisect bad 9b3358e6
git bisect good 050332220d9a
# Modify bisect script to check boot_start_to_core_ready duration
git bisect run scripts/ci/bisect_boot_phase.sh
```

**If Step 3 shows scheduler phase regression:**
```bash
# Bisect with scheduler timing threshold
git bisect start
git bisect bad 9b3358e6
git bisect good 050332220d9a
# Modify bisect script to check first_sched_activity_to_first_user_entry
git bisect run scripts/ci/bisect_sched_phase.sh
```

**If Step 4 shows mailbox timing regression (not count):**
```bash
# Bisect with mailbox mean_ticks threshold
git bisect start
git bisect bad 9b3358e6
git bisect good 050332220d9a
# Modify bisect script to check fallback.mean_ticks or arbiter.ticks
git bisect run scripts/ci/bisect_mailbox_timing.sh
```

**If Step 2 shows environment drift:**
```bash
# No bisect needed - this is environment change, not code regression
# Options:
# 1. Update baseline to new environment
# 2. Pin CI environment to baseline environment
# 3. Accept performance variation due to environment
```

---

## Decision Tree

```
GitHub CI Evidence
    ↓
env_hash same? ──NO──> Environment drift → Update baseline or pin environment
    ↓ YES
    ↓
mailbox stats same? ──YES──> Regression NOT in mailbox logic
    ↓                         ↓
    ↓                    Check phase_breakdown_ticks
    ↓                         ↓
    ↓                    Which phase regressed?
    ↓                         ↓
    ↓                    Bisect on that phase
    ↓
    NO (mailbox stats differ)
    ↓
Check mailbox timing (not just count)
    ↓
fallback.mean_ticks increased? ──YES──> Fallback path got slower
    ↓                                    ↓
    ↓                               Bisect on fallback timing
    ↓
arbiter.ticks increased? ──YES──> Arbiter logic got slower
    ↓                              ↓
    ↓                         Bisect on arbiter timing
    ↓
extract.ticks increased? ──YES──> Extract logic got slower
                                   ↓
                              Bisect on extract timing
```

---

## Critical Rules

1. **ONLY GitHub CI evidence is authoritative**
   - Ignore local macOS measurements
   - Ignore local Linux measurements
   - Only use: `github-hosted-ubuntu-24.04-x64` environment

2. **Compare apples to apples**
   - Same env_hash → code regression
   - Different env_hash → environment drift
   - Don't compare across environments

3. **Don't assume mailbox is the problem**
   - 61/61 pattern exists in baseline
   - Pattern might be normal
   - Look at timing, not just counts

4. **Bisect on specific metrics**
   - Don't bisect on overall boot_time_ms (too coarse)
   - Bisect on specific phase that regressed
   - Use targeted thresholds

5. **Accept environment drift if proven**
   - If env_hash differs and explains regression
   - Update baseline via authorized workflow
   - Don't fight environment changes

---

## Expected Outcomes

### Outcome A: Environment Drift
- `env_hash` differs between baseline and current
- QEMU version, compiler, or runner image changed
- Performance difference explained by environment
- **Action:** Update baseline or pin environment

### Outcome B: Boot Phase Regression
- `boot_start_to_core_ready` increased significantly
- Regression in early boot, not scheduler
- **Action:** Bisect on boot phase timing, fix boot code

### Outcome C: Scheduler Phase Regression
- `first_sched_activity_to_first_user_entry` increased
- Regression in scheduler initialization
- **Action:** Bisect on scheduler phase timing, fix scheduler init

### Outcome D: Mailbox Timing Regression
- Mailbox count same (61/61) but timing increased
- `fallback.mean_ticks` or `arbiter.ticks` increased
- **Action:** Bisect on mailbox timing, optimize hot path

### Outcome E: Userspace Entry Regression
- `first_user_entry_to_first_syscall_gate_entry` increased
- Regression in Ring3 entry or syscall gate
- **Action:** Bisect on userspace entry timing, fix entry path

---

## Next Immediate Action

**Download GitHub CI evidence:**

```bash
# Option 1: From GitHub Actions UI
# Go to: https://github.com/kenanay/AykenOS/actions/runs/24535837417
# Download artifacts (if available)

# Option 2: From GitHub Actions logs
# Extract actual.lock.json from logs
# Look for "== CI GATE PERFORMANCE ==" section

# Option 3: Re-run in GitHub CI
# Push current HEAD to trigger CI
# Download fresh evidence

# Store in:
mkdir -p evidence/github-ci-failure
# Place actual.lock.json, report.json, env.json here
```

Once evidence is downloaded, proceed with Step 2 comparison.

---

## Why This Approach is Correct

1. **Authority-based:** Only GitHub CI is authoritative
2. **Evidence-based:** Use actual measurements, not assumptions
3. **Targeted:** Bisect on specific subsystem, not overall metric
4. **Falsifiable:** Each step can prove/disprove hypothesis
5. **Actionable:** Each outcome has clear next action

---

## What We Learned from Bisect

1. ✅ 61/61 mailbox fallback is NOT a recent regression
2. ✅ Pattern exists in baseline (050332220d9a)
3. ✅ Pattern exists in all commits since baseline
4. ✅ Performance regression exists despite same pattern
5. ✅ Regression is NOT explained by fallback count alone

**Conclusion:** Look at timing, not counts. Look at phases, not just mailbox. Use GitHub CI evidence as single source of truth.

