//! ExecutionPlan Validator Implementation
//!
//! **Created By:** Kenan AY
//! **Date:** 16 Ocak 2026
//! **Architectural Reference:** C2 Section "Testing Strategy"
//!
//! Validates ExecutionPlan IR for correctness, determinism, and architectural compliance.

use super::dataflow::DataflowError;
use super::{BlockId, BlockTerminator, ExecutionPlan, IRBlock, IRInstruction, RegisterId};
use std::collections::HashSet;

/// ExecutionPlan validator
pub struct ExecutionPlanValidator {
    /// Validation configuration
    config: ValidationConfig,
}

impl ExecutionPlanValidator {
    /// Create new validator with default configuration
    pub fn new() -> Self {
        Self {
            config: ValidationConfig::default(),
        }
    }

    /// Create validator with custom configuration
    pub fn with_config(config: ValidationConfig) -> Self {
        Self { config }
    }

    /// Validate complete execution plan
    pub fn validate(&self, plan: &ExecutionPlan) -> Result<ValidationReport, ValidationError> {
        let mut report = ValidationReport::new();

        // 1. Validate basic structure
        self.validate_structure(plan, &mut report)?;

        // 2. Validate control flow
        self.validate_control_flow(plan, &mut report)?;

        // 3. Validate register usage
        self.validate_register_usage(plan, &mut report)?;

        // 4. Validate dataflow
        self.validate_dataflow(plan, &mut report)?;

        // 5. Validate determinism
        if self.config.check_determinism {
            self.validate_determinism(plan, &mut report)?;
        }

        // 6. Validate architectural compliance
        self.validate_architectural_compliance(plan, &mut report)?;

        Ok(report)
    }

    /// Validate basic execution plan structure
    fn validate_structure(
        &self,
        plan: &ExecutionPlan,
        report: &mut ValidationReport,
    ) -> Result<(), ValidationError> {
        // Check entry block exists
        if plan.get_block(plan.entry_block).is_none() {
            return Err(ValidationError::InvalidEntryBlock {
                block_id: plan.entry_block,
            });
        }

        // Check all blocks are valid
        for block in &plan.blocks {
            self.validate_block_structure(block, report)?;
        }

        // Check for duplicate block IDs
        let mut block_ids = HashSet::new();
        for block in &plan.blocks {
            if !block_ids.insert(block.id) {
                return Err(ValidationError::DuplicateBlockId { block_id: block.id });
            }
        }

        report.structure_valid = true;
        Ok(())
    }

    /// Validate individual block structure
    fn validate_block_structure(
        &self,
        block: &IRBlock,
        _report: &mut ValidationReport,
    ) -> Result<(), ValidationError> {
        // Check block is not empty (must have terminator at minimum)
        if block.instructions.is_empty() && matches!(block.terminator, BlockTerminator::Jump { .. })
        {
            return Err(ValidationError::EmptyBlock { block_id: block.id });
        }

        // Check no terminator instructions in instruction list
        for (i, instruction) in block.instructions.iter().enumerate() {
            if instruction.is_terminator() {
                return Err(ValidationError::TerminatorInInstructions {
                    block_id: block.id,
                    instruction_index: i,
                });
            }
        }

        // Validate each instruction
        for instruction in &block.instructions {
            self.validate_instruction(instruction)?;
        }

        Ok(())
    }

    /// Validate individual instruction
    fn validate_instruction(&self, instruction: &IRInstruction) -> Result<(), ValidationError> {
        match instruction {
            IRInstruction::LoadContext { context_id, .. } => {
                if context_id.is_empty() {
                    return Err(ValidationError::InvalidInstruction {
                        reason: "LoadContext context_id cannot be empty".to_string(),
                    });
                }
            }
            IRInstruction::LoadField { field_name, .. } => {
                if field_name.is_empty() {
                    return Err(ValidationError::InvalidInstruction {
                        reason: "LoadField field_name cannot be empty".to_string(),
                    });
                }
            }
            IRInstruction::LogicalOp {
                operand_registers, ..
            } => {
                if operand_registers.is_empty() {
                    return Err(ValidationError::InvalidInstruction {
                        reason: "LogicalOp must have at least one operand".to_string(),
                    });
                }
            }
            _ => {
                // Other instructions are structurally valid by construction
            }
        }

        Ok(())
    }

