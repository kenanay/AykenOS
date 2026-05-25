# CI Integration: Automated Dev Loop Validation

## Overview

The AykenOS dev loop is integrated into GitHub Actions CI pipeline to provide automatic validation on every PR. When validation fails, an automated bisect process identifies the first commit that broke the system.

## CI Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                         PR Opened                                │
└────────────────────────┬────────────────────────────────────────┘
                         │
                         ▼
                  ┌──────────────┐
                  │  Smoke Test  │  (10 min timeout)
                  │   (5-10s)    │
                  └──────┬───────┘
                         │
                    ┌────┴────┐
                    │  PASS?  │
                    └────┬────┘
                         │
                    ┌────┴────┐
                    │   YES   │
                    └────┬────┘
                         │
                         ▼
                  ┌──────────────┐
                  │Contract Test │  (15 min timeout)
                  │  (1-2 min)   │
                  └──────┬───────┘
                         │
                    ┌────┴────┐
                    │  PASS?  │
                    └────┬────┘
                         │
                    ┌────┴────┐
                    │   YES   │
                    └────┬────┘
                         │
                         ▼
                  ┌──────────────┐
                  │  Full Test   │  (20 min timeout)
                  │  (2-3 min)   │
                  └──────┬───────┘
                         │
                    ┌────┴────┐
                    │  PASS?  │
                    └────┬────┘
                         │
                    ┌────┴────┐
                    │   YES   │
                    └────┬────┘
                         │
                         ▼
                  ┌──────────────┐
                  │  Isolation   │  (10 min timeout)
                  │Property Test │
                  └──────┬───────┘
                         │
                    ┌────┴────┐
                    │  PASS?  │
                    └────┬────┘
                         │
        ┌────────────────┼────────────────┐
        │                │                │
        ▼                ▼                ▼
    ┌───────┐      ┌─────────┐      ┌────────────┐
    │  YES  │      │   NO    │      │   FAIL     │
    └───┬───┘      └────┬────┘      └─────┬──────┘
        │               │                  │
        ▼               ▼                  ▼
  ┌──────────┐   ┌──────────────┐   ┌──────────────┐
  │ PR READY │   │ Auto-Bisect  │   │ Block Merge  │
  │TO MERGE  │   │  (30 min)    │   │              │
  └──────────┘   └──────┬───────┘   └──────────────┘
                        │
                        ▼
                 ┌──────────────┐
                 │Find First Bad│
                 │   Commit     │
                 └──────┬───────┘
                        │
                        ▼
                 ┌──────────────┐
                 │ Comment on PR│
                 │ with Result  │
                 └──────────────┘
```

## CI Jobs

### 1. Smoke Test
**Purpose**: Quick boot validation
**Timeout**: 10 minutes
**Command**: `./scripts/dev_loop.sh smoke`
**Checks**:
- Build succeeds
- Kernel boots
- `[[AYKEN_BOOT_OK]]` marker present

**Artifacts**: `smoke-logs`

### 2. Contract Test
**Purpose**: Runtime contract validation
**Timeout**: 15 minutes
**Depends on**: smoke
**Command**: `./scripts/dev_loop.sh contract`
**Checks**:
- All smoke validations
- VCP runtime hook behavior
- VCP trust verification
- VCP fail-closed behavior

**Artifacts**: `contract-logs`

### 3. Full Test
**Purpose**: Comprehensive validation
**Timeout**: 20 minutes
**Depends on**: contract
**Command**: `./scripts/dev_loop.sh full`
**Checks**:
- All contract validations
- VCP evidence layer
- Verification layer contracts

**Artifacts**: `full-logs`

### 4. Isolation Property Test
**Purpose**: Constitutional compliance validation
**Timeout**: 10 minutes
**Depends on**: full
**Command**: `./scripts/test_devloop_isolation.sh`
**Checks**:
- Dev loop does not affect kernel behavior
- Baseline vs dev loop runs produce identical results
- Marker sets are identical
- Kernel unaffected by broken dev loop

**Artifacts**: `isolation-logs`

### 5. Auto-Bisect (Conditional)
**Purpose**: Automatic regression detection
**Timeout**: 30 minutes
**Trigger**: Only when PR validation fails
**Depends on**: [smoke, contract, full]
**Command**: `git bisect run` with oracle

**Process**:
1. Determine good commit (merge base with target branch)
2. Determine bad commit (PR HEAD)
3. Run git bisect with `./scripts/oracle.sh`
4. Test each commit automatically
5. Identify first bad commit
6. Upload bisect logs
7. Comment on PR with results

**Artifacts**: `bisect-logs` (retention: 14 days)

**Assumptions**:
- Base branch (main/develop) is known-good
- Oracle is deterministic
- No flaky tests

## Auto-Bisect Details

### Good Commit Selection

```bash
GOOD=$(git merge-base origin/${{ github.base_ref }} HEAD)
```

**Rationale**: The merge base is the last common ancestor between the PR branch and the target branch. This is assumed to be a known-good commit.

**Risk**: If the base branch is broken, bisect will produce incorrect results.

### Bad Commit Selection

```bash
BAD=$(git rev-parse HEAD)
```

**Rationale**: The PR HEAD is the commit that failed validation.

### Oracle Script

The oracle script (`scripts/oracle.sh`) is used to test each commit:

```bash
./scripts/dev_loop.sh smoke >/dev/null 2>&1
```

**Exit codes**:
- `0` = PASS (commit is good)
- `non-zero` = FAIL (commit is bad)

**Determinism requirement**: The oracle MUST produce the same result for the same commit. Flaky tests will cause bisect to produce incorrect results.

### Bisect Process

```bash
git bisect start "$BAD" "$GOOD"
git bisect run bash -c '
  commit=$(git rev-parse --short HEAD)
  log="out/logs/bisect/${commit}.log"

  ./scripts/dev_loop.sh smoke >"$log" 2>&1
  rc=$?

  if [ "$rc" -eq 0 ]; then
    exit 0  # Good commit
  else
    exit 1  # Bad commit
  fi
