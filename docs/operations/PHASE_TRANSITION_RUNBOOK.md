# Phase Transition Runbook

**Authority:** ARCHITECTURE_FREEZE.md  
**Last Updated:** 2026-02-26  
**Status:** ACTIVE

## Overview

This runbook documents the operational procedure for transitioning between phases in AykenOS. Based on Phase 9 transition experience.

## Prerequisites

### 1. Infrastructure Complete
- All phase deliverables implemented
- Tests passing locally
- Documentation complete
- Evidence committed

### 2. CI Validation
- All 12 gates passing on feature branch
- Performance baseline current
- No outstanding violations

### 3. Branch Protection Active
- Required status check: `freeze`
- Strict mode: enabled
- Review thread resolution: required (via ruleset)

## Merge Blockers (Two Authorities)

AykenOS uses **two separate enforcement layers**:

### Layer 1: Branch Protection (`branches/main/protection`)
- Classic GitHub branch protection
- Required status checks
- Admin bypass available

### Layer 2: Repository Rulesets (`rulesets/{id}`)
- Modern GitHub rules engine
- `required_review_thread_resolution: true`
- **No admin bypass** (by design)

**Critical:** Both layers must be satisfied for merge. Ruleset violations cannot be bypassed.

## Pre-Merge Checklist

### 1. Check Unresolved Threads

```bash
gh api graphql -f query='
query($owner:String!, $name:String!, $number:Int!) {
  repository(owner:$owner, name:$name) {
    pullRequest(number:$number) {
      reviewThreads(first: 50) {
        nodes { id isResolved }
      }
    }
  }
}' -F owner=kenanay -F name=AykenOS -F number=<PR_NUMBER> \
--jq '[.data.repository.pullRequest.reviewThreads.nodes[] | select(.isResolved==false)] | length'
```

**Expected:** `0` (no unresolved threads)

**If non-zero:** Resolve threads before merge attempt.

### 2. Resolve Review Threads

```bash
# Get thread IDs
gh api graphql -f query='
query {
  repository(owner: "kenanay", name: "AykenOS") {
    pullRequest(number: <PR_NUMBER>) {
      reviewThreads(first: 10) {
        nodes {
          id
          isResolved
          comments(first: 1) {
            nodes { body }
          }
        }
      }
    }
  }
}' --jq '.data.repository.pullRequest.reviewThreads.nodes[] | {id, isResolved, body: .comments.nodes[0].body[0:80]}'

# Resolve each thread
gh api graphql -f query='
mutation {
  resolveReviewThread(input: {threadId: "<THREAD_ID>"}) {
    thread { id isResolved }
  }
}'
```

### 3. Verify CI Status

```bash
gh pr view <PR_NUMBER> --json statusCheckRollup \
  --jq '.statusCheckRollup[] | select(.name=="freeze") | {name, status, conclusion}'
```

**Expected:** `status: COMPLETED, conclusion: SUCCESS`

## Phase Transition Steps

### Step 1: Update Phase Number

```bash
git checkout -b phase/bump-to-<N>
echo "CURRENT_PHASE=<N>" > docs/roadmap/CURRENT_PHASE
```

### Step 2: Update Phase-Dependent Configuration

**Example (Phase 9 - Drift Activation):**

```bash
# Enable drift blocking
sed -i '' 's/enabled: false/enabled: true/' constitution/drift_blocking_activation.md
```

**Check for phase-dependent gates:**
- Constitutional gate: phase-aware checks
- Governance policy: phase requirements
- Drift activation: phase minimum

### Step 3: Update Phase-Aware Gates

If new phase introduces constitutional requirements, update gates:

```bash
# Example: Make gate phase-aware
# 1. Source lib-phase.sh
# 2. Read CURRENT_PHASE
# 3. Pass to Python via env
# 4. Implement phase-conditional logic
```

**Phase 9 Example:**
- `scripts/ci/gate_constitutional.sh`: drift activation check
- Phase < 9: `enabled=false` required
- Phase >= 9: `enabled=true` required

### Step 4: Create Completion Report

```bash
cat > docs/development/PHASE_<N>_COMPLETION_REPORT.md <<'EOF'
# Phase <N> Completion Report

**Date:** $(date -u +%Y-%m-%d)
**Status:** COMPLETE

## Deliverables
- [ ] Item 1
- [ ] Item 2

## Evidence
- PR #X: Implementation
- CI Run: <URL>

## Validation
- All gates: PASS
- Performance: No regression
EOF
```

### Step 5: Local Validation

```bash
# Test phase-dependent gates
make ci-gate-constitutional
make ci-gate-drift-activation
make ci-gate-governance-policy

# Full freeze suite
make ci-freeze
```

**Expected:** All gates PASS

### Step 6: Commit and Push

```bash
git add docs/roadmap/CURRENT_PHASE \
        constitution/* \
        docs/development/PHASE_<N>_COMPLETION_REPORT.md \
        scripts/ci/*

git commit -m "feat(phase): Bump to Phase <N> and activate <feature>

Phase <N> Transition: <Feature> Complete

Changes:
- CURRENT_PHASE: <N-1> → <N>
- <Feature>: enabled (constitutional requirement)
- Phase <N> completion report added

Rationale:
Phase <N> infrastructure complete and validated:
- <Deliverable 1>
- <Deliverable 2>

Evidence:
- PR #X: Implementation
- CI validation: <URL>

Constitutional Compliance:
- <Requirement 1>
- <Requirement 2>

Authority: ARCHITECTURE_FREEZE.md"

git push origin phase/bump-to-<N>
```

