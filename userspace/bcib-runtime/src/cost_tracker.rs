/// Cost accounting for BCIB v3 execution slices.
///
/// Implements per-context instruction cost tracking with separate accounting
/// for External (AI/UI) instructions (Requirements 17.1, 17.2, 17.3).
///
/// # Cost Constants
///
/// | Class         | Cost |
/// |---------------|------|
/// | Pure          |    1 |
/// | DataMutating  |   10 |
/// | External      |  100 |
///
/// These constants enforce the invariant `pure < data-mutating < external`,
/// which reflects the relative resource cost of each side-effect class.

use crate::types::{BcibError, CostUnit, COST_DATA_MUTATING, COST_EXTERNAL, COST_PURE};

// Re-export constants so callers can import them from this module.
pub use crate::types::{COST_DATA_MUTATING as DATA_MUTATING, COST_EXTERNAL as EXTERNAL, COST_PURE as PURE};

/// Tracks instruction cost consumption for a single execution context.
///
/// `CostTracker` is created once per `ExecutionContext` and lives for the
/// entire lifetime of that context. It enforces two independent budgets:
///
/// 1. **Main budget** (`total` / `remaining`): covers all instructions.
///    `charge()` deducts from `remaining`; when `remaining` reaches zero
///    the slice must yield (`Running → Yielded`).
///
/// 2. **External budget** (`external_budget` / `external_used`): covers
///    only `External` (AI/UI) instructions. `charge_external()` deducts
///    from this budget independently. Exhausting the external budget is a
///    hard fail (`BCIB_ERR_RESOURCE_EXHAUSTED`), not a yield.
///
/// # Invariants
///
/// - `remaining` never goes below zero (saturating semantics).
/// - `external_used` never exceeds `external_budget`.
/// - Both budgets are independent; exhausting one does not affect the other.
#[derive(Debug, Clone, Default)]
pub struct CostTracker {
    /// Total cost budget for this context (set at creation, never mutated).
    pub total: CostUnit,
    /// Remaining cost budget. Decremented by `charge()`.
    /// Reaches zero when the slice must yield.
    pub remaining: CostUnit,
    /// Cumulative cost consumed by External instructions so far.
    pub external_used: CostUnit,
    /// Maximum cost budget reserved for External (AI/UI) instructions.
    pub external_budget: CostUnit,
}

impl CostTracker {
    /// Create a new `CostTracker` with the given budgets.
    ///
    /// - `total`: main cost budget for the execution context.
    /// - `external_budget`: separate budget for External instructions.
    pub fn new(total: CostUnit, external_budget: CostUnit) -> Self {
        Self {
            total,
            remaining: total,
            external_used: 0,
            external_budget,
        }
    }

    /// Deduct `cost` from the main budget (Requirement 17.2).
    ///
    /// If `cost` exceeds `remaining`, `remaining` is set to zero and
    /// `Err(ResourceExhausted)` is returned. The caller (run_slice) treats
    /// this as a yield signal (`Running → Yielded`), not a hard failure.
    ///
    /// # Errors
    ///
    /// Returns `BCIB_ERR_RESOURCE_EXHAUSTED` when the main budget is exhausted.
    pub fn charge(&mut self, cost: CostUnit) -> Result<(), BcibError> {
        if cost > self.remaining {
            self.remaining = 0;
            Err(BcibError::ResourceExhausted("cost budget exhausted"))
        } else {
            self.remaining -= cost;
            Ok(())
        }
    }

    /// Deduct `cost` from the external budget (Requirement 17.3).
    ///
    /// External instructions (AI/UI) are accounted separately to prevent
    /// a flood of cheap Pure instructions from masking external resource
    /// exhaustion. Exhausting the external budget is a hard fail.
    ///
    /// # Errors
    ///
    /// Returns `BCIB_ERR_RESOURCE_EXHAUSTED` when the external budget is exhausted.
    pub fn charge_external(&mut self, cost: CostUnit) -> Result<(), BcibError> {
        let available = self.external_budget.saturating_sub(self.external_used);
        if cost > available {
            Err(BcibError::ResourceExhausted("external cost budget exhausted"))
        } else {
            self.external_used += cost;
            Ok(())
        }
    }

    /// Returns `true` when the main budget is fully consumed.
    pub fn is_exhausted(&self) -> bool {
        self.remaining == 0
    }

    /// Returns the remaining main budget.
    pub fn remaining(&self) -> CostUnit {
        self.remaining
    }

    /// Returns the remaining external budget.
    pub fn external_remaining(&self) -> CostUnit {
        self.external_budget.saturating_sub(self.external_used)
    }
}

