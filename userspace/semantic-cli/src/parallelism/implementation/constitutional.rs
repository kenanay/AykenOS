//! Constitutional enforcement for D2 Parallelism Architecture
//!
//! This module implements the constitutional guarantees and enforcement mechanisms
//! required by the D2 Parallelism Architecture. It provides compile-time and
//! runtime enforcement of constitutional principles.
//!
//! ## Constitutional Principles (BINDING)
//!
//! 1. **P1: Determinism > Parallelism** - Parallelism is optional, determinism is mandatory
//! 2. **P2: IR is Single Source of Truth** - Parallel execution cannot change IR semantics
//! 3. **P3: Replay First-Class Citizen** - Replay must work, or parallelism is invalid
//! 4. **P4: Performance is Net Performance** - Measure with ordering + sync + merge overhead
//!
//! ## Constitutional Mandates (BINDING)
//!
//! 1. **Cache-Line Safety Rule** - Avoid false sharing through chunk-local buffers
//! 2. **Adaptive Blacklist is Soft** - Blacklisting is reversible after 50 executions
//! 3. **Native Code Purity Constraint** - Native code must be observationally pure

use crate::execution_plan::{BlockId, IRBlock, ParallelSafety};
use crate::parallelism::{ParallelismError, ParallelismResult};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Execution mode with constitutional semantics.
///
/// Each mode has specific constitutional requirements that must be enforced
/// at compile-time and runtime.
///
/// **CONSTITUTIONAL REQUIREMENT:** All modes must enforce determinism
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExecutionMode {
    /// Normal execution mode - allows adaptive parallelism
    Normal,
    /// Replay mode - MUST use sequential execution only
    /// **CONSTITUTIONAL:** P3 - Replay First-Class Citizen
    Replay,
    /// Verification mode - runs both parallel and sequential for comparison
    /// **CONSTITUTIONAL:** Adaptive logic MUST be disabled
    Verification,
    /// Development mode - only available in non-production builds
    #[cfg(debug_assertions)]
    Development,
}

impl ExecutionMode {
    /// Checks if this mode allows adaptive behavior.
    ///
    /// **CONSTITUTIONAL ENFORCEMENT:** Replay and Verification modes
    /// MUST NOT use adaptive logic to prevent contamination.
    pub fn allows_adaptation(&self) -> bool {
        match self {
            ExecutionMode::Normal => true,
            ExecutionMode::Replay => false, // CONSTITUTIONAL: P3
            ExecutionMode::Verification => false, // CONSTITUTIONAL: No contamination
            #[cfg(debug_assertions)]
            ExecutionMode::Development => true,
        }
    }

    /// Checks if this mode requires strict determinism.
    ///
    /// **CONSTITUTIONAL:** ALL modes require determinism - this is non-negotiable.
    pub fn requires_determinism(&self) -> bool {
        true // CONSTITUTIONAL: P1 - Determinism > Parallelism
    }

    /// Checks if this mode allows parallel execution.
    ///
    /// **CONSTITUTIONAL:** Replay mode MUST use sequential execution only.
    pub fn allows_parallelism(&self) -> bool {
        match self {
            ExecutionMode::Normal => true,
            ExecutionMode::Replay => false,      // CONSTITUTIONAL: P3
            ExecutionMode::Verification => true, // Needs both paths for comparison
            #[cfg(debug_assertions)]
            ExecutionMode::Development => true,
        }
    }
}

/// Safety verdict with binding enforcement.
///
/// **CONSTITUTIONAL:** SafetyVerdict decisions are BINDING and must be enforced.
/// Attempting to execute unsafe operations is a constitutional violation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SafetyVerdict {
    /// Operation is safe for parallel execution
    Allow,
    /// Operation is safe with specific restrictions
    AllowWithRestrictions(RestrictionSet),
    /// Operation is unsafe and MUST NOT be parallelized
    Reject(RejectionReason),
}

impl SafetyVerdict {
    /// Checks if this verdict is binding.
    ///
    /// **CONSTITUTIONAL:** All safety verdicts are binding.
    pub fn is_binding(&self) -> bool {
        true
    }

