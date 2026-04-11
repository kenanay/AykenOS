/// BCIB_Verifier/Planner — Layer 1 of the three-layer v3 architecture.
///
/// Responsibilities (Requirement 1.6):
///   - Run the four-phase validation pipeline (structural → control-flow →
///     capability → bounds) in fail-fast order.
///   - Produce an immutable `ExecutionPlan` on success.
///   - Never make execution decisions — only validate and plan.
///
/// This module communicates with other layers exclusively through the types
/// defined in `types.rs`. No cross-layer implementation dependencies.
///
/// ## Instruction Binary Encoding (Instruction Section, section_id = 0x01)
///
/// Each instruction is encoded as:
///   byte 0     : opcode (u8)
///   byte 1     : operand_count (u8)
///   bytes 2..  : operands (operand_count × u32 LE)
///
/// Total size per instruction: 2 + operand_count * 4 bytes.
use crate::binary_format::{parse_header_and_sections, SectionId};
use crate::capability_manager::{CapabilityCheck, CapabilityResource, NoopCapabilityManager};
use crate::opcode_registry::lookup_opcode;
use crate::types::{
    BcibError, BcibInstruction, CapabilitySet, ExecutionPlan, ResourceLimits, SideEffectClass,
};

// ---------------------------------------------------------------------------
// Public struct
// ---------------------------------------------------------------------------

/// Validates a raw BCIB graph and produces an `ExecutionPlan`.
///
/// The planner is stateless — it holds no mutable state between calls.
/// All validation is performed in a single `verify_and_plan()` call.
pub struct BcibVerifierPlanner;

impl BcibVerifierPlanner {
    pub fn new() -> Self {
        Self
    }

    /// Four-phase validation pipeline + plan production.
    ///
    /// Phases run in strict order (fail-fast):
    ///   1. Structural validation
    ///   2. Control-flow validation
    ///   3. Capability validation
    ///   4. Bounds validation
    ///
    /// Returns an immutable `ExecutionPlan` on success, or the first
    /// `BcibError` encountered (fail-closed, Requirement 4.2).
    pub fn verify_and_plan(
        &self,
        graph: &[u8],
        capability_set: &CapabilitySet,
        resource_limits: &ResourceLimits,
    ) -> Result<ExecutionPlan, BcibError> {
        // Phase 1 — structural
        let (mut instructions, version) = self.verify_structural(graph)?;

        // Phase 2 — control-flow
        self.verify_control_flow(&instructions)?;

        // Phase 3 — capability
        self.verify_capabilities(&mut instructions, capability_set)?;

        // Phase 4 — bounds
        self.verify_bounds(&instructions, resource_limits)?;

        Ok(ExecutionPlan::new(instructions, version))
    }

    /// Validate whether a BCIB graph is structurally safe to submit.
    ///
    /// This deliberately does not run capability validation or produce an
    /// execution plan. Callers that cross an authority boundary must validate
    /// semantic capabilities before using this graph-level check.
    pub fn verify_submittable_graph(
        &self,
        graph: &[u8],
        resource_limits: &ResourceLimits,
    ) -> Result<(), BcibError> {
        let (instructions, _) = self.verify_structural(graph)?;

        self.verify_control_flow(&instructions)?;
        self.verify_bounds(&instructions, resource_limits)?;

        Ok(())
    }

    // -----------------------------------------------------------------------
    // Phase 1: Structural validation (Requirement 16.1, 16.4)
    // -----------------------------------------------------------------------

    /// Validates BCIB header magic, version, section layout, and opcode set.
    /// Assigns `SideEffectClass` and cost to each instruction.
    ///
    /// Steps:
    ///   1. Parse and validate the 16-byte header (magic + version).
    ///   2. Parse and validate the section table (overlap + bounds check).
    ///   3. Locate the Instructions section (section_id = 0x01).
    ///   4. Decode each instruction; for each opcode call `lookup_opcode()`.
    ///      Unknown opcode → `BCIB_ERR_INVALID_GRAPH` (fail-closed).
    ///   5. Assign `SideEffectClass` and cost from the opcode descriptor.
    ///
    /// Returns the decoded instruction list and the BCIB version on success.
    fn verify_structural(&self, graph: &[u8]) -> Result<(Vec<BcibInstruction>, u16), BcibError> {
        // Single-pass decode: parse header + section table in one call.
        // Fail-fast: any header or section-table error returns immediately
        // before any instruction decoding begins (Requirement 19.1, 19.2).
        let (header, sections) = parse_header_and_sections(graph)?;
        let version = header.version;

        // Step 3 — overlap check: no two sections may share any byte.
        // We sort a copy of the (offset, end) pairs and verify no overlap.
        {
            let mut ranges: Vec<(usize, usize)> = sections
                .iter()
                .map(|s| (s.offset as usize, s.offset as usize + s.length as usize))
                .collect();
            ranges.sort_unstable_by_key(|r| r.0);
            for window in ranges.windows(2) {
                let (_, end_a) = window[0];
                let (start_b, _) = window[1];
                if end_a > start_b {
                    return Err(BcibError::InvalidGraph(
                        "section table entries overlap in the buffer",
                    ));
                }
            }
        }

        // Step 4 — locate the Instructions section (section_id = 0x01).
        let instr_section = sections
            .iter()
            .find(|s| s.section_id == SectionId::Instructions as u16)
            .ok_or(BcibError::InvalidGraph(
                "missing Instructions section (section_id = 0x01)",
            ))?;

        let instr_data = &graph[instr_section.offset as usize
            ..instr_section.offset as usize + instr_section.length as usize];

        // Step 5 — decode instructions and validate each opcode.
        //
        // Instruction encoding:
        //   byte 0     : opcode (u8)
        //   byte 1     : operand_count (u8)
        //   bytes 2..  : operands (operand_count × u32 LE)
        let mut instructions = Vec::new();
        let mut cursor = 0usize;

        while cursor < instr_data.len() {
            // Need at least 2 bytes for opcode + operand_count.
            if cursor + 2 > instr_data.len() {
                return Err(BcibError::InvalidGraph(
                    "truncated instruction: missing opcode or operand_count byte",
                ));
            }

            let opcode = instr_data[cursor];
            let operand_count = instr_data[cursor + 1] as usize;
            cursor += 2;

            // Validate that all operand bytes are present.
            let operands_byte_len = operand_count
                .checked_mul(4)
                .ok_or(BcibError::InvalidGraph("operand count overflow"))?;

            if cursor + operands_byte_len > instr_data.len() {
                return Err(BcibError::InvalidGraph(
                    "truncated instruction: operand data extends beyond section boundary",
                ));
            }

            // Decode operands (u32 LE each).
            let mut operands = Vec::with_capacity(operand_count);
            for _ in 0..operand_count {
                let val = u32::from_le_bytes([
                    instr_data[cursor],
                    instr_data[cursor + 1],
                    instr_data[cursor + 2],
                    instr_data[cursor + 3],
                ]);
                operands.push(val);
                cursor += 4;
            }

            // Look up opcode — unknown opcode → fail-closed (Requirement 16.1).
            let descriptor = lookup_opcode(opcode)?;

            instructions.push(BcibInstruction {
                opcode,
                operands,
                side_effect_class: descriptor.side_effect,
                cost: descriptor.cost,
                required_capabilities: Vec::new(), // pre-bound in Phase 3
            });
        }

        Ok((instructions, version))
    }

