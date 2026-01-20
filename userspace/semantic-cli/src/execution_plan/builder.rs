//! IR Builder Implementation
//! 
//! **Created By:** Kenan AY
//! **Date:** 16 Ocak 2026
//! **Architectural Reference:** C2 Section "IR Builder Architecture"
//! 
//! Converts normalized BCIB instructions into ExecutionPlan IR with flat instruction
//! graph and explicit control flow.

use super::{
    IRInstruction, IRBlock, BlockTerminator, ExecutionPlan, ExecutionMetadata,
    RegisterId, BlockId, ExecutionPlanError, ParallelSafety,
};
use crate::normalizer::{NormalizedBCIB, NormalizedInstruction, InstructionGroup};
use crate::bcib::{BCIBInstruction, ContextInstruction, QueryInstruction, OperandRef, Value, FilterExpression, ComparisonOp};
use crate::execution_plan::dataflow::DataflowGraph;
use crate::parallelism::{ParallelSafetyAnalyzer, DefaultSafetyAnalyzer};
use std::collections::HashMap;

/// IR Builder - converts normalized BCIB to ExecutionPlan
pub struct IRBuilder {
    next_block_id: BlockId,
    current_block_instructions: Vec<IRInstruction>,
    blocks: Vec<IRBlock>,
    register_mapping: HashMap<u16, RegisterId>,
    next_temp_register: u16, // C9: For auto-generated LoadField/LoadLiteral
    safety_analyzer: DefaultSafetyAnalyzer, // D2: Parallel safety analyzer
}

impl IRBuilder {
    /// Create new IR builder
    pub fn new() -> Self {
        Self {
            next_block_id: 0,
            current_block_instructions: Vec::new(),
            blocks: Vec::new(),
            register_mapping: HashMap::new(),
            next_temp_register: 100, // C9: Start temp registers at 100 to avoid conflicts
            safety_analyzer: DefaultSafetyAnalyzer::new(), // D2: Initialize safety analyzer
        }
    }
    
    /// Build execution plan from normalized BCIB
    /// 
    /// **Architectural Reference:** C2 Section "IR Generation Pipeline"
    pub fn build_execution_plan(&mut self, normalized_bcib: NormalizedBCIB) -> Result<ExecutionPlan, IRBuildError> {
        // Reset builder state
        self.reset();
        
        // 1. Create entry block
        let entry_block_id = self.allocate_block_id();
        
        // 2. Convert normalized instructions to IR
        let ir_instructions = self.convert_instructions(&normalized_bcib.instructions)?;
        
        // 3. Build single block for Gate C (no complex control flow yet)
        let output_register = normalized_bcib.output_register.unwrap_or(0);
        let entry_block = self.build_single_block(entry_block_id, ir_instructions, output_register)?;
        self.blocks.push(entry_block);
        
        // 4. Analyze dataflow
        let dataflow_graph = self.analyze_dataflow(&self.blocks)?;
        
        // 5. Create execution metadata
        let metadata = self.create_execution_metadata(&normalized_bcib);
        
        // 6. Create execution plan
        let mut plan = ExecutionPlan::new(
            self.blocks.clone(),
            entry_block_id,
            normalized_bcib.register_allocation,
            dataflow_graph,
            metadata,
        );
        
        // 7. Compute determinism fingerprint
        let fingerprint = plan.compute_determinism_fingerprint();
        plan.metadata.determinism_fingerprint = fingerprint;
        
        // 8. Validate execution plan
        plan.validate().map_err(|e| IRBuildError::ValidationFailed { 
            reason: e.to_string() 
        })?;
        
        Ok(plan)
    }
    
    /// Reset builder state for new build
    fn reset(&mut self) {
        self.next_block_id = 0;
        self.current_block_instructions.clear();
        self.blocks.clear();
        self.register_mapping.clear();
        self.next_temp_register = 100; // C9: Reset temp register counter
    }
    
    /// Allocate new block ID
    fn allocate_block_id(&mut self) -> BlockId {
        let id = self.next_block_id;
        self.next_block_id += 1;
        id
    }
    
