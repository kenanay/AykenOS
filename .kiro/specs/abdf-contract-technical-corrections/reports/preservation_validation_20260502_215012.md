# Preservation Validation Report

**Generated**: 2026-05-02 21:50:12 UTC
**Script**: validate_preservation.py
**Version**: 2.0.0 (Python - CI-Authoritative)

---

## Input Files

- **ORIGINAL**: `/Users/asel/Desktop/AykenOS/.kiro/specs/abdf-contract-technical-corrections/test_fixtures/fail_unexpected_change/original.md`
  - Hash: 8fc3c497303a134e756a9b9eb35258e727e279c41a368c369a60fb3ac9a986fa
- **FIXED**: `/Users/asel/Desktop/AykenOS/.kiro/specs/abdf-contract-technical-corrections/test_fixtures/fail_unexpected_change/fixed.md`
  - Hash: 3e67e72576d9f8241c14aa2189b035f693133b415f0837605b2b4b313c7d1183
- **Diff**: `/Users/asel/Desktop/AykenOS/.kiro/specs/abdf-contract-technical-corrections/test_fixtures/../reports/diff_20260502_215012.patch`

---

## Validation Results

❌ **FAIL**: Preservation validation failed

### Changed Sections (2)

- `ROOT > # Test Document - Unexpected Change > ## 🧵 String Pool Section`
- `ROOT > # Test Document - Unexpected Change > ## 📐 Binary Layout (Preserved)`

### Expected Changes (1)

- ✅ `🧵 String Pool Section` [id: `string_pool_section`]

### Unexpected Changes (1)

- ❌ ID: `binary_layout_preserved`

---

**Validation Level**: Level 3 (Complete Audit Trail)
**Authority**: Constitutional Enforcement (Phase-17.5)
**CI-Authoritative**: ✅ YES (deterministic diff→section mapping)

