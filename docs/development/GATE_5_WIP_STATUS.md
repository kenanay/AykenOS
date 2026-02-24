# Gate-5 Work In Progress Status

**Date:** 2026-02-23  
**Branch:** feature/gate-5-constitutional-lock  
**Status:** IN PROGRESS  
**Phase:** 4.6 (planned)

## Objective

Establish constitutional runtime contract lock to make runtime sözleşmeleri immutable.

## Scope (Minimal Surface)

### 1. Mailbox ABI Freeze
- Semantic layout freeze (not binary hash)
- `sizeof`, `offsetof`, field order, field types
- Baseline: `constitution/abi_mailbox.json`

### 2. Marker Contract Lock
- Runtime markers registry
- Baseline: `constitution/runtime_markers.json`
- Current markers:
  - `[[AYKEN_SCHED_MB_ACCEPT]]`
  - `[[R3OK]]`

### 3. Constitutional Versioning
- Version: `constitution/version.json`
- Current: 1.0.0
- Bump rules:
  - ABI layout change → MAJOR++
  - Marker add/remove → MINOR++
  - Gate surface change → MINOR++

## Implementation Progress

### ✅ Completed

1. **Constitution Directory Structure**
   - `constitution/version.json` (1.0.0 baseline)
   - `constitution/abi_mailbox.json` (mailbox struct layout)
   - `constitution/runtime_markers.json` (marker registry)

2. **ABI Verification Tool**
   - `tools/dump_abi_layout.c` (compile-time layout dumper)

### 🚧 In Progress

1. **Gate Script**
   - `scripts/ci/gate_5_constitutional_lock.sh`
   - ABI diff detection
   - Marker surface scanning
   - Version bump validation

2. **CI Integration**
   - Makefile target: `ci-gate-constitutional-lock`
   - Wire into `ci-freeze` chain
   - Evidence directory structure

3. **Documentation**
   - Update steering docs
   - Update product status
   - Create RFC template

### ⏳ Pending

1. **Testing**
   - Local smoke test
   - CI freeze validation
   - Negative test cases

2. **PR Preparation**
   - Commit cleanup
   - Evidence generation
   - Review checklist

## Design Decisions

### Why Minimal Surface?

- **Syscall IDs**: Already covered by `ci-gate-abi`
- **Sched IDs**: Runtime detail, not contract
- **Constitution**: Contract surface only

### Why Semantic (Not Binary Hash)?

- Compiler/toolchain independence
- Padding/alignment transparency
- Human-readable diffs

### Why Separate Markers?

- Runtime behavior markers ≠ ABI
- Different change velocity
- Independent versioning

## Files Structure

```
constitution/
├── version.json           # 1.0.0
├── abi_mailbox.json       # Semantic layout
└── runtime_markers.json   # Marker registry

tools/
└── dump_abi_layout.c      # ABI verification

scripts/ci/
└── gate_5_constitutional_lock.sh  # (pending)
```

## Verification Plan

### ABI Freeze Test
1. Modify mailbox struct
2. Run gate → should FAIL
3. Bump version → should PASS

### Marker Lock Test
1. Add new marker
2. Run gate → should FAIL
3. Update registry + bump version → should PASS

### Version Enforcement Test
1. Change ABI without version bump → FAIL
2. Change marker without version bump → FAIL

## Integration Points

### Makefile
```makefile
ci-gate-constitutional-lock:
	@bash scripts/ci/gate_5_constitutional_lock.sh

ci-freeze: ... ci-gate-constitutional-lock ...
```

### Evidence
```
evidence/run-<RUN_ID>/
└── gates/
    └── constitutional-lock/
        ├── abi_diff.json
        ├── marker_diff.json
        ├── version_check.json
        └── report.json
```

## Next Actions

1. Complete gate script implementation
2. Add Makefile integration
3. Test locally
4. Update documentation
5. Generate evidence
6. Open PR

## References

- Gate-4 completion: `docs/development/GATE_4_COMPLETION_REPORT.md`
- Constitution baseline: `constitution/`
- ABI tool: `tools/dump_abi_layout.c`

---

**Maintained by:** AykenOS Architecture Board  
**Last Updated:** 2026-02-23  
**Status:** WIP - Foundation complete, gate script pending
