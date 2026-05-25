# Branch Protection Rules for Constitutional CI

**Authority:** `ARCHITECTURE_FREEZE.md`
**Decision record:** `docs/architecture-board/decisions/20260525-single-maintainer-authority-model.md`
**Duzenleyen / Gelistiren / Olusturan / Mimari Sorumlu:** Kenan AY
**Effective update:** 2026-05-25

## Purpose

Branch protection is a repository authority boundary. It prevents a change
from entering `main` unless the required constitutional CI verdict exists for
the submitted SHA.

AykenOS currently has one human maintainer. Therefore protection MUST NOT
require an impossible self-approval or present another account controlled by
the same person as independent review.

## Enforced Main Configuration

| Control | Required value | Rationale |
|---|---|---|
| Protected branch | `main` | Accepted source of repository authority |
| Required status check | `freeze` | Full constitutional CI chain |
| Require branch up to date | `true` | Verdict covers the current base |
| Required approval count | `0` | One maintainer cannot independently approve own change |
| Code-owner review requirement | `false` | `CODEOWNERS` is ownership metadata, not self-review authority |
| Administrator enforcement | `true` | Maintainer cannot bypass failed CI |
| Force push / deletion | disabled | Accepted history remains controlled |
| Review thread resolution | enabled by active ruleset | Unresolved findings remain visible blockers |

The active ownership map in `.github/CODEOWNERS` routes protected surfaces to
`@kenanay`. It records accountability and review routing only.

## CI Boundary

The required merge authority is the remote `freeze` check emitted by the
constitutional workflow. Jobs such as `smoke`, `contract`, `full`,
`isolation`, `performance` and auto-bisect may still provide dev-loop evidence
or diagnostics, but they are not independently configured live merge
requirements under the single-maintainer decision.

A green diagnostic result is not a waiver for a failed or missing `freeze`
verdict. A green `freeze` verdict is not a phase closure tag or manifest.

## Protected Branch Scope

`main` is the protected authoritative branch in this decision. A `develop`
branch or any future integration branch does not gain protected-authority
status until it is expressly adopted through a reviewed decision and is
configured with the required constitutional verdict.

## Configuration and Validation

Configure the current contract:

```bash
./scripts/setup_branch_protection.sh
```

Validate the live repository configuration:

```bash
./scripts/validate_branch_protection.sh
```

Both scripts target `main`, require strict `freeze`, preserve administrator
enforcement, and verify that the repository does not claim independent
self-review.

The API template is `scripts/branch-protection-config.json`.

## Merge Procedure

1. Open a pull request against the intended accepted base.
2. Obtain remote `freeze` PASS on the exact candidate SHA and current base.
3. For architecture, governance, baseline or closure-affecting changes,
   record Kenan AY's maintainer decision with the related evidence.
4. Resolve outstanding review discussions or tracked blockers.
5. Merge only after GitHub reports the protected checks and configuration as
   satisfied.

## Failure and Emergency Handling

- A missing or failed `freeze` check blocks merge.
- A local PASS is diagnostic evidence only and cannot substitute for remote
  required CI.
- A second account or an automation script controlled by the same maintainer
  cannot be recorded as an independent reviewer.
- Protection MUST NOT be relaxed for an emergency change without a tracked
  maintainer decision and follow-up restoration evidence.

## Future Multi-Maintainer Transition

The single-maintainer model is reversible. When a genuinely separate,
assignable reviewer joins the project, the change requires:

1. A new architecture/governance decision record.
2. Updated `CODEOWNERS` assignments and live branch-protection settings.
3. Validation evidence showing the independent review requirement is
   enforceable.
4. Updated operating documentation before relying on the new authority.

## Constitutional Boundary

Branch protection enforces merge conditions. It does not itself establish
runtime correctness, a performance baseline renewal, or Phase-17 official
closure.

---

**Dijital imza / attribution:** Kenan AY - Duzenleyen, Gelistiren,
Olusturan ve Mimari Sorumlu
**Yetki notu:** Belgesel ve repository-governance metadata'si; runtime
karari veya evidence verdict'i degildir.
