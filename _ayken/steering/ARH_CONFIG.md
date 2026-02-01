# Auto-Refactor Hints (ARH) Configuration

## 🎯 Constitutional Framework

**Authority**: Kenan AY - Architectural Steward  
**Status**: Constitutional Configuration  
**Principle**: "Otomatik refactor = güvenli refactor" (Automatic refactor = safe refactor)  
**Last Modified**: 2026-01-31  

## 🔒 Constitutional Constraints

**CRITICAL**: ARH configuration can only **tighten** behavior, never soften constitutional requirements. All confidence thresholds represent **minimums** that can be increased (made stricter) but never decreased.

## 🤖 Automation Level Configuration

### Confidence Thresholds

```toml
[confidence_thresholds]
# SafeAutofix minimum confidence (constitutional minimum: 95%)
safe_autofix_threshold = 95

# AssistedFix minimum confidence (constitutional minimum: 60%)
assisted_fix_threshold = 60

# DesignHint maximum confidence (constitutional maximum: 60%)
design_hint_threshold = 60

# Confidence calculation precision (constitutional requirement: 2 decimal places)
confidence_precision = 2
```

### Automation Boundaries

```toml
[automation_boundaries]
# SafeAutofix restrictions (constitutional requirements)
safe_autofix_userspace_only = true
safe_autofix_no_kernel = true
safe_autofix_no_hot_path = true
safe_autofix_deterministic_only = true

# AssistedFix requirements (constitutional requirements)
assisted_fix_preview_required = true
assisted_fix_approval_required = true
assisted_fix_impact_analysis = true

# DesignHint scope (constitutional requirements)
design_hint_kernel_allowed = true
design_hint_educational_only = true
design_hint_no_execution = true
```

## 🛡️ Safety Configuration

### Trust Boundary Enforcement

```toml
[trust_boundaries]
# Kernel code restrictions (constitutional requirements)
kernel_safeautofix_prohibited = true
kernel_assistedfix_manual_approval = true
kernel_designhint_only = false  # DesignHint allowed in kernel

# Userspace code permissions (constitutional requirements)
userspace_safeautofix_allowed = true
userspace_assistedfix_allowed = true
userspace_designhint_allowed = true

# Tooling code permissions (constitutional requirements)
tooling_safeautofix_allowed = true
tooling_assistedfix_allowed = true
tooling_designhint_allowed = true
```

### Risk Assessment Configuration

```toml
[risk_assessment]
# Risk level thresholds (constitutional requirements)
low_risk_threshold = 2.0
medium_risk_threshold = 5.0
high_risk_threshold = 8.0
# Above 8.0: Critical risk

# Risk factors (constitutional minimums)
security_impact_weight = 0.4
performance_impact_weight = 0.3
maintainability_impact_weight = 0.2
complexity_impact_weight = 0.1

# Security risk multipliers (constitutional minimums)
capability_change_multiplier = 2.0
boundary_crossing_multiplier = 3.0
privilege_change_multiplier = 5.0
```

## 🔧 Safe Autofix Configuration

### Pattern Library

```toml
[safe_autofix_patterns]
# Import replacement patterns (constitutional requirement: deterministic)
"std::time::Instant" = "ayken::time::PerfInstant"
"std::time::SystemTime" = "ayken::time::SystemClock"
"rand::random" = "ayken::rand::SeededRng::generate"
"std::thread::sleep" = "ayken::time::Sleep::for_duration"

# Expression replacement patterns (constitutional requirement: semantic equivalence)
"unwrap()" = "expect(\"Safe unwrap: validated precondition\")"
"panic!()" = "ayken::error::controlled_panic"
"Box::new" = "ayken::alloc::arena_alloc"
```

### Safety Validation

```toml
[safety_validation]
# Semantic equivalence validation (constitutional requirement: true)
semantic_equivalence_check = true

# Side effect detection (constitutional requirement: true)
side_effect_detection = true

# Security invariant checking (constitutional requirement: true)
security_invariant_check = true

# Hot path detection (constitutional requirement: true)
hot_path_detection = true

# Kernel code detection (constitutional requirement: true)
kernel_code_detection = true
```

### Workspace Edit Generation

```toml
[workspace_edits]
# VS Code integration enabled (constitutional requirement: true)
vscode_integration = true

# Atomic edits required (constitutional requirement: true)
atomic_edits = true

# Rollback capability required (constitutional requirement: true)
rollback_capability = true

# Preview before apply (constitutional requirement: true)
preview_before_apply = true
```

## 🤝 Assisted Fix Configuration

### Preview Generation

```toml
[assisted_fix_preview]
# Detailed preview required (constitutional requirement: true)
detailed_preview = true

# Before/after code display (constitutional requirement: true)
before_after_display = true

# Security delta summary (constitutional requirement: true)
security_delta_summary = true

# Performance impact estimate (constitutional requirement: true)
performance_impact_estimate = true

# Signature change analysis (constitutional requirement: true)
signature_change_analysis = true
```

### Approval Workflow

