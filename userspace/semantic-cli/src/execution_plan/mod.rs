//! ExecutionPlan IR Implementation (C6)
//! 
//! **Created By:** Kenan AY
//! **Date:** 16 Ocak 2026
//! **Architectural Reference:** C2 - ExecutionPlan IR Design Specification
//! 
//! Register-based intermediate representation that serves as the single source of truth
//! for deterministic execution. Transforms normalized BCIB into optimizable, replayable
//! execution graph.
//! 
//! **Key Principle:** Flat instruction graph with explicit control flow and register-based data flow.

use crate::normalizer::{NormalizedBCIB, NormalizedInstruction, InstructionGroup, RegisterAllocation};
use crate::bcib::{BCIBInstruction, ContextInstruction, QueryInstruction, Value, ComparisonOp, LogicalOperator, FilterExpression};
use std::collections::HashMap;
use std::hash::{DefaultHasher, Hash, Hasher};

pub mod builder;
pub mod dataflow;
pub mod validator;

use builder::IRBuilder;
use dataflow::DataflowGraph;

/// Register identifier for IR execution
pub type RegisterId = u16;

/// Block identifier for control flow
pub type BlockId = u16;

/// Instruction identifier for dataflow analysis
pub type InstructionId = u32;

/// Parallel safety classification for IR blocks
/// 
/// **Architectural Reference:** D2 Parallelism Architecture - Section "Parallel Safety Classification"
/// 
/// Indicates whether an IR block can be safely parallelized:
/// - `Safe`: Pure data transformations with no side effects (map, filter, projection)
/// - `Unsafe`: Operations with side effects, IO, or order-sensitive operations
/// - `ReductionOnly`: Reducible but not mappable (e.g., fold with state)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ParallelSafety {
    /// Pure operations that can be safely parallelized
    Safe,
    /// Operations with side effects, IO, or order-sensitive operations
    Unsafe,
    /// Operations that can be reduced but not mapped
    ReductionOnly,
}

/// Core IR instruction set - flat instruction graph with explicit control flow
#[derive(Debug, Clone, PartialEq)]
pub enum IRInstruction {
    /// Load context data into register
    LoadContext {
        context_id: String,
        target_register: RegisterId,
    },
    
    /// Load field from context register
    LoadField {
        source_register: RegisterId,  // Context register
        field_name: String,
        target_register: RegisterId,
    },
    
    /// Load literal value into register
    LoadLiteral {
        value: Value,
        target_register: RegisterId,
    },
    
    /// Compare two registers and store boolean result
    Compare {
        left_register: RegisterId,
        operator: ComparisonOp,
        right_register: RegisterId,
        target_register: RegisterId,
    },
    
    /// Apply logical operation to registers
    LogicalOp {
        operation: LogicalOperator,
        operand_registers: Vec<RegisterId>,
        target_register: RegisterId,
    },
    
    /// Conditional branch based on register value
    Branch {
        condition_register: RegisterId,
        true_block: BlockId,
        false_block: BlockId,
    },
    
    /// Apply filter to context using filter expression (C9: Per-item evaluation)
    ApplyFilter {
        context_register: RegisterId,
        filter_expression: FilterExpression,  // C9: Keep for per-item evaluation
        target_register: RegisterId,
    },
    
    /// Return value from register
    Return {
        source_register: RegisterId,
    },
}

impl IRInstruction {
    /// Get input registers for this instruction
    pub fn input_registers(&self) -> Vec<RegisterId> {
        match self {
            Self::LoadContext { .. } => vec![],
            Self::LoadField { source_register, .. } => vec![*source_register],
            Self::LoadLiteral { .. } => vec![],
            Self::Compare { left_register, right_register, .. } => {
                vec![*left_register, *right_register]
            },
            Self::LogicalOp { operand_registers, .. } => operand_registers.clone(),
            Self::Branch { condition_register, .. } => vec![*condition_register],
            Self::ApplyFilter { context_register, .. } => {
                vec![*context_register]  // C9: Only context register as input
            },
            Self::Return { source_register } => vec![*source_register],
        }
    }
    