    /// Convert normalized instructions to IR instructions
    /// 
    /// **Architectural Reference:** C2 Section "Instruction Conversion Rules"
    fn convert_instructions(&mut self, instructions: &[NormalizedInstruction]) -> Result<Vec<IRInstruction>, IRBuildError> {
        let mut ir_instructions = Vec::new();
        
        for normalized_inst in instructions {
            // C9: convert_instruction may emit multiple IR instructions
            let mut inst_list = self.convert_instruction(normalized_inst)?;
            ir_instructions.append(&mut inst_list);
        }
        
        Ok(ir_instructions)
    }
    
    /// Convert single normalized instruction to IR instruction(s)
    /// 
    /// **C9 Change:** Returns Vec<IRInstruction> to support operand lowering
    fn convert_instruction(&mut self, normalized_inst: &NormalizedInstruction) -> Result<Vec<IRInstruction>, IRBuildError> {
        match &normalized_inst.instruction {
            BCIBInstruction::Context(context_inst) => {
                let inst = self.convert_context_instruction(context_inst, normalized_inst)?;
                Ok(vec![inst])
            },
            BCIBInstruction::Query(query_inst) => {
                self.convert_query_instruction(query_inst, normalized_inst)
            },
            BCIBInstruction::System(_) => {
                // System instructions are handled by system executor, not IR
                Err(IRBuildError::UnsupportedInstruction { 
                    instruction: "System instructions not supported in IR".to_string() 
                })
            },
            BCIBInstruction::Debug(_) => {
                // Debug instructions are handled by debug executor, not IR
                Err(IRBuildError::UnsupportedInstruction { 
                    instruction: "Debug instructions not supported in IR".to_string() 
                })
            },
            BCIBInstruction::Loop(_) => {
                // TODO: Phase 1 - Loop instruction conversion not implemented yet
                Err(IRBuildError::UnsupportedInstruction { 
                    instruction: "Loop instructions not yet implemented in Phase 1".to_string() 
                })
            },
            BCIBInstruction::ControlFlow(_) => {
                // TODO: Phase 2.3 - Control flow instruction conversion not implemented yet
                Err(IRBuildError::UnsupportedInstruction { 
                    instruction: "Control flow instructions not yet implemented in Phase 2.3".to_string() 
                })
            },
        }
    }
    
    /// Convert context instruction to IR
    fn convert_context_instruction(
        &mut self, 
        context_inst: &ContextInstruction,
        normalized_inst: &NormalizedInstruction,
    ) -> Result<IRInstruction, IRBuildError> {
        match context_inst {
            ContextInstruction::LoadContext { path, .. } => {
                let target_register = self.get_output_register(normalized_inst)?;
                Ok(IRInstruction::LoadContext {
                    context_id: path.clone(),
                    target_register,
                })
            },
            ContextInstruction::Return { .. } => {
                // Return instruction: if no input register, use register 0 (implicit context)
                let source_register = if normalized_inst.input_registers.is_empty() {
                    0 // Default to register 0 (the loaded context)
                } else {
                    self.get_input_register(normalized_inst, 0)?
                };
                Ok(IRInstruction::Return {
                    source_register,
                })
            },
        }
    }
    
