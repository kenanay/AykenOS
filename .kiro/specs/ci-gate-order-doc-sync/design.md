# CI Gate Order Documentation Synchronization - Design

**Feature:** ci-gate-order-doc-sync  
**Status:** Active  
**Design Version:** 1.0

## Design Overview

This design addresses the documentation synchronization required after reordering CI gates in the Makefile. The solution involves updating two key documentation files to accurately reflect the new execution order and establishing a protocol for future changes.

## Architecture

### Current State

**Makefile `ci-freeze` target (after change):**
```makefile
ci-freeze: ci-freeze-guard preflight-mode-guard \
  ci-gate-abi \
  ci-gate-boundary \
  ci-gate-ring0-exports \
  ci-gate-hygiene \
  ci-gate-tooling-isolation \
  ci-gate-constitutional \
  ci-gate-governance-policy \
  ci-gate-drift-activation \
  ci-gate-structural-abi \
  ci-gate-runtime-marker-contract \
  ci-gate-user-bin-lock \
  ci-gate-embedded-elf-hash \
  ci-gate-performance \                    # MOVED EARLIER
  ci-gate-ring3-execution-phase10a2 \
  ci-gate-syscall-semantics-phase10b \
  $(PHASE10C_FREEZE_GATE) \
  ci-gate-workspace \
  ci-gate-syscall-v2-runtime \
  ci-gate-sched-bridge-runtime \
  ci-gate-behavioral-suite \
  ci-gate-policy-accept
```

**Key Change:** `ci-gate-performance` moved from last position to before `ci-gate-ring3-execution-phase10a2`.

### Design Decisions

#### DD-1: Gate Order Significance
**Decision:** Document that gate execution order is intentional and matters for CI efficiency.

**Rationale:**
- Early gates catch common issues quickly (fail-fast principle)
- Expensive gates (performance, runtime) run after quick checks
- Performance gate moved earlier to catch regressions before expensive runtime tests

**Implications:**
- Future gate reordering requires documentation update
- Order changes should be intentional, not arbitrary

#### DD-2: Documentation Structure
**Decision:** Update both tech.md (developer reference) and freeze-enforcement-workflow.md (operational guide).

**Rationale:**
- tech.md is the primary technical reference for developers
- freeze-enforcement-workflow.md is the operational authority for CI process
- Both must be consistent to avoid confusion

#### DD-3: Rationale Documentation
**Decision:** Document the rationale for performance gate repositioning.

**Rationale:**
- Performance regressions should be caught before expensive runtime tests
- Moving performance gate earlier saves CI time on regression failures
- Establishes precedent for future optimization decisions

## Component Design

### Component 1: tech.md Update

**Location:** `.kiro/steering/tech.md`

**Changes Required:**

1. **Add execution order note** in "Mandatory Gates (Fail-Closed)" section:
```markdown
**Mandatory Gates (Fail-Closed):**
```bash
# Individual gates (execution order is intentional for ci-freeze)
make ci-gate-abi            # ABI stability check (MUST pass)
make ci-gate-boundary       # Ring0/Ring3 boundary enforcement (MUST pass)
make ci-gate-ring0-exports  # Ring0 export surface check (MUST pass)
make ci-gate-hygiene        # Repository cleanliness (MUST pass)
make ci-gate-tooling-isolation  # Tooling isolation check (MUST pass)
make ci-gate-constitutional # Constitutional compliance (MUST pass)
make ci-gate-governance-policy  # Governance policy enforcement (MUST pass)
make ci-gate-drift-activation   # Drift blocking activation requirement (MUST pass)
make ci-gate-structural-abi     # Structural ABI check (MUST pass)
make ci-gate-runtime-marker-contract  # Runtime marker contract (MUST pass)
make ci-gate-user-bin-lock      # User binary lock check (MUST pass)
make ci-gate-embedded-elf-hash  # Embedded ELF hash check (MUST pass)
make ci-gate-performance    # Performance regression check (MUST pass)
make ci-gate-ring3-execution-phase10a2  # Ring3 execution validation (MUST pass)
make ci-gate-syscall-semantics-phase10b  # Syscall semantics validation (MUST pass)
make ci-gate-workspace      # Workspace integrity (MUST pass)
make ci-gate-syscall-v2-runtime  # Syscall runtime validation (MUST pass)
make ci-gate-sched-bridge-runtime  # Scheduler bridge runtime validation (MUST pass)
make ci-gate-behavioral-suite  # Behavioral test suite (MUST pass)
make ci-gate-policy-accept  # Policy accept proof (MUST pass)

# Full CI suite
make ci                     # Standard CI (enforced gates)
make ci-freeze              # Strict freeze suite (all gates, fail-closed)
                            # Note: Gate execution order in ci-freeze is intentional
                            # - Quick checks first (ABI, boundary, hygiene)
                            # - Performance gate before expensive runtime tests
                            # - Runtime validation gates last
make ci-freeze-local        # Local freeze (skip perf/tooling)
```
```

2. **Add note about gate order** in "Gate Failure Policy" section:
```markdown
**Gate Failure Policy:**
- Any gate failure → **PR BLOCKED**
- Evidence MUST be reviewed
- Manual intervention required
- No auto-fix allowed
- Gate execution order is intentional (fail-fast optimization)
```

### Component 2: freeze-enforcement-workflow.md Update

