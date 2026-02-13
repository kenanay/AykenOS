# RFC-0001 Template

## Metadata

- RFC ID:
- Title:
- Author:
- Date:
- Freeze Impact: `none | compatible | breaking`
- Related Issue:

## Motivation

Bu değişiklik neden gerekli?

## Scope

Neler dahil, neler hariç?

## Architectural Impact Analysis

1. ABI impact
2. Ring0/Ring3 boundary impact
3. Performance impact
4. Security impact

## Contract Changes

- Changed contracts: `yes/no`
- If yes: exact path + delta summary

## Gate Plan

Required gates:
1. `ci-gate-abi`
2. `ci-gate-boundary`
3. `ci-gate-workspace`
4. `ci-gate-hygiene`
5. `ci-gate-performance`

Evidence run id:
- `run_id`:
- `evidence_path`:

## Regression Plan

Nasıl tespit edilecek?

## Rollback Plan

Nasıl geri alınacak?

## Waiver Need

- Waiver required: `yes/no`
- If yes: link `docs/waivers/<id>.md`

## Approval

- Architecture Board decision link:
- Final status: `approved | rejected | deferred`