    /// Convert query instruction to IR
    fn convert_query_instruction(
        &mut self, 
        query_inst: &QueryInstruction,
        normalized_inst: &NormalizedInstruction,
    ) -> Result<Vec<IRInstruction>, IRBuildError> {
        match query_inst {
            QueryInstruction::LoadField { field, target_register, .. } => {
                let source_register = self.get_input_register(normalized_inst, 0)?;
                Ok(vec![IRInstruction::LoadField {
                    source_register,
                    field_name: field.clone(),
                    target_register: *target_register,
                }])
            },
            QueryInstruction::LoadLiteral { value, target_register, .. } => {
                Ok(vec![IRInstruction::LoadLiteral {
                    value: value.clone(),
                    target_register: *target_register,
                }])
            },
            QueryInstruction::Compare { left, operator, right, target_register, .. } => {
                // C9: Handle Field/Literal operands by emitting LoadField/LoadLiteral
                let mut instructions = Vec::new();
                
                let left_register = self.convert_operand_to_register(left, &mut instructions)?;
                let right_register = self.convert_operand_to_register(right, &mut instructions)?;
                
                instructions.push(IRInstruction::Compare {
                    left_register,
                    operator: *operator,
                    right_register,
                    target_register: *target_register,
                });
                
                Ok(instructions)
            },
            QueryInstruction::LogicalOp { operator, operands, target_register, .. } => {
                let mut instructions = Vec::new();
                let mut operand_registers = Vec::new();
                
                for operand in operands {
                    let reg = self.convert_operand_to_register(operand, &mut instructions)?;
                    operand_registers.push(reg);
                }
                
                instructions.push(IRInstruction::LogicalOp {
                    operation: *operator,
                    operand_registers,
                    target_register: *target_register,
                });
                
                Ok(instructions)
            },
            QueryInstruction::ApplyFilter { expression, .. } => {
                // C9: Pass FilterExpression through to IR for per-item evaluation
                let context_register = 0; // Assume context is in register 0
                let target_register = self.get_output_register(normalized_inst)?;
                Ok(vec![IRInstruction::ApplyFilter {
                    context_register,
                    filter_expression: expression.clone(),
                    target_register,
                }])
            },
            QueryInstruction::ApplyFilterBool { filter_register, .. } => {
                // C9: For ApplyFilterBool, we need to create a FilterExpression from the register
                // For now, this is a placeholder - full implementation in Gate D
                let context_register = 0;
                let target_register = self.get_output_register(normalized_inst)?;
                
                // Create a placeholder filter expression
                // In Gate D, we'll properly handle register-based filters
                let placeholder_filter = FilterExpression::new(
                    "placeholder".to_string(),
                    ComparisonOp::Equal,
                    OperandRef::TempRegister(*filter_register),
                );
                
                Ok(vec![IRInstruction::ApplyFilter {
                    context_register,
                    filter_expression: placeholder_filter,
                    target_register,
                }])
            },
        }
    }
    
    /// Convert OperandRef to register ID, emitting LoadField/LoadLiteral if needed (C9)
    fn convert_operand_to_register(
        &mut self,
        operand: &OperandRef,
        instructions: &mut Vec<IRInstruction>,
    ) -> Result<RegisterId, IRBuildError> {
        match operand {
            OperandRef::TempRegister(reg_id) => Ok(*reg_id),
            OperandRef::Field(field_name) => {
                // Emit LoadField instruction
                let field_register = self.next_temp_register;
                self.next_temp_register += 1;
                
                instructions.push(IRInstruction::LoadField {
                    source_register: 0, // Assume context in register 0
                    field_name: field_name.clone(),
                    target_register: field_register,
                });
                
                Ok(field_register)
            },
            OperandRef::Literal(value) => {
                // Emit LoadLiteral instruction
                let literal_register = self.next_temp_register;
                self.next_temp_register += 1;
                
                instructions.push(IRInstruction::LoadLiteral {
                    value: value.clone(),
                    target_register: literal_register,
                });
                
                Ok(literal_register)
            },
        }
    }
    
    /// Get input register from normalized instruction
    fn get_input_register(&self, normalized_inst: &NormalizedInstruction, index: usize) -> Result<RegisterId, IRBuildError> {
        if index >= normalized_inst.input_registers.len() {
            return Err(IRBuildError::InvalidRegisterIndex { 
                index, 
                available: normalized_inst.input_registers.len() 
            });
        }
        
        match &normalized_inst.input_registers[index] {
            crate::normalizer::dependency_tracker::RegisterId::Data(id) => Ok(*id),
            crate::normalizer::dependency_tracker::RegisterId::Context(id) => Ok(*id),
            crate::normalizer::dependency_tracker::RegisterId::Filter(id) => Ok(*id),
        }
    }
    
