# Requirements Document

## Introduction

Proofd is a read-only diagnostics service for the AykenOS verification pipeline. Its core invariant — the Phase-14 epistemic boundary — guarantees that the service produces no truth, no decision, and no ranking. All responses flow through the `observability_json_response` pipeline (contract → schema → validation → forbidden field scan).

The current `/diagnostics/summary` endpoint returns a `human_readable` projection: a narrative explanation alongside aggregate counts. This feature adds a `machine_structured` projection to the same endpoint, enabling CLI tools, dashboards, and CI automation to consume summary data without text parsing. The structured projection contains only counts, flags, and groupings — no scores, no priorities, no routing hints, no authority over verification outcome.

This is a pure additive change. The existing endpoint path, the existing `human_readable` default, the Phase-14 graph contract, and all CI gates remain untouched.

## Glossary

- **Proofd**: The read-only diagnostics service for the AykenOS verification pipeline.
- **Summary_Endpoint**: The HTTP endpoint at `/diagnostics/summary` that returns aggregate diagnostic data.
- **Display_Mode**: A query parameter on the Summary_Endpoint that selects the response projection. Valid values: `human_readable` (default), `machine_structured` (opt-in).
- **Human_Readable_Projection**: The existing summary response format containing narrative `explanation` strings alongside aggregate counts.
- **Machine_Structured_Projection**: The new opt-in response format containing only counts, flags, and groupings — no narrative text, no scores, no rankings.
- **Epistemic_Boundary**: The Phase-14 invariant: Proofd produces no truth, no decision, no ranking. Enforced via the forbidden field scan on every response.
- **Forbidden_Field_Scan**: The `scan_forbidden_observability_fields` function that rejects any response containing fields from the Phase-13/14 forbidden list (e.g., "winner", "score", "recommended_action", "priority").
- **Observability_Pipeline**: The response pipeline: contract resolution → schema validation → forbidden field scan → JSON serialization.
- **Structured_Counts**: Integer counts of nodes, incidents, partitions, agreements, conflicts, and islands derived from the same evidence as the human_readable projection.
- **Structured_Flags**: Boolean indicators derived from the epistemic boundary object (e.g., `produces_truth: false`).
- **Structured_Groupings**: Incident type breakdowns as a map of string keys to integer counts.
- **Evidence_Dir**: The filesystem directory containing run artifacts that Proofd reads to build responses.

## Requirements

### Requirement 1: Display Mode Selection

**User Story:** As a CLI tool author, I want to request a machine-readable summary from the existing `/diagnostics/summary` endpoint, so that I can parse diagnostic data without text processing.

#### Acceptance Criteria

1. WHEN a GET request is made to `/diagnostics/summary` without a `display_mode` query parameter, THE Summary_Endpoint SHALL return a response with `display_mode` equal to `"human_readable"`.
2. WHEN a GET request is made to `/diagnostics/summary` with `display_mode=human_readable`, THE Summary_Endpoint SHALL return a response identical to the response produced without any `display_mode` parameter.
3. WHEN a GET request is made to `/diagnostics/summary` with `display_mode=machine_structured`, THE Summary_Endpoint SHALL return a response with `display_mode` equal to `"machine_structured"`.
4. WHEN a GET request is made to `/diagnostics/summary` with an unrecognized `display_mode` value, THE Summary_Endpoint SHALL return HTTP 400 with an error body.
5. THE Summary_Endpoint SHALL accept `display_mode` as the only new query parameter; no other new query parameters SHALL be introduced.

### Requirement 2: Machine-Structured Projection Content

**User Story:** As a dashboard engineer, I want the machine-structured summary to contain numeric counts and boolean flags only, so that I can render visualizations without parsing narrative text.

#### Acceptance Criteria

