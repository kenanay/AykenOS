# Observability Boundary Disclosure

**Maintainer**: Kenan AY — System Architect
**Status**: Constitutional Requirement
**Version**: 1.0.0

---

## Purpose

This document discloses the **observability boundary** between validation and evidence in the Development Loop & Boot Monitoring System. It defines what can and cannot be observed, and what can and cannot influence validation decisions.

---

## Constitutional Authority

### R26: Direct Observation Source Constraint

**Requirement**: Validation decisions SHALL use only raw boot logs as input.

**Enforcement**: This boundary ensures validation remains authoritative and deterministic.

---

### R27: Evidence State Isolation

**Requirement**: Evidence artifacts SHALL remain stateless and non-influential to validation.

**Enforcement**: This boundary prevents evidence from becoming authority.

---

## Observability Model

### Validation Layer (Authoritative)

**Input**: Raw boot logs only (`out/logs/boot_watch.log`)

**Process**:
1. Build kernel with validation profile
2. Launch QEMU with timeout
3. Capture serial output to log
4. Parse log for required markers
5. Validate marker presence and sequence
6. Produce PASS/FAIL decision

**Output**: Exit status (0=PASS, 1=FAIL)

**Authority**: FULL - validation decisions are authoritative

---

### Evidence Layer (Non-Authoritative)

**Input**: Raw boot logs (read-only, after validation)

**Process**:
1. Wait for validation to complete
2. Read raw boot logs
3. Parse markers and metadata
4. Generate structured reports
5. Update run history
6. Persist evidence artifacts

**Output**: Evidence artifacts (`out/evidence/`)

**Authority**: ZERO - evidence is purely diagnostic

---

## Boundary Rules

### Allowed Flows

```
✅ Raw Logs → Validation
✅ Validation → Exit Status
✅ Raw Logs → Evidence Generation
✅ Evidence → Dashboard
✅ Evidence → Diff Engine
✅ Evidence → Historical Analysis
```

**Rationale**: Unidirectional flow preserves authority model.

---

### Forbidden Flows

```
❌ Evidence → Validation
❌ Dashboard → Validation
❌ History → Validation
❌ Evidence → Exit Status
❌ Evidence → Kernel Execution
❌ Dashboard → Kernel Execution
```

**Rationale**: Evidence must never influence validation or execution.

---

## Observation Sources

### Authoritative Source

**Source**: `out/logs/boot_watch.log`

**Properties**:
- Raw serial output from QEMU
- Append-only during boot
- Unmodified kernel output
- Direct observation of execution

**Usage**: Validation decisions ONLY

**Authority**: FULL - source of truth

---

### Derived Sources

**Sources**: `out/evidence/{run-id}/reports/*.json`

**Properties**:
- Generated AFTER validation
- Parsed from raw logs
- Structured for visualization
- Read-only for diagnostics

**Usage**: Dashboard, diff, analysis

**Authority**: ZERO - derived data only

---

## Evidence Artifacts

### Format Version

All evidence artifacts include `format_version` field for schema evolution.

**Current Version**: 1.0

---

### Metadata (`meta/run.json`)

**Purpose**: Run identification and context

**Fields**:
- `run_id`: Unique run identifier
- `time_utc`: Timestamp (ISO 8601)
- `source`: Execution source (dev_loop/ci/manual)
- `git_sha`: Git commit hash
- `git_branch`: Git branch name
- `git_dirty`: Working tree state
- `deterministic`: Determinism flag
- `developer`: Developer attribution
- `generated_by`: Generator attribution

**Authority**: ZERO - metadata only

---

### Summary (`reports/summary.json`)

**Purpose**: Boot status summary

**Fields**:
- `format_version`: Schema version
- `status.boot`: Boot status (PASS/FAIL/UNKNOWN)
- `status.reason`: Failure reason (if applicable)
- `status.markers_ok`: Marker validation result
- `status.fail_closed`: Fail-closed marker presence
- `validation.source`: Validation input source
- `validation.authoritative`: Authority flag (always false)
- `validation.diagnostic_only`: Diagnostic flag (always true)
- `isolation.non_influential`: Isolation guarantee
- `isolation.read_only`: Read-only guarantee
- `isolation.post_validation`: Timing guarantee
- `generated_by`: Generator attribution

**Authority**: ZERO - summary only

---

### Markers (`reports/markers.json`)

**Purpose**: Marker presence tracking

**Fields**:
- `EARLY_BOOT_OK`: Early boot marker presence
- `LATE_INIT_END`: Late init marker presence
- `BOOT_OK`: Full boot marker presence
- `FAIL_CLOSED`: Fail-closed marker presence
- `generated_by`: Generator attribution

