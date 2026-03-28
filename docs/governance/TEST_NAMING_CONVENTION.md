# AYKENOS TEST NAMING CONVENTION

Status: ACTIVE
Scope: External test scenarios, validators, and CI-visible kernel/system/project test identifiers

## 1. Purpose

This document defines the canonical naming contract for AykenOS tests.

Goals:

- deterministic identification
- zero ambiguity in CI output
- alignment with invariant-based verification
- independence from phase/version names

## 2. Core Principle

A test name MUST describe:

1. which domain owns the test
2. which verification layer it belongs to
3. which architectural surface is validated
4. which invariant is proven

Phase names are forbidden in test identifiers.

## 3. Test Identifier Format

Canonical validator/test identifier:

`AYK_<DOMAIN>_<LAYER>_<SURFACE>_<INVARIANT>[_<VARIANT>]`

Canonical scenario identifier:

`AYK_SCN_<DOMAIN>_<SURFACE>_<GOAL>[_<VARIANT>]`

## 4. Components

### 4.1 Prefix

- `AYK` is mandatory for validators/tests
- `AYK_SCN` is mandatory for scenarios

### 4.2 Domain

Stable domain codes:

- `KRN` = kernel-level truth producer verification
- `SYS` = system-level runner / validator / CI behavior
- `PRJ` = project-level quality and governance checks
- `INT` = integration-level checks when a separate integration surface is needed

### 4.3 Layer

Stable layer codes:

- `L0` = proof / correctness
- `L1` = determinism
- `L2` = boundary
- `L3` = negative / guard

### 4.4 Surface

Stable architectural surfaces include, but are not limited to:

- `RING3`
- `CR3`
- `MMU`
- `SCHED`
- `EXEC`
- `MAILBOX`
- `CI`

### 4.5 Invariant / Goal

Invariant or goal names MUST be uppercase, descriptive, and time-stable.

Examples:

- `WITNESS_EQ_PROBE`
- `USER_CODE_REACHED`
- `PROBE_COUNT_STABLE`
- `FIRST_FETCH_BASE`

### 4.6 Variant

Variants are optional and should capture controlled variation only.

Examples:

- `IRQ_MASKED`
- `IRQ_UNMASKED`
- `RUN10`
- `TIMEOUT_SHORT`

## 5. Valid Examples

- `AYK_KRN_L0_RING3_WITNESS_EQ_PROBE`
- `AYK_KRN_L0_RING3_USER_CODE_REACHED`
- `AYK_SYS_L1_RING3_PROBE_COUNT_STABLE_RUN10`
- `AYK_PRJ_L0_CI_ERROR_CODE_VALIDATION`
- `AYK_SCN_KRN_RING3_FIRST_FETCH_BASE`
- `AYK_SCN_KRN_RING3_FIRST_FETCH_IRQ_UNMASKED`

## 6. Forbidden Patterns

The following are forbidden:

- phase-based names such as `phase10_test`
- generic names such as `test1`, `final_test`, `ring3_test`
- version-only names such as `test_v2`
- identifiers that describe a temporary implementation detail instead of an invariant

## 7. Enforcement

This naming contract is enforced by:

- `tools/ci/validate_test_naming.py`
- `make ci-gate-test-naming`

The enforcement is fail-closed for the external test tree.

## 8. Design Rule

The identifier MUST answer:

`which truth, on which surface, under which verification layer, is being proven?`

If a reader cannot answer that question from the name alone, the name is invalid.
