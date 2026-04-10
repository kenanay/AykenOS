//! Register Allocator - Sequential Virtual Register Assignment
//!
//! **Created By:** Kenan AY
//! **Date:** 15 Ocak 2026
//! **Architectural Reference:** C1 Section "Register Allocation Strategy", C3 Register Model Specification
//!
//! Implements sequential register allocation with no reuse (Gate C constraint).

use crate::bcib::{
    BCIBInstruction, BCIBSequence, ContextInstruction, OperandRef, QueryInstruction,
};
use crate::normalizer::dependency_tracker::RegisterId;
use crate::normalizer::RegisterAllocation;
use std::collections::{HashMap, HashSet};

/// Register allocation errors
#[derive(Debug, thiserror::Error)]
pub enum RegisterAllocationError {
    #[error("Register limit exceeded for category {category}: {count} > {limit}")]
    RegisterLimitExceeded {
        category: String,
        count: u32,
        limit: u32,
    },

    #[error("Invalid instruction for register allocation: {instruction}")]
    InvalidInstruction { instruction: String },

    #[error("Register type mismatch: expected {expected}, got {actual}")]
    TypeMismatch { expected: String, actual: String },
}

/// Register allocator with sequential assignment
///
/// **Architectural Reference:** C3 Section "Virtual Register Architecture"
pub struct RegisterAllocator {
    next_data_register: u16,
    next_context_register: u16,
    next_filter_register: u16,
    allocated_registers: HashSet<RegisterId>,
}

impl RegisterAllocator {
    /// Create new register allocator
    ///
    /// **Architectural Reference:** C3 Section "Register Allocation Algorithm"
    pub fn new() -> Self {
        Self {
            next_data_register: 0,
            next_context_register: 0,
            next_filter_register: 0,
            allocated_registers: HashSet::new(),
        }
    }

    /// Allocate data register (R0, R1, R2, ...)
    ///
    /// **Architectural Reference:** C3 Section "Register Categories"
    pub fn allocate_data_register(&mut self) -> Result<RegisterId, RegisterAllocationError> {
        const DATA_REGISTER_LIMIT: u16 = 100; // R0-R99

        if self.next_data_register >= DATA_REGISTER_LIMIT {
            return Err(RegisterAllocationError::RegisterLimitExceeded {
                category: "Data".to_string(),
                count: self.next_data_register as u32,
                limit: DATA_REGISTER_LIMIT as u32,
            });
        }

        let reg = RegisterId::Data(self.next_data_register);
        self.next_data_register += 1;
        self.allocated_registers.insert(reg.clone());
        Ok(reg)
    }

    /// Allocate context register (C0, C1, C2, ...)
    ///
    /// **Architectural Reference:** C3 Section "Register Categories"
    pub fn allocate_context_register(&mut self) -> Result<RegisterId, RegisterAllocationError> {
        const CONTEXT_REGISTER_LIMIT: u16 = 100; // C0-C99

        if self.next_context_register >= CONTEXT_REGISTER_LIMIT {
            return Err(RegisterAllocationError::RegisterLimitExceeded {
                category: "Context".to_string(),
                count: self.next_context_register as u32,
                limit: CONTEXT_REGISTER_LIMIT as u32,
            });
        }

        let reg = RegisterId::Context(self.next_context_register);
        self.next_context_register += 1;
        self.allocated_registers.insert(reg.clone());
        Ok(reg)
    }

    /// Allocate filter register (F0, F1, F2, ...)
    ///
    /// **Architectural Reference:** C3 Section "Register Categories"
    pub fn allocate_filter_register(&mut self) -> Result<RegisterId, RegisterAllocationError> {
        const FILTER_REGISTER_LIMIT: u16 = 100; // F0-F99

        if self.next_filter_register >= FILTER_REGISTER_LIMIT {
            return Err(RegisterAllocationError::RegisterLimitExceeded {
                category: "Filter".to_string(),
                count: self.next_filter_register as u32,
                limit: FILTER_REGISTER_LIMIT as u32,
            });
        }

        let reg = RegisterId::Filter(self.next_filter_register);
        self.next_filter_register += 1;
        self.allocated_registers.insert(reg.clone());
        Ok(reg)
    }

