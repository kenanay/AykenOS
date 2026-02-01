# Phase Matrix - Constitutional Authority

## 🎯 Phase Matrix Overview

The Phase Matrix serves as the foundational authority in the constitutional decision tree, determining base behavior for each rule across different development phases. This matrix is consulted BEFORE any exception mechanisms (Allow attributes or Waivers).

## 📊 Phase Matrix Table

| Rule Category | Rule ID | P4.4 (Dev) | P4.5 (Stab) | P5 (Prod) | Description |
|---------------|---------|------------|--------------|-----------|-------------|
| **DETERMINISM** | DETERMINISM.GLOBAL | ERROR | ERROR | ERROR | Global state mutations |
| | DETERMINISM.RNG | WARN | ERROR | ERROR | Non-seeded random generation |
| | DETERMINISM.TIME | ALLOW | WARN | ERROR | Non-deterministic time operations |
| **MEMORY** | MEMORY.CONTRACT.VIOLATION | ERROR | ERROR | ERROR | Memory safety violations |
| | MEMORY.LEAK | WARN | ERROR | ERROR | Memory leak patterns |
| | MEMORY.DOUBLE_FREE | ERROR | ERROR | ERROR | Double-free vulnerabilities |
| **ALLOC** | ALLOC.GLOBAL | ALLOW | WARN | ERROR | Global allocator usage |
| | ALLOC.HEAP_DIRECT | ALLOW | ALLOW | WARN | Direct heap allocation |
| | ALLOC.STACK_OVERFLOW | WARN | ERROR | ERROR | Stack overflow risk |
| **TIME** | TIME.INSTANT | ALLOW | WARN | ERROR | Direct time measurement |
| | TIME.SLEEP | WARN | WARN | ERROR | Blocking time operations |
| | TIME.TIMEOUT | ALLOW | ALLOW | WARN | Timeout-based logic |
| **ERROR** | ERROR.UNWRAP | WARN | WARN | ERROR | Panic-inducing unwrap |
| | ERROR.EXPECT | WARN | WARN | ERROR | Panic-inducing expect |
| | ERROR.PANIC | ERROR | ERROR | ERROR | Direct panic calls |
| **SECURITY** | SECURITY.BOUNDARY.VIOLATION | ERROR | ERROR | ERROR | Security boundary violations |
| | SECURITY.PRIVILEGE.ESCALATION | ERROR | ERROR | ERROR | Privilege escalation |
| | SECURITY.INFORMATION.LEAK | WARN | ERROR | ERROR | Information disclosure |
| **KERNEL** | KERNEL.SAFETY.CRITICAL | ERROR | ERROR | ERROR | Critical kernel safety |
| | KERNEL.RING0.POLICY | ERROR | ERROR | ERROR | Policy in Ring0 |
| | KERNEL.CAPABILITY.BYPASS | ERROR | ERROR | ERROR | Capability system bypass |
| **STYLE** | STYLE.FORMATTING | ALLOW | ALLOW | ALLOW | Code formatting issues |
| | STYLE.NAMING | ALLOW | ALLOW | WARN | Naming convention violations |
| | STYLE.DOCUMENTATION | ALLOW | WARN | WARN | Missing documentation |

## 🔄 Phase Definitions

### P4.4 - Development Phase
**Purpose**: Active development with flexibility for experimentation  
**Characteristics**:
- More permissive for non-critical violations
- Focus on functionality over strict compliance
- Allow temporary shortcuts for rapid prototyping
- Emphasis on developer productivity

**Behavior Patterns**:
- `ALLOW`: Violations are permitted without restriction
- `WARN`: Violations generate warnings but don't block
- `ERROR`: Violations block progress and require attention

### P4.5 - Stabilization Phase
**Purpose**: Code stabilization and quality improvement  
**Characteristics**:
- Increased restrictions as code matures
- Focus on architectural quality
- Preparation for production deployment
- Balance between flexibility and compliance

