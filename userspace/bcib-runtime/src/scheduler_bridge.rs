/// Scheduler_Submit_Bridge — Layer 3 of the three-layer v3 architecture.
///
/// Responsibilities (Requirements 1.2, 1.7):
///   - Forward BCIB graphs to Ring0 via `SYS_V2_SUBMIT_EXECUTION (1003)`.
///   - Manage yield/resume signaling with the kernel scheduler.
///   - Track starvation and emit diagnostic signals (NOT scheduling decisions).
///
/// SHALL NOT:
///   - Make scheduling decisions.
///   - Change execution order.
///   - Drop execution requests.
///   - Carry policy semantics into Ring0.
///
/// This module communicates with other layers exclusively through the types
/// defined in `types.rs`. No cross-layer implementation dependencies.
use std::collections::HashMap;

use crate::types::{BcibError, ExecutionContextId};

// ---------------------------------------------------------------------------
// Syscall constants — ABI freeze (Requirements 1.2, 1.7)
// ---------------------------------------------------------------------------

/// Syscall base offset for v2 interface (1000-1014 range, ABI freeze).
const SYS_V2_BASE: u64 = 1000;

/// Submit a BCIB graph to Ring0 for execution.
/// ABI freeze: this number MUST NOT change (Requirement 1.2).
const SYS_V2_SUBMIT_EXECUTION: u64 = SYS_V2_BASE + 3; // 1003

/// Wait for an execution result from Ring0.
/// ABI freeze: this number MUST NOT change.
const SYS_V2_WAIT_RESULT: u64 = SYS_V2_BASE + 4; // 1004

// ---------------------------------------------------------------------------
// Signal / protocol constants
// ---------------------------------------------------------------------------

/// `arg4` value passed to `SYS_V2_SUBMIT_EXECUTION` to indicate a yield
/// signal rather than a graph submission. The kernel distinguishes these by
/// inspecting arg4; a zero arg4 means graph submission.
const YIELD_SIGNAL: u64 = 1;

/// Sentinel returned by `SYS_V2_WAIT_RESULT` when the context has not yet
/// been resumed (scheduler is still deciding). The bridge continues polling.
/// Value 0 is chosen because a successful resume returns a positive execution
/// status; negative values are errors.
const WAIT_RESULT_PENDING: i64 = 0;

/// Per-poll timeout passed to `SYS_V2_WAIT_RESULT` in `await_resume()`.
/// Keeps each poll bounded so the starvation counter can advance.
const RESUME_POLL_TIMEOUT_MS: u64 = 10;

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// Opaque execution ID returned by Ring0 after a successful submission.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExecutionId(pub u64);

/// Result returned by Ring0 after execution completes.
#[derive(Debug, Clone)]
pub struct ExecutionResult {
    pub execution_id: ExecutionId,
    pub status: u64,
}

// ---------------------------------------------------------------------------
// SchedulerSubmitBridge
// ---------------------------------------------------------------------------

/// Starvation threshold — number of consecutive slices a context may remain
/// in Yielded/Waiting state without being resumed before a diagnostic signal
/// is emitted. This is a configurable constant; the default is 10 slices.
///
/// Emitting the signal is NOT a scheduling decision — it is a diagnostic event
/// only. The scheduler retains full authority over when to resume the context
/// (Requirements 2.8, 2.9, 20.3, 20.4).
pub const STARVATION_THRESHOLD: u32 = 10;

/// Bridges BCIB execution requests to the Ring0 kernel scheduler.
///
/// Tracks per-context starvation counters. When a context has been waiting
/// for `starvation_threshold` consecutive slices without being resumed, a
/// diagnostic log is emitted. This is NOT a scheduling decision.
///
/// SHALL NOT:
///   - Make scheduling decisions.
///   - Change execution order.
///   - Drop execution requests.
///   - Carry policy semantics into Ring0.
pub struct SchedulerSubmitBridge {
    /// Per-context starvation counter.
    ///
    /// Incremented on each `yield_slice()` call for a context.
    /// Reset to 0 on `await_resume()` when the context is successfully resumed.
    /// When the counter reaches `starvation_threshold`, a diagnostic log is
    /// emitted — NOT a scheduling decision (Requirements 2.8, 2.9, 20.3, 20.4).
    starvation_counters: HashMap<ExecutionContextId, u32>,
    /// Number of consecutive un-resumed slices before a diagnostic signal is emitted.
    starvation_threshold: u32,
}

