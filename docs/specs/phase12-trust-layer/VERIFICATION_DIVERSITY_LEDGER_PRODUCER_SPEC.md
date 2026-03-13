# Verification Diversity Ledger Producer

**Version:** 0.1  
**Status:** Implemented (Phase-13 VDL producer V0)  
**Date:** 2026-03-14  
**Phase:** Phase-13 distributed verification observability  
**Type:** Producer specification  

---

## 1. Purpose

The Verification Diversity Ledger Producer is the canonical append surface for VDL entries.

Its purpose is to:

- derive VDL entries from verifier-local audit evidence
- bind verifier-node evidence to explicit verifier identity metadata
- enforce append-only ledger growth
- prevent duplicate entry insertion
- emit reproducible append reports for downstream diversity and cartel harnesses

The producer is not a gate.

It is a measurement substrate.

The shortest rule is:

`verification run -> audit evidence -> VDL append`

---

## 2. Canonical Inputs

The V0 producer consumes:

- `verification_audit_ledger.jsonl`
- `verification_diversity_ledger_binding.json`
- existing `verification_diversity_ledger.json` when present

V0 intentionally derives:

- `subject_bundle_id` from audit `bundle_id`
- `verification_context_id` from audit `policy_hash`
- `verification_node_id` from audit `verifier_node_id`
- `receipt_hash` from audit `receipt_hash`

V0 binds the remaining identity fields from the binding manifest:

- `verifier_id`
- `authority_chain_id`
- `lineage_id`
- `execution_cluster_id`

---

## 3. Binding Manifest

The V0 binding manifest is:

```json
{
  "binding_version": 1,
  "run_id": "<verification-run-id>",
  "verification_context_id_source": "policy_hash",
  "node_bindings": [
    {
      "verification_node_id": "<node_id>",
      "verifier_key_id": "<optional_key_id>",
      "verifier_id": "<verifier_identity>",
      "authority_chain_id": "<authority_chain_id>",
      "lineage_id": "<lineage_id>",
      "execution_cluster_id": "<optional_cluster_id>"
    }
  ]
}
```

Node bindings MUST be unique by `verification_node_id`.

If `verifier_key_id` is present in the manifest, the audit event MUST match it.

---

## 4. Append Rules

The producer MUST:

1. load the current VDL if it exists
2. derive canonical candidate entries from audit events
3. compute content-addressed `entry_id`
4. reject malformed candidate entries
5. skip already-present identical entries
6. fail if the same `entry_id` maps to different content
7. write the final ledger in stable order

Stable order for V0 is:

- `timestamp_unix_ns` ascending
- then `entry_id`

---

## 5. Canonical Entry Identity

Every produced entry MUST carry a content-addressed `entry_id`.

V0 computes it as:

- canonicalize the entry without `entry_id`
- hash canonical bytes with SHA-256
- encode as `sha256:<digest>`

This provides:

- duplicate-entry guard
- append determinism
- forensic reproducibility

---

## 6. Output Artifacts

The producer exports:

- `verification_diversity_ledger.json`
- `verification_diversity_ledger_append_report.json`
- `report.json`
- `violations.txt`

The append report MUST include at least:

- source audit ledger path
- binding manifest path
- target ledger path
- `run_id`
- candidate entry count
- appended entry count
- duplicate skipped count
- final entry count

---

## 7. Forbidden Semantics

The producer MUST NOT:

- assign trust ranking
- infer routing preference
- elect authority
- derive reputation scores

It only materializes behavioral observability entries.

The shortest rule is:

`producer materializes evidence; it does not interpret authority`

---

## 8. Relationship to Existing Harnesses

The VDL producer directly feeds:

- `ci-gate-verification-diversity-floor`
- `ci-gate-verifier-cartel-correlation`

It is therefore the first measurement substrate for:

- distribution health
- independence health
- later temporal collapse harnesses

---

## 9. Short Rule

The shortest correct reading is:

`without a canonical VDL producer, diversity and cartel harnesses remain manually fed`
