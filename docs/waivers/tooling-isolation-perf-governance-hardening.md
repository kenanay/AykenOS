# Tooling Isolation Waiver: Performance Governance Hardening

**Waiver ID:** `tooling-isolation-perf-governance-hardening`  
**Date:** 2026-02-21  
**PR:** #11  
**Branch:** `docs/perf-baseline-governance`  
**Commits:** `965cabe8..f17d7269`

- Expiry: 2026-03-21
- Related Issue: https://github.com/kenanay/AykenOS/issues/11

## Rationale

This PR introduces performance baseline governance hardening, which includes:

1. **Deterministic intentional regression hook** - requires kernel changes for timer-based delay
2. **validation-strict -Werror cleanup** - removes warnings from kernel files

These kernel changes are **intentional and necessary** for constitutional performance enforcement.

## Affected Files

Kernel files modified for legitimate reasons:

- `kernel/kernel.c` - deterministic regression hook (timer-based)
- `kernel/arch/x86_64/pic.c` - strict warning cleanup
- `kernel/drivers/console/fb_console.c` - strict warning cleanup
- `kernel/drivers/ui/logo_animator.c` - strict warning cleanup
- `kernel/fs/devfs.c` - strict warning cleanup
- `kernel/fs/vfs.c` - strict warning cleanup
- `kernel/mm/paging.c` - strict warning cleanup
- `kernel/proc/proc.c` - strict warning cleanup
- `kernel/sched/sched.c` - strict warning cleanup
- `kernel/sys/capability_manager.c` - strict warning cleanup
- `kernel/sys/syscall_v2.c` - strict warning cleanup

## Justification

1. **Deterministic regression hook** (`kernel/kernel.c`):
   - Compile-time gated: `#if AYKEN_INTENTIONAL_PERF_REGRESSION_MS > 0`
   - Default OFF
   - Required for performance constitution enforcement
   - Timer/tick based (not busy-loop)

2. **Strict warning cleanup**:
   - Enables `validation-strict` target with `-Werror`
   - Removes technical debt
   - No functional changes
   - Prepares for constitutional enforcement

## Approval

This waiver is approved for PR #11 as these changes are:
- Intentional and documented
- Required for performance constitution
- Low risk (hook default OFF, warnings cleanup)
- Aligned with governance policy

**Approved by:** Architecture Board (constitutional enforcement)  
**Valid for:** PR #11 only