'
```

**Binary search**: Git bisect uses binary search to minimize the number of commits tested. For N commits, it tests approximately log₂(N) commits.

**Example**: For 100 commits, bisect tests ~7 commits.

### PR Comment

When bisect completes, a comment is posted to the PR with:
- List of failed commits
- Link to bisect logs artifact
- Instructions for viewing detailed logs

**Example comment**:
```
## 🔴 Automated Regression Detection

The dev loop validation failed. Bisect has identified the problematic commit.

### Failed Commits:
- `e4f5g6h`
- `a1b2c3d`

### Bisect Logs
Download the `bisect-logs` artifact to see detailed logs for each tested commit.
```

## Artifacts

All CI jobs upload logs as artifacts for debugging:

| Artifact | Retention | Contents |
|----------|-----------|----------|
| `smoke-logs` | 7 days | Smoke test logs |
| `contract-logs` | 7 days | Contract test logs |
| `full-logs` | 7 days | Full validation logs |
| `isolation-logs` | 7 days | Isolation property test logs |
| `bisect-logs` | 14 days | Bisect logs (only on failure) |

**Accessing artifacts**:
1. Go to GitHub Actions run
2. Scroll to "Artifacts" section
3. Download desired artifact
4. Extract and view logs

## Branch Protection

Branch protection is controlled by `ARCHITECTURE_FREEZE.md` and
`docs/architecture-board/decisions/20260525-single-maintainer-authority-model.md`.
The authoritative protected branch is `main`.

### Required Merge Status

- `freeze` - complete constitutional CI chain on the submitted SHA and
  current protected base.

The `smoke`, `contract`, `full`, `isolation`, `performance` and `auto-bisect`
jobs described in this document are supplemental dev-loop evidence or
diagnostics. They do not independently replace or override the protected
`freeze` verdict.

### Configuration

**Automated Setup** (Recommended):
```bash
./scripts/setup_branch_protection.sh
```

**Validation**:
```bash
./scripts/validate_branch_protection.sh
```

**Manual Setup**: See `docs/dev-loop/BRANCH_PROTECTION.md` for detailed instructions.

### What Gets Enforced
- Required remote `freeze` PASS with strict base synchronization.
- Required approval count `0` under the single-maintainer authority model.
- Required code-owner review `false`; `.github/CODEOWNERS` maps
  accountability to `@kenanay` and is not independent self-review.
- Settings apply to administrators.
- Force pushes and branch deletion remain disabled.
- A CI PASS does not establish a phase closure tag or manifest.

For complete documentation, see: `docs/dev-loop/BRANCH_PROTECTION.md`

## Workflow Triggers

The CI workflow triggers on:

```yaml
on:
  pull_request:
    branches: [ main, develop ]
  push:
    branches: [ main, develop ]
