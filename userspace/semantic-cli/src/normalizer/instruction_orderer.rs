//! Instruction Orderer - Canonical Instruction Ordering
//! 
//! **Created By:** Kenan AY
//! **Date:** 15 Ocak 2026
//! **Architectural Reference:** C1 Section "Canonical Instruction Order"
//! 
//! Orders instructions according to canonical rules for deterministic execution.

use crate::bcib::{BCIBSequence, BCIBInstruction, ContextInstruction, QueryInstruction};
use crate::normalizer::{NormalizedInstruction, InstructionGroup};
use crate::normalizer::dependency_tracker::{DependencyGraph, InstructionId};
use std::collections::HashMap;

/// Instruction ordering errors
#[derive(Debug, thiserror::Error)]
pub enum OrderingError {
    #[error("Invalid instruction order: {reason}")]
    InvalidOrder { reason: String },
    
    #[error("Dependency constraint violation: {constraint}")]
    DependencyViolation { constraint: String },
    
    #[error("Unknown instruction type: {instruction}")]
    UnknownInstruction { instruction: String },
}

/// Instruction orderer
/// 
/// **Architectural Reference:** C1 Section "Canonical Instruction Order"
pub struct InstructionOrderer {
    // Internal state for ordering
}

impl InstructionOrderer {
    /// Create new instruction orderer
    pub fn new() -> Self {
        Self {}
    }
    
    /// Order instructions according to canonical rules
    /// 
    /// **Architectural Reference:** C1 Section "Normalization Rules"
    /// **Gate C Rule:** Simple canonical ordering, dependency analysis separate
    /// 
    /// **Ordering Priority:**
    /// 1. LoadContext instructions (context dependencies first)
    /// 2. LoadField instructions (field loading)
    /// 3. LoadLiteral instructions (literal values)
    /// 4. Compare instructions (comparisons)
    /// 5. LogicalOp instructions (logical operations)
    /// 6. ApplyFilter instructions (filter application)
    /// 7. Return instructions (results)
    pub fn order(&mut self, bcib: &BCIBSequence, _dependencies: &DependencyGraph) -> Result<Vec<NormalizedInstruction>, OrderingError> {
        // **Step 1: Group instructions by type**
        let grouped_instructions = self.group_instructions_by_type(bcib)?;
        
        // **Step 2: Order within groups in canonical order**
        let mut ordered_instructions = Vec::new();
        
        // Process groups in canonical order
        let group_order = [
            InstructionGroup::Context,
            InstructionGroup::Data,
            InstructionGroup::Compute,
            InstructionGroup::Control,
        ];
        
        for group in &group_order {
            if let Some(group_instructions) = grouped_instructions.get(group) {
                let ordered_group = self.order_within_group(group_instructions, _dependencies)?;
                ordered_instructions.extend(ordered_group);
            }
        }
        
        Ok(ordered_instructions)
    }
    
