# Implementation Plan: obs-cli-consumer

## Overview

Incremental build of the `obs-cli` crate in `userspace/obs-cli/`. Organized as two phases:

**Phase 1 — MVP vertical slice** (Tasks 1–7): uçtan uca çalışan minimal CLI. `error → models → parser → formatter → printer → main (file input)`. Her task bağımsız commit edilebilir ve hygiene gate'i geçer.

**Phase 2 — Full feature** (Tasks 8–15): `fetcher`, `cli`, `threshold`, `diff`, full wiring. Phase 1 tamamlandıktan sonra başlanır.

## Tasks

### Phase 1 — MVP Vertical Slice

- [x] 1. Scaffold crate and register in workspace
  - Create `userspace/obs-cli/Cargo.toml` with `[package]` (name = "obs-cli",
    edition = "2021") and `[dependencies]` for `serde`, `serde_json`
    (Phase 1 only — `ureq` added in Phase 2), and `[dev-dependencies]` for `proptest` and
    `tempfile`
  - Create `userspace/obs-cli/src/main.rs` with a stub `fn main() {}`
  - Add `"obs-cli"` to the `members` list in `userspace/Cargo.toml`
  - Verify `cargo check -p obs-cli` passes
  - _Requirements: 11.1, 11.2_

- [x] 2. Implement `error.rs` — `AppError` enum and exit codes
  - [x] 2.1 Create `userspace/obs-cli/src/error.rs`
    - Define `AppError` enum with variants: `Usage(String)`, `Http(u16, String)`,
      `Io(String)`, `Parse(String)`, `Schema(String)`, `Threshold(Vec<String>)`
    - Implement `exit_code(&self) -> i32`: Usage→1, Http→2, Io→2, Parse→3, Schema→3,
      Threshold→4
    - Implement `message(&self) -> String` that formats each variant; for
      `Threshold` join all violation strings with newlines; for `Http` include
      status code and body
    - Implement `std::fmt::Display` for `AppError` delegating to `message()`
    - No `unwrap()`/`expect()` anywhere in this file
    - _Requirements: 10.3_

- [x] 3. Implement data models — `Snapshot`, `Counts`, `SnapshotFlags`, `Diff`
  - [x] 3.1 Create `userspace/obs-cli/src/models.rs` (or inline in `lib.rs`)
    - Define `Counts` with six `usize` fields; derive `Debug, Clone, PartialEq,
      Serialize, Deserialize`
    - Define `SnapshotFlags` with three `bool` fields (`produces_truth`,
      `produces_decision`, `produces_ranking`); same derives; used as
      `flags: SnapshotFlags` in `Snapshot` — name is `SnapshotFlags` everywhere,
      no alias
    - Define `Snapshot` with `authority_classification: String`, `counts: Counts`,
      `flags: SnapshotFlags`, `incident_groups: BTreeMap<String, usize>`; same
      derives; `BTreeMap` guarantees lexicographic key order
    - Define `CountsDiff` with six `i64` fields; derive `Debug, Clone, PartialEq`
    - Define `IncidentGroupDelta` enum: `Added(usize)`, `Removed(usize)`,
      `Changed { baseline: usize, current: usize, delta: i64 }`, `Unchanged(usize)`
    - Define `Diff` with `counts: CountsDiff` and
      `incident_groups: BTreeMap<String, IncidentGroupDelta>`
    - No `f32`/`f64` anywhere; no `unwrap()`/`expect()`
    - _Requirements: 2.1, 11.4_