    /// Get output registers for this instruction
    pub fn output_registers(&self) -> Vec<RegisterId> {
        match self {
            Self::LoadContext { target_register, .. } => vec![*target_register],
            Self::LoadField { target_register, .. } => vec![*target_register],
            Self::LoadLiteral { target_register, .. } => vec![*target_register],
            Self::Compare { target_register, .. } => vec![*target_register],
            Self::LogicalOp { target_register, .. } => vec![*target_register],
            Self::Branch { .. } => vec![], // Branches don't produce values
            Self::ApplyFilter { target_register, .. } => vec![*target_register],
            Self::Return { .. } => vec![], // Return doesn't produce values
        }
    }
    
    /// Check if this instruction is a terminator (ends a block)
    pub fn is_terminator(&self) -> bool {
        matches!(self, Self::Branch { .. } | Self::Return { .. })
    }
}

/// Block terminator - how execution exits a block
#[derive(Debug, Clone, PartialEq, Hash)]
pub enum BlockTerminator {
    /// Return with value from register
    Return { register: RegisterId },
    /// Conditional branch to two blocks
    Branch { 
        condition: RegisterId, 
        true_block: BlockId, 
        false_block: BlockId 
    },
    /// Unconditional jump to block
    Jump { target_block: BlockId },
}

/// IR execution block - sequence of instructions with terminator
#[derive(Debug, Clone, PartialEq)]
pub struct IRBlock {
    pub id: BlockId,
    pub instructions: Vec<IRInstruction>,
    pub terminator: BlockTerminator,
    pub parallel_safety: ParallelSafety,
}

impl IRBlock {
    /// Create new IR block
    pub fn new(id: BlockId, instructions: Vec<IRInstruction>, terminator: BlockTerminator) -> Self {
        Self { 
            id, 
            instructions, 
            terminator,
            parallel_safety: ParallelSafety::Unsafe, // Default to Unsafe for safety
        }
    }
    
    /// Create new IR block with explicit parallel safety annotation
    pub fn with_safety(
        id: BlockId, 
        instructions: Vec<IRInstruction>, 
        terminator: BlockTerminator,
        parallel_safety: ParallelSafety,
    ) -> Self {
        Self { 
            id, 
            instructions, 
            terminator,
            parallel_safety,
        }
    }
    
    /// Get all registers used by this block
    pub fn used_registers(&self) -> Vec<RegisterId> {
        let mut registers = Vec::new();
        
        for instruction in &self.instructions {
            registers.extend(instruction.input_registers());
            registers.extend(instruction.output_registers());
        }
        
        match &self.terminator {
            BlockTerminator::Return { register } => registers.push(*register),
            BlockTerminator::Branch { condition, .. } => registers.push(*condition),
            BlockTerminator::Jump { .. } => {},
        }
        
        registers.sort_unstable();
        registers.dedup();
        registers
    }
}

/// Complete execution plan - single source of truth for deterministic execution
#[derive(Debug, Clone, PartialEq)]
pub struct ExecutionPlan {
    /// All execution blocks
    pub blocks: Vec<IRBlock>,
    /// Entry point block ID
    pub entry_block: BlockId,
    /// Register allocation information
    pub register_allocation: RegisterAllocation,
    /// Dataflow analysis graph
    pub dataflow_graph: DataflowGraph,
    /// Execution metadata
    pub metadata: ExecutionMetadata,
}

impl ExecutionPlan {
    /// Create new execution plan
    pub fn new(
        blocks: Vec<IRBlock>,
        entry_block: BlockId,
        register_allocation: RegisterAllocation,
        dataflow_graph: DataflowGraph,
        metadata: ExecutionMetadata,
    ) -> Self {
        Self {
            blocks,
            entry_block,
            register_allocation,
            dataflow_graph,
            metadata,
        }
    }
    
