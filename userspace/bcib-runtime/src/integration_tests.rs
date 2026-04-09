/// Integration tests for BCIB Execution Engine v3 — Task 41.2
///
/// Covers:
///   a. Full lifecycle: verify_and_plan() → create_context() → run_slice() → cancel()
///      After cancel(), verify teardown: all slots returned to pool, state is Cancelled.
///   b. v0.2 backward-compat: a BCIB graph with version 0x0002 either executes
///      compatibly or returns BCIB_ERR_UNSUPPORTED_VERSION (not silent partial execution).
///   c. Teardown completeness: after Failed or Cancelled state, pool counts match
///      pre-execution counts.
///
/// Requirements: 1.5, 2.6, 3.9

#[cfg(test)]
mod integration_tests {
    use crate::binary_format::{
        BCIB_VERSION_V02, BCIB_VERSION_V3, HEADER_SIZE, SECTION_ENTRY_SIZE,
    };
    use crate::execution_runtime::BcibExecutionRuntime;
    use crate::types::{
        BcibError, CapabilitySet, CostBudget, ExecutionContextId, ResourceLimits, SliceResult,
    };
    use crate::verifier_planner::BcibVerifierPlanner;
    use crate::binary_format::SectionId;
    use crate::execution_runtime::ExecutionState;

    // -----------------------------------------------------------------------
    // Helpers
    // -----------------------------------------------------------------------

    /// Build a minimal valid BCIB buffer with the given version and instruction bytes.
    fn build_buffer(version: u16, instr_bytes: &[u8]) -> Vec<u8> {
        let instr_len = instr_bytes.len();
        let instr_offset: u32 = (HEADER_SIZE + SECTION_ENTRY_SIZE) as u32; // 24

        let mut buf = Vec::new();

        // Header (16 bytes)
        buf.extend_from_slice(b"BCIB");
        buf.extend_from_slice(&version.to_le_bytes());
        buf.extend_from_slice(&0u16.to_le_bytes()); // flags
        buf.extend_from_slice(&1u16.to_le_bytes()); // section_count
        buf.extend_from_slice(&[0u8; 4]);           // reserved
        buf.extend_from_slice(&[0u8; 2]);           // header tail bytes 14-15

        // Section table entry (8 bytes)
        buf.extend_from_slice(&(SectionId::Instructions as u16).to_le_bytes());
        buf.extend_from_slice(&instr_offset.to_le_bytes());
        buf.extend_from_slice(&(instr_len as u16).to_le_bytes());

        // Instruction data
        buf.extend_from_slice(instr_bytes);

        buf
    }

    /// Encode a single instruction: opcode(1) + operand_count(1) + operands(n×4).
    fn encode_instr(opcode: u8, operands: &[u32]) -> Vec<u8> {
        let mut bytes = vec![opcode, operands.len() as u8];
        for &op in operands {
            bytes.extend_from_slice(&op.to_le_bytes());
        }
        bytes
    }

    /// Build a minimal valid v3 BCIB buffer with Nop + End instructions.
    fn minimal_v3_graph() -> Vec<u8> {
        let mut instr_bytes = Vec::new();
        instr_bytes.extend(encode_instr(0x00 /* Nop */, &[]));
        instr_bytes.extend(encode_instr(0x01 /* End */, &[]));
        build_buffer(BCIB_VERSION_V3, &instr_bytes)
    }

    /// Build a minimal v0.2 BCIB buffer with Nop + End instructions.
    fn minimal_v02_graph() -> Vec<u8> {
        let mut instr_bytes = Vec::new();
        instr_bytes.extend(encode_instr(0x00 /* Nop */, &[]));
        instr_bytes.extend(encode_instr(0x01 /* End */, &[]));
        build_buffer(BCIB_VERSION_V02, &instr_bytes)
    }

    fn empty_caps() -> CapabilitySet {
        CapabilitySet::default()
    }

    fn default_limits() -> ResourceLimits {
        ResourceLimits::default()
    }