    /// Validate control flow graph
    fn validate_control_flow(
        &self,
        plan: &ExecutionPlan,
        report: &mut ValidationReport,
    ) -> Result<(), ValidationError> {
        let mut reachable_blocks = HashSet::new();
        let mut to_visit = vec![plan.entry_block];

        // Find all reachable blocks
        while let Some(block_id) = to_visit.pop() {
            if reachable_blocks.contains(&block_id) {
                continue;
            }

            reachable_blocks.insert(block_id);

            if let Some(block) = plan.get_block(block_id) {
                match &block.terminator {
                    BlockTerminator::Branch {
                        true_block,
                        false_block,
                        ..
                    } => {
                        // Validate branch targets exist
                        if plan.get_block(*true_block).is_none() {
                            return Err(ValidationError::InvalidBranchTarget {
                                source_block: block_id,
                                target_block: *true_block,
                            });
                        }
                        if plan.get_block(*false_block).is_none() {
                            return Err(ValidationError::InvalidBranchTarget {
                                source_block: block_id,
                                target_block: *false_block,
                            });
                        }

                        to_visit.push(*true_block);
                        to_visit.push(*false_block);
                    }
                    BlockTerminator::Jump { target_block } => {
                        // Validate jump target exists
                        if plan.get_block(*target_block).is_none() {
                            return Err(ValidationError::InvalidJumpTarget {
                                source_block: block_id,
                                target_block: *target_block,
                            });
                        }

                        to_visit.push(*target_block);
                    }
                    BlockTerminator::Return { .. } => {
                        // Return terminators are always valid
                    }
                }
            }
        }

        // Check for unreachable blocks
        let unreachable_blocks: Vec<_> = plan
            .blocks
            .iter()
            .filter(|block| !reachable_blocks.contains(&block.id))
            .map(|block| block.id)
            .collect();

        if !unreachable_blocks.is_empty() && self.config.warn_unreachable_blocks {
            report.warnings.push(ValidationWarning::UnreachableBlocks {
                block_ids: unreachable_blocks,
            });
        }

        report.control_flow_valid = true;
        Ok(())
    }

    /// Validate register usage
    fn validate_register_usage(
        &self,
        plan: &ExecutionPlan,
        report: &mut ValidationReport,
    ) -> Result<(), ValidationError> {
        let mut defined_registers = HashSet::new();
        let mut used_registers = HashSet::new();

        // Collect register definitions and uses
        for block in &plan.blocks {
            for instruction in &block.instructions {
                // Check input registers are defined
                for input_reg in instruction.input_registers() {
                    used_registers.insert(input_reg);
                    if !defined_registers.contains(&input_reg)
                        && self.config.strict_register_checking
                    {
                        return Err(ValidationError::UndefinedRegister {
                            register: input_reg,
                            block_id: block.id,
                        });
                    }
                }

                // Add output registers to defined set
                for output_reg in instruction.output_registers() {
                    if defined_registers.contains(&output_reg) && self.config.single_assignment {
                        return Err(ValidationError::RegisterRedefinition {
                            register: output_reg,
                            block_id: block.id,
                        });
                    }
                    defined_registers.insert(output_reg);
                }
            }

            // Check terminator register usage
            match &block.terminator {
                BlockTerminator::Return { register } => {
                    used_registers.insert(*register);
                    if !defined_registers.contains(register) && self.config.strict_register_checking
                    {
                        return Err(ValidationError::UndefinedRegister {
                            register: *register,
                            block_id: block.id,
                        });
                    }
                }
                BlockTerminator::Branch { condition, .. } => {
                    used_registers.insert(*condition);
                    if !defined_registers.contains(condition)
                        && self.config.strict_register_checking
                    {
                        return Err(ValidationError::UndefinedRegister {
                            register: *condition,
                            block_id: block.id,
                        });
                    }
                }
                BlockTerminator::Jump { .. } => {
                    // Jump doesn't use registers
                }
            }
        }

        // Check for unused registers
        let unused_registers: Vec<_> = defined_registers
            .iter()
            .filter(|reg| !used_registers.contains(reg))
            .copied()
            .collect();

        if !unused_registers.is_empty() && self.config.warn_unused_registers {
            report.warnings.push(ValidationWarning::UnusedRegisters {
                registers: unused_registers,
            });
        }

        report.register_usage_valid = true;
        Ok(())
    }

    /// Validate dataflow graph
    fn validate_dataflow(
        &self,
        plan: &ExecutionPlan,
        report: &mut ValidationReport,
    ) -> Result<(), ValidationError> {
        plan.dataflow_graph
            .validate()
            .map_err(|e| ValidationError::DataflowError { source: e })?;

        // Check for circular dependencies
        if plan.dataflow_graph.has_circular_dependencies() {
            return Err(ValidationError::CircularDependency);
        }

        report.dataflow_valid = true;
        Ok(())
    }

