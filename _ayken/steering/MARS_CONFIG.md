# Module-level Architecture Risk Score (MARS) Configuration

## 🎯 Constitutional Framework

**Authority**: Kenan AY - Architectural Steward  
**Status**: Constitutional Configuration  
**Principle**: "Mimari sorunlar lokaldir, bedeli küresel olur" (Architectural problems are local, cost is global)  
**Last Modified**: 2026-01-31  

## 🔒 Constitutional Constraints

**CRITICAL**: MARS configuration can only **tighten** behavior, never soften constitutional requirements. All risk thresholds represent **maximums** that can be decreased (made stricter) but never increased.

## 🏗️ Module Boundary Configuration

### Automatic Detection Rules

```toml
[module_detection]
# Deterministic boundary detection (constitutional requirement: true)
deterministic_detection = true

# Conflict enforcement enabled (constitutional requirement: true)
conflict_enforcement = true

# Complete file coverage required (constitutional requirement: true)
complete_coverage = true

# No overlaps allowed (constitutional requirement: true)
no_overlaps = true
```

### Module Hierarchy Patterns

```toml
[module_patterns]
# Kernel modules
"kernel/mm/**" = { module_id = "kernel.mm", type = "kernel", criticality = "high" }
"kernel/sched/**" = { module_id = "kernel.sched", type = "kernel", criticality = "high" }
"kernel/fs/**" = { module_id = "kernel.fs", type = "kernel", criticality = "high" }
"kernel/drivers/**" = { module_id = "kernel.drivers", type = "kernel", criticality = "medium" }

# Userspace modules
"userspace/semantic-cli/**" = { module_id = "userspace.cli", type = "userspace", criticality = "medium" }
"userspace/ai-runtime/**" = { module_id = "userspace.ai", type = "userspace", criticality = "medium" }
"userspace/bcib-runtime/**" = { module_id = "userspace.bcib", type = "userspace", criticality = "low" }

# Tooling modules
"tools/**" = { module_id = "tools", type = "tooling", criticality = "low" }
"bootloader/**" = { module_id = "bootloader", type = "system", criticality = "high" }

# Repository infrastructure
"docs/**" = { module_id = "repository.docs", type = "infrastructure", criticality = "low" }
".github/**" = { module_id = "repository.ci", type = "infrastructure", criticality = "low" }
```

## ⚖️ Risk Calculation Configuration

### Rule Weight Hierarchy

```toml
[rule_weights]
# DETERMINISM rules (highest weight - constitutional minimum: 5.0)
"DETERMINISM.GLOBAL" = 5.0
"DETERMINISM.RNG" = 5.0
"DETERMINISM.TIME" = 5.0

# MEMORY/ALLOC rules (critical weight - constitutional minimum: 4.0)
"MEMORY.CONTRACT.VIOLATION" = 4.0
"MEMORY.LEAK" = 4.0
"ALLOC.GLOBAL" = 4.0
"ALLOC.HEAP_DIRECT" = 3.5

# TIME rules (critical weight - constitutional minimum: 4.0)
"TIME.INSTANT" = 4.0
"TIME.SLEEP" = 3.5
"TIME.TIMEOUT" = 3.0

# ERROR rules (important weight - constitutional minimum: 2.5)
"ERROR.UNWRAP" = 3.0
"ERROR.EXPECT" = 2.5
"ERROR.PANIC" = 3.5

# SECURITY rules (critical weight - constitutional minimum: 4.0)
"SECURITY.BOUNDARY.VIOLATION" = 5.0
"SECURITY.PRIVILEGE.ESCALATION" = 5.0
"SECURITY.INFORMATION.LEAK" = 4.0

# KERNEL rules (critical weight - constitutional minimum: 5.0)
"KERNEL.SAFETY.CRITICAL" = 5.0
"KERNEL.RING0.POLICY" = 5.0
"KERNEL.CAPABILITY.BYPASS" = 5.0

# STYLE rules (lowest weight - constitutional minimum: 1.0)
"STYLE.FORMATTING" = 1.0
"STYLE.NAMING" = 1.0
"STYLE.DOCUMENTATION" = 1.5
```

### Exception Type Multipliers

```toml
[exception_multipliers]
# Allow attribute multiplier (constitutional minimum: 1.0)
allow_multiplier = 1.0

# Waiver multiplier (constitutional minimum: 1.5)
waiver_multiplier = 1.5

# Expired waiver multiplier (constitutional minimum: 2.0)
expired_waiver_multiplier = 2.0

# Undocumented allow multiplier (constitutional minimum: 3.0)
undocumented_allow_multiplier = 3.0

# Escalated allow multiplier (3+ same rule - constitutional minimum: 2.0)
escalated_allow_multiplier = 2.0
```