    // -----------------------------------------------------------------------
    // Phase 2: Control-flow validation (Requirement 16.1, 16.2)
    // -----------------------------------------------------------------------

    /// Detects infinite loops (DFS cycle detection), unreachable instructions,
    /// and out-of-bounds jump targets.
    ///
    /// MUST only be called after Phase 1 succeeds (fail-fast ordering).
    ///
    /// ## Algorithm
    ///
    /// 1. **Jump target bounds check** — for every Jump (0x02) and JumpIf (0x03)
    ///    instruction, operand[0] must be a valid instruction index.  An
    ///    out-of-bounds target is rejected immediately with
    ///    `BCIB_ERR_CONTROL_FLOW_VIOLATION` (fail-closed, Requirement 16.2).
    ///
    /// 2. **DFS cycle detection** — we build a successor list for each
    ///    instruction and run an iterative DFS from index 0.  A back-edge
    ///    (reaching a node that is currently on the DFS stack) indicates an
    ///    infinite loop → `BCIB_ERR_CONTROL_FLOW_VIOLATION`.
    ///
    ///    Successor rules:
    ///    - `End` (0x01): no successors (terminates the program).
    ///    - `Jump` (0x02): single successor = operand[0] (unconditional branch).
    ///    - `JumpIf` (0x03): two successors = operand[0] (taken) and pc+1 (not taken).
    ///    - All other instructions: single successor = pc+1 (fall-through),
    ///      unless pc is the last instruction (no successor).
    ///
    /// 3. **Reachability analysis** — after the DFS, any instruction whose
    ///    `visited` flag is still `false` is unreachable dead code →
    ///    `BCIB_ERR_CONTROL_FLOW_VIOLATION` (Requirement 16.2).
    ///
    /// No heap allocation beyond the three fixed-size `Vec<bool>` / `Vec<usize>`
    /// structures; no global mutable state (DETERMINISM.GLOBAL).
    fn verify_control_flow(&self, instructions: &[BcibInstruction]) -> Result<(), BcibError> {
        let n = instructions.len();

        // An empty program is trivially valid.
        if n == 0 {
            return Ok(());
        }

        // ---------------------------------------------------------------
        // Step 1 — Jump target bounds check.
        //
        // Opcodes that carry a target operand:
        //   0x02 Jump   — operand[0] is the absolute target index.
        //   0x03 JumpIf — operand[0] is the taken-branch target index.
        // ---------------------------------------------------------------
        const OPCODE_JUMP: u8 = 0x02;
        const OPCODE_JUMPIF: u8 = 0x03;
        const OPCODE_END: u8 = 0x01;

        for (pc, instr) in instructions.iter().enumerate() {
            if instr.opcode == OPCODE_JUMP || instr.opcode == OPCODE_JUMPIF {
                // Both Jump and JumpIf require at least one operand.
                if instr.operands.is_empty() {
                    return Err(BcibError::ControlFlowViolation(
                        "Jump/JumpIf instruction missing target operand",
                    ));
                }
                let target = instr.operands[0] as usize;
                if target >= n {
                    return Err(BcibError::ControlFlowViolation(
                        "jump target index is out of bounds",
                    ));
                }
                // Self-loop on an unconditional Jump is an infinite loop.
                if instr.opcode == OPCODE_JUMP && target == pc {
                    return Err(BcibError::ControlFlowViolation(
                        "unconditional jump to self creates an infinite loop",
                    ));
                }
            }
        }

        // ---------------------------------------------------------------
        // Step 2 — Build successor lists.
        // ---------------------------------------------------------------
        // successors[i] holds the indices of instructions that can be
        // reached directly from instruction i.
        let mut successors: Vec<Vec<usize>> = Vec::with_capacity(n);
        for (pc, instr) in instructions.iter().enumerate() {
            let succs = match instr.opcode {
                OPCODE_END => {
                    // End terminates execution — no successors.
                    vec![]
                }
                OPCODE_JUMP => {
                    // Unconditional branch — only the target.
                    let target = instr.operands[0] as usize;
                    vec![target]
                }
                OPCODE_JUMPIF => {
                    // Conditional branch — taken target + fall-through.
                    let target = instr.operands[0] as usize;
                    let fall_through = pc + 1;
                    if fall_through < n {
                        vec![target, fall_through]
                    } else {
                        // Last instruction is a JumpIf — fall-through would
                        // be out of bounds; only the taken branch is valid.
                        vec![target]
                    }
                }
                _ => {
                    // All other instructions fall through to pc+1.
                    if pc + 1 < n {
                        vec![pc + 1]
                    } else {
                        vec![]
                    }
                }
            };
            successors.push(succs);
        }

        // ---------------------------------------------------------------
        // Step 3 — Iterative DFS from instruction 0.
        //
        // We track two boolean arrays:
        //   visited[i]  — true once node i has been fully explored.
        //   on_stack[i] — true while node i is on the current DFS path.
        //
        // A back-edge (successor already on_stack) means a cycle.
        // ---------------------------------------------------------------
        let mut visited: Vec<bool> = vec![false; n];
        let mut on_stack: Vec<bool> = vec![false; n];

        // Iterative DFS using an explicit stack.
        // Each stack entry is (node_index, successor_iterator_position).
        let mut dfs_stack: Vec<(usize, usize)> = Vec::with_capacity(n);

        // Start from instruction 0.
        visited[0] = true;
        on_stack[0] = true;
        dfs_stack.push((0, 0));

        while let Some((node, succ_idx)) = dfs_stack.last_mut() {
            let node = *node;
            let succs = &successors[node];

            if *succ_idx < succs.len() {
                let next = succs[*succ_idx];
                *succ_idx += 1;

                if on_stack[next] {
                    // Back-edge detected — infinite loop.
                    return Err(BcibError::ControlFlowViolation(
                        "cycle detected in control-flow graph (infinite loop)",
                    ));
                }

                if !visited[next] {
                    visited[next] = true;
                    on_stack[next] = true;
                    dfs_stack.push((next, 0));
                }
                // If already visited but not on_stack, it is a cross/forward
                // edge — safe to skip.
            } else {
                // All successors of `node` have been explored — pop.
                on_stack[node] = false;
                dfs_stack.pop();
            }
        }

        // ---------------------------------------------------------------
        // Step 4 — Reachability check.
        //
        // Any instruction not visited by the DFS is unreachable dead code.
        // ---------------------------------------------------------------
        for (_, &was_visited) in visited.iter().enumerate() {
            if !was_visited {
                return Err(BcibError::ControlFlowViolation(
                    "unreachable instruction detected (dead code)",
                ));
            }
        }

        Ok(())
    }

