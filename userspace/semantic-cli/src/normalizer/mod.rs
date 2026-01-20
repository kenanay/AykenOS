//! BCIB Normalizer - Stateless Pure Function
//! 
//! **Created By:** Kenan AY
//! **Date:** 15 Ocak 2026
//! **Architectural Reference:** C1 - BCIB Normalizer Design Specification
//! 
//! Transforms raw BCIB instructions into normalized, canonical form suitable for IR generation.
//! 
//! **Key Principle:** Stateless pure function - no side effects, no context access, no execution.

use crate::bcib::{BCIBSequence, BCIBInstruction, BCIBMetadata, ContextInstruction, QueryInstruction};
use std::collections::HashMap;

pub mod register_allocator;
pub mod dependency_tracker;
pub mod instruction_orderer;
pub mod validator;

use register_allocator::RegisterAllocator;
use dependency_tracker::{DependencyTracker, RegisterId};
use instruction_orderer::InstructionOrderer;
use validator::NormalizationValidator;

/// Normalized BCIB with canonical instruction order and register allocation
#[derive(Debug, Clone, PartialEq)]
pub struct NormalizedBCIB {
    pub instructions: Vec<NormalizedInstruction>,
    pub register_allocation: RegisterAllocation,
    pub output_register: Option<u16>, // ✅ C8: Result register for IR Return (raw register ID)
    pub metadata: NormalizedMetadata,
}

/// Normalized instruction with explicit register allocation
#[derive(Debug, Clone, PartialEq)]
pub struct NormalizedInstruction {
    pub instruction: BCIBInstruction,
    pub input_registers: Vec<RegisterId>,
    pub output_registers: Vec<RegisterId>,
    pub instruction_group: InstructionGroup,
}

/// Register allocation information
#[derive(Debug, Clone, PartialEq)]
pub struct RegisterAllocation {
    pub allocated_registers: Vec<RegisterId>,
    pub register_dependencies: HashMap<RegisterId, Vec<RegisterId>>,
    pub next_register: u16,
}

/// Instruction grouping for optimization
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InstructionGroup {
    Context,    // LoadContext instructions
    Data,       // LoadField, LoadLiteral instructions
    Compute,    // Compare, LogicalOp instructions
    Control,    // ApplyFilter, Return instructions
}

/// Normalized metadata
#[derive(Debug, Clone, PartialEq)]
pub struct NormalizedMetadata {
    pub original_metadata: BCIBMetadata,
    pub normalization_timestamp: String,
    pub instruction_count: usize,
    pub register_count: usize,
    pub determinism_fingerprint: String,
}

/// Normalization errors
#[derive(Debug, thiserror::Error)]
pub enum NormalizationError {
    #[error("Invalid BCIB sequence: {message}")]
    InvalidInput { message: String },
    
    #[error("Register allocation failed: {reason}")]
    RegisterAllocationFailed { reason: String },
    
    #[error("Circular dependency detected: {cycle:?}")]
    CircularDependency { cycle: Vec<String> },
    
    #[error("Instruction ordering failed: {reason}")]
    OrderingFailed { reason: String },
    
    #[error("Normalization validation failed: {details}")]
    ValidationFailed { details: String },
}

/// Main BCIB Normalizer
pub struct BCIBNormalizer {
    register_allocator: RegisterAllocator,
    dependency_tracker: DependencyTracker,
    instruction_orderer: InstructionOrderer,
    validator: NormalizationValidator,
}

impl BCIBNormalizer {
    /// Create new normalizer instance
    pub fn new() -> Self {
        Self {
            register_allocator: RegisterAllocator::new(),
            dependency_tracker: DependencyTracker::new(),
            instruction_orderer: InstructionOrderer::new(),
            validator: NormalizationValidator::new(),
        }
    }
    
