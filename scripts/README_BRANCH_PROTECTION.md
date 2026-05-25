# Branch Protection Scripts

**Authority:** `ARCHITECTURE_FREEZE.md`
**Decision record:** `docs/architecture-board/decisions/20260525-single-maintainer-authority-model.md`
**Maintainer:** Kenan AY

## Current Contract

These scripts implement the single-maintainer repository authority model:

| Setting | Required value |
|---|---|
| Protected branch | `main` |
| Required status | `freeze` with strict base synchronization |
| Required approvals | `0` |
| Required code-owner reviews | `false` |
| Administrator enforcement | enabled |
| Force push / branch deletion | disabled |

`.github/CODEOWNERS` maps protected paths to `@kenanay` as accountability
metadata. It is not an independent approval mechanism for a change authored
by the same maintainer.

## Configure

```bash
./scripts/setup_branch_protection.sh
```

Prerequisites:

- GitHub CLI (`gh`) installed and authenticated.
- Repository administrator permission.
- A recorded decision when changing the current authority contract.

The setup script configures `main` for the mandatory remote `freeze` verdict.
It does not grant closure or replace runtime evidence.

## Validate

```bash
./scripts/validate_branch_protection.sh
```

The validator reads the live GitHub configuration and fails if:

- strict `freeze` is not required on `main`;
- an unavailable self-review requirement is configured;
- administrator enforcement is absent; or
- force push or deletion is allowed.

Exit codes:

- `0`: configured contract matches the active authority decision.
- `1`: live configuration diverges and merge authority must remain blocked.

## API Template

The equivalent request payload is kept in:

```text
scripts/branch-protection-config.json
```

The template is for controlled administrative application only; edits to it
require the same review and CI discipline as the scripts.

## Diagnostics Boundary

Dev-loop jobs such as `smoke`, `contract`, `full`, `isolation`,
`performance`, or auto-bisect may generate useful evidence. The required
protected-branch authority is the `freeze` check. Diagnostic PASS is not
merge approval, baseline acceptance or phase closure.

## Migration

If the project later gains an independent maintainer, change this model only
through a new decision record, updated live GitHub settings and matching
validation evidence.

---

**Dijital imza / attribution:** Kenan AY - Duzenleyen, Gelistiren,
Olusturan ve Mimari Sorumlu
**Yetki notu:** Repository-governance metadata'si; runtime karari veya
closure verdict'i degildir.
