# AYKENOS TEST PIPELINE CONTRACT

Status: ACTIVE
Scope: External kernel/system/project test pipeline

## 1. Purpose

This contract defines the external AykenOS test pipeline.

The pipeline is authority-oriented:

1. scenario defines the run condition
2. kernel produces truth markers and evidence
3. normalizer converts evidence into a stable data shape
4. validators prove individual invariants
5. error codes describe invariant violations
6. CI consumes only the structured report

## 2. Authority Rule

The kernel is the truth producer.
The external test stack is the truth verifier.

Test logic MUST NOT be pushed into kernel runtime paths.

## 3. Pipeline Stages

### 3.1 Scenario

Scenario files define:

- scenario identifier
- domain / surface / goal
- runner command and environment
- selected validators

Scenarios describe conditions only. They do not implement validation logic.

### 3.2 Runner

The runner:

- executes the selected scenario
- collects evidence
- writes a structured run report

### 3.3 Normalizer

The normalizer:

- reads raw evidence
- emits a stable normalized report
- strips incidental log noise from validator inputs

Validators MUST consume normalized data, not raw logs.

### 3.4 Validator Set

Each validator MUST prove exactly one invariant.

Each validator MUST emit:

- `validator`
- `verdict`
- `error_code` on failure
- `message`
- `details`

### 3.5 Final Verdict

The final pipeline verdict is computed only from the structured validator results.

Raw logs are debug evidence, not primary authority.

## 4. Naming Contract

Validators/tests:

- `AYK_<DOMAIN>_<LAYER>_<SURFACE>_<INVARIANT>[_<VARIANT>]`

Scenarios:

- `AYK_SCN_<DOMAIN>_<SURFACE>_<GOAL>[_<VARIANT>]`

Error codes:

- `AYK-E<NNN>`

## 5. Exit Code Contract

- `0` = PASS
- `2` = validation failure
- `3` = scenario / infra failure

## 6. Initial Kernel-Level Scope

The first external kernel-level stack is intentionally narrow:

- domain: `KRN`
- surface: `RING3`
- goal: first fetch / first user marker boundary

Initial authoritative success chain:

- `P10_TEXT_FRAME_WITNESS`
- `P10_POST_CR3_TEXT_PROBE`
- `P10_RING3_USER_CODE`

Current v1 implementation note:

- the external kernel stack currently consumes the authoritative
  `ring3-user-leaf-rule` gate outputs as its raw evidence source
- this makes it a low-noise external verification wrapper, not yet a fully
  independent raw-evidence truth extractor
- a later revision may remove this dependency by driving the boot lane directly
  and normalizing raw evidence without the intermediate gate report

## 7. Non-Goals

This contract does not claim:

- full-kernel stability proof
- fuzzing authority
- random stress authority
- phase-based naming or phase-coupled test identity

It exists to provide a low-noise, invariant-based external verification path.
