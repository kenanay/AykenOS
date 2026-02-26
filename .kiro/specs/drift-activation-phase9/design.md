# Drift Activation Phase-9 Design

**Feature:** Drift Blocking Activation Protocol  
**Phase:** 9 (Governance Stabilization)  
**Status:** DRAFT

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│                    CI Gate: drift-activation                │
│                                                             │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────┐ │
│  │ Phase Reader │───▶│ State Reader │───▶│ Enforcement  │ │
│  └──────────────┘    └──────────────┘    │   Logic      │ │
│         │                    │            └──────┬───────┘ │
│         │                    │                   │         │
│         ▼                    ▼                   ▼         │
│  CURRENT_PHASE.md   drift_blocking_      PASS/FAIL/SKIP   │
│                     activation.md                          │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
                    ┌──────────────────┐
                    │ Evidence Output  │
                    │  - report.json   │
                    │  - meta.txt      │
                    │  - violations.txt│
                    └──────────────────┘
```

## Component Design

### 1. Phase Detection

**File:** `scripts/ci/lib-phase.sh`

```bash
#!/usr/bin/env bash

get_current_phase() {
    local phase_file="${ROOT}/docs/roadmap/CURRENT_PHASE"
    
    if [[ ! -f "${phase_file}" ]]; then
        echo "ERROR: Phase file not found: ${phase_file}" >&2
        return 3
    fi
    
    # Extract phase number from simple format
    # Expected format: "CURRENT_PHASE=8"
    local phase=$(grep -E "^CURRENT_PHASE=[0-9]+$" "${phase_file}" | \
                  cut -d'=' -f2)
    
    if [[ -z "${phase}" ]]; then
        echo "ERROR: Could not parse phase number from ${phase_file}" >&2
        return 3
    fi
    
    echo "${phase}"
}
```

### 2. Activation State Schema

**File:** `constitution/drift_blocking_activation.md`

```yaml
---
# Drift Blocking Activation Configuration
# Authority: ARCHITECTURE_FREEZE.md
# Phase Requirement: >= 9

# Activation state (explicit only)
enabled: false

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

- **Enabled:** `false` (drift blocking inactive)
- **Phase Minimum:** `9` (enforcement starts at Phase 9)
- **Policy:** `phase_guard` (CI enforces requirement, no auto-enable)
- **N-Run Threshold:** `3` (regression must persist for 3 consecutive runs)

## Activation Protocol

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
- **Scope:** Authority-scoped (git SHA + toolchain + QEMU version)
- **Lifetime:** Persists across CI runs with same authority
- **Reset:** Automatic on authority hash change

**Why not in repository?**
- Constitution documents are **policy**, not **state**
- Runtime state in repo → merge conflicts, governance noise
- CI artifact → clean separation, branch isolation

## Authority Hash

Authority hash computed from:
- Git commit SHA
- Toolchain version (clang, ld.lld)
- QEMU version

When authority hash changes:
- All drift counters reset to 0
- New baseline authority established
- Reset event logged in evidence

**Authority hash is stored in CI artifact, not this file.**

## Fork Behavior

When repository is forked:
- Fork has **different git SHA** → different authority hash
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
```

### 3. CI Gate Implementation

**File:** `scripts/ci/gate_drift_activation.sh`

**Responsibility:** Enforce activation requirement only (no drift detection logic)

```bash
#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CI_TOOLS="${ROOT}/tools/ci"

source "${CI_TOOLS}/lib.sh"
source "${CI_TOOLS}/lib-phase.sh"

usage() {
    cat <<'USAGE'
Usage: scripts/ci/gate_drift_activation.sh --evidence-dir <path>

Responsibility:
  Enforce drift blocking activation requirement (Phase >= 9).
  Does NOT perform drift detection or N-run persistence.
  Drift detection is handled by ci-gate-performance.

Exit codes:
  0: pass/skip
  2: drift activation violations
  3: usage/tooling error
USAGE
}

EVIDENCE_DIR=""

