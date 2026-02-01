# Allow → Refactor Recommendation Engine (ARRE) Configuration

## 🎯 Constitutional Framework

**Authority**: Kenan AY - Architectural Steward  
**Status**: Constitutional Configuration  
**Principle**: "Allow = Semptom, Refactor = Tedavi" (Allow = Symptom, Refactor = Treatment)  
**Last Modified**: 2026-01-31  

## 🔒 Constitutional Constraints

**CRITICAL**: ARRE configuration can only **tighten** behavior, never soften constitutional requirements. All age thresholds represent **maximums** that can be decreased (made stricter) but never increased.

## 🔄 Refactor Pattern Classification

### Canonical Refactor Classes

```toml
[refactor_classes]
# Six canonical refactor classes (constitutional requirement)
ClockAbstraction = { 
    description = "Abstract time operations through dependency injection",
    target_rules = ["TIME.INSTANT", "TIME.SLEEP", "TIME.TIMEOUT"],
    difficulty = "medium",
    automation_potential = "high"
}

ErrorBoundary = {
    description = "Implement proper error handling boundaries",
    target_rules = ["ERROR.UNWRAP", "ERROR.EXPECT", "ERROR.PANIC"],
    difficulty = "low",
    automation_potential = "high"
}

MemoryArena = {
    description = "Implement arena-based memory allocation",
    target_rules = ["ALLOC.GLOBAL", "ALLOC.HEAP_DIRECT"],
    difficulty = "high",
    automation_potential = "medium"
}

SeededExecution = {
    description = "Replace non-deterministic operations with seeded alternatives",
    target_rules = ["DETERMINISM.RNG", "DETERMINISM.TIME"],
    difficulty = "medium",
    automation_potential = "medium"
}

StateParameterization = {
    description = "Convert global state to parameterized state",
    target_rules = ["DETERMINISM.GLOBAL"],
    difficulty = "high",
    automation_potential = "low"
}

TypedDecision = {
    description = "Replace decision logic with type-safe alternatives",
    target_rules = ["STYLE.NAMING", "STYLE.DOCUMENTATION"],
    difficulty = "low",
    automation_potential = "high"
}
```

### Rule-to-Pattern Mapping

```toml
[rule_pattern_mapping]
# Deterministic mapping (constitutional requirement: single-source)
"TIME.INSTANT" = "ClockAbstraction"
"TIME.SLEEP" = "ClockAbstraction"
"TIME.TIMEOUT" = "ClockAbstraction"

"ERROR.UNWRAP" = "ErrorBoundary"
"ERROR.EXPECT" = "ErrorBoundary"
"ERROR.PANIC" = "ErrorBoundary"

"ALLOC.GLOBAL" = "MemoryArena"
"ALLOC.HEAP_DIRECT" = "MemoryArena"

"DETERMINISM.RNG" = "SeededExecution"
"DETERMINISM.TIME" = "SeededExecution"

"DETERMINISM.GLOBAL" = "StateParameterization"

"STYLE.NAMING" = "TypedDecision"
"STYLE.DOCUMENTATION" = "TypedDecision"
```

## ⏰ Age-Based Triggering Configuration

### Module-Specific Age Thresholds

```toml
[age_thresholds]
# Kernel modules (constitutional maximum: 20 commits)
kernel = 20

# Userspace modules (constitutional maximum: 50 commits)
userspace = 50

# Tooling modules (constitutional maximum: 100 commits)
tooling = 100

# System modules (constitutional maximum: 30 commits)
system = 30

# Infrastructure modules (constitutional maximum: 200 commits)
infrastructure = 200
```

### Escalation System

```toml
[escalation]
# Information level threshold (constitutional maximum: 50% of module threshold)
information_threshold_percent = 50

# Warning level threshold (constitutional maximum: 80% of module threshold)
warning_threshold_percent = 80

# Error level threshold (constitutional maximum: 100% of module threshold)
error_threshold_percent = 100

# Grace period for complex refactors (constitutional maximum: 50 commits)
grace_period_commits = 50

# Architectural approval required for grace period (constitutional requirement: true)
architectural_approval_required = true
```

## 🎯 Recommendation Generation Configuration

### Difficulty Estimation

```toml
[difficulty_estimation]
# Difficulty factors (constitutional minimums)
scope_factor_weight = 0.3
complexity_factor_weight = 0.3
dependency_factor_weight = 0.2
risk_factor_weight = 0.2

# Difficulty levels
easy_threshold = 2.0
medium_threshold = 5.0
hard_threshold = 8.0
# Above 8.0: Very Hard
```