**Authority**: ZERO - tracking only

---

### Performance (`reports/perf.json`)

**Purpose**: Performance proxy metrics

**Fields**:
- `format_version`: Schema version
- `value`: Performance proxy value
- `unit`: Measurement unit
- `method`: Measurement method
- `valid`: Validity flag
- `diagnostic_only`: Diagnostic flag (always true)
- `non_authoritative`: Authority flag (always true)
- `disclaimer`: Non-authority disclaimer
- `generated_by`: Generator attribution

**Authority**: ZERO - diagnostic only

**Disclaimer**: Performance metrics are diagnostic only and do not affect validation decisions.

---

### Logs (`logs/boot.log`)

**Purpose**: Boot log preservation

**Content**: Copy of raw boot log from validation

**Authority**: ZERO - copy only (original is authoritative)

---

## Run History

### History File (`out/evidence/runs.json`)

**Purpose**: Track validation run history

**Fields**:
- `format_version`: Schema version
- `runs`: Array of run summaries (last 100)
- `generated_by`: Generator attribution

**Run Entry**:
- `run_id`: Unique run identifier
- `timestamp`: Run timestamp
- `git_sha`: Git commit hash
- `git_branch`: Git branch name
- `source`: Execution source
- `boot_status`: Boot status
- `markers_ok`: Marker validation result
- `perf_value`: Performance proxy value

**Authority**: ZERO - historical tracking only

**Usage**: Dashboard, trending, analysis

---

## Dashboard Observability

### Read-Only Guarantee

**Principle**: Dashboard has ZERO validation authority.

**Implementation**:
- Static HTML/CSS/JS only
- No backend server
- No writes to kernel
- No writes to logs
- No runtime coupling

**Enforcement**: Static files, no execution authority

---

### Data Flow

```
Raw Logs (authoritative)
    ↓
Validation (authoritative)
    ↓
Evidence Generation (non-authoritative)
    ↓
Evidence Artifacts (non-authoritative)
    ↓
Dashboard Visualization (non-authoritative)
```

**Critical**: Dashboard is at the END of the pipeline.

---

### Capabilities

**Allowed**:
- ✅ Read evidence artifacts
- ✅ Display status
- ✅ Show markers
- ✅ Visualize performance
- ✅ Display logs
- ✅ Show run history
- ✅ Compare runs (diff)

**Forbidden**:
- ❌ Write to logs
- ❌ Write to kernel
- ❌ Affect validation
- ❌ Affect execution
- ❌ Modify evidence
- ❌ Influence decisions

---

## Diff Engine Observability

### Purpose

Compare two validation runs for diagnostic analysis.

**Authority**: ZERO - purely observational

---

### Capabilities

**Allowed**:
- ✅ Read evidence artifacts
- ✅ Compare metadata
- ✅ Compare boot status
- ✅ Compare markers
- ✅ Compare performance
- ✅ Diff logs

**Forbidden**:
- ❌ Affect validation
- ❌ Modify evidence
- ❌ Influence decisions

---

### Output

Diagnostic comparison report showing:
- Metadata differences
- Boot status changes
- Marker presence changes
- Performance deltas
- Log differences

**Authority**: ZERO - diagnostic only

---

## Evidence Misuse Guard

### Purpose

Detect patterns where evidence artifacts are used as validation input.

**Authority**: Governance enforcement

---

### Checks

1. **Validation Isolation**: Validation scripts must not read evidence artifacts
2. **Conditional Logic**: Evidence must not be used in conditional logic
3. **Exit Status**: Evidence must not affect exit status
4. **Dashboard Isolation**: Dashboard must not write to validation inputs
5. **Generation Timing**: Evidence generation must happen after validation

---

### Enforcement

**Script**: `scripts/check_evidence_misuse.sh`

**CI Integration**: Runs in parallel with other governance checks

**Failure**: Blocks merge if violations detected

---

## Violation Examples

### ❌ Evidence as Validation Input

```bash
# FORBIDDEN: Reading evidence for validation decision
if grep -q '"boot": "PASS"' out/evidence/latest/reports/summary.json; then
    exit 0
fi
```

**Why Forbidden**: Evidence becomes authority, violates R26.

**Correct Approach**: Read raw logs only.

---

### ❌ Dashboard Affecting Validation

```javascript
// FORBIDDEN: Dashboard writing to logs
fetch('out/logs/boot_watch.log', { method: 'POST', body: data });
```

**Why Forbidden**: Dashboard becomes control plane, violates R27.

