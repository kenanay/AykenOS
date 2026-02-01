# AykenOS Constitutional Rule System - Design Specification

## 🎯 System Overview

The AykenOS Constitutional Rule System is a comprehensive architectural governance framework that enforces constitutional principles through a hierarchical decision tree with exception mechanisms and continuous improvement loops.

## 🏛️ Constitutional Philosophy

### Core Principles

1. **"İstisna = bilinçli karar"** (Exception = conscious decision)
   - Every exception must be explicitly documented and justified
   - No silent failures or implicit bypasses

2. **"İyi mimari → istisnasız mimaridir"** (Good architecture → exception-free architecture)
   - The goal is to eliminate exceptions through architectural improvement
   - Exceptions are symptoms, refactoring is the cure

3. **"Tek snapshot yalan söyler. Trend asla yalan söylemez."** (Single snapshot lies. Trend never lies.)
   - Architectural health is measured over time, not at single points
   - Trend analysis reveals true architectural direction

4. **"Mimari sorunlar lokaldir, bedeli küresel olur"** (Architectural problems are local, cost is global)
   - Local violations compound into system-wide architectural debt
   - Module-level governance prevents global degradation

## 🔄 System Architecture

### Decision Flow Hierarchy

```
NON_OVERRIDABLE Rules (Absolute Gate)
    ↓
Phase Matrix (Foundational Authority)
    ↓
Allow Attributes (ERROR cases only)
    ↓
Waiver System (ERROR cases only)
    ↓
Constitutional Violation (FAIL)
```

