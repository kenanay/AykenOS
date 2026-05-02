# Requirements Document

## Introduction

`ayken obs` is a CLI consumer for the AykenOS observability system. It fetches the
`machine_structured` projection from proofd's `/diagnostics/summary` endpoint and
formats the data for human consumption, diff comparison, and CI integration.

The tool is a pure formatter. It holds no state, produces no authority, and makes no
decisions. proofd is the single source of truth; `ayken obs` is a read-only display
layer on top of it. All output is deterministic: identical input always produces
identical output. No floating-point values appear anywhere in the tool — neither in
internal logic nor in output.

Forbidden output vocabulary: "best node", "recommended path", "optimal run",
"trust score", "ranking", "recommendation", "decision".
Allowed output vocabulary: counts, flags, incident type labels, diff deltas.

---

## Glossary

- **CLI**: The `ayken obs` command-line tool being specified here.
- **proofd**: The read-only AykenOS diagnostics HTTP service. Exposes
  `GET /diagnostics/summary?display_mode=machine_structured`. Not an authority.
- **machine_structured response**: The typed JSON body returned by proofd when
  `display_mode=machine_structured` is requested. Contains `counts`, `flags`, and
  `incident_groups` — all integers and booleans, no floats, no narrative.
- **Snapshot**: A single fetched-and-parsed `machine_structured` response, held
  in memory or serialized to a file for diff comparison.
- **Diff**: A field-by-field comparison of two Snapshots showing which integer
  counts changed and by how much (delta as a signed integer).
- **Exit code**: The integer process exit code returned by the CLI to the calling
  shell or CI system.
- **Formatter**: The component that converts a Snapshot into human-readable text.
- **Fetcher**: The component that issues the HTTP GET request to proofd and returns
  the raw response body.
- **Parser**: The component that deserializes the raw JSON response body into a
  typed Snapshot struct.
- **Printer**: The component that serializes a Snapshot back to canonical JSON
  (used for snapshot files and round-trip verification).
- **CI mode**: An invocation mode where the CLI applies user-supplied threshold
  conditions to a Snapshot and exits with a non-zero code when any condition is
  violated.
- **Threshold condition**: A user-supplied rule of the form `<field> <op> <value>`
  where `<field>` is a count field name, `<op>` is `>`, `>=`, `<`, `<=`, or `==`,
  and `<value>` is a non-negative integer.

---

## Requirements

### Requirement 1: Fetch the machine_structured projection

**User Story:** As a developer, I want the CLI to fetch the machine_structured
projection from proofd, so that I can inspect the current observability state
without writing curl commands.

#### Acceptance Criteria

1. WHEN the CLI is invoked without a `--snapshot-file` flag, THE Fetcher SHALL
   issue a single `GET /diagnostics/summary?display_mode=machine_structured`
   request to the configured proofd address.
2. WHEN proofd returns HTTP 200, THE Fetcher SHALL pass the response body to the
   Parser.
3. WHEN proofd returns a non-200 HTTP status code, THE CLI SHALL print the status
   code and the response body to stderr and exit with code 2.
4. WHEN the TCP connection to proofd cannot be established within the configured
   timeout, THE CLI SHALL print a connection-failure message to stderr and exit
   with code 2.
5. THE CLI SHALL accept a `--proofd-addr` flag (default: `http://127.0.0.1:7777`)
   that sets the base URL for all proofd requests.
6. THE CLI SHALL accept a `--timeout-ms` flag (default: `5000`) that sets the
   HTTP request timeout in milliseconds as a non-negative integer.
7. IF the `--timeout-ms` value cannot be parsed as a non-negative integer, THEN
   THE CLI SHALL print a usage error to stderr and exit with code 1.

---

### Requirement 2: Parse the machine_structured response

**User Story:** As a developer, I want the CLI to parse the JSON response into a
typed structure, so that downstream formatting and diffing operate on verified data.

#### Acceptance Criteria

1. WHEN the Parser receives a valid `machine_structured` JSON body, THE Parser
   SHALL deserialize it into a typed Snapshot containing: `counts`
   (`partition_count`, `total_nodes`, `total_incidents`, `agreement_count`,
   `conflict_count`, `island_count` — all non-negative integers), `flags`
   (`produces_truth`, `produces_decision`, `produces_ranking` — all booleans),
   and `incident_groups` (a map of string keys to non-negative integer values).
2. WHEN the Parser encounters a field whose value is a floating-point number,
   THE Parser SHALL reject the response, print a schema-violation error to stderr,
   and exit with code 3.
3. WHEN the Parser encounters a response body where `authority_classification` is
   not `"non_authoritative"`, THE Parser SHALL reject the response, print a
   schema-violation error to stderr, and exit with code 3.
4. WHEN the Parser encounters a response body where any of `produces_truth`,
   `produces_decision`, or `produces_ranking` is `true`, THE Parser SHALL reject
   the response, print a schema-violation error to stderr, and exit with code 3.
5. WHEN the Parser encounters a malformed JSON body (not valid JSON), THE Parser
   SHALL print a parse-error message to stderr and exit with code 3.
