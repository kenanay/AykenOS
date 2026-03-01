# Scheduler Mailbox Developer Guide

Status: ACTIVE  
Audience: Kernel and CI contributors  
Scope: Practical usage for mailbox protocol v1 (C1 single-authority)

Normative contract is defined in:

1. `docs/governance/MAILBOX_PROTOCOL_V1_FREEZE.md`
2. `kernel/include/sched_mailbox_abi.h`
3. `constitution/abi_mailbox.json`

This guide is operational and explanatory. When conflict exists, freeze doc wins.

## 1. Mental Model

Current architecture:

1. Ring3 publishes mailbox candidate proposal.
2. Ring0 validates proposal (timer path + scheduler path checks).
3. Ring0 applies mechanism only (decision consume + context switch).
4. Gates validate proof chain with markers.

C1 authority rule:

1. single owner (`AYKEN_SCHED_OWNER_PID`)
2. owner missing/mismatch is fail-closed in strict mode

## 2. Data Contract (What Ring3 Must Write)

Ring3 publish preconditions:

1. `magic == AYKEN_SCHED_MB_MAGIC`
2. `version == AYKEN_SCHED_MB_VERSION`
3. `kind == AYKEN_SCHED_HINT_CANDIDATE`
4. `epoch` strictly monotonic
5. `proposer_pid` is owner pid (Gate-4/Gate-4.5 proof modes)
6. `candidate_pid != 0`

Recommended publish sequence:

1. write payload fields
2. write `epoch` last

Reason:

1. Ring0 uses double-read epoch guard (`e1`, `e2`) to detect torn reads.

## 3. Ring0 Validation Summary

Validation path entry:

1. `sched_mailbox_validate_ring3(proc)` (timer-driven proof path)

Core checks:

1. ABI header checks (magic/version/kind)
2. torn read check (`e1 == e2`)
3. monotonic epoch (`epoch > mailbox_last_epoch`, `epoch != 0`)
4. candidate PID lookup/runnable checks
5. owner sovereignty checks (Gate-4/Gate-4.5 modes)

Outcomes:

1. ACCEPT marker: `[[AYKEN_SCHED_MB_ACCEPT]]`
2. REJECT marker: `[[AYKEN_SCHED_MB_REJECT]]` or validation result markers

## 4. Site Semantics Developers Must Respect

Decision sites in scheduler:

1. START
2. YIELD
3. BLOCK

Strict (`AYKEN_SCHED_BOOTSTRAP_POLICY=0`) expectations:

1. no owner decision -> fail-closed
2. no fallback policy path
3. BLOCK path never keep-running blocked process

Transitional (`=1`) may keep-running on yield, but this is not constitutional strict.

## 5. Marker Cheat Sheet (Proof-Critical)

Primary proof markers:

1. `[[AYKEN_RING3_PUBLISH]]`
2. `[[AYKEN_SCHED_MB_ACCEPT]]`
3. `[[AYKEN_SCHED_ARBITER_DECISION]]`
4. `[[AYKEN_CTX_SWITCH]]`
5. `[[AYKEN_PROOF_DONE]]`

Phase10 decision markers:

1. `P10_MAILBOX_DECISION ...`
2. `P10_DECISION_APPLIED ...`

Fatal/mismatch examples:

1. `P10_MAILBOX_OWNER_MISSING_FATAL`
2. `P10_MAILBOX_OWNER_NOT_READY_FATAL`
3. `P10_MAILBOX_OWNER_MISMATCH`
4. `P10_SCHED_FALLBACK_FORBIDDEN`

Do not rename proof markers without updating gate scripts and tests.

## 6. Build Knobs You Will Use

Common proof knobs:

1. `AYKEN_GATE4_POLICY_TEST`
2. `AYKEN_GATE45_PROOF`
3. `AYKEN_SCHED_BOOTSTRAP_POLICY`
4. `AYKEN_DETERMINISTIC_EXIT`
5. `AYKEN_MB_SELFTEST`

Recommended local runs:

```sh
RUN_ID=local-g4-$(date -u +%Y%m%dT%H%M%SZ) \
AYKEN_GATE45_PROOF=0 \
bash scripts/ci/gate_4_policy_accept.sh
```

```sh
RUN_ID=local-g45-$(date -u +%Y%m%dT%H%M%SZ) \
AYKEN_DETERMINISTIC_EXIT=1 \
bash scripts/ci/gate_4_5_decision_switch_proof.sh
```

## 7. Common Failure Patterns

`target_accept_mismatch`:

1. publish not emitted or not owner-consistent
2. epoch not monotonic
3. candidate not runnable

`marker_order_invalid` (Gate-4.5):

1. arbiter/switch marker emitted too early
2. missing accept-before-decision invariant

`qemu_deterministic_exit_mismatch`:

1. proof-done marker not reached
2. deterministic exit disabled/misconfigured

## 8. Do and Do Not

Do:

1. keep authority/fail-closed semantics explicit
2. update gates in same patch when marker contract changes
3. keep proof-only behavior validation-gated

Do not:

1. reintroduce hidden fallback in strict mode
2. make owner matching implicit
3. modify ABI layout without version/governance updates

## 9. State Machine (Operational View)

```text
PUBLISHED -> VALIDATED -> CONSUMED -> APPLIED -> PROOF_DONE
     |           |            |          |
     v           v            v          v
   REJECT      REJECT      FATAL      HALT/FAIL-CLOSED
```

Transition ownership:

1. publish: Ring3 policy
2. validate/consume/apply: Ring0 mechanism
3. proof_done deterministic termination: validation-only control path