    /// Get block by ID
    pub fn get_block(&self, block_id: BlockId) -> Option<&IRBlock> {
        self.blocks.iter().find(|block| block.id == block_id)
    }
    
    /// Get entry block
    pub fn entry_block(&self) -> Option<&IRBlock> {
        self.get_block(self.entry_block)
    }
    
    /// Validate execution plan structure
    pub fn validate(&self) -> Result<(), ExecutionPlanError> {
        // Check entry block exists
        if self.get_block(self.entry_block).is_none() {
            return Err(ExecutionPlanError::InvalidEntryBlock { 
                block_id: self.entry_block 
            });
        }
        
        // Validate each block
        for block in &self.blocks {
            self.validate_block(block)?;
        }
        
        // Validate control flow
        self.validate_control_flow()?;
        
        Ok(())
    }
    
    /// Validate individual block
    fn validate_block(&self, block: &IRBlock) -> Result<(), ExecutionPlanError> {
        // Check block has instructions or terminator
        if block.instructions.is_empty() && matches!(block.terminator, BlockTerminator::Jump { .. }) {
            return Err(ExecutionPlanError::EmptyBlock { block_id: block.id });
        }
        
        // Check no terminators in instruction list
        for instruction in &block.instructions {
            if instruction.is_terminator() {
                return Err(ExecutionPlanError::TerminatorInInstructions { 
                    block_id: block.id 
                });
            }
        }
        
        Ok(())
    }
    
    /// Validate control flow graph
    fn validate_control_flow(&self) -> Result<(), ExecutionPlanError> {
        for block in &self.blocks {
            match &block.terminator {
                BlockTerminator::Branch { true_block, false_block, .. } => {
                    if self.get_block(*true_block).is_none() {
                        return Err(ExecutionPlanError::InvalidBranchTarget { 
                            source_block: block.id,
                            target_block: *true_block,
                        });
                    }
                    if self.get_block(*false_block).is_none() {
                        return Err(ExecutionPlanError::InvalidBranchTarget { 
                            source_block: block.id,
                            target_block: *false_block,
                        });
                    }
                },
                BlockTerminator::Jump { target_block } => {
                    if self.get_block(*target_block).is_none() {
                        return Err(ExecutionPlanError::InvalidJumpTarget { 
                            source_block: block.id,
                            target_block: *target_block,
                        });
                    }
                },
                BlockTerminator::Return { .. } => {
                    // Return terminators are always valid
                },
            }
        }
        
        Ok(())
    }
    
    /// Compute determinism fingerprint for replay validation
    /// 
    /// **Architectural Reference:** C2 Section "Determinism Guarantee"
    pub fn compute_determinism_fingerprint(&self) -> String {
        let mut hasher = DefaultHasher::new();
        
        // Hash instruction sequence using Debug format (C9: FilterExpression doesn't implement Hash)
        for block in &self.blocks {
            block.id.hash(&mut hasher);
            for instruction in &block.instructions {
                // Use Debug format for hashing since FilterExpression doesn't implement Hash
                format!("{:?}", instruction).hash(&mut hasher);
            }
            block.terminator.hash(&mut hasher);
        }
        
        // Hash register allocation
        self.register_allocation.allocated_registers.hash(&mut hasher);
        self.entry_block.hash(&mut hasher);
        
        format!("{:x}", hasher.finish())
    }
    
    /// Get all registers used in execution plan
    pub fn all_registers(&self) -> Vec<RegisterId> {
        let mut registers = Vec::new();
        
        for block in &self.blocks {
            registers.extend(block.used_registers());
        }
        
        registers.sort_unstable();
        registers.dedup();
        registers
    }
}