    /// Allocate registers for entire BCIB sequence
    ///
    /// **Architectural Reference:** C1 Section "Register Allocation Strategy"
    pub fn allocate_for_sequence(
        &mut self,
        bcib: &BCIBSequence,
    ) -> Result<RegisterAllocation, RegisterAllocationError> {
        let mut allocated_registers = Vec::new();
        let mut register_dependencies = HashMap::new();

        for instruction in &bcib.instructions {
            match instruction {
                BCIBInstruction::Context(ContextInstruction::LoadContext { .. }) => {
                    // LoadContext produces context register
                    let reg = self.allocate_context_register()?;
                    allocated_registers.push(reg);
                }

                BCIBInstruction::Query(QueryInstruction::LoadField {
                    target_register, ..
                }) => {
                    // LoadField produces data register
                    let reg = RegisterId::Data(*target_register);
                    allocated_registers.push(reg);
                }

                BCIBInstruction::Query(QueryInstruction::LoadLiteral {
                    target_register, ..
                }) => {
                    // LoadLiteral produces data register
                    let reg = RegisterId::Data(*target_register);
                    allocated_registers.push(reg);
                }

                BCIBInstruction::Query(QueryInstruction::Compare {
                    left,
                    right,
                    target_register,
                    ..
                }) => {
                    // Compare consumes two registers, produces data register
                    let reg = RegisterId::Data(*target_register);
                    allocated_registers.push(reg.clone());

                    // Track dependencies: target depends on left and right
                    let mut deps = Vec::new();
                    if let OperandRef::TempRegister(reg_id) = left {
                        deps.push(RegisterId::Data(*reg_id));
                    }
                    if let OperandRef::TempRegister(reg_id) = right {
                        deps.push(RegisterId::Data(*reg_id));
                    }
                    register_dependencies.insert(reg, deps);
                }

                BCIBInstruction::Query(QueryInstruction::LogicalOp {
                    operands,
                    target_register,
                    ..
                }) => {
                    // LogicalOp consumes multiple registers, produces data register
                    let reg = RegisterId::Data(*target_register);
                    allocated_registers.push(reg.clone());

                    // Track dependencies: target depends on all operands
                    let mut deps = Vec::new();
                    for operand in operands {
                        if let OperandRef::TempRegister(reg_id) = operand {
                            deps.push(RegisterId::Data(*reg_id));
                        }
                    }
                    register_dependencies.insert(reg, deps);
                }

                BCIBInstruction::Query(QueryInstruction::ApplyFilter { .. }) => {
                    // ApplyFilter modifies context in-place, no new registers
                    // It consumes context register and filter expression
                }

                BCIBInstruction::Query(QueryInstruction::ApplyFilterBool {
                    filter_register,
                    ..
                }) => {
                    // ApplyFilterBool consumes register, produces no new registers
                    // Track dependency for completeness
                    let reg = RegisterId::Filter(*filter_register);
                    register_dependencies.insert(reg, vec![]);
                }

                BCIBInstruction::Context(ContextInstruction::Return { .. }) => {
                    // Return consumes register, produces no new registers
                    // No specific register tracking needed for Return
                }

                _ => {
                    return Err(RegisterAllocationError::InvalidInstruction {
                        instruction: format!("{:?}", instruction),
                    });
                }
            }
        }

        Ok(RegisterAllocation {
            allocated_registers,
            register_dependencies,
            next_register: self
                .next_data_register
                .max(self.next_context_register)
                .max(self.next_filter_register),
        })
    }

    /// Get allocation statistics
    pub fn get_allocation_stats(&self) -> AllocationStats {
        AllocationStats {
            data_registers_allocated: self.next_data_register,
            context_registers_allocated: self.next_context_register,
            filter_registers_allocated: self.next_filter_register,
            total_registers_allocated: self.allocated_registers.len() as u16,
        }
    }

    /// Reset allocator state
    pub fn reset(&mut self) {
        self.next_data_register = 0;
        self.next_context_register = 0;
        self.next_filter_register = 0;
        self.allocated_registers.clear();
    }
}

/// Allocation statistics
#[derive(Debug, Clone, PartialEq)]
pub struct AllocationStats {
    pub data_registers_allocated: u16,
    pub context_registers_allocated: u16,
    pub filter_registers_allocated: u16,
    pub total_registers_allocated: u16,
}