**Behavior Patterns**:
- Previous `ALLOW` → `WARN`: Increased scrutiny
- Previous `WARN` → `ERROR`: Stricter enforcement
- `ERROR` remains `ERROR`: No relaxation of critical rules

### P5 - Production Phase
**Purpose**: Production-ready code with maximum compliance  
**Characteristics**:
- Strictest enforcement of all rules
- Maximum architectural quality required
- No tolerance for technical debt
- Focus on reliability and maintainability

**Behavior Patterns**:
- Most violations become `ERROR`
- Only style violations remain `ALLOW` or `WARN`
- Critical violations always `ERROR` across all phases

## 🏛️ Constitutional Authority

### Foundational Role

The Phase Matrix serves as the **foundational authority** in the constitutional decision tree:

1. **First Consultation**: After NON_OVERRIDABLE check, Phase Matrix is consulted
2. **Base Behavior**: Determines the base behavior for each rule
3. **Exception Gateway**: Only `ERROR` results can be overridden by exceptions
4. **Progressive Hardening**: Automatically increases strictness over time

### Decision Flow Integration

```
NON_OVERRIDABLE Check (Absolute Gate)
    ↓
Phase Matrix Lookup (Foundational Authority)
    ↓
If ALLOW → Pass (No exceptions needed)
If WARN → Warning (No exceptions needed)
If ERROR → Check Exceptions (Allow/Waiver)
    ↓
Exception Resolution or Constitutional Violation
```

### Authority Hierarchy

1. **NON_OVERRIDABLE Rules** - Absolute authority (cannot be overridden)
2. **Phase Matrix** - Foundational authority (determines base behavior)
3. **Allow Attributes** - Exception authority (only for ERROR cases)
4. **Waiver System** - Bulk exception authority (only for ERROR cases)

## 📈 Progressive Hardening

### Automatic Progression

The Phase Matrix implements automatic progressive hardening:

- **P4.4 → P4.5**: Increased warnings, some allows become errors
- **P4.5 → P5**: Maximum strictness, most violations become errors
- **No Regression**: Phases cannot move backward in strictness

### Hardening Patterns

**Pattern 1: ALLOW → WARN → ERROR**
- P4.4: `ALLOW` (Permitted)
- P4.5: `WARN` (Warning issued)
- P5: `ERROR` (Blocks progress)

**Pattern 2: WARN → ERROR**
- P4.4: `WARN` (Warning issued)
- P4.5: `ERROR` (Blocks progress)
- P5: `ERROR` (Continues blocking)

**Pattern 3: ERROR (Constant)**
- P4.4: `ERROR` (Critical from start)
- P4.5: `ERROR` (Remains critical)
- P5: `ERROR` (Always critical)

## 🔧 Configuration and Customization

### Matrix Modification

The Phase Matrix can be customized with constitutional constraints:

**Allowed Modifications**:
- Making rules stricter (ALLOW → WARN → ERROR)
- Adding new rules with appropriate progression
- Customizing phase transition points

**Forbidden Modifications**:
- Making rules more permissive
- Removing existing rules
- Bypassing NON_OVERRIDABLE rules
- Creating inconsistent progressions

### Project-Specific Overrides

Projects can create project-specific phase matrices:

```toml
# ayken/steering/PROJECT_PHASES.toml
[phase_overrides]
"TIME.INSTANT" = { P4_4 = "WARN", P4_5 = "ERROR", P5 = "ERROR" }
"ALLOC.GLOBAL" = { P4_4 = "WARN", P4_5 = "ERROR", P5 = "ERROR" }
```

**Override Constraints**:
- Can only make rules stricter
- Must maintain progressive hardening
- Cannot override NON_OVERRIDABLE rules
- Must be constitutionally validated

## 🎯 Rule Categories Explained

### DETERMINISM Rules
Focus on predictable, reproducible behavior:
- **GLOBAL**: Global state breaks determinism
- **RNG**: Non-seeded randomness breaks reproducibility
- **TIME**: System time is non-deterministic