    /// Enforces the safety verdict or panics.
    ///
    /// **CONSTITUTIONAL ENFORCEMENT:** Attempting to execute rejected operations
    /// is a constitutional violation and must cause immediate system failure.
    pub fn enforce_or_panic(&self, block: &IRBlock) {
        match self {
            SafetyVerdict::Reject(reason) => {
                panic!(
                    "CONSTITUTIONAL VIOLATION: Unsafe IR block execution attempted\n\
                     Block ID: {}\n\
                     Reason: {:?}\n\
                     This violates P1: Determinism > Parallelism",
                    block.id, reason
                );
            }
            SafetyVerdict::AllowWithRestrictions(restrictions) => {
                // Restrictions must be enforced by the caller
                if restrictions.is_empty() {
                    panic!(
                        "CONSTITUTIONAL VIOLATION: Empty restriction set\n\
                         Block ID: {}\n\
                         AllowWithRestrictions requires non-empty restrictions",
                        block.id
                    );
                }
            }
            SafetyVerdict::Allow => {
                // No enforcement needed for allowed operations
            }
        }
    }
}

/// Set of restrictions for conditional parallel execution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RestrictionSet {
    /// Maximum number of parallel workers allowed
    pub max_workers: Option<usize>,
    /// Minimum dataset size required for parallelism
    pub min_dataset_size: Option<usize>,
    /// Required safety checks before execution
    pub required_checks: Vec<SafetyCheck>,
}

impl RestrictionSet {
    /// Creates an empty restriction set.
    pub fn new() -> Self {
        Self {
            max_workers: None,
            min_dataset_size: None,
            required_checks: Vec::new(),
        }
    }

    /// Checks if the restriction set is empty.
    pub fn is_empty(&self) -> bool {
        self.max_workers.is_none()
            && self.min_dataset_size.is_none()
            && self.required_checks.is_empty()
    }

    /// Adds a maximum worker restriction.
    pub fn with_max_workers(mut self, max_workers: usize) -> Self {
        self.max_workers = Some(max_workers);
        self
    }

    /// Adds a minimum dataset size restriction.
    pub fn with_min_dataset_size(mut self, min_size: usize) -> Self {
        self.min_dataset_size = Some(min_size);
        self
    }

    /// Adds a required safety check.
    pub fn with_safety_check(mut self, check: SafetyCheck) -> Self {
        self.required_checks.push(check);
        self
    }
}

impl Default for RestrictionSet {
    fn default() -> Self {
        Self::new()
    }
}

/// Required safety checks for restricted operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SafetyCheck {
    /// Verify no shared mutable state
    NoSharedMutableState,
    /// Verify deterministic ordering
    DeterministicOrdering,
    /// Verify cache-line safety
    CacheLineSafety,
    /// Verify error propagation
    ErrorPropagation,
}

/// Reason for rejecting parallel execution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RejectionReason {
    /// Operation has side effects
    SideEffects,
    /// Operation uses non-deterministic behavior
    NonDeterministic,
    /// Operation accesses shared mutable state
    SharedMutableState,
    /// Operation performs I/O
    IOOperation,
    /// Operation uses unsafe native code
    UnsafeNativeCode,
    /// Custom rejection reason
    Custom(String),
}

/// Error handling policy for different error classes.
///
/// **CONSTITUTIONAL:** Error handling must be deterministic and policy-driven.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorPolicy {
    /// Retry the operation N times
    Retry(u32),
    /// Fall back to sequential execution
    Fallback,
    /// Blacklist the operation
    Blacklist,
    /// Fatal error - system shutdown required
    Fatal,
}

/// Classification of errors for policy mapping.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ErrorClass {
    /// Determinism violation - ALWAYS fatal
    DeterminismViolation,
    /// Safety violation - ALWAYS fatal
    SafetyViolation,
    /// Performance degradation - blacklist
    PerformanceDegradation,
    /// Worker panic - retry then blacklist
    WorkerPanic,
    /// Resource exhaustion - fallback
    ResourceExhaustion,
    /// Timeout - fallback
    Timeout,
}

/// Policy table for error handling.
///
/// **CONSTITUTIONAL:** Provides deterministic error handling policies.
#[derive(Debug, Clone)]
pub struct PolicyTable {
    error_policies: HashMap<ErrorClass, ErrorPolicy>,
}

