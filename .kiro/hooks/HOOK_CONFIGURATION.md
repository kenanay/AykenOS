# AykenOS Hook Configuration

**Version:** 1.0  
**Date:** 2026-02-22  
**Status:** ACTIVE

## Hook Philosophy

AykenOS hooks are **pre-CI discipline layers**, not CI replacements. They enforce:

1. **Fail-Closed**: Stop on violation, never auto-fix architectural issues
2. **Path-Based**: Target specific file patterns, not broad wildcards
3. **Evidence-Based**: Generate reports, require manual intervention
4. **Constitutional**: Align with ARCHITECTURE_FREEZE.md rules

## Active Hooks

### 1. Documentation Sync Mandatory
**Event:** `fileEdited`  
**Patterns:** Critical architectural files (ayken_abi.h, syscall_v2.c, context_switch.asm, Makefile, ARCHITECTURE_FREEZE.md)  
**Action:** Enforce documentation updates for architectural changes  
**Enforcement:** Fail-closed, blocks until docs synchronized

### 2. Ring0 Build Guard
**Event:** `fileEdited`  
**Patterns:** kernel/**/*.{c,h,asm,S}, bootloader/**/*.{c,h,S}, linker.ld  
**Action:** Strict build validation with -Werror  
**Enforcement:** Fail-closed, stops on build failure

### 3. Rust Constitutional Check
**Event:** `fileEdited`  
**Patterns:** ayken/**/*.rs, ayken-core/**/*.rs, userspace/**/*.rs  
**Action:** Run cargo test + clippy, verify constitutional compliance  
**Enforcement:** Fail-closed, rejects locked module changes

### 4. CI Gate Simulation
**Event:** `agentStop`  
**Patterns:** N/A (runs after agent execution)  
**Action:** Simulate CI gates (ABI, boundary, hygiene, constitutional)  
**Enforcement:** Fail-closed, stops on first gate failure

### 5. ABI Drift Guard
**Event:** `fileEdited`  
**Patterns:** ayken_abi.h, context_switch.asm, syscall_v2.c  
**Action:** Detect ABI changes, enforce regeneration discipline  
**Enforcement:** Fail-closed, requires RFC for ABI changes

### 6. Ring3 Boundary Guard
**Event:** `fileEdited`  
**Patterns:** kernel/**/*.{c,h}, userspace/**/*.rs  
**Action:** Verify Ring0/Ring3 separation, detect policy leakage  
**Enforcement:** Fail-closed, requires RFC for boundary changes

## Hook Execution Flow

```
File Save → Hook Trigger → Validation → PASS/FAIL
                                           ↓
                                         FAIL → STOP
                                           ↓
                                    Report Violation
                                           ↓
                                  Require Manual Fix
```

## What Hooks Are NOT

- ❌ CI replacement (real CI gates are mandatory for merge)
- ❌ Branch protection (use GitHub branch rules)
- ❌ CODEOWNERS enforcement (use .github/CODEOWNERS)
- ❌ Auto-fix tools (violations require manual intervention)

## What Hooks ARE

- ✅ Pre-CI discipline layer
- ✅ Early violation detection
- ✅ Developer feedback loop
- ✅ Constitutional enforcement reminder

## Hook Maintenance

### Adding New Hooks
1. Identify specific file patterns (no broad wildcards)
2. Choose correct event type (fileEdited, agentStop)
3. Write fail-closed prompt (no advisory language)
4. Test with actual file changes
5. Document in this file

### Modifying Hooks
1. Preserve fail-closed semantics
2. Keep path patterns specific
3. Update version number
4. Document changes in git commit

### Disabling Hooks
Set `"enabled": false` in hook JSON. Document reason in commit message.

## Constitutional Alignment

All hooks align with ARCHITECTURE_FREEZE.md:

- **Section 3.1**: Syscall Contract Invariants → ABI Drift Guard
- **Section 3.2**: Ring0/Ring3 Boundary Invariants → Ring3 Boundary Guard
- **Section 4.1**: ABI Gate → ABI Drift Guard
- **Section 4.2**: Boundary Gate → Ring0 Build Guard + Ring3 Boundary Guard
- **Section 4.6**: Constitutional Gate → Rust Constitutional Check

## Next Steps

After hooks are stable:

1. Add CODEOWNERS for kernel/, bootloader/, ayken_abi.h
2. Configure branch protection (require CI gates)
3. Enable required checks in GitHub
4. Document in docs/operations/

## Evidence Location

Hook execution does not generate evidence. For evidence:
- Run `make ci-gate-*` commands manually
- Check `evidence/run-<RUN_ID>/` directories
- Review `reports/summary.json` for gate verdicts

---

**Maintained by:** AykenOS Core Team  
**Last Updated:** 2026-02-22
