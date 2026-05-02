# Implementation Plan: Summary Structured Insight Layer

## Overview

Additive change to `/diagnostics/summary` in Proofd. Adds a `machine_structured` projection via `display_mode=machine_structured` query parameter. All changes flow through the existing `observability_json_response` pipeline. No new endpoints, no new error paths, no changes to existing contracts.

## Tasks

- [x] 1. Update query key allowlist in `api_contract.rs`
  - Add `const SUMMARY_QUERY_KEYS: &[&str] = &["display_mode"];` before the `ROOT_DIAGNOSTICS_ENDPOINTS` array
  - Change the `Summary` entry's `allowed_query_keys` from `NO_QUERY_KEYS` to `SUMMARY_QUERY_KEYS`
  - _Requirements: 1.5, 10.1, 10.3_

- [x] 2. Add schema constant and validator in `api_schema.rs`
  - [x] 2.1 Add `MACHINE_SUMMARY_REQUIRED_FIELDS` constant
    - Define the seven `SchemaField` entries: `summary_origin`, `authority_classification`, `display_mode`, `epistemic_boundary`, `counts`, `flags`, `incident_groups`
    - _Requirements: 2.1, 2.2, 2.3, 3.1, 3.2, 3.3_

  - [ ]* 2.2 Write property test for `MACHINE_SUMMARY_REQUIRED_FIELDS` constant
    - **Property 9: Forbidden field scan returns empty for machine_structured**
    - **Validates: Requirements 3.4, 2.5**

  - [x] 2.3 Implement `validate_machine_structured_summary_contract_v1`
    - Call `require_exact_string` for `summary_origin` (`"derived"`), `authority_classification` (`"non_authoritative"`), `display_mode` (`"machine_structured"`)
    - Call `require_object` + `require_exact_bool` for `epistemic_boundary` (all three booleans `false`)
    - Call `require_object` + `require_number_field` for all six fields in `counts`
    - Call `require_object` + `require_exact_bool` for `flags` (all three booleans `false`)
    - Call `require_object` + `validate_numeric_object_values` for `incident_groups`
    - _Requirements: 2.1, 2.2, 2.3, 3.1, 3.2, 3.3, 4.1, 11.2, 11.3_

  - [x] 2.4 Update `validate_endpoint_specific_contract` dispatch for `Summary`
    - Read `display_mode` from the serialized `Value` root object
    - Dispatch `"human_readable"` → `validate_observability_summary_contract_v1` (unchanged)
    - Dispatch `"machine_structured"` → `validate_machine_structured_summary_contract_v1`
    - Return `Err(SchemaValidationError::InvalidFieldValue { field: "display_mode" })` for unknown values
    - _Requirements: 4.1, 4.2, 4.4_

- [x] 3. Add structs and constant in `lib.rs`
  - Add `const SUMMARY_DISPLAY_MODE_MACHINE_STRUCTURED: &str = "machine_structured";`
  - Add `MachineSummaryCounts` struct with six `usize` fields: `partition_count`, `total_nodes`, `total_incidents`, `agreement_count`, `conflict_count`, `island_count`
  - Add `MachineSummaryFlags` struct with three `bool` fields: `produces_truth`, `produces_decision`, `produces_ranking`
  - Add `MachineSummaryBody` struct with fields: `summary_origin: &'static str`, `authority_classification: &'static str`, `display_mode: &'static str`, `epistemic_boundary: SummaryEpistemicBoundary`, `counts: MachineSummaryCounts`, `flags: MachineSummaryFlags`, `incident_groups: BTreeMap<String, usize>`
  - All structs derive `Debug`, `Clone`, `Serialize`; no floats permitted
  - _Requirements: 2.1, 2.2, 2.3, 2.4, 3.1, 3.2, 3.3, 5.3, 8.1, 8.4, 9.3, 11.1, 11.2_

- [x] 4. Update `build_root_summary_diagnostics` and `Summary` handler arm in `lib.rs`
  - [x] 4.1 Add `display_mode: &str` parameter to `build_root_summary_diagnostics`
    - Keep all existing dependency loading code unchanged
    - Wrap existing `RootDiagnosticsSummaryBody` construction in `SUMMARY_DISPLAY_MODE_HUMAN_READABLE` match arm
    - Add `SUMMARY_DISPLAY_MODE_MACHINE_STRUCTURED` match arm that constructs `MachineSummaryBody` from loaded data and calls `serde_json::to_value`
    - Add wildcard arm returning `Err(ServiceError::BadRequest("invalid_display_mode"))`
    - _Requirements: 1.1, 1.2, 1.3, 1.4, 2.1, 2.2, 2.3, 2.4, 2.6, 3.1, 3.2, 3.3, 5.2, 6.3_

  - [x] 4.2 Update `Summary` arm in `handle_diagnostics_endpoint`
    - Extract `display_mode` from query string using `parse_query`; default to `SUMMARY_DISPLAY_MODE_HUMAN_READABLE` when absent
    - Pass `display_mode` to `build_root_summary_diagnostics`
    - _Requirements: 1.1, 1.2, 1.3, 1.4_

