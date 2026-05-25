# Automated Regression Finder

**Author**: Kenan AY — System Architect  
**Last Updated**: 2026-05-03

---

## Overview

The Automated Regression Finder provides automated detection of regressions using git bisect. When a previously passing validation fails, this system automatically identifies which commit introduced the regression.

---

## Architecture

### Components

1. **Oracle Script** (`scripts/oracle.sh`)
   - Provides deterministic PASS/FAIL validation
   - Returns exit 0 for PASS, exit 1 for FAIL
   - Uses smoke mode for fast iteration
   - Produces clear failure reasons

2. **Regression Finder** (`scripts/find_regression.sh`)
   - Automates git bisect process
   - Uses oracle for validation
   - Saves individual commit logs
   - Identifies first bad commit
   - Preserves git state

3. **Test Coverage** (`scripts/test_known_regressions.sh`)
   - Validates coverage of known regression patterns
   - Ensures all failure modes are detectable

---

## Usage

### Quick Start

```bash
# Find regression between last known good commit and HEAD
./scripts/find_regression.sh a1b2c3d

# Find regression between two specific commits
./scripts/find_regression.sh a1b2c3d HEAD
```

### Manual Oracle Check

```bash
# Test current commit
./scripts/oracle.sh

# Exit codes:
#   0 = PASS (validation succeeded)
#   1 = FAIL (validation failed)
```

---

## How It Works

### Binary Search

Git bisect uses binary search to minimize the number of commits tested:

- For N commits, tests approximately log₂(N) commits
- Example: 100 commits → ~7 tests
- Example: 1000 commits → ~10 tests

### Oracle Validation

For each commit, the oracle:

1. Runs smoke validation (`dev_loop.sh smoke`)
2. Checks for build failures
3. Checks for boot timeouts
4. Validates marker presence and sequence
5. Returns deterministic PASS/FAIL

### Failure Reasons

The oracle provides clear failure reasons:

- `build_failure` - Build did not complete
- `boot_timeout` - Boot exceeded timeout
- `missing_marker` - Required marker not found
- `marker_sequence_violation` - Markers out of order
- `test_failure` - Test execution failed

---

## Known Regression Patterns

The system detects these regression types:

### Build System Regressions
- Makefile changes breaking build
- Build configuration errors
- Dependency issues

### Kernel Initialization Failures
- Missing `[K][EARLY_BOOT_OK]` marker
- Early boot phase failures
- Hardware initialization issues

### Late Initialization Failures
- Missing `[K][LATE_INIT_END]` marker
- Subsystem initialization failures
- Driver initialization issues

### Boot Completion Failures
- Missing `[[AYKEN_BOOT_OK]]` marker
- Full boot sequence failures
- Runtime initialization issues

### Marker Sequence Violations
- Out-of-order markers
- Duplicate markers
- Sequence logic errors

### Runtime Contract Test Failures
- VCP runtime hook failures
- VCP trust verification failures
- VCP fail-closed failures

### Evidence Layer Test Failures
- Evidence consistency failures
- Evidence generation failures
- Evidence validation failures

---

## Output

### Bisect Logs

Individual test logs are saved to `out/logs/bisect/<commit>.log`

```bash
# View specific commit's log
cat out/logs/bisect/a1b2c3d.log
```

### First Bad Commit

The finder identifies and displays the first commit that broke validation:

```
First bad commit:
a1b2c3d Fix marker emission logic
```

### Git State

Git bisect state is automatically reset after completion, preserving your working state.

---

## Constitutional Compliance

### DETERMINISM.GLOBAL

**Requirement**: No global state mutations

**Compliance**:
- Oracle is stateless
- Same commit → same result
- No random sources used
- Reproducible validation

### Observation-Only

**Requirement**: Read-only validation

**Compliance**:
- Oracle only observes logs
- No kernel state modification
- No execution flow changes
- Pure validation logic

---

## Performance

### Smoke Mode

Oracle uses smoke mode for speed:
- Build + boot validation only
- 5-10 seconds per commit
- No contract or evidence tests
- Fast feedback for bisect