/// Execution metadata for determinism and debugging
#[derive(Debug, Clone, PartialEq)]
pub struct ExecutionMetadata {
    /// Original BCIB sequence ID
    pub source_sequence_id: String,
    /// IR generation timestamp
    pub generated_at: String,
    /// Determinism fingerprint
    pub determinism_fingerprint: String,
    /// Number of blocks
    pub block_count: usize,
    /// Number of instructions
    pub instruction_count: usize,
    /// Number of registers
    pub register_count: usize,
}

impl ExecutionMetadata {
    /// Create new execution metadata
    pub fn new(
        source_sequence_id: String,
        block_count: usize,
        instruction_count: usize,
        register_count: usize,
    ) -> Self {
        Self {
            source_sequence_id,
            generated_at: chrono::Utc::now().to_rfc3339(),
            determinism_fingerprint: String::new(), // Will be computed later
            block_count,
            instruction_count,
            register_count,
        }
    }
}

/// ExecutionPlan errors
#[derive(Debug, thiserror::Error)]
pub enum ExecutionPlanError {
    #[error("Invalid entry block: {block_id}")]
    InvalidEntryBlock { block_id: BlockId },
    
    #[error("Empty block: {block_id}")]
    EmptyBlock { block_id: BlockId },
    
    #[error("Terminator instruction in block {block_id} instruction list")]
    TerminatorInInstructions { block_id: BlockId },
    
    #[error("Invalid branch target: block {source_block} -> {target_block}")]
    InvalidBranchTarget { source_block: BlockId, target_block: BlockId },
    
    #[error("Invalid jump target: block {source_block} -> {target_block}")]
    InvalidJumpTarget { source_block: BlockId, target_block: BlockId },
    
    #[error("IR build failed: {reason}")]
    BuildFailed { reason: String },
    
    #[error("Dataflow analysis failed: {reason}")]
    DataflowFailed { reason: String },
}

/// Main ExecutionPlan builder - converts NormalizedBCIB to ExecutionPlan
pub struct ExecutionPlanBuilder {
    ir_builder: IRBuilder,
}

impl ExecutionPlanBuilder {
    /// Create new execution plan builder
    pub fn new() -> Self {
        Self {
            ir_builder: IRBuilder::new(),
        }
    }
    
    /// Build execution plan from normalized BCIB
    /// 
    /// **Architectural Reference:** C2 Section "IR Builder Architecture"
    pub fn build(&mut self, normalized_bcib: NormalizedBCIB) -> Result<ExecutionPlan, ExecutionPlanError> {
        self.ir_builder.build_execution_plan(normalized_bcib)
            .map_err(|e| ExecutionPlanError::BuildFailed { 
                reason: e.to_string() 
            })
    }
}