### Component Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Constitutional System                     │
├─────────────────────────────────────────────────────────────┤
│  Phase 1-2: Core Infrastructure & Rule System              │
│  ├── Diagnostic System (VS Code, CI, CLI)                  │
│  ├── Rule Registry (NON_OVERRIDABLE, Constitutional Core)  │
│  └── Phase Matrix (Foundational Authority)                 │
├─────────────────────────────────────────────────────────────┤
│  Phase 3-4: Exception Mechanisms & Decision Tree           │
│  ├── Allow Attributes (9 Architectural Classes)            │
│  ├── Waiver System (TOML-based, Expiry Management)         │
│  └── Constitutional Decision Tree (Unified Processing)     │
├─────────────────────────────────────────────────────────────┤
│  Phase 5: Developer Experience                             │
│  ├── CLI Commands (check, explain, audit)                  │
│  ├── VS Code Integration (Diagnostics, Quick Fixes)        │
│  └── Steering Files Management                             │
├─────────────────────────────────────────────────────────────┤
│  Phase 6-7: Waiver Lifecycle & Renewal                     │
│  ├── Three-Layer Aging (Time, Usage, Phase)                │
│  ├── CI Progressive Hardening                              │
│  ├── Mandatory PR Templates                                │
│  └── Renewal Counter & Limits                              │
├─────────────────────────────────────────────────────────────┤
│  Phase 8: Architecture Health Score (AHS)                  │
│  ├── Weighted Scoring (Rule Weights × Phase Multipliers)   │
│  ├── CI Regression Gates                                   │
│  └── VS Code Health Indicators                             │
├─────────────────────────────────────────────────────────────┤
│  Phase 9: Architecture Health Time-Series (AHTS)           │
│  ├── Trend Analysis (Linear Regression + Oscillation)      │
│  ├── Proactive Debt Detection                              │
│  └── CI Trend-Based Validation                             │
├─────────────────────────────────────────────────────────────┤
│  Phase 10: Module-level Architecture Risk Score (MARS)     │
│  ├── Module Boundary Detection                             │
│  ├── Risk Classification (Healthy → Quarantine)            │
│  └── Cross-Module Risk Analysis                            │
├─────────────────────────────────────────────────────────────┤
│  Phase 11: Allow → Refactor Recommendation Engine (ARRE)   │
│  ├── Refactor Pattern Classification                       │
│  ├── Age-Based Triggering                                  │
│  └── Architectural Debt Explosion Detection                │
├─────────────────────────────────────────────────────────────┤
│  Phase 12: Auto-Refactor Hints (ARH)                       │
│  ├── Safe Autofix Engine                                   │
│  ├── Assisted Fix with Preview                             │
│  └── Design Hint & Architectural Guidance                  │
└─────────────────────────────────────────────────────────────┘
```

## 📊 Rule Classification System

### Rule Weight Hierarchy

1. **DETERMINISM** (Weight: 5.0) - Highest Priority
   - `DETERMINISM.GLOBAL` - Global state mutations
   - `DETERMINISM.RNG` - Non-seeded random number generation
   - `DETERMINISM.TIME` - Non-deterministic time operations

2. **MEMORY/ALLOC** (Weight: 4.0) - Critical
   - `ALLOC.GLOBAL` - Global allocator usage
   - `MEMORY.CONTRACT.VIOLATION` - Memory safety violations
   - `MEMORY.LEAK` - Memory leak patterns

3. **TIME** (Weight: 4.0) - Critical
   - `TIME.INSTANT` - Direct time measurement
   - `TIME.SLEEP` - Blocking time operations
   - `TIME.TIMEOUT` - Timeout-based logic

4. **ERROR** (Weight: 3.0) - Important
   - `ERROR.UNWRAP` - Panic-inducing unwrap operations
   - `ERROR.EXPECT` - Panic-inducing expect operations
   - `ERROR.PANIC` - Direct panic calls

5. **STYLE** (Weight: 1.0) - Lowest Priority
   - Code formatting and style violations

### Architectural Allow Classes

1. **BootstrapRuntime** - System initialization code
2. **BenchmarkMeasurementOnly** - Performance measurement code
3. **TestingInfrastructure** - Test-specific code
4. **LegacyCompatibility** - Legacy system integration
5. **ExternalIntegration** - Third-party system integration
6. **PerformanceCritical** - Performance-optimized code
7. **PlatformSpecific** - Platform-specific implementations
8. **TemporaryWorkaround** - Temporary solutions
9. **DebugDiagnostic** - Debug and diagnostic code

## 🔒 Constitutional Guarantees

### NON_OVERRIDABLE Rules

These rules cannot be bypassed by any exception mechanism:

- **DETERMINISM.GLOBAL** - Global state mutations are forbidden
- **MEMORY.CONTRACT.VIOLATION** - Memory safety must be maintained
- **KERNEL.SAFETY.CRITICAL** - Kernel safety cannot be compromised
- **SECURITY.BOUNDARY.VIOLATION** - Security boundaries are inviolable

### Phase Matrix Authority

The Phase Matrix serves as the foundational authority that determines base behavior:

- **P4.4**: Development phase - More permissive
- **P4.5**: Stabilization phase - Moderate restrictions
- **P5**: Production phase - Strict enforcement

### Exception Hierarchy

1. **NON_OVERRIDABLE** - Absolute gate (no exceptions)
2. **Phase Matrix** - Foundational authority
3. **Allow Attributes** - Only for ERROR cases
4. **Waiver System** - Only for ERROR cases with no Allow
5. **Constitutional Violation** - Final failure state

## 📈 Health Scoring System

### Architecture Health Score (AHS)

Formula: `100 - Σ(rule_weight × phase_multiplier × (1 + age_penalty) + renewal_penalty)`

- **Rule Weights**: DETERMINISM (5.0), ALLOC (4.0), TIME (3.0), ERROR (2.5), STYLE (1.0)
- **Phase Multipliers**: P4.4 (×1.0), P4.5 (×1.3), P5 (×2.0)
- **Age Penalty**: `days_since_creation / 30`
- **Renewal Penalty**: `renewal_count × 1.5`

### Health Levels

- **90-100**: Clean - No restrictions
- **75-89**: Warning - Increased monitoring
- **60-74**: Degraded - Refactoring required
- **40-59**: Critical - Immediate intervention
- **0-39**: Quarantine - Cannot progress

### CI Thresholds

- **P4.4**: Minimum AHS 85
- **P4.5**: Minimum AHS 90
- **P5**: Minimum AHS 95

## 🔄 Lifecycle Management

### Allow → Waiver → Refactor Cycle

1. **Allow Phase**: Temporary exception with expiry
2. **Waiver Phase**: Bulk exception with usage tracking
3. **Refactor Phase**: Architectural improvement

### Waiver Expiry System

Three-layer aging mechanism:

1. **Time-based**: Hard expiry dates
2. **Usage-based**: Maximum usage counts
3. **Phase-based**: Automatic invalidation on phase progression

### Renewal Process

- **Maximum 3 renewals** per waiver
- **Progressive expiry shortening** (25% reduction per renewal)
- **Mandatory PR templates** for renewal justification
- **Approval authority matrix** based on phase

## 🎯 Continuous Improvement

### Trend Analysis

- **Linear regression** for architectural health trajectory
- **Oscillation detection** for architectural panic patterns
- **Confidence intervals** for statistical significance
- **Predictive analysis** for future health projection

### Refactor Recommendations

Six canonical refactor classes:

1. **ClockAbstraction** - For TIME violations
2. **ErrorBoundary** - For ERROR violations
3. **MemoryArena** - For ALLOC violations
4. **SeededExecution** - For DETERMINISM violations
5. **StateParameterization** - For global state issues
6. **TypedDecision** - For decision logic improvements

### Auto-Refactor Hints (ARH)

Three automation levels:

1. **SafeAutofix** - 95%+ confidence, fully automated
2. **AssistedFix** - 60-90% confidence, human approval required
3. **DesignHint** - <60% confidence, architectural guidance only

## 🛡️ Security & Safety

### Trust Boundaries

- **Kernel Code**: Maximum restrictions, DesignHint only
- **Userspace Code**: Moderate restrictions, AssistedFix allowed
- **Tooling Code**: Minimum restrictions, SafeAutofix allowed

### Risk Assessment

- **Local Impact**: Function-level changes
- **Module Impact**: Module-level changes
- **Cross-Module Impact**: System-wide changes
- **Architectural Impact**: Fundamental design changes

### Safety Guarantees

- **No silent failures** - All violations are reported
- **Immutable audit trail** - All decisions are logged
- **Constitutional compliance** - Core principles cannot be violated
- **Deterministic behavior** - Same input produces same output

## 📁 File Structure

```
ayken/
├── diagnostic/           # Phase 1: Core diagnostic system
├── rules/               # Phase 2: Rule registry and management
├── phase/               # Phase 2: Phase detection and matrix
├── allow/               # Phase 3: Allow attribute system
├── waiver/              # Phase 3: Waiver management system
├── exception/           # Phase 3: Exception hierarchy
├── decision/            # Phase 4: Constitutional decision tree
├── cli/                 # Phase 5: Command-line interface
├── explain/             # Phase 5: Educational content system
├── audit/               # Phase 5: Audit trail system
├── vscode/              # Phase 5: VS Code integration
├── steering/            # Phase 5: Configuration management
├── ahs/                 # Phase 8: Architecture Health Score
├── ahts/                # Phase 9: Health Time-Series analysis
├── mars/                # Phase 10: Module-level risk scoring
├── arre/                # Phase 11: Refactor recommendations
└── arh/                 # Phase 12: Auto-refactor hints
```

## 🔧 Configuration System

### Steering Files

- `ayken/steering/NON_OVERRIDABLE.md` - Immutable constitutional rules
- `ayken/steering/CLASSES.md` - Allow class definitions
- `ayken/steering/PHASES.md` - Phase matrix configuration
- `ayken/steering/AHS_CONFIG.toml` - Health scoring configuration
- `ayken/steering/AHTS_CONFIG.md` - Trend analysis configuration
- `ayken/steering/MARS_CONFIG.md` - Module risk configuration
- `ayken/steering/ARRE_CONFIG.md` - Refactor recommendation configuration
- `ayken/steering/ARH_CONFIG.md` - Auto-refactor hints configuration

### Configuration Principles

- **Constitutional constraints** - Core principles cannot be softened
- **Tightening only** - Configuration can only make rules stricter
- **Single source of truth** - Steering files are authoritative
- **Validation required** - All configuration changes are validated

## 🧪 Testing Strategy

### Test Categories

1. **Unit Tests** - Individual component testing
2. **Integration Tests** - Cross-component workflows
3. **Property Tests** - Mathematical invariants
4. **Constitutional Tests** - Core principle enforcement
5. **Performance Tests** - Scalability and speed
6. **Regression Tests** - Prevent quality degradation

### Test Coverage Requirements

- **>90% code coverage** for all modules
- **350+ tests passing** across all phases
- **Property-based testing** for critical algorithms
- **Constitutional drift prevention** tests

## 📊 Metrics & Monitoring

### Key Performance Indicators

- **Architecture Health Score** - Overall system health
- **Exception Count** - Number of active exceptions
- **Refactor Completion Rate** - Improvement velocity
- **CI Failure Rate** - Constitutional compliance
- **Trend Direction** - Architectural trajectory

### Monitoring Dashboards

- **Executive Dashboard** - High-level health overview
- **Developer Dashboard** - Detailed violation analysis
- **Module Dashboard** - Component-level risk assessment
- **Trend Dashboard** - Historical analysis and predictions

## 🚀 Deployment & Integration

### CI/CD Integration

- **Pre-commit hooks** - Early violation detection
- **Pull request validation** - Constitutional compliance checking
- **Build pipeline integration** - Automated health assessment
- **Deployment gates** - Health threshold enforcement

### IDE Integration

- **VS Code extension** - Real-time violation detection
- **Quick fixes** - Automated improvement suggestions
- **Diagnostic panels** - Detailed violation analysis
- **Educational content** - Learning and improvement guidance

## 📚 Documentation & Training

### User Documentation

- **Getting Started Guide** - Quick setup and basic usage
- **Developer Guide** - Comprehensive system documentation
- **Configuration Reference** - Complete configuration options
- **Troubleshooting Guide** - Common issues and solutions

### Training Materials

- **Constitutional Principles** - Core philosophy and concepts
- **Best Practices** - Recommended usage patterns
- **Case Studies** - Real-world implementation examples
- **Migration Guide** - Transitioning from existing tools

---

This design specification serves as the authoritative reference for the AykenOS Constitutional Rule System implementation. All development must adhere to these principles and architectural decisions.