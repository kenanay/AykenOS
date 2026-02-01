# ARH Configuration Philosophy

This file configures ARH behavior.
It does NOT:
- force fixes
- weaken safety rules
- override constitutional limits

Configuration may only tighten behavior, never relax it.

## Minimal Safe Example

[confidence]
safe_autofix_min = 95
assisted_fix_min = 90

[preferences]
prefer_safe_over_assisted = true
disabled_rules = []

disabled_patterns = []

[safety]
allow_kernel_fixes = false
allow_cross_module = false
allow_design_hint_enforcement = false

## Invalid Examples

# ❌ assisted_fix_min = 70
# Rejected: below constitutional minimum

# ❌ allow_design_hint_enforcement = true
# Rejected: design hints are advisory-only

# ❌ allow_kernel_fixes = true
# Rejected: kernel fixes cannot be enabled via config