impl SchedulerSubmitBridge {
    pub fn new() -> Self {
        Self {
            starvation_counters: HashMap::new(),
            starvation_threshold: STARVATION_THRESHOLD,
        }
    }

    /// Construct with a custom starvation threshold (useful for tests).
    pub fn with_threshold(threshold: u32) -> Self {
        Self {
            starvation_counters: HashMap::new(),
            starvation_threshold: threshold,
        }
    }

    /// Returns the current starvation counter for a context (tests only).
    #[cfg(test)]
    pub(crate) fn starvation_count(&self, ctx_id: ExecutionContextId) -> u32 {
        self.starvation_counters.get(&ctx_id).copied().unwrap_or(0)
    }

    // -----------------------------------------------------------------------
    // Public API
    // -----------------------------------------------------------------------

    /// Submit a BCIB graph to Ring0 via `SYS_V2_SUBMIT_EXECUTION (1003)`.
    ///
    /// Returns the kernel-assigned `ExecutionId` on success.
    /// Bridge does NOT make execution decisions — it only forwards the request.
    pub fn submit(
        &self,
        graph_ptr: *const u8,
        graph_len: usize,
        context_id: ExecutionContextId,
    ) -> Result<ExecutionId, BcibError> {
        let ret = unsafe {
            syscall_v2(
                SYS_V2_SUBMIT_EXECUTION,
                graph_ptr as u64,
                graph_len as u64,
                context_id,
                0, // arg4 = 0 → graph submission (not a yield signal)
            )
        };
        if (ret as i64) <= 0 {
            return Err(BcibError::SchedulerBridgeFail(
                "submit: SYS_V2_SUBMIT_EXECUTION returned error or zero",
            ));
        }
        Ok(ExecutionId(ret))
    }

    /// Wait for an execution result via `SYS_V2_WAIT_RESULT (1004)`.
    pub fn wait_result(
        &self,
        execution_id: ExecutionId,
        timeout_ms: u64,
    ) -> Result<ExecutionResult, BcibError> {
        let ret = unsafe { syscall_v2(SYS_V2_WAIT_RESULT, execution_id.0, timeout_ms, 0, 0) };
        if (ret as i64) < 0 {
            return Err(BcibError::SchedulerBridgeFail(
                "wait_result: SYS_V2_WAIT_RESULT returned error",
            ));
        }
        Ok(ExecutionResult {
            execution_id,
            status: ret,
        })
    }

    /// Emit a yield signal — voluntarily release the CPU slice back to the scheduler.
    ///
    /// Invokes `SYS_V2_SUBMIT_EXECUTION (1003)` with `arg4 = YIELD_SIGNAL` to
    /// signal intent to yield. The scheduler decides when to resume; BCIB only
    /// signals intent and does NOT make scheduling decisions (Requirements 1.7, 2.9).
    ///
    /// Also increments the per-context starvation counter. If the counter reaches
    /// `starvation_threshold`, a diagnostic log is emitted (NOT a scheduling
    /// decision — Requirements 2.8, 20.3, 20.4).
    ///
    /// Returns `BCIB_ERR_SCHEDULER_BRIDGE_FAIL` if the syscall returns a
    /// negative value (kernel rejected the signal). The context state transition
    /// (Running → Yielded) is performed by `BcibExecutionRuntime` before this
    /// call; a bridge failure causes fail-closed execution termination
    /// (Requirement 2.7).
    pub fn yield_slice(&mut self, ctx_id: ExecutionContextId) -> Result<(), BcibError> {
        // arg1 = ctx_id (identifies which context is yielding)
        // arg2 = 0      (no graph pointer — this is a signal, not a submission)
        // arg3 = 0      (no graph length)
        // arg4 = YIELD_SIGNAL (distinguishes yield signal from graph submission)
        let ret = unsafe { syscall_v2(SYS_V2_SUBMIT_EXECUTION, ctx_id, 0, 0, YIELD_SIGNAL) };
        if (ret as i64) < 0 {
            return Err(BcibError::SchedulerBridgeFail(
                "yield_slice: SYS_V2_SUBMIT_EXECUTION returned error",
            ));
        }

        // Increment the starvation counter for this context.
        // This tracks how many consecutive slices the context has yielded
        // without being resumed. When the threshold is reached, a diagnostic
        // signal is emitted — NOT a scheduling decision (Requirements 2.8, 20.3, 20.4).
        let counter = self.starvation_counters.entry(ctx_id).or_insert(0);
        *counter += 1;
        if *counter >= self.starvation_threshold {
            // Diagnostic signal only — bridge SHALL NOT make scheduling decisions.
            // The scheduler retains full authority over when to resume this context.
            eprintln!(
                "[BCIB DIAGNOSTIC] Starvation detected: context {} has been yielded \
                 for {} consecutive slices without resume. \
                 This is a diagnostic event — scheduling authority remains with the scheduler.",
                ctx_id, *counter
            );
        }

        Ok(())
    }