while [[ $# -gt 0 ]]; do
    case "$1" in
        --evidence-dir) EVIDENCE_DIR="$2"; shift 2 ;;
        -h|--help) usage; exit 0 ;;
        *) echo "Unknown arg: $1" >&2; usage; exit 3 ;;
    esac
done

if [[ -z "${EVIDENCE_DIR}" ]]; then
    usage
    exit 3
fi

mkdir -p "${EVIDENCE_DIR}"

PHASE_FILE="${ROOT}/docs/roadmap/CURRENT_PHASE"
ACTIVATION_FILE="${ROOT}/constitution/drift_blocking_activation.md"
REPORT_JSON="${EVIDENCE_DIR}/report.json"
META_TXT="${EVIDENCE_DIR}/meta.txt"
VIOLATIONS_TXT="${EVIDENCE_DIR}/violations.txt"

NOW="$(ci_now_utc)"
GIT_SHA="$(git -C "${ROOT}" rev-parse HEAD 2>/dev/null || echo NO_GIT)"

# Get current phase
if ! CURRENT_PHASE=$(get_current_phase); then
    echo "ERROR: Failed to detect current phase" >&2
    exit 3
fi

# Parse activation state
if [[ ! -f "${ACTIVATION_FILE}" ]]; then
    echo "ERROR: Activation file not found: ${ACTIVATION_FILE}" >&2
    exit 3
fi

ENABLED=$(grep -E "^enabled:\s*(true|false)" "${ACTIVATION_FILE}" | \
          grep -oE "(true|false)" || echo "false")
PHASE_MIN=$(grep -E "^phase_minimum:\s*[0-9]+" "${ACTIVATION_FILE}" | \
            grep -oE "[0-9]+" || echo "9")

# Enforcement logic (activation requirement only)
VERDICT="SKIP"
REASON=""
VIOLATIONS=()

if [[ "${CURRENT_PHASE}" -lt "${PHASE_MIN}" ]]; then
    VERDICT="SKIP"
    REASON="phase_below_minimum"
elif [[ "${ENABLED}" == "false" ]]; then
    VERDICT="FAIL"
    REASON="drift_blocking_required_but_disabled"
    VIOLATIONS+=("phase=${CURRENT_PHASE}:enabled=false:required=true")
else
    VERDICT="PASS"
    REASON="drift_blocking_enabled"
fi

# Write evidence
cat > "${META_TXT}" <<META
time_utc=${NOW}
git_sha=${GIT_SHA}
current_phase=${CURRENT_PHASE}
phase_minimum=${PHASE_MIN}
enabled=${ENABLED}
verdict=${VERDICT}
reason=${REASON}
META

printf "%s\n" "${VIOLATIONS[@]}" > "${VIOLATIONS_TXT}"

python3 - <<'PY' "${REPORT_JSON}" "${VERDICT}" "${REASON}" "${CURRENT_PHASE}" "${PHASE_MIN}" "${ENABLED}" "${VIOLATIONS[@]}"
import json
import sys

path, verdict, reason, phase, phase_min, enabled = sys.argv[1:7]
violations = sys.argv[7:] if len(sys.argv) > 7 else []

report = {
    "gate": "drift-activation",
    "verdict": verdict,
    "reason": reason,
    "current_phase": int(phase),
    "phase_minimum": int(phase_min),
    "enabled": enabled == "true",
    "violations": violations,
    "note": "This gate enforces activation requirement only. Drift detection is handled by ci-gate-performance."
}

with open(path, "w", encoding="utf-8") as fh:
    json.dump(report, fh, indent=2, sort_keys=True)
    fh.write("\n")
PY

if [[ "${VERDICT}" == "FAIL" ]]; then
    echo "drift-activation: FAIL (${REASON})"
    exit 2
elif [[ "${VERDICT}" == "SKIP" ]]; then
    echo "drift-activation: SKIP (${REASON})"
    exit 0
else
    echo "drift-activation: PASS"
    exit 0
