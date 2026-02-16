# Self-Hosted Runner Hardening (Optional Authority Profile)

## Purpose
This runbook defines an optional future profile for dedicated self-hosted freeze authority runners. Current repo workflow uses GitHub-hosted `ubuntu-latest`.

## Scope
- Repository: `kenanay/AykenOS`
- Workflow: `.github/workflows/ci-freeze.yml`
- Authority ID: `self-hosted-baremetal-x86_64-perf01`

## 1) Runner Registration (Required)
Register one dedicated runner for this repo with labels:
- `self-hosted`
- `linux`
- `x64`
- `aykenos-perf01`

Runner must be visible as `online` and `busy=false` before opening freeze PRs.

## 2) Pinned Digest Authority File (Required)
Create authority file on runner host:

```bash
sudo mkdir -p /etc/aykenos
echo "aykenos-runner-image-2026-02-14" | sudo tee /etc/aykenos/ci_image_digest
sudo chown root:root /etc/aykenos/ci_image_digest
sudo chmod 400 /etc/aykenos/ci_image_digest
```

Workflow enforces:
- file exists
- owner/group is `root:root`
- mode is `400`
- value is not empty and not `unknown`

## 3) Branch Governance (Required)
`main` protection policy:
- Require pull request before merge
- Require status checks to pass
- Require branches up to date
- Include administrators
- Disallow force push
- Disallow deletions
- Required check: `freeze`

## 4) Performance Host Stabilization (Recommended)
For deterministic perf baseline:
- Dedicated bare-metal host (no shared CI workload)
- CPU governor: `performance`
- Keep BIOS/firmware and microcode stable
- Keep QEMU/toolchain versions pinned
- Minimize background services
- Avoid containerized runner for baseline authority runs

## 5) Baseline Init Procedure (CI Only)
1. Trigger `workflow_dispatch` with `init_perf_baseline=true`
2. Provide pinned `ci_image_digest` input (required; unknown/fallback values are rejected)
3. Expect `ci-gate-performance` to return fail-closed with baseline write marker
4. Commit `scripts/ci/perf-baseline.lock.json` via PR
5. Re-run normal `ci-freeze` and verify performance gate compares against committed baseline

## 6) Verification Checklist
1. Workflow listed and triggered on `push` and `pull_request`
2. `freeze` check blocks merge when pending/failing
3. Direct push to `main` returns protected-branch rejection
4. Runner API shows labels and online status
5. Perf evidence includes:
   - `env.baseline_authority`
   - `env.ci_image_digest`
   - marker contract fields