### Impact Calculation

```toml
[impact_calculation]
# MARS integration enabled (constitutional requirement: true)
mars_integration = true

# Risk reduction factors by refactor class (constitutional minimums)
ClockAbstraction_impact = 0.40    # 40% risk reduction
ErrorBoundary_impact = 0.25       # 25% risk reduction
MemoryArena_impact = 0.35         # 35% risk reduction
SeededExecution_impact = 0.30     # 30% risk reduction
StateParameterization_impact = 0.45 # 45% risk reduction
TypedDecision_impact = 0.15       # 15% risk reduction
```

### Automation Level Assessment

```toml
[automation_levels]
# Fully automated threshold (constitutional minimum: 90%)
fully_automated_threshold = 90

# Semi-automated threshold (constitutional minimum: 60%)
semi_automated_threshold = 60

# Manual threshold: below semi_automated_threshold

# Automation factors
pattern_complexity_weight = 0.4
scope_impact_weight = 0.3
risk_assessment_weight = 0.3
```

## 🚨 CI Enforcement Configuration

### Age Threshold Enforcement

```toml
[ci_enforcement]
# Age threshold enforcement enabled (constitutional requirement: true)
age_threshold_enforcement = true

# CI fail on exceeded allows (constitutional requirement: true)
ci_fail_exceeded_allows = true

# Architectural debt explosion detection (constitutional requirement: true)
debt_explosion_detection = true

# Debt explosion threshold (constitutional maximum: 5 files)
debt_explosion_threshold = 5
```

### Refactor Acknowledgment

```toml
[refactor_acknowledgment]
# Acknowledgment tracking enabled (constitutional requirement: true)
acknowledgment_tracking = true

# Progress monitoring enabled (constitutional requirement: true)
progress_monitoring = true

# Architectural approval support (constitutional requirement: true)
architectural_approval_support = true

# Extended timeline approval required (constitutional requirement: true)
extended_timeline_approval = true
```

## 📊 Pattern Library Configuration

### Implementation Templates

```toml
[pattern_library]
# Template completeness required (constitutional requirement: true)
template_completeness = true

# Before/after examples required (constitutional requirement: true)
before_after_examples = true

# Step-by-step guides required (constitutional requirement: true)
step_by_step_guides = true

# Testing strategies required (constitutional requirement: true)
testing_strategies = true

# Troubleshooting guides required (constitutional requirement: true)
troubleshooting_guides = true
```

### Pattern Extensibility

```toml
[pattern_extensibility]
# Custom pattern support (constitutional requirement: true)
custom_pattern_support = true

# Organization-specific patterns allowed (constitutional requirement: true)
org_specific_patterns = true

# Pattern validation required (constitutional requirement: true)
pattern_validation = true

# Constitutional compliance required (constitutional requirement: true)
constitutional_compliance = true
```

## 🔍 Architectural Debt Explosion Detection

### Cross-Module Analysis

```toml
[debt_explosion]
# Cross-module pattern analysis (constitutional requirement: true)
cross_module_analysis = true

# Same pattern threshold (constitutional maximum: 5 files/modules)
same_pattern_threshold = 5

# Escalation to system-wide changes (constitutional requirement: true)
system_wide_escalation = true

# Shared component recommendations (constitutional requirement: true)
shared_component_recommendations = true
```

### Fragmentation Prevention

```toml
[fragmentation_prevention]
# Architectural fragmentation detection (constitutional requirement: true)
fragmentation_detection = true

# Pattern unification suggestions (constitutional requirement: true)
pattern_unification = true

# Consistency enforcement (constitutional requirement: true)
consistency_enforcement = true

# Proactive pattern detection (constitutional requirement: true)
proactive_detection = true
```

## 📈 Progress Tracking Configuration

### Lifecycle Management

```toml
[lifecycle_management]
# Status tracking enabled (constitutional requirement: true)
status_tracking = true

# Timeline tracking enabled (constitutional requirement: true)
timeline_tracking = true

# Effort estimation vs actuals (constitutional requirement: true)
effort_tracking = true

# Completion metrics (constitutional requirement: true)
completion_metrics = true
```

### Refactor Backlog

```toml
[refactor_backlog]
# Backlog generation enabled (constitutional requirement: true)
backlog_generation = true

# Impact-based prioritization (constitutional requirement: true)
impact_prioritization = true

# Sprint planning integration (constitutional requirement: true)
sprint_integration = true

# Project management integration (constitutional requirement: true)
pm_integration = true
```