    /// Validate determinism properties
    fn validate_determinism(
        &self,
        plan: &ExecutionPlan,
        report: &mut ValidationReport,
    ) -> Result<(), ValidationError> {
        // Check determinism fingerprint is present
        if plan.metadata.determinism_fingerprint.is_empty() {
            return Err(ValidationError::MissingDeterminismFingerprint);
        }

        // Check for non-deterministic instructions
        for block in &plan.blocks {
            for instruction in &block.instructions {
                if self.is_non_deterministic_instruction(instruction) {
                    return Err(ValidationError::NonDeterministicInstruction {
                        block_id: block.id,
                        instruction: format!("{:?}", instruction),
                    });
                }
            }
        }

        report.determinism_valid = true;
        Ok(())
    }

    /// Check if instruction is non-deterministic
    fn is_non_deterministic_instruction(&self, _instruction: &IRInstruction) -> bool {
        // For Gate C, all instructions are deterministic
        // Future gates may add non-deterministic instructions (random, time, etc.)
        false
    }

    /// Validate architectural compliance
    fn validate_architectural_compliance(
        &self,
        plan: &ExecutionPlan,
        report: &mut ValidationReport,
    ) -> Result<(), ValidationError> {
        // Check Gate C architectural constraints

        // 1. No loops (Gate C constraint)
        if self.has_loops(plan) {
            return Err(ValidationError::LoopsNotAllowed);
        }

        // 2. Flat instruction graph (no nested expressions)
        if !self.is_flat_instruction_graph(plan) {
            return Err(ValidationError::NestedExpressionsNotAllowed);
        }

        // 3. Single assignment (SSA-like)
        if self.config.single_assignment {
            // Already checked in register validation
        }

        report.architectural_compliance_valid = true;
        Ok(())
    }

    /// Check for loops in control flow
    fn has_loops(&self, plan: &ExecutionPlan) -> bool {
        // Simple loop detection: check if any block can reach itself
        for block in &plan.blocks {
            if self.can_reach_self(plan, block.id, &mut HashSet::new()) {
                return true;
            }
        }
        false
    }

    /// Check if block can reach itself (loop detection)
    fn can_reach_self(
        &self,
        plan: &ExecutionPlan,
        start_block: BlockId,
        visited: &mut HashSet<BlockId>,
    ) -> bool {
        if visited.contains(&start_block) {
            return true; // Found a cycle
        }

        visited.insert(start_block);

        if let Some(block) = plan.get_block(start_block) {
            match &block.terminator {
                BlockTerminator::Branch {
                    true_block,
                    false_block,
                    ..
                } => {
                    if self.can_reach_self(plan, *true_block, visited)
                        || self.can_reach_self(plan, *false_block, visited)
                    {
                        return true;
                    }
                }
                BlockTerminator::Jump { target_block } => {
                    if self.can_reach_self(plan, *target_block, visited) {
                        return true;
                    }
                }
                BlockTerminator::Return { .. } => {
                    // Return terminators don't continue execution
                }
            }
        }

        visited.remove(&start_block);
        false
    }

    /// Check if execution plan uses flat instruction graph
    fn is_flat_instruction_graph(&self, _plan: &ExecutionPlan) -> bool {
        // For Gate C, all instructions are flat by construction
        // This check is more relevant for future gates with complex expressions
        true
    }
}

impl Default for ExecutionPlanValidator {
    fn default() -> Self {
        Self::new()
    }
}

/// Validation configuration
#[derive(Debug, Clone)]
pub struct ValidationConfig {
    /// Check for determinism properties
    pub check_determinism: bool,
    /// Strict register checking (all uses must have definitions)
    pub strict_register_checking: bool,
    /// Single assignment constraint (SSA-like)
    pub single_assignment: bool,
    /// Warn about unreachable blocks
    pub warn_unreachable_blocks: bool,
    /// Warn about unused registers
    pub warn_unused_registers: bool,
}

impl Default for ValidationConfig {
    fn default() -> Self {
        Self {
            check_determinism: true,
            strict_register_checking: true,
            single_assignment: true,
            warn_unreachable_blocks: true,
            warn_unused_registers: true,
        }
    }
}

