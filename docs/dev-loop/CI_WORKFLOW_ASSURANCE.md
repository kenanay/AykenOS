# CI Workflow Assurance Capability

**Author**: Kenan AY — System Architect  
**Purpose**: Validates CI workflow configuration is correct and operational

---

## Overview

The CI workflow assurance capability validates that the GitHub Actions CI workflow for the dev loop is correctly configured and operational. This ensures that the CI pipeline itself is reliable and follows best practices.

## Purpose

The CI workflow is critical infrastructure for automated validation. If the workflow is misconfigured, it can lead to:
- False positives (passing when it should fail)
- False negatives (failing when it should pass)
- Missing validation steps
- Incorrect job dependencies
- Missing artifacts for debugging

The CI workflow assurance capability prevents these issues by validating the workflow configuration itself.

## What It Validates

### 1. Workflow File Existence and Syntax
- Workflow file exists at `.github/workflows/devloop-ci.yml`
- YAML syntax is valid
- Workflow has a descriptive name

### 2. Required Jobs
Validates that all required jobs are present:
- `smoke` - Quick boot validation
- `contract` - Runtime contract validation
- `full` - Comprehensive validation
- `isolation` - Isolation property test
- `performance` - Performance regression check
- `auto-bisect` - Automated regression finder

### 3. Job Dependencies
Validates correct job dependency chain:
- `contract` depends on `smoke`
- `full` depends on `contract`
- `isolation` depends on `full`
- `performance` depends on `isolation`
- `auto-bisect` depends on all validation jobs

This ensures fail-fast behavior: if smoke fails, contract doesn't run.

### 4. Artifact Upload
Validates that all jobs upload artifacts for debugging:
- Each job uploads logs to GitHub Actions artifacts
- Artifacts have appropriate retention periods
- Artifact names are descriptive

### 5. Timeout Values
Validates that timeout values are reasonable:
- `smoke`: 5-15 minutes
- `contract`: 10-20 minutes
- `full`: 15-25 minutes
- `isolation`: 5-15 minutes
- `performance`: 10-20 minutes
- `auto-bisect`: 25-40 minutes

### 6. Required Scripts
Validates that all required scripts exist and are executable:
- `scripts/dev_loop.sh` - Main dev loop script
- `scripts/oracle.sh` - Oracle for bisect
- `scripts/find_regression.sh` - Regression finder
- `scripts/test_devloop_isolation.sh` - Isolation property test
- `scripts/check_perf_regression.sh` - Performance regression check

### 7. Developer Attribution
Validates that developer attribution is present in the workflow file.

### 8. Workflow Triggers
Validates that workflow triggers are configured:
- Triggers on `pull_request` events
- Triggers on `push` events (optional)

### 9. Auto-Bisect Configuration
Validates auto-bisect specific configuration:
- Runs conditionally on failure
- Fetches full git history (`fetch-depth: 0`)
- Has appropriate timeout

### 10. Dependencies Installation
Validates that required dependencies are installed:
- `qemu-system-x86` - QEMU emulator
- `build-essential` - Build tools
- `clang` - C compiler
- `lld` - Linker
- `make` - Build system

### 11. Environment Variables
Validates that required environment variables are configured:
- `QEMU_TIMEOUT_SECONDS` - QEMU timeout

## Usage

### Basic Usage

```bash
./scripts/validate_ci_workflow.sh
```

This runs all validation checks and reports PASS or FAIL.

### Verbose Mode

```bash
./scripts/validate_ci_workflow.sh --verbose
```

This shows detailed information about each check.

### Help

```bash
./scripts/validate_ci_workflow.sh --help
```

Shows usage information.

## Exit Codes

- `0` - PASS: CI workflow is correctly configured
- `1` - FAIL: CI workflow has configuration errors
- `2` - USAGE: Invalid arguments

## Output

### Success Output