### Typical Bisect Times

| Commits | Tests | Time (smoke) |
|---------|-------|--------------|
| 10      | ~4    | ~30s         |
| 50      | ~6    | ~1min        |
| 100     | ~7    | ~1.5min      |
| 500     | ~9    | ~2min        |
| 1000    | ~10   | ~2.5min      |

---

## Examples

### Example 1: Recent Regression

```bash
# Last known good: yesterday's commit
./scripts/find_regression.sh abc123

# Output:
# [TEST] Testing commit def456... ✅ PASS
# [TEST] Testing commit ghi789... ❌ FAIL
# [TEST] Testing commit jkl012... ✅ PASS
# [TEST] Testing commit mno345... ❌ FAIL
#
# First bad commit:
# mno345 Refactor boot sequence
```

### Example 2: Older Regression

```bash
# Last known good: last week's release tag
./scripts/find_regression.sh v1.2.0

# Output:
# [TEST] Testing commit abc123... ✅ PASS
# [TEST] Testing commit def456... ✅ PASS
# [TEST] Testing commit ghi789... ❌ FAIL
# [TEST] Testing commit jkl012... ✅ PASS
# [TEST] Testing commit mno345... ❌ FAIL
# [TEST] Testing commit pqr678... ❌ FAIL
#
# First bad commit:
# pqr678 Update marker emission timing
```

### Example 3: Specific Range

```bash
# Test between two specific commits
./scripts/find_regression.sh abc123 def456

# Output:
# Good commit: abc123
# Bad commit:  def456
#
# [TEST] Testing commit ghi789... ✅ PASS
# [TEST] Testing commit jkl012... ❌ FAIL
#
# First bad commit:
# jkl012 Change boot timeout value
```

---

## Troubleshooting

### Oracle Always Fails

If oracle consistently returns FAIL:

1. Check current system state: `./scripts/oracle.sh`
2. Review oracle log: `out/evidence/regression_detection/oracle_run.log`
3. Fix current issues before running bisect

### Oracle Always Passes

If oracle consistently returns PASS:

1. Verify "bad" commit is actually bad
2. Check if issue is intermittent
3. Consider using contract or full mode

### Git State Issues

If git bisect state is corrupted:

```bash
# Manually reset bisect
git bisect reset

# Clean working directory
git clean -fd
git reset --hard
```

### Build Cache Issues

If builds are inconsistent:

```bash
# Clean build artifacts
make clean

# Run bisect again
./scripts/find_regression.sh <good-commit>
```

---

## Integration with CI

The regression finder can be integrated with CI for automated regression detection:

```yaml
# .github/workflows/auto-bisect.yml
name: Auto-Bisect on Failure

on:
  workflow_dispatch:
    inputs:
      good_commit:
        description: 'Last known good commit'
        required: true

jobs:
  bisect:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
        with:
          fetch-depth: 0  # Full history for bisect
      
      - name: Run bisect
        run: |
          ./scripts/find_regression.sh ${{ github.event.inputs.good_commit }}
      
      - name: Upload logs
        uses: actions/upload-artifact@v3
        with:
          name: bisect-logs
          path: out/logs/bisect/
```

---

## Testing

### Validate Oracle

```bash
# Test oracle mechanism
./scripts/test_regression_detection_capability.sh
```

### Validate Known Regression Coverage

```bash
# Test known regression patterns
./scripts/test_known_regressions.sh
```

### Validate Complete System

```bash
# Test all subtasks
./scripts/test_task12_automated_regression_finder.sh
```

---

## References

- **Requirements**: `.kiro/specs/dev-loop-boot-monitoring/requirements.md` (R21)
- **Design**: `.kiro/specs/dev-loop-boot-monitoring/design.md`
- **Tasks**: `.kiro/specs/dev-loop-boot-monitoring/tasks.md` (Task 12)
- **Dev Loop**: `docs/dev-loop/IMPLEMENTATION_GUIDE.md`
- **CI Integration**: `docs/dev-loop/CI_INTEGRATION.md`

---

**Maintainer**: Kenan AY — System Architect
