# Branch Protection Scripts

**Author**: Kenan AY — System Architect

## Quick Reference

### Setup Branch Protection

Configure branch protection rules for `main` and `develop` branches:

```bash
./scripts/setup_branch_protection.sh
```

**Prerequisites**:
- GitHub CLI (`gh`) installed
- Authenticated with GitHub (`gh auth login`)
- Repository admin permissions

**What it does**:
- Configures protection for `main` and `develop`
- Sets required status checks: `smoke`, `contract`, `full`, `isolation`, `performance`
- Requires PR reviews (1 approval)
- Enforces for administrators
- Disables force pushes and branch deletion

### Validate Branch Protection

Check if branch protection is correctly configured:

```bash
./scripts/validate_branch_protection.sh
```

**Exit codes**:
- `0` = All checks pass
- `1` = Configuration issues found

**What it checks**:
- ✅ Required status checks enabled
- ✅ Branches must be up to date
- ✅ All 5 required checks present
- ✅ PR reviews configured
- ✅ Stale reviews dismissed
- ✅ Enforced for administrators
- ✅ Force pushes disabled
- ✅ Branch deletion disabled

### API Configuration Template

For infrastructure-as-code or API-based setup:

```bash
# Using curl with GitHub API
curl -X PUT \
  -H "Accept: application/vnd.github+json" \
  -H "Authorization: Bearer $GITHUB_TOKEN" \
  https://api.github.com/repos/OWNER/REPO/branches/main/protection \
  -d @scripts/branch-protection-config.json
```

Configuration template: `scripts/branch-protection-config.json`

## Files

| File | Purpose |
|------|---------|
| `setup_branch_protection.sh` | Configure branch protection via GitHub CLI |
| `validate_branch_protection.sh` | Verify branch protection configuration |
| `branch-protection-config.json` | API configuration template |

## Documentation

For detailed documentation, see: `docs/dev-loop/BRANCH_PROTECTION.md`

## Troubleshooting

### "Not authenticated with GitHub CLI"

```bash
gh auth login
```

Follow the prompts to authenticate.

### "Not in a GitHub repository"

Ensure you're in the repository root directory:

```bash
cd /path/to/AykenOS
./scripts/setup_branch_protection.sh
```

### "Failed to configure protection"

Verify you have admin permissions:

```bash
gh api /repos/OWNER/REPO --jq .permissions
```

Should show `"admin": true`.

### Status check not appearing in PR

1. Verify workflow file exists on target branch
2. Check workflow triggers include target branch
3. Ensure job names match exactly (case-sensitive)
4. Re-run workflow manually if needed

## Integration with CI

Branch protection enforces that all CI validation jobs pass before merge:

```
PR Opened → CI Runs → All Checks Pass → Merge Enabled
                   ↓
              Any Check Fails → Merge Blocked
                   ↓
              Auto-Bisect Runs → Identifies Bad Commit
```

See: `docs/dev-loop/CI_INTEGRATION.md`

## Constitutional Compliance

Branch protection enforces:
- **R2**: Multi-level validation modes
- **R11**: Regression detection
- **R12**: Constitutional compliance
- **R22**: Performance regression detection

---

**Last Updated**: 2026-05-03  
**Maintainer**: Kenan AY — System Architect