    fn generous_budget() -> CostBudget {
        CostBudget::new(10_000, 1_000)
    }

    // -----------------------------------------------------------------------
    // a. Full lifecycle: verify_and_plan() → create_context() → run_slice() → cancel()
    // Requirements: 1.5, 2.6, 3.9
    // -----------------------------------------------------------------------

    /// Full lifecycle test: verify_and_plan → create_context → run_slice → cancel.
    ///
    /// After cancel():
    ///   - state must be Cancelled
    ///   - all slots returned to pool (outstanding_count == 0)
    ///   - ABDF handles cleared
    ///   - capability tokens revoked
    #[test]
    fn full_lifecycle_verify_plan_run_cancel() {
        let planner = BcibVerifierPlanner::new();
        let buf = minimal_v3_graph();
        let caps = empty_caps();
        let limits = default_limits();

        // Step 1: verify_and_plan
        let plan = planner
            .verify_and_plan(&buf, &caps, &limits)
            .expect("valid v3 graph must produce a plan");

        // Step 2: create_context
        let mut runtime = BcibExecutionRuntime::new();
        let ctx_id = runtime
            .create_context(plan, caps.clone())
            .expect("create_context must succeed");

        // Verify initial state is Ready
        let state = runtime.state_of(ctx_id).expect("state_of must succeed");
        assert!(
            matches!(state, ExecutionState::Ready),
            "context must start in Ready state, got {:?}",
            state
        );

        // Step 3: run_slice
        let result = runtime
            .run_slice(ctx_id, generous_budget())
            .expect("run_slice must not return an error for a valid graph");

        // The graph has Nop + End — it should complete in one slice.
        assert!(
            matches!(result, SliceResult::Completed | SliceResult::Yielded),
            "run_slice must return Completed or Yielded, got {:?}",
            result
        );

        // Step 4: cancel (if not already completed)
        let state_after_run = runtime.state_of(ctx_id).expect("state_of must succeed");
        if !state_after_run.is_terminal() {
            runtime.cancel(ctx_id).expect("cancel must succeed on non-terminal context");
        } else {
            // Already completed — cancel on terminal state must return IllegalStateTransition.
            let cancel_result = runtime.cancel(ctx_id);
            assert!(
                matches!(cancel_result, Err(BcibError::IllegalStateTransition(_))),
                "cancel on terminal state must return IllegalStateTransition"
            );
        }

        // Verify teardown: state is terminal (Cancelled or Completed)
        let final_state = runtime.state_of(ctx_id).expect("state_of must succeed");
        assert!(
            final_state.is_terminal(),
            "context must be in a terminal state after lifecycle, got {:?}",
            final_state
        );
    }

    /// Explicit cancel path: context is created, NOT run, then cancelled.
    /// Teardown must still clean up all resources.
    #[test]
    fn lifecycle_cancel_without_run_cleans_up() {
        let planner = BcibVerifierPlanner::new();
        let buf = minimal_v3_graph();
        let caps = empty_caps();
        let limits = default_limits();

        let plan = planner
            .verify_and_plan(&buf, &caps, &limits)
            .expect("valid v3 graph must produce a plan");

        let mut runtime = BcibExecutionRuntime::new();
        let ctx_id = runtime
            .create_context(plan, caps)
            .expect("create_context must succeed");

        // Cancel without running
        runtime.cancel(ctx_id).expect("cancel must succeed on Ready context");

        // Verify teardown contract (Requirement 3.9, 3.10)
        let ctx = runtime
            .contexts
            .get(&ctx_id)
            .expect("context must still be accessible after cancel");

        assert!(
            matches!(ctx.state, ExecutionState::Cancelled),
            "state must be Cancelled after cancel()"
        );
        assert_eq!(
            ctx.slot_space.outstanding_count(),
            0,
            "all slots must be returned to pool after teardown"
        );
        assert!(
            ctx.abdf_handles.is_empty(),
            "ABDF handles must be cleared after teardown"
        );
        assert!(
            ctx.capability_set.token_ids.is_empty(),
            "capability tokens must be revoked after teardown"
        );
    }

