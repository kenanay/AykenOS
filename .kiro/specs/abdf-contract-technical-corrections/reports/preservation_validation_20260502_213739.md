# Preservation Validation Report

**Generated**: 2026-05-02 21:37:39 UTC
**Script**: validate_preservation.py
**Version**: 2.0.0 (Python - CI-Authoritative)

---

## Input Files

- **ORIGINAL**: `test_fixtures/pass_only_expected_changes/original.md`
  - Hash: e18b589dbdffb74a594acc2b629d1c3518c379b34761233fe538e5588fa0d66e
- **FIXED**: `test_fixtures/pass_only_expected_changes/fixed.md`
  - Hash: d924707b8eaafeb32eb9390de8708670c7d8aac1c25285e560bfa0e6ee2f49df
- **Diff**: `/Users/asel/Desktop/AykenOS/.kiro/specs/abdf-contract-technical-corrections/reports/diff_20260502_213739.patch`

---

## Validation Results

❌ **FAIL**: Preservation validation failed

### Changed Sections (4)

- `## 📐 Binary Layout (Preserved)`
- `## 🧵 String Pool Section`
- `## 🔒 Header Structure`
- `# Test Document - Fixed`

### Expected Changes (2)

- ✅ `🧵 String Pool Section`
- ✅ `🔒 Header Structure`

### Unexpected Changes (1)

- ❌ `binary layout preserved`

---

**Validation Level**: Level 3 (Complete Audit Trail)
**Authority**: Constitutional Enforcement (Phase-17.5)
**CI-Authoritative**: ✅ YES (deterministic diff→section mapping)