- [x] 4. Implement `parser.rs` — deserialize and validate `Snapshot`
  - [x] 4.1 Create `userspace/obs-cli/src/parser.rs`
    - Implement `pub fn parse_snapshot(raw: &[u8]) -> Result<Snapshot, AppError>`
    - Step 1: `serde_json::from_slice` into `serde_json::Value` → `AppError::Parse` on malformed JSON
    - Step 2: check all required top-level fields present → `AppError::Parse` naming the absent field
    - Step 3: float rejection — for each numeric field in `counts` and each value
      in `incident_groups`, check via `serde_json::Number::is_i64() || is_u64()`;
      if neither is true (i.e. it is a float), return `AppError::Schema` with the
      field name. Do NOT rely on raw string scanning.
    - Step 4: assert `authority_classification == "non_authoritative"` → `AppError::Schema`
    - Step 5: assert all three flags (`produces_truth`, `produces_decision`,
      `produces_ranking`) are `false` → `AppError::Schema`
    - Step 6: validate `incident_groups` keys: empty string key → `AppError::Schema`;
      key that parses as a non-negative integer → `AppError::Schema`
    - Step 7: construct and return typed `Snapshot`
    - No `unwrap()`/`expect()` in non-test code
    - _Requirements: 2.1, 2.2, 2.3, 2.4, 2.5, 2.6_

  - [ ] 4.2 Write property test — Property 1: snapshot round-trip
    - `// Feature: obs-cli-consumer, Property 1: snapshot round-trip`
    - Use `arb_snapshot()` strategy; serialize with `to_canonical_json`, parse
      result, assert equality
    - **Property 1: Snapshot round-trip**
    - **Validates: Requirements 2.7, 2.8**

  - [ ]* 4.3 Write property test — Property 2: parser rejects float values
    - `// Feature: obs-cli-consumer, Property 2: parser rejects float values`
    - Inject a float into a counts field; assert `exit_code() == 3`
    - **Property 2: Parser rejects float values**
    - **Validates: Requirements 2.2, 11.4**

  - [ ]* 4.4 Write property test — Property 3: parser enforces epistemic boundary
    - `// Feature: obs-cli-consumer, Property 3: parser enforces epistemic boundary`
    - Replace `authority_classification` with a non-`"non_authoritative"` string;
      assert `exit_code() == 3`
    - **Property 3: Parser enforces epistemic boundary**
    - **Validates: Requirements 2.3, 2.4, 9.4**

  - [ ]* 4.5 Write property test — Property 4: parser rejects missing required fields
    - `// Feature: obs-cli-consumer, Property 4: parser rejects missing required fields`
    - Remove one required field at a time; assert `exit_code() == 3`
    - **Property 4: Parser rejects missing required fields**
    - **Validates: Requirements 2.6**

  - [ ]* 4.6 Write property test — Property 5: parser rejects malformed JSON
    - `// Feature: obs-cli-consumer, Property 5: parser rejects malformed JSON`
    - Use `arb_invalid_json_bytes()` strategy; assert `exit_code() == 3`
    - **Property 5: Parser rejects malformed JSON**
    - **Validates: Requirements 2.5**

  - [ ]* 4.7 Write unit tests for `parser.rs`
    - Valid body → correct `Snapshot`
    - Float in `counts` → `AppError::Schema`
    - `authority_classification: "authoritative"` → `AppError::Schema`
    - `produces_truth: true` → `AppError::Schema`
    - Missing `conflict_count` → `AppError::Parse` naming the field
    - Invalid JSON bytes → `AppError::Parse`
    - _Requirements: 2.1–2.6_

- [x] 5. Checkpoint — parser layer complete
  - Ensure `cargo test -p obs-cli` passes for all parser tests; ask the user if questions arise.

- [x] 6. Implement `formatter.rs` — human-readable output
  - [x] 6.1 Create `userspace/obs-cli/src/formatter.rs`
    - Implement `pub fn format_snapshot(snapshot: &Snapshot) -> String`
      - Header block: `authority`, `produces_truth`, `produces_decision`, `produces_ranking`
      - Counts block: all six fields, right-aligned values
      - Incident groups block: lexicographic order (guaranteed by `BTreeMap`);
        if empty, emit the string literal `"no incidents recorded"` (exact, no
        trailing whitespace — this string is asserted in tests)
    - Define `pub const FORBIDDEN: &[&str]` slice with the eight forbidden words;
      checked in tests
    - No `unwrap()`/`expect()`; no `f32`/`f64`
    - _Requirements: 3.1, 3.2, 3.3, 3.4, 3.5, 3.6, 3.7, 9.1, 9.2, 9.3_

  - [ ]* 6.2 Write unit tests for `formatter.rs`
    - Empty `incident_groups` → output contains `"no incidents recorded"`
    - Non-empty `incident_groups` → keys in lexicographic order
    - Output contains all six count labels and three flag labels
    - _Requirements: 3.1–3.7_

- [x] 7. Wire MVP `main.rs` — file input, no HTTP
  - [x] 7.1 Implement minimal `main.rs`
    - Read snapshot from a file path passed as the first positional argument
      (e.g. `obs-cli snapshot.json`); no HTTP, no flag parsing yet
    - Use `std::fs::read(path)` directly (fetcher.rs added in Phase 2)
    - Call `parse_snapshot` → on error write to stderr and `std::process::exit(code)`
    - Call `format_snapshot` → print to stdout
    - No `unwrap()`/`expect()`
    - Smoke test: `echo '<valid json>' > /tmp/snap.json && cargo run -p obs-cli -- /tmp/snap.json`
    - _Requirements: 3.1_