## 🎛️ Performance Configuration

### Analysis Performance

```toml
[performance]
# Incremental analysis enabled (constitutional requirement: true)
incremental_analysis = true

# Caching enabled (constitutional requirement: true)
caching_enabled = true

# Parallel processing enabled (constitutional requirement: true)
parallel_processing = true

# Deterministic analysis required (constitutional requirement: true)
deterministic_analysis = true
```

### Memory Management

```toml
[memory_management]
# Memory optimization enabled (constitutional requirement: true)
memory_optimization = true

# Lazy loading enabled (constitutional requirement: true)
lazy_loading = true

# Pattern library streaming (constitutional requirement: true)
pattern_streaming = true
```

## 🧪 Testing Configuration

### Test Requirements

```toml
[testing]
# Recommendation generation tests (constitutional requirement: true)
recommendation_tests = true

# Pattern classification tests (constitutional requirement: true)
pattern_classification_tests = true

# Age triggering tests (constitutional requirement: true)
age_triggering_tests = true

# CI enforcement tests (constitutional requirement: true)
ci_enforcement_tests = true

# Progress tracking tests (constitutional requirement: true)
progress_tracking_tests = true
```

### Validation Tests

```toml
[validation_tests]
# Constitutional compliance tests (constitutional requirement: true)
constitutional_compliance_tests = true

# Pattern mapping tests (constitutional requirement: true)
pattern_mapping_tests = true

# Debt explosion tests (constitutional requirement: true)
debt_explosion_tests = true

# Lifecycle management tests (constitutional requirement: true)
lifecycle_tests = true
```

## 🔧 Project Customization Examples

### Tightening Configuration

```toml
# Example 1: Stricter age thresholds
[age_thresholds]
kernel = 15         # Decreased from default 20
userspace = 40      # Decreased from default 50
tooling = 80        # Decreased from default 100

# Example 2: Lower debt explosion threshold
[debt_explosion]
same_pattern_threshold = 3  # Decreased from default 5

# Example 3: Higher automation requirements
[automation_levels]
fully_automated_threshold = 95  # Increased from default 90
semi_automated_threshold = 70   # Increased from default 60
```

### Module-Specific Overrides

```toml
# Example: Critical kernel modules have stricter requirements
[module_overrides."kernel.mm"]
age_threshold = 10
escalation.information_threshold_percent = 30
escalation.warning_threshold_percent = 60

# Example: Tooling modules have relaxed requirements
[module_overrides."tools"]
age_threshold = 150
escalation.grace_period_commits = 100
```

## 📊 Integration Configuration

### MARS Integration

```toml
[mars_integration]
# MARS integration enabled (constitutional requirement: true)
enabled = true

# Risk reduction calculation (constitutional requirement: true)
risk_reduction_calculation = true

# Module impact analysis (constitutional requirement: true)
module_impact_analysis = true

# ROI calculation (constitutional requirement: true)
roi_calculation = true
```

### VS Code Integration

```toml
[vscode_integration]
# Information-level diagnostics (constitutional requirement: true)
information_diagnostics = true

# Positive messaging (constitutional requirement: true)
positive_messaging = true

# Quick fixes for simple patterns (constitutional requirement: true)
quick_fixes = true

# Implementation guides (constitutional requirement: true)
implementation_guides = true
```

## 🔒 Constitutional Guarantee

**IMMUTABLE PRINCIPLES**:

1. **Age-Based Triggering is Mandatory** - Cannot be disabled
2. **Six Canonical Refactor Classes** - Cannot be modified without constitutional amendment
3. **Single-Source Rule Mapping** - Rule-to-pattern mapping is deterministic
4. **CI Enforcement Required** - Age thresholds must be enforced in CI
5. **Debt Explosion Detection** - Cross-module pattern analysis is mandatory

**FORBIDDEN MODIFICATIONS**:
- Disabling age-based triggering
- Softening age thresholds beyond constitutional maximums
- Bypassing CI enforcement
- Removing debt explosion detection
- Modifying canonical refactor classes without authority

**ENFORCEMENT**: These principles are enforced by the constitutional system and cannot be overridden by any configuration, exception, or authority.

**AUTHORITY**: Constitutional Steward (Kenan AY) - Final authority on ARRE constitutional requirements.

---

**IMPLEMENTATION NOTE**: This configuration system enforces the constitutional principle that "Allow = Symptom, Refactor = Treatment" by providing systematic transformation from temporary exceptions to permanent architectural improvements.