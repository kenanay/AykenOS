# Self-Hosted Runner Security (Public Repo)

## ⚠️ WARNING

**DO NOT use self-hosted runners on public repositories without proper security hardening.**

GitHub official warning:
> "We recommend that you do not use self-hosted runners with public repositories. Forks of your public repository can potentially run dangerous code on your self-hosted runner machine by creating a pull request."

## Risk Assessment

### Attack Vector
1. Attacker forks public repo
2. Adds malicious code to workflow
3. Opens PR
4. Malicious code runs on your self-hosted runner
5. Attacker gains access to:
   - Runner machine
   - GitHub tokens
   - Secrets
   - Network access

### Impact
- **Critical**: Full machine compromise
- **High**: Secret exfiltration
- **High**: Network lateral movement
- **Medium**: Resource abuse (crypto mining)

## Security Hardening (If You Must)

### 1. Workflow Approval Required
```yaml
# .github/workflows/ci-freeze.yml
on:
  pull_request_target:  # Use pull_request_target instead of pull_request
    types: [opened, synchronize]
```

Enable "Require approval for all outside collaborators" in Settings → Actions.

### 2. Ephemeral Runners
Use Docker containers that are destroyed after each job:

```bash
# Run runner in Docker
docker run --rm \
  -e RUNNER_NAME="ephemeral-runner" \
  -e RUNNER_WORKDIR="/tmp/runner" \
  -e RUNNER_TOKEN="YOUR_TOKEN" \
  myorg/github-runner:latest
```

### 3. Network Isolation
```bash
# Firewall rules
iptables -A OUTPUT -d github.com -j ACCEPT
iptables -A OUTPUT -d api.github.com -j ACCEPT
iptables -A OUTPUT -j DROP  # Block all other outbound
```

### 4. No Secrets on Self-Hosted
Never store secrets on self-hosted runners for public repos.

### 5. Separate Runner Group
Create dedicated runner group for trusted workflows only.

### 6. Audit Logging
```bash
# Enable audit logging
./config.sh --url https://github.com/org/repo \
  --token TOKEN \
  --labels audit-enabled \
  --work /tmp/runner
```

### 7. Read-Only Filesystem
```bash
# Mount runner workspace as read-only where possible
docker run --read-only \
  --tmpfs /tmp \
  --tmpfs /var/run \
  github-runner:latest
```

## Recommended Approach for AykenOS

### Option 1: GitHub-Hosted (Recommended)
- ✅ Secure by default
- ✅ No maintenance
- ✅ Isolated per job
- ❌ Requires billing

### Option 2: Local Development Only
- ✅ No security risk
- ✅ Fast iteration
- ✅ No GitHub dependency
- ❌ No CI enforcement

### Option 3: Private Repo + Self-Hosted
- ✅ Self-hosted performance
- ✅ Controlled access
- ❌ Not open source

## Decision Matrix

| Scenario | Solution | Security | Cost |
|----------|----------|----------|------|
| Public repo, active development | GitHub-hosted | ✅ High | 💰 Paid |
| Public repo, no billing | Local dev only | ✅ High | 💰 Free |
| Private repo | Self-hosted OK | ⚠️ Medium | 💰 Free |
| Public repo + self-hosted | **DO NOT** | ❌ Critical | 💰 Free |

## Current AykenOS Strategy

```
Development:  Local baseline (local-dev-Darwin-arm64)
CI/Production: GitHub-hosted (github-hosted-ubuntu-latest-x64)
```

**Do not add self-hosted runner to public AykenOS repo.**

## References

- [GitHub: Security hardening for self-hosted runners](https://docs.github.com/en/actions/security-guides/security-hardening-for-github-actions#hardening-for-self-hosted-runners)
- [GitHub: About self-hosted runners](https://docs.github.com/en/actions/hosting-your-own-runners/managing-self-hosted-runners/about-self-hosted-runners)