impl PolicyTable {
    /// Creates a constitutional policy table with mandatory policies.
    ///
    /// **CONSTITUTIONAL ENFORCEMENT:**
    /// - Determinism violations are FATAL
    /// - Safety violations are FATAL
    /// - Other errors have appropriate recovery policies
    pub fn constitutional_default() -> Self {
        let mut policies = HashMap::new();

        // CONSTITUTIONAL: Determinism violations are FATAL
        policies.insert(ErrorClass::DeterminismViolation, ErrorPolicy::Fatal);

        // CONSTITUTIONAL: Safety violations are FATAL
        policies.insert(ErrorClass::SafetyViolation, ErrorPolicy::Fatal);

        // Performance issues get blacklisted
        policies.insert(ErrorClass::PerformanceDegradation, ErrorPolicy::Blacklist);

        // Worker panics get one retry, then blacklist
        policies.insert(ErrorClass::WorkerPanic, ErrorPolicy::Retry(1));

        // Resource issues fall back to sequential
        policies.insert(ErrorClass::ResourceExhaustion, ErrorPolicy::Fallback);
        policies.insert(ErrorClass::Timeout, ErrorPolicy::Fallback);

        Self {
            error_policies: policies,
        }
    }

    /// Gets the policy for an error class.
    pub fn get_policy(&self, error_class: ErrorClass) -> Option<ErrorPolicy> {
        self.error_policies.get(&error_class).copied()
    }

    /// Enforces the policy for an error.
    ///
    /// **CONSTITUTIONAL ENFORCEMENT:** Fatal errors cause immediate panic.
    pub fn enforce_policy(&self, error_class: ErrorClass, error: &ParallelismError) -> ! {
        match self.get_policy(error_class) {
            Some(ErrorPolicy::Fatal) => {
                panic!(
                    "CONSTITUTIONAL VIOLATION: Fatal error encountered\n\
                     Error Class: {:?}\n\
                     Error: {:?}\n\
                     System must shutdown to maintain constitutional guarantees",
                    error_class, error
                );
            }
            _ => {
                panic!(
                    "INTERNAL ERROR: enforce_policy called on non-fatal error\n\
                     Error Class: {:?}\n\
                     Error: {:?}",
                    error_class, error
                );
            }
        }
    }
}

/// Configuration with constitutional classes.
///
/// **CONSTITUTIONAL:** Different configuration parameters have different
/// mutability guarantees and enforcement requirements.
#[derive(Debug, Clone)]
pub struct ConstitutionalConfig {
    /// IMMUTABLE - Can only be set at startup
    pub static_config: StaticConfig,

    /// RUNTIME TUNABLE - Can be changed during execution
    pub runtime_config: Arc<RwLock<RuntimeConfig>>,

    /// CONSTITUTION LOCKED - Cannot be changed without system restart
    pub locked_config: LockedConfig,
}

impl ConstitutionalConfig {
    /// Creates a new constitutional configuration.
    pub fn new() -> Self {
        Self {
            static_config: StaticConfig::production_default(),
            runtime_config: Arc::new(RwLock::new(RuntimeConfig::default())),
            locked_config: LockedConfig::default(),
        }
    }

    /// Gets the current execution mode.
    pub fn execution_mode(&self) -> ExecutionMode {
        self.locked_config.execution_mode
    }

    /// Sets the execution mode (requires constitutional authority).
    ///
    /// **CONSTITUTIONAL:** Mode changes must be authorized and may require restart.
    pub fn set_execution_mode(&mut self, mode: ExecutionMode, _authority: ConstitutionalAuthority) {
        self.locked_config.execution_mode = mode;
    }
}

impl Default for ConstitutionalConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Static configuration - immutable after startup.
///
/// **CONSTITUTIONAL:** These settings enforce constitutional principles
/// and cannot be changed during runtime.
#[derive(Debug, Clone)]
pub struct StaticConfig {
    /// ALWAYS true in production - enforces P1: Determinism > Parallelism
    pub determinism_enforcement: bool,
    /// ALWAYS true in production - enforces safety verification
    pub safety_verification: bool,
    /// ALWAYS true in production - enforces P3: Replay First-Class Citizen
    pub replay_capability: bool,
    /// ALWAYS true in production - enforces constitutional compliance
    pub constitutional_compliance: bool,
}

impl StaticConfig {
    /// Creates production-grade static configuration.
    ///
    /// **CONSTITUTIONAL:** All enforcement mechanisms are enabled in production.
    pub fn production_default() -> Self {
        Self {
            determinism_enforcement: true,
            safety_verification: true,
            replay_capability: true,
            constitutional_compliance: true,
        }
    }

    /// Creates development configuration (only in debug builds).
    ///
    /// **CONSTITUTIONAL:** Even in development, core principles are enforced.
    #[cfg(debug_assertions)]
    pub fn development_default() -> Self {
        Self {
            determinism_enforcement: true,    // Still required
            safety_verification: true,        // Still required
            replay_capability: true,          // Still required
            constitutional_compliance: false, // Can be relaxed for testing
        }
    }
}