fi
```

### 4. Makefile Integration

```makefile
ci-gate-drift-activation: ci-evidence-dir
	@echo "== CI GATE DRIFT ACTIVATION =="
	@echo "run_id: $(RUN_ID)"
	@./scripts/ci/gate_drift_activation.sh --evidence-dir "$(EVIDENCE_RUN_DIR)/gates/drift-activation"
	@cp -f "$(EVIDENCE_RUN_DIR)/gates/drift-activation/report.json" "$(EVIDENCE_RUN_DIR)/reports/drift-activation.json"
	@$(MAKE) ci-summarize RUN_ID=$(RUN_ID) EVIDENCE_ROOT=$(EVIDENCE_ROOT)
	@echo "OK: drift-activation evidence at $(EVIDENCE_RUN_DIR)"

# Add to ci-freeze chain (after governance-policy, before structural-abi)
ci-freeze: ci-freeze-guard ci-gate-abi ci-gate-boundary ci-gate-ring0-exports \
           ci-gate-hygiene ci-gate-tooling-isolation ci-gate-constitutional \
           ci-gate-governance-policy ci-gate-drift-activation ci-gate-structural-abi \
           ci-gate-runtime-marker-contract ci-gate-workspace ci-gate-syscall-v2-runtime \
           ci-gate-sched-bridge-runtime ci-gate-behavioral-suite ci-gate-policy-accept \
           ci-gate-performance
	@echo "Freeze CI suite completed successfully!"
```

## Data Flow

### Phase < 9 (Current State)

```
make ci-gate-drift-activation
  ↓
Read CURRENT_PHASE.md → phase=8
  ↓
Read drift_blocking_activation.md → enabled=false, phase_minimum=9
  ↓
Enforcement: phase < phase_minimum
  ↓
Verdict: SKIP (reason: phase_below_minimum)
  ↓
Evidence: report.json (verdict=SKIP)
  ↓
Exit 0 (success)
```

### Phase 9, Drift Blocking Disabled

```
make ci-gate-drift-activation
  ↓
Read CURRENT_PHASE.md → phase=9
  ↓
Read drift_blocking_activation.md → enabled=false, phase_minimum=9
  ↓
Enforcement: phase >= phase_minimum AND enabled=false
  ↓
Verdict: FAIL (reason: drift_blocking_required_but_disabled)
  ↓
Evidence: report.json (verdict=FAIL, violations=[...])
  ↓
Exit 2 (failure)
```

### Phase 9, Drift Blocking Enabled

```
make ci-gate-drift-activation
  ↓
Read CURRENT_PHASE.md → phase=9
  ↓
Read drift_blocking_activation.md → enabled=true, phase_minimum=9
  ↓
Enforcement: phase >= phase_minimum AND enabled=true
  ↓
Verdict: PASS (reason: drift_blocking_enabled)
  ↓
Evidence: report.json (verdict=PASS)
  ↓
Exit 0 (success)
```

## N-Run Persistence Design

**Responsibility:** Performance gate (NOT drift activation gate)

**Runtime State Storage:** CI Artifact (not repository)

### Integration Point

N-run persistence logic is integrated into `ci-gate-performance`, not `ci-gate-drift-activation`.

**Why?**
- Drift activation gate → requirement enforcement only
- Performance gate → drift detection + N-run threshold + blocking
- Single responsibility principle

### Artifact Key Schema
```
drift-state-${authority_hash}
```

### Authority Hash Computation
```bash
compute_authority_hash() {
    local git_sha="$(git -C "${ROOT}" rev-parse HEAD)"
    local clang_ver="$(clang --version | head -1)"
    local qemu_ver="$(qemu-system-x86_64 --version | head -1)"
    
    echo -n "${git_sha}:${clang_ver}:${qemu_ver}" | sha256sum | cut -d' ' -f1
}
```

### Runtime State Schema
```json
{
  "authority_hash": "abc123...",
  "counters": {
    "boot_time_ms": 2,
    "context_switch_ns": 1
  },
  "last_updated": "2026-02-25T12:00:00Z"
}
```

### CI Workflow Integration

**GitHub Actions:**
```yaml
- name: Restore drift state
  uses: actions/cache/restore@v3
  with:
    key: drift-state-${{ env.AUTHORITY_HASH }}
    path: .ci-state/drift_state.json