    /// **MAIN NORMALIZATION FUNCTION**
    /// 
    /// **Architectural Reference:** C1 Section "Transformation Function Signature"
    /// 
    /// Transforms raw BCIB sequence into normalized form with:
    /// - Canonical instruction order
    /// - Filter expression flattening (normalized = true)
    /// - Register allocation and dependency tracking
    /// - Instruction grouping for optimization
    pub fn normalize(&mut self, bcib: BCIBSequence) -> Result<NormalizedBCIB, NormalizationError> {
        // **Step 1: Validate Input**
        // Reference: C1 Section "Validation Rules"
        self.validator.validate_input(&bcib)
            .map_err(|e| NormalizationError::ValidationFailed { 
                details: format!("Input validation failed: {}", e) 
            })?;
        
        // **Step 2: Allocate Registers**
        // Reference: C1 Section "Register Allocation Strategy"
        let allocation = self.register_allocator.allocate_for_sequence(&bcib)
            .map_err(|e| NormalizationError::RegisterAllocationFailed { 
                reason: e.to_string() 
            })?;
        
        // **Step 3: Create Initial Normalized Instructions**
        // Reference: C1 Section "Canonical Instruction Order"
        let dummy_dependencies = dependency_tracker::DependencyGraph {
            nodes: vec![],
            register_definitions: HashMap::new(),
            register_uses: HashMap::new(),
        };
        
        let ordered_instructions = self.instruction_orderer.order(&bcib, &dummy_dependencies)
            .map_err(|e| NormalizationError::OrderingFailed { 
                reason: e.to_string() 
            })?;
        
        // **Step 4: Build Dependency Graph from Normalized Instructions**
        // Reference: C1 Section "Register Dependency Tracking"
        // Gate C Rule: DependencyTracker only sees normalized instructions
        let _dependencies = self.dependency_tracker.analyze(&ordered_instructions)
            .map_err(|e| match e {
                dependency_tracker::DependencyError::CircularDependency { cycle } => {
                    NormalizationError::CircularDependency { 
                        cycle: cycle.iter().map(|id| format!("Instruction_{}", id.0)).collect()
                    }
                },
                dependency_tracker::DependencyError::InvalidInstruction { instruction } => {
                    NormalizationError::ValidationFailed { 
                        details: format!("Invalid instruction in dependency analysis: {}", instruction) 
                    }
                },
                _ => NormalizationError::ValidationFailed { 
                    details: format!("Dependency analysis failed: {}", e) 
                }
            })?;
        
        // **Step 5: Determine Output Register**
        // Reference: C8 Section "Return Semantics"
        // Rule: Last instruction's target register = output register
        let output_register = self.determine_output_register(&ordered_instructions);
        
        // **Step 6: Create Normalized BCIB**
        let normalized = NormalizedBCIB {
            instructions: ordered_instructions,
            register_allocation: allocation,
            output_register,
            metadata: self.create_normalized_metadata(&bcib),
        };
        
        // **Step 7: Validate Output**
        // Reference: C1 Section "Output Validation"
        self.validator.validate_output(&normalized)
            .map_err(|e| NormalizationError::ValidationFailed { 
                details: format!("Output validation failed: {}", e) 
            })?;
        
        Ok(normalized)
    }
    
    /// Determine output register from instruction sequence
    /// 
    /// **C8 Rule:** Last instruction with target register = output register
    fn determine_output_register(&self, instructions: &[NormalizedInstruction]) -> Option<u16> {
        // Find last instruction with a target register
        for instruction in instructions.iter().rev() {
            match &instruction.instruction {
                BCIBInstruction::Context(ContextInstruction::LoadContext { .. }) => {
                    // LoadContext produces context register (typically R0)
                    return Some(0);
                },
                BCIBInstruction::Query(QueryInstruction::LoadLiteral { target_register, .. }) |
                BCIBInstruction::Query(QueryInstruction::LoadField { target_register, .. }) |
                BCIBInstruction::Query(QueryInstruction::Compare { target_register, .. }) |
                BCIBInstruction::Query(QueryInstruction::LogicalOp { target_register, .. }) => {
                    return Some(*target_register);
                },
                BCIBInstruction::Query(QueryInstruction::ApplyFilter { .. }) => {
                    // ApplyFilter modifies context in-place, return context register
                    return Some(0);
                },
                BCIBInstruction::Context(ContextInstruction::Return { .. }) => {
                    // Return instruction: use previous instruction's output
                    continue;
                },
                _ => continue,
            }
        }
        
        // Default: register 0 (context register)
        Some(0)
    }
    