    // -----------------------------------------------------------------------
    // Phase 3: Capability validation (Requirement 5.2, 16.5)
    // -----------------------------------------------------------------------

    /// Checks that every `DataMutating` and `External` instruction has a
    /// corresponding capability token in `capability_set`.
    ///
    /// For each instruction whose `side_effect_class` is `DataMutating` or
    /// `External`, this method iterates over all token IDs in `capability_set`
    /// and calls `CapabilityCheck::check()` for the appropriate resource type.
    /// At least one token must pass the check; if none do, the method returns
    /// `BCIB_ERR_CAPABILITY_DENIED` immediately (fail-fast, Requirement 5.2).
    ///
    /// The matching token ID is pre-bound into `instruction.required_capabilities`
    /// so the runtime can enforce the same check without re-scanning the set
    /// (Requirement 4.1 — plan/runtime consistency).
    ///
    /// `ctx_id = 0` is used during planning (no context exists yet). The
    /// `NoopCapabilityManager` stub ignores it; the real implementation
    /// (Group 7, Task 27) will enforce context binding at runtime.
    ///
    /// Kernel bypass is impossible: all checks go through the `CapabilityCheck`
    /// trait in Ring3 (Requirement 5.2, NON_OVERRIDABLE: KERNEL.CAPABILITY.BYPASS).
    fn verify_capabilities(
        &self,
        instructions: &mut Vec<BcibInstruction>,
        capability_set: &CapabilitySet,
    ) -> Result<(), BcibError> {
        // Use the NoopCapabilityManager stub (Group 1, Task 1.4).
        // Group 7 (Task 27) replaces this with the real CapabilityManager.
        let checker = NoopCapabilityManager;
        // Planning-time context ID: no real context exists yet.
        // The stub ignores this value; the real implementation enforces it at runtime.
        const PLANNING_CTX_ID: u64 = 0;

        for instr in instructions.iter_mut() {
            let resource = match instr.side_effect_class {
                SideEffectClass::Pure => continue, // Pure instructions need no capability.
                SideEffectClass::DataMutating => CapabilityResource::DataWrite,
                SideEffectClass::External => CapabilityResource::ExternalCall,
            };

            // Fail-fast: find the first token that passes the check.
            // If the capability set is empty or no token passes, deny immediately.
            let mut granted_token = None;
            for &token_id in &capability_set.token_ids {
                if checker.check(token_id, &resource, PLANNING_CTX_ID).is_ok() {
                    granted_token = Some(token_id);
                    break;
                }
            }

            match granted_token {
                Some(token_id) => {
                    // Pre-bind the token into the instruction for runtime enforcement.
                    instr.required_capabilities.push(token_id);
                }
                None => {
                    // No valid token found — fail-closed (Requirement 5.2).
                    return Err(BcibError::CapabilityDenied(
                        "no capability token covers this data-mutating or external instruction",
                    ));
                }
            }
        }

        Ok(())
    }

    // -----------------------------------------------------------------------
    // Phase 4: Bounds validation (Requirement 16.3, 3.5)
    // -----------------------------------------------------------------------

    /// Enforces index bounds and all `ResourceLimits` fields.
    ///
    /// Checks (in order, fail-fast):
    ///   1. Total instruction count ≤ `resource_limits.max_instruction_count`
    ///   2. Every operand index is within bounds of the instruction array length
    ///   3. Concurrent handle count (External instructions) ≤ `resource_limits.max_concurrent_handles`
    ///   4. AI instruction count ≤ `resource_limits.max_ai_quota`
    ///
    /// Note: `max_memory_per_context` is a runtime pool limit enforced by the
    /// bounded pool allocator (Group 4). At planning time we have no allocation
    /// size information, so we validate the other three limits here and leave
    /// memory enforcement to the runtime (Requirement 3.4, 16.3).
    ///
    /// Any violation → `BCIB_ERR_BOUNDS_VIOLATION` (fail-closed, Requirement 16.3, 3.5).
    fn verify_bounds(
        &self,
        instructions: &[BcibInstruction],
        resource_limits: &ResourceLimits,
    ) -> Result<(), BcibError> {
        let instruction_count = instructions.len();

        // --- Check 1: total instruction count ---
        if instruction_count > resource_limits.max_instruction_count {
            return Err(BcibError::BoundsViolation(
                "instruction count exceeds max_instruction_count limit",
            ));
        }

        // --- Check 2: operand index bounds ---
        // Each operand that is used as an instruction index must be < instruction_count.
        // Operands on control-flow opcodes (Jump=0x02, JumpIf=0x03) are instruction
        // indices; all other operands are data values and are not index-checked here.
        // Jump/JumpIf target validity was already verified in Phase 2 (control-flow),
        // but we re-enforce the raw numeric bound here as a defence-in-depth check.
        for instr in instructions {
            for &operand in &instr.operands {
                let idx = operand as usize;
                if idx >= instruction_count && instruction_count > 0 {
                    return Err(BcibError::BoundsViolation(
                        "operand index exceeds instruction array bounds",
                    ));
                }
            }
        }

        // --- Check 3: concurrent handle count (External instructions) ---
        let external_count = instructions
            .iter()
            .filter(|i| i.side_effect_class == SideEffectClass::External)
            .count();
        if external_count > resource_limits.max_concurrent_handles {
            return Err(BcibError::BoundsViolation(
                "external instruction count exceeds max_concurrent_handles limit",
            ));
        }

        // --- Check 4: AI instruction quota ---
        // AI instructions are External instructions whose opcode falls in the
        // ai class range (0x30–0x3F, Requirement 16.3, design.md §Opcode Registry).
        const AI_OPCODE_MIN: u8 = 0x30;
        const AI_OPCODE_MAX: u8 = 0x3F;
        let ai_count = instructions
            .iter()
            .filter(|i| i.opcode >= AI_OPCODE_MIN && i.opcode <= AI_OPCODE_MAX)
            .count();
        if ai_count > resource_limits.max_ai_quota {
            return Err(BcibError::BoundsViolation(
                "AI instruction count exceeds max_ai_quota limit",
            ));
        }

        Ok(())
    }
}