impl Default for RegisterAllocator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bcib::BCIBMetadata;

    #[test]
    fn test_sequential_allocation() {
        let mut allocator = RegisterAllocator::new();

        // Allocate data registers sequentially
        let r0 = allocator.allocate_data_register().unwrap();
        let r1 = allocator.allocate_data_register().unwrap();
        let r2 = allocator.allocate_data_register().unwrap();

        assert_eq!(r0, RegisterId::Data(0));
        assert_eq!(r1, RegisterId::Data(1));
        assert_eq!(r2, RegisterId::Data(2));

        // Allocate context registers sequentially
        let c0 = allocator.allocate_context_register().unwrap();
        let c1 = allocator.allocate_context_register().unwrap();

        assert_eq!(c0, RegisterId::Context(0));
        assert_eq!(c1, RegisterId::Context(1));

        // Allocate filter registers sequentially
        let f0 = allocator.allocate_filter_register().unwrap();
        let f1 = allocator.allocate_filter_register().unwrap();

        assert_eq!(f0, RegisterId::Filter(0));
        assert_eq!(f1, RegisterId::Filter(1));
    }

    #[test]
    fn test_allocation_for_simple_sequence() {
        let mut allocator = RegisterAllocator::new();

        let bcib = BCIBSequence {
            instructions: vec![
                BCIBInstruction::Context(ContextInstruction::LoadContext {
                    path: "users".to_string(),
                    location: crate::types::SourceLocation::new(1, 1, 0),
                }),
                BCIBInstruction::Context(ContextInstruction::Return {
                    location: crate::types::SourceLocation::new(1, 1, 0),
                }),
            ],
            metadata: BCIBMetadata::default(),
        };

        let allocation = allocator.allocate_for_sequence(&bcib).unwrap();

        // Should allocate one context register for LoadContext
        assert_eq!(allocation.allocated_registers.len(), 1);
        assert_eq!(allocation.allocated_registers[0], RegisterId::Context(0));
    }

    #[test]
    fn test_register_limit_enforcement() {
        let mut allocator = RegisterAllocator::new();

        // Manually set register counter near limit
        allocator.next_data_register = 99;

        // Should succeed (R99)
        let r99 = allocator.allocate_data_register();
        assert!(r99.is_ok());
        assert_eq!(r99.unwrap(), RegisterId::Data(99));

        // Should fail (would be R100, exceeds limit)
        let r100 = allocator.allocate_data_register();
        assert!(r100.is_err());

        match r100.unwrap_err() {
            RegisterAllocationError::RegisterLimitExceeded {
                category,
                count,
                limit,
            } => {
                assert_eq!(category, "Data");
                assert_eq!(count, 100);
                assert_eq!(limit, 100);
            }
            _ => panic!("Expected RegisterLimitExceeded error"),
        }
    }

    /// **Property Test: Deterministic Allocation**
    ///
    /// **Architectural Reference:** C1 Section "Determinism Guarantee"
    #[test]
    fn test_allocation_determinism() {
        let bcib = BCIBSequence {
            instructions: vec![
                BCIBInstruction::Context(ContextInstruction::LoadContext {
                    path: "users".to_string(),
                    location: crate::types::SourceLocation::new(1, 1, 0),
                }),
                BCIBInstruction::Query(QueryInstruction::LoadField {
                    field: "name".to_string(),
                    target_register: 1,
                    location: crate::types::SourceLocation::new(1, 1, 0),
                }),
                BCIBInstruction::Context(ContextInstruction::Return {
                    location: crate::types::SourceLocation::new(1, 1, 0),
                }),
            ],
            metadata: BCIBMetadata::default(),
        };

        let mut allocator1 = RegisterAllocator::new();
        let mut allocator2 = RegisterAllocator::new();

        let allocation1 = allocator1.allocate_for_sequence(&bcib).unwrap();
        let allocation2 = allocator2.allocate_for_sequence(&bcib).unwrap();

        // Same input should produce same allocation
        assert_eq!(
            allocation1.allocated_registers,
            allocation2.allocated_registers
        );
    }
}