    /// Get output register from normalized instruction
    fn get_output_register(&self, normalized_inst: &NormalizedInstruction) -> Result<RegisterId, IRBuildError> {
        if normalized_inst.output_registers.is_empty() {
            // **C8 WORKAROUND:** If no output register assigned, infer from instruction type
            // This handles cases where normalizer hasn't properly assigned registers
            match &normalized_inst.instruction {
                BCIBInstruction::Context(ContextInstruction::LoadContext { .. }) => {
                    // LoadContext always produces register 0 (context register)
                    Ok(0)
                },
                BCIBInstruction::Query(QueryInstruction::LoadLiteral { target_register, .. }) => {
                    Ok(*target_register)
                },
                BCIBInstruction::Query(QueryInstruction::LoadField { target_register, .. }) => {
                    Ok(*target_register)
                },
                BCIBInstruction::Query(QueryInstruction::Compare { target_register, .. }) => {
                    Ok(*target_register)
                },
                BCIBInstruction::Query(QueryInstruction::LogicalOp { target_register, .. }) => {
                    Ok(*target_register)
                },
                BCIBInstruction::Query(QueryInstruction::ApplyFilter { .. }) => {
                    // ApplyFilter modifies context in-place, returns context register
                    Ok(0)
                },
                _ => Err(IRBuildError::NoOutputRegister),
            }
        } else {
            match &normalized_inst.output_registers[0] {
                crate::normalizer::dependency_tracker::RegisterId::Data(id) => Ok(*id),
                crate::normalizer::dependency_tracker::RegisterId::Context(id) => Ok(*id),
                crate::normalizer::dependency_tracker::RegisterId::Filter(id) => Ok(*id),
            }
        }
    }
    
    /// Build single execution block (Gate C simplification)
    fn build_single_block(&self, block_id: BlockId, instructions: Vec<IRInstruction>, output_register: u16) -> Result<IRBlock, IRBuildError> {
        if instructions.is_empty() {
            return Err(IRBuildError::EmptyInstructionList);
        }
        
        // Filter out Return instructions (they become block terminators)
        let mut block_instructions = Vec::new();
        
        for instruction in instructions {
            if let IRInstruction::Return { .. } = instruction {
                // Skip - will be added as terminator
            } else {
                block_instructions.push(instruction);
            }
        }
        
        // **C8 FIX:** Always create Return terminator with output_register
        let terminator = BlockTerminator::Return { register: output_register };
        
        // **D2 PARALLELISM:** Create block with default safety, then analyze
        let mut block = IRBlock::new(block_id, block_instructions, terminator);
        
        // **D2 PARALLELISM:** Analyze parallel safety and update block annotation
        let safety = self.safety_analyzer.analyze_block(&block);
        block.parallel_safety = safety;
        
        Ok(block)
    }
    
    /// Analyze dataflow for execution plan
    fn analyze_dataflow(&self, blocks: &[IRBlock]) -> Result<DataflowGraph, IRBuildError> {
        let mut dataflow = DataflowGraph::new();
        
        // For Gate C, we'll do basic dataflow analysis
        // More sophisticated analysis will be added in later gates
        for block in blocks {
            for (inst_id, instruction) in block.instructions.iter().enumerate() {
                let instruction_id = inst_id as u32;
                
                // Add dataflow node
                dataflow.add_instruction(
                    instruction_id,
                    instruction.input_registers(),
                    instruction.output_registers(),
                    block.id,
                );
                
                // Add register definitions
                for output_reg in instruction.output_registers() {
                    dataflow.add_register_definition(output_reg, instruction_id);
                }
                
                // Add register uses
                for input_reg in instruction.input_registers() {
                    dataflow.add_register_use(input_reg, instruction_id);
                }
            }
        }
        
        Ok(dataflow)
    }
    
    /// Create execution metadata
    fn create_execution_metadata(&self, normalized_bcib: &NormalizedBCIB) -> ExecutionMetadata {
        let instruction_count = self.blocks.iter()
            .map(|block| block.instructions.len())
            .sum();
        
        ExecutionMetadata::new(
            normalized_bcib.metadata.determinism_fingerprint.clone(),
            self.blocks.len(),
            instruction_count,
            normalized_bcib.register_allocation.allocated_registers.len(),
        )
    }
}

impl Default for IRBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// IR build errors
#[derive(Debug, thiserror::Error)]
pub enum IRBuildError {
    #[error("Unsupported instruction: {instruction}")]
    UnsupportedInstruction { instruction: String },
    