1. WHEN `display_mode=machine_structured` is requested, THE Summary_Endpoint SHALL include a `counts` object containing integer fields: `partition_count`, `total_nodes`, `total_incidents`, `agreement_count`, `conflict_count`, `island_count`.
2. WHEN `display_mode=machine_structured` is requested, THE Summary_Endpoint SHALL include a `flags` object containing boolean fields derived from the epistemic boundary: `produces_truth`, `produces_decision`, `produces_ranking`.
3. WHEN `display_mode=machine_structured` is requested, THE Summary_Endpoint SHALL include an `incident_groups` object where every value is a non-negative integer count keyed by incident type string.
4. WHEN `display_mode=machine_structured` is requested, THE Summary_Endpoint SHALL NOT include a narrative `explanation` array.
5. THE Machine_Structured_Projection SHALL NOT contain any field whose normalized name appears in the Forbidden_Field_Scan list (including but not limited to: "score", "priority", "winner", "recommended_action", "routing_hint", "mitigation", "rank").
6. FOR ALL valid Evidence_Dir states, the `counts` fields in the Machine_Structured_Projection SHALL equal the corresponding numeric values in the Human_Readable_Projection produced from the same Evidence_Dir.

### Requirement 3: Epistemic Boundary Preservation

**User Story:** As the AykenOS architectural steward, I want the machine-structured projection to carry the same epistemic boundary declaration as the human-readable projection, so that consumers cannot mistake the structured output for an authoritative verdict.

#### Acceptance Criteria

1. WHEN `display_mode=machine_structured` is requested, THE Summary_Endpoint SHALL include `summary_origin` equal to `"derived"`.
2. WHEN `display_mode=machine_structured` is requested, THE Summary_Endpoint SHALL include `authority_classification` equal to `"non_authoritative"`.
3. WHEN `display_mode=machine_structured` is requested, THE Summary_Endpoint SHALL include an `epistemic_boundary` object with `produces_truth: false`, `produces_decision: false`, and `produces_ranking: false`.
4. THE Forbidden_Field_Scan SHALL return an empty violation list for every Machine_Structured_Projection response.
5. THE Machine_Structured_Projection SHALL NOT contain any field that implies authority over the verification outcome, including fields that suggest routing, execution control, or node selection.

### Requirement 4: Observability Pipeline Compliance

**User Story:** As a CI gate maintainer, I want the machine-structured projection to pass all existing pipeline checks, so that no new CI gates are required and no existing gates are weakened.

#### Acceptance Criteria

1. WHEN `display_mode=machine_structured` is requested, THE Observability_Pipeline SHALL apply schema validation to the Machine_Structured_Projection before returning the response.
2. WHEN `display_mode=machine_structured` is requested, THE Observability_Pipeline SHALL apply the Forbidden_Field_Scan to the Machine_Structured_Projection before returning the response.
3. IF the Forbidden_Field_Scan detects a violation in the Machine_Structured_Projection, THEN THE Summary_Endpoint SHALL return HTTP 500 and SHALL NOT return the violating response body.
4. IF schema validation fails for the Machine_Structured_Projection, THEN THE Summary_Endpoint SHALL return HTTP 500 and SHALL NOT return the invalid response body.
5. THE Summary_Endpoint SHALL NOT modify the Phase-14 graph contract or any existing schema definition to accommodate the Machine_Structured_Projection.

### Requirement 5: Determinism

**User Story:** As a CI automation engineer, I want repeated calls to the machine-structured summary with the same evidence state to produce identical output, so that I can use the response in deterministic pipelines.

#### Acceptance Criteria

1. FOR ALL valid Evidence_Dir states, calling GET `/diagnostics/summary?display_mode=machine_structured` twice in succession SHALL produce byte-identical JSON response bodies.
2. THE Machine_Structured_Projection SHALL derive all field values exclusively from the Evidence_Dir contents; THE Summary_Endpoint SHALL NOT incorporate system time, random values, or process-local state into the Machine_Structured_Projection.
3. THE Machine_Structured_Projection SHALL use deterministic key ordering (lexicographic) in all JSON objects.

### Requirement 6: Fail-Closed Behavior

**User Story:** As a CI automation engineer, I want the endpoint to fail closed when evidence data is missing or malformed, so that consumers never receive a misleading partial structured response.

#### Acceptance Criteria

1. IF the Evidence_Dir is absent or unreadable when `display_mode=machine_structured` is requested, THEN THE Summary_Endpoint SHALL return a non-200 HTTP status code.
2. IF the underlying parity or graph data required to compute counts is malformed when `display_mode=machine_structured` is requested, THEN THE Summary_Endpoint SHALL return a non-200 HTTP status code rather than a partial Machine_Structured_Projection.
3. WHILE the Evidence_Dir is readable but contains no run data, THE Summary_Endpoint SHALL return HTTP 200 with all counts set to zero and `incident_groups` set to an empty object.