```toml
[approval_workflow]
# Human approval required (constitutional requirement: true)
human_approval_required = true

# Kernel code opt-in approval (constitutional requirement: true)
kernel_opt_in_approval = true

# Interactive approval workflow (constitutional requirement: true)
interactive_workflow = true

# Approval tracking (constitutional requirement: true)
approval_tracking = true

# Approval authority validation (constitutional requirement: true)
approval_authority_validation = true
```

### Impact Analysis

```toml
[impact_analysis]
# Ripple analysis enabled (constitutional requirement: true)
ripple_analysis = true

# Call site analysis (constitutional requirement: true)
call_site_analysis = true

# Dependency impact analysis (constitutional requirement: true)
dependency_impact = true

# Layer boundary analysis (constitutional requirement: true)
layer_boundary_analysis = true
```

## 🎓 Design Hint Configuration

### Architectural Guidance

```toml
[design_hints]
# Comprehensive guidance required (constitutional requirement: true)
comprehensive_guidance = true

# Implementation roadmaps required (constitutional requirement: true)
implementation_roadmaps = true

# Educational content required (constitutional requirement: true)
educational_content = true

# Best practices included (constitutional requirement: true)
best_practices = true

# Anti-patterns warnings (constitutional requirement: true)
anti_patterns = true
```

### Pattern-Specific Guidance

```toml
[pattern_guidance]
# ALLOC.GLOBAL guidance (constitutional requirement: comprehensive)
alloc_global_guidance = "comprehensive"

# DETERMINISM.RNG guidance (constitutional requirement: comprehensive)
determinism_rng_guidance = "comprehensive"

# TIME.INSTANT guidance (constitutional requirement: comprehensive)
time_instant_guidance = "comprehensive"

# ERROR.UNWRAP guidance (constitutional requirement: comprehensive)
error_unwrap_guidance = "comprehensive"

# SECURITY violations guidance (constitutional requirement: comprehensive)
security_guidance = "comprehensive"
```

## 🎯 Hint Generation Configuration

### Orchestration Settings

```toml
[hint_orchestration]
# Comprehensive hint generation (constitutional requirement: true)
comprehensive_generation = true

# Canonical fix mapping (constitutional requirement: true)
canonical_fix_mapping = true

# Hint prioritization enabled (constitutional requirement: true)
hint_prioritization = true

# Combined risk scoring (constitutional requirement: true)
combined_risk_scoring = true

# ARRE precedence enforcement (constitutional requirement: true)
arre_precedence = true
```

### Generation Performance

```toml
[generation_performance]
# VS Code profile latency SLA (constitutional maximum: 100ms)
vscode_latency_sla = 100

# CI profile latency SLA (constitutional maximum: 5000ms)
ci_latency_sla = 5000

# Incremental generation enabled (constitutional requirement: true)
incremental_generation = true

# Caching enabled (constitutional requirement: true)
caching_enabled = true
```

## 🖥️ VS Code Integration Configuration

### Code Actions

```toml
[vscode_code_actions]
# Native code action generation (constitutional requirement: true)
native_code_actions = true

# Quick fixes for SafeAutofix (constitutional requirement: true)
quick_fixes_safeautofix = true

# Refactor actions for AssistedFix (constitutional requirement: true)
refactor_actions_assistedfix = true

# Information actions for DesignHint (constitutional requirement: true)
information_actions_designhint = true

# Lightbulb indicators (constitutional requirement: true)
lightbulb_indicators = true
```

### Diagnostic Integration

```toml
[vscode_diagnostics]
# Automation level display (constitutional requirement: true)
automation_level_display = true

# Confidence score display (constitutional requirement: true)
confidence_score_display = true

# Risk assessment display (constitutional requirement: true)
risk_assessment_display = true

# Security risk warnings (constitutional requirement: true)
security_risk_warnings = true

# Kernel file restrictions (constitutional requirement: true)
kernel_file_restrictions = true
```

### Real-time Analysis

```toml
[realtime_analysis]
# Incremental analysis (constitutional requirement: true)
incremental_analysis = true

# Debounced updates (constitutional requirement: true)
debounced_updates = true

# Stability optimization (constitutional requirement: true)
stability_optimization = true

# Performance monitoring (constitutional requirement: true)
performance_monitoring = true
```

## 🚀 CLI Integration Configuration

### Fix Commands

```toml
[cli_commands]
# Batch processing enabled (constitutional requirement: true)
batch_processing = true

# Interactive workflows (constitutional requirement: true)
interactive_workflows = true

# Dry-run mode (constitutional requirement: true)
dry_run_mode = true

# Progress reporting (constitutional requirement: true)
progress_reporting = true

# Rollback capability (constitutional requirement: true)
rollback_capability = true
```

### Validation and Testing

```toml
[cli_validation]
# Pre-application validation (constitutional requirement: true)
pre_application_validation = true

# Post-application testing (constitutional requirement: true)
post_application_testing = true

# Regression detection (constitutional requirement: true)
regression_detection = true

# Quality assurance (constitutional requirement: true)
quality_assurance = true
```