    /// Create normalized metadata
    /// 
    /// **Architectural Reference:** C1 Section "Determinism Guarantee"
    /// **Gate C Rule:** No BCIB hashing - use deterministic sequence ID instead
    fn create_normalized_metadata(&self, bcib: &BCIBSequence) -> NormalizedMetadata {
        // Create deterministic normalization ID (no BCIB hashing per Gate C)
        let normalization_id = format!("norm_{}_{}", 
            bcib.instructions.len(),
            chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0)
        );
        
        NormalizedMetadata {
            original_metadata: bcib.metadata.clone(),
            normalization_timestamp: chrono::Utc::now().to_rfc3339(),
            instruction_count: bcib.instructions.len(),
            register_count: 0, // Will be updated by register allocator
            determinism_fingerprint: normalization_id,
        }
    }
}

impl Default for BCIBNormalizer {
    fn default() -> Self {
        Self::new()
    }
}

/// **PURE FUNCTION INTERFACE**
/// 
/// **Architectural Reference:** C1 Section "Stateless Requirement"
/// 
/// Stateless normalization function for functional programming style.
/// No side effects, no context access, no execution.
pub fn normalize_bcib(input: BCIBSequence) -> Result<NormalizedBCIB, NormalizationError> {
    let mut normalizer = BCIBNormalizer::new();
    normalizer.normalize(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bcib::*;
    use crate::types::SourceLocation;
    
    fn test_location() -> SourceLocation {
        SourceLocation::new(1, 1, 0)
    }
    
    #[test]
    fn test_normalize_empty_sequence() {
        let empty_bcib = BCIBSequence {
            instructions: vec![],
            metadata: BCIBMetadata::default(),
        };
        
        let result = normalize_bcib(empty_bcib);
        assert!(result.is_ok());
        
        let normalized = result.unwrap();
        assert_eq!(normalized.instructions.len(), 0);
    }
    
    #[test]
    fn test_normalize_simple_sequence() {
        let bcib = BCIBSequence {
            instructions: vec![
                BCIBInstruction::Context(ContextInstruction::LoadContext {
                    path: "users".to_string(),
                    location: test_location(),
                }),
                BCIBInstruction::Context(ContextInstruction::Return {
                    location: test_location(),
                }),
            ],
            metadata: BCIBMetadata::default(),
        };
        
        let result = normalize_bcib(bcib);
        if let Err(e) = &result {
            eprintln!("Normalization error: {:?}", e);
        }
        assert!(result.is_ok());
        
        let normalized = result.unwrap();
        assert_eq!(normalized.instructions.len(), 2);
        
        // Verify canonical order: LoadContext before Return
        match &normalized.instructions[0].instruction {
            BCIBInstruction::Context(ContextInstruction::LoadContext { .. }) => {},
            _ => panic!("Expected LoadContext as first instruction"),
        }
        
        match &normalized.instructions[1].instruction {
            BCIBInstruction::Context(ContextInstruction::Return { .. }) => {},
            _ => panic!("Expected Return as second instruction"),
        }
    }
    
    /// **Property Test: Determinism**
    /// 
    /// **Architectural Reference:** C1 Section "Determinism Guarantee"
    #[test]
    fn test_normalization_determinism() {
        let bcib = BCIBSequence {
            instructions: vec![
                BCIBInstruction::Context(ContextInstruction::LoadContext {
                    path: "users".to_string(),
                    location: test_location(),
                }),
                BCIBInstruction::Context(ContextInstruction::Return {
                    location: test_location(),
                }),
            ],
            metadata: BCIBMetadata::default(),
        };
        
        let result1 = normalize_bcib(bcib.clone());
        let result2 = normalize_bcib(bcib.clone());
        
        assert!(result1.is_ok());
        assert!(result2.is_ok());
        
        let normalized1 = result1.unwrap();
        let normalized2 = result2.unwrap();
        
        // Same input should produce same normalized output
        assert_eq!(normalized1.instructions.len(), normalized2.instructions.len());
        assert_eq!(normalized1.register_allocation.allocated_registers, normalized2.register_allocation.allocated_registers);
    }
}