impl Default for BcibVerifierPlanner {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binary_format::{
        BCIB_VERSION_V02, BCIB_VERSION_V3, HEADER_SIZE, SECTION_ENTRY_SIZE,
    };

    // -----------------------------------------------------------------------
    // Helpers
    // -----------------------------------------------------------------------

    fn empty_caps() -> CapabilitySet {
        CapabilitySet::default()
    }

    fn default_limits() -> ResourceLimits {
        ResourceLimits::default()
    }

    fn planner() -> BcibVerifierPlanner {
        BcibVerifierPlanner::new()
    }

    /// Build a minimal valid v3 BCIB buffer with a given instruction payload.
    ///
    /// Layout:
    ///   [0..16]  header (magic=BCIB, version=v3, section_count=1)
    ///   [16..24] section table (1 entry: Instructions at offset 24)
    ///   [24..]   instruction bytes
    fn build_v3_buffer(instr_bytes: &[u8]) -> Vec<u8> {
        let instr_len = instr_bytes.len();
        let instr_offset: u32 = (HEADER_SIZE + SECTION_ENTRY_SIZE) as u32; // 24

        let mut buf = Vec::new();

        // Header (16 bytes)
        buf.extend_from_slice(b"BCIB");
        buf.extend_from_slice(&BCIB_VERSION_V3.to_le_bytes());
        buf.extend_from_slice(&0u16.to_le_bytes()); // flags
        buf.extend_from_slice(&1u16.to_le_bytes()); // section_count
        buf.extend_from_slice(&[0u8; 4]); // reserved
        buf.extend_from_slice(&[0u8; 2]); // header tail bytes 14-15

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

    // -----------------------------------------------------------------------
    // Structural validation tests (Requirements 16.1, 16.4)
    // -----------------------------------------------------------------------

    /// Valid v3 BCIB with a single Nop instruction → Ok.
    #[test]
    fn verify_structural_valid_nop() {
        let instr = encode_instr(0x00 /* Nop */, &[]);
        let buf = build_v3_buffer(&instr);
        let (instructions, version) = planner()
            .verify_structural(&buf)
            .expect("valid Nop should pass structural validation");
        assert_eq!(version, BCIB_VERSION_V3);
        assert_eq!(instructions.len(), 1);
        assert_eq!(instructions[0].opcode, 0x00);
    }

    /// Valid v3 BCIB with multiple instructions → correct count and side-effect classes.
    #[test]
    fn verify_structural_multiple_instructions() {
        let mut instr_bytes = Vec::new();
        instr_bytes.extend(encode_instr(0x00 /* Nop, Pure */, &[]));
        instr_bytes.extend(encode_instr(
            0x10, /* DataCreate, DataMutating */
            &[1, 2],
        ));
        instr_bytes.extend(encode_instr(0x30 /* AiAsk, External */, &[42]));

        let buf = build_v3_buffer(&instr_bytes);
        let (instructions, _) = planner()
            .verify_structural(&buf)
            .expect("valid multi-instruction buffer should pass");

        assert_eq!(instructions.len(), 3);
        assert_eq!(
            instructions[0].side_effect_class,
            crate::types::SideEffectClass::Pure
        );
        assert_eq!(
            instructions[1].side_effect_class,
            crate::types::SideEffectClass::DataMutating
        );
        assert_eq!(
            instructions[2].side_effect_class,
            crate::types::SideEffectClass::External
        );
    }

    /// Empty instruction section (zero instructions) → Ok with empty list.
    #[test]
    fn verify_structural_empty_instruction_section() {
        let buf = build_v3_buffer(&[]);
        let (instructions, _) = planner()
            .verify_structural(&buf)
            .expect("empty instruction section is valid");
        assert!(instructions.is_empty());
    }

    /// Invalid magic bytes → `BCIB_ERR_INVALID_GRAPH`.
    #[test]
    fn verify_structural_bad_magic() {
        let mut buf = build_v3_buffer(&[]);
        buf[0] = b'X'; // corrupt magic
        let err = planner().verify_structural(&buf).unwrap_err();
        assert!(
            matches!(err, BcibError::InvalidGraph(_)),
            "bad magic → InvalidGraph"
        );
    }

    /// Unsupported version → `BCIB_ERR_UNSUPPORTED_VERSION`.
    #[test]
    fn verify_structural_unsupported_version() {
        let mut buf = build_v3_buffer(&[]);
        let bad: u16 = 0x0001;
        buf[4..6].copy_from_slice(&bad.to_le_bytes());
        let err = planner().verify_structural(&buf).unwrap_err();
        assert!(matches!(err, BcibError::UnsupportedVersion(_)));
    }

    /// v0.2 version is accepted (backward-compat path).
    #[test]
    fn verify_structural_v02_accepted() {
        let instr = encode_instr(0x00, &[]);
        let mut buf = build_v3_buffer(&instr);
        buf[4..6].copy_from_slice(&BCIB_VERSION_V02.to_le_bytes());
        let (_, version) = planner()
            .verify_structural(&buf)
            .expect("v0.2 should be accepted");
        assert_eq!(version, BCIB_VERSION_V02);
    }

    /// Unknown opcode → `BCIB_ERR_INVALID_GRAPH` (fail-closed, Requirement 16.1).
    #[test]
    fn verify_structural_unknown_opcode() {
        let instr = encode_instr(0xFF /* unregistered */, &[]);
        let buf = build_v3_buffer(&instr);
        let err = planner().verify_structural(&buf).unwrap_err();
        assert!(
            matches!(err, BcibError::InvalidGraph(_)),
            "unknown opcode must produce BCIB_ERR_INVALID_GRAPH"
        );
    }

    /// Truncated instruction (opcode present but operand bytes missing) → `BCIB_ERR_INVALID_GRAPH`.
    #[test]
    fn verify_structural_truncated_instruction() {
        // Encode DataCreate (0x10) with 2 operands but only provide 1 byte of operand data.
        let mut instr_bytes = vec![0x10u8, 2u8]; // opcode=DataCreate, operand_count=2
        instr_bytes.extend_from_slice(&[0x01u8]); // only 1 byte instead of 8
        let buf = build_v3_buffer(&instr_bytes);
        let err = planner().verify_structural(&buf).unwrap_err();
        assert!(matches!(err, BcibError::InvalidGraph(_)));
    }

    /// SideEffectClass is correctly assigned from opcode registry (Requirement 16.4).
    #[test]
    fn verify_structural_side_effect_class_assigned() {
        let instr = encode_instr(0x50 /* TraceEmit, Pure */, &[]);
        let buf = build_v3_buffer(&instr);
        let (instructions, _) = planner().verify_structural(&buf).unwrap();
        assert_eq!(
            instructions[0].side_effect_class,
            crate::types::SideEffectClass::Pure
        );
        assert_eq!(instructions[0].cost, crate::types::COST_PURE);
    }

    /// Cost is correctly assigned from opcode registry.
    #[test]
    fn verify_structural_cost_assigned() {
        let mut instr_bytes = Vec::new();
        instr_bytes.extend(encode_instr(0x00 /* Nop, COST_PURE */, &[]));
        instr_bytes.extend(encode_instr(
            0x21, /* DataWrite, COST_DATA_MUTATING */
            &[],
        ));
        instr_bytes.extend(encode_instr(0x31 /* AiStream, COST_EXTERNAL */, &[]));

        let buf = build_v3_buffer(&instr_bytes);
        let (instructions, _) = planner().verify_structural(&buf).unwrap();

        assert_eq!(instructions[0].cost, crate::types::COST_PURE);
        assert_eq!(instructions[1].cost, crate::types::COST_DATA_MUTATING);
        assert_eq!(instructions[2].cost, crate::types::COST_EXTERNAL);
    }

    /// Missing Instructions section → `BCIB_ERR_INVALID_GRAPH`.
    #[test]
    fn verify_structural_missing_instructions_section() {
        // Build a buffer with a Capabilities section instead of Instructions.
        let cap_offset: u32 = (HEADER_SIZE + SECTION_ENTRY_SIZE) as u32;
        let cap_data = [0u8; 4];

        let mut buf = Vec::new();
        buf.extend_from_slice(b"BCIB");
        buf.extend_from_slice(&BCIB_VERSION_V3.to_le_bytes());
        buf.extend_from_slice(&0u16.to_le_bytes());
        buf.extend_from_slice(&1u16.to_le_bytes()); // section_count = 1
        buf.extend_from_slice(&[0u8; 4]);
        buf.extend_from_slice(&[0u8; 2]);

        // Section entry: Capabilities (0x02), not Instructions (0x01)
        buf.extend_from_slice(&(SectionId::Capabilities as u16).to_le_bytes());
        buf.extend_from_slice(&cap_offset.to_le_bytes());
        buf.extend_from_slice(&(cap_data.len() as u16).to_le_bytes());
        buf.extend_from_slice(&cap_data);

        let err = planner().verify_structural(&buf).unwrap_err();
        assert!(matches!(err, BcibError::InvalidGraph(_)));
    }

    /// Overlapping sections → `BCIB_ERR_INVALID_GRAPH`.
    #[test]
    fn verify_structural_overlapping_sections() {
        // Build a buffer with two sections that overlap.
        let base_offset: u32 = (HEADER_SIZE + 2 * SECTION_ENTRY_SIZE) as u32; // 32

        let mut buf = Vec::new();
        buf.extend_from_slice(b"BCIB");
        buf.extend_from_slice(&BCIB_VERSION_V3.to_le_bytes());
        buf.extend_from_slice(&0u16.to_le_bytes());
        buf.extend_from_slice(&2u16.to_le_bytes()); // section_count = 2
        buf.extend_from_slice(&[0u8; 4]);
        buf.extend_from_slice(&[0u8; 2]);

        // Section 1: Instructions at offset 32, length 4
        buf.extend_from_slice(&(SectionId::Instructions as u16).to_le_bytes());
        buf.extend_from_slice(&base_offset.to_le_bytes());
        buf.extend_from_slice(&4u16.to_le_bytes());

        // Section 2: Capabilities at offset 34 (overlaps with section 1 which ends at 36)
        let overlap_offset = base_offset + 2; // starts inside section 1
        buf.extend_from_slice(&(SectionId::Capabilities as u16).to_le_bytes());
        buf.extend_from_slice(&overlap_offset.to_le_bytes());
        buf.extend_from_slice(&4u16.to_le_bytes());

        // Payload (enough bytes for both sections)
        buf.extend_from_slice(&[0u8; 8]);

        let err = planner().verify_structural(&buf).unwrap_err();
        assert!(
            matches!(err, BcibError::InvalidGraph(_)),
            "overlapping sections must be rejected"
        );
    }

    // -----------------------------------------------------------------------
    // Control-flow validation tests (Requirements 16.1, 16.2)
    // Task 8.1
    // -----------------------------------------------------------------------

    /// Helper: build a BcibInstruction directly (bypasses binary encoding).
    fn make_instr(opcode: u8, operands: &[u32]) -> BcibInstruction {
        use crate::opcode_registry::lookup_opcode;
        let desc = lookup_opcode(opcode).expect("test opcode must be registered");
        BcibInstruction {
            opcode,
            operands: operands.to_vec(),
            side_effect_class: desc.side_effect,
            cost: desc.cost,
            required_capabilities: Vec::new(),
        }
    }

    /// A single Nop (no jumps, no cycles) → Ok.
    #[test]
    fn verify_control_flow_single_nop_ok() {
        let instrs = vec![make_instr(0x00 /* Nop */, &[])];
        planner()
            .verify_control_flow(&instrs)
            .expect("single Nop has no control-flow issues");
    }

    /// Empty instruction list → Ok (trivially valid).
    #[test]
    fn verify_control_flow_empty_ok() {
        planner()
            .verify_control_flow(&[])
            .expect("empty instruction list is valid");
    }

    /// Linear sequence: Nop → Nop → End → Ok.
    #[test]
    fn verify_control_flow_linear_sequence_ok() {
        let instrs = vec![
            make_instr(0x00 /* Nop */, &[]),
            make_instr(0x00 /* Nop */, &[]),
            make_instr(0x01 /* End */, &[]),
        ];
        planner()
            .verify_control_flow(&instrs)
            .expect("linear sequence is valid");
    }

    /// Unconditional Jump to self → infinite loop → BCIB_ERR_CONTROL_FLOW_VIOLATION.
    #[test]
    fn verify_control_flow_self_loop_jump() {
        // Instruction 0: Jump to 0 (self-loop)
        let instrs = vec![make_instr(0x02 /* Jump */, &[0])];
        let err = planner().verify_control_flow(&instrs).unwrap_err();
        assert!(
            matches!(err, BcibError::ControlFlowViolation(_)),
            "self-loop Jump must produce BCIB_ERR_CONTROL_FLOW_VIOLATION"
        );
    }

    /// Cycle: instruction 0 jumps to 1, instruction 1 jumps back to 0
    /// → infinite loop → BCIB_ERR_CONTROL_FLOW_VIOLATION.
    #[test]
    fn verify_control_flow_two_node_cycle() {
        // 0: Jump → 1
        // 1: Jump → 0
        let instrs = vec![
            make_instr(0x02 /* Jump */, &[1]),
            make_instr(0x02 /* Jump */, &[0]),
        ];
        let err = planner().verify_control_flow(&instrs).unwrap_err();
        assert!(
            matches!(err, BcibError::ControlFlowViolation(_)),
            "two-node cycle must produce BCIB_ERR_CONTROL_FLOW_VIOLATION"
        );
    }

    /// Unreachable instruction: Nop at index 0 jumps over index 1 to End at index 2.
    /// Index 1 is never reachable → BCIB_ERR_CONTROL_FLOW_VIOLATION.
    #[test]
    fn verify_control_flow_unreachable_instruction() {
        // 0: Jump → 2
        // 1: Nop  (unreachable)
        // 2: End
        let instrs = vec![
            make_instr(0x02 /* Jump */, &[2]),
            make_instr(0x00 /* Nop  */, &[]),
            make_instr(0x01 /* End  */, &[]),
        ];
        let err = planner().verify_control_flow(&instrs).unwrap_err();
        assert!(
            matches!(err, BcibError::ControlFlowViolation(_)),
            "unreachable instruction must produce BCIB_ERR_CONTROL_FLOW_VIOLATION"
        );
    }

    /// Jump target out of bounds → BCIB_ERR_CONTROL_FLOW_VIOLATION.
    #[test]
    fn verify_control_flow_invalid_jump_target() {
        // 0: Jump → 99 (only 1 instruction exists)
        let instrs = vec![make_instr(0x02 /* Jump */, &[99])];
        let err = planner().verify_control_flow(&instrs).unwrap_err();
        assert!(
            matches!(err, BcibError::ControlFlowViolation(_)),
            "out-of-bounds jump target must produce BCIB_ERR_CONTROL_FLOW_VIOLATION"
        );
    }

    /// JumpIf with out-of-bounds taken-branch target → BCIB_ERR_CONTROL_FLOW_VIOLATION.
    #[test]
    fn verify_control_flow_jumpif_invalid_target() {
        // 0: JumpIf → 99 (out of bounds)
        // 1: End
        let instrs = vec![
            make_instr(0x03 /* JumpIf */, &[99]),
            make_instr(0x01 /* End   */, &[]),
        ];
        let err = planner().verify_control_flow(&instrs).unwrap_err();
        assert!(
            matches!(err, BcibError::ControlFlowViolation(_)),
            "JumpIf with out-of-bounds target must produce BCIB_ERR_CONTROL_FLOW_VIOLATION"
        );
    }

    /// Valid forward Jump: Nop → Jump(2) → [skipped] → End.
    /// All instructions reachable via JumpIf fall-through path.
    #[test]
    fn verify_control_flow_jumpif_both_paths_reachable() {
        // 0: JumpIf → 2   (taken: go to 2; not-taken: fall to 1)
        // 1: Nop           (fall-through path)
        // 2: End           (taken path)
        let instrs = vec![
            make_instr(0x03 /* JumpIf */, &[2]),
            make_instr(0x00 /* Nop   */, &[]),
            make_instr(0x01 /* End   */, &[]),
        ];
        planner()
            .verify_control_flow(&instrs)
            .expect("JumpIf with both paths reachable is valid");
    }

    /// Jump missing operand → BCIB_ERR_CONTROL_FLOW_VIOLATION.
    #[test]
    fn verify_control_flow_jump_missing_operand() {
        // Manually construct a Jump with no operands (invalid encoding).
        let instr = BcibInstruction {
            opcode: 0x02, // Jump
            operands: vec![],
            side_effect_class: crate::types::SideEffectClass::Pure,
            cost: crate::types::COST_PURE,
            required_capabilities: vec![],
        };
        let err = planner().verify_control_flow(&[instr]).unwrap_err();
        assert!(
            matches!(err, BcibError::ControlFlowViolation(_)),
            "Jump without operand must produce BCIB_ERR_CONTROL_FLOW_VIOLATION"
        );
    }

    /// Full verify_and_plan() rejects a program with an infinite loop.
    /// Confirms fail-fast ordering: structural passes, control-flow fails.
    #[test]
    fn verify_and_plan_rejects_infinite_loop() {
        // Build a valid v3 buffer: Jump → self (index 0)
        let instr = encode_instr(0x02 /* Jump */, &[0u32]);
        let buf = build_v3_buffer(&instr);
        let err = planner()
            .verify_and_plan(&buf, &empty_caps(), &default_limits())
            .unwrap_err();
        assert!(
            matches!(err, BcibError::ControlFlowViolation(_)),
            "verify_and_plan must reject infinite loop with BCIB_ERR_CONTROL_FLOW_VIOLATION"
        );
    }

    /// Full verify_and_plan() rejects a program with an unreachable instruction.
    #[test]
    fn verify_and_plan_rejects_unreachable_instruction() {
        // 0: Jump → 2
        // 1: Nop  (unreachable)
        // 2: End
        let mut instr_bytes = Vec::new();
        instr_bytes.extend(encode_instr(0x02 /* Jump */, &[2u32]));
        instr_bytes.extend(encode_instr(0x00 /* Nop  */, &[]));
        instr_bytes.extend(encode_instr(0x01 /* End  */, &[]));
        let buf = build_v3_buffer(&instr_bytes);
        let err = planner()
            .verify_and_plan(&buf, &empty_caps(), &default_limits())
            .unwrap_err();
        assert!(
            matches!(err, BcibError::ControlFlowViolation(_)),
            "verify_and_plan must reject unreachable instruction"
        );
    }

    // -----------------------------------------------------------------------
    // Capability validation tests (Requirements 5.2, 16.5)
    // Task 9
    // -----------------------------------------------------------------------

    /// Pure-only program with empty capability set → Ok (no capability needed).
    #[test]
    fn verify_capabilities_pure_only_no_caps_needed() {
        let instr = encode_instr(0x00 /* Nop, Pure */, &[]);
        let buf = build_v3_buffer(&instr);
        // Empty capability set is fine for Pure-only programs.
        planner()
            .verify_and_plan(&buf, &empty_caps(), &default_limits())
            .expect("Pure-only program needs no capability tokens");
    }

    /// DataMutating instruction with empty capability set → BCIB_ERR_CAPABILITY_DENIED.
    #[test]
    fn verify_capabilities_data_mutating_empty_caps_denied() {
        let instr = encode_instr(0x10 /* DataCreate, DataMutating */, &[]);
        let buf = build_v3_buffer(&instr);
        let err = planner()
            .verify_and_plan(&buf, &empty_caps(), &default_limits())
            .unwrap_err();
        assert!(
            matches!(err, BcibError::CapabilityDenied(_)),
            "DataMutating instruction with empty caps must produce BCIB_ERR_CAPABILITY_DENIED"
        );
    }

    /// External instruction with empty capability set → BCIB_ERR_CAPABILITY_DENIED.
    #[test]
    fn verify_capabilities_external_empty_caps_denied() {
        let instr = encode_instr(0x30 /* AiAsk, External */, &[]);
        let buf = build_v3_buffer(&instr);
        let err = planner()
            .verify_and_plan(&buf, &empty_caps(), &default_limits())
            .unwrap_err();
        assert!(
            matches!(err, BcibError::CapabilityDenied(_)),
            "External instruction with empty caps must produce BCIB_ERR_CAPABILITY_DENIED"
        );
    }

    /// DataMutating instruction with a valid token → Ok; token pre-bound in plan.
    #[test]
    fn verify_capabilities_data_mutating_with_token_ok() {
        let instr = encode_instr(0x10 /* DataCreate, DataMutating */, &[]);
        let buf = build_v3_buffer(&instr);
        let caps = CapabilitySet {
            token_ids: vec![42],
        };
        let plan = planner()
            .verify_and_plan(&buf, &caps, &default_limits())
            .expect("DataMutating with valid token must pass");
        // Token must be pre-bound into the instruction's required_capabilities.
        assert_eq!(
            plan.instructions()[0].required_capabilities,
            vec![42],
            "granted token must be pre-bound into required_capabilities"
        );
    }

    /// External instruction with a valid token → Ok; token pre-bound in plan.
    #[test]
    fn verify_capabilities_external_with_token_ok() {
        let instr = encode_instr(0x30 /* AiAsk, External */, &[]);
        let buf = build_v3_buffer(&instr);
        let caps = CapabilitySet {
            token_ids: vec![99],
        };
        let plan = planner()
            .verify_and_plan(&buf, &caps, &default_limits())
            .expect("External with valid token must pass");
        assert_eq!(plan.instructions()[0].required_capabilities, vec![99]);
    }

    /// Fail-fast: first DataMutating instruction fails → second is never checked.
    /// The error is returned immediately on the first missing token.
    #[test]
    fn verify_capabilities_fail_fast_on_first_missing() {
        // Two DataMutating instructions; empty caps → fails on the first one.
        let mut instr_bytes = Vec::new();
        instr_bytes.extend(encode_instr(0x10 /* DataCreate */, &[]));
        instr_bytes.extend(encode_instr(0x11 /* DataAdd    */, &[]));
        let buf = build_v3_buffer(&instr_bytes);
        let err = planner()
            .verify_and_plan(&buf, &empty_caps(), &default_limits())
            .unwrap_err();
        assert!(
            matches!(err, BcibError::CapabilityDenied(_)),
            "must fail-fast on first DataMutating instruction without a token"
        );
    }

    /// Pure instructions are skipped; only DataMutating/External are checked.
    #[test]
    fn verify_capabilities_pure_skipped_non_pure_checked() {
        // Nop (Pure) followed by DataCreate (DataMutating).
        // Empty caps → fails on DataCreate, not on Nop.
        let mut instr_bytes = Vec::new();
        instr_bytes.extend(encode_instr(0x00 /* Nop        */, &[]));
        instr_bytes.extend(encode_instr(0x10 /* DataCreate */, &[]));
        let buf = build_v3_buffer(&instr_bytes);
        let err = planner()
            .verify_and_plan(&buf, &empty_caps(), &default_limits())
            .unwrap_err();
        assert!(
            matches!(err, BcibError::CapabilityDenied(_)),
            "Pure instruction must be skipped; DataMutating must be checked"
        );
    }

    // -----------------------------------------------------------------------
    // Phase 4: Bounds validation tests (Task 10.1)
    // Requirements: 16.3, 3.5
    // -----------------------------------------------------------------------

    /// Instruction count within limit → Ok.
    #[test]
    fn verify_bounds_within_limit_ok() {
        let instr = encode_instr(0x00 /* Nop */, &[]);
        let buf = build_v3_buffer(&instr);
        let limits = ResourceLimits {
            max_instruction_count: 10,
            ..default_limits()
        };
        planner()
            .verify_and_plan(&buf, &empty_caps(), &limits)
            .expect("single instruction within limit must pass");
    }

    /// Instruction count exceeds max_instruction_count → BCIB_ERR_BOUNDS_VIOLATION.
    /// Requirements: 16.3, 3.5
    #[test]
    fn verify_bounds_max_instruction_count_exceeded() {
        // Build a buffer with 3 Nop instructions.
        let mut instr_bytes = Vec::new();
        for _ in 0..3 {
            instr_bytes.extend(encode_instr(0x00 /* Nop */, &[]));
        }
        let buf = build_v3_buffer(&instr_bytes);
        // Set limit to 2 — 3 instructions must be rejected.
        let limits = ResourceLimits {
            max_instruction_count: 2,
            ..default_limits()
        };
        let err = planner()
            .verify_and_plan(&buf, &empty_caps(), &limits)
            .unwrap_err();
        assert!(
            matches!(err, BcibError::BoundsViolation(_)),
            "instruction count exceeding max_instruction_count must produce BCIB_ERR_BOUNDS_VIOLATION, got: {:?}",
            err
        );
    }

    /// Operand index that equals instruction count → BCIB_ERR_BOUNDS_VIOLATION.
    /// Requirements: 16.3, 3.5
    #[test]
    fn verify_bounds_invalid_operand_index() {
        // Single Nop with operand 99 — instruction array has length 1, so index 99 is OOB.
        let instr = encode_instr(0x00 /* Nop */, &[99]);
        let buf = build_v3_buffer(&instr);
        let err = planner()
            .verify_and_plan(&buf, &empty_caps(), &default_limits())
            .unwrap_err();
        assert!(
            matches!(err, BcibError::BoundsViolation(_)),
            "operand index >= instruction count must produce BCIB_ERR_BOUNDS_VIOLATION, got: {:?}",
            err
        );
    }

    /// Operand index 0 with a single instruction → valid (index 0 < length 1).
    #[test]
    fn verify_bounds_operand_index_zero_single_instr_ok() {
        // Operand 0 is within bounds for a 1-instruction array.
        let instr = encode_instr(0x00 /* Nop */, &[0]);
        let buf = build_v3_buffer(&instr);
        planner()
            .verify_and_plan(&buf, &empty_caps(), &default_limits())
            .expect("operand index 0 with 1 instruction must be within bounds");
    }

    /// AI instruction count exceeds max_ai_quota → BCIB_ERR_BOUNDS_VIOLATION.
    /// Requirements: 16.3, 3.5
    #[test]
    fn verify_bounds_max_ai_quota_exceeded() {
        // Two AiAsk (0x30) instructions; quota = 1.
        let caps = CapabilitySet { token_ids: vec![1] };
        let mut instr_bytes = Vec::new();
        instr_bytes.extend(encode_instr(0x30 /* AiAsk */, &[]));
        instr_bytes.extend(encode_instr(0x30 /* AiAsk */, &[]));
        let buf = build_v3_buffer(&instr_bytes);
        let limits = ResourceLimits {
            max_ai_quota: 1,
            max_concurrent_handles: 64,
            ..default_limits()
        };
        let err = planner().verify_and_plan(&buf, &caps, &limits).unwrap_err();
        assert!(
            matches!(err, BcibError::BoundsViolation(_)),
            "AI instruction count exceeding max_ai_quota must produce BCIB_ERR_BOUNDS_VIOLATION, got: {:?}",
            err
        );
    }

    /// External instruction count exceeds max_concurrent_handles → BCIB_ERR_BOUNDS_VIOLATION.
    /// Requirements: 16.3, 3.5
    #[test]
    fn verify_bounds_max_concurrent_handles_exceeded() {
        // Two External (UiRender 0x40) instructions; handle limit = 1.
        let caps = CapabilitySet { token_ids: vec![1] };
        let mut instr_bytes = Vec::new();
        instr_bytes.extend(encode_instr(0x40 /* UiRender */, &[]));
        instr_bytes.extend(encode_instr(0x40 /* UiRender */, &[]));
        let buf = build_v3_buffer(&instr_bytes);
        let limits = ResourceLimits {
            max_concurrent_handles: 1,
            max_ai_quota: 8,
            ..default_limits()
        };
        let err = planner().verify_and_plan(&buf, &caps, &limits).unwrap_err();
        assert!(
            matches!(err, BcibError::BoundsViolation(_)),
            "external count exceeding max_concurrent_handles must produce BCIB_ERR_BOUNDS_VIOLATION, got: {:?}",
            err
        );
    }

    /// All limits satisfied with mixed instructions → Ok.
    #[test]
    fn verify_bounds_all_limits_satisfied_ok() {
        let caps = CapabilitySet { token_ids: vec![1] };
        let mut instr_bytes = Vec::new();
        instr_bytes.extend(encode_instr(0x00 /* Nop    */, &[]));
        instr_bytes.extend(encode_instr(0x30 /* AiAsk  */, &[]));
        instr_bytes.extend(encode_instr(0x40 /* UiRender */, &[]));
        let buf = build_v3_buffer(&instr_bytes);
        let limits = ResourceLimits {
            max_instruction_count: 10,
            max_concurrent_handles: 5,
            max_ai_quota: 5,
            ..default_limits()
        };
        planner()
            .verify_and_plan(&buf, &caps, &limits)
            .expect("all limits satisfied must pass");
    }

    // -----------------------------------------------------------------------
    // Task 41.1 — Property 1: Execution Determinism
    // Feature: phase15-bcib-execution-engine, Property 1: Execution Determinism
    // Validates: Requirements 4.1, 4.4
    // -----------------------------------------------------------------------
    //
    // Generator: random valid BCIB graph (pure-only instructions, no capability
    //            required) + fixed environment conditions (empty caps, default limits).
    // Assertion: verify_and_plan() called twice with identical inputs produces
    //            identical ExecutionPlan::canonical_hash() values.
    //            Divergence → DETERMINISM.GLOBAL violation.

    proptest::proptest! {
        // Feature: phase15-bcib-execution-engine, Property 1: Execution Determinism
        // Validates: Requirements 4.1, 4.4
        #![proptest_config(proptest::prelude::ProptestConfig::with_cases(100))]
        #[test]
        fn prop_execution_determinism(
            // Generate 0–8 pure instructions (Nop = 0x00, End = 0x01).
            // Using only pure opcodes avoids capability requirements so the
            // graph is always valid with an empty CapabilitySet.
            opcodes in proptest::collection::vec(
                proptest::strategy::Just(0x00u8), // Nop — always Pure, always valid
                0usize..=8,
            ),
        ) {
            let mut instr_bytes = Vec::new();
            for opcode in &opcodes {
                instr_bytes.extend(encode_instr(*opcode, &[]));
            }
            // Always append End (0x01) so the graph has a proper terminator.
            instr_bytes.extend(encode_instr(0x01 /* End */, &[]));

            let buf = build_v3_buffer(&instr_bytes);
            let caps = empty_caps();
            let limits = default_limits();
            let p = planner();

            // First call
            let plan_a = p.verify_and_plan(&buf, &caps, &limits)
                .expect("valid graph must produce a plan");

            // Second call — identical inputs, fixed environment
            let plan_b = p.verify_and_plan(&buf, &caps, &limits)
                .expect("second call with same inputs must also succeed");

            // Both plans must produce the same canonical hash (DETERMINISM.GLOBAL).
            proptest::prop_assert_eq!(
                plan_a.canonical_hash(),
                plan_b.canonical_hash(),
                "DETERMINISM.GLOBAL violation: same graph produced different canonical_hash \
                 ({} vs {})",
                plan_a.canonical_hash(),
                plan_b.canonical_hash(),
            );

            // Also verify instruction count is identical.
            proptest::prop_assert_eq!(
                plan_a.instructions().len(),
                plan_b.instructions().len(),
                "instruction count must be identical across two calls with same input",
            );
        }
    }
}