    /// Group instructions by their canonical type
    /// 
    /// **Architectural Reference:** C1 Section "Instruction Grouping"
    fn group_instructions_by_type<'a>(&self, bcib: &'a BCIBSequence) -> Result<HashMap<InstructionGroup, Vec<(usize, &'a BCIBInstruction)>>, OrderingError> {
        let mut groups: HashMap<InstructionGroup, Vec<(usize, &'a BCIBInstruction)>> = HashMap::new();
        
        for (idx, instruction) in bcib.instructions.iter().enumerate() {
            let group = self.classify_instruction(instruction)?;
            groups.entry(group).or_insert_with(Vec::new).push((idx, instruction));
        }
        
        Ok(groups)
    }
    
    /// Classify instruction into canonical group
    /// 
    /// **Architectural Reference:** C1 Section "Instruction Grouping"
    fn classify_instruction(&self, instruction: &BCIBInstruction) -> Result<InstructionGroup, OrderingError> {
        match instruction {
            BCIBInstruction::Context(ContextInstruction::LoadContext { .. }) => Ok(InstructionGroup::Context),
            
            BCIBInstruction::Query(QueryInstruction::LoadField { .. }) | 
            BCIBInstruction::Query(QueryInstruction::LoadLiteral { .. }) => Ok(InstructionGroup::Data),
            
            BCIBInstruction::Query(QueryInstruction::Compare { .. }) | 
            BCIBInstruction::Query(QueryInstruction::LogicalOp { .. }) => Ok(InstructionGroup::Compute),
            
            BCIBInstruction::Query(QueryInstruction::ApplyFilter { .. }) |
            BCIBInstruction::Query(QueryInstruction::ApplyFilterBool { .. }) |
            BCIBInstruction::Context(ContextInstruction::Return { .. }) => Ok(InstructionGroup::Control),
            
            _ => Err(OrderingError::UnknownInstruction {
                instruction: format!("{:?}", instruction),
            }),
        }
    }
    
    /// Order instructions within a group respecting dependencies
    /// 
    /// **Architectural Reference:** C2 Section "Dependency Analysis"
    /// **Gate C Rule:** Simple canonical ordering, dependency analysis separate
    fn order_within_group(&self, group_instructions: &[(usize, &BCIBInstruction)], _dependencies: &DependencyGraph) -> Result<Vec<NormalizedInstruction>, OrderingError> {
        let mut ordered = Vec::new();
        
        // Simple canonical ordering within group (no dependency analysis needed here)
        for (_idx, instruction) in group_instructions {
            let instruction_group = self.classify_instruction(instruction)?;
            
            // Create normalized instruction with empty register info (will be filled by dependency tracker)
            let normalized = NormalizedInstruction {
                instruction: (*instruction).clone(),
                input_registers: vec![], // Will be filled by dependency tracker
                output_registers: vec![], // Will be filled by dependency tracker
                instruction_group,
            };
            
            ordered.push(normalized);
        }
        
        Ok(ordered)
    }
    
    /// Create normalized instruction with register information
    /// 
    /// **Architectural Reference:** C1 Section "Normalized Instruction"
    fn create_normalized_instruction(&self, idx: usize, instruction: &BCIBInstruction, dependencies: &DependencyGraph) -> Result<NormalizedInstruction, OrderingError> {
        let instruction_id = InstructionId(idx);
        
        // Find dependency node for this instruction
        let dep_node = dependencies.nodes.iter()
            .find(|node| node.instruction_id == instruction_id)
            .ok_or_else(|| OrderingError::InvalidOrder {
                reason: format!("Dependency node not found for instruction {}", idx),
            })?;
        
        let instruction_group = self.classify_instruction(instruction)?;
        
        Ok(NormalizedInstruction {
            instruction: (*instruction).clone(),
            input_registers: dep_node.inputs.clone(),
            output_registers: dep_node.outputs.clone(),
            instruction_group,
        })
    }
    
    /// Apply topological ordering based on dependency graph
    /// 
    /// **Architectural Reference:** C1 Section "Canonical Instruction Order"
    /// **Gate C Rule:** Normalizer controls canonical ordering, not DependencyTracker
    pub fn apply_topological_order(&self, instructions: &[NormalizedInstruction], dependencies: &DependencyGraph) -> Result<Vec<NormalizedInstruction>, OrderingError> {
        // **Step 1: Build instruction index mapping**
        let mut instruction_map = HashMap::new();
        for (pos, normalized_inst) in instructions.iter().enumerate() {
            // Find corresponding dependency node
            for dep_node in &dependencies.nodes {
                if dep_node.inputs == normalized_inst.input_registers && 
                   dep_node.outputs == normalized_inst.output_registers {
                    instruction_map.insert(dep_node.instruction_id.0, pos);
                    break;
                }
            }
        }
        
        // **Step 2: Compute topological order using Kahn's algorithm**
        let topo_order = self.compute_topological_order(dependencies)?;
        
        // **Step 3: Reorder instructions according to topological order**
        let mut ordered_instructions = Vec::new();
        for instruction_id in topo_order {
            if let Some(&pos) = instruction_map.get(&instruction_id.0) {
                ordered_instructions.push(instructions[pos].clone());
            }
        }
        
        // **Step 4: Validate final order**
        self.validate_dependency_order(&ordered_instructions, dependencies)?;
        
        Ok(ordered_instructions)
    }
    
    /// Compute topological order using Kahn's algorithm
    /// 
    /// **Architectural Reference:** C1 Section "Canonical Instruction Order"
    /// **Gate C Rule:** Single method for deterministic ordering
    fn compute_topological_order(&self, dependencies: &DependencyGraph) -> Result<Vec<InstructionId>, OrderingError> {
        let mut in_degree = HashMap::new();
        let mut adj_list: HashMap<InstructionId, Vec<InstructionId>> = HashMap::new();
        
        // Initialize in-degree and adjacency list
        for node in &dependencies.nodes {
            in_degree.insert(node.instruction_id.clone(), 0);
            adj_list.insert(node.instruction_id.clone(), Vec::new());
        }
        
        // Build adjacency list and compute in-degrees
        for node in &dependencies.nodes {
            for dep in &node.dependencies {
                adj_list.get_mut(dep).unwrap().push(node.instruction_id.clone());
                *in_degree.get_mut(&node.instruction_id).unwrap() += 1;
            }
        }
        
        // Kahn's algorithm
        let mut queue = Vec::new();
        let mut result = Vec::new();
        
        // Find all nodes with in-degree 0
        for (node_id, &degree) in &in_degree {
            if degree == 0 {
                queue.push(node_id.clone());
            }
        }
        
        // Sort queue for deterministic ordering
        queue.sort_by_key(|id| id.0);
        
        while let Some(current) = queue.pop() {
            result.push(current.clone());
            
            // Process neighbors
            if let Some(neighbors) = adj_list.get(&current) {
                let mut new_zero_degree = Vec::new();
                for neighbor in neighbors {
                    let degree = in_degree.get_mut(neighbor).unwrap();
                    *degree -= 1;
                    if *degree == 0 {
                        new_zero_degree.push(neighbor.clone());
                    }
                }
                
                // Sort new zero-degree nodes for deterministic ordering
                new_zero_degree.sort_by_key(|id| id.0);
                queue.extend(new_zero_degree);
                queue.sort_by_key(|id| id.0);
            }
        }
        
        // Check for cycles
        if result.len() != dependencies.nodes.len() {
            return Err(OrderingError::InvalidOrder {
                reason: "Circular dependency detected in topological sort".to_string(),
            });
        }
        
        Ok(result)
    }
    /// Validate that final order respects all dependencies
    /// 
    /// **Architectural Reference:** C1 Section "Output Validation"
    fn validate_dependency_order(&self, ordered_instructions: &[NormalizedInstruction], dependencies: &DependencyGraph) -> Result<(), OrderingError> {
        // Build position map for instructions
        let mut position_map = HashMap::new();
        for (pos, normalized_inst) in ordered_instructions.iter().enumerate() {
            // Find original instruction index
            for (orig_idx, dep_node) in dependencies.nodes.iter().enumerate() {
                if dep_node.inputs == normalized_inst.input_registers && 
                   dep_node.outputs == normalized_inst.output_registers {
                    position_map.insert(orig_idx, pos);
                    break;
                }
            }
        }
        
        // Check that dependencies are satisfied
        for (orig_idx, dep_node) in dependencies.nodes.iter().enumerate() {
            let current_pos = position_map.get(&orig_idx)
                .ok_or_else(|| OrderingError::DependencyViolation {
                    constraint: format!("Instruction {} not found in final order", orig_idx),
                })?;
            
            for dep_instruction in &dep_node.dependencies {
                let dep_pos = position_map.get(&dep_instruction.0)
                    .ok_or_else(|| OrderingError::DependencyViolation {
                        constraint: format!("Dependency instruction {} not found", dep_instruction.0),
                    })?;
                
                if dep_pos >= current_pos {
                    return Err(OrderingError::DependencyViolation {
                        constraint: format!(
                            "Instruction {} depends on instruction {} but appears before it",
                            orig_idx, dep_instruction.0
                        ),
                    });
                }
            }
        }
        
        Ok(())
    }
}

