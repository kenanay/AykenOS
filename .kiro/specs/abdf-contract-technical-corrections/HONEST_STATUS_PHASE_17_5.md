# Honest Status: Phase-17.5 Validation Infrastructure

**Date**: 2026-05-02  
**Status**: 🟡 ALPHA - Not CI-Authoritative

---

## What Was Delivered

### ✅ validate_preservation.sh (ALPHA)

**What it does**:
- Generates unified diff (ORIGINAL → FIXED)
- Parses expected changes from YAML
- Checks for unexpected changes in preserved sections
- Generates evidence reports
- CI-compatible exit codes

**What it does well**:
- Basic diff-based validation
- Whitelist checking
- Evidence generation
- Report formatting

**What it does NOT do well** (CRITICAL LIMITATIONS):

1. **Line Count Calculation** ❌
   ```bash
   # BROKEN:
   grep -c "^+" "$DIFF_FILE" | grep -v "^+++"
   # grep -c outputs a number, second grep doesn't filter diff lines
   
   # FIXED:
   grep "^+" "$DIFF_FILE" | grep -v "^+++" | wc -l
   ```

2. **YAML Parsing** ❌
   - Regex-based (fragile to format changes)
   - No validation of YAML structure
   - Will break on complex YAML
   - **Need**: Python parser or `yq` tool

3. **Section Matching** ❌
   - Heuristic: searches for section name in diff
   - Misses changes if section name only in context
   - No diff hunk → section line range mapping
   - **Need**: Parse `@@ -X,Y +A,B @@` and map to document structure

4. **False Positive Risk** ❌
   - Preserved section mentioned in diff context → flagged as changed
   - No distinction between changed lines vs context lines
   - **Need**: Parse diff hunks, check only +/- lines

5. **False Negative Risk** ❌
   - Section content changed but section name not in diff → missed
   - Emoji normalization may miss matches
   - **Need**: Line range → section mapping

---

## CI-Authoritative Status: ❌ NOT YET

