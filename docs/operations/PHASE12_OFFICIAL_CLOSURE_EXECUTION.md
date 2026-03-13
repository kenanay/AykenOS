# Phase-12 Official Closure Execution

**Status:** ACTIVE  
**Scope:** Local signed closure candidate, clean SHA alignment, dedicated closure tag, remote confirmation, formal phase transition

This runbook assumes the Phase-12 closure generator and preflight flow already exist:

- `make phase12-official-closure-prep`
- `make phase12-official-closure-preflight`
- `make phase12-official-closure-execute`

The execution rule is strict:

`official closure = signed candidate + clean git state + SHA alignment + dedicated tag + remote ci-freeze confirmation + formal phase transition`

## 1. Prepare a Clean Closure Branch

Do not execute official closure from a dirty development worktree.

1. Stage only the closure-related files you want to carry into the closure SHA.
2. Commit them.
3. Move to a clean worktree or clean branch tip before generating the signed candidate.

Example:

```bash
git checkout -b phase12/official-closure
git add Makefile \
        ayken-core/crates/proof-verifier/src/bin/closure-attest.rs \
        tools/ci/generate_phase12_closure_bundle.py \
        tools/ci/generate_phase12_official_closure_preflight.py \
        tools/ci/test_generate_phase12_closure_bundle.py \
        tools/ci/test_generate_phase12_official_closure_preflight.py \
        reports/phase12_official_closure_candidate
git commit -m "ops(phase12): prepare official closure execution"
git status --short
```

Expected:

- `git status --short` returns empty
- `git rev-parse HEAD` is the closure candidate SHA target

If the worktree is still dirty, stop here.

## 2. Export Real Attestor Material

Official closure requires real signer material.

```bash
export PHASE12_CLOSURE_ATTESTOR_NODE_ID="<real-attestor-node-id>"
export PHASE12_CLOSURE_ATTESTOR_KEY_ID="<real-attestor-key-id>"
export PHASE12_CLOSURE_ATTESTOR_PRIVATE_KEY="base64:<real-ed25519-private-key>"
export PHASE12_CLOSURE_ATTESTOR_PUBLIC_KEY="base64:<real-ed25519-public-key>"
export PHASE12_CLOSURE_ATTESTED_AT_UTC="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"
```

Expected:

- Private key is accepted only for local detached attestation generation
- Public key is used by preflight to verify the detached attestation

## 3. Regenerate the Signed Closure Candidate

Generate the candidate on the clean closure SHA, not before.

```bash
make phase12-official-closure-prep
```

Expected outputs:

- `reports/phase12_official_closure_candidate/closure_manifest.json`
- `reports/phase12_official_closure_candidate/evidence_index.json`
- `reports/phase12_official_closure_candidate/closure_manifest.attestation.payload.json`
- `reports/phase12_official_closure_candidate/closure_manifest.attestation.json`

Expected manifest state:

- `closure_state = LOCAL_CLOSURE_READY`
- `closure_attestation.attestation_state = SIGNED`
- `run.git_sha = $(git rev-parse HEAD)`

## 4. Run Local Official Closure Preflight

First produce the blocker report, then require fail-closed execution.

```bash
make phase12-official-closure-preflight
make phase12-official-closure-execute
```

Expected:

- `phase12-official-closure-preflight` writes a report under `reports/phase12_official_closure_preflight/`
- `phase12-official-closure-execute` exits `0`

If preflight is blocked, the expected blockers to clear are:

- `ATTESTATION_UNSIGNED`
- `WORKTREE_DIRTY`
- `HEAD_SHA_MISMATCH`
- any attestation verification failure

Do not create the official tag until `phase12-official-closure-execute` passes.

## 5. Create the Dedicated Closure Tag

Once local execution readiness is green, create the dedicated annotated tag on the same SHA.

```bash
git tag -a phase12-official-closure \
  -m "Phase-12 official closure candidate

manifest: reports/phase12_official_closure_candidate/closure_manifest.json
evidence_index: reports/phase12_official_closure_candidate/evidence_index.json"
git rev-parse phase12-official-closure
git rev-parse HEAD
```

Expected:

- `git rev-parse phase12-official-closure`
- `git rev-parse HEAD`

must return the same commit SHA.

## 6. Push Branch and Tag

```bash
git push origin phase12/official-closure
git push origin phase12-official-closure
```

Expected:

- remote branch contains the exact closure SHA
- remote tag points at the exact same SHA

## 7. Obtain Remote Official Confirmation

Run or observe remote `ci-freeze` on the tagged SHA.

```bash
gh run list --workflow ci-freeze --branch phase12/official-closure --limit 5
gh run watch <RUN_ID> --exit-status
```

When the run is successful, bind the run id into the preflight report:

```bash
make phase12-official-closure-preflight \
  PHASE12_CLOSURE_REMOTE_CI_RUN_ID=<RUN_ID>
```

Expected:

- remote `ci-freeze` passes on the same SHA as the signed candidate and closure tag

## 8. Execute the Formal Phase Transition

Only after signed candidate, clean local execution, tag, and remote confirmation are all satisfied:

```bash
git checkout -b phase12/formal-transition
echo "CURRENT_PHASE=12" > docs/roadmap/CURRENT_PHASE
git add docs/roadmap/CURRENT_PHASE
git commit -m "feat(phase): transition CURRENT_PHASE to 12 after official closure"
git push origin phase12/formal-transition
```

Then follow the generic transition authority:

- `docs/operations/PHASE_TRANSITION_RUNBOOK.md`

## 9. Stop Conditions

Stop the flow immediately if any of the following is true:

- the signed candidate was produced on a different SHA than `HEAD`
- the worktree is not clean
- the detached attestation does not verify with the provided public key
- the dedicated tag points anywhere other than the candidate SHA
- remote `ci-freeze` passes on a different SHA than the tag

## 10. Closure Semantics

The role split is:

- `phase12-official-closure-prep` = signed candidate generation
- `phase12-official-closure-preflight` = local readiness report
- `phase12-official-closure-execute` = local fail-closed readiness gate

Official closure still requires:

- dedicated closure tag
- remote `ci-freeze` confirmation
- formal phase transition