impl Default for ExecutionPlanBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// **PURE FUNCTION INTERFACE**
/// 
/// **Architectural Reference:** C2 Section "IR Builder Architecture"
/// 
/// Stateless IR generation function for functional programming style.
pub fn build_execution_plan(normalized_bcib: NormalizedBCIB) -> Result<ExecutionPlan, ExecutionPlanError> {
    let mut builder = ExecutionPlanBuilder::new();
    builder.build(normalized_bcib)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    use crate::types::SourceLocation;
    
    fn test_location() -> SourceLocation {
        SourceLocation::new(1, 1, 0)
    }
    
    #[test]
    fn test_ir_instruction_registers() {
        let load_context = IRInstruction::LoadContext {
            context_id: "users".to_string(),
            target_register: 0,
        };
        assert_eq!(load_context.input_registers(), Vec::<RegisterId>::new());
        assert_eq!(load_context.output_registers(), vec![0]);
        assert!(!load_context.is_terminator());
        
        let compare = IRInstruction::Compare {
            left_register: 1,
            operator: ComparisonOp::Equal,
            right_register: 2,
            target_register: 3,
        };
        assert_eq!(compare.input_registers(), vec![1, 2]);
        assert_eq!(compare.output_registers(), vec![3]);
        assert!(!compare.is_terminator());
        
        let branch = IRInstruction::Branch {
            condition_register: 3,
            true_block: 1,
            false_block: 2,
        };
        assert_eq!(branch.input_registers(), vec![3]);
        assert_eq!(branch.output_registers(), Vec::<RegisterId>::new());
        assert!(branch.is_terminator());
        
        let return_inst = IRInstruction::Return {
            source_register: 0,
        };
        assert_eq!(return_inst.input_registers(), vec![0]);
        assert_eq!(return_inst.output_registers(), Vec::<RegisterId>::new());
        assert!(return_inst.is_terminator());
    }
    
    #[test]
    fn test_ir_block_creation() {
        let instructions = vec![
            IRInstruction::LoadContext {
                context_id: "users".to_string(),
                target_register: 0,
            },
            IRInstruction::LoadField {
                source_register: 0,
                field_name: "name".to_string(),
                target_register: 1,
            },
        ];
        
        let terminator = BlockTerminator::Return { register: 1 };
        let block = IRBlock::with_safety(0, instructions, terminator, ParallelSafety::Safe);
        
        assert_eq!(block.id, 0);
        assert_eq!(block.instructions.len(), 2);
        assert_eq!(block.parallel_safety, ParallelSafety::Safe);
        
        let used_registers = block.used_registers();
        assert!(used_registers.contains(&0));
        assert!(used_registers.contains(&1));
    }
    
    #[test]
    fn test_execution_plan_validation() {
        // Valid execution plan
        let block = IRBlock::with_safety(
            0,
            vec![IRInstruction::LoadContext {
                context_id: "users".to_string(),
                target_register: 0,
            }],
            BlockTerminator::Return { register: 0 },
            ParallelSafety::Safe, // Pure context load
        );
        
        let plan = ExecutionPlan::new(
            vec![block],
            0,
            RegisterAllocation {
                allocated_registers: vec![],
                register_dependencies: HashMap::new(),
                next_register: 1,
            },
            DataflowGraph::new(),
            ExecutionMetadata::new("test".to_string(), 1, 1, 1),
        );
        
        assert!(plan.validate().is_ok());
        assert!(plan.entry_block().is_some());
        assert_eq!(plan.get_block(0).unwrap().id, 0);
    }
    
    #[test]
    fn test_execution_plan_invalid_entry_block() {
        let plan = ExecutionPlan::new(
            vec![],
            0,
            RegisterAllocation {
                allocated_registers: vec![],
                register_dependencies: HashMap::new(),
                next_register: 0,
            },
            DataflowGraph::new(),
            ExecutionMetadata::new("test".to_string(), 0, 0, 0),
        );
        
        assert!(plan.validate().is_err());
    }
    
    #[test]
    fn test_determinism_fingerprint() {
        let block = IRBlock::with_safety(
            0,
            vec![IRInstruction::LoadContext {
                context_id: "users".to_string(),
                target_register: 0,
            }],
            BlockTerminator::Return { register: 0 },
            ParallelSafety::Safe, // Pure context load
        );
        
        let plan1 = ExecutionPlan::new(
            vec![block.clone()],
            0,
            RegisterAllocation {
                allocated_registers: vec![],
                register_dependencies: HashMap::new(),
                next_register: 1,
            },
            DataflowGraph::new(),
            ExecutionMetadata::new("test".to_string(), 1, 1, 1),
        );
        
        let plan2 = ExecutionPlan::new(
            vec![block],
            0,
            RegisterAllocation {
                allocated_registers: vec![],
                register_dependencies: HashMap::new(),
                next_register: 1,
            },
            DataflowGraph::new(),
            ExecutionMetadata::new("test".to_string(), 1, 1, 1),
        );
        
        // Same plans should have same fingerprint
        assert_eq!(
            plan1.compute_determinism_fingerprint(),
            plan2.compute_determinism_fingerprint()
        );
    }
    
    #[test]
    fn test_execution_plan_registers() {
        let block = IRBlock::with_safety(
            0,
            vec![
                IRInstruction::LoadContext {
                    context_id: "users".to_string(),
                    target_register: 0,
                },
                IRInstruction::LoadField {
                    source_register: 0,
                    field_name: "age".to_string(),
                    target_register: 1,
                },
                IRInstruction::Compare {
                    left_register: 1,
                    operator: ComparisonOp::GreaterThan,
                    right_register: 2,
                    target_register: 3,
                },
            ],
            BlockTerminator::Return { register: 3 },
            ParallelSafety::Safe, // Pure data operations
        );
        
        let plan = ExecutionPlan::new(
            vec![block],
            0,
            RegisterAllocation {
                allocated_registers: vec![],
                register_dependencies: HashMap::new(),
                next_register: 4,
            },
            DataflowGraph::new(),
            ExecutionMetadata::new("test".to_string(), 1, 3, 4),
        );
        
        let all_registers = plan.all_registers();
        assert!(all_registers.contains(&0));
        assert!(all_registers.contains(&1));
        assert!(all_registers.contains(&2));
        assert!(all_registers.contains(&3));
    }
    
    #[test]
    fn test_parallel_safety_enum() {
        // Test that ParallelSafety enum has all required traits
        let safe = ParallelSafety::Safe;
        let unsafe_variant = ParallelSafety::Unsafe;
        let reduction_only = ParallelSafety::ReductionOnly;
        
        // Test Debug trait
        assert_eq!(format!("{:?}", safe), "Safe");
        assert_eq!(format!("{:?}", unsafe_variant), "Unsafe");
        assert_eq!(format!("{:?}", reduction_only), "ReductionOnly");
        
        // Test Clone trait
        let safe_clone = safe.clone();
        assert_eq!(safe, safe_clone);
        
        // Test PartialEq trait
        assert_eq!(safe, ParallelSafety::Safe);
        assert_ne!(safe, unsafe_variant);
        assert_ne!(safe, reduction_only);
        
        // Test Hash trait (by using in a HashSet)
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(safe);
        set.insert(unsafe_variant);
        set.insert(reduction_only);
        assert_eq!(set.len(), 3);
    }
    
    #[test]
    fn test_ir_block_parallel_safety_default() {
        // Test that IRBlock::new() defaults to Unsafe for safety
        let instructions = vec![
            IRInstruction::LoadContext {
                context_id: "users".to_string(),
                target_register: 0,
            },
        ];
        
        let terminator = BlockTerminator::Return { register: 0 };
        let block = IRBlock::new(0, instructions, terminator);
        
        assert_eq!(block.parallel_safety, ParallelSafety::Unsafe);
    }
    
    #[test]
    fn test_ir_block_with_safety() {
        // Test that IRBlock::with_safety() allows explicit safety annotation
        let instructions = vec![
            IRInstruction::LoadContext {
                context_id: "users".to_string(),
                target_register: 0,
            },
        ];
        
        let terminator = BlockTerminator::Return { register: 0 };
        
        // Test Safe variant
        let safe_block = IRBlock::with_safety(
            0, 
            instructions.clone(), 
            terminator.clone(),
            ParallelSafety::Safe
        );
        assert_eq!(safe_block.parallel_safety, ParallelSafety::Safe);
        
        // Test Unsafe variant
        let unsafe_block = IRBlock::with_safety(
            1, 
            instructions.clone(), 
            terminator.clone(),
            ParallelSafety::Unsafe
        );
        assert_eq!(unsafe_block.parallel_safety, ParallelSafety::Unsafe);
        
        // Test ReductionOnly variant
        let reduction_block = IRBlock::with_safety(
            2, 
            instructions, 
            terminator,
            ParallelSafety::ReductionOnly
        );
        assert_eq!(reduction_block.parallel_safety, ParallelSafety::ReductionOnly);
    }
}