- name: Run performance gate
  run: make ci-gate-performance

- name: Save drift state
  uses: actions/cache/save@v3
  with:
    key: drift-state-${{ env.AUTHORITY_HASH }}
    path: .ci-state/drift_state.json
```

### State Management Library

**File:** `scripts/ci/lib-drift-persistence.sh`

```bash
#!/usr/bin/env bash

DRIFT_STATE_FILE="${ROOT}/.ci-state/drift_state.json"

compute_authority_hash() {
    local git_sha="$(git -C "${ROOT}" rev-parse HEAD)"
    local clang_ver="$(clang --version | head -1)"
    local qemu_ver="$(qemu-system-x86_64 --version | head -1)"
    
    echo -n "${git_sha}:${clang_ver}:${qemu_ver}" | sha256sum | cut -d' ' -f1
}

load_drift_state() {
    if [[ ! -f "${DRIFT_STATE_FILE}" ]]; then
        echo '{"authority_hash":"","counters":{}}'
        return
    fi
    cat "${DRIFT_STATE_FILE}"
}

save_drift_state() {
    local state="$1"
    mkdir -p "$(dirname "${DRIFT_STATE_FILE}")"
    echo "${state}" > "${DRIFT_STATE_FILE}"
}

increment_drift_counter() {
    local metric="$1"
    local state="$(load_drift_state)"
    local current_hash="$(compute_authority_hash)"
    
    # Check authority hash
    local stored_hash="$(echo "${state}" | jq -r '.authority_hash // ""')"
    if [[ "${stored_hash}" != "${current_hash}" ]]; then
        # Authority changed, reset all counters
        state='{"authority_hash": "'${current_hash}'", "counters": {}}'
    fi
    
    # Increment counter
    state="$(echo "${state}" | jq --arg m "${metric}" \
        '.counters[$m] = (.counters[$m] // 0) + 1')"
    
    save_drift_state "${state}"
    
    # Return counter value
    echo "${state}" | jq -r --arg m "${metric}" '.counters[$m]'
}

check_drift_threshold() {
    local metric="$1"
    local threshold="${2:-3}"
    local count="$(increment_drift_counter "${metric}")"
    
    if [[ "${count}" -ge "${threshold}" ]]; then
        return 0  # Threshold exceeded
    else
        return 1  # Below threshold
    fi
}
```

### Fork Behavior

**Authority hash ensures fork independence:**

```bash
# Upstream repo
git_sha = "abc123..."  # upstream commit
authority_hash = sha256("abc123...:clang:qemu")
artifact_key = "drift-state-abc123..."

# Fork repo (different git SHA)
git_sha = "def456..."  # fork commit (different)
authority_hash = sha256("def456...:clang:qemu")  # DIFFERENT
artifact_key = "drift-state-def456..."  # DIFFERENT