**Why not CI-authoritative**:
- No fixture-based validation (can't prove PASS/FAIL behavior)
- No false positive/negative testing
- Heuristic section matching (unreliable)
- Regex YAML parsing (fragile)
- Line count calculation bug (fixed but not tested)

**What "CI-authoritative" means**:
- Script FAIL blocks merge (no manual override)
- Script PASS is trusted evidence
- False positive rate < 1%
- False negative rate = 0%
- Fixture tests prove correctness

**Current status**:
- Script FAIL should be investigated, not blocking
- Script PASS is suggestive, not proof
- Manual verification still required
- **Use for**: Development feedback, not enforcement

---

## Hardening Required (Phase 1.5)

### 1. Fixture-Based Validation

**Purpose**: Prove script correctness

**Test Cases**:
```
test_fixtures/
├── pass_cases/
│   ├── only_expected_changes/
│   │   ├── ORIGINAL.md
│   │   ├── FIXED.md
│   │   ├── expected_changes.yml
│   │   └── expected_result: PASS
│   └── no_changes/
│       ├── ORIGINAL.md
│       ├── FIXED.md (identical)
│       ├── expected_changes.yml
│       └── expected_result: PASS
└── fail_cases/
    ├── unexpected_change/
    │   ├── ORIGINAL.md
    │   ├── FIXED.md (has unexpected change)
    │   ├── expected_changes.yml
    │   └── expected_result: FAIL
    ├── missing_expected_change/
    │   ├── ORIGINAL.md
    │   ├── FIXED.md (missing expected change)
    │   ├── expected_changes.yml
    │   └── expected_result: FAIL
    └── false_positive_test/
        ├── ORIGINAL.md
        ├── FIXED.md (preserved section in diff context only)
        ├── expected_changes.yml
        └── expected_result: PASS (should NOT flag context)
```

**Test Runner**:
```bash
#!/usr/bin/env bash
# test_validate_preservation.sh

for test_case in test_fixtures/*/; do
    ORIGINAL="$test_case/ORIGINAL.md"
    FIXED="$test_case/FIXED.md"
    EXPECTED_CHANGES="$test_case/expected_changes.yml"
    EXPECTED_RESULT=$(cat "$test_case/expected_result")
    
    ./validate_preservation.sh "$ORIGINAL" "$FIXED" "$EXPECTED_CHANGES"
    ACTUAL_RESULT=$?
    
    if [[ "$EXPECTED_RESULT" == "PASS" && $ACTUAL_RESULT -eq 0 ]]; then
        echo "✅ $test_case"
    elif [[ "$EXPECTED_RESULT" == "FAIL" && $ACTUAL_RESULT -eq 1 ]]; then
        echo "✅ $test_case"
    else
        echo "❌ $test_case (expected $EXPECTED_RESULT, got exit code $ACTUAL_RESULT)"
        exit 1
    fi
done

echo "✅ All fixture tests passed"
```

### 2. Diff Hunk → Section Resolver

**Purpose**: Accurate section change detection

**Algorithm**:
```python
# parse_diff_sections.py

import re

def parse_diff_hunks(diff_file):
    """Parse diff hunks and extract changed line ranges."""
    hunks = []
    with open(diff_file) as f:
        for line in f:
            # Match: @@ -10,5 +12,7 @@ Section Name
            match = re.match(r'^@@ -(\d+),(\d+) \+(\d+),(\d+) @@(.*)$', line)
            if match:
                old_start, old_count, new_start, new_count, context = match.groups()
                hunks.append({
                    'old_range': (int(old_start), int(old_count)),
                    'new_range': (int(new_start), int(new_count)),
                    'context': context.strip()
                })
    return hunks

def map_lines_to_sections(document_file):
    """Build line number → section name mapping."""
    sections = {}
    current_section = None
    with open(document_file) as f:
        for line_num, line in enumerate(f, start=1):
            # Detect section headers (markdown)
            if line.startswith('#'):
                current_section = line.strip()
            if current_section:
                sections[line_num] = current_section
    return sections

def find_changed_sections(diff_file, document_file):
    """Determine which sections were actually changed."""
    hunks = parse_diff_hunks(diff_file)
    sections = map_lines_to_sections(document_file)
    
    changed_sections = set()
    for hunk in hunks:
        new_start, new_count = hunk['new_range']
        for line_num in range(new_start, new_start + new_count):
            if line_num in sections:
                changed_sections.add(sections[line_num])
    
    return changed_sections
```

### 3. Robust YAML Parsing

**Option A: Python**:
```python
import yaml

with open('expected_changes.yml') as f:
    config = yaml.safe_load(f)

expected_sections = [fix['section'] for fix in config['fixes']]
preserved_sections = [p['section'] for p in config['preservation']]
```

**Option B: yq tool**:
```bash
# Install: brew install yq (Python-based YAML processor)

expected_sections=$(yq -r '.fixes[].section' expected_changes.yml)
preserved_sections=$(yq -r '.preservation[].section' expected_changes.yml)
```

### 4. Integration Tests

**Test with real specs**:
- ABDF spec (if ORIGINAL existed)
- Create synthetic spec with known changes
- Test edge cases (emoji, special characters, nested sections)

---

## Decision Matrix

| Use Case | Current Status | Recommendation |
|----------|---------------|----------------|
| **Development feedback** | ✅ OK | Use script, investigate failures |
| **CI gate (blocking)** | ❌ NOT READY | Wait for Phase 1.5 hardening |
| **Manual verification aid** | ✅ OK | Use script output as guide |
| **Merge decision** | ❌ NOT SUFFICIENT | Require manual review |
| **Audit trail** | 🟡 PARTIAL | Evidence generated, but not proof |

---

## Honest Assessment

### What We Achieved ✅
- Initial validation infrastructure
- Template for future hardening
- Evidence generation framework
- Clear path to CI-authoritative status

### What We Did NOT Achieve ❌
- CI-authoritative validation
- Proven correctness (no fixture tests)
- Robust section detection
- Production-grade reliability

### What We Learned 🧠
- "Production-grade" requires fixture-based validation
- Heuristic approaches need testing to prove correctness
- CI-authoritative status is earned, not declared
- Honest limitations > false confidence

---

## Next Steps (Phase 1.5)

**Week 2 Priority**:
1. Create fixture-based tests (PASS/FAIL cases)
2. Implement diff hunk → section resolver (Python script)
3. Replace regex YAML parsing (use yq or Python)
4. Test false positive/negative scenarios
5. Achieve CI-authoritative status

**Acceptance Criteria**:
- [ ] All fixture tests pass
- [ ] False positive rate < 1%
- [ ] False negative rate = 0%
- [ ] Integration tests with real specs pass
- [ ] Documentation updated with test results

---

## Commitment

**This script is ALPHA status.**

- ✅ Use for development feedback
- ❌ Do NOT use as CI gate yet
- ⏳ Hardening required before CI-authoritative
- 🎯 Phase 1.5 will complete the work

**No shortcuts. No premature declarations. Honest status only.**

---

**Status**: 🟡 ALPHA  
**CI-Authoritative**: ❌ NOT YET  
**Next Phase**: 1.5 Hardening (Week 2)  
**Owner**: Kiro + Kenan AY (Review)