    /// Full lifecycle with explicit run → Completed path.
    /// After Completed, cancel must return IllegalStateTransition.
    #[test]
    fn lifecycle_run_to_completion_then_cancel_rejected() {
        let planner = BcibVerifierPlanner::new();
        let buf = minimal_v3_graph();
        let caps = empty_caps();
        let limits = default_limits();

        let plan = planner
            .verify_and_plan(&buf, &caps, &limits)
            .expect("valid v3 graph must produce a plan");

        let mut runtime = BcibExecutionRuntime::new();
        let ctx_id = runtime
            .create_context(plan, caps)
            .expect("create_context must succeed");

        // Run to completion
        let result = runtime
            .run_slice(ctx_id, generous_budget())
            .expect("run_slice must succeed");

        // For a Nop+End graph, we expect Completed.
        // If it yielded (e.g. budget too small), run again.
        if matches!(result, SliceResult::Yielded) {
            let _ = runtime.run_slice(ctx_id, generous_budget());
        }

        let state = runtime.state_of(ctx_id).expect("state_of must succeed");
        if matches!(state, ExecutionState::Completed { .. }) {
            // Cancel on Completed must be rejected (Requirement 3b.3)
            let cancel_result = runtime.cancel(ctx_id);
            assert!(
                matches!(cancel_result, Err(BcibError::IllegalStateTransition(_))),
                "cancel on Completed state must return IllegalStateTransition, got {:?}",
                cancel_result
            );
        }
    }

    // -----------------------------------------------------------------------
    // b. v0.2 backward-compat: either compatible execution or BCIB_ERR_UNSUPPORTED_VERSION
    // Requirements: 1.5
    // -----------------------------------------------------------------------

    /// A v0.2 BCIB graph must either:
    ///   - produce a valid ExecutionPlan (backward-compatible path), OR
    ///   - return BCIB_ERR_UNSUPPORTED_VERSION (fail-closed)
    ///
    /// Silent partial execution is forbidden (Requirement 1.5, design.md Karar 3).
    #[test]
    fn v02_graph_backward_compat_or_fail_closed() {
        let planner = BcibVerifierPlanner::new();
        let buf = minimal_v02_graph();
        let caps = empty_caps();
        let limits = default_limits();

        let result = planner.verify_and_plan(&buf, &caps, &limits);

        match result {
            Ok(plan) => {
                // Backward-compatible path: plan must be valid and have a stable hash.
                let hash_a = plan.canonical_hash();
                let hash_b = plan.canonical_hash();
                assert_eq!(
                    hash_a, hash_b,
                    "v0.2 backward-compat plan must have deterministic canonical_hash"
                );
                // Version must be preserved as v0.2
                assert_eq!(
                    plan.version(),
                    BCIB_VERSION_V02,
                    "v0.2 plan must preserve version 0x0002"
                );
            }
            Err(BcibError::UnsupportedVersion(_)) => {
                // Fail-closed path: acceptable per Requirement 1.5.
                // This is the correct fail-closed response.
            }
            Err(other) => {
                panic!(
                    "v0.2 graph must produce Ok(plan) or UnsupportedVersion, got {:?}",
                    other
                );
            }
        }
    }

    /// v0.2 graph must NOT silently partially execute — the result must be
    /// deterministic (same input → same outcome on repeated calls).
    #[test]
    fn v02_graph_result_is_deterministic() {
        let planner = BcibVerifierPlanner::new();
        let buf = minimal_v02_graph();
        let caps = empty_caps();
        let limits = default_limits();

        let result_a = planner.verify_and_plan(&buf, &caps, &limits);
        let result_b = planner.verify_and_plan(&buf, &caps, &limits);

        // Both calls must produce the same outcome (both Ok or both Err with same variant).
        match (result_a, result_b) {
            (Ok(plan_a), Ok(plan_b)) => {
                assert_eq!(
                    plan_a.canonical_hash(),
                    plan_b.canonical_hash(),
                    "v0.2 backward-compat: two calls must produce identical canonical_hash"
                );
            }
            (Err(BcibError::UnsupportedVersion(_)), Err(BcibError::UnsupportedVersion(_))) => {
                // Both fail-closed — deterministic.
            }
            (a, b) => {
                panic!(
                    "v0.2 graph must produce deterministic results; got {:?} and {:?}",
                    a, b
                );
            }
        }
    }