    /// Block until the scheduler sends a resume signal for `ctx_id`.
    ///
    /// Polls `SYS_V2_WAIT_RESULT (1004)` in a bounded loop. Each iteration
    /// passes `ctx_id` as the execution identifier and `RESUME_POLL_TIMEOUT_MS`
    /// as the per-poll timeout. The loop terminates when:
    ///   - The kernel returns a non-negative value (resume received) → `Ok(())`
    ///     and the starvation counter for this context is reset to 0.
    ///   - The kernel returns `WAIT_RESULT_PENDING` (still waiting) → continue.
    ///     If the starvation threshold is exceeded, a diagnostic log is emitted
    ///     (NOT a scheduling decision — bridge continues waiting).
    ///   - The kernel returns a negative error code → `BCIB_ERR_SCHEDULER_BRIDGE_FAIL`
    ///
    /// BCIB does NOT make scheduling decisions here — it only waits for the
    /// kernel's resume signal (Requirements 1.7, 2.9).
    pub fn await_resume(&mut self, ctx_id: ExecutionContextId) -> Result<(), BcibError> {
        let mut slices_waited: u32 = 0;

        loop {
            // arg1 = ctx_id (identifies which context we are waiting to resume)
            // arg2 = RESUME_POLL_TIMEOUT_MS (per-poll timeout in milliseconds)
            // arg3 = 0, arg4 = 0 (reserved)
            let ret =
                unsafe { syscall_v2(SYS_V2_WAIT_RESULT, ctx_id, RESUME_POLL_TIMEOUT_MS, 0, 0) };

            let ret_signed = ret as i64;

            if ret_signed < 0 {
                // Kernel signalled an error — fail-closed (Requirement 2.7).
                return Err(BcibError::SchedulerBridgeFail(
                    "await_resume: SYS_V2_WAIT_RESULT returned error",
                ));
            }

            if ret_signed == WAIT_RESULT_PENDING {
                // Scheduler has not yet resumed this context.
                slices_waited += 1;

                if slices_waited >= self.starvation_threshold {
                    // Starvation detected: emit diagnostic signal only.
                    // Bridge SHALL NOT make scheduling decisions — it only signals.
                    // The scheduler retains full authority (Requirements 2.8, 2.9, 20.3, 20.4).
                    eprintln!(
                        "[BCIB DIAGNOSTIC] Starvation detected in await_resume: context {} \
                         has not been resumed for {} consecutive poll slices. \
                         This is a diagnostic event — scheduling authority remains with the scheduler.",
                        ctx_id, slices_waited
                    );
                    // Reset the threshold counter so we emit periodically, not once.
                    slices_waited = 0;
                }

                // Continue waiting — scheduler will resume when ready.
                continue;
            }

            // Non-negative, non-pending: resume received.
            // Reset the starvation counter for this context.
            self.starvation_counters.insert(ctx_id, 0);
            return Ok(());
        }
    }
}

