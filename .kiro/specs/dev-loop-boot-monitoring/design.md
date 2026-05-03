# Design Document: Development Loop & Boot Monitoring System

**Implementation Guide**: For detailed implementation instructions, see `docs/dev-loop/IMPLEMENTATION_GUIDE.md`

---

## 1. Purpose

The Development Loop & Boot Monitoring System provides **deterministic external validation** of kernel boot execution.

### Core Function

The dev loop:
- **Observes**: Captures kernel boot output
- **Validates**: Checks for required markers
- **Reports**: Produces PASS/FAIL decision

### Critical Constraint

The dev loop **NEVER** affects kernel execution behavior.

---

## 2. Architectural Principles

### 2.1 Non-Interference

**Principle**: Dev loop is read-only relative to runtime.

**Guarantees**:
- No kernel state modification
- No kernel memory writes
- No execution flow changes
- Validation markers are pure output

**Enforcement**: Isolation property test verifies this guarantee.

---

### 2.2 Observation Source Constraint

**Principle**: Validation uses only raw boot logs as input.

**Allowed**:
- ✅ `out/logs/boot_watch.log`

**Forbidden**:
- ❌ `out/evidence/*` (derived data)
- ❌ Historical runs
- ❌ Dashboard state

**Rationale**: Prevents evidence from becoming authority.

---

### 2.3 Evidence ≠ Authority

**Principle**: Evidence is derived data, never decision input.

**Evidence Properties**:
- Generated AFTER validation
- Read-only for visualization
- Never affects validation outcome
- Never affects execution flow

**Rationale**: Prevents tool-driven runtime.

---

### 2.4 Determinism

**Principle**: Same input → same output.

**Guarantees**:
- No global state mutations
- Reproducible builds
- Deterministic marker emission
- Consistent test execution order

**Rationale**: Enables reliable regression detection.

---

## 3. System Layers

### Layer 1: Kernel

**Responsibility**: Emit boot markers during initialization.

**Behavior**:
- Validation build: Markers enabled
- Production build: Markers compiled out
- Markers are pure output (no side effects)

**Isolation**: Markers do not affect runtime behavior.

---

### Layer 2: Dev Loop

**Responsibility**: Orchestrate validation pipeline.

**Behavior**:
- Build kernel with validation profile
- Launch QEMU with timeout
- Capture serial output to log
- Validate marker presence and sequence
- Produce PASS/FAIL decision

**Isolation**: Userspace script, no kernel coupling.

---

### Layer 3: Evidence Pipeline

**Responsibility**: Transform logs into structured reports.

**Behavior**:
- Parse raw logs
- Generate structured JSON
- Create performance metrics
- Update run history

**Isolation**: Runs AFTER validation, never affects it.

---

### Layer 4: Visualization

**Responsibility**: Display validation results.

**Behavior**:
- Live status monitoring
- Run comparison (diff)
- Performance trending
- Historical analysis

**Isolation**: Read-only, no validation authority.

---

## 4. Validation Model

### 4.1 Marker-Based Validation

**Concept**: Boot success determined by marker presence and sequence.

**Required Markers**:
1. `[K][EARLY_BOOT_OK]` - Early boot complete
2. `[K][LATE_INIT_END]` - Late init complete
3. `[[AYKEN_BOOT_OK]]` - Full boot complete

**Sequence**: EARLY → LATE → BOOT_OK (strictly ordered)

**Rationale**: Provides fine-grained boot phase visibility.

---

### 4.2 Validation Levels

**Smoke** (5-10s):
- Build + boot validation
- Marker presence check
- Fast feedback for iteration

**Contract** (~1-2 min):
- Smoke + runtime contract tests
- VCP behavior validation
- Capability system checks

**Full** (~2-3 min):
- Contract + evidence tests
- Verification layer validation
- Comprehensive coverage

**Rationale**: Time/coverage trade-off for different development tasks.

---

### 4.3 Exit Contract

**PASS**: Exit status 0
- Build succeeds
- All required markers present
- Markers in correct sequence
- All tests pass (if applicable)

**FAIL**: Exit status 1
- Build fails
- Boot timeout
- Missing marker
- Marker sequence violation
- Test failure

**Rationale**: Deterministic, scriptable validation outcome.

---

## 5. Isolation Model

### 5.1 Strict Boundary

```
Validation → Logs (✅ allowed)
Validation → Evidence (❌ forbidden)
```

**Enforcement**:
- Static analysis: `check_observation_boundary.sh`
- CI check: Fails on boundary violation

---

### 5.2 Forbidden Flow

```
Evidence → Validation (❌)
Dashboard → Validation (❌)
History → Validation (❌)
```

**Rationale**: Prevents derived data from becoming authority.

---

### 5.3 Allowed Flow

```
Logs → Validation (✅)
Validation → Evidence (✅)
Evidence → Dashboard (✅)
```

**Rationale**: Unidirectional data flow preserves authority model.

---

## 6. Evidence Model

### 6.1 Derived Nature

**Principle**: Evidence is generated AFTER validation completes.

**Process**:
1. Validation produces PASS/FAIL
2. Evidence generator reads logs
3. Structured reports created
4. Dashboard updated

**Guarantee**: Evidence cannot affect validation outcome.

---

### 6.2 Non-Authority

**Principle**: Evidence is observational, not decisional.

**Properties**:
- Read-only for visualization
- Never used as validation input
- Never affects execution flow
- Purely diagnostic

**Enforcement**: `check_evidence_isolation.sh` verifies this.

---

### 6.3 Observability

**Purpose**: Provide structured diagnostic output.