    #[error("Invalid operand: {operand}")]
    InvalidOperand { operand: String },
    
    #[error("Invalid register index {index}, available: {available}")]
    InvalidRegisterIndex { index: usize, available: usize },
    
    #[error("No output register available")]
    NoOutputRegister,
    
    #[error("Empty instruction list")]
    EmptyInstructionList,
    
    #[error("Validation failed: {reason}")]
    ValidationFailed { reason: String },
    
    #[error("Dataflow analysis failed: {reason}")]
    DataflowFailed { reason: String },
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::normalizer::{RegisterAllocation, NormalizedMetadata};
    use crate::normalizer::dependency_tracker::RegisterId as NormRegId;
    use crate::bcib::{ComparisonOp, Value};
    use crate::types::SourceLocation;
    use std::collections::HashMap;
    
    fn test_location() -> SourceLocation {
        SourceLocation::new(1, 1, 0)
    }
    
    fn create_test_normalized_bcib() -> NormalizedBCIB {
        let instructions = vec![
            NormalizedInstruction {
                instruction: BCIBInstruction::Context(ContextInstruction::LoadContext {
                    path: "users".to_string(),
                    location: test_location(),
                }),
                input_registers: vec![],
                output_registers: vec![NormRegId::Context(0)],
                instruction_group: InstructionGroup::Context,
            },
            NormalizedInstruction {
                instruction: BCIBInstruction::Context(ContextInstruction::Return {
                    location: test_location(),
                }),
                input_registers: vec![NormRegId::Context(0)],
                output_registers: vec![],
                instruction_group: InstructionGroup::Control,
            },
        ];
        
        NormalizedBCIB {
            instructions,
            register_allocation: RegisterAllocation {
                allocated_registers: vec![NormRegId::Context(0)],
                register_dependencies: HashMap::new(),
                next_register: 1,
            },
            output_register: Some(0), // ✅ C8: Output register
            metadata: NormalizedMetadata {
                original_metadata: crate::bcib::BCIBMetadata::default(),
                normalization_timestamp: chrono::Utc::now().to_rfc3339(),
                instruction_count: 2,
                register_count: 1,
                determinism_fingerprint: "test".to_string(),
            },
        }
    }
    
    #[test]
    fn test_ir_builder_basic() {
        let mut builder = IRBuilder::new();
        let normalized_bcib = create_test_normalized_bcib();
        
        let result = builder.build_execution_plan(normalized_bcib);
        assert!(result.is_ok());
        
        let plan = result.unwrap();
        assert_eq!(plan.blocks.len(), 1);
        assert_eq!(plan.entry_block, 0);
        assert!(plan.validate().is_ok());
    }
    
    #[test]
    fn test_convert_context_instructions() {
        let mut builder = IRBuilder::new();
        
        let load_context = NormalizedInstruction {
            instruction: BCIBInstruction::Context(ContextInstruction::LoadContext {
                path: "users".to_string(),
                location: test_location(),
            }),
            input_registers: vec![],
            output_registers: vec![NormRegId::Context(0)],
            instruction_group: InstructionGroup::Context,
        };
        
        let ir_insts = builder.convert_instruction(&load_context).unwrap();
        assert_eq!(ir_insts.len(), 1);
        match &ir_insts[0] {
            IRInstruction::LoadContext { context_id, target_register } => {
                assert_eq!(context_id, "users");
                assert_eq!(*target_register, 0);
            },
            _ => panic!("Expected LoadContext instruction"),
        }
    }
    
    #[test]
    fn test_convert_query_instructions() {
        let mut builder = IRBuilder::new();
        
        let load_literal = NormalizedInstruction {
            instruction: BCIBInstruction::Query(QueryInstruction::LoadLiteral {
                value: Value::Number(42.0),
                target_register: 1,
                location: test_location(),
            }),
            input_registers: vec![],
            output_registers: vec![NormRegId::Data(1)],
            instruction_group: InstructionGroup::Data,
        };
        
        let ir_insts = builder.convert_instruction(&load_literal).unwrap();
        assert_eq!(ir_insts.len(), 1);
        match &ir_insts[0] {
            IRInstruction::LoadLiteral { value, target_register } => {
                assert_eq!(*value, Value::Number(42.0));
                assert_eq!(*target_register, 1);
            },
            _ => panic!("Expected LoadLiteral instruction"),
        }
    }
    