impl Default for SchedulerSubmitBridge {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Syscall shim (mirrors executor.rs pattern)
// ---------------------------------------------------------------------------

#[cfg(all(target_arch = "x86_64", not(test)))]
#[inline(always)]
pub(crate) unsafe fn syscall_v2(num: u64, arg1: u64, arg2: u64, arg3: u64, arg4: u64) -> u64 {
    use core::arch::asm;
    let ret: u64;
    asm!(
        "int 0x80",
        in("rax") num,
        in("rdi") arg1,
        in("rsi") arg2,
        in("rdx") arg3,
        in("r10") arg4,
        lateout("rax") ret,
        options(nostack, preserves_flags)
    );
    ret
}

#[cfg(all(not(target_arch = "x86_64"), not(test)))]
#[inline(always)]
pub(crate) unsafe fn syscall_v2(_num: u64, _arg1: u64, _arg2: u64, _arg3: u64, _arg4: u64) -> u64 {
    0 // non-x86_64 stub (ARM macOS, etc.)
}

#[cfg(test)]
pub(crate) unsafe fn syscall_v2(num: u64, arg1: u64, arg2: u64, arg3: u64, arg4: u64) -> u64 {
    test_hook::invoke(num, arg1, arg2, arg3, arg4)
}

// ---------------------------------------------------------------------------
// Test syscall hook (mirrors executor.rs pattern)
// ---------------------------------------------------------------------------

#[cfg(test)]
pub(crate) mod test_hook {
    use std::{cell::RefCell, collections::VecDeque};

    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
    pub struct RecordedCall {
        pub num: u64,
        pub arg1: u64,
        pub arg2: u64,
        pub arg3: u64,
        pub arg4: u64,
    }

    #[derive(Debug, Default)]
    struct HookState {
        enabled: bool,
        /// Sequence of return values to emit, one per call (FIFO).
        /// If empty, returns 0.
        return_values: VecDeque<u64>,
        calls: Vec<RecordedCall>,
    }

    thread_local! {
        static STATE: RefCell<HookState> = RefCell::new(HookState::default());
    }

    /// Install the hook with a single return value (used for all calls).
    pub fn install(return_value: u64) {
        STATE.with(|state| {
            let mut g = state.borrow_mut();
            g.enabled = true;
            g.return_values.clear();
            g.return_values.push_back(return_value);
            g.calls.clear();
        });
    }

    /// Install the hook with a sequence of return values (one per call, FIFO).
    pub fn install_seq(values: &[u64]) {
        STATE.with(|state| {
            let mut g = state.borrow_mut();
            g.enabled = true;
            g.return_values = values.iter().copied().collect();
            g.calls.clear();
        });
    }

    pub fn take_calls() -> Vec<RecordedCall> {
        STATE.with(|state| {
            let mut g = state.borrow_mut();
            std::mem::take(&mut g.calls)
        })
    }

    pub fn uninstall() {
        STATE.with(|state| {
            *state.borrow_mut() = HookState::default();
        });
    }

    pub fn invoke(num: u64, arg1: u64, arg2: u64, arg3: u64, arg4: u64) -> u64 {
        STATE.with(|state| {
            let mut g = state.borrow_mut();
            if !g.enabled {
                // Hook not installed — return 0 (success/pending) so callers that
                // don't need syscall recording still work (e.g. execution_runtime tests).
                return 0;
            }
            g.calls.push(RecordedCall {
                num,
                arg1,
                arg2,
                arg3,
                arg4,
            });
            g.return_values.pop_front().unwrap_or(0)
        })
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::test_hook;
    use super::*;

    /// Verify syscall constants match the ABI freeze specification.
    #[test]
    fn syscall_constants_match_abi_freeze() {
        assert_eq!(SYS_V2_SUBMIT_EXECUTION, 1003);
        assert_eq!(SYS_V2_WAIT_RESULT, 1004);
    }

    /// Verify bridge is constructible with default settings.
    #[test]
    fn bridge_constructs_with_defaults() {
        let bridge = SchedulerSubmitBridge::new();
        assert_eq!(bridge.starvation_threshold, STARVATION_THRESHOLD);
    }

    // -----------------------------------------------------------------------
    // submit() tests
    // -----------------------------------------------------------------------

    /// submit() returns ExecutionId on positive kernel return.
    #[test]
    fn submit_returns_execution_id_on_success() {
        test_hook::install(42);
        let bridge = SchedulerSubmitBridge::new();
        let dummy = [0u8; 4];
        let result = bridge.submit(dummy.as_ptr(), dummy.len(), 1);
        test_hook::uninstall();

        assert_eq!(result, Ok(ExecutionId(42)));
    }

    /// submit() uses SYS_V2_SUBMIT_EXECUTION (1003) with arg4 = 0.
    #[test]
    fn submit_uses_correct_syscall_and_args() {
        test_hook::install(99);
        let bridge = SchedulerSubmitBridge::new();
        let dummy = [0xABu8; 8];
        let _ = bridge.submit(dummy.as_ptr(), dummy.len(), 7);
        let calls = test_hook::take_calls();
        test_hook::uninstall();

        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].num, SYS_V2_SUBMIT_EXECUTION);
        assert_eq!(calls[0].arg1, dummy.as_ptr() as u64);
        assert_eq!(calls[0].arg2, 8);
        assert_eq!(calls[0].arg3, 7);
        assert_eq!(
            calls[0].arg4, 0,
            "arg4 must be 0 for graph submission (not yield signal)"
        );
    }