- [x] 8. Checkpoint — MVP vertical slice complete
  - Ensure `cargo test -p obs-cli` passes; verify `cargo run -p obs-cli -- <file>` prints formatted output; ask the user if questions arise.

### Phase 2 — Full Feature

- [x] 9. Implement `printer.rs` — canonical JSON serializer
  - [x] 9.1 Create `userspace/obs-cli/src/printer.rs`
    - Implement `pub fn to_canonical_json(snapshot: &Snapshot) -> Result<Vec<u8>, AppError>`
    - Use `serde_json::to_vec`; `BTreeMap` in `Snapshot` guarantees key order
    - Map serialization errors to `AppError::Io`
    - No `unwrap()`/`expect()`
    - _Requirements: 2.7, 2.8, 4.1, 4.3_

- [x] 10. Implement `fetcher.rs` — HTTP GET and file I/O
  - [x] 10.1 Add `ureq` to `userspace/obs-cli/Cargo.toml` dependencies (sync feature only, no async)
  - [x] 10.2 Create `userspace/obs-cli/src/fetcher.rs`
    - Implement `pub fn fetch_from_proofd(addr: &str, timeout_ms: u64) -> Result<Vec<u8>, AppError>`
      - URL: `{addr}/diagnostics/summary?display_mode=machine_structured`
      - Single GET with configured timeout via `ureq`
      - HTTP 200 → return body bytes
      - Non-200 → `AppError::Http(status_code, body_string)` (NOT `AppError::Io`)
      - Connection failure / timeout → `AppError::Io`
    - Implement `pub fn read_snapshot_file(path: &Path) -> Result<Vec<u8>, AppError>`
      - `std::fs::read` → `AppError::Io` on failure
    - Implement `pub fn write_snapshot_file(path: &Path, data: &[u8]) -> Result<(), AppError>`
      - `std::fs::write` → `AppError::Io` on failure
    - No `unwrap()`/`expect()`
    - _Requirements: 1.1, 1.2, 1.3, 1.4, 5.2, 5.4_

  - [x]* 10.3 Write property test — Property 12: snapshot save/load round-trip
    - `// Feature: obs-cli-consumer, Property 12: snapshot save/load round-trip`
    - Use `tempfile::tempdir()`; write then read; assert parsed snapshot equals
      original
    - **Property 12: Snapshot save/load round-trip**
    - **Validates: Requirements 5.1, 5.3**

  - [x]* 10.4 Write unit tests for `fetcher.rs`
    - `read_snapshot_file` with non-existent path → `AppError::Io`
    - `write_snapshot_file` to unwritable path → `AppError::Io`
    - Non-200 mock response → `AppError::Http` with correct status code
    - _Requirements: 1.3, 5.2, 5.4_

