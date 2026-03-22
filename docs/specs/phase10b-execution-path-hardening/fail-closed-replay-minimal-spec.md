# Fail-Closed Replay Minimal Spec

**Status:** Frozen minimal contract for the Phase 10-B fail-closed replay slice  
**Scope:** Normalized replay evidence emitted by the landed fail-closed proof gate

## 1. Purpose

The landed fail-closed proof path already emits a raw runtime transcript:

- debugcon markers
- one proof meta row
- one or more per-slot trace rows
- one SHA-256 transcript hash

That raw transcript is necessary for low-level audit, but it is not sufficient
as a stable replay contract. Absolute ticks, raw execution IDs, and boot-time
incidental details are observation data, not the portable proof identity.

This spec freezes the smallest replayable evidence layer above that transcript.

The goal is not to claim a complete proof system. The goal is narrower:

- keep the current fail-closed proof slice replayable
- keep the replay identity deterministic across equivalent runs
- prevent replay evidence from drifting into ad hoc log format debt

## 2. Non-Negotiable Rules

The Phase 10-B replay surface remains mechanism-only:

- kernel emits evidence
- CI validates and hashes evidence
- replay artifacts are descriptive, not authoritative runtime policy

Therefore:

- raw runtime transcript remains the source observation surface
- normalized replay artifacts remain additive above the transcript
- replay widening MUST preserve the current v1 artifact meanings
- future widening MUST use new schema/version labels instead of mutating v1

## 3. Artifact Set

The frozen replay artifact bundle for this slice is:

- `report.json`
- `proof.json`
- `replay_trace.jsonl`
- `replay_trace_hash.txt`
- `replay_report.json`
- `replay_manifest.json`
- `final_state_hash.txt`
- `replay_result_hash.txt`

Roles:

- `report.json`: gate verdict plus proof/replay summary
- `proof.json`: raw observed proof rows and transcript-hash validation result
- `replay_trace.jsonl`: normalized replay rows
- `replay_trace_hash.txt`: SHA-256 over canonical `replay_trace.jsonl` bytes
- `replay_report.json`: replay-oriented summary for consumers
- `replay_manifest.json`: compact binding object for replay hashes
- `final_state_hash.txt`: SHA-256 over the normalized final-state payload
- `replay_result_hash.txt`: SHA-256 over the pair `(replay_execution_trace_hash, final_state_hash)`

## 4. Normalization Rules

The frozen v1 normalization rules are:

- replay row order is the authoritative observed trace order
- replay rows use a synthetic `slot_ordinal`, not raw `execution_id`
- this Phase 10-B slice fixes `slot_ordinal = 1`
- local tick is `ltick = tick - first_trace_tick`
- the first replay row MUST therefore have `ltick = 0`
- raw `execution_id` and raw `generation` are audit fields only and MUST NOT
  participate in replay hashes
- absolute raw tick values MUST NOT participate in replay hashes

Each normalized replay row has this shape:

```json
{
  "trace_seq": 1,
  "slot_ordinal": 1,
  "ltick": 0,
  "event_type": "execution_slot_transition",
  "actor": 7,
  "from_state": 0,
  "to_state": 1
}
```

For the current narrow fail-closed slice:

- `trace_seq` is 1-based and contiguous
- `event_type` is fixed to `execution_slot_transition`
- `actor`, `from_state`, and `to_state` come from the observed trace row

## 5. Hash Rules

All replay hashes use canonical JSON encoding:

- UTF-8
- sorted keys
- compact separators `(",", ":")`

The frozen v1 hashes are:

1. `replay_execution_trace_hash`
   SHA-256 over the exact JSONL bytes of `replay_trace.jsonl`
2. `final_state_hash`
   SHA-256 over the normalized final-state payload
3. `replay_result_hash`
   SHA-256 over:

```json
{
  "mode": "phase10b_fail_closed_replay_v1",
  "replay_execution_trace_hash": "<sha256>",
  "final_state_hash": "<sha256>"
}
```

4. `manifest_hash`
   SHA-256 over `replay_manifest.json` with the `manifest_hash` field removed

## 6. Frozen Fields

The following identifiers are frozen for this slice:

- `mode = phase10b_fail_closed_replay_v1`
- `trace_schema = phase10b_fail_closed_replay_trace_v1`
- `final_state_schema = phase10b_fail_closed_final_state_v1`
- `manifest_version = 1`

Changing any frozen identifier requires a new replay schema version. New
consumer-facing convenience fields may be added to `report.json` or `proof.json`
only if they do not change the meaning of the frozen replay artifacts.

## 7. Phase10B Gate Integration

The canonical Phase 10-B gate MUST surface the replay bundle from the nested
fail-closed proof evidence directory.

At minimum, `scripts/ci/gate_syscall_semantics_phase10b.sh` report output must
carry paths for:

- `fail_closed_replay_trace_jsonl`
- `fail_closed_replay_trace_hash_file`
- `fail_closed_replay_report`
- `fail_closed_replay_manifest`
- `fail_closed_final_state_hash_file`
- `fail_closed_replay_result_hash_file`

The official `make ci-gate-syscall-semantics-phase10b` target SHOULD also copy
the replay report and replay manifest into the top-level `reports/` directory
for same-run review.

## 8. Explicit Non-Goals

This minimal replay contract does not yet authorize:

- multi-slot replay identity
- concurrent execution proof composition
- cryptographic signatures
- cross-host trust portability
- scheduler fairness proofs
- output-payload semantic replay
- replacement of the raw debugcon proof transcript

## 9. Follow-On Rule

Future work may widen proof coverage, but it MUST do so additively:

- keep this v1 replay format stable
- introduce new schema/version labels for wider replay shapes
- avoid retrofitting raw boot-time incidental fields into replay hashes

This keeps Phase 10-B aligned with the repo rule:

- evidence over claims
- determinism before convenience