### Requirement 8: Counts Semantic Contract

**User Story:** As the AykenOS architectural steward, I want the counts object to be explicitly marked as descriptive aggregates, so that consumers cannot treat numeric counts as evaluative signals or decision inputs.

#### Acceptance Criteria

1. THE `counts` object in the Machine_Structured_Projection SHALL be a descriptive aggregate only; it SHALL NOT imply evaluation, ranking, or ordering of any verification node, partition, or run.
2. THE schema definition for the `counts` object SHALL document that all integer fields are observation counts derived from evidence, not scores or weights.
3. A consumer reading only the `counts` object SHALL NOT be able to derive a verdict, ranking, or routing decision from its values alone.
4. THE `counts` fields SHALL be named to reflect observation semantics (e.g., `total_incidents`, `partition_count`) and SHALL NOT use names that imply evaluation (e.g., `incident_severity_score`, `partition_weight`).

### Requirement 9: Incident Groups Semantic Contract

**User Story:** As the AykenOS architectural steward, I want the incident_groups object to carry no implied ordering or priority, so that consumers cannot use grouping keys as a severity ranking.

#### Acceptance Criteria

1. THE `incident_groups` object SHALL NOT imply ordering, priority, or severity ranking among its keys.
2. THE keys of `incident_groups` SHALL be incident type identifiers only; they SHALL NOT be numeric indices or ranked labels.
3. THE values of `incident_groups` SHALL be non-negative integer counts only; no floating-point, percentage, or ratio values are permitted.
4. THE ordering of keys in `incident_groups` SHALL be lexicographic (deterministic) and SHALL NOT reflect frequency, severity, or any other evaluative ordering.

### Requirement 10: Query Parameter Isolation

**User Story:** As a CI gate maintainer, I want the display_mode query parameter to be the only accepted parameter on the summary endpoint, so that no parameter combination can produce an authoritative or filtered response.

#### Acceptance Criteria

1. THE Summary_Endpoint SHALL accept `display_mode` as the sole query parameter; any request containing additional query parameters SHALL return HTTP 400.
2. THE `display_mode` parameter SHALL NOT be combinable with any filter, sort, or scope parameter.
3. THE allowlist for `/diagnostics/summary` in `allowed_query_keys_for_path` SHALL contain exactly one entry: `"display_mode"`.
4. THE query validation logic SHALL reject any request where the query string contains more than one unique key, even if the extra key is unrecognized.

### Requirement 11: No Derived Ratio or Composite Fields

**User Story:** As the AykenOS architectural steward, I want to prevent any derived composite metric from appearing in the machine-structured projection, so that consumers cannot extract an implicit evaluation from count combinations.

#### Acceptance Criteria

1. THE Machine_Structured_Projection SHALL NOT contain any field that combines two or more counts into a ratio, rate, percentage, or comparative metric (e.g., `conflict_ratio`, `agreement_rate`, `incident_density`).
2. ALL numeric fields in the Machine_Structured_Projection SHALL be raw integer observation counts only; floating-point values are not permitted.
3. THE schema definition for the Machine_Structured_Projection SHALL enforce integer types for all numeric fields and SHALL reject floating-point values at validation time.
4. A consumer reading the Machine_Structured_Projection SHALL NOT be able to derive an implicit ranking or score without performing external arithmetic that is outside the contract of this endpoint.

### Requirement 7: No New Endpoint Paths

**User Story:** As the AykenOS architectural steward, I want the structured insight layer to be delivered on the existing endpoint only, so that the API surface does not expand.

#### Acceptance Criteria

1. THE Summary_Endpoint SHALL serve both `human_readable` and `machine_structured` projections exclusively at `/diagnostics/summary`.
2. THE Proofd service SHALL NOT register any new URL path to serve the Machine_Structured_Projection.
3. FOR ALL HTTP methods other than GET on `/diagnostics/summary`, THE Summary_Endpoint SHALL return HTTP 405, regardless of the `display_mode` parameter.