- [x] 11. Implement `threshold.rs` — condition parsing and evaluation
  - [x] 11.1 Create `userspace/obs-cli/src/threshold.rs`
    - Define `CountField` enum: `PartitionCount`, `TotalNodes`, `TotalIncidents`,
      `AgreementCount`, `ConflictCount`, `IslandCount`
    - Define `CompareOp` enum: `Gt`, `Gte`, `Lt`, `Lte`, `Eq`
    - Define `ThresholdCondition { field: CountField, op: CompareOp, value: usize }`
    - Implement `pub fn resolve_field(s: &str) -> Option<CountField>`
    - Implement `ThresholdCondition::parse(s: &str) -> Result<ThresholdCondition, AppError>`
      - Trim input; resolve operator longest-match first (`>=` before `>`,
        `<=` before `<`, `==` before `=`)
      - Unknown field → `AppError::Usage` with descriptive message listing valid fields
      - Non-integer value → `AppError::Usage`
      - Any other parse failure → `AppError::Usage` with expected-format message
    - Implement `ThresholdCondition::evaluate(&self, snapshot: &Snapshot) -> bool`
    - Implement `pub fn evaluate_all(conditions: &[ThresholdCondition], snapshot: &Snapshot) -> Result<(), AppError>`
      - Collect all violations; return `Ok(())` if none, `Err(AppError::Threshold(violations))` otherwise
    - No `unwrap()`/`expect()`
    - _Requirements: 7.1, 7.2, 7.3, 7.4, 7.5, 7.6, 7.7_

  - [ ]* 11.2 Write property test — Property 13: threshold exit code reflects violation status
    - `// Feature: obs-cli-consumer, Property 13: threshold evaluation exit code reflects violation status`
    - For any snapshot and conditions, verify `evaluate_all` exit code is 0 iff
      no condition is violated
    - **Property 13: Threshold evaluation — exit code reflects violation status**
    - **Validates: Requirements 7.2, 7.3**

  - [ ]* 11.3 Write property test — Property 14: invalid --fail-if → exit code 1
    - `// Feature: obs-cli-consumer, Property 14: invalid --fail-if syntax or unknown field → exit code 1`
    - Use `arb_invalid_condition_string()` strategy; assert `exit_code() == 1`
    - **Property 14: Invalid --fail-if syntax or unknown field → exit code 1**
    - **Validates: Requirements 7.5, 7.6**

  - [ ]* 11.4 Write property test — Property 19: threshold evaluation is deterministic
    - `// Feature: obs-cli-consumer, Property 19: same snapshot + same conditions → identical exit code`
    - Call `evaluate_all` twice with same inputs; assert identical result
    - **Property 19: Same snapshot and same conditions produce identical exit code**
    - **Validates: Requirements 7.2, 7.3, 8.2**

  - [ ]* 11.5 Write unit tests for `threshold.rs`
    - `parse("conflict_count>0")` → correct struct
    - `parse("total_incidents>=5")` → correct struct
    - `parse("conflict_count > 0")` (with spaces) → correct struct (whitespace tolerance)
    - `parse("unknown_field>0")` → `AppError::Usage`
    - `parse("conflict_count>>0")` → `AppError::Usage`
    - `evaluate_all` with no violations → `Ok(())`
    - `evaluate_all` with one violation → `Err(Threshold(...))`
    - _Requirements: 7.1–7.7_

- [x] 12. Implement `cli.rs` — flag parsing
  - [x] 12.1 Create `userspace/obs-cli/src/cli.rs`
    - Define `Flags` struct with all fields as specified in design
    - Implement `Flags::parse(args: &[String]) -> Result<Flags, AppError>`
      - Walk `args` manually (no external arg-parsing crate unless already in workspace)
      - Accept both `--flag value` and `--flag=value` forms
      - `--proofd-addr` and `--snapshot-file` mutually exclusive → `AppError::Usage`
      - `--timeout-ms` must parse as `u64` → `AppError::Usage` on failure
      - `--fail-if` values forwarded to `ThresholdCondition::parse`
      - Unknown flags → `AppError::Usage`
    - No `unwrap()`/`expect()`
    - _Requirements: 1.5, 1.6, 1.7, 5.3, 5.6, 7.1_

  - [ ]* 12.2 Write property test — Property 15: invalid --timeout-ms → exit code 1
    - `// Feature: obs-cli-consumer, Property 15: invalid --timeout-ms value → exit code 1`
    - Use `arb_non_integer_string()` strategy; assert `exit_code() == 1`
    - **Property 15: Invalid --timeout-ms value → exit code 1**
    - **Validates: Requirements 1.7**

  - [ ]* 12.3 Write unit tests for `cli.rs`
    - `--snapshot-file` + `--proofd-addr` both present → `AppError::Usage`
    - `--timeout-ms abc` → `AppError::Usage`
    - `--flag value` and `--flag=value` both parse correctly
    - Valid flags parse to correct `Flags` struct
    - _Requirements: 1.5, 1.6, 1.7, 5.6_

- [x] 13. Checkpoint — fetcher, threshold, cli layer complete
  - Ensure `cargo test -p obs-cli` passes for all tests so far; ask the user if questions arise.