6. WHEN the Parser encounters a JSON body that is missing any required field,
   THE Parser SHALL print a missing-field error naming the absent field to stderr
   and exit with code 3.
7. THE Printer SHALL serialize a Snapshot back to canonical JSON.
8. FOR ALL valid Snapshots, parsing the Printer output SHALL produce a Snapshot
   equal to the original (round-trip property).

---

### Requirement 3: Display the snapshot in human-readable format

**User Story:** As a developer, I want the CLI to display counts and flags in a
readable table, so that I can understand the current observability state at a glance.

#### Acceptance Criteria

1. WHEN a Snapshot is successfully parsed, THE Formatter SHALL write a formatted
   summary to stdout containing all six count fields and their integer values.
2. THE Formatter SHALL display each count field on its own line in the format
   `<label>: <value>` where `<label>` is a fixed human-readable label and
   `<value>` is the integer count.
3. THE Formatter SHALL display the three flag fields (`produces_truth`,
   `produces_decision`, `produces_ranking`) and their boolean values.
4. THE Formatter SHALL display each `incident_groups` entry as
   `<incident_type>: <count>` sorted lexicographically by incident type key.
5. IF `incident_groups` is empty, THEN THE Formatter SHALL display the text
   `no incidents recorded` in the incident groups section.
6. THE Formatter SHALL NOT use the words "best", "recommended", "optimal",
   "ranking", "decision", "trust", or "score" anywhere in its output.
7. WHEN the same Snapshot is formatted twice, THE Formatter SHALL produce
   byte-identical output both times (determinism invariant).

---

### Requirement 4: JSON output mode

**User Story:** As a developer, I want a `--json` flag that emits the raw parsed
snapshot as canonical JSON, so that I can pipe the output to other tools.

#### Acceptance Criteria

1. WHEN the `--json` flag is present, THE CLI SHALL write the Snapshot as
   canonical JSON to stdout instead of the human-readable format.
2. WHEN the `--json` flag is present, THE CLI SHALL NOT write any human-readable
   text to stdout.
3. THE JSON output SHALL be the Printer output — identical to what the Printer
   produces for round-trip verification.
4. WHEN the same Snapshot is emitted with `--json` twice, THE CLI SHALL produce
   byte-identical output both times (determinism invariant).

---

### Requirement 5: Save and load snapshots

**User Story:** As a developer, I want to save a snapshot to a file and reload it
later, so that I can compare two runs without keeping proofd running.

#### Acceptance Criteria

1. THE CLI SHALL accept a `--save-snapshot <path>` flag that writes the Printer
   output of the current Snapshot to the specified file path after display.
2. WHEN `--save-snapshot` is used and the file cannot be written, THE CLI SHALL
   print a write-error message to stderr and exit with code 2.
3. THE CLI SHALL accept a `--snapshot-file <path>` flag that reads a previously
   saved snapshot file instead of fetching from proofd.
4. WHEN `--snapshot-file` is used and the file cannot be read, THE CLI SHALL
   print a read-error message to stderr and exit with code 2.
5. WHEN `--snapshot-file` is used and the file content fails Parser validation,
   THE CLI SHALL apply the same exit codes as Requirement 2 (exit code 3).
6. WHEN both `--snapshot-file` and `--proofd-addr` are supplied simultaneously,
   THE CLI SHALL print a usage error to stderr and exit with code 1.

---

### Requirement 6: Diff mode

**User Story:** As a developer, I want to compare two snapshots and see which
counts changed, so that I can detect regressions between runs.

#### Acceptance Criteria

1. THE CLI SHALL accept a `--diff <path>` flag that loads a baseline Snapshot
   from the specified file and compares it to the current Snapshot (fetched or
   loaded via `--snapshot-file`).
2. WHEN `--diff` is used, THE Formatter SHALL display a diff table showing, for
   each count field: the baseline value, the current value, and the signed integer
   delta (`current - baseline`).
3. WHEN a count field is unchanged (delta is zero), THE Formatter SHALL display
   the delta as `0` without any sign prefix.
4. WHEN a count field increased (delta is positive), THE Formatter SHALL display
   the delta prefixed with `+`.
5. WHEN a count field decreased (delta is negative), THE Formatter SHALL display
   the delta prefixed with `-`.
6. WHEN `--diff` is used, THE Formatter SHALL also display a diff of
   `incident_groups`, showing added keys, removed keys, and changed counts.
7. WHEN `--diff` is used and the baseline file cannot be read or fails validation,
   THE CLI SHALL print an error to stderr and exit with code 2.
8. WHEN the same two Snapshots are diffed twice, THE Formatter SHALL produce
   byte-identical diff output both times (determinism invariant).
9. WHEN a Snapshot is diffed against itself, THE Formatter SHALL show all deltas
   as zero and all `incident_groups` entries as unchanged.

---

### Requirement 7: CI mode — exit codes based on threshold conditions

**User Story:** As a CI engineer, I want the CLI to exit with a non-zero code when
specific count thresholds are exceeded, so that I can fail a pipeline when
observability conditions are violated.