    /// submit() returns BCIB_ERR_SCHEDULER_BRIDGE_FAIL on negative return.
    #[test]
    fn submit_fails_on_negative_syscall_return() {
        test_hook::install(u64::MAX); // -1 as i64
        let bridge = SchedulerSubmitBridge::new();
        let dummy = [0u8; 4];
        let result = bridge.submit(dummy.as_ptr(), dummy.len(), 1);
        test_hook::uninstall();

        assert!(matches!(result, Err(BcibError::SchedulerBridgeFail(_))));
    }

    /// submit() returns BCIB_ERR_SCHEDULER_BRIDGE_FAIL on zero return.
    #[test]
    fn submit_fails_on_zero_syscall_return() {
        test_hook::install(0);
        let bridge = SchedulerSubmitBridge::new();
        let dummy = [0u8; 4];
        let result = bridge.submit(dummy.as_ptr(), dummy.len(), 1);
        test_hook::uninstall();

        assert!(matches!(result, Err(BcibError::SchedulerBridgeFail(_))));
    }

    // -----------------------------------------------------------------------
    // wait_result() tests
    // -----------------------------------------------------------------------

    /// wait_result() returns ExecutionResult on non-negative kernel return.
    #[test]
    fn wait_result_returns_execution_result_on_success() {
        test_hook::install(5);
        let bridge = SchedulerSubmitBridge::new();
        let result = bridge.wait_result(ExecutionId(10), 1000);
        test_hook::uninstall();

        let r = result.unwrap();
        assert_eq!(r.execution_id, ExecutionId(10));
        assert_eq!(r.status, 5);
    }

    /// wait_result() uses SYS_V2_WAIT_RESULT (1004).
    #[test]
    fn wait_result_uses_correct_syscall() {
        test_hook::install(1);
        let bridge = SchedulerSubmitBridge::new();
        let _ = bridge.wait_result(ExecutionId(55), 500);
        let calls = test_hook::take_calls();
        test_hook::uninstall();

        assert_eq!(calls[0].num, SYS_V2_WAIT_RESULT);
        assert_eq!(calls[0].arg1, 55);
        assert_eq!(calls[0].arg2, 500);
    }

    /// wait_result() returns BCIB_ERR_SCHEDULER_BRIDGE_FAIL on negative return.
    #[test]
    fn wait_result_fails_on_negative_syscall_return() {
        test_hook::install(u64::MAX); // -1 as i64
        let bridge = SchedulerSubmitBridge::new();
        let result = bridge.wait_result(ExecutionId(1), 100);
        test_hook::uninstall();

        assert!(matches!(result, Err(BcibError::SchedulerBridgeFail(_))));
    }

    // -----------------------------------------------------------------------
    // yield_slice() tests — Requirements 1.2, 1.7, 2.2, 2.4
    // -----------------------------------------------------------------------

    /// yield_slice() calls SYS_V2_SUBMIT_EXECUTION (1003) with YIELD_SIGNAL in arg4.
    /// This verifies the syscall wiring and that the bridge does NOT use a
    /// different syscall number (ABI freeze, Requirement 1.2).
    #[test]
    fn yield_slice_calls_submit_execution_with_yield_signal() {
        test_hook::install(0); // kernel accepts yield
        let mut bridge = SchedulerSubmitBridge::new();
        let result = bridge.yield_slice(42);
        let calls = test_hook::take_calls();
        test_hook::uninstall();

        assert_eq!(result, Ok(()));
        assert_eq!(calls.len(), 1);
        assert_eq!(
            calls[0].num, SYS_V2_SUBMIT_EXECUTION,
            "must use SYS_V2_SUBMIT_EXECUTION (1003)"
        );
        assert_eq!(calls[0].arg1, 42, "arg1 must be ctx_id");
        assert_eq!(calls[0].arg2, 0, "arg2 must be 0 (no graph pointer)");
        assert_eq!(calls[0].arg3, 0, "arg3 must be 0 (no graph length)");
        assert_eq!(
            calls[0].arg4, YIELD_SIGNAL,
            "arg4 must be YIELD_SIGNAL to distinguish from graph submission"
        );
    }