**Correct Approach**: Dashboard is read-only.

---

### ❌ Evidence in Conditional Logic

```bash
# FORBIDDEN: Evidence affecting control flow
PERF=$(jq -r '.value' out/evidence/latest/reports/perf.json)
if [[ $PERF -gt 1000 ]]; then
    exit 1
fi
```

**Why Forbidden**: Evidence influences validation, violates R26.

**Correct Approach**: Performance is diagnostic only.

---

## Correct Patterns

### ✅ Validation from Raw Logs

```bash
# CORRECT: Validation reads raw logs only
if grep -q '\[\[AYKEN_BOOT_OK\]\]' out/logs/boot_watch.log; then
    echo "PASS"
    exit 0
else
    echo "FAIL"
    exit 1
fi
```

**Why Correct**: Uses authoritative source (raw logs).

---

### ✅ Evidence After Validation

```bash
# CORRECT: Evidence generated after validation
./scripts/dev_loop.sh smoke
VALIDATION_EXIT=$?

if [[ $VALIDATION_EXIT -eq 0 ]]; then
    ./scripts/generate_evidence.sh
fi
```

**Why Correct**: Evidence generated after validation completes.

---

### ✅ Dashboard Read-Only

```javascript
// CORRECT: Dashboard reads evidence only
fetch('out/evidence/latest/reports/summary.json')
    .then(response => response.json())
    .then(data => displayStatus(data));
```

**Why Correct**: Dashboard is read-only observer.

---

## Enforcement Mechanisms

### Static Analysis

**Script**: `scripts/check_evidence_misuse.sh`

**Checks**:
- Validation scripts do not read evidence
- Evidence not used in conditional logic
- Evidence does not affect exit status
- Dashboard is read-only
- Evidence generated after validation

**Frequency**: Every CI run

---

### CI Integration

**Workflow**: `.github/workflows/governance.yml`

**Checks**:
- Evidence isolation
- Observation boundary
- Naming compliance
- Spec purity

**Failure**: Blocks merge

---

### Code Review

**Requirement**: All PRs reviewed for boundary violations

**Checklist**:
- [ ] Validation uses raw logs only
- [ ] Evidence generated after validation
- [ ] Dashboard is read-only
- [ ] No evidence in conditional logic
- [ ] No evidence affecting exit status

---

## Boundary Disclosure

### In Code

All evidence-related scripts include header:

```bash
# Purpose: [Description]
# Authority: ZERO - [explanation]
#
# Maintainer: Kenan AY — System Architect
```

---

### In Dashboard

Dashboard includes disclaimer:

```
This dashboard is a read-only observer with ZERO validation authority.
All validation decisions use only raw boot logs as input.
Evidence artifacts displayed here are generated AFTER validation completes
and cannot influence validation outcomes or kernel execution.
```

---

### In Evidence

All evidence artifacts include:

```json
{
  "validation": {
    "source": "raw_boot_logs",
    "authoritative": false,
    "diagnostic_only": true
  },
  "isolation": {
    "non_influential": true,
    "read_only": true,
    "post_validation": true
  }
}
```

---

## Summary

### Observability Boundary

```
┌─────────────────────────────────────────┐
│         Authoritative Layer             │
│                                         │
│  Raw Logs → Validation → Exit Status   │
│                                         │
└─────────────────────────────────────────┘
              ↓ (read-only)
┌─────────────────────────────────────────┐
│       Non-Authoritative Layer           │
│                                         │
│  Evidence → Dashboard → Diff → History  │
│                                         │
└─────────────────────────────────────────┘
```

---

### Key Principles

1. **Validation uses raw logs only** (R26)
2. **Evidence is non-authoritative** (R27)
3. **Evidence generated after validation**
4. **Dashboard is read-only observer**
5. **Evidence cannot affect validation**
6. **Evidence cannot affect execution**

---

### Enforcement

- Static analysis: `check_evidence_misuse.sh`
- CI checks: Parallel governance validation
- Code review: Boundary violation checklist
- Documentation: This disclosure document

---

## References

- **Requirements**: `.kiro/specs/dev-loop-boot-monitoring/requirements.md`
- **Design**: `.kiro/specs/dev-loop-boot-monitoring/design.md`
- **Tasks**: `.kiro/specs/dev-loop-boot-monitoring/tasks.md`
- **Constitution**: `docs/dev-loop/DEV_LOOP_CONSTITUTION.md`
- **Governance**: `docs/dev-loop/GOVERNANCE.md`

---

**Last Updated**: 2026-05-08
**Maintainer**: Kenan AY — System Architect