    #[test]
    fn test_block_allocation() {
        let mut builder = IRBuilder::new();
        
        let id1 = builder.allocate_block_id();
        let id2 = builder.allocate_block_id();
        
        assert_eq!(id1, 0);
        assert_eq!(id2, 1);
        assert_ne!(id1, id2);
    }
    
    #[test]
    fn test_empty_instruction_list_error() {
        let builder = IRBuilder::new();
        let result = builder.build_single_block(0, vec![], 0); // Add output_register parameter
        assert!(result.is_err());
    }
    
    #[test]
    fn test_determinism_fingerprint() {
        let mut builder1 = IRBuilder::new();
        let mut builder2 = IRBuilder::new();
        
        let normalized_bcib = create_test_normalized_bcib();
        
        let plan1 = builder1.build_execution_plan(normalized_bcib.clone()).unwrap();
        let plan2 = builder2.build_execution_plan(normalized_bcib).unwrap();
        
        // Same input should produce same fingerprint
        assert_eq!(
            plan1.metadata.determinism_fingerprint,
            plan2.metadata.determinism_fingerprint
        );
    }
    
    #[test]
    fn test_parallel_safety_annotation() {
        let mut builder = IRBuilder::new();
        let normalized_bcib = create_test_normalized_bcib();
        
        let plan = builder.build_execution_plan(normalized_bcib).unwrap();
        
        // Verify that the block has been annotated with parallel safety
        assert_eq!(plan.blocks.len(), 1);
        let block = &plan.blocks[0];
        
        // LoadContext is a pure operation, so the block should be Safe
        assert_eq!(block.parallel_safety, ParallelSafety::Safe);
    }
    
    #[test]
    fn test_parallel_safety_annotation_with_filter() {
        let mut builder = IRBuilder::new();
        
        // Create a normalized BCIB with a filter operation
        let instructions = vec![
            NormalizedInstruction {
                instruction: BCIBInstruction::Context(ContextInstruction::LoadContext {
                    path: "users".to_string(),
                    location: test_location(),
                }),
                input_registers: vec![],
                output_registers: vec![NormRegId::Context(0)],
                instruction_group: InstructionGroup::Context,
            },
            NormalizedInstruction {
                instruction: BCIBInstruction::Query(QueryInstruction::ApplyFilter {
                    expression: FilterExpression::new(
                        "age".to_string(),
                        ComparisonOp::GreaterThan,
                        OperandRef::Literal(Value::Number(18.0)),
                    ),
                    location: test_location(),
                }),
                input_registers: vec![NormRegId::Context(0)],
                output_registers: vec![NormRegId::Context(0)],
                instruction_group: InstructionGroup::Control,
            },
            NormalizedInstruction {
                instruction: BCIBInstruction::Context(ContextInstruction::Return {
                    location: test_location(),
                }),
                input_registers: vec![NormRegId::Context(0)],
                output_registers: vec![],
                instruction_group: InstructionGroup::Control,
            },
        ];
        
        let normalized_bcib = NormalizedBCIB {
            instructions,
            register_allocation: RegisterAllocation {
                allocated_registers: vec![NormRegId::Context(0)],
                register_dependencies: HashMap::new(),
                next_register: 1,
            },
            output_register: Some(0),
            metadata: NormalizedMetadata {
                original_metadata: crate::bcib::BCIBMetadata::default(),
                normalization_timestamp: chrono::Utc::now().to_rfc3339(),
                instruction_count: 3,
                register_count: 1,
                determinism_fingerprint: "test".to_string(),
            },
        };
        
        let plan = builder.build_execution_plan(normalized_bcib).unwrap();
        
        // Verify that the block has been annotated with parallel safety
        assert_eq!(plan.blocks.len(), 1);
        let block = &plan.blocks[0];
        
        // LoadContext and ApplyFilter are both pure operations, so the block should be Safe
        assert_eq!(block.parallel_safety, ParallelSafety::Safe);
    }
}