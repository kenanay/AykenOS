# D4 Constitutional Policy Engine

**IMPORTANT**: This crate does not execute enforcement. It defines constitutional policy, authority hierarchy, and violation semantics. Runtime enforcement is delegated to higher layers.

## Architecture Mode: B (Policy Specification + Contract Generator)

This crate implements the constitutional **policy framework** as a specification and contract generator rather than runtime enforcement.

### What This Crate Does ✅
- Defines constitutional policy and authority hierarchy
- Validates operations against policy specifications  
- Generates violation reports and compliance contracts
- Creates property tests for constitutional compliance
- Specifies semantic lock policies (not runtime locks)
- Validates Gate E transition readiness through testing

### What This Crate Does NOT Do ❌
- Block runtime operations
- Kill threads or abort processes
- Rollback JIT compilation or allocation commits
- Enforce policies at runtime (that's for higher layers)

## Correct AykenOS Architecture

```
🟦 d4-constitutional (This Crate)
Role: Constitutional Policy Engine
- Authority hierarchy specification
- Policy validation and violation detection
- Contract generation for Gate transitions
- Specification compliance testing

🟥 d4-runtime-enforcement (Separate Crate)  
Role: Runtime Policy Enforcement
- Thread termination and process isolation
- JIT pipeline interruption
- Allocation rollback and cache disabling
- Executes SystemResponse decisions from policy engine
```

## 🔒 Constitutional Lock Status

**CRITICAL**: Several modules in this crate are under **PERMANENT CONSTITUTIONAL LOCK**:

- 🔒 **`bmode/register_invariants/`** - Register allocation analysis (LOCKED)
- 🔒 **`bmode/integration/`** - Integration orchestration pipeline (LOCKED)  
- 🔒 **`bmode/reports.rs`** - B-MODE reporting extensions (LOCKED)

**Changes to locked modules require Constitutional RFC approval.**

See `CONSTITUTIONAL_LOCK_MANIFEST.md` for complete governance details.

## Directory Structure

```
d4-constitutional/
├── src/
│   ├── lib.rs                          # Main library entry point
│   ├── types.rs                        # Core constitutional types
│   ├── testing.rs                      # Property testing framework
│   │
│   ├── bmode/                          # 🔒 B-MODE Core (Partially Locked)
│   │   ├── mod.rs                      # B-MODE module orchestration
│   │   ├── constitutional.rs           # Constitutional rule analysis
│   │   ├── contracts.rs                # Contract specification
│   │   ├── determinism.rs              # Deterministic behavior analysis
│   │   ├── failure_matrix.rs           # Failure pattern analysis
│   │   ├── reports.rs                  # 🔒 B-MODE report extensions (LOCKED)
│   │   ├── semantic_spec_catalog.rs    # Semantic specification catalog
│   │   ├── templates.rs                # Template system analysis
│   │   ├── tests.rs                    # B-MODE compliance tests
│   │   ├── types.rs                    # B-MODE specific types
│   │   ├── validation_location.rs      # Validation location tracking
│   │   │
│   │   ├── register_invariants/        # 🔒 Register Analysis (LOCKED)
│   │   │   ├── mod.rs                  # 🔒 Single entry point
│   │   │   ├── uniqueness.rs           # 🔒 Allocation uniqueness analysis
│   │   │   ├── conflicts.rs            # 🔒 Enhanced conflict analysis
│   │   │   ├── spill_analysis.rs       # 🔒 Spill overhead analysis
│   │   │   └── README.md               # 🔒 Constitutional documentation
│   │   │
│   │   ├── integration/                # 🔒 Integration Pipeline (LOCKED)
│   │   │   ├── mod.rs                  # 🔒 Public API orchestration
│   │   │   ├── pipeline.rs             # 🔒 Pure orchestration pipeline
│   │   │   ├── template_pass.rs        # 🔒 Template analysis pass
│   │   │   ├── compliance_pass.rs      # 🔒 Compliance integration pass
│   │   │   ├── gate_pass.rs            # 🔒 Gate readiness analysis pass
│   │   │   └── README.md               # 🔒 Integration documentation
│   │   │
│   │   └── reports/                    # 🔒 B-MODE Reports (LOCKED)
│   │       └── README.md               # 🔒 Report documentation
│   │
│   ├── errors/                         # Error and Report Framework
│   │   ├── mod.rs                      # Error module orchestration
│   │   ├── framework_error.rs          # Constitutional framework errors
│   │   └── specification_reports.rs    # Specification reporting system
│   │
│   ├── runtime/                        # Runtime Integration (Empty)
│   │   └── (reserved for future runtime integration)
│   │
│   ├── build_fingerprint.rs            # Build fingerprinting system
│   ├── compliance.rs                   # Compliance analysis engine
│   ├── gate_readiness.rs               # Gate readiness validation
│   ├── integration_tests.rs            # Integration test suite
│   ├── jit_bounds.rs                   # JIT boundary analysis
│   │
│   └── *_property_tests.rs             # Property-based test suites
│       ├── bmode_purity_property_tests.rs
│       ├── build_fingerprint_property_tests.rs
│       ├── compliance_property_tests.rs
│       └── error_type_property_tests.rs
│
├── proptest-regressions/               # Property test regression data
│   ├── constitutional.txt
│   ├── determinism.txt
│   ├── error_type_property_tests.txt
│   ├── jit_bounds.txt
│   └── register_invariants.txt
│
├── Cargo.toml                          # Crate configuration
├── README.md                           # This file
└── test_ci_deterministic.sh            # CI deterministic testing script
```

## 🔒 Constitutional Lock Modules

### Register Invariants (`bmode/register_invariants/`) - LOCKED 🔒
- **Purpose**: Pure register allocation analysis without enforcement
- **Status**: Permanent constitutional lock - RFC required for changes
- **Capabilities**: Uniqueness analysis, conflict detection, spill overhead analysis
- **Entry Point**: `analyze_register_invariants()` unified API

### Integration Pipeline (`bmode/integration/`) - LOCKED 🔒  
- **Purpose**: Pure orchestration of constitutional analysis passes
- **Status**: Permanent constitutional lock - RFC required for changes
- **Capabilities**: Template analysis, compliance integration, gate readiness
- **Entry Point**: `ConstitutionalIntegrationPipeline` orchestration

### B-MODE Reports (`bmode/reports.rs`) - LOCKED 🔒
- **Purpose**: B-MODE specific reporting extensions and compliance analysis
- **Status**: Permanent constitutional lock - RFC required for changes
- **Capabilities**: Immutable report builders, compliance assessment, recommendations
- **Safety**: Strict f64 usage rules, no Eq/Ord derivation

## Usage

```rust
use d4_constitutional::ConstitutionalFramework;

let framework = ConstitutionalFramework::new()?;

// Policy validation (generates reports, doesn't block)
let result = framework.validate_operation(&operation, component);
match result {
    Ok(_) => println!("Operation complies with constitutional policy"),
    Err(violation) => println!("Policy violation detected: {}", violation),
}

// Contract generation for Gate transitions
let contract = framework.generate_integrated_implementation_contracts(component)?;
```

### 🔒 Using Locked Modules

```rust
// Register invariants analysis (constitutional lock protected)
use d4_constitutional::bmode::register_invariants::analyze_register_invariants;

let report = analyze_register_invariants(&allocations);
// Analysis only - never enforces allocation decisions

// Integration pipeline (constitutional lock protected)  
use d4_constitutional::bmode::integration::ConstitutionalIntegrationPipeline;

let pipeline = ConstitutionalIntegrationPipeline::new(component_id);
let analysis = pipeline.analyze_constitutional_compliance(&context);
// Orchestration only - never enforces compliance decisions

// B-MODE reports (constitutional lock protected)
use d4_constitutional::bmode::reports::{BModeSpecificationReport, analyze_bmode_compliance};

let bmode_report = analyze_bmode_compliance(specification_report);
// Reporting only - never enforces B-MODE compliance
```

## Testing

```bash
# Run all tests
cargo test

# Run B-MODE specific tests  
cargo test bmode

# Run property-based tests
cargo test property

# Run deterministic CI tests
./test_ci_deterministic.sh
```

## Constitutional Governance

This crate follows strict constitutional governance:

- **Locked Modules**: Require Constitutional RFC for any changes
- **B-MODE Purity**: Analysis only, never enforcement
- **Deterministic Behavior**: Same input always produces same output
- **Academic Quality**: Publication-ready implementations
- **Industrial Grade**: Production compiler infrastructure ready

See `CONSTITUTIONAL_LOCK_MANIFEST.md` for complete governance details and RFC process.

---

This is the correct B-mode architecture for AykenOS constitutional framework.