    /// yield_slice() returns Ok(()) when the kernel accepts the yield signal.
    #[test]
    fn yield_slice_returns_ok_on_success() {
        test_hook::install(0);
        let mut bridge = SchedulerSubmitBridge::new();
        let result = bridge.yield_slice(1);
        test_hook::uninstall();

        assert_eq!(result, Ok(()));
    }

    /// yield_slice() returns BCIB_ERR_SCHEDULER_BRIDGE_FAIL when the kernel
    /// rejects the yield signal (negative return). Execution must be
    /// fail-closed (Requirement 2.7).
    #[test]
    fn yield_slice_returns_bridge_fail_on_negative_syscall() {
        test_hook::install(u64::MAX); // -1 as i64
        let mut bridge = SchedulerSubmitBridge::new();
        let result = bridge.yield_slice(1);
        test_hook::uninstall();

        assert!(
            matches!(result, Err(BcibError::SchedulerBridgeFail(_))),
            "yield_slice must return BCIB_ERR_SCHEDULER_BRIDGE_FAIL on syscall error"
        );
    }

    /// yield_slice() does NOT make scheduling decisions — it only signals.
    /// Verified by checking that no SYS_V2_WAIT_RESULT call is made.
    #[test]
    fn yield_slice_does_not_call_wait_result() {
        test_hook::install(0);
        let mut bridge = SchedulerSubmitBridge::new();
        let _ = bridge.yield_slice(5);
        let calls = test_hook::take_calls();
        test_hook::uninstall();

        for call in &calls {
            assert_ne!(
                call.num, SYS_V2_WAIT_RESULT,
                "yield_slice must not call SYS_V2_WAIT_RESULT"
            );
        }
    }

    /// yield_slice() increments the starvation counter for the context.
    #[test]
    fn yield_slice_increments_starvation_counter() {
        test_hook::install_seq(&[0, 0, 0]);
        let mut bridge = SchedulerSubmitBridge::new();
        bridge.yield_slice(10).unwrap();
        bridge.yield_slice(10).unwrap();
        bridge.yield_slice(10).unwrap();
        test_hook::uninstall();

        assert_eq!(
            bridge.starvation_counters.get(&10).copied().unwrap_or(0),
            3,
            "starvation counter must be incremented on each yield_slice call"
        );
    }

    /// yield_slice() emits a diagnostic log (does NOT fail) when starvation
    /// threshold is reached (Requirements 2.8, 20.3, 20.4).
    #[test]
    fn yield_slice_emits_diagnostic_at_threshold_does_not_fail() {
        let threshold = 3u32;
        // Install enough return values for threshold calls.
        let values: Vec<u64> = vec![0; threshold as usize + 1];
        test_hook::install_seq(&values);

        let mut bridge = SchedulerSubmitBridge::with_threshold(threshold);
        // Call yield_slice threshold times — must NOT return Err.
        for _ in 0..threshold {
            let result = bridge.yield_slice(99);
            assert!(
                result.is_ok(),
                "yield_slice must not fail when starvation threshold is reached — it only emits a diagnostic"
            );
        }
        test_hook::uninstall();

        // Counter must be at threshold.
        assert_eq!(
            bridge.starvation_counters.get(&99).copied().unwrap_or(0),
            threshold,
        );
    }

    // -----------------------------------------------------------------------
    // await_resume() tests — Requirements 1.7, 2.2, 2.4
    // -----------------------------------------------------------------------

    /// await_resume() calls SYS_V2_WAIT_RESULT (1004) with ctx_id as arg1.
    #[test]
    fn await_resume_calls_wait_result_with_ctx_id() {
        // Return value > 0 → resume received immediately.
        test_hook::install(1);
        let mut bridge = SchedulerSubmitBridge::new();
        let result = bridge.await_resume(77);
        let calls = test_hook::take_calls();
        test_hook::uninstall();

        assert_eq!(result, Ok(()));
        assert_eq!(calls.len(), 1);
        assert_eq!(
            calls[0].num, SYS_V2_WAIT_RESULT,
            "must use SYS_V2_WAIT_RESULT (1004)"
        );
        assert_eq!(calls[0].arg1, 77, "arg1 must be ctx_id");
        assert_eq!(
            calls[0].arg2, RESUME_POLL_TIMEOUT_MS,
            "arg2 must be per-poll timeout"
        );
    }

