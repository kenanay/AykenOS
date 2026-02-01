# Architecture Health Time-Series (AHTS) Configuration

## 🎯 Constitutional Framework

**Authority**: Kenan AY - Architectural Steward  
**Status**: Constitutional Configuration  
**Principle**: "Tek snapshot yalan söyler. Trend asla yalan söylemez." (Single snapshot lies. Trend never lies.)  
**Last Modified**: 2026-01-31  

## 🔒 Constitutional Constraints

**CRITICAL**: AHTS configuration can only **tighten** behavior, never soften constitutional requirements. All thresholds represent **maximums** that can be decreased (made stricter) but never increased.

## 📊 Trend Analysis Configuration

### Linear Regression Parameters

```toml
[trend_analysis]
# Minimum samples required for trend calculation (constitutional minimum: 5)
minimum_samples = 5

# Window size for trend analysis in commits (constitutional minimum: 5)
trend_window_size = 10

# Confidence threshold for trend significance (constitutional minimum: 0.7)
confidence_threshold = 0.8

# Standard deviation threshold for trend validity (constitutional maximum: 2.0)
std_dev_threshold = 1.5
```

### Trend Classification Thresholds

```toml
[trend_classification]
# Positive trend threshold (constitutional minimum: +0.2)
positive_threshold = 0.2

# Negative trend threshold (constitutional maximum: -0.2)
negative_threshold = -0.2

# Stable trend range: between negative and positive thresholds
# Stable range: -0.2 to +0.2 (constitutional requirement)
```

### Oscillation Detection (MANDATORY)

```toml
[oscillation_detection]
# Oscillation detection enabled (constitutional requirement: true)
enabled = true

# Minimum direction changes to detect oscillation (constitutional minimum: 2)
min_direction_changes = 2

# Minimum amplitude for oscillation detection (constitutional minimum: 8.0)
min_amplitude = 10.0

# Period threshold for oscillation pattern (constitutional maximum: 10)
period_threshold = 8

# Oscillation risk levels (constitutional requirement)
risk_levels = ["None", "Low", "Medium", "High", "Critical"]
```

## 🚨 Constitutional Oscillation Rules

### Oscillation Risk Classification

**CONSTITUTIONAL IMPERATIVE**: Oscillation detection is **MANDATORY** and represents architectural panic, not statistical noise.

```toml
[oscillation_risk_rules]
# High risk criteria (constitutional minimums)
high_risk_direction_changes = 3
high_risk_amplitude = 15.0
high_risk_period = 6

# Critical risk criteria (constitutional minimums)
critical_risk_direction_changes = 4
critical_risk_amplitude = 20.0
critical_risk_waiver_correlation = true

# Oscillation + waiver increase = Critical risk (constitutional rule)
waiver_correlation_critical = true
```

### Oscillation CI Rules

```toml
[oscillation_ci_rules]
# Positive slope + oscillation = CI WARN (constitutional rule)
positive_slope_oscillation_warn = true

# Stable average + high amplitude = CI FAIL (constitutional rule)
stable_high_amplitude_fail = true

# Oscillation + waiver increase = CI FAIL (constitutional rule)
oscillation_waiver_increase_fail = true

# Oscillation 3 windows = CI FAIL (constitutional rule)
three_window_oscillation_fail = true
```

## 📈 CI Trend Validation Configuration

### Trend-Based CI Rules

```toml
[ci_trend_validation]
# Confidence gate threshold (constitutional minimum: 0.8)
confidence_gate_threshold = 0.8

# Negative trend CI fail enabled (constitutional requirement: true)
negative_trend_fail = true

# Debt acceleration detection enabled (constitutional requirement: true)
debt_acceleration_detection = true

# Sustained regression threshold in commits (constitutional maximum: 3)
sustained_regression_threshold = 3
```

### Trend Authority Rules