- [x] 14. Implement `diff.rs` — snapshot diff computation
  - [x] 14.1 Create `userspace/obs-cli/src/diff.rs`
    - Implement `pub fn compute_diff(baseline: &Snapshot, current: &Snapshot) -> Diff`
    - All arithmetic: `current_value as i64 - baseline_value as i64`
    - For `incident_groups`: key in both → `Changed` or `Unchanged`; key only in
      current → `Added`; key only in baseline → `Removed`
    - No floats; no `unwrap()`/`expect()`
    - _Requirements: 6.1, 6.2, 6.3, 6.4, 6.5, 6.6, 6.9_

  - [ ]* 14.2 Write property test — Property 10: diff deltas equal current minus baseline
    - `// Feature: obs-cli-consumer, Property 10: diff deltas equal current minus baseline`
    - For all six count fields assert `delta == current - baseline` as `i64`
    - **Property 10: Diff correctness — deltas equal current minus baseline**
    - **Validates: Requirements 6.2, 6.3, 6.4, 6.5**

  - [ ]* 14.3 Write property test — Property 11: self-diff shows all zeros
    - `// Feature: obs-cli-consumer, Property 11: self-diff shows all zeros`
    - Diff snapshot against itself; assert all count deltas zero and all
      incident group entries `Unchanged`
    - **Property 11: Self-diff shows all zeros**
    - **Validates: Requirements 6.9**

  - [ ]* 14.4 Write unit tests for `diff.rs`
    - Identical snapshots → all deltas zero
    - Differing counts → correct signed deltas
    - Added/removed incident group keys → correct `IncidentGroupDelta` variants
    - _Requirements: 6.2, 6.9_

- [x] 15. Wire full `main.rs` — replace MVP stub
  - [x] 15.1 Implement `main.rs` — full wiring
    - Call `Flags::parse(&std::env::args().collect::<Vec<_>>())`; on `Err` write
      message to stderr and `std::process::exit(code)`
    - Determine snapshot source: `--snapshot-file` → `read_snapshot_file` else
      `fetch_from_proofd`
    - Call `parse_snapshot`; on `Err` exit with code 3
    - If `--save-snapshot`: call `to_canonical_json` then `write_snapshot_file`;
      on `Err` exit with code 2
    - If `--diff`: load baseline with `read_snapshot_file` + `parse_snapshot`;
      call `compute_diff`; call `format_diff`; print to stdout
    - Else if `--json`: call `to_canonical_json`; write bytes to stdout
    - Else: call `format_snapshot`; print to stdout
    - Call `evaluate_all` for any `--fail-if` conditions; on `Err` write each
      violation to stderr and exit with code 4
    - All error paths: write to stderr only, never stdout; call
      `std::process::exit(code)`
    - No `unwrap()`/`expect()`
    - _Requirements: 1.1–1.7, 2.1–2.8, 3.1–3.7, 4.1–4.4, 5.1–5.6, 6.1–6.9,
      7.1–7.8, 8.1–8.5, 9.1–9.4, 10.1–10.4, 11.1–11.7_

  - [ ]* 15.2 Write property test — Property 17: error messages go to stderr not stdout
    - `// Feature: obs-cli-consumer, Property 17: error messages go to stderr not stdout`
    - For each `AppError` variant, assert `message()` is non-empty and that the
      error path never writes to stdout (unit-level: verify `AppError::message()`
      is non-empty for all variants)
    - **Property 17: Error messages go to stderr, not stdout**
    - **Validates: Requirements 10.1, 10.2, 10.4**

  - [ ]* 15.3 Write property test — Property 16: non-200 HTTP response → exit code 2
    - `// Feature: obs-cli-consumer, Property 16: non-200 HTTP response → exit code 2`
    - Use a mock HTTP server (e.g. `mockito` or `wiremock`) returning arbitrary
      non-200 status; assert `fetch_from_proofd` returns `AppError::Http` with
      `exit_code() == 2`
    - **Property 16: Non-200 HTTP response → exit code 2**
    - **Validates: Requirements 1.3**

- [x] 16. Final checkpoint — full integration
  - Ensure `cargo test -p obs-cli` passes for all tests; ensure `cargo clippy -p obs-cli -- -D warnings` passes; ask the user if questions arise.

## Notes

- Tasks marked with `*` are optional and can be skipped for a faster MVP
- Each task references specific requirements for traceability
- Checkpoints ensure incremental validation at natural layer boundaries
- Property tests validate universal correctness across all valid and invalid inputs
- Unit tests validate specific examples, edge cases, and error conditions
- `BTreeMap<String, usize>` for `incident_groups` provides lexicographic ordering as a type-level guarantee — no runtime sort needed
- All numeric types are `usize` (counts) or `i64` (deltas); no `f32`/`f64` anywhere
- No `unwrap()`/`expect()` in non-test code — all errors propagate via `Result` and map to `AppError`