    /// await_resume() returns Ok(()) when the kernel signals resume immediately.
    #[test]
    fn await_resume_returns_ok_on_immediate_resume() {
        test_hook::install(1); // positive → resume received
        let mut bridge = SchedulerSubmitBridge::new();
        let result = bridge.await_resume(1);
        test_hook::uninstall();

        assert_eq!(result, Ok(()));
    }

    /// await_resume() polls until resume is received (WAIT_RESULT_PENDING → resume).
    /// Verifies the polling loop: first N calls return 0 (pending), then 1 (resume).
    #[test]
    fn await_resume_polls_until_resume_received() {
        // 3 pending (0), then resume (1)
        test_hook::install_seq(&[0, 0, 0, 1]);
        let mut bridge = SchedulerSubmitBridge::new();
        let result = bridge.await_resume(10);
        let calls = test_hook::take_calls();
        test_hook::uninstall();

        assert_eq!(result, Ok(()));
        assert_eq!(
            calls.len(),
            4,
            "must poll 3 times (pending) then once more (resume)"
        );
        for call in &calls {
            assert_eq!(call.num, SYS_V2_WAIT_RESULT);
            assert_eq!(call.arg1, 10);
        }
    }

    /// await_resume() returns BCIB_ERR_SCHEDULER_BRIDGE_FAIL on negative syscall return.
    /// Fail-closed semantics (Requirement 2.7).
    #[test]
    fn await_resume_returns_bridge_fail_on_negative_syscall() {
        test_hook::install(u64::MAX); // -1 as i64
        let mut bridge = SchedulerSubmitBridge::new();
        let result = bridge.await_resume(1);
        test_hook::uninstall();

        assert!(
            matches!(result, Err(BcibError::SchedulerBridgeFail(_))),
            "await_resume must return BCIB_ERR_SCHEDULER_BRIDGE_FAIL on syscall error"
        );
    }

    /// await_resume() emits a diagnostic log (does NOT fail) when the starvation
    /// threshold is exceeded (Requirements 2.8, 20.3, 20.4).
    ///
    /// The bridge SHALL NOT make scheduling decisions — it only emits a diagnostic
    /// signal and continues waiting. The scheduler retains full authority.
    #[test]
    fn await_resume_emits_diagnostic_at_threshold_does_not_fail() {
        let threshold = 4u32;
        // All pending (0) for threshold iterations, then resume (1).
        let mut values: Vec<u64> = vec![0; threshold as usize];
        values.push(1); // resume after threshold pending polls
        test_hook::install_seq(&values);

        let mut bridge = SchedulerSubmitBridge::with_threshold(threshold);
        // Must NOT fail — starvation only emits a diagnostic.
        let result = bridge.await_resume(99);
        test_hook::uninstall();

        assert_eq!(
            result,
            Ok(()),
            "await_resume must NOT fail when starvation threshold is exceeded — \
             it emits a diagnostic and continues waiting"
        );
    }

    /// await_resume() resets the starvation counter on successful resume.
    #[test]
    fn await_resume_resets_starvation_counter_on_resume() {
        // First yield to set counter to 1.
        test_hook::install_seq(&[0, 1]); // yield ok, then resume
        let mut bridge = SchedulerSubmitBridge::with_threshold(10);
        bridge.yield_slice(5).unwrap(); // counter = 1
        bridge.await_resume(5).unwrap(); // counter reset to 0
        test_hook::uninstall();

        assert_eq!(
            bridge.starvation_counters.get(&5).copied().unwrap_or(0),
            0,
            "starvation counter must be reset to 0 after successful resume"
        );
    }

    /// await_resume() does NOT make scheduling decisions — it only waits.
    /// Verified by checking that no SYS_V2_SUBMIT_EXECUTION call is made.
    #[test]
    fn await_resume_does_not_call_submit_execution() {
        test_hook::install(1); // immediate resume
        let mut bridge = SchedulerSubmitBridge::new();
        let _ = bridge.await_resume(3);
        let calls = test_hook::take_calls();
        test_hook::uninstall();

        for call in &calls {
            assert_ne!(
                call.num, SYS_V2_SUBMIT_EXECUTION,
                "await_resume must not call SYS_V2_SUBMIT_EXECUTION"
            );
        }
    }
}
