# Core OS Phase 4.5 Specification (Draft)

**Status:** Draft
**Owner:** Core OS
**Target Window:** Q2 2026 (tentative)
**Prerequisites:** Phase 4.4 closure report with PASS evidence (Ring3, QEMU, syscall roundtrip)

## Purpose
Phase 4.5 advances the kernel architecture toward a minimal Ring0 mechanism set with a hardened, explicit syscall interface, while expanding hardware validation and performance governance. This phase is about *minimization, determinism, and verification* rather than new features.

## Goals
1) Minimize Ring0 responsibilities (mechanism-only).
2) Formalize syscall interface with versioning, ABI stability rules, and compatibility tests.
3) Expand hardware validation for ARM64 and RISC-V where possible.
4) Improve observability of execution flow (deterministic logging + trace points).
5) Lock performance constraints for hot paths (regression gates).

## Non-Goals
- New user-facing features or UI systems.
- Network stack.
- AI integration features (planned Phase 3 in roadmap).

## Deliverables
- Updated syscall ABI specification and versioned interface tests.
- Ring0 minimization report (diff + inventory of mechanisms only).
- Expanded QEMU validation matrix with PASS artifacts.
- Deterministic performance regression tests and thresholds.
- Phase 4.5 completion report with evidence bundle.

## Task Breakdown (Draft)

### 4.5.1 Ring0 Minimization Audit
**Objective:** Verify Ring0 contains only mechanisms and no policy.
**Acceptance Criteria:**
- Inventory of Ring0 subsystems with clear mechanism vs policy separation.
- Any policy code migrated to Ring3 or removed.
- Diff-based evidence and sign-off report.

### 4.5.2 Syscall ABI v3 (or formalization of v2)
**Objective:** Stabilize syscall ABI and versioning rules.
**Acceptance Criteria:**
- ABI spec doc with numbering, argument conventions, and error codes.
- Backward compatibility rules defined and tested.
- Golden tests for syscall signature correctness.

### 4.5.3 Ring3 Compatibility Pass
**Objective:** Ensure Ring3 userland continues to boot and execute with the new ABI rules.
**Acceptance Criteria:**
- Ring3 validation test PASS.
- Syscall roundtrip test PASS.
- Failure diagnostics are explicit and deterministic.

### 4.5.4 Multi-Arch Validation Matrix
**Objective:** Validate core boot and syscall path for additional architectures.
**Acceptance Criteria:**
- QEMU boot PASS for x86_64.
- QEMU boot PASS for ARM64 and RISC-V (where supported).
- Each architecture has a named validation artifact (log + report).

### 4.5.5 Performance Governance Gates
**Objective:** Prevent performance regression in kernel hot paths.
**Acceptance Criteria:**
- Define baseline performance metrics with thresholds.
- Automated regression detection (FAIL on breach).
- Documented methodology and evidence logs.

### 4.5.6 Phase 4.5 Closure Report
**Objective:** Provide a formal closure report with full evidence bundle.
**Acceptance Criteria:**
- Report includes test logs, command list, artifacts, and PASS summary.
- Explicit go/no-go decision for Phase 5.

## Evidence Requirements (Minimum Set)
- Updated Ring3 validation PASS log.
- Updated syscall roundtrip PASS log.
- QEMU boot PASS logs for each required architecture.
- Syscall ABI spec and validation tests.
- Performance baseline and regression gate evidence.

## Testing Plan
- `tools/validation/validate_toolchain.sh` with QEMU enabled.
- `tools/validation/ring3_validation_test.sh` with logs preserved.
- `tools/validation/syscall_roundtrip_test.sh` with logs preserved.
- Architecture-specific boot tests (ARM64, RISC-V) where toolchain is available.
- Performance regression suite (define scripts and thresholds).

## Risks
- Toolchain or QEMU dependencies missing on developer machines.
- ABI changes break Ring3 compatibility without clear diagnostics.
- Performance baselines are not reproducible across hosts.

## Open Questions
- Exact syscall ABI versioning scheme (v2 formalization vs v3).
- Minimum supported architectures for Phase 4.5 closure.
- Performance baseline hardware spec for reproducibility.

## Decision Gate
Phase 4.5 can only begin after Phase 4.4 has a PASS closure report with current evidence.