```

**PR events**: Triggers on PR open, update, synchronize
**Push events**: Triggers on direct push to main/develop (not recommended)

## Environment Variables

```yaml
env:
  QEMU_TIMEOUT_SECONDS: 20
```

**Configurable per job**: Override in job-specific steps if needed.

## Timeout Strategy

| Job | Timeout | Rationale |
|-----|---------|-----------|
| smoke | 10 min | Quick test, should complete in 5-10s |
| contract | 15 min | Runtime tests, should complete in 1-2 min |
| full | 20 min | Comprehensive, should complete in 2-3 min |
| isolation | 10 min | Property test, should complete quickly |
| auto-bisect | 30 min | Multiple commits tested, longer timeout |

**Fail-fast**: If timeout is reached, job fails immediately.

## Debugging CI Failures

### Smoke Failure

1. Download `smoke-logs` artifact
2. Check `boot_watch.log` for boot markers
3. Look for `[[AYKEN_BOOT_OK]]` marker
4. Check for build errors

### Contract Failure

1. Download `contract-logs` artifact
2. Check which contract test failed
3. Review test-specific logs
4. Check for VCP runtime issues

### Full Failure

1. Download `full-logs` artifact
2. Check evidence test logs
3. Review verification layer behavior

### Isolation Failure

1. Download `isolation-logs` artifact
2. Check baseline vs dev loop comparison
3. Look for marker set differences
4. Verify kernel behavior consistency

### Auto-Bisect Failure

1. Download `bisect-logs` artifact
2. Check individual commit logs
3. Identify first bad commit
4. Review commit changes
5. Fix identified issue

## Local Testing

Verify CI workflow locally using `act`:

```bash
# Install act
brew install act  # macOS
# or
curl https://raw.githubusercontent.com/nektos/act/master/install.sh | sudo bash

# Run smoke job
act pull_request -j smoke

# Run all jobs
act pull_request
```

**Note**: `act` may not perfectly replicate GitHub Actions environment. Always verify in actual CI.

## Performance Considerations

### Build Caching

Currently, builds are not cached between jobs. Each job rebuilds the kernel.

**Future improvement**: Use GitHub Actions cache to share build artifacts between jobs.

### Parallel Execution

Jobs run sequentially (smoke → contract → full → isolation) to fail-fast.

**Rationale**: No point running contract tests if smoke fails.

### Auto-Bisect Cost

Auto-bisect can be expensive (30 min timeout, multiple builds).

**Mitigation**: Only runs on failure, not on every PR.

## Constitutional Compliance

The CI integration maintains constitutional compliance:

- ✅ **DETERMINISM.GLOBAL**: No global state mutations in validation
- ✅ **KERNEL.RING0.POLICY**: No policy decisions in Ring0
- ✅ **SECURITY.BOUNDARY.VIOLATION**: Userspace tooling only
- ✅ **Isolation guarantee**: Validated by isolation property test

**Evidence**: All logs uploaded as artifacts for audit trail.

## Future Enhancements

1. **Build caching**: Share build artifacts between jobs
2. **Parallel contract tests**: Run contract tests in parallel
3. **Performance regression detection**: Detect timing regressions
4. **Flaky test detection**: Identify non-deterministic tests
5. **Bisect optimization**: Use cached builds for faster bisect

## CI Workflow Assurance

The CI workflow configuration itself is validated using the CI workflow assurance capability:

```bash
./scripts/validate_ci_workflow.sh
```

This validates:
- Workflow file syntax
- Required jobs are present
- Job dependencies are correct
- Artifact upload is configured
- Timeout values are reasonable
- Required scripts exist and are executable
- Developer attribution is present

See `docs/dev-loop/CI_WORKFLOW_ASSURANCE.md` for details.

## References

- Workflow file: `.github/workflows/devloop-ci.yml`
- Oracle script: `scripts/oracle.sh`
- Regression finder: `scripts/find_regression.sh`
- Dev loop: `scripts/dev_loop.sh`
- Isolation test: `scripts/test_devloop_isolation.sh`
- CI workflow assurance: `scripts/validate_ci_workflow.sh`
- CI workflow assurance docs: `docs/dev-loop/CI_WORKFLOW_ASSURANCE.md`
- Branch protection setup: `scripts/setup_branch_protection.sh`
- Branch protection validation: `scripts/validate_branch_protection.sh`
- Branch protection docs: `docs/dev-loop/BRANCH_PROTECTION.md`
- Spec: `.kiro/specs/dev-loop-boot-monitoring/`