#### Acceptance Criteria

1. THE CLI SHALL accept one or more `--fail-if <condition>` flags where
   `<condition>` is a string of the form `<field><op><value>` (e.g.
   `conflict_count>0`, `total_incidents>=5`).
2. WHEN all `--fail-if` conditions are evaluated and none are true, THE CLI SHALL
   exit with code 0.
3. WHEN one or more `--fail-if` conditions evaluate to true, THE CLI SHALL print
   each violated condition to stderr and exit with code 4.
4. THE CLI SHALL evaluate conditions only against the six count fields:
   `partition_count`, `total_nodes`, `total_incidents`, `agreement_count`,
   `conflict_count`, `island_count`.
5. IF a `--fail-if` condition references a field name that is not one of the six
   count fields, THEN THE CLI SHALL print a usage error to stderr and exit with
   code 1.
6. IF a `--fail-if` condition cannot be parsed (invalid operator or non-integer
   value), THEN THE CLI SHALL print a usage error to stderr and exit with code 1.
7. WHEN multiple `--fail-if` flags are supplied, THE CLI SHALL evaluate all
   conditions and report all violations before exiting.
8. THE CLI SHALL NOT use `--fail-if` evaluation results to produce any
   recommendation, ranking, or decision text in its output.

---

### Requirement 8: Determinism

**User Story:** As a CI engineer, I want the CLI output to be deterministic, so
that identical inputs always produce identical outputs and diffs are reproducible.

#### Acceptance Criteria

1. THE CLI SHALL NOT read system time, random number generators, or any
   non-deterministic source during formatting or diff computation.
2. WHEN the CLI is invoked twice with the same Snapshot input and the same flags,
   THE CLI SHALL produce byte-identical stdout output both times.
3. THE CLI SHALL NOT embed timestamps, run IDs, or any session-specific data in
   its stdout output.
4. THE Formatter SHALL sort `incident_groups` entries lexicographically by key
   before rendering, regardless of the order in which they appear in the JSON.
5. THE Formatter SHALL sort diff entries for `incident_groups` lexicographically
   by key.

---

### Requirement 9: Epistemic boundary enforcement

**User Story:** As an architect, I want the CLI to enforce the epistemic boundary
of proofd, so that no authority, ranking, or decision language ever appears in
CLI output.

#### Acceptance Criteria

1. THE CLI SHALL NOT produce any output containing the words "best", "worst",
   "recommended", "optimal", "trust score", "ranking", "decision", or
   "recommendation".
2. THE CLI SHALL display the `authority_classification` field value from the
   Snapshot verbatim in its output header, so that consumers can confirm they are
   reading a non-authoritative source.
3. THE CLI SHALL display the `produces_truth`, `produces_decision`, and
   `produces_ranking` flag values verbatim so that consumers can confirm all
   three are `false`.
4. IF the CLI detects that any of the three flags is `true` in the parsed
   Snapshot, THEN THE CLI SHALL reject the response per Requirement 2.4 (exit
   code 3) rather than displaying it.

---

### Requirement 10: Error reporting

**User Story:** As a developer, I want clear, actionable error messages, so that
I can diagnose failures quickly.

#### Acceptance Criteria

1. THE CLI SHALL write all error messages to stderr, never to stdout.
2. WHEN the CLI exits with a non-zero code, THE CLI SHALL print at least one
   error message to stderr before exiting.
3. THE CLI SHALL use the following exit code convention:
   - `0` — success, no threshold violations
   - `1` — usage error (bad flags, unknown field in condition, etc.)
   - `2` — I/O error (connection failure, file read/write error, non-200 HTTP)
   - `3` — schema/parse error (invalid JSON, missing field, float detected,
     authority violation)
   - `4` — threshold violation (one or more `--fail-if` conditions are true)
4. THE CLI SHALL NOT mix stdout and stderr output for the same logical message.

---

### Requirement 11: Crate placement and build integration

**User Story:** As a developer, I want the CLI to live in the existing userspace
workspace, so that it builds with the same toolchain and CI gates as the rest of
the project.

#### Acceptance Criteria

1. THE CLI SHALL be implemented as a new crate named `obs-cli` located at
   `userspace/obs-cli/`.
2. THE `obs-cli` crate SHALL be added to the `members` list in
   `userspace/Cargo.toml`.
3. THE `obs-cli` crate SHALL NOT depend on any kernel crates, Ring0 interfaces,
   or capability system crates.
4. THE `obs-cli` crate SHALL NOT use `f32` or `f64` anywhere in its source.
5. THE `obs-cli` crate SHALL NOT use `Box::leak`, `mem::forget`, or any
   intentional memory leak pattern.
6. THE `obs-cli` crate SHALL NOT use `unwrap()` or `expect()` on `Result` or
   `Option` values in non-test code.
7. WHERE the build profile is P4.4 or higher, THE `obs-cli` crate SHALL compile
   without errors under the Phase Matrix rules defined in `_ayken/steering/PHASES.md`.