**Location:** `docs/roadmap/freeze-enforcement-workflow.md`

**Changes Required:**

1. **Update Section 2.1** to match Makefile order:
```markdown
### 2.1 Mandatory Gate Targets

The following gates execute in order during `make ci-freeze`. Execution order is intentional:
- Quick validation gates first (fail-fast principle)
- Performance gate before expensive runtime tests
- Runtime validation gates last

**Execution Order:**

1. `make ci-gate-abi` - ABI stability check
2. `make ci-gate-boundary` - Ring0/Ring3 boundary enforcement
3. `make ci-gate-ring0-exports` - Ring0 export surface check
4. `make ci-gate-hygiene` - Repository cleanliness
5. `make ci-gate-tooling-isolation` - Tooling isolation check
6. `make ci-gate-constitutional` - Constitutional compliance
7. `make ci-gate-governance-policy` - Governance policy enforcement
8. `make ci-gate-drift-activation` - Drift blocking activation
9. `make ci-gate-structural-abi` - Structural ABI check
10. `make ci-gate-runtime-marker-contract` - Runtime marker contract
11. `make ci-gate-user-bin-lock` - User binary lock check
12. `make ci-gate-embedded-elf-hash` - Embedded ELF hash check
13. `make ci-gate-performance` - Performance regression check (moved earlier for fail-fast)
14. `make ci-gate-ring3-execution-phase10a2` - Ring3 execution validation
15. `make ci-gate-syscall-semantics-phase10b` - Syscall semantics validation
16. `make ci-gate-workspace` - Workspace integrity
17. `make ci-gate-syscall-v2-runtime` - Syscall runtime validation
18. `make ci-gate-sched-bridge-runtime` - Scheduler bridge runtime validation
19. `make ci-gate-behavioral-suite` - Behavioral test suite
20. `make ci-gate-policy-accept` - Policy accept proof
21. `make ci-summarize` - Generate summary report

**Rationale for Order:**
- Gates 1-12: Quick validation checks (< 30s each) catch common issues early
- Gate 13: Performance check moved earlier to catch regressions before expensive tests
- Gates 14-20: Runtime validation gates (expensive, run after quick checks pass)
- Gate 21: Summary generation (always runs last)
```

2. **Add note in Section 2.3** about order changes:
```markdown
### 2.3 CI Entry Point Contract

1. `make ci` = mevcut minimum zorunlu zincir (`ci-gate-boundary` + `ci-gate-hygiene` + `validate-full`)
2. `make ci-freeze` = strict freeze suite (tüm implemented gate'ler, execution order intentional)
3. `summary.json` verdict `PASS` değilse ilgili make hedefi fail eder.
4. CI orchestration workflow: `.github/workflows/ci-freeze.yml` (GitHub-hosted `ubuntu-latest` + fail-closed baseline policy).
5. Runner hardening/runbook: `docs/operations/SELF_HOSTED_RUNNER_HARDENING.md`.
6. Tooling isolation guard: perf/preempt tooling PR'larında `kernel/**` dokunuşu fail-closed (`make ci-gate-tooling-isolation`).
7. **Gate execution order changes require documentation update** (Constitutional Rule 7).
```

## Implementation Strategy

### Phase 1: Documentation Updates
1. Update `.kiro/steering/tech.md` with new gate order and rationale
2. Update `docs/roadmap/freeze-enforcement-workflow.md` with synchronized order
3. Commit both changes together

### Phase 2: Validation
1. Verify documentation matches Makefile exactly
2. Verify rationale is clear and documented
3. Verify constitutional compliance restored

## Testing Strategy

### Manual Verification
1. Compare documented gate order with Makefile line-by-line
2. Verify all gates are listed in both documents
3. Verify order is identical in both documents
4. Verify rationale is documented

### Constitutional Compliance Check
1. Verify Rule 7 (Documentation Synchronization) is satisfied
2. Verify documentation references Makefile target explicitly
3. Verify change rationale is documented

## Rollout Plan

1. Create documentation updates
2. Review for accuracy
3. Commit with reference to Makefile change
4. Verify constitutional compliance restored

## Risks and Mitigations

### Risk 1: Future Order Changes
**Risk:** Future Makefile changes may desync documentation again.

**Mitigation:**
- Document protocol for gate order changes
- Add note in Makefile near ci-freeze target
- Consider adding CI check for doc/Makefile sync (future enhancement)

### Risk 2: Incomplete Rationale
**Risk:** Rationale for performance gate move may be unclear.

**Mitigation:**
- Document fail-fast principle clearly
- Explain cost/benefit of early performance check
- Reference CI efficiency optimization

## Success Metrics

1. Documentation matches Makefile exactly (100% accuracy)
2. Rationale is documented and clear
3. Constitutional Rule 7 compliance restored
4. Future maintainers understand gate order is intentional

## Open Questions

1. **Q:** Why was performance gate moved earlier specifically?
   **A:** To catch performance regressions before expensive runtime tests run, saving CI time on failures.

2. **Q:** Are there dependencies between gates that dictate order?
   **A:** No hard dependencies, but logical ordering (quick checks first, expensive checks last) improves CI efficiency.

3. **Q:** Should we add automated doc/Makefile sync checking?
   **A:** Out of scope for this change, but worth considering as future enhancement.