### MEMORY Rules
Focus on memory safety and management:
- **CONTRACT.VIOLATION**: Memory safety violations
- **LEAK**: Memory leaks compromise stability
- **DOUBLE_FREE**: Double-free causes undefined behavior

### ALLOC Rules
Focus on allocation patterns and performance:
- **GLOBAL**: Global allocator usage
- **HEAP_DIRECT**: Direct heap allocation
- **STACK_OVERFLOW**: Stack overflow risks

### TIME Rules
Focus on time-related operations:
- **INSTANT**: Direct time measurement
- **SLEEP**: Blocking time operations
- **TIMEOUT**: Timeout-based logic

### ERROR Rules
Focus on error handling patterns:
- **UNWRAP**: Panic-inducing unwrap operations
- **EXPECT**: Panic-inducing expect operations
- **PANIC**: Direct panic calls

### SECURITY Rules
Focus on security boundaries and safety:
- **BOUNDARY.VIOLATION**: Security boundary violations
- **PRIVILEGE.ESCALATION**: Privilege escalation attempts
- **INFORMATION.LEAK**: Information disclosure

### KERNEL Rules
Focus on kernel-specific safety:
- **SAFETY.CRITICAL**: Critical kernel safety violations
- **RING0.POLICY**: Policy decisions in Ring0
- **CAPABILITY.BYPASS**: Capability system bypasses

### STYLE Rules
Focus on code quality and maintainability:
- **FORMATTING**: Code formatting issues
- **NAMING**: Naming convention violations
- **DOCUMENTATION**: Missing documentation

## 📊 Monitoring and Analytics

### Phase Compliance Metrics

- **Compliance Rate**: Percentage of code compliant with current phase
- **Violation Distribution**: Distribution of violations by rule category
- **Progression Readiness**: Readiness for next phase transition
- **Exception Usage**: Usage of Allow/Waiver exceptions by phase

### Trend Analysis

- **Violation Trends**: Trends in violation counts over time
- **Phase Progression**: Historical phase progression patterns
- **Exception Patterns**: Patterns in exception usage
- **Quality Metrics**: Code quality trends by phase

### Reporting

- **Phase Reports**: Current phase compliance status
- **Progression Reports**: Readiness for phase advancement
- **Violation Reports**: Detailed violation analysis
- **Exception Reports**: Exception usage and trends

## 🚀 Implementation Notes

### Matrix Storage

The Phase Matrix is stored in structured format:

```rust
pub struct PhaseMatrix {
    rules: HashMap<RuleId, PhaseRule>,
    current_phase: Phase,
    progression_history: Vec<PhaseTransition>,
}

pub struct PhaseRule {
    p4_4: PhaseBehavior,
    p4_5: PhaseBehavior,
    p5: PhaseBehavior,
}

pub enum PhaseBehavior {
    Allow,
    Warn,
    Error,
}
```

### Lookup Algorithm

```rust
impl PhaseMatrix {
    pub fn lookup(&self, rule_id: &RuleId) -> PhaseBehavior {
        let rule = self.rules.get(rule_id)?;
        match self.current_phase {
            Phase::P4_4 => rule.p4_4,
            Phase::P4_5 => rule.p4_5,
            Phase::P5 => rule.p5,
        }
    }
}
```

### Integration Points

- **Constitutional Decision Tree**: Primary consumer of Phase Matrix
- **CLI Commands**: `ayken phase` commands for phase management
- **CI Integration**: Automatic phase detection and enforcement
- **VS Code Integration**: Real-time phase-aware diagnostics

---

**CONSTITUTIONAL AUTHORITY**: The Phase Matrix is the foundational authority for constitutional behavior. All exception mechanisms must respect Phase Matrix decisions.

**IMPLEMENTATION GUARANTEE**: The Phase Matrix is consulted before any exception mechanisms and its decisions cannot be bypassed by configuration or customization.