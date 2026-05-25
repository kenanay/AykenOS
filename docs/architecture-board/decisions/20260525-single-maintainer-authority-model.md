# Single-Maintainer Authority Model

## Metadata

- Decision ID: S2-B-20260525
- Status: approved
- Date: 2026-05-25
- Decision Authority: Kenan AY
- GitHub Owner Account: `@kenanay`
- Related Issue: https://github.com/kenanay/AykenOS/issues/145
- Related RFC: N/A - governance authority correction
- Related Waiver: None

## Context

AykenOS is currently developed and governed by one human maintainer, Kenan AY.
The repository previously named architecture, DevOps and documentation teams
in `CODEOWNERS` even though those assignable reviewer identities do not exist
in the live repository. Requiring independent approval through an alternate
account controlled by the same human, or through a local automation script,
would record a false governance claim rather than provide review independence.

This decision corrects the authority model without weakening technical
acceptance. Remote constitutional CI, immutable evidence boundaries,
locked-baseline performance checks and closure requirements remain mandatory.

## Decision

`approved`

1. Kenan AY is the sole human maintainer and the human decision authority for
   architecture, CI/build, baseline renewal, merge and closure decisions while
   the repository remains single-maintainer.
2. `.github/CODEOWNERS` MUST map protected surfaces to the actual accountable
   GitHub account, `@kenanay`. In this model it is ownership and routing
   metadata; it is not an independent self-review requirement.
3. Remote constitutional CI and evidence checks remain fail-closed merge
   requirements. A documented maintainer decision does not replace a failing
   or absent mandatory check.
4. A secondary GitHub account controlled by Kenan AY, an automated reviewer,
   or an untracked/local script MUST NOT be represented as an independent
   human approval.
5. This governance decision changes no runtime authority. Evidence and
   diagnostics remain output-only and cannot direct execution or policy.

## Repository Protection Alignment

While this decision is active, live repository protection must not require an
impossible self-approval or pretend that code-owner review supplies independent
review. It must continue to enforce the required status checks and protected
branch safeguards applicable to the project.

Transition procedure:

1. Validate this governance change through its pull request CI checks.
2. Align live branch/ruleset review configuration to this decision, retaining
   mandatory status checks and protected-branch controls.
3. Record the live configuration result on issue #145.
4. Only then may blocked predecessor PRs be reviewed or restacked under the
   accepted authority model.

## Future Multi-Maintainer Migration

The single-maintainer model is reversible. When an additional genuinely
independent maintainer is available, a new decision record MUST define roles,
assignable owners and required-review rules. The GitHub protection
configuration and `CODEOWNERS` must be changed in the same governed
transition before independent approval is claimed.

## Evidence and Boundaries

- Required evidence: pull request CI results for this change and the recorded
  live-protection alignment on issue #145.
- Non-goal: granting Phase-17 closure or bypassing a required CI result.
- Non-goal: changing kernel, runtime, ABI, performance threshold or baseline.

## Follow-ups

1. Validate and accept this S2-B governance change.
2. Align live repository protection and resolve issue #145 against this model.
3. Restack/review the blocked Phase-17 predecessor chain on the accepted base.
4. Re-evaluate multi-maintainer governance only when independent ownership
   actually exists.

## Sign-off

- Maintainer Decision: Kenan AY
- Capacity: Duzenleyen, Gelistiren, Olusturan ve Mimari Sorumlu
- Boundary: Documentation and repository governance authority only; not a
  runtime decision or evidence verdict.