// ---------------------------------------------------------------------------
// Tests (Requirements 17.1, 17.2, 17.3)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -----------------------------------------------------------------------
    // Cost constant ordering invariant (Requirement 17.1)
    // -----------------------------------------------------------------------

    #[test]
    fn cost_constants_ordering() {
        assert!(COST_PURE < COST_DATA_MUTATING, "pure must be cheaper than data-mutating");
        assert!(COST_DATA_MUTATING < COST_EXTERNAL, "data-mutating must be cheaper than external");
    }

    #[test]
    fn cost_constants_values() {
        assert_eq!(COST_PURE, 1);
        assert_eq!(COST_DATA_MUTATING, 10);
        assert_eq!(COST_EXTERNAL, 100);
    }

    // -----------------------------------------------------------------------
    // charge() — main budget (Requirement 17.2)
    // -----------------------------------------------------------------------

    #[test]
    fn charge_deducts_from_remaining() {
        let mut tracker = CostTracker::new(100, 50);
        tracker.charge(COST_PURE).unwrap();
        assert_eq!(tracker.remaining, 99);
    }

    #[test]
    fn charge_multiple_deductions() {
        let mut tracker = CostTracker::new(100, 50);
        tracker.charge(COST_PURE).unwrap();          // -1  → 99
        tracker.charge(COST_DATA_MUTATING).unwrap(); // -10 → 89
        assert_eq!(tracker.remaining, 89);
    }

    #[test]
    fn charge_exact_budget_exhaustion_returns_ok() {
        let mut tracker = CostTracker::new(10, 50);
        // Exactly exhaust the budget.
        assert!(tracker.charge(10).is_ok());
        assert_eq!(tracker.remaining, 0);
        assert!(tracker.is_exhausted());
    }

    #[test]
    fn charge_over_budget_returns_err_and_zeroes_remaining() {
        let mut tracker = CostTracker::new(5, 50);
        let result = tracker.charge(10);
        assert!(result.is_err());
        assert_eq!(tracker.remaining, 0, "remaining must be zeroed on over-budget charge");
        assert!(tracker.is_exhausted());
    }

    #[test]
    fn charge_over_budget_error_is_resource_exhausted() {
        let mut tracker = CostTracker::new(5, 50);
        let err = tracker.charge(10).unwrap_err();
        assert!(
            matches!(err, BcibError::ResourceExhausted(_)),
            "expected ResourceExhausted, got {:?}",
            err
        );
    }

    #[test]
    fn charge_remaining_never_underflows() {
        let mut tracker = CostTracker::new(1, 50);
        // First charge exhausts the budget.
        let _ = tracker.charge(1);
        // Second charge on zero budget must not underflow.
        let result = tracker.charge(1);
        assert!(result.is_err());
        assert_eq!(tracker.remaining, 0);
    }

    // -----------------------------------------------------------------------
    // charge_external() — external budget (Requirement 17.3)
    // -----------------------------------------------------------------------

    #[test]
    fn charge_external_deducts_from_external_used() {
        let mut tracker = CostTracker::new(1000, 200);
        tracker.charge_external(COST_EXTERNAL).unwrap();
        assert_eq!(tracker.external_used, 100);
    }

    #[test]
    fn charge_external_multiple_deductions() {
        let mut tracker = CostTracker::new(1000, 300);
        tracker.charge_external(COST_EXTERNAL).unwrap(); // 100
        tracker.charge_external(COST_EXTERNAL).unwrap(); // 200
        assert_eq!(tracker.external_used, 200);
        assert_eq!(tracker.external_remaining(), 100);
    }

    #[test]
    fn charge_external_exact_budget_exhaustion_returns_ok() {
        let mut tracker = CostTracker::new(1000, 100);
        assert!(tracker.charge_external(100).is_ok());
        assert_eq!(tracker.external_remaining(), 0);
    }

    #[test]
    fn charge_external_over_budget_returns_err() {
        let mut tracker = CostTracker::new(1000, 50);
        let result = tracker.charge_external(COST_EXTERNAL); // 100 > 50
        assert!(result.is_err());
    }

    #[test]
    fn charge_external_over_budget_error_is_resource_exhausted() {
        let mut tracker = CostTracker::new(1000, 50);
        let err = tracker.charge_external(COST_EXTERNAL).unwrap_err();
        assert!(
            matches!(err, BcibError::ResourceExhausted(_)),
            "expected ResourceExhausted, got {:?}",
            err
        );
    }

    #[test]
    fn charge_external_does_not_affect_main_remaining() {
        let mut tracker = CostTracker::new(100, 200);
        tracker.charge_external(COST_EXTERNAL).unwrap();
        // Main budget must be untouched.
        assert_eq!(tracker.remaining, 100);
    }

    #[test]
    fn charge_main_does_not_affect_external_used() {
        let mut tracker = CostTracker::new(100, 200);
        tracker.charge(COST_PURE).unwrap();
        // External accounting must be untouched.
        assert_eq!(tracker.external_used, 0);
    }

    // -----------------------------------------------------------------------
    // is_exhausted() / remaining() / external_remaining()
    // -----------------------------------------------------------------------

    #[test]
    fn is_exhausted_false_when_budget_available() {
        let tracker = CostTracker::new(100, 50);
        assert!(!tracker.is_exhausted());
    }

    #[test]
    fn is_exhausted_true_when_remaining_zero() {
        let mut tracker = CostTracker::new(1, 50);
        tracker.charge(1).unwrap();
        assert!(tracker.is_exhausted());
    }

    #[test]
    fn remaining_accessor_matches_field() {
        let tracker = CostTracker::new(42, 10);
        assert_eq!(tracker.remaining(), 42);
    }

    #[test]
    fn external_remaining_decreases_with_charges() {
        let mut tracker = CostTracker::new(1000, 300);
        assert_eq!(tracker.external_remaining(), 300);
        tracker.charge_external(100).unwrap();
        assert_eq!(tracker.external_remaining(), 200);
        tracker.charge_external(100).unwrap();
        assert_eq!(tracker.external_remaining(), 100);
    }

    // -----------------------------------------------------------------------
    // Default / new constructors
    // -----------------------------------------------------------------------

    #[test]
    fn new_sets_remaining_equal_to_total() {
        let tracker = CostTracker::new(500, 100);
        assert_eq!(tracker.total, 500);
        assert_eq!(tracker.remaining, 500);
        assert_eq!(tracker.external_budget, 100);
        assert_eq!(tracker.external_used, 0);
    }

    #[test]
    fn default_is_zero_budget() {
        let tracker = CostTracker::default();
        assert_eq!(tracker.total, 0);
        assert_eq!(tracker.remaining, 0);
        assert_eq!(tracker.external_budget, 0);
        assert_eq!(tracker.external_used, 0);
    }
}
