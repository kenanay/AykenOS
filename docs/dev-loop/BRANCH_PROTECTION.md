# Branch Protection Rules for Dev Loop CI

**Author**: Kenan AY — System Architect

## Overview

Branch protection rules enforce that all dev loop validation checks pass before code can be merged to protected branches. This ensures that broken code never enters `main` or `develop` branches.

## Required Status Checks

The following CI jobs MUST pass before merge:

| Check | Purpose | Typical Duration | Timeout |
|-------|---------|------------------|---------|
| `smoke` | Quick boot validation | 5-10s | 10 min |
| `contract` | Runtime contract validation | 1-2 min | 15 min |
| `full` | Comprehensive validation | 2-3 min | 20 min |
| `isolation` | Constitutional compliance | Quick | 10 min |
| `performance` | Performance regression check | Quick | 15 min |

**Note**: `auto-bisect` is NOT a required check. It only runs when validation fails and helps identify the problematic commit.

## Protected Branches

The following branches MUST be protected:

- `main` - Production-ready code
- `develop` - Integration branch for features

## Branch Protection Configuration

### Required Settings

1. **Require status checks to pass before merging**
   - ✅ Enabled
   - Required checks: `smoke`, `contract`, `full`, `isolation`, `performance`

2. **Require branches to be up to date before merging**
   - ✅ Enabled
   - Ensures PR is tested against latest target branch

3. **Require pull request reviews before merging** (Recommended)
   - ✅ Enabled
   - Required approvals: 1 (minimum)
   - Dismiss stale reviews: Enabled

4. **Do not allow bypassing the above settings**
   - ✅ Enabled
   - Applies to administrators

5. **Require linear history** (Optional)
   - ⚠️ Optional
   - Enforces rebase or squash merge

### Optional Settings

- **Require signed commits**: Recommended for security
- **Require deployments to succeed**: Not applicable
- **Lock branch**: Not recommended (prevents all pushes)

## Configuration Methods

### Method 1: GitHub Web Interface (Recommended for Initial Setup)

1. Navigate to repository **Settings**
2. Click **Branches** in left sidebar
3. Click **Add branch protection rule**
4. Configure as follows:

   **Branch name pattern**: `main`
   
   - ✅ Require a pull request before merging
     - Required approvals: 1
     - ✅ Dismiss stale pull request approvals when new commits are pushed
   
   - ✅ Require status checks to pass before merging
     - ✅ Require branches to be up to date before merging
     - Search and select required checks:
       - `smoke`
       - `contract`
       - `full`
       - `isolation`
       - `performance`
   
   - ✅ Do not allow bypassing the above settings
     - ✅ Apply to administrators

5. Click **Create** or **Save changes**
6. Repeat for `develop` branch

### Method 2: GitHub CLI (Recommended for Automation)

Use the provided script:

```bash
./scripts/setup_branch_protection.sh
```

This script uses GitHub CLI (`gh`) to configure branch protection rules programmatically.

**Prerequisites**:
- GitHub CLI installed (`gh`)
- Authenticated with repository access (`gh auth login`)
- Repository admin permissions

**What it does**:
- Configures protection for `main` and `develop` branches
- Sets required status checks
- Enforces PR reviews
- Applies settings to administrators

### Method 3: GitHub API (Advanced)

For integration into infrastructure-as-code:

```bash
curl -X PUT \
  -H "Accept: application/vnd.github+json" \
  -H "Authorization: Bearer $GITHUB_TOKEN" \
  https://api.github.com/repos/OWNER/REPO/branches/main/protection \
  -d @branch-protection-config.json
```

See `scripts/branch-protection-config.json` for the configuration template.

## Validation

To verify branch protection is correctly configured:

```bash
./scripts/validate_branch_protection.sh
```

This script checks:
- ✅ Protected branches exist (`main`, `develop`)
- ✅ Required status checks are configured
- ✅ All 5 required checks are present
- ✅ PR reviews are required
- ✅ Branches must be up to date
- ✅ Settings apply to administrators

**Exit codes**:
- `0` = All checks pass
- `1` = Configuration issues found