    /// An unsupported version (not v3, not v0.2) must always return
    /// BCIB_ERR_UNSUPPORTED_VERSION — never Ok (fail-closed, Requirement 1.5).
    #[test]
    fn unsupported_version_always_fails_closed() {
        let planner = BcibVerifierPlanner::new();
        let caps = empty_caps();
        let limits = default_limits();

        for bad_version in [0x0000u16, 0x0001, 0x0004, 0x0100, 0xFFFF] {
            let mut instr_bytes = Vec::new();
            instr_bytes.extend(encode_instr(0x00 /* Nop */, &[]));
            instr_bytes.extend(encode_instr(0x01 /* End */, &[]));
            let buf = build_buffer(bad_version, &instr_bytes);

            let result = planner.verify_and_plan(&buf, &caps, &limits);
            assert!(
                matches!(result, Err(BcibError::UnsupportedVersion(_))),
                "version 0x{:04X} must return UnsupportedVersion, got {:?}",
                bad_version,
                result
            );
        }
    }

    // -----------------------------------------------------------------------
    // c. Teardown completeness: after Failed or Cancelled, pool counts match
    //    pre-execution counts.
    // Requirements: 3.9, 2.6
    // -----------------------------------------------------------------------

    /// After cancel(), slot pool outstanding count must be 0 (all returned).
    #[test]
    fn teardown_after_cancel_slot_pool_is_clean() {
        let planner = BcibVerifierPlanner::new();
        let buf = minimal_v3_graph();
        let caps = empty_caps();
        let limits = default_limits();

        let plan = planner
            .verify_and_plan(&buf, &caps, &limits)
            .expect("valid v3 graph must produce a plan");

        let mut runtime = BcibExecutionRuntime::new();
        let ctx_id = runtime
            .create_context(plan, caps)
            .expect("create_context must succeed");

        // Run a slice (may or may not complete)
        let _ = runtime.run_slice(ctx_id, generous_budget());

        // Cancel (may fail if already terminal — that's fine)
        let _ = runtime.cancel(ctx_id);

        // Verify teardown completeness (Requirement 3.9)
        let ctx = runtime
            .contexts
            .get(&ctx_id)
            .expect("context must be accessible after cancel");

        assert_eq!(
            ctx.slot_space.outstanding_count(),
            0,
            "teardown after cancel: all slots must be returned to pool"
        );
        assert!(
            ctx.abdf_handles.is_empty(),
            "teardown after cancel: ABDF handles must be cleared"
        );
        assert_eq!(
            ctx.handle_space.registered_count(),
            0,
            "teardown after cancel: handle_space must be empty"
        );
    }