/// Validation report
#[derive(Debug, Clone)]
pub struct ValidationReport {
    /// Structure validation passed
    pub structure_valid: bool,
    /// Control flow validation passed
    pub control_flow_valid: bool,
    /// Register usage validation passed
    pub register_usage_valid: bool,
    /// Dataflow validation passed
    pub dataflow_valid: bool,
    /// Determinism validation passed
    pub determinism_valid: bool,
    /// Architectural compliance validation passed
    pub architectural_compliance_valid: bool,
    /// Validation warnings
    pub warnings: Vec<ValidationWarning>,
}

impl ValidationReport {
    /// Create new validation report
    pub fn new() -> Self {
        Self {
            structure_valid: false,
            control_flow_valid: false,
            register_usage_valid: false,
            dataflow_valid: false,
            determinism_valid: false,
            architectural_compliance_valid: false,
            warnings: Vec::new(),
        }
    }

    /// Check if all validations passed
    pub fn is_valid(&self) -> bool {
        self.structure_valid
            && self.control_flow_valid
            && self.register_usage_valid
            && self.dataflow_valid
            && self.determinism_valid
            && self.architectural_compliance_valid
    }

    /// Get validation summary
    pub fn summary(&self) -> String {
        if self.is_valid() {
            format!(
                "ExecutionPlan validation PASSED ({} warnings)",
                self.warnings.len()
            )
        } else {
            "ExecutionPlan validation FAILED".to_string()
        }
    }
}

impl Default for ValidationReport {
    fn default() -> Self {
        Self::new()
    }
}

/// Validation warnings
#[derive(Debug, Clone)]
pub enum ValidationWarning {
    /// Unreachable blocks detected
    UnreachableBlocks { block_ids: Vec<BlockId> },
    /// Unused registers detected
    UnusedRegisters { registers: Vec<RegisterId> },
    /// Performance warning
    PerformanceWarning { message: String },
}