## 🔄 ARRE-ARH Integration Configuration

### Unified Workflow

```toml
[arre_arh_integration]
# Integrated refactor system (constitutional requirement: true)
integrated_system = true

# Unified guidance structure (constitutional requirement: true)
unified_guidance = true

# Consistency validation (constitutional requirement: true)
consistency_validation = true

# Implementation path synthesis (constitutional requirement: true)
implementation_synthesis = true

# Coordinated hint generation (constitutional requirement: true)
coordinated_generation = true
```

### Priority Management

```totml
[priority_management]
# ARRE strategic priority (constitutional requirement: true)
arre_strategic_priority = true

# ARH tactical implementation (constitutional requirement: true)
arh_tactical_implementation = true

# Recommendation alignment (constitutional requirement: true)
recommendation_alignment = true

# Architectural impact consideration (constitutional requirement: true)
architectural_impact = true
```

## 🧪 Testing Configuration

### Test Requirements

```toml
[testing]
# Hint generation tests (constitutional requirement: true)
hint_generation_tests = true

# Safety validation tests (constitutional requirement: true)
safety_validation_tests = true

# Automation level tests (constitutional requirement: true)
automation_level_tests = true

# Integration tests (constitutional requirement: true)
integration_tests = true

# Performance tests (constitutional requirement: true)
performance_tests = true
```

### Quality Assurance

```toml
[quality_assurance]
# Constitutional compliance tests (constitutional requirement: true)
constitutional_compliance_tests = true

# Safety boundary tests (constitutional requirement: true)
safety_boundary_tests = true

# Risk assessment tests (constitutional requirement: true)
risk_assessment_tests = true

# Workflow integration tests (constitutional requirement: true)
workflow_integration_tests = true
```

## 🔧 Project Customization Examples

### Tightening Configuration

```toml
# Example 1: Stricter confidence thresholds
[confidence_thresholds]
safe_autofix_threshold = 98    # Increased from default 95
assisted_fix_threshold = 70   # Increased from default 60
design_hint_threshold = 50    # Decreased from default 60

# Example 2: Stricter risk assessment
[risk_assessment]
low_risk_threshold = 1.5      # Decreased from default 2.0
medium_risk_threshold = 4.0   # Decreased from default 5.0
high_risk_threshold = 6.0     # Decreased from default 8.0

# Example 3: Enhanced safety validation
[safety_validation]
# All safety checks remain enabled (cannot be disabled)
# Additional project-specific validations can be added
```

### Module-Specific Overrides

```toml
# Example: Kernel modules have maximum restrictions
[module_overrides."kernel/**"]
safe_autofix_threshold = 100  # No SafeAutofix in kernel
assisted_fix_threshold = 90   # Very high threshold for AssistedFix
design_hint_only = true       # Only DesignHint allowed

# Example: Tooling modules have relaxed thresholds
[module_overrides."tools/**"]
safe_autofix_threshold = 90   # Slightly lower threshold
assisted_fix_threshold = 50   # Lower threshold for tooling
```

## 📊 Monitoring Configuration

### Metrics Collection

```toml
[monitoring]
# Hint generation metrics (constitutional requirement: true)
hint_generation_metrics = true

# Automation success rates (constitutional requirement: true)
automation_success_rates = true

# Safety validation metrics (constitutional requirement: true)
safety_validation_metrics = true

# User acceptance rates (constitutional requirement: true)
user_acceptance_rates = true

# Performance metrics (constitutional requirement: true)
performance_metrics = true
```

### Reporting

```toml
[reporting]
# Usage analytics (constitutional requirement: true)
usage_analytics = true

# Safety incident reporting (constitutional requirement: true)
safety_incident_reporting = true

# Quality metrics reporting (constitutional requirement: true)
quality_metrics_reporting = true

# Improvement recommendations (constitutional requirement: true)
improvement_recommendations = true
```

## 🔒 Constitutional Guarantee

**IMMUTABLE PRINCIPLES**:

1. **Safety First** - SafeAutofix must be 95%+ confidence and userspace-only
2. **Human Approval Required** - AssistedFix requires human approval
3. **Educational Purpose** - DesignHint provides guidance, never execution
4. **Trust Boundary Enforcement** - Kernel code has maximum restrictions
5. **Constitutional Compliance** - All hints must respect constitutional rules

**FORBIDDEN MODIFICATIONS**:
- Lowering SafeAutofix confidence below 95%
- Disabling human approval for AssistedFix
- Allowing SafeAutofix in kernel code
- Bypassing safety validation
- Softening trust boundary restrictions

**ENFORCEMENT**: These principles are enforced by the constitutional system and cannot be overridden by any configuration, exception, or authority.

**AUTHORITY**: Constitutional Steward (Kenan AY) - Final authority on ARH constitutional requirements.

---

**IMPLEMENTATION NOTE**: This configuration system enforces the constitutional principle that "Automatic refactor = safe refactor" by providing strict safety boundaries and human oversight for all automated code transformations.