**Artifacts**:
- `summary.json` - Boot status
- `markers.json` - Marker presence
- `perf.json` - Performance proxy
- `meta.json` - Run metadata

**Usage**: Dashboard visualization, diff analysis, trending.

---

## 7. Dashboard Model

### 7.1 Read-Only Visualization

**Principle**: Dashboard displays results, never affects them.

**Features**:
- Live status monitoring
- Run comparison
- Performance trending
- Historical analysis

**Guarantee**: Dashboard has zero validation authority.

---

### 7.2 No Decision Authority

**Principle**: Dashboard cannot influence validation or execution.

**Enforcement**:
- Static HTML/JS (no backend)
- No writes to kernel
- No runtime coupling
- Pure visualization

**Rationale**: Prevents UI from becoming control plane.

---

## 8. Performance Model

### 8.1 Diagnostic Separation

**Principle**: Performance metrics are diagnostic, not authoritative.

**Measurement**:
- Marker-based proxy (quick)
- TSC-based accurate (full)

**Usage**: Regression detection, trending, analysis.

**Guarantee**: Performance does not affect validation decisions.

---

### 8.2 Non-Blocking

**Principle**: Performance measurement does not block validation.

**Behavior**:
- Performance check runs separately
- Failure is warning, not error
- Baseline missing = skip, not fail

**Rationale**: Performance is diagnostic, not critical.

---

## 9. Governance Model

### 9.1 Enforcement Mechanisms

**Evidence Isolation**:
- Script: `check_evidence_isolation.sh`
- Verifies: Evidence not used as validation input

**Observation Boundary**:
- Script: `check_observation_boundary.sh`
- Verifies: Validation uses only raw logs

**Naming Compliance**:
- Script: `check_naming_compliance.sh`
- Verifies: Naming conventions followed

**Spec Purity**:
- Script: `check_spec_purity.sh`
- Verifies: Spec contains no implementation details

---

### 9.2 CI Integration

**Workflow**: All governance checks run in parallel.

**Failure**: Any check failure blocks merge.

**Rationale**: Automated enforcement prevents drift.

---

## 10. Anti-Patterns

### ❌ Evidence as Validation Input

**Problem**: Using `out/evidence/` for validation decisions.

**Impact**: Evidence becomes authority, violates isolation.

**Prevention**: Static analysis detects this pattern.

---

### ❌ Dashboard as Control Plane

**Problem**: Dashboard affecting validation or execution.

**Impact**: UI becomes decision maker, violates authority model.

**Prevention**: Dashboard is static HTML/JS, no backend.

---

### ❌ Dev Loop Affecting Kernel

**Problem**: Dev loop modifying kernel behavior.

**Impact**: Validation results don't reflect production.

**Prevention**: Isolation property test verifies this.

---

### ❌ Spec Containing Implementation

**Problem**: Code snippets, commands, schemas in spec.

**Impact**: Spec becomes tutorial, loses authority.

**Prevention**: Spec purity check enforces this.

---

## 11. Design Rationale

### Why Userspace Scripts?

**Decision**: Dev loop implemented in `scripts/` directory.

**Rationale**:
- Strict isolation from kernel
- Independent evolution
- No accidental coupling
- Easy to test and verify

---

### Why Marker-Based Validation?

**Decision**: Use deterministic string markers, not just exit codes.

**Rationale**:
- Fine-grained boot phase visibility
- Sequence validation possible
- Fail-fast detection enabled
- Clear diagnostic information

---

### Why 3 Validation Levels?

**Decision**: Smoke, contract, full.

**Rationale**:
- Smoke: Fast iteration (5-10s)
- Contract: Runtime validation (~1-2 min)
- Full: Comprehensive coverage (~2-3 min)
- Time/coverage trade-off for different tasks

---

### Why Evidence Pipeline?

**Decision**: Separate evidence generation from validation.

**Rationale**:
- Preserves isolation
- Prevents authority drift
- Enables rich observability
- Maintains determinism

---

## 12. Constitutional Compliance

### DETERMINISM.GLOBAL

**Requirement**: No global state mutations.

**Compliance**:
- Validation logic is stateless
- Reproducible builds
- Deterministic marker emission

---

### KERNEL.RING0.POLICY

**Requirement**: No policy decisions in Ring0.

**Compliance**:
- Validation markers are pure output
- No policy logic in kernel
- Dev loop is userspace

---

### SECURITY.BOUNDARY.VIOLATION

**Requirement**: No Ring3 accessing Ring0 directly.

**Compliance**:
- Dev loop is userspace script
- Markers emitted to serial (Ring0 → Ring3)
- No direct memory access

---

## 13. Future Enhancements

**Note**: For implementation details, see `docs/dev-loop/IMPLEMENTATION_GUIDE.md`

1. **Automated Regression Finder**: Git bisect integration
2. **Parallel Test Execution**: Reduce total validation time
3. **Test Result Caching**: Skip unchanged tests
4. **Incremental Validation**: Run only affected tests
5. **CI Integration**: GitHub Actions workflow
6. **Performance Regression Detection**: Baseline comparison

---

## References

- **Requirements**: `requirements.md`
- **Tasks**: `tasks.md`
- **Constitution**: `DEV_LOOP_CONSTITUTION.md`
- **Governance**: `GOVERNANCE.md`
- **Implementation Guide**: `docs/dev-loop/IMPLEMENTATION_GUIDE.md`
- **CI Integration**: `docs/dev-loop/CI_INTEGRATION.md`
- **Performance**: `docs/dev-loop/PERFORMANCE_INTEGRATION.md`

---

**Last Updated**: 2026-05-03  
**Maintainer**: Kenan AY — System Architect
