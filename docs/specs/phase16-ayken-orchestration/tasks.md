# Phase-16A Implementation Tasks

**Status:** Planned
**Scope:** Minimal deterministic query pipeline

## Tasks

- [ ] 1. Freeze the Phase-16A scope
  - [ ] 1.1 Mark the supported production command surface as `list`, `show`, and `query ... where ...`
  - [ ] 1.2 Mark mutation, AI, UI, debug, system, loop, and control-flow widening as explicitly unsupported in the first slice
  - [x] 1.3 Add source-level guards or tests proving unsupported commands fail closed instead of degrading to placeholder behavior
  - [x] 1.4 Rename implementation surfaces away from phase labels and toward purpose-based names
  - Reference: Requirements 1, 1A, 7

- [ ] 2. Freeze the canonical IR contract
  - [ ] 2.1 Ratify the existing execution-plan subset as the only Phase-16A canonical IR
  - [x] 2.2 Freeze the `r0 = active context/result register` invariant
  - [x] 2.3 Ensure every successful canonical plan terminates in `Return`
  - [ ] 2.4 Add validation rejecting hidden state or direct `DSL -> BCIB` production shortcuts
  - Reference: Requirements 2, 3

- [ ] 3. Implement `DSL -> Canonical IR` for the supported surface
  - [x] 3.1 Lower `list <context>` to `LoadContext -> Return`
  - [x] 3.2 Lower `show <context> <id>` to `LoadContext -> LoadLiteral/Compare -> ApplyFilter -> Return`
  - [x] 3.3 Lower `query <context> where <predicate>` to `LoadContext -> predicate lowering -> ApplyFilter -> Return`
  - [x] 3.4 Reject unsupported predicate forms with explicit lowering errors
  - Reference: Requirements 1, 2, 7

- [ ] 4. Add canonical IR validation
  - [x] 4.1 Reject empty contexts
  - [x] 4.2 Reject missing or invalid terminal `Return`
  - [ ] 4.3 Reject IR nodes outside the frozen Phase-16A subset
  - [x] 4.4 Emit deterministic plan identity/fingerprint for proof/replay binding
  - [x] 4.5 Emit a deterministic Canonical Query Binding for `list`, `show`, and `query`
  - Reference: Requirements 2, 5, 9, 10

- [ ] 5. Implement NOP-free `Canonical IR -> BCIB` lowering
  - [x] 5.1 Emit only `DataQuery`, `End`, and optional `TraceEmit`
  - [x] 5.2 Reject any lowering path that would require `Nop`
  - [x] 5.3 Reject any lowering path that would require `DataCreate`, `DataAdd`, `UiRender`, or `AiAsk`
  - [x] 5.4 Bind operands deterministically so the same canonical plan yields the same lowered BCIB
  - [x] 5.5 Preserve the semantic distinction between `list`, `show`, and `query` in lowering metadata
  - Reference: Requirements 3, 4, 5, 9, 10

- [x] 6. Replace placeholder orchestration/submission surfaces
  - [x] 6.1 Replace the `userspace/orchestration` placeholder with a real submit-only router
  - [x] 6.2 Replace placeholder submission-bridge behavior with a real submit adapter
  - [x] 6.3 Implement non-empty, fail-closed capability/submission validation
  - [x] 6.4 Keep direct kernel-facing submit calls out of semantic parsing/lowering layers
  - [x] 6.5 Enforce explicit, non-empty context-read capability validation for the supported query surface
  - Reference: Requirements 6, 7, 8, 9

- [x] 7. Close the Phase-16A proof/replay slice
  - [x] 7.1 Produce one canonical end-to-end artifact chain: command string -> IR fingerprint -> Canonical Query Binding fingerprint -> BCIB SHA-256 -> submission result
  - [x] 7.2 Bind proof material to the lowered BCIB identity
  - [x] 7.3 Add replay verification for the same canonical BCIB input
  - [x] 7.4 Fail closed on replay deviation
  - Reference: Requirement 10

- [x] 8. Add regression guards
  - [x] 8.1 Add tests proving the production path never emits `Nop`
  - [x] 8.2 Add tests proving unsupported commands/predicates never degrade to placeholder success
  - [x] 8.3 Add tests proving the production path does not bypass orchestration with direct executor calls
  - [x] 8.4 Add one end-to-end deterministic query scenario for `list`, `show`, and `query`
  - [x] 8.5 Add tests proving orchestration rejects or packages work without reinterpreting semantic intent
  - Reference: Requirements 3, 4, 6, 7, 8, 9, 10

- [ ] 9. Keep docs synchronized while implementation lands
  - [ ] 9.1 Update `README.md` or current truth surfaces only when the production path actually changes
  - [x] 9.2 Update this checklist in the same change set as code
  - [ ] 9.3 Record completed slices and validation in a follow-up progress note when implementation starts
  - [ ] 9.4 Keep code-facing names stable if the phase/spec label changes later
  - Reference: All requirements
