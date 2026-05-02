# Preservation Validation Report

**Generated**: 2026-05-02 21:50:13 UTC
**Script**: validate_preservation.py
**Version**: 2.0.0 (Python - CI-Authoritative)

---

## Input Files

- **ORIGINAL**: `/Users/asel/Desktop/AykenOS/.kiro/specs/abdf-contract-technical-corrections/test_fixtures/fail_missing_expected/original.md`
  - Hash: afa3b2c82dc8f4827a958c814a45829bfcc646880901f5ee3d97aec0e09ae4ad
- **FIXED**: `/Users/asel/Desktop/AykenOS/.kiro/specs/abdf-contract-technical-corrections/test_fixtures/fail_missing_expected/fixed.md`
  - Hash: 34a070791e9391cc1f15bdb74c3df42f1c6505d20f678c92907b6c591df84ae5
- **Diff**: `/Users/asel/Desktop/AykenOS/.kiro/specs/abdf-contract-technical-corrections/test_fixtures/../reports/diff_20260502_215013.patch`

---

## Validation Results

❌ **FAIL**: Preservation validation failed

### Changed Sections (1)

- `ROOT > # Test Document - Missing Expected > ## 🧵 String Pool Section`

### Expected Changes (2)

- ✅ `🧵 String Pool Section` [id: `string_pool_section`]
- ❌ `🔒 Header Structure` [id: `header_structure`] (NOT FOUND)

### Missing Expected Changes (1)

- ❌ ID: `header_structure`

---

**Validation Level**: Level 3 (Complete Audit Trail)
**Authority**: Constitutional Enforcement (Phase-17.5)
**CI-Authoritative**: ✅ YES (deterministic diff→section mapping)