impl Default for InstructionOrderer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bcib::{BCIBMetadata, Value, OperandRef, ComparisonOp};
    use crate::normalizer::dependency_tracker::DependencyNode;
    use crate::normalizer::RegisterId;
    use std::collections::HashMap;
    
    #[test]
    fn test_instruction_classification() {
        let orderer = InstructionOrderer::new();
        
        // Test LoadContext classification
        let load_context = BCIBInstruction::Context(ContextInstruction::LoadContext {
            path: "users".to_string(),
            location: crate::types::SourceLocation::new(1, 1, 0),
        });
        assert_eq!(orderer.classify_instruction(&load_context).unwrap(), InstructionGroup::Context);
        
        // Test LoadField classification
        let load_field = BCIBInstruction::Query(QueryInstruction::LoadField {
            field: "name".to_string(),
            target_register: 0,
            location: crate::types::SourceLocation::new(1, 1, 0),
        });
        assert_eq!(orderer.classify_instruction(&load_field).unwrap(), InstructionGroup::Data);
        
        // Test Compare classification
        let compare = BCIBInstruction::Query(QueryInstruction::Compare {
            left: OperandRef::TempRegister(0),
            operator: ComparisonOp::Equal,
            right: OperandRef::TempRegister(1),
            target_register: 2,
            location: crate::types::SourceLocation::new(1, 1, 0),
        });
        assert_eq!(orderer.classify_instruction(&compare).unwrap(), InstructionGroup::Compute);
        
        // Test Return classification
        let return_inst = BCIBInstruction::Context(ContextInstruction::Return {
            location: crate::types::SourceLocation::new(1, 1, 0),
        });
        assert_eq!(orderer.classify_instruction(&return_inst).unwrap(), InstructionGroup::Control);
    }
    
    #[test]
    fn test_canonical_ordering() {
        let mut orderer = InstructionOrderer::new();
        
        // Create BCIB with instructions in non-canonical order
        let bcib = BCIBSequence {
            instructions: vec![
                // Return first (should be last)
                BCIBInstruction::Context(ContextInstruction::Return {
                    location: crate::types::SourceLocation::new(1, 1, 0),
                }),
                // LoadField second (should be after LoadContext)
                BCIBInstruction::Query(QueryInstruction::LoadField {
                    field: "name".to_string(),
                    target_register: 0,
                    location: crate::types::SourceLocation::new(1, 1, 0),
                }),
                // LoadContext third (should be first)
                BCIBInstruction::Context(ContextInstruction::LoadContext {
                    path: "users".to_string(),
                    location: crate::types::SourceLocation::new(1, 1, 0),
                }),
            ],
            metadata: BCIBMetadata::default(),
        };
        
        // Create dependency graph
        let dependencies = DependencyGraph {
            nodes: vec![
                DependencyNode {
                    instruction_id: InstructionId(0),
                    inputs: vec![RegisterId::Data(0)],
                    outputs: vec![],
                    dependencies: vec![InstructionId(1)],
                },
                DependencyNode {
                    instruction_id: InstructionId(1),
                    inputs: vec![RegisterId::Context(0)],
                    outputs: vec![RegisterId::Data(0)],
                    dependencies: vec![InstructionId(2)],
                },
                DependencyNode {
                    instruction_id: InstructionId(2),
                    inputs: vec![],
                    outputs: vec![RegisterId::Context(0)],
                    dependencies: vec![],
                },
            ],
            register_definitions: HashMap::new(),
            register_uses: HashMap::new(),
        };
        
        let ordered = orderer.order(&bcib, &dependencies).unwrap();
        
        // Should be reordered to: LoadContext, LoadField, Return
        assert_eq!(ordered.len(), 3);
        
        // Check canonical order
        assert_eq!(ordered[0].instruction_group, InstructionGroup::Context);
        assert_eq!(ordered[1].instruction_group, InstructionGroup::Data);
        assert_eq!(ordered[2].instruction_group, InstructionGroup::Control);
        
        // Verify specific instructions
        match &ordered[0].instruction {
            BCIBInstruction::Context(ContextInstruction::LoadContext { .. }) => {},
            _ => panic!("Expected LoadContext as first instruction"),
        }
        
        match &ordered[1].instruction {
            BCIBInstruction::Query(QueryInstruction::LoadField { .. }) => {},
            _ => panic!("Expected LoadField as second instruction"),
        }
        
        match &ordered[2].instruction {
            BCIBInstruction::Context(ContextInstruction::Return { .. }) => {},
            _ => panic!("Expected Return as third instruction"),
        }
    }
    
    #[test]
    fn test_dependency_validation() {
        let orderer = InstructionOrderer::new();
        
        // Create ordered instructions that violate dependencies
        let ordered_instructions = vec![
            NormalizedInstruction {
                instruction: BCIBInstruction::Context(ContextInstruction::Return {
                    location: crate::types::SourceLocation::new(1, 1, 0),
                }),
                input_registers: vec![RegisterId::Data(0)],
                output_registers: vec![],
                instruction_group: InstructionGroup::Control,
            },
            NormalizedInstruction {
                instruction: BCIBInstruction::Query(QueryInstruction::LoadLiteral {
                    value: Value::String("test".to_string()),
                    target_register: 0,
                    location: crate::types::SourceLocation::new(1, 1, 0),
                }),
                input_registers: vec![],
                output_registers: vec![RegisterId::Data(0)],
                instruction_group: InstructionGroup::Data,
            },
        ];
        
        // Create dependency graph showing Return depends on LoadLiteral
        let dependencies = DependencyGraph {
            nodes: vec![
                DependencyNode {
                    instruction_id: InstructionId(0),
                    inputs: vec![RegisterId::Data(0)],
                    outputs: vec![],
                    dependencies: vec![InstructionId(1)],
                },
                DependencyNode {
                    instruction_id: InstructionId(1),
                    inputs: vec![],
                    outputs: vec![RegisterId::Data(0)],
                    dependencies: vec![],
                },
            ],
            register_definitions: HashMap::new(),
            register_uses: HashMap::new(),
        };
        
        // Validation should fail because Return appears before LoadLiteral
        let result = orderer.validate_dependency_order(&ordered_instructions, &dependencies);
        assert!(result.is_err());
        
        match result.unwrap_err() {
            OrderingError::DependencyViolation { constraint } => {
                assert!(constraint.contains("depends on"));
            },
            _ => panic!("Expected DependencyViolation error"),
        }
    }
}