# Result: Fork has no drift state (fresh start)
```

**Fork drift state:**
- Fork CI runs with different git SHA
- Authority hash is different
- CI artifact key is different
- No upstream drift state found
- Fork starts with empty counters
- Fork is independent governance instance

### State File Location

**Local development:**
```
.ci-state/drift_state.json  # gitignored
```

**CI environment:**
```
.ci-state/drift_state.json  # restored from artifact
```

**Repository:**
```
constitution/drift_blocking_activation.md  # policy only (no state)
```

### Gitignore Entry

```gitignore
# CI runtime state (not committed)
.ci-state/
```

## Allowlist Design

**File:** `constitution/drift_blocking_allowlist.json`

```json
{
  "version": "1.0",
  "metrics": []
}
```

Allowlist check in performance gate:

```bash
is_metric_allowlisted() {
    local metric="$1"
    local allowlist="${ROOT}/constitution/drift_blocking_allowlist.json"
    
    if [[ ! -f "${allowlist}" ]]; then
        return 1  # Not allowlisted
    fi
    
    if jq -e --arg m "${metric}" '.metrics | index($m)' "${allowlist}" >/dev/null; then
        return 0  # Allowlisted
    else
        return 1  # Not allowlisted
    fi
}
```

## Error Handling

### Missing Phase File
- Gate exits with code 3 (tooling error)
- Evidence includes error message
- CI fails with clear diagnostic

### Missing Activation File
- Gate exits with code 3 (tooling error)
- Evidence includes error message
- CI fails with clear diagnostic

### Invalid Phase Number
- Gate exits with code 3 (tooling error)
- Evidence includes parse error
- CI fails with clear diagnostic

### Invalid Activation State
- Default to `enabled: false`
- Log warning in evidence
- Continue with enforcement logic

## Testing Strategy

### Unit Tests
- Phase detection logic
- Activation state parsing
- Enforcement logic (all branches)
- Authority hash computation
- N-run counter increment/reset

### Integration Tests
- Full gate execution (Phase < 9)
- Full gate execution (Phase 9, disabled)
- Full gate execution (Phase 9, enabled)
- Evidence generation
- Makefile integration

### Property Tests
- Phase number always >= 0
- Activation state always boolean
- Authority hash always 64 hex chars
- Counter values always >= 0
- Allowlist always valid JSON array

## Security Considerations

- No auto-enable (explicit activation only)
- All state changes require git commit
- Evidence immutability enforced by hygiene gate
- Authority hash prevents baseline tampering
- Allowlist changes audited in git history

## Performance Considerations

- Gate execution: < 5 seconds (no QEMU)
- Phase detection: single file read
- Activation state: single file parse
- Evidence generation: < 1 second

## Rollout Plan

1. **Phase 8 (Current):** Implement gate, always SKIP
2. **Phase 8.5:** Test gate in CI (verify SKIP behavior)
3. **Phase 9 Transition:** Update CURRENT_PHASE.md
4. **Phase 9:** Gate starts enforcing (FAIL if disabled)
5. **Phase 9 Activation:** Developer enables drift blocking
6. **Phase 9+:** Gate enforces (PASS if enabled)

## Correctness Properties

### Property 1: Phase-Driven Enforcement
**Validates: Requirements 1.2, 1.3, 1.4**

For all runs:
- If phase < 9, then verdict = SKIP
- If phase >= 9 AND enabled = false, then verdict = FAIL
- If phase >= 9 AND enabled = true, then verdict = PASS

### Property 2: Explicit Activation
**Validates: Requirements 2.1, 2.2, 2.3, 2.4**

For all activation state changes:
- enabled field must be explicitly set (no default to true)
- Activation requires git commit
- No auto-enable logic exists

### Property 3: Evidence Immutability
**Validates: Requirements 4.1, 4.2, 4.3**

For all evidence files:
- Evidence never modified after creation
- Evidence includes timestamp, git SHA, phase, state
- Evidence format is valid JSON

### Property 4: N-Run Persistence
**Validates: Requirements 5.1, 5.2, 5.3, 5.4**

For all drift counters:
- Counter increments on each regression detection
- Counter resets on authority hash change
- Threshold = 3 consecutive runs
- Single-run regression does not block

### Property 5: Authority Hash Reset
**Validates: Requirements 6.1, 6.2, 6.3, 6.4**

For all authority hash changes:
- All counters reset to 0
- New authority hash stored
- Reset event logged in evidence

## References

- `ARCHITECTURE_FREEZE.md`: Determinism requirement
- `constitution/drift_blocking_activation.md`: Activation state
- `docs/governance/CONSTITUTION_BOUNDARY.md`: Governance boundary
- `scripts/ci/gate_performance.sh`: Drift metric source
