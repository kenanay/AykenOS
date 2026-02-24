# Gate-6 Behavioral Proof Suite

Gate-6 is a behavioral proof layer, not a permanent constitutional lock.

## Layering Model

- Tier-1 (permanent): ABI and structural contract (Gate-5A)
- Tier-2 (phase-scoped): marker identity/format and source anchors (Gate-5B)
- Tier-3 (behavioral): runtime proof suite (Gate-6, this directory)

This model keeps constitutional structure stable while allowing scheduler and
AI-policy behavior to evolve through phase-driven proofs.

## Files

- `suite.json`: active proofs per phase
- `semver_policy.json`: versioning guidance for behavioral suite changes
- `envelopes/*.json`: profile-scoped behavioral regression envelopes
- `DRIFT_DETECTOR.md`: drift governance rules and phase policy
- `drift_schema.json`: canonical drift report schema (ABDF/BCIB context-ready)
- `drift_profiles/*.json`: profile-scoped drift telemetry detector config
- `REPORT_CONTRACT.md`: stable Gate-6 report/output contract
- `../abdf_context.md`: canonical drift context/hash inputs
- `../drift_history_policy.md`: history retention and mutation policy
- `../drift_blocking_activation.md`: Phase-9 drift blocking activation protocol
- `../ARCHITECTURE_GOVERNANCE.md`: global layer contract map

## Phase Policy

- Phase 5 starts with minimal proofs:
  - `sched_bridge_smoke`
  - `ring3_presence_smoke`
- In Phase 5, `ring3_presence_smoke` is advisory (`WARN`) until dedicated
  Ring3 runtime evidence is enabled in the default boot harness.
- Phase 6 transition threshold:
  - `ring3_presence_smoke` returns to hard-fail mode
  - `strict_mode` is enabled for the phase
  - default boot harness must emit stable Ring3 evidence markers
- Additional behavioral proofs are enabled in later phases without changing
  Tier-1/Tier-2 constitutional baseline.
- Suite includes Phase 8/9 proof scaffolds to keep invariant coverage stable
  while envelope/drift policy evolves independently.

## Envelope Direction (Phase 7/8+)

- Gate-6 remains a proof scorer core and gains an envelope layer on top.
- Envelope policy is profile-scoped (no single global baseline):
  - `envelopes/validation.json`
  - `envelopes/experimental.json`
  - `envelopes/perf.json`
- Current CI gate executes `validation` profile; other profile envelopes are
  committed scaffolds for Phase-7/8 rollout.
- Phase strategy:
  - Phase 7: envelope telemetry / warn-first guardrails
  - Phase 8: selective hard-fail thresholds for stable profiles
- Envelope strict escalation is profile/phase controlled via
  `warn_escalates_in_strict` (Phase 7 defaults to `false`).
- Threshold rules are fail-closed on ambiguous configuration:
  choose either explicit `min/max` or `baseline+delta`, not both in one rule.
- Current envelope metrics are count/rate guards. Latency envelopes should be
  enabled after canonical marker timestamp/tick fields are added (target: Phase 9).

## Drift Placeholder (Current)

- Gate-6 report includes a non-blocking `drift` block.
- Drift uses deterministic `context_key` hashing over ABDF/BCIB context fields.
- If required context metadata is missing, drift detector stays informational only.
- Drift must not affect gate exit code until Phase-9 persistent policy is enabled.
- Phase-7/8 guarantee: drift remains non-blocking regardless of detector verdict.
- Phase-9 blocking policy exists as scaffold but is disabled by default in
  `suite.json` (`defaults.drift_blocking_policy.enabled=false`).