### Temporal Factor Configuration

```toml
[temporal_factors]
# Age calculation method (constitutional requirement: "commits")
age_calculation = "commits"

# Age divisor for temporal penalty (constitutional maximum: 50)
age_divisor = 50

# Maximum temporal multiplier (constitutional maximum: 3.0)
max_temporal_multiplier = 3.0

# Temporal factor formula: risk *= (1 + age_in_commits / age_divisor)
# Capped at max_temporal_multiplier
```

## 🎯 Risk Classification System

### Risk Level Thresholds

```toml
[risk_classification]
# Healthy level (constitutional maximum: 20)
healthy_threshold = 20

# Monitored level (constitutional maximum: 40)
monitored_threshold = 40

# Risky level (constitutional maximum: 60)
risky_threshold = 60

# Critical level (constitutional maximum: 80)
critical_threshold = 80

# Quarantine level: above critical_threshold (automatic)
# 81-100: Quarantine (constitutional requirement)
```

### Risk Level Actions

```toml
[risk_actions]
# Healthy (0-20): No restrictions
healthy_actions = ["allow_progression", "standard_review"]

# Monitored (21-40): Increased attention
monitored_actions = ["increased_review", "trend_monitoring"]

# Risky (41-60): Mandatory planning
risky_actions = ["refactoring_plan_required", "architectural_review"]

# Critical (61-80): Immediate intervention
critical_actions = ["immediate_intervention", "block_progression", "mandatory_refactor"]

# Quarantine (81-100): Cannot progress
quarantine_actions = ["block_all_progression", "emergency_refactor", "architectural_redesign"]
```

## 🚨 CI Module Risk Validation

### CI Threshold Enforcement

```toml
[ci_validation]
# Critical module threshold (constitutional maximum: 60)
critical_module_threshold = 60

# Quarantine module threshold (constitutional maximum: 80)
quarantine_module_threshold = 80

# Block progression on critical (constitutional requirement: true)
block_progression_critical = true

# Block progression on quarantine (constitutional requirement: true)
block_progression_quarantine = true

# Fail CI on quarantine (constitutional requirement: true)
fail_ci_quarantine = true
```

### Module Trend Validation

```toml
[trend_validation]
# Negative trend detection enabled (constitutional requirement: true)
negative_trend_detection = true

# Trend window size in commits (constitutional minimum: 5)
trend_window_size = 10

# Significant degradation threshold (constitutional maximum: 10.0)
degradation_threshold = 10.0

# Fail CI on sustained degradation (constitutional requirement: true)
fail_ci_degradation = true
```

## 📊 Module-Specific Configuration

### Module Type Multipliers

```toml
[module_type_multipliers]
# Kernel modules have highest multiplier (constitutional minimum: 1.5)
kernel = 1.5

# System modules have high multiplier (constitutional minimum: 1.3)
system = 1.3

# Userspace modules have standard multiplier (constitutional minimum: 1.0)
userspace = 1.0

# Tooling modules have reduced multiplier (constitutional minimum: 0.8)
tooling = 0.8

# Infrastructure modules have lowest multiplier (constitutional minimum: 0.5)
infrastructure = 0.5
```

### Criticality Adjustments

```toml
[criticality_adjustments]
# High criticality modules (constitutional minimum: 1.2)
high = 1.2

# Medium criticality modules (constitutional minimum: 1.0)
medium = 1.0

# Low criticality modules (constitutional minimum: 0.8)
low = 0.8
```

## 🔍 Cross-Module Risk Analysis

### Dependency Risk Configuration

```toml
[dependency_risk]
# Cross-module analysis enabled (constitutional requirement: true)
enabled = true

# High-risk affecting low-risk detection (constitutional requirement: true)
risk_propagation_detection = true

# Architectural boundary violation detection (constitutional requirement: true)
boundary_violation_detection = true

# Dependency risk multiplier (constitutional minimum: 1.2)
dependency_risk_multiplier = 1.2
```

### Architectural Pattern Validation

```toml
[architectural_patterns]
# Pattern violation detection enabled (constitutional requirement: true)
pattern_violation_detection = true

# Cross-module pattern consistency (constitutional requirement: true)
cross_module_consistency = true

# Architectural debt propagation tracking (constitutional requirement: true)
debt_propagation_tracking = true
```

## 🎛️ Performance and Scalability

### Calculation Optimization