/// Validation errors
#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    #[error("Invalid entry block: {block_id}")]
    InvalidEntryBlock { block_id: BlockId },

    #[error("Empty block: {block_id}")]
    EmptyBlock { block_id: BlockId },

    #[error("Duplicate block ID: {block_id}")]
    DuplicateBlockId { block_id: BlockId },

    #[error("Terminator instruction in block {block_id} at index {instruction_index}")]
    TerminatorInInstructions {
        block_id: BlockId,
        instruction_index: usize,
    },

    #[error("Invalid instruction: {reason}")]
    InvalidInstruction { reason: String },

    #[error("Invalid branch target: block {source_block} -> {target_block}")]
    InvalidBranchTarget {
        source_block: BlockId,
        target_block: BlockId,
    },

    #[error("Invalid jump target: block {source_block} -> {target_block}")]
    InvalidJumpTarget {
        source_block: BlockId,
        target_block: BlockId,
    },

    #[error("Undefined register {register} in block {block_id}")]
    UndefinedRegister {
        register: RegisterId,
        block_id: BlockId,
    },

    #[error("Register {register} redefined in block {block_id}")]
    RegisterRedefinition {
        register: RegisterId,
        block_id: BlockId,
    },

    #[error("Circular dependency detected")]
    CircularDependency,

    #[error("Missing determinism fingerprint")]
    MissingDeterminismFingerprint,

    #[error("Non-deterministic instruction in block {block_id}: {instruction}")]
    NonDeterministicInstruction {
        block_id: BlockId,
        instruction: String,
    },

    #[error("Loops not allowed in Gate C")]
    LoopsNotAllowed,

    #[error("Nested expressions not allowed in flat instruction graph")]
    NestedExpressionsNotAllowed,

    #[error("Dataflow validation failed")]
    DataflowError {
        #[from]
        source: DataflowError,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::execution_plan::dataflow::DataflowGraph;
    use crate::execution_plan::{
        BlockTerminator, ExecutionMetadata, ExecutionPlan, IRBlock, IRInstruction, ParallelSafety,
    };
    use crate::normalizer::RegisterAllocation;

    use std::collections::HashMap;

    fn create_test_execution_plan() -> ExecutionPlan {
        let block = IRBlock::with_safety(
            0,
            vec![
                IRInstruction::LoadContext {
                    context_id: "users".to_string(),
                    target_register: 0,
                },
                IRInstruction::LoadField {
                    source_register: 0,
                    field_name: "name".to_string(),
                    target_register: 1,
                },
            ],
            BlockTerminator::Return { register: 1 },
            ParallelSafety::Safe, // Pure data transformation
        );

        ExecutionPlan::new(
            vec![block],
            0,
            RegisterAllocation {
                allocated_registers: vec![],
                register_dependencies: HashMap::new(),
                next_register: 2,
            },
            DataflowGraph::new(),
            ExecutionMetadata::new("test".to_string(), 1, 2, 2),
        )
    }

    #[test]
    fn test_validator_creation() {
        let validator = ExecutionPlanValidator::new();
        assert!(validator.config.check_determinism);
        assert!(validator.config.strict_register_checking);
        assert!(validator.config.single_assignment);
    }

    #[test]
    fn test_valid_execution_plan() {
        let validator = ExecutionPlanValidator::new();
        let mut plan = create_test_execution_plan();
        plan.metadata.determinism_fingerprint = "test_fingerprint".to_string();

        let result = validator.validate(&plan);
        assert!(result.is_ok());

        let report = result.unwrap();
        assert!(report.is_valid());
        assert_eq!(
            report.summary(),
            "ExecutionPlan validation PASSED (0 warnings)"
        );
    }

    #[test]
    fn test_invalid_entry_block() {
        let validator = ExecutionPlanValidator::new();
        let mut plan = create_test_execution_plan();
        plan.entry_block = 999; // Non-existent block

        let result = validator.validate(&plan);
        assert!(result.is_err());

        match result.unwrap_err() {
            ValidationError::InvalidEntryBlock { block_id } => {
                assert_eq!(block_id, 999);
            }
            _ => panic!("Expected InvalidEntryBlock error"),
        }
    }

    #[test]
    fn test_empty_block_validation() {
        let validator = ExecutionPlanValidator::new();

        // Empty block with jump terminator should fail
        let empty_block = IRBlock::with_safety(
            0,
            vec![],
            BlockTerminator::Jump { target_block: 1 },
            ParallelSafety::Unsafe, // Empty block is unsafe
        );

        let result = validator.validate_block_structure(&empty_block, &mut ValidationReport::new());
        assert!(result.is_err());
    }

    #[test]
    fn test_terminator_in_instructions() {
        let validator = ExecutionPlanValidator::new();

        // Block with terminator in instruction list
        let invalid_block = IRBlock::with_safety(
            0,
            vec![
                IRInstruction::LoadContext {
                    context_id: "users".to_string(),
                    target_register: 0,
                },
                IRInstruction::Return { source_register: 0 }, // Terminator in instructions
            ],
            BlockTerminator::Return { register: 0 },
            ParallelSafety::Unsafe, // Invalid structure is unsafe
        );

        let result =
            validator.validate_block_structure(&invalid_block, &mut ValidationReport::new());
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_instruction_validation() {
        let validator = ExecutionPlanValidator::new();

        // LoadContext with empty context_id
        let invalid_instruction = IRInstruction::LoadContext {
            context_id: "".to_string(),
            target_register: 0,
        };

        let result = validator.validate_instruction(&invalid_instruction);
        assert!(result.is_err());

        // LoadField with empty field_name
        let invalid_field = IRInstruction::LoadField {
            source_register: 0,
            field_name: "".to_string(),
            target_register: 1,
        };

        let result = validator.validate_instruction(&invalid_field);
        assert!(result.is_err());
    }

    #[test]
    fn test_missing_determinism_fingerprint() {
        let validator = ExecutionPlanValidator::new();
        let plan = create_test_execution_plan();
        // Plan has empty determinism_fingerprint by default

        let result = validator.validate(&plan);
        assert!(result.is_err());

        match result.unwrap_err() {
            ValidationError::MissingDeterminismFingerprint => {}
            _ => panic!("Expected MissingDeterminismFingerprint error"),
        }
    }

    #[test]
    fn test_validation_config() {
        let config = ValidationConfig {
            check_determinism: false,
            strict_register_checking: false,
            single_assignment: false,
            warn_unreachable_blocks: false,
            warn_unused_registers: false,
        };

        let validator = ExecutionPlanValidator::with_config(config);
        let plan = create_test_execution_plan();

        // Should pass validation with relaxed config
        let result = validator.validate(&plan);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validation_report() {
        let mut report = ValidationReport::new();
        assert!(!report.is_valid());

        // Mark all validations as passed
        report.structure_valid = true;
        report.control_flow_valid = true;
        report.register_usage_valid = true;
        report.dataflow_valid = true;
        report.determinism_valid = true;
        report.architectural_compliance_valid = true;

        assert!(report.is_valid());
        assert_eq!(
            report.summary(),
            "ExecutionPlan validation PASSED (0 warnings)"
        );

        // Add a warning
        report.warnings.push(ValidationWarning::UnusedRegisters {
            registers: vec![42],
        });
        assert_eq!(
            report.summary(),
            "ExecutionPlan validation PASSED (1 warnings)"
        );
    }
}