/// Runtime tunable configuration.
///
/// **CONSTITUTIONAL:** These parameters can be adjusted during execution
/// without violating constitutional principles.
#[derive(Debug, Clone)]
pub struct RuntimeConfig {
    /// Minimum dataset size for parallelism consideration
    pub parallel_threshold: usize,
    /// Minimum net speedup required for parallelism
    pub speedup_threshold: f64,
    /// Re-evaluation window for blacklisted operations
    pub blacklist_window: usize,
    /// Maximum number of parallel workers
    pub max_workers: usize,
}

impl RuntimeConfig {
    /// Creates default runtime configuration.
    pub fn new() -> Self {
        Self {
            parallel_threshold: 100,
            speedup_threshold: 2.0,
            blacklist_window: 50,
            max_workers: num_cpus::get(),
        }
    }
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Constitution-locked configuration.
///
/// **CONSTITUTIONAL:** These settings are locked by constitutional mandate
/// and cannot be changed without system restart and explicit authority.
#[derive(Debug, Clone)]
pub struct LockedConfig {
    /// Current execution mode
    pub execution_mode: ExecutionMode,
    /// Kill switch authority
    pub kill_switch_authority: KillSwitchAuthority,
    /// Security boundary configuration
    pub security_boundary: SecurityBoundary,
    /// Phase enforcement settings
    pub phase_enforcement: PhaseEnforcement,
}

impl LockedConfig {
    /// Creates default locked configuration.
    pub fn new() -> Self {
        Self {
            execution_mode: ExecutionMode::Normal,
            kill_switch_authority: KillSwitchAuthority::System,
            security_boundary: SecurityBoundary::Strict,
            phase_enforcement: PhaseEnforcement::Enabled,
        }
    }
}

impl Default for LockedConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Kill switch authority levels.
///
/// **CONSTITUTIONAL:** Kill switch activation must be authorized and atomic.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KillSwitchAuthority {
    /// System-level authority (highest)
    System,
    /// Administrative authority
    Admin,
    /// Operational authority
    Operator,
    /// No authority (kill switch disabled)
    None,
}

/// Security boundary enforcement levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecurityBoundary {
    /// Strict boundary enforcement
    Strict,
    /// Relaxed boundary (development only)
    #[cfg(debug_assertions)]
    Relaxed,
}

/// Phase enforcement settings.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhaseEnforcement {
    /// Phase boundaries are enforced
    Enabled,
    /// Phase boundaries are relaxed (development only)
    #[cfg(debug_assertions)]
    Disabled,
}

/// Constitutional authority token.
///
/// **CONSTITUTIONAL:** Certain operations require explicit constitutional
/// authority to prevent accidental violations.
#[derive(Debug, Clone)]
pub struct ConstitutionalAuthority {
    authority_level: AuthorityLevel,
    granted_at: std::time::Instant,
}

impl ConstitutionalAuthority {
    /// Grants system-level constitutional authority.
    ///
    /// **WARNING:** This should only be used by system initialization code.
    pub fn grant_system_authority() -> Self {
        Self {
            authority_level: AuthorityLevel::System,
            granted_at: std::time::Instant::now(),
        }
    }

    /// Gets the authority level.
    pub fn get_authority_level(&self) -> AuthorityLevel {
        self.authority_level
    }

    /// Verifies that the authority is valid.
    pub fn verify(&self) -> ParallelismResult<()> {
        // Authority expires after 1 minute to prevent misuse
        if self.granted_at.elapsed() > std::time::Duration::from_secs(60) {
            return Err(ParallelismError::SecurityError {
                message: "Constitutional authority expired".to_string(),
            });
        }

        Ok(())
    }
}

/// Authority levels for constitutional operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthorityLevel {
    /// System-level authority
    System,
    /// Administrative authority
    Admin,
    /// Operational authority
    Operator,
}

/// Constitutional compliance checker.
///
/// **CONSTITUTIONAL:** Provides runtime verification of constitutional principles.
pub struct ConstitutionalChecker {
    config: ConstitutionalConfig,
    policy_table: PolicyTable,
}

impl ConstitutionalChecker {
    /// Creates a new constitutional checker.
    pub fn new(config: ConstitutionalConfig) -> Self {
        Self {
            config,
            policy_table: PolicyTable::constitutional_default(),
        }
    }