```toml
[performance]
# Incremental calculation enabled (constitutional requirement: true)
incremental_calculation = true

# Caching enabled (constitutional requirement: true)
caching_enabled = true

# Parallel processing enabled (constitutional requirement: true)
parallel_processing = true

# Deterministic calculation required (constitutional requirement: true)
deterministic_calculation = true
```

### Memory Management

```toml
[memory_management]
# Memory optimization enabled (constitutional requirement: true)
memory_optimization = true

# Lazy loading enabled (constitutional requirement: true)
lazy_loading = true

# Memory growth prevention (constitutional requirement: true)
memory_growth_prevention = true
```

## 📈 Monitoring and Reporting

### Dashboard Configuration

```toml
[dashboard]
# Top risky modules count (constitutional minimum: 5)
top_risky_modules_count = 10

# Risk trend visualization enabled (constitutional requirement: true)
risk_trend_visualization = true

# Module comparison enabled (constitutional requirement: true)
module_comparison = true

# Executive summary enabled (constitutional requirement: true)
executive_summary = true
```

### Reporting Configuration

```toml
[reporting]
# Risk breakdown by category (constitutional requirement: true)
risk_breakdown = true

# Refactoring ROI analysis (constitutional requirement: true)
refactoring_roi = true

# Sprint planning recommendations (constitutional requirement: true)
sprint_recommendations = true

# Team assignment suggestions (constitutional requirement: true)
team_assignments = true
```

## 🧪 Testing and Validation

### Test Requirements

```toml
[testing]
# Module risk calculation tests (constitutional requirement: true)
calculation_tests = true

# Boundary detection tests (constitutional requirement: true)
boundary_detection_tests = true

# Cross-module analysis tests (constitutional requirement: true)
cross_module_tests = true

# Performance tests (constitutional requirement: true)
performance_tests = true

# Determinism tests (constitutional requirement: true)
determinism_tests = true
```

### Validation Rules

```toml
[validation]
# Constitutional constraint enforcement (cannot be disabled)
enforce_constitutional_constraints = true

# Risk threshold validation (cannot be disabled)
validate_risk_thresholds = true

# Module coverage validation (cannot be disabled)
validate_module_coverage = true

# Boundary consistency validation (cannot be disabled)
validate_boundary_consistency = true
```

## 🔧 Project Customization Examples

### Tightening Configuration

```toml
# Example 1: Stricter risk thresholds
[risk_classification]
healthy_threshold = 15      # Decreased from default 20
monitored_threshold = 35    # Decreased from default 40
risky_threshold = 50        # Decreased from default 60
critical_threshold = 70     # Decreased from default 80

# Example 2: Higher rule weights for project-specific concerns
[rule_weights]
"TIME.INSTANT" = 5.0        # Increased from default 4.0
"ALLOC.GLOBAL" = 5.0        # Increased from default 4.0

# Example 3: Stricter CI validation
[ci_validation]
critical_module_threshold = 50  # Decreased from default 60
quarantine_module_threshold = 70 # Decreased from default 80
```

### Module-Specific Overrides

```toml
# Example: Kernel memory management has strictest requirements
[module_overrides."kernel.mm"]
rule_weights."MEMORY.CONTRACT.VIOLATION" = 6.0
rule_weights."ALLOC.GLOBAL" = 5.0
critical_threshold = 50
quarantine_threshold = 70

# Example: Tooling modules have relaxed requirements
[module_overrides."tools"]
healthy_threshold = 30
monitored_threshold = 50
risky_threshold = 70
```

## 🔒 Constitutional Guarantee

**IMMUTABLE PRINCIPLES**:

1. **Deterministic Boundary Detection** - Module boundaries must be deterministic
2. **Complete Coverage** - All files must belong to exactly one module
3. **Risk Threshold Enforcement** - Critical/Quarantine modules block progression
4. **Cross-Module Analysis** - Dependency risks must be tracked
5. **Constitutional Constraint Enforcement** - Configuration cannot soften requirements

**FORBIDDEN MODIFICATIONS**:
- Disabling deterministic calculation
- Removing complete coverage requirement
- Bypassing CI threshold enforcement
- Softening constitutional minimums
- Disabling cross-module analysis

**ENFORCEMENT**: These principles are enforced by the constitutional system and cannot be overridden by any configuration, exception, or authority.

**AUTHORITY**: Constitutional Steward (Kenan AY) - Final authority on MARS constitutional requirements.

---

**IMPLEMENTATION NOTE**: This configuration system enforces the constitutional principle that "Architectural problems are local, cost is global" by providing precise module-level risk attribution and preventing local problems from becoming global costs.