```
==========================================
CI Workflow Assurance Capability
Author: Kenan AY — System Architect
==========================================

[PASS] Workflow file exists: .github/workflows/devloop-ci.yml
[PASS] YAML syntax is valid
[PASS] Job 'smoke' is present
[PASS] Job 'contract' is present
[PASS] Job 'full' is present
[PASS] Job 'isolation' is present
[PASS] Job 'performance' is present
[PASS] Job 'auto-bisect' is present
[PASS] Job 'contract' depends on 'smoke'
[PASS] Job 'full' depends on 'contract'
[PASS] Job 'isolation' depends on 'full'
[PASS] Job 'performance' depends on 'isolation'
[PASS] Job 'auto-bisect' depends on all validation jobs
[PASS] Job 'smoke' uploads artifacts
[PASS] Job 'contract' uploads artifacts
[PASS] Job 'full' uploads artifacts
[PASS] Job 'isolation' uploads artifacts
[PASS] Job 'performance' uploads artifacts
[PASS] Job 'auto-bisect' uploads artifacts
[PASS] Job 'smoke' has reasonable timeout: 10 minutes
[PASS] Job 'contract' has reasonable timeout: 15 minutes
[PASS] Job 'full' has reasonable timeout: 20 minutes
[PASS] Job 'isolation' has reasonable timeout: 10 minutes
[PASS] Job 'performance' has reasonable timeout: 15 minutes
[PASS] Job 'auto-bisect' has reasonable timeout: 30 minutes
[PASS] Script 'scripts/dev_loop.sh' exists and is executable
[PASS] Script 'scripts/oracle.sh' exists and is executable
[PASS] Script 'scripts/find_regression.sh' exists and is executable
[PASS] Script 'scripts/test_devloop_isolation.sh' exists and is executable
[PASS] Script 'scripts/check_perf_regression.sh' exists and is executable
[PASS] Developer attribution present in workflow file
[PASS] Workflow triggers on pull_request
[PASS] Workflow triggers on push
[PASS] Auto-bisect runs conditionally on failure
[PASS] QEMU timeout environment variable is configured
[PASS] Dependency 'qemu-system-x86' is installed in workflow
[PASS] Dependency 'build-essential' is installed in workflow
[PASS] Dependency 'clang' is installed in workflow
[PASS] Dependency 'lld' is installed in workflow
[PASS] Dependency 'make' is installed in workflow
[PASS] Artifact retention is configured
[PASS] Auto-bisect fetches full git history
[PASS] Workflow has descriptive name: 'AykenOS Dev Loop CI'

==========================================
Validation Summary
==========================================

✅ PASS: CI workflow is correctly configured

All checks passed:
  - Workflow file exists and is valid YAML
  - All required jobs are present
  - Job dependencies are correct
  - Artifact upload is configured
  - Timeout values are reasonable
  - Required scripts exist and are executable
  - Developer attribution is present
  - Workflow triggers are configured
  - Auto-bisect runs conditionally
  - Dependencies are installed
```

### Failure Output

When validation fails, the output shows which checks failed:

```
==========================================
CI Workflow Assurance Capability
Author: Kenan AY — System Architect
==========================================

[PASS] Workflow file exists: .github/workflows/devloop-ci.yml
[FAIL] YAML syntax is invalid
[PASS] Job 'smoke' is present
[FAIL] Job 'contract' is missing
...

==========================================
Validation Summary
==========================================

❌ FAIL: CI workflow has 2 configuration error(s)

Review the errors above and fix the workflow configuration.
```

## Integration with CI

The CI workflow assurance capability can be run in CI to validate the workflow configuration itself:

```yaml
- name: Validate CI workflow configuration
  run: |
    chmod +x scripts/validate_ci_workflow.sh
    ./scripts/validate_ci_workflow.sh
```

This ensures that any changes to the CI workflow are validated before merge.

## Testing

The CI workflow assurance capability has its own test suite:

```bash
./scripts/test_ci_workflow_assurance.sh
```

This validates that the assurance capability itself works correctly.

## Design Rationale

### Why Validate the CI Workflow?

The CI workflow is critical infrastructure. If it's misconfigured:
- Broken code can be merged
- Valid code can be rejected
- Debugging becomes difficult (missing artifacts)
- Time is wasted on incorrect failures

Validating the workflow configuration prevents these issues.

### Why Not Use GitHub Actions Validation?

GitHub Actions validates YAML syntax, but doesn't validate:
- Job dependencies are correct
- Required jobs are present
- Timeout values are reasonable
- Required scripts exist
- Artifact upload is configured

This tool provides deeper validation specific to the dev loop workflow.

### Why Check Script Existence?

If a required script is missing, the CI job will fail at runtime. Checking script existence upfront provides faster feedback.

### Why Check Timeout Values?

Timeout values that are too short cause false failures. Timeout values that are too long waste CI resources. Reasonable ranges ensure efficient CI usage.

## Constitutional Compliance

The CI workflow assurance capability maintains constitutional compliance:

- ✅ **DETERMINISM.GLOBAL**: No global state mutations
- ✅ **KERNEL.RING0.POLICY**: Userspace tooling only
- ✅ **SECURITY.BOUNDARY.VIOLATION**: No Ring0 access
- ✅ **R2**: Supports multi-level validation modes
- ✅ **R12**: Validates constitutional compliance

## Future Enhancements

1. **Workflow simulation**: Simulate workflow execution locally
2. **Performance prediction**: Estimate CI runtime based on configuration
3. **Cost estimation**: Estimate CI cost based on timeout values
4. **Dependency version checking**: Validate dependency versions are pinned
5. **Security scanning**: Check for security issues in workflow configuration

## References

- CI Integration Guide: `docs/dev-loop/CI_INTEGRATION.md`
- CI Workflow: `.github/workflows/devloop-ci.yml`
- Validator Script: `scripts/validate_ci_workflow.sh`
- Test Script: `scripts/test_ci_workflow_assurance.sh`
- Spec: `.kiro/specs/dev-loop-boot-monitoring/`

---

**Last Updated**: 2026-05-03  
**Maintainer**: Kenan AY — System Architect