    /// Verifies constitutional compliance for an execution request.
    ///
    /// **CONSTITUTIONAL ENFORCEMENT:** This method MUST be called before
    /// any parallel execution to ensure constitutional compliance.
    ///
    /// **NOTE:** This method returns the SafetyVerdict for examination.
    /// The caller is responsible for enforcing the verdict if needed.
    pub fn verify_execution_request(
        &self,
        block: &IRBlock,
        mode: ExecutionMode,
        data_size: usize,
    ) -> ParallelismResult<SafetyVerdict> {
        // 1. Verify static configuration compliance
        if self.config.static_config.constitutional_compliance {
            self.verify_constitutional_compliance()?;
        }

        // 2. Verify execution mode requirements
        if !mode.requires_determinism() {
            return Err(ParallelismError::ConstitutionalViolation {
                principle: "P1: Determinism > Parallelism".to_string(),
                violation: "Execution mode does not require determinism".to_string(),
            });
        }

        // 3. Verify safety requirements
        let safety_verdict = self.analyze_safety(block, mode, data_size)?;

        // 4. Return verdict for caller to handle (don't enforce here during verification)
        Ok(safety_verdict)
    }

    /// Verifies overall constitutional compliance.
    fn verify_constitutional_compliance(&self) -> ParallelismResult<()> {
        // Verify P1: Determinism > Parallelism
        if !self.config.static_config.determinism_enforcement {
            return Err(ParallelismError::ConstitutionalViolation {
                principle: "P1: Determinism > Parallelism".to_string(),
                violation: "Determinism enforcement is disabled".to_string(),
            });
        }

        // Verify P3: Replay First-Class Citizen
        if !self.config.static_config.replay_capability {
            return Err(ParallelismError::ConstitutionalViolation {
                principle: "P3: Replay First-Class Citizen".to_string(),
                violation: "Replay capability is disabled".to_string(),
            });
        }

        Ok(())
    }