### Step 7: Create PR

```bash
gh pr create \
  --title "feat(phase): Bump to Phase <N> and activate <feature>" \
  --body "$(cat docs/development/PHASE_<N>_COMPLETION_REPORT.md)" \
  --base main \
  --head phase/bump-to-<N>
```

### Step 8: Monitor CI

```bash
# Wait for CI to start
sleep 10

# Get run ID
RUN_ID=$(gh run list --branch phase/bump-to-<N> --limit 1 --json databaseId --jq '.[0].databaseId')

# Watch CI
gh run watch $RUN_ID --exit-status
```

**Expected:** All 12 gates PASS

### Step 9: Resolve Review Threads

**If bot comments appear:**

```bash
# Check for unresolved threads
gh api graphql -f query='...' # (see Pre-Merge Checklist)

# Resolve each thread
gh api graphql -f query='mutation { resolveReviewThread(...) }'
```

### Step 10: Merge

```bash
gh pr merge <PR_NUMBER> --squash --delete-branch \
  --body "Phase <N> transition complete. All 12 CI gates PASS. <Feature> now active."
```

**Expected:** Merge succeeds

### Step 11: Verify Post-Merge

```bash
git checkout main
git pull

# Verify phase
cat docs/roadmap/CURRENT_PHASE
# Expected: CURRENT_PHASE=<N>

# Verify phase-dependent config
grep "^enabled:" constitution/drift_blocking_activation.md
# Expected: enabled: true (if Phase >= 9)

# Verify gates
make ci-gate-drift-activation
# Expected: drift-activation: PASS (if Phase >= 9)
```

## Common Issues

### Issue 1: "Repository rule violations... conversation must be resolved"

**Cause:** Unresolved review threads (bot comments, suggestions)

**Solution:**
1. List threads: `gh api graphql ...` (see Pre-Merge Checklist)
2. Resolve each: `gh api graphql -f query='mutation { resolveReviewThread(...) }'`
3. Retry merge

**Prevention:** Check for unresolved threads before merge attempt.

### Issue 2: Constitutional gate fails on phase bump

**Cause:** Gate has static phase-dependent check (e.g., `enabled=false` hardcoded)

**Solution:**
1. Make gate phase-aware:
   - Source `lib-phase.sh`
   - Read `CURRENT_PHASE`
   - Pass to Python via env
   - Implement conditional logic
2. Commit gate fix
3. Push and wait for CI

**Prevention:** Review all gates for phase-dependent checks before transition.

### Issue 3: Baseline lock mutation violation

**Cause:** PR includes baseline update (immutability enforcement)

**Solution:**
1. Merge baseline update separately (authorized path)
2. Rebase phase bump PR on updated main
3. Retry CI

**Prevention:** Update baseline before phase bump PR.

### Issue 4: Branch protection allows merge but ruleset blocks

**Cause:** Two separate enforcement layers (see Merge Blockers)

**Solution:**
1. Check ruleset violations: `gh pr view <PR> --json mergeStateStatus`
2. Satisfy ruleset requirements (e.g., resolve threads)
3. Retry merge

**Prevention:** Understand both enforcement layers.

## Rollback Procedure

If phase transition causes issues:

### Step 1: Revert Phase Number

```bash
git checkout -b phase/rollback-to-<N-1>
echo "CURRENT_PHASE=<N-1>" > docs/roadmap/CURRENT_PHASE
```

### Step 2: Revert Phase-Dependent Configuration

```bash
# Example: Disable drift blocking
sed -i '' 's/enabled: true/enabled: false/' constitution/drift_blocking_activation.md
```

### Step 3: Create Rollback PR

```bash
git commit -am "fix(phase): Rollback to Phase <N-1>

Reason: <Issue description>

Tracking: <Issue URL>"

git push origin phase/rollback-to-<N-1>
gh pr create --title "fix(phase): Rollback to Phase <N-1>" ...
```

### Step 4: Emergency Merge (if needed)

```bash
# If CI is broken, use admin bypass (branch protection only)
gh pr merge <PR> --admin --squash
```

**Note:** Ruleset violations cannot be bypassed. Fix must satisfy rules.

## Phase-Specific Notes

### Phase 9: Drift Activation
- **Requirement:** `enabled: true` in `constitution/drift_blocking_activation.md`
- **Gate:** `ci-gate-drift-activation` (SKIP → PASS)
- **Constitutional:** Phase-aware check in `ci-gate-constitutional`

### Phase 10+: TBD
- Document phase-specific requirements here

## References

- Phase 9 Transition: PR #18
- Constitutional Gate: `scripts/ci/gate_constitutional.sh`
- Drift Activation: `constitution/drift_blocking_activation.md`
- Branch Protection: `.github/workflows/ci-freeze.yml`

---

**Maintained by:** AykenOS Architecture Board  
**Next Review:** After each phase transition
