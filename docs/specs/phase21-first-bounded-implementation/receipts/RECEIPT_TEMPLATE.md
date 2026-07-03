# Phase-21 First Bounded Receipt Template

This template records the possible shape of a later exact-subject receipt.
It is not a receipt instance and does not accept evidence.

```text
receipt_id: ayken.phase21.first_bounded.receipt.<exact-sha>
subject_sha: <exact-sha>
changed_files:
  - <exact file path>
category_mapping:
  <exact file path>: <fileset category>
non_executing_boundary: recorded
denied_authorities:
  - runtime_implementation_procedure
  - code_execution
  - process_start
  - runtime_state_creation
  - package_loading
  - package_execution
  - capability_issuance
  - registry_publication
  - trust_assignment
  - source_merge_authority
evidence_runs:
  ci_freeze: <run-id>
  dev_loop: <run-id>
result: not_assigned_by_template
```

The template is not proof.

The template is not package acceptance.

The template is not source acceptance.

The template is not source merge authority.