```toml
[trend_authority]
# Trend results are decision-forcing (constitutional requirement: true)
trend_decision_forcing = true

# Confidence below threshold ignores trend (constitutional requirement: true)
confidence_gate_enabled = true

# High confidence trend overrides snapshot (constitutional requirement: true)
trend_overrides_snapshot = true
```

## 🔍 Proactive Debt Detection

### Velocity-Based Detection

```toml
[velocity_detection]
# Velocity monitoring enabled (constitutional requirement: true)
enabled = true

# Velocity threshold for alerts (constitutional maximum: 0.1)
velocity_alert_threshold = 0.05

# Allow increase velocity threshold (constitutional maximum: 2.0)
allow_velocity_threshold = 1.0

# Critical rule emergence threshold (constitutional maximum: 1)
critical_rule_threshold = 1
```

### Early Warning System

```toml
[early_warning]
# Early warning enabled (constitutional requirement: true)
enabled = true

# Leading indicator sensitivity (constitutional minimum: 0.8)
leading_indicator_sensitivity = 0.9

# Pattern shift detection enabled (constitutional requirement: true)
pattern_shift_detection = true

# Wait-and-see mode prohibited (constitutional requirement: true)
wait_and_see_prohibited = true
```

## 📊 Dashboard and Reporting Configuration

### Executive Lie Prevention

```toml
[executive_reporting]
# Executive lie prevention enabled (constitutional requirement: true)
lie_prevention_enabled = true

# Current health alone prohibited (constitutional requirement: true)
current_health_alone_prohibited = true

# Trend vs snapshot conflict area required (constitutional requirement: true)
trend_snapshot_conflict_required = true

# Decision traceability required (constitutional requirement: true)
decision_traceability_required = true
```

### Reporting Requirements

```toml
[reporting_requirements]
# Minimum snapshot references in diagnostics (constitutional minimum: 3)
min_snapshot_references = 3

# Temporal context required (constitutional requirement: true)
temporal_context_required = true

# No soft language allowed (constitutional requirement: true)
no_soft_language = true

# Architectural regression language required (constitutional requirement: true)
architectural_regression_language = true
```

## 🎛️ Performance and Scalability

### Performance Configuration

```toml
[performance]
# Deterministic computation required (constitutional requirement: true)
deterministic_computation = true

# Canonical ordering required (constitutional requirement: true)
canonical_ordering = true

# Floating-point determinism locked (constitutional requirement: true)
floating_point_determinism_locked = true

# Memory growth prevention enabled (constitutional requirement: true)
memory_growth_prevention = true
```

### Caching Configuration

```toml
[caching]
# Deterministic caching enabled (constitutional requirement: true)
deterministic_caching = true

# Cache invalidation on trend change (constitutional requirement: true)
invalidate_on_trend_change = true

# Fixed-precision score hashing (constitutional requirement: true)
fixed_precision_hashing = true
```

## 🧪 Constitutional Testing Requirements

### Test Categories (MANDATORY)

```toml
[constitutional_tests]
# Constitutional drift tests required (constitutional requirement: true)
constitutional_drift_tests = true

# Good score bad trend fail test (constitutional requirement: true)
good_score_bad_trend_test = true

# Oscillation masked as stability test (constitutional requirement: true)
oscillation_stability_test = true

# Trend ignored by CI test (constitutional requirement: true)
trend_ignored_ci_test = true

# Trend exists no decision test (constitutional requirement: true)
trend_no_decision_test = true
```

### Oscillation Testing

```toml
[oscillation_tests]
# Positive slope + oscillation test (constitutional requirement: true)
positive_slope_oscillation_test = true

# Stable average + high amplitude test (constitutional requirement: true)
stable_high_amplitude_test = true

# Waiver correlation test (constitutional requirement: true)
waiver_correlation_test = true

# No crash test prevention (constitutional requirement: true)
no_crash_test_prevention = true
```

## 🔧 Project Customization (Constitutional Constraints)

### Allowed Customizations

Projects may **tighten** the following parameters:

