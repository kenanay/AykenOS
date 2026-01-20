# Loop Engine Warning Budget Policy

## Overview

The loop engine module follows a strict warning budget policy to maintain code quality and prevent technical debt accumulation. This policy is enforced through CI checks that specifically target the loop_engine module.

## Warning Budget

- **Maximum allowed warnings**: 5
- **Current warnings**: 0 (as of task 13.1 completion)
- **Scope**: `userspace/semantic-cli/src/loop_engine/` module only

## Policy Rules

### 1. Warning Budget Enforcement
- The CI system monitors warnings specifically in the loop_engine module
- Any PR that increases warnings above the budget limit will fail CI
- The budget is checked on every push and pull request affecting loop_engine files

### 2. Clippy Compliance
- The loop_engine module must pass clippy with a focused set of lints
- Clippy checks are run with the following configuration:
  - **Denied**: correctness, suspicious (critical issues only)
  - **Allowed**: Style and complexity lints that don't affect correctness
  - **Specific allowances**: cast_abs_to_unsigned, manual_div_ceil, manual_clamp, needless_borrows_for_generic_args, redundant_pattern_matching, needless_return, manual_is_multiple_of, new_without_default, derivable_impls

This configuration focuses on preventing bugs and security issues while allowing flexibility in code style and minor optimizations.

### 3. Warning Categories Monitored
- Unused imports
- Unused variables  
- Dead code
- Deprecated usage
- Unreachable patterns
- Unused mut variables

## CI Implementation

The warning budget is enforced through `.github/workflows/loop-engine-clippy.yml`:

1. **Targeted Monitoring**: Only files in `src/loop_engine/` trigger the workflow
2. **Filtered Output**: Clippy and warning checks focus only on loop_engine module
3. **Budget Validation**: Automatic failure if warning count exceeds limit
4. **Clear Feedback**: CI provides specific counts and actionable error messages

## Developer Guidelines

### Before Committing
```bash
# Check current warning count in loop_engine
cd userspace/semantic-cli
cargo check 2>&1 | grep "src/loop_engine/" | grep "warning:" | wc -l

# Run clippy on loop_engine focused lints
cargo clippy --lib -- \
  --allow clippy::all \
  --deny clippy::correctness \
  --deny clippy::suspicious \
  --allow clippy::cast_abs_to_unsigned \
  --allow clippy::manual_div_ceil \
  --allow clippy::manual_clamp \
  --allow clippy::needless_borrows_for_generic_args \
  --allow clippy::redundant_pattern_matching \
  --allow clippy::needless_return \
  --allow clippy::manual_is_multiple_of \
  --allow clippy::new_without_default \
  --allow clippy::derivable_impls
```

### Fixing Warnings
1. **Unused imports**: Remove or add `#[allow(unused_imports)]` if needed for future use
2. **Unused variables**: Prefix with `_` or add `#[allow(unused_variables)]` if intentional
3. **Dead code**: Remove or add `#[allow(dead_code)]` if part of planned API
4. **Deprecated usage**: Update to recommended alternatives

### Adding New Code
- All new code in loop_engine must not introduce warnings
- Use `#[allow(...)]` attributes sparingly and with justification
- Consider refactoring if approaching the warning budget limit

## Budget Adjustment Process

If the warning budget needs adjustment:

1. **Document justification** in a GitHub issue
2. **Update the budget** in `.github/workflows/loop-engine-clippy.yml`
3. **Update this documentation** with the new limit and rationale
4. **Get approval** from code owners before merging

## Rationale

This policy ensures:
- **Code Quality**: Maintains high standards in the critical loop_engine module
- **Technical Debt Prevention**: Prevents accumulation of warnings over time
- **Focused Enforcement**: Allows flexibility in other modules while being strict where it matters
- **Clear Boundaries**: Provides explicit limits and automated enforcement

## Related Files

- `.github/workflows/loop-engine-clippy.yml` - CI enforcement
- `userspace/semantic-cli/src/loop_engine/` - Monitored module
- This document - Policy specification

## History

- **Task 13.1**: Reduced warnings from 61 to 0 in loop_engine module
- **Task 13.2**: Established warning budget policy and CI enforcement