## Enforcement Behavior

### When PR is Opened

1. CI workflow triggers automatically
2. All required jobs run sequentially (smoke → contract → full → isolation → performance)
3. GitHub shows status checks in PR interface
4. Merge button is disabled until all checks pass

### When Validation Fails

1. Merge button remains disabled
2. Auto-bisect job runs (if applicable)
3. Bisect results posted as PR comment
4. Developer must fix issues and push new commits
5. CI re-runs automatically on new push

### When All Checks Pass

1. Merge button becomes enabled
2. PR can be merged (subject to review requirements)
3. Protected branch receives only validated code

## Bypass Scenarios

### Emergency Hotfixes

If branch protection must be temporarily bypassed:

1. **DO NOT** disable branch protection
2. Instead, create a temporary branch protection rule with relaxed settings
3. Apply hotfix
4. Immediately restore strict protection
5. Document bypass in audit log

**Constitutional requirement**: All bypasses MUST be documented and reviewed.

### Administrator Override

Even administrators are subject to branch protection (recommended setting).

**Rationale**: Prevents accidental merges of broken code, even by maintainers.

## Troubleshooting

### Status Check Not Appearing

**Problem**: Required check not showing in PR

**Solutions**:
1. Verify workflow file (`.github/workflows/devloop-ci.yml`) is on target branch
2. Check workflow triggers include target branch
3. Ensure job names match exactly (case-sensitive)
4. Re-run workflow manually if needed

### Merge Button Disabled Despite Passing Checks

**Problem**: All checks pass but merge still blocked

**Possible causes**:
1. Branch not up to date with target
   - Solution: Merge or rebase target branch into PR
2. Required review not approved
   - Solution: Request and obtain review approval
3. Stale review after new commits
   - Solution: Re-request review

### Check Stuck in Pending State

**Problem**: Status check never completes

**Solutions**:
1. Check GitHub Actions for workflow errors
2. Verify runner availability
3. Check for timeout issues
4. Re-run workflow

## Integration with Auto-Bisect

Auto-bisect is NOT a required status check because:

1. It only runs when validation fails
2. It's a diagnostic tool, not a validation gate
3. Its purpose is to identify the problematic commit, not block merge

**Workflow**:
1. Required checks fail → Merge blocked
2. Auto-bisect runs → Identifies first bad commit
3. Developer fixes issue → Pushes new commits
4. Required checks re-run → Must pass to unblock merge

## Constitutional Compliance

Branch protection enforces constitutional requirements:

- **R2**: Multi-level validation modes (smoke, contract, full)
- **R11**: Regression detection (auto-bisect on failure)
- **R12**: Constitutional compliance (isolation property test)
- **R22**: Performance regression detection

**Audit trail**: All merge attempts and status check results are logged by GitHub.

## Maintenance

### Adding New Required Checks

1. Add new job to `.github/workflows/devloop-ci.yml`
2. Update branch protection rules to include new check
3. Update this documentation
4. Run validation script to verify

### Removing Required Checks

1. Remove job from workflow (or make optional)
2. Update branch protection rules
3. Update documentation
4. Verify with validation script

### Changing Protected Branches

1. Add new branch to protection rules
2. Update workflow triggers if needed
3. Update documentation
4. Communicate to team

## References

- CI Workflow: `.github/workflows/devloop-ci.yml`
- Setup Script: `scripts/setup_branch_protection.sh`
- Validation Script: `scripts/validate_branch_protection.sh`
- CI Integration Guide: `docs/dev-loop/CI_INTEGRATION.md`
- GitHub Docs: [About protected branches](https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/managing-protected-branches/about-protected-branches)

## Future Enhancements

1. **CODEOWNERS integration**: Require specific reviewers for certain paths
2. **Required deployments**: Require staging deployment before production merge
3. **Commit signature verification**: Require GPG-signed commits
4. **Status check timeout**: Fail checks that exceed expected duration
5. **Flaky test detection**: Automatically retry flaky checks

---

**Last Updated**: 2026-05-03  
**Maintainer**: Kenan AY — System Architect