```toml
[project_overrides]
# Example: Stricter confidence threshold
# confidence_threshold = 0.9  # Increased from 0.8

# Example: Smaller trend window for faster detection
# trend_window_size = 8  # Decreased from 10

# Example: Stricter oscillation detection
# min_amplitude = 8.0  # Decreased from 10.0

# Example: More sensitive velocity detection
# velocity_alert_threshold = 0.03  # Decreased from 0.05
```

### Forbidden Customizations

The following **CANNOT** be modified (constitutional constraints):

- `oscillation_detection.enabled` - Must remain `true`
- `trend_decision_forcing` - Must remain `true`
- `constitutional_drift_tests` - Must remain `true`
- `lie_prevention_enabled` - Must remain `true`
- `deterministic_computation` - Must remain `true`

## 📋 Configuration Validation

### Validation Rules

```toml
[validation]
# Constitutional constraint enforcement (cannot be disabled)
enforce_constitutional_constraints = true

# Softening prevention (cannot be disabled)
prevent_softening = true

# Dead control detection (cannot be disabled)
detect_dead_controls = true

# Configuration connection validation (cannot be disabled)
validate_config_connections = true
```

### Validation Errors

The system will reject configurations that:

- Soften constitutional requirements
- Disable mandatory features
- Create dead controls (unused config fields)
- Violate oscillation detection requirements
- Bypass trend authority rules

## 🎯 Usage Examples

### Tightening Configuration

```toml
# Example 1: Stricter trend detection
[trend_analysis]
confidence_threshold = 0.9  # Increased from constitutional minimum 0.8
trend_window_size = 8       # Decreased from default 10

# Example 2: More sensitive oscillation detection
[oscillation_detection]
min_amplitude = 8.0         # Decreased from default 10.0
min_direction_changes = 3   # Increased from constitutional minimum 2

# Example 3: Stricter CI validation
[ci_trend_validation]
sustained_regression_threshold = 2  # Decreased from constitutional maximum 3
confidence_gate_threshold = 0.9     # Increased from constitutional minimum 0.8
```

### Module-Specific Configuration

```toml
# Example: Kernel modules have stricter requirements
[module_overrides."kernel/**"]
confidence_threshold = 0.95
min_amplitude = 6.0
velocity_alert_threshold = 0.02

# Example: Userspace modules have standard requirements
[module_overrides."userspace/**"]
# Uses default constitutional minimums
```

## 📊 Monitoring and Alerting

### Alert Configuration

```toml
[alerts]
# Oscillation detection alerts (constitutional requirement: true)
oscillation_alerts = true

# Trend degradation alerts (constitutional requirement: true)
trend_degradation_alerts = true

# Velocity increase alerts (constitutional requirement: true)
velocity_alerts = true

# Constitutional violation alerts (constitutional requirement: true)
constitutional_violation_alerts = true
```

### Notification Channels

```toml
[notifications]
# Slack integration for trend alerts
slack_webhook = "https://hooks.slack.com/services/..."

# Email notifications for critical oscillations
email_alerts = ["architect@ayken.os", "team@ayken.os"]

# Dashboard updates for all trend changes
dashboard_updates = true
```

## 🔒 Constitutional Guarantee

**IMMUTABLE PRINCIPLES**:

1. **Oscillation Detection is Mandatory** - Cannot be disabled or bypassed
2. **Trend Authority is Binding** - Trend results force decisions
3. **Configuration Cannot Soften** - Only tightening is permitted
4. **Constitutional Tests Required** - All constitutional tests must pass
5. **Executive Lie Prevention** - Dashboard must show trend conflicts

**ENFORCEMENT**: These principles are enforced by the constitutional system and cannot be overridden by any configuration, exception, or authority.

**AUTHORITY**: Constitutional Steward (Kenan AY) - Final authority on AHTS constitutional requirements.

---

**IMPLEMENTATION NOTE**: This configuration system enforces the constitutional principle that "Trend never lies" by making oscillation detection mandatory and trend analysis binding for CI decisions.