- [x] 5. Checkpoint — ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

- [x] 6. Write unit tests in `lib.rs` test module
  - [x] 6.1 Test: `display_mode` absent → `human_readable` response
    - _Requirements: 1.1_
  - [x] 6.2 Test: `display_mode=human_readable` → identical to absent
    - _Requirements: 1.2_
  - [x] 6.3 Test: `display_mode=machine_structured` → `MachineSummaryBody` structure with correct field values
    - _Requirements: 1.3, 2.1, 2.2, 2.3, 2.4, 3.1, 3.2, 3.3_
  - [x] 6.4 Test: `display_mode=unknown_value` → HTTP 400
    - _Requirements: 1.4_
  - [x] 6.5 Test: extra query param alongside `display_mode` → HTTP 400
    - _Requirements: 10.1, 10.4_
  - [x] 6.6 Test: empty evidence dir → HTTP 200, all counts zero, `incident_groups: {}`
    - _Requirements: 6.3_
  - [x] 6.7 Test: missing evidence dir → non-200
    - _Requirements: 6.1_
  - [x] 6.8 Test: `allowed_query_keys_for_path("/diagnostics/summary")` returns `["display_mode"]` exactly
    - _Requirements: 10.3_
  - [x] 6.9 Test: schema validator rejects `machine_structured` response with float in `counts`
    - _Requirements: 11.2, 11.3_
  - [x] 6.10 Test: schema validator rejects `machine_structured` response with `explanation` field present
    - _Requirements: 2.4_
  - [x] 6.11 Test: schema validator rejects `machine_structured` response with `authority_classification: "authoritative"`
    - _Requirements: 3.2_
  - [x] 6.12 Test: non-GET methods on `/diagnostics/summary` → HTTP 405
    - _Requirements: 7.3_
  - [x] 6.13 Test: no new URL paths registered for `machine_structured`
    - _Requirements: 7.1, 7.2_

- [ ] 7. Write property-based tests in `lib.rs` test module using `proptest`
  - [ ]* 7.1 Write property test: default projection is human_readable
    - **Property 1: Default projection is human_readable**
    - **Validates: Requirements 1.1**

  - [ ]* 7.2 Write property test: explicit human_readable equals default
    - **Property 2: Explicit human_readable equals default**
    - **Validates: Requirements 1.2**

  - [ ]* 7.3 Write property test: machine_structured returned when requested
    - **Property 3: machine_structured projection is returned when requested**
    - **Validates: Requirements 1.3**

  - [ ]* 7.4 Write property test: unknown display_mode values are rejected
    - **Property 4: Unknown display_mode values are rejected**
    - **Validates: Requirements 1.4**

  - [ ]* 7.5 Write property test: machine_structured counts match human_readable counts
    - **Property 5: machine_structured counts match human_readable counts**
    - **Validates: Requirements 2.1, 2.6**

  - [ ]* 7.6 Write property test: machine_structured flags are always false
    - **Property 6: machine_structured flags are always false**
    - **Validates: Requirements 2.2**

  - [ ]* 7.7 Write property test: incident_groups values are non-negative integers and explanation is absent
    - **Property 7: incident_groups values are integers and explanation absent**
    - **Validates: Requirements 2.3, 2.4**

  - [ ]* 7.8 Write property test: epistemic boundary invariants hold for machine_structured
    - **Property 8: Epistemic boundary invariants hold for machine_structured**
    - **Validates: Requirements 3.1, 3.2, 3.3**

  - [ ]* 7.9 Write property test: forbidden field scan returns empty for machine_structured
    - **Property 9: Forbidden field scan returns empty for machine_structured**
    - **Validates: Requirements 3.4, 2.5**

  - [ ]* 7.10 Write property test: byte-identical across repeated calls
    - **Property 10: machine_structured responses are byte-identical across repeated calls**
    - **Validates: Requirements 5.1**

  - [ ]* 7.11 Write property test: incident_groups keys are non-numeric strings
    - **Property 11: incident_groups keys are non-numeric strings**
    - **Validates: Requirements 9.2**

  - [ ]* 7.12 Write property test: extra query parameters are rejected
    - **Property 12: Extra query parameters are rejected**
    - **Validates: Requirements 10.1**

  - [ ]* 7.13 Write property test: non-GET methods return 405
    - **Property 13: Non-GET methods return 405**
    - **Validates: Requirements 7.3**

- [x] 8. Final checkpoint — ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

## Notes

- Tasks marked with `*` are optional and can be skipped for faster MVP
- Property tests use `proptest` (already in `Cargo.toml`); minimum 100 iterations per property
- Each property test must carry the tag comment: `// Feature: summary-structured-insight-layer, Property N: <property_text>`
- No floating-point fields are permitted anywhere in `MachineSummaryBody` or its nested structs
- `incident_groups` uses `BTreeMap<String, usize>` — lexicographic key order is guaranteed by the type, satisfying Requirements 5.3 and 9.4
- The `machine_structured` path shares the same dependency chain as `human_readable`; no new error variants are introduced