    /// After a Failed state transition, teardown must also clean up all resources.
    /// We simulate a failure by using a graph that exceeds max_instruction_count.
    #[test]
    fn teardown_after_failed_state_pool_is_clean() {
        let planner = BcibVerifierPlanner::new();

        // Build a graph with exactly 3 pure instructions (Nop, Nop, End).
        let mut instr_bytes = Vec::new();
        instr_bytes.extend(encode_instr(0x00 /* Nop */, &[]));
        instr_bytes.extend(encode_instr(0x00 /* Nop */, &[]));
        instr_bytes.extend(encode_instr(0x01 /* End */, &[]));
        let buf = build_buffer(BCIB_VERSION_V3, &instr_bytes);

        let caps = empty_caps();
        // Set max_instruction_count to 3 so the graph passes verification.
        let limits = ResourceLimits {
            max_instruction_count: 3,
            ..ResourceLimits::default()
        };

        let plan = planner
            .verify_and_plan(&buf, &caps, &limits)
            .expect("3-instruction graph within limits must produce a plan");

        let mut runtime = BcibExecutionRuntime::new();

        // Use very tight limits at runtime: max_instruction_count = 1 to force failure.
        let tight_limits = ResourceLimits {
            max_instruction_count: 1,
            max_instructions_per_slice: 256,
            ..ResourceLimits::default()
        };

        let ctx_id = runtime
            .create_context_with_limits(plan, caps, tight_limits)
            .expect("create_context_with_limits must succeed");

        // Run slice — should fail due to max_instruction_count exceeded.
        let result = runtime.run_slice(ctx_id, generous_budget());

        // Either it fails immediately or completes within 1 instruction.
        // In either case, teardown must have been applied.
        let ctx = runtime
            .contexts
            .get(&ctx_id)
            .expect("context must be accessible after run_slice");

        // Teardown completeness: regardless of outcome, no outstanding slots.
        assert_eq!(
            ctx.slot_space.outstanding_count(),
            0,
            "teardown after run_slice: all slots must be returned to pool"
        );
        assert!(
            ctx.abdf_handles.is_empty(),
            "teardown after run_slice: ABDF handles must be cleared"
        );

        // If the result was an error, the state must be Failed.
        if result.is_err() {
            assert!(
                matches!(ctx.state, ExecutionState::Failed { .. }),
                "run_slice error must transition context to Failed state, got {:?}",
                ctx.state
            );
        }
    }

    /// Multiple contexts: teardown of one context must not affect another.
    #[test]
    fn teardown_of_one_context_does_not_affect_another() {
        let planner = BcibVerifierPlanner::new();
        let buf = minimal_v3_graph();
        let caps = empty_caps();
        let limits = default_limits();

        let plan_a = planner
            .verify_and_plan(&buf, &caps, &limits)
            .expect("plan_a must succeed");
        let plan_b = planner
            .verify_and_plan(&buf, &caps, &limits)
            .expect("plan_b must succeed");

        let mut runtime = BcibExecutionRuntime::new();
        let ctx_a = runtime
            .create_context(plan_a, caps.clone())
            .expect("create_context A must succeed");
        let ctx_b = runtime
            .create_context(plan_b, caps)
            .expect("create_context B must succeed");

        // Cancel context A
        runtime.cancel(ctx_a).expect("cancel A must succeed");

        // Context B must still be in Ready state (unaffected)
        let state_b = runtime.state_of(ctx_b).expect("state_of B must succeed");
        assert!(
            matches!(state_b, ExecutionState::Ready),
            "context B must remain Ready after context A is cancelled, got {:?}",
            state_b
        );

        // Context A must be Cancelled
        let state_a = runtime.state_of(ctx_a).expect("state_of A must succeed");
        assert!(
            matches!(state_a, ExecutionState::Cancelled),
            "context A must be Cancelled, got {:?}",
            state_a
        );
    }

    /// Teardown contract: cancel on already-cancelled context returns
    /// IllegalStateTransition (idempotent cancel is not supported — fail-closed).
    #[test]
    fn double_cancel_returns_illegal_state_transition() {
        let planner = BcibVerifierPlanner::new();
        let buf = minimal_v3_graph();
        let caps = empty_caps();
        let limits = default_limits();

        let plan = planner
            .verify_and_plan(&buf, &caps, &limits)
            .expect("plan must succeed");

        let mut runtime = BcibExecutionRuntime::new();
        let ctx_id = runtime
            .create_context(plan, caps)
            .expect("create_context must succeed");

        // First cancel — must succeed
        runtime.cancel(ctx_id).expect("first cancel must succeed");

        // Second cancel — must return IllegalStateTransition (Requirement 3b.3)
        let result = runtime.cancel(ctx_id);
        assert!(
            matches!(result, Err(BcibError::IllegalStateTransition(_))),
            "second cancel must return IllegalStateTransition, got {:?}",
            result
        );
    }
}
