---
# Drift Blocking Activation Configuration
# Authority: ARCHITECTURE_FREEZE.md
# Phase Requirement: >= 9 (Phase is >= 9)

# Activation state (explicit only)
enabled: true

# Minimum phase for enforcement
phase_minimum: 9

# Auto-activation policy (phase_guard = CI enforces, but no auto-enable)
auto_activation_policy: phase_guard

# N-run persistence threshold
n_run_threshold: 3
---

# Drift Blocking Activation Protocol

This document controls drift blocking activation for AykenOS CI.

## Current State

- **Enabled:** `true` (drift blocking active)
- **Phase Minimum:** `9` (enforcement starts at Phase 9)
- **Policy:** `phase_guard` (CI enforces requirement, no auto-enable)
- **N-Run Threshold:** `3` (regression must persist for 3 consecutive runs)

## Activation Protocol

**No auto-activation:** Drift blocking never enables automatically.

1. System reaches Phase 9 maturity
2. CI gate `ci-gate-drift-activation` starts enforcing
3. Developer explicitly sets `enabled: true`
4. Commit change with justification
5. CI gate passes, drift blocking active

## N-Run Persistence

Drift blocking uses N-run persistence to avoid false positives:

- Regression must appear in **3 consecutive runs** to block
- Single-run regression → warning only
- Counter state stored in **CI artifact** (not repository)
- Counter resets on authority hash change

## Runtime State (CI Artifact Only)

Drift counters and authority hash are **NOT stored in this file**.

Runtime state is managed by CI artifact store:
- **Artifact key:** `drift-state-${authority_hash}`
- **Storage:** GitHub Actions cache/artifact
- **Scope:** Authority-scoped (toolchain + QEMU + optional salt)
- **Lifetime:** Persists across CI runs with same authority
- **Reset:** Automatic on authority hash change

**Why not in repository?**
- Constitution documents are **policy**, not **state**
- Runtime state in repo → merge conflicts, governance noise
- CI artifact → clean separation, branch isolation

## Authority Hash

Authority hash computed from:
- Toolchain version (`clang --version`, first line)
- Runtime version (`qemu-system-x86_64 --version`, first line)
- Optional salt (`PERF_AUTHORITY_SALT`)

CI sets:
- `PERF_AUTHORITY_SALT=${{ github.repository }}`

This keeps counters stable across commits while isolating fork instances.

When authority hash changes:
- All drift counters reset to 0
- New baseline authority established
- Reset event logged in evidence

**Authority hash is stored in CI artifact, not this file.**

## Fork Behavior

When repository is forked:
- Fork has **different repository salt** (`owner/repo`) → different authority hash
- Drift state **does not transfer** to fork
- Fork starts with **fresh drift state** (N-run counter = 0)
- Fork is **independent governance instance**

This ensures:
- Fork independence
- No upstream coupling
- Fork establishes own baseline

## Allowlist Mechanism

Metrics can be allowlisted via `constitution/drift_blocking_allowlist.json`:

```json
{
  "version": "1.0",
  "metrics": [
    "boot_time_variance",
    "memory_allocation_jitter"
  ]
}
```

Allowlisted metrics:
- Still collected and logged
- Do not trigger CI failure
- Bypass logged in evidence