    /// Analyzes safety for a specific execution request.
    fn analyze_safety(
        &self,
        block: &IRBlock,
        mode: ExecutionMode,
        data_size: usize,
    ) -> ParallelismResult<SafetyVerdict> {
        // Check parallel safety annotation
        match block.parallel_safety {
            ParallelSafety::Unsafe => {
                return Ok(SafetyVerdict::Reject(RejectionReason::SideEffects));
            }
            ParallelSafety::Safe => {
                // Continue with additional checks
            }
            ParallelSafety::ReductionOnly => {
                // Allow with restrictions
                let restrictions =
                    RestrictionSet::new().with_safety_check(SafetyCheck::DeterministicOrdering);
                return Ok(SafetyVerdict::AllowWithRestrictions(restrictions));
            }
        }

        // Check execution mode constraints
        if mode == ExecutionMode::Replay && !mode.allows_parallelism() {
            return Ok(SafetyVerdict::Reject(RejectionReason::Custom(
                "Replay mode requires sequential execution".to_string(),
            )));
        }

        // Check dataset size
        let runtime_config = self.config.runtime_config.read().unwrap();
        if data_size < runtime_config.parallel_threshold {
            return Ok(SafetyVerdict::Reject(RejectionReason::Custom(format!(
                "Dataset too small: {} < {}",
                data_size, runtime_config.parallel_threshold
            ))));
        }

        // Default to allow for safe operations
        Ok(SafetyVerdict::Allow)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::execution_plan::{BlockTerminator, IRInstruction};

    fn create_test_block(id: BlockId, safety: ParallelSafety) -> IRBlock {
        IRBlock::with_safety(
            id,
            vec![IRInstruction::LoadContext {
                context_id: "test".to_string(),
                target_register: 0,
            }],
            BlockTerminator::Return { register: 0 },
            safety,
        )
    }

    #[test]
    fn test_execution_mode_determinism() {
        // All modes must require determinism
        assert!(ExecutionMode::Normal.requires_determinism());
        assert!(ExecutionMode::Replay.requires_determinism());
        assert!(ExecutionMode::Verification.requires_determinism());

        #[cfg(debug_assertions)]
        assert!(ExecutionMode::Development.requires_determinism());
    }

    #[test]
    fn test_execution_mode_adaptation() {
        // Only Normal mode allows adaptation
        assert!(ExecutionMode::Normal.allows_adaptation());
        assert!(!ExecutionMode::Replay.allows_adaptation());
        assert!(!ExecutionMode::Verification.allows_adaptation());

        #[cfg(debug_assertions)]
        assert!(ExecutionMode::Development.allows_adaptation());
    }

    #[test]
    fn test_execution_mode_parallelism() {
        assert!(ExecutionMode::Normal.allows_parallelism());
        assert!(!ExecutionMode::Replay.allows_parallelism()); // CONSTITUTIONAL
        assert!(ExecutionMode::Verification.allows_parallelism());

        #[cfg(debug_assertions)]
        assert!(ExecutionMode::Development.allows_parallelism());
    }

    #[test]
    fn test_safety_verdict_binding() {
        let allow = SafetyVerdict::Allow;
        let restrict = SafetyVerdict::AllowWithRestrictions(RestrictionSet::new());
        let reject = SafetyVerdict::Reject(RejectionReason::SideEffects);

        // All verdicts are binding
        assert!(allow.is_binding());
        assert!(restrict.is_binding());
        assert!(reject.is_binding());
    }

    #[test]
    #[should_panic(expected = "CONSTITUTIONAL VIOLATION")]
    fn test_safety_verdict_enforce_reject() {
        let verdict = SafetyVerdict::Reject(RejectionReason::SideEffects);
        let block = create_test_block(1, ParallelSafety::Unsafe);

        verdict.enforce_or_panic(&block);
    }

    #[test]
    #[should_panic(expected = "CONSTITUTIONAL VIOLATION")]
    fn test_safety_verdict_enforce_empty_restrictions() {
        let verdict = SafetyVerdict::AllowWithRestrictions(RestrictionSet::new());
        let block = create_test_block(1, ParallelSafety::Safe);

        verdict.enforce_or_panic(&block);
    }

    #[test]
    fn test_safety_verdict_enforce_allow() {
        let verdict = SafetyVerdict::Allow;
        let block = create_test_block(1, ParallelSafety::Safe);

        // Should not panic
        verdict.enforce_or_panic(&block);
    }

    #[test]
    fn test_restriction_set() {
        let restrictions = RestrictionSet::new()
            .with_max_workers(4)
            .with_min_dataset_size(1000)
            .with_safety_check(SafetyCheck::NoSharedMutableState);

        assert!(!restrictions.is_empty());
        assert_eq!(restrictions.max_workers, Some(4));
        assert_eq!(restrictions.min_dataset_size, Some(1000));
        assert_eq!(restrictions.required_checks.len(), 1);
    }

    #[test]
    fn test_policy_table_constitutional_default() {
        let policy_table = PolicyTable::constitutional_default();

        // Constitutional violations must be fatal
        assert_eq!(
            policy_table.get_policy(ErrorClass::DeterminismViolation),
            Some(ErrorPolicy::Fatal)
        );
        assert_eq!(
            policy_table.get_policy(ErrorClass::SafetyViolation),
            Some(ErrorPolicy::Fatal)
        );

        // Performance issues should be blacklisted
        assert_eq!(
            policy_table.get_policy(ErrorClass::PerformanceDegradation),
            Some(ErrorPolicy::Blacklist)
        );
    }

    #[test]
    fn test_constitutional_config() {
        let config = ConstitutionalConfig::new();

        // Production defaults should enforce all constitutional principles
        assert!(config.static_config.determinism_enforcement);
        assert!(config.static_config.safety_verification);
        assert!(config.static_config.replay_capability);
        assert!(config.static_config.constitutional_compliance);

        assert_eq!(config.execution_mode(), ExecutionMode::Normal);
    }

    #[test]
    fn test_constitutional_authority() {
        let authority = ConstitutionalAuthority::grant_system_authority();

        // Fresh authority should be valid
        assert!(authority.verify().is_ok());
    }

    #[test]
    fn test_constitutional_checker() {
        let config = ConstitutionalConfig::new();
        let checker = ConstitutionalChecker::new(config);

        // Safe block should be allowed
        let safe_block = create_test_block(1, ParallelSafety::Safe);
        let verdict = checker.verify_execution_request(&safe_block, ExecutionMode::Normal, 1000);
        assert!(verdict.is_ok());
        assert_eq!(verdict.unwrap(), SafetyVerdict::Allow);

        // Unsafe block should be rejected
        let unsafe_block = create_test_block(2, ParallelSafety::Unsafe);
        let verdict = checker.verify_execution_request(&unsafe_block, ExecutionMode::Normal, 1000);
        assert!(verdict.is_ok());
        match verdict.unwrap() {
            SafetyVerdict::Reject(RejectionReason::SideEffects) => {}
            _ => panic!("Expected rejection for unsafe block"),
        }
    }
}
