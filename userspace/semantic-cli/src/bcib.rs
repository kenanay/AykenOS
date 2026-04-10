//! BCIB (Binary/Bounded/Capability-aware Instruction Block) Definitions
//!
//! This module defines the BCIB instruction set for Phase 3.5.1.
//! BCIB represents "WHAT to do" not "HOW to do it" - it's a semantic execution contract.
//!
//! # Design Principles
//!
//! 1. **ATOMIC**: Each instruction carries a single intent
//! 2. **SIDE-EFFECT FREE**: Instructions don't modify state (Phase 3.5.1)
//! 3. **SERIALIZABLE**: All instructions can be serialized/deserialized
//! 4. **CAPABILITY AWARE**: Security starts at BCIB level, not executor
//! 5. **CATEGORIZED**: Instructions grouped by domain for clarity
//! 6. **INSTRUCTION GRAPH**: Flat instruction list using OperandRef (AR-1)
//!
//! # Phase 3.5.1 Scope
//!
//! **INCLUDED:**
//! - Context operations (load, access)
//! - Query operations (filter, compare, logical) with OperandRef model
//! - System operations (status, agents)
//! - Debug operations (explain, dry-run) with sequence references
//!
//! **NOT INCLUDED (Phase 3.5.2+):**
//! - Arithmetic operations
//! - Mutation operations
//! - Pipeline operations
//! - Agent orchestration
//! - Parallel execution
//!
//! # Architectural Requirements (AR-1 to AR-4)
//!
//! - **AR-1**: OperandRef model instead of embedded expressions
//! - **AR-2**: Filter normalization flags
//! - **AR-3**: Debug instruction sequence references
//! - **AR-4**: Contextual capabilities

use crate::error::{ErrorCode, Result, SemanticCLIError};
use crate::types::SourceLocation;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Main BCIB instruction enum - categorized by domain
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BCIBInstruction {
    /// Context-related operations
    Context(ContextInstruction),
    /// Query-related operations  
    Query(QueryInstruction),
    /// System-related operations
    System(SystemInstruction),
    /// Debug-related operations
    Debug(DebugInstruction),
    /// Loop-related operations (D3 Loop Support)
    Loop(LoopInstruction),
    /// Control flow operations (D3 Loop Support - Phase 2.3)
    ControlFlow(ControlFlowInstruction),
}

impl BCIBInstruction {
    /// Validate this instruction for correctness
    pub fn validate(&self) -> Result<()> {
        match self {
            Self::Context(inst) => inst.validate(),
            Self::Query(inst) => inst.validate(),
            Self::System(inst) => inst.validate(),
            Self::Debug(inst) => inst.validate(),
            Self::Loop(inst) => inst.validate(),
            Self::ControlFlow(inst) => inst.validate(),
        }
    }

    /// Check if this instruction is allowed in Phase 3.5.1
    pub fn is_phase_compatible(&self) -> bool {
        match self {
            Self::Context(_) => true,
            Self::Query(_) => true,
            Self::System(_) => true,
            Self::Debug(_) => true,
            Self::Loop(_) => true, // D3 Loop Support is part of Phase 3.5
            Self::ControlFlow(_) => true, // D3 Break/Continue is part of Phase 3.5
        }
    }

    /// Get the required capability for this instruction
    pub fn required_capability(&self) -> Option<Capability> {
        match self {
            Self::Context(inst) => inst.required_capability(),
            Self::Query(inst) => inst.required_capability(),
            Self::System(inst) => inst.required_capability(),
            Self::Debug(inst) => inst.required_capability(),
            Self::Loop(inst) => inst.required_capability(),
            Self::ControlFlow(inst) => inst.required_capability(),
        }
    }
}

/// Context-related instructions
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ContextInstruction {
    /// Load a context for subsequent operations
    LoadContext {
        /// Context path (e.g., "data.users")
        path: String,
        /// Source location for error reporting
        location: SourceLocation,
    },
    /// Return the current result
    Return {
        /// Source location for error reporting
        location: SourceLocation,
    },
}

impl ContextInstruction {
    /// Validate this context instruction
    pub fn validate(&self) -> Result<()> {
        match self {
            Self::LoadContext { path, .. } => {
                if path.is_empty() {
                    return Err(SemanticCLIError::validation_error(
                        "Context path cannot be empty",
                        "Provide a valid context path like 'data.users'",
                        ErrorCode::E300,
                    ));
                }

                // Validate path format (should contain at least one dot)
                if !path.contains('.') {
                    return Err(SemanticCLIError::validation_error(
                        format!("Invalid context path format: '{}'", path),
                        "Context path should be in format 'domain.resource' (e.g., 'data.users')",
                        ErrorCode::E300,
                    ));
                }

                Ok(())
            }
            Self::Return { .. } => Ok(()),
        }
    }

    /// Get the required capability for this instruction (AR-4: Contextual)
    pub fn required_capability(&self) -> Option<Capability> {
        match self {
            Self::LoadContext { path, .. } => {
                // Generate contextual capability based on path
                Some(Capability::Read {
                    context: path.clone(),
                })
            }
            Self::Return { .. } => None,
        }
    }
}

/// OperandRef - Reference to operands in instruction graph (AR-1)
///
/// This replaces embedded Value types to create a flat instruction graph
/// instead of expression trees, enabling optimizer insertion between
/// transformer and executor.
#[derive(Debug, Clone, PartialEq, Hash, Serialize, Deserialize)]
pub enum OperandRef {
    /// Reference to field in current context
    Field(String),
    /// Literal value (string, number, bool)
    Literal(Value),
    /// Temporary register for intermediate results
    TempRegister(u16),
}

impl OperandRef {
    /// Validate this operand reference
    pub fn validate(&self) -> Result<()> {
        match self {
            Self::Field(field) => {
                if field.is_empty() {
                    return Err(SemanticCLIError::validation_error(
                        "Field reference cannot be empty",
                        "Provide a valid field name",
                        ErrorCode::E300,
                    ));
                }
                Ok(())
            }
            Self::Literal(value) => value.validate(),
            Self::TempRegister(_) => {
                // Temp registers are always valid
                Ok(())
            }
        }
    }

    /// Get the type of this operand reference
    pub fn operand_type(&self) -> OperandType {
        match self {
            Self::Field(_) => OperandType::Field,
            Self::Literal(value) => OperandType::Literal(value.value_type()),
            Self::TempRegister(_) => OperandType::TempRegister,
        }
    }
}

/// Operand types for type checking
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OperandType {
    Field,
    Literal(ValueType),
    TempRegister,
}
/// Query-related instructions (AR-1: REFACTORED to use OperandRef with flat instruction graph)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum QueryInstruction {
    /// Load field value into temporary register (AR-1: Register generation)
    LoadField {
        /// Field name to load
        field: String,
        /// Target temporary register
        target_register: u16,
        /// Source location for error reporting
        location: SourceLocation,
    },
    /// Load literal value into temporary register (AR-1: Register generation)
    LoadLiteral {
        /// Literal value to load
        value: Value,
        /// Target temporary register
        target_register: u16,
        /// Source location for error reporting
        location: SourceLocation,
    },
    /// Apply a filter to the current context
    ApplyFilter {
        /// Normalized filter expression
        expression: FilterExpression,
        /// Source location for error reporting
        location: SourceLocation,
    },
    /// Apply filter using boolean register result (AR-1: Register-based filtering)
    ApplyFilterBool {
        /// Register containing boolean filter result
        filter_register: u16,
        /// Source location for error reporting
        location: SourceLocation,
    },
    /// Compare two operands and store result in register (AR-1: REFACTORED to use OperandRef)
    Compare {
        /// Left operand reference (CHANGED: was Value)
        left: OperandRef,
        /// Comparison operator
        operator: ComparisonOp,
        /// Right operand reference (CHANGED: was Value)
        right: OperandRef,
        /// Target register for boolean result
        target_register: u16,
        /// Source location for error reporting
        location: SourceLocation,
    },
    /// Apply logical operation and store result in register (AR-1: REFACTORED to use OperandRef)
    LogicalOp {
        /// Logical operator
        operator: LogicalOperator,
        /// Operand references (CHANGED: was Vec<Value>)
        operands: Vec<OperandRef>,
        /// Target register for boolean result
        target_register: u16,
        /// Source location for error reporting
        location: SourceLocation,
    },
}

impl QueryInstruction {
    /// Validate this query instruction
    pub fn validate(&self) -> Result<()> {
        match self {
            Self::LoadField { field, .. } => {
                if field.is_empty() {
                    return Err(SemanticCLIError::validation_error(
                        "Field name cannot be empty",
                        "Provide a valid field name",
                        ErrorCode::E300,
                    ));
                }
                Ok(())
            }
            Self::LoadLiteral { value, .. } => value.validate(),
            Self::ApplyFilter { expression, .. } => expression.validate(),
            Self::ApplyFilterBool { .. } => {
                // Register-based filter is always valid structurally
                Ok(())
            }
            Self::Compare { left, right, .. } => {
                left.validate()?;
                right.validate()
            }
            Self::LogicalOp {
                operator, operands, ..
            } => {
                match operator {
                    LogicalOperator::Not => {
                        if operands.len() != 1 {
                            return Err(SemanticCLIError::validation_error(
                                "NOT operator requires exactly 1 operand",
                                "Provide a single boolean operand",
                                ErrorCode::E300,
                            ));
                        }
                    }
                    LogicalOperator::And | LogicalOperator::Or => {
                        if operands.len() != 2 {
                            return Err(SemanticCLIError::validation_error(
                                format!("{:?} operator requires exactly 2 operands", operator),
                                "Provide two boolean operands",
                                ErrorCode::E300,
                            ));
                        }
                    }
                }

                for operand in operands {
                    operand.validate()?;
                }

                Ok(())
            }
        }
    }

    /// Get the required capability for this instruction (AR-4: Context-dependent)
    pub fn required_capability(&self) -> Option<Capability> {
        // Query operations don't have inherent capability requirements
        // Capability is determined by the context being queried (LoadContext)
        None
    }
}

/// System-related instructions
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SystemInstruction {
    /// Get system status
    SystemStatus {
        /// Source location for error reporting
        location: SourceLocation,
    },
    /// List active agents
    ListAgents {
        /// Source location for error reporting
        location: SourceLocation,
    },
}

impl SystemInstruction {
    /// Validate this system instruction
    pub fn validate(&self) -> Result<()> {
        // System instructions are always valid in Phase 3.5.1
        Ok(())
    }

    /// Get the required capability for this instruction (AR-4: Contextual)
    pub fn required_capability(&self) -> Option<Capability> {
        match self {
            Self::SystemStatus { .. } => Some(Capability::System {
                scope: SystemScope::Status,
            }),
            Self::ListAgents { .. } => Some(Capability::System {
                scope: SystemScope::Agents,
            }),
        }
    }
}

/// Debug-related instructions (AR-3: REFACTORED to use sequence references)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DebugInstruction {
    /// Explain what a sequence of instructions would do (AR-3: REFACTORED)
    Explain {
        /// Target sequence ID reference (CHANGED: was Vec<BCIBInstruction>)
        target_sequence_id: String,
        /// Source location for error reporting
        location: SourceLocation,
    },
    /// Dry-run a sequence of instructions (AR-3: REFACTORED)
    DryRun {
        /// Target sequence ID reference (CHANGED: was Vec<BCIBInstruction>)
        target_sequence_id: String,
        /// Source location for error reporting
        location: SourceLocation,
    },
    /// Show command history
    History {
        /// Source location for error reporting
        location: SourceLocation,
    },
}

impl DebugInstruction {
    /// Validate this debug instruction
    pub fn validate(&self) -> Result<()> {
        match self {
            Self::Explain {
                target_sequence_id, ..
            }
            | Self::DryRun {
                target_sequence_id, ..
            } => {
                if target_sequence_id.is_empty() {
                    return Err(SemanticCLIError::validation_error(
                        "Debug instruction requires a valid sequence ID",
                        "Provide a sequence ID to explain or dry-run",
                        ErrorCode::E300,
                    ));
                }

                Ok(())
            }
            Self::History { .. } => Ok(()),
        }
    }

    /// Get the required capability for this instruction
    pub fn required_capability(&self) -> Option<Capability> {
        // Debug operations require debug capability
        Some(Capability::Debug)
    }
}

/// Loop-related instructions (D3 Loop Support - Phase 1)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LoopInstruction {
    /// While loop with condition evaluation
    While {
        /// Unique loop identifier
        id: LoopID,
        /// Loop condition expression
        condition: OperandRef,
        /// Loop body as IR block reference
        body: String, // IR block reference for now
        /// Loop configuration
        config: LoopConfig,
        /// Source location for error reporting
        location: SourceLocation,
    },
    /// For loop with range iteration
    For {
        /// Unique loop identifier
        id: LoopID,
        /// Range specification
        range: LoopRange,
        /// Iterator variable name
        iterator_var: String,
        /// Loop body as IR block reference
        body: String, // IR block reference for now
        /// Loop configuration
        config: LoopConfig,
        /// Source location for error reporting
        location: SourceLocation,
    },
    /// ForEach loop over collections
    ForEach {
        /// Unique loop identifier
        id: LoopID,
        /// Collection to iterate over
        collection: OperandRef,
        /// Collection type for determinism validation
        collection_type: CollectionType,
        /// Iterator variable name
        iterator_var: String,
        /// Loop body as IR block reference
        body: String, // IR block reference for now
        /// Loop configuration
        config: LoopConfig,
        /// Source location for error reporting
        location: SourceLocation,
    },
}

impl LoopInstruction {
    /// Validate this loop instruction
    pub fn validate(&self) -> Result<()> {
        match self {
            Self::While {
                id,
                condition,
                config,
                ..
            } => {
                id.validate()?;
                condition.validate()?;
                config.validate()
            }
            Self::For {
                id, range, config, ..
            } => {
                id.validate()?;
                range.validate()?;
                config.validate()
            }
            Self::ForEach {
                id,
                collection,
                collection_type,
                config,
                ..
            } => {
                id.validate()?;
                collection.validate()?;
                collection_type.validate()?;
                config.validate()
            }
        }
    }

    /// Get the required capability for this instruction
    pub fn required_capability(&self) -> Option<Capability> {
        // Loop operations require execution capability
        Some(Capability::Execute)
    }

    /// Get the loop type for parallelization decisions
    pub fn loop_type(&self) -> LoopType {
        match self {
            Self::While { .. } => LoopType::While,
            Self::For { .. } => LoopType::For,
            Self::ForEach { .. } => LoopType::ForEach,
        }
    }
}

/// Control flow instructions (D3 Loop Support - Phase 2.3)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ControlFlowInstruction {
    /// Break statement - early loop termination
    Break {
        /// Source location for error reporting
        location: SourceLocation,
    },
    /// Continue statement - skip remaining iteration body
    Continue {
        /// Source location for error reporting
        location: SourceLocation,
    },
}

impl ControlFlowInstruction {
    /// Validate this control flow instruction
    pub fn validate(&self) -> Result<()> {
        // Break and Continue instructions are always valid
        // Context validation (must be within loop) is handled by the transformer
        Ok(())
    }

    /// Get the required capability for this instruction
    pub fn required_capability(&self) -> Option<Capability> {
        // Control flow operations require execution capability
        Some(Capability::Execute)
    }

    /// Get the control flow type
    pub fn control_flow_type(&self) -> ControlFlowType {
        match self {
            Self::Break { .. } => ControlFlowType::Break,
            Self::Continue { .. } => ControlFlowType::Continue,
        }
    }
}

/// Control flow types for execution decisions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ControlFlowType {
    /// Normal execution - continue to next iteration
    Normal,
    /// Break - early termination
    Break,
    /// Continue - skip remaining body
    Continue,
}

/// Loop identifier type
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LoopID(pub String);

impl LoopID {
    /// Create a new loop ID
    pub fn new(id: String) -> Self {
        Self(id)
    }

    /// Generate a unique loop ID
    pub fn generate() -> Self {
        Self(uuid::Uuid::new_v4().to_string())
    }

    /// Validate this loop ID
    pub fn validate(&self) -> Result<()> {
        if self.0.is_empty() {
            return Err(SemanticCLIError::validation_error(
                "Loop ID cannot be empty",
                "Provide a valid loop identifier",
                ErrorCode::E300,
            ));
        }
        Ok(())
    }
}

/// Loop configuration (Constitutional Alignment - Phase 0.5)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LoopConfig {
    /// Maximum iterations allowed (Constitutional: never exceeded)
    pub iteration_limit: u32,
    /// Budget timeout in logical units (Constitutional: deterministic)
    pub budget_timeout: u64,
    /// Budget measurement method
    pub budget_measurement: BudgetMeasurement,
    /// Initial accumulator value
    pub initial_accumulator: Value,
    /// Expected accumulator type
    pub accumulator_type: ValueType,
    /// Error recovery policy (Constitutional: explicit only)
    pub error_recovery: ErrorRecoveryPolicy,
}

impl LoopConfig {
    /// Create a new loop configuration with defaults
    pub fn new(initial_accumulator: Value, accumulator_type: ValueType) -> Self {
        Self {
            iteration_limit: 10_000,   // Constitutional default
            budget_timeout: 1_000_000, // Default budget
            budget_measurement: BudgetMeasurement::IterationCount,
            initial_accumulator,
            accumulator_type,
            error_recovery: ErrorRecoveryPolicy::Abort, // Constitutional default
        }
    }

    /// Validate this loop configuration
    pub fn validate(&self) -> Result<()> {
        if self.iteration_limit == 0 {
            return Err(SemanticCLIError::validation_error(
                "Iteration limit must be greater than 0",
                "Provide a positive iteration limit",
                ErrorCode::E300,
            ));
        }

        if self.iteration_limit > 10_000 {
            return Err(SemanticCLIError::validation_error(
                "Iteration limit exceeds constitutional maximum of 10,000",
                "Use a limit within constitutional bounds",
                ErrorCode::E300,
            ));
        }

        if self.budget_timeout == 0 {
            return Err(SemanticCLIError::validation_error(
                "Budget timeout must be greater than 0",
                "Provide a positive budget timeout",
                ErrorCode::E300,
            ));
        }

        self.initial_accumulator.validate()?;
        self.error_recovery.validate()
    }
}

/// Loop range for For loops
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LoopRange {
    /// Start value (inclusive)
    pub start: i64,
    /// End value (exclusive)
    pub end: i64,
    /// Step value (must be non-zero)
    pub step: i64,
}

impl LoopRange {
    /// Create a new loop range
    pub fn new(start: i64, end: i64, step: i64) -> Self {
        Self { start, end, step }
    }

    /// Validate this loop range
    pub fn validate(&self) -> Result<()> {
        if self.step == 0 {
            return Err(SemanticCLIError::validation_error(
                "Loop range step cannot be zero",
                "Provide a non-zero step value",
                ErrorCode::E300,
            ));
        }

        // Check for infinite loops
        if (self.step > 0 && self.start >= self.end) || (self.step < 0 && self.start <= self.end) {
            return Err(SemanticCLIError::validation_error(
                "Loop range would result in zero iterations",
                "Ensure range direction matches step sign",
                ErrorCode::E300,
            ));
        }

        Ok(())
    }

    /// Calculate the number of iterations this range will produce
    pub fn iteration_count(&self) -> u32 {
        if (self.step > 0 && self.start >= self.end) || (self.step < 0 && self.start <= self.end) {
            return 0;
        }

        let diff = (self.end - self.start).abs();
        let step_abs = self.step.abs();
        ((diff + step_abs - 1) / step_abs) as u32 // Ceiling division
    }
}

/// Collection types for ForEach loops (Constitutional Alignment)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CollectionType {
    /// Array with index-based iteration (0, 1, 2, ...)
    Array,
    /// List with insertion-order iteration
    List,
    /// Sorted map with key-order iteration
    SortedMap,
    /// Hash map - REJECTED unless canonical ordering provided
    HashMap { canonical_ordering: Option<String> },
    /// Hash set - REJECTED unless canonical ordering provided
    HashSet { canonical_ordering: Option<String> },
}

impl CollectionType {
    /// Validate this collection type for deterministic iteration
    pub fn validate(&self) -> Result<()> {
        match self {
            Self::Array | Self::List | Self::SortedMap => Ok(()),
            Self::HashMap { canonical_ordering } | Self::HashSet { canonical_ordering } => {
                if canonical_ordering.is_none() {
                    return Err(SemanticCLIError::validation_error(
                        "Unordered collections require canonical ordering for deterministic iteration",
                        "Provide explicit ordering or use ordered collection types",
                        ErrorCode::E300,
                    ));
                }
                Ok(())
            }
        }
    }
}

/// Loop types for parallelization decisions (Constitutional Alignment)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LoopType {
    /// While loop - NEVER parallelized (Constitutional rule)
    While,
    /// For loop - Can be parallelized if safe
    For,
    /// ForEach loop - Can be parallelized if safe
    ForEach,
}

/// Budget measurement methods (Constitutional Alignment)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BudgetMeasurement {
    /// Count iterations (simple, fast)
    IterationCount,
    /// Count instructions (fine-grained, requires instrumentation)
    InstructionCount { weight: u64 },
    /// Hybrid approach (iteration count × average instruction count)
    Hybrid { multiplier: f64 },
}

/// Error recovery policies (Constitutional Alignment - Phase 0.5)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ErrorRecoveryPolicy {
    /// Abort on error (Constitutional default)
    Abort,
    /// Retry with increased limit (Constitutional: explicit only)
    RetryWithIncreasedLimit {
        new_limit: u32,   // Must not exceed global max (10,000)
        max_retries: u32, // Must be bounded (default: 1, max: 3)
    },
    /// Return partial results (Constitutional: explicit only)
    ReturnPartialResults { include_error_info: bool },
}

impl ErrorRecoveryPolicy {
    /// Validate this error recovery policy
    pub fn validate(&self) -> Result<()> {
        match self {
            Self::Abort => Ok(()),
            Self::RetryWithIncreasedLimit {
                new_limit,
                max_retries,
            } => {
                if *new_limit > 10_000 {
                    return Err(SemanticCLIError::validation_error(
                        "Retry limit exceeds constitutional maximum of 10,000",
                        "Use a limit within constitutional bounds",
                        ErrorCode::E300,
                    ));
                }
                if *max_retries > 3 {
                    return Err(SemanticCLIError::validation_error(
                        "Max retries exceeds constitutional maximum of 3",
                        "Use bounded retry count",
                        ErrorCode::E300,
                    ));
                }
                Ok(())
            }
            Self::ReturnPartialResults { .. } => Ok(()),
        }
    }
}

/// Normalized filter expression (simplified from AST) (AR-2: Added normalization flag)
#[derive(Debug, Clone, PartialEq, Hash, Serialize, Deserialize)]
pub struct FilterExpression {
    /// Field name being filtered
    pub field: String,
    /// Comparison operator
    pub operator: ComparisonOp,
    /// Value to compare against (AR-2: Changed to OperandRef for consistency)
    pub value: OperandRef,
    /// Whether this filter has been normalized by transformer (AR-2)
    pub normalized: bool,
}

impl FilterExpression {
    /// Create a new filter expression
    pub fn new(field: String, operator: ComparisonOp, value: OperandRef) -> Self {
        Self {
            field,
            operator,
            value,
            normalized: false, // Default to not normalized
        }
    }

    /// Create a new normalized filter expression
    pub fn new_normalized(field: String, operator: ComparisonOp, value: OperandRef) -> Self {
        Self {
            field,
            operator,
            value,
            normalized: true,
        }
    }

    /// Validate this filter expression
    pub fn validate(&self) -> Result<()> {
        if self.field.is_empty() {
            return Err(SemanticCLIError::validation_error(
                "Filter field cannot be empty",
                "Provide a valid field name",
                ErrorCode::E300,
            ));
        }

        self.value.validate()
    }
}

/// Comparison operators
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ComparisonOp {
    Equal,
    NotEqual,
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,
}

/// Logical operators
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LogicalOperator {
    And,
    Or,
    Not,
}

/// Values that can be used in BCIB instructions (AR-1: Literals only, no field references)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Value {
    /// String literal
    String(String),
    /// Number literal
    Number(f64),
    /// Boolean literal
    Boolean(bool),
    /// Array with deterministic index-based iteration (0, 1, 2, ...)
    Array(Vec<Value>),
    /// List with deterministic insertion-order iteration
    List(Vec<Value>),
    /// Sorted map with deterministic key-order iteration
    SortedMap(std::collections::BTreeMap<String, Value>),
}

impl std::hash::Hash for Value {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            Self::String(s) => {
                0u8.hash(state);
                s.hash(state);
            }
            Self::Number(n) => {
                1u8.hash(state);
                // Convert f64 to bits for consistent hashing
                n.to_bits().hash(state);
            }
            Self::Boolean(b) => {
                2u8.hash(state);
                b.hash(state);
            }
            Self::Array(arr) => {
                3u8.hash(state);
                arr.len().hash(state);
                for item in arr {
                    item.hash(state);
                }
            }
            Self::List(list) => {
                4u8.hash(state);
                list.len().hash(state);
                for item in list {
                    item.hash(state);
                }
            }
            Self::SortedMap(map) => {
                5u8.hash(state);
                map.len().hash(state);
                // BTreeMap iteration is deterministic by key order
                for (key, value) in map {
                    key.hash(state);
                    value.hash(state);
                }
            }
        }
    }
}

impl Value {
    /// Validate this value
    pub fn validate(&self) -> Result<()> {
        match self {
            Self::String(_s) => {
                // String values are always valid (can be empty)
                Ok(())
            }
            Self::Number(n) => {
                if n.is_nan() || n.is_infinite() {
                    return Err(SemanticCLIError::validation_error(
                        "Number value cannot be NaN or infinite",
                        "Use a finite number value",
                        ErrorCode::E300,
                    ));
                }
                Ok(())
            }
            Self::Boolean(_) => {
                // Boolean values are always valid
                Ok(())
            }
            Self::Array(arr) => {
                // Validate all array elements
                for (index, item) in arr.iter().enumerate() {
                    item.validate().map_err(|e| {
                        SemanticCLIError::validation_error(
                            format!("Array element at index {} is invalid: {}", index, e),
                            "Ensure all array elements are valid values",
                            ErrorCode::E300,
                        )
                    })?;
                }
                Ok(())
            }
            Self::List(list) => {
                // Validate all list elements
                for (index, item) in list.iter().enumerate() {
                    item.validate().map_err(|e| {
                        SemanticCLIError::validation_error(
                            format!("List element at index {} is invalid: {}", index, e),
                            "Ensure all list elements are valid values",
                            ErrorCode::E300,
                        )
                    })?;
                }
                Ok(())
            }
            Self::SortedMap(map) => {
                // Validate all map values
                for (key, value) in map {
                    value.validate().map_err(|e| {
                        SemanticCLIError::validation_error(
                            format!("SortedMap value for key '{}' is invalid: {}", key, e),
                            "Ensure all map values are valid",
                            ErrorCode::E300,
                        )
                    })?;
                }
                Ok(())
            }
        }
    }

    /// Get the type of this value
    pub fn value_type(&self) -> ValueType {
        match self {
            Self::String(_) => ValueType::String,
            Self::Number(_) => ValueType::Number,
            Self::Boolean(_) => ValueType::Boolean,
            Self::Array(_) => ValueType::Array,
            Self::List(_) => ValueType::List,
            Self::SortedMap(_) => ValueType::SortedMap,
        }
    }

    /// Get the collection type for this value (if it's a collection)
    pub fn collection_type(&self) -> Option<CollectionType> {
        match self {
            Self::Array(_) => Some(CollectionType::Array),
            Self::List(_) => Some(CollectionType::List),
            Self::SortedMap(_) => Some(CollectionType::SortedMap),
            _ => None,
        }
    }

    /// Get the size of this collection (if it's a collection)
    pub fn collection_size(&self) -> Option<usize> {
        match self {
            Self::Array(arr) => Some(arr.len()),
            Self::List(list) => Some(list.len()),
            Self::SortedMap(map) => Some(map.len()),
            _ => None,
        }
    }

    /// Check if this value is a collection
    pub fn is_collection(&self) -> bool {
        matches!(self, Self::Array(_) | Self::List(_) | Self::SortedMap(_))
    }

    /// Create an iterator over collection elements in deterministic order
    /// Returns None if this value is not a collection
    pub fn iter_collection(&self) -> Option<CollectionIterator<'_>> {
        match self {
            Self::Array(arr) => Some(CollectionIterator::Array {
                items: arr,
                index: 0,
            }),
            Self::List(list) => Some(CollectionIterator::List {
                items: list,
                index: 0,
            }),
            Self::SortedMap(map) => {
                let items: Vec<_> = map.iter().collect();
                Some(CollectionIterator::SortedMap { items, index: 0 })
            }
            _ => None,
        }
    }
}

/// Deterministic collection iterator for ForEach loops
/// Ensures consistent iteration order across all collection types
#[derive(Debug)]
pub enum CollectionIterator<'a> {
    /// Array iterator - index order (0, 1, 2, ...)
    Array { items: &'a Vec<Value>, index: usize },
    /// List iterator - insertion order
    List { items: &'a Vec<Value>, index: usize },
    /// SortedMap iterator - key sort order
    SortedMap {
        items: Vec<(&'a String, &'a Value)>,
        index: usize,
    },
}

impl<'a> Iterator for CollectionIterator<'a> {
    type Item = CollectionElement<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            CollectionIterator::Array { items, index } => {
                if *index < items.len() {
                    let item = &items[*index];
                    let element = CollectionElement::ArrayElement {
                        index: *index,
                        value: item,
                    };
                    *index += 1;
                    Some(element)
                } else {
                    None
                }
            }
            CollectionIterator::List { items, index } => {
                if *index < items.len() {
                    let item = &items[*index];
                    let element = CollectionElement::ListElement {
                        index: *index,
                        value: item,
                    };
                    *index += 1;
                    Some(element)
                } else {
                    None
                }
            }
            CollectionIterator::SortedMap { items, index } => {
                if *index < items.len() {
                    let (key, value) = items[*index];
                    let element = CollectionElement::MapElement { key, value };
                    *index += 1;
                    Some(element)
                } else {
                    None
                }
            }
        }
    }
}

/// Collection element returned by deterministic iteration
#[derive(Debug, Clone, PartialEq)]
pub enum CollectionElement<'a> {
    /// Array element with index
    ArrayElement { index: usize, value: &'a Value },
    /// List element with insertion order index
    ListElement { index: usize, value: &'a Value },
    /// Map element with key-value pair
    MapElement { key: &'a String, value: &'a Value },
}

impl<'a> CollectionElement<'a> {
    /// Get the value from this collection element
    pub fn value(&self) -> &'a Value {
        match self {
            CollectionElement::ArrayElement { value, .. } => value,
            CollectionElement::ListElement { value, .. } => value,
            CollectionElement::MapElement { value, .. } => value,
        }
    }

    /// Get the key from this collection element (for maps)
    pub fn key(&self) -> Option<&'a String> {
        match self {
            CollectionElement::MapElement { key, .. } => Some(key),
            _ => None,
        }
    }

    /// Get the index from this collection element (for arrays and lists)
    pub fn index(&self) -> Option<usize> {
        match self {
            CollectionElement::ArrayElement { index, .. } => Some(*index),
            CollectionElement::ListElement { index, .. } => Some(*index),
            _ => None,
        }
    }
}

/// Value types for type checking (AR-1: Field removed, only literals)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ValueType {
    String,
    Number,
    Boolean,
    Array,
    List,
    SortedMap,
}

/// Capability types for security (AR-4: Contextual capabilities)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Capability {
    /// Read access to specific contexts
    Read { context: String },
    /// System operations access with scope
    System { scope: SystemScope },
    /// Debug operations access
    Debug,
    /// Loop execution access (D3 Loop Support)
    Execute,
}

/// System operation scopes for fine-grained access control (AR-4)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SystemScope {
    /// System status information
    Status,
    /// Agent management
    Agents,
    /// Full system access (for future use)
    Full,
}

/// BCIB instruction sequence - represents a complete command
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BCIBSequence {
    /// Sequence of instructions
    pub instructions: Vec<BCIBInstruction>,
    /// Metadata about this sequence
    pub metadata: BCIBMetadata,
}

impl BCIBSequence {
    /// Create a new BCIB sequence
    pub fn new(instructions: Vec<BCIBInstruction>) -> Self {
        Self {
            instructions,
            metadata: BCIBMetadata::new(),
        }
    }

    /// Validate the entire sequence
    pub fn validate(&self) -> Result<()> {
        if self.instructions.is_empty() {
            return Err(SemanticCLIError::validation_error(
                "BCIB sequence cannot be empty",
                "Provide at least one instruction",
                ErrorCode::E300,
            ));
        }

        // Validate each instruction
        for instruction in &self.instructions {
            instruction.validate()?;

            // Check phase compatibility
            if !instruction.is_phase_compatible() {
                return Err(SemanticCLIError::validation_error(
                    "Instruction not compatible with Phase 3.5.1",
                    "Use only Core DSL instructions in this phase",
                    ErrorCode::E300,
                ));
            }
        }

        Ok(())
    }

    /// Get all required capabilities for this sequence
    pub fn required_capabilities(&self) -> Vec<Capability> {
        let mut capabilities = Vec::new();

        for instruction in &self.instructions {
            if let Some(cap) = instruction.required_capability() {
                if !capabilities.contains(&cap) {
                    capabilities.push(cap);
                }
            }
        }

        capabilities
    }

    /// Serialize to JSON
    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string_pretty(self)
            .map_err(|e| SemanticCLIError::serialization_error(e.to_string(), ErrorCode::E301))
    }

    /// Deserialize from JSON
    pub fn from_json(json: &str) -> Result<Self> {
        serde_json::from_str(json)
            .map_err(|e| SemanticCLIError::serialization_error(e.to_string(), ErrorCode::E301))
    }

    /// Serialize to binary (using bincode)
    pub fn to_binary(&self) -> Result<Vec<u8>> {
        bincode::serialize(self)
            .map_err(|e| SemanticCLIError::serialization_error(e.to_string(), ErrorCode::E301))
    }

    /// Deserialize from binary
    pub fn from_binary(data: &[u8]) -> Result<Self> {
        bincode::deserialize(data)
            .map_err(|e| SemanticCLIError::serialization_error(e.to_string(), ErrorCode::E301))
    }
}

/// Metadata for BCIB sequences
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BCIBMetadata {
    /// Unique sequence ID
    pub sequence_id: String,
    /// Creation timestamp
    pub created_at: u64,
    /// Phase compatibility
    pub phase: String,
    /// Determinism level
    pub determinism: DeterminismLevel,
}

impl BCIBMetadata {
    /// Create new metadata
    pub fn new() -> Self {
        Self {
            sequence_id: uuid::Uuid::new_v4().to_string(),
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            phase: "3.5.1".to_string(),
            determinism: DeterminismLevel::Deterministic,
        }
    }
}

impl Default for BCIBMetadata {
    fn default() -> Self {
        Self::new()
    }
}

/// BCIB Sequence Registry for AR-3 (Debug instruction sequence references)
///
/// In-memory, append-only registry that maps sequence IDs to BCIB sequences.
/// This enables debug instructions to reference sequences by ID instead of
/// carrying recursive BCIB structures.
#[derive(Debug, Clone)]
pub struct BCIBSequenceRegistry {
    /// Map from sequence ID to BCIB sequence
    sequences: HashMap<String, BCIBSequence>,
}

impl BCIBSequenceRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            sequences: HashMap::new(),
        }
    }

    /// Register a sequence and return its ID
    pub fn register(&mut self, sequence: BCIBSequence) -> String {
        let sequence_id = sequence.metadata.sequence_id.clone();
        self.sequences.insert(sequence_id.clone(), sequence);
        sequence_id
    }

    /// Get a sequence by ID
    pub fn get(&self, sequence_id: &str) -> Option<&BCIBSequence> {
        self.sequences.get(sequence_id)
    }

    /// Check if a sequence exists
    pub fn contains(&self, sequence_id: &str) -> bool {
        self.sequences.contains_key(sequence_id)
    }

    /// Get all sequence IDs (for history/audit)
    pub fn sequence_ids(&self) -> Vec<String> {
        self.sequences.keys().cloned().collect()
    }

    /// Get the number of registered sequences
    pub fn len(&self) -> usize {
        self.sequences.len()
    }

    /// Check if the registry is empty
    pub fn is_empty(&self) -> bool {
        self.sequences.is_empty()
    }
}

impl Default for BCIBSequenceRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Determinism level for replay capability
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeterminismLevel {
    /// Same input → same output (required for replay)
    Deterministic,
    /// May vary (e.g., network latency, system load)
    BestEffort,
    /// Explicitly non-deterministic (e.g., random, time-based)
    NonDeterministic,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::SourceLocation;

    fn test_location() -> SourceLocation {
        SourceLocation::new(1, 1, 0)
    }

    #[test]
    fn test_context_instruction_validation() {
        // Valid context instruction (AR-4: No embedded capability)
        let valid = ContextInstruction::LoadContext {
            path: "data.users".to_string(),
            location: test_location(),
        };
        assert!(valid.validate().is_ok());

        // Check contextual capability generation (AR-4)
        assert_eq!(
            valid.required_capability(),
            Some(Capability::Read {
                context: "data.users".to_string()
            })
        );

        // Invalid - empty path
        let invalid_empty = ContextInstruction::LoadContext {
            path: "".to_string(),
            location: test_location(),
        };
        assert!(invalid_empty.validate().is_err());

        // Invalid - no dot in path
        let invalid_format = ContextInstruction::LoadContext {
            path: "users".to_string(),
            location: test_location(),
        };
        assert!(invalid_format.validate().is_err());
    }

    #[test]
    fn test_query_instruction_validation() {
        // Valid filter (AR-2: Updated with OperandRef and normalized flag)
        let valid_filter = QueryInstruction::ApplyFilter {
            expression: FilterExpression::new(
                "age".to_string(),
                ComparisonOp::GreaterThan,
                OperandRef::Literal(Value::Number(18.0)),
            ),
            location: test_location(),
        };
        assert!(valid_filter.validate().is_ok());

        // Valid comparison using OperandRef with target register (AR-1)
        let valid_compare = QueryInstruction::Compare {
            left: OperandRef::Field("age".to_string()),
            operator: ComparisonOp::GreaterThan,
            right: OperandRef::Literal(Value::Number(18.0)),
            target_register: 0,
            location: test_location(),
        };
        assert!(valid_compare.validate().is_ok());

        // Valid logical operation using OperandRef with target register (AR-1)
        let valid_logical = QueryInstruction::LogicalOp {
            operator: LogicalOperator::And,
            operands: vec![
                OperandRef::Literal(Value::Boolean(true)),
                OperandRef::Literal(Value::Boolean(false)),
            ],
            target_register: 1,
            location: test_location(),
        };
        assert!(valid_logical.validate().is_ok());

        // Valid register generation instructions (AR-1)
        let valid_load_field = QueryInstruction::LoadField {
            field: "age".to_string(),
            target_register: 0,
            location: test_location(),
        };
        assert!(valid_load_field.validate().is_ok());

        let valid_load_literal = QueryInstruction::LoadLiteral {
            value: Value::Number(42.0),
            target_register: 1,
            location: test_location(),
        };
        assert!(valid_load_literal.validate().is_ok());

        // Invalid - NOT with wrong operand count
        let invalid_not = QueryInstruction::LogicalOp {
            operator: LogicalOperator::Not,
            operands: vec![
                OperandRef::Literal(Value::Boolean(true)),
                OperandRef::Literal(Value::Boolean(false)),
            ],
            target_register: 2,
            location: test_location(),
        };
        assert!(invalid_not.validate().is_err());
    }

    #[test]
    fn test_operand_ref_validation() {
        // Valid operand references (AR-1)
        assert!(OperandRef::Field("age".to_string()).validate().is_ok());
        assert!(OperandRef::Literal(Value::Number(42.0)).validate().is_ok());
        assert!(OperandRef::TempRegister(0).validate().is_ok());

        // Invalid operand references
        assert!(OperandRef::Field("".to_string()).validate().is_err());
        assert!(OperandRef::Literal(Value::Number(f64::NAN))
            .validate()
            .is_err());
    }

    #[test]
    fn test_operand_ref_types() {
        // Test operand type detection (AR-1)
        assert_eq!(
            OperandRef::Field("age".to_string()).operand_type(),
            OperandType::Field
        );
        assert_eq!(
            OperandRef::Literal(Value::Number(42.0)).operand_type(),
            OperandType::Literal(ValueType::Number)
        );
        assert_eq!(
            OperandRef::TempRegister(0).operand_type(),
            OperandType::TempRegister
        );
    }

    #[test]
    fn test_value_validation() {
        // Valid values (AR-1: Field removed from Value)
        assert!(Value::String("test".to_string()).validate().is_ok());
        assert!(Value::Number(42.0).validate().is_ok());
        assert!(Value::Boolean(true).validate().is_ok());

        // Invalid values
        assert!(Value::Number(f64::NAN).validate().is_err());
        assert!(Value::Number(f64::INFINITY).validate().is_err());
    }

    #[test]
    fn test_bcib_sequence_validation() {
        let instructions = vec![
            BCIBInstruction::Context(ContextInstruction::LoadContext {
                path: "data.users".to_string(),
                location: test_location(),
            }),
            BCIBInstruction::Context(ContextInstruction::Return {
                location: test_location(),
            }),
        ];

        let sequence = BCIBSequence::new(instructions);
        assert!(sequence.validate().is_ok());

        // Empty sequence should fail
        let empty_sequence = BCIBSequence::new(vec![]);
        assert!(empty_sequence.validate().is_err());
    }

    #[test]
    fn test_serialization_round_trip_json() {
        let instructions = vec![
            BCIBInstruction::Context(ContextInstruction::LoadContext {
                path: "data.users".to_string(),
                location: test_location(),
            }),
            BCIBInstruction::Query(QueryInstruction::ApplyFilter {
                expression: FilterExpression::new(
                    "age".to_string(),
                    ComparisonOp::GreaterThan,
                    OperandRef::Literal(Value::Number(18.0)),
                ),
                location: test_location(),
            }),
        ];

        let original = BCIBSequence::new(instructions);

        // Serialize to JSON
        let json = original.to_json().unwrap();
        assert!(!json.is_empty());

        // Deserialize from JSON
        let deserialized = BCIBSequence::from_json(&json).unwrap();

        // Should be identical
        assert_eq!(original.instructions, deserialized.instructions);
    }

    #[test]
    fn test_serialization_round_trip_binary() {
        let instructions = vec![BCIBInstruction::System(SystemInstruction::SystemStatus {
            location: test_location(),
        })];

        let original = BCIBSequence::new(instructions);

        // Serialize to binary
        let binary = original.to_binary().unwrap();
        assert!(!binary.is_empty());

        // Deserialize from binary
        let deserialized = BCIBSequence::from_binary(&binary).unwrap();

        // Should be identical
        assert_eq!(original.instructions, deserialized.instructions);
    }

    #[test]
    fn test_capability_requirements() {
        // Context instruction with contextual capability (AR-4)
        let context_inst = BCIBInstruction::Context(ContextInstruction::LoadContext {
            path: "data.users".to_string(),
            location: test_location(),
        });
        assert_eq!(
            context_inst.required_capability(),
            Some(Capability::Read {
                context: "data.users".to_string()
            })
        );

        // System instruction with scoped capability (AR-4)
        let system_inst = BCIBInstruction::System(SystemInstruction::SystemStatus {
            location: test_location(),
        });
        assert_eq!(
            system_inst.required_capability(),
            Some(Capability::System {
                scope: SystemScope::Status
            })
        );

        let debug_inst = BCIBInstruction::Debug(DebugInstruction::History {
            location: test_location(),
        });
        assert_eq!(debug_inst.required_capability(), Some(Capability::Debug));
    }

    #[test]
    fn test_phase_compatibility() {
        let valid_inst = BCIBInstruction::Context(ContextInstruction::LoadContext {
            path: "data.users".to_string(),
            location: test_location(),
        });
        assert!(valid_inst.is_phase_compatible());
    }

    #[test]
    fn test_filter_expression() {
        // AR-2: Updated with OperandRef and normalized flag
        let expr = FilterExpression::new(
            "age".to_string(),
            ComparisonOp::GreaterThan,
            OperandRef::Literal(Value::Number(18.0)),
        );

        assert!(expr.validate().is_ok());
        assert_eq!(expr.field, "age");
        assert_eq!(expr.operator, ComparisonOp::GreaterThan);
        assert_eq!(expr.value, OperandRef::Literal(Value::Number(18.0)));
        assert!(!expr.normalized); // Default to not normalized

        // Test normalized filter
        let normalized_expr = FilterExpression::new_normalized(
            "status".to_string(),
            ComparisonOp::Equal,
            OperandRef::Literal(Value::String("active".to_string())),
        );
        assert!(normalized_expr.normalized);

        // Invalid - empty field
        let invalid_expr = FilterExpression::new(
            "".to_string(),
            ComparisonOp::Equal,
            OperandRef::Literal(Value::String("test".to_string())),
        );
        assert!(invalid_expr.validate().is_err());
    }

    #[test]
    fn test_value_types() {
        // AR-1: Field removed from Value
        assert_eq!(
            Value::String("test".to_string()).value_type(),
            ValueType::String
        );
        assert_eq!(Value::Number(42.0).value_type(), ValueType::Number);
        assert_eq!(Value::Boolean(true).value_type(), ValueType::Boolean);
    }

    #[test]
    fn test_bcib_metadata() {
        let metadata = BCIBMetadata::new();

        assert!(!metadata.sequence_id.is_empty());
        assert!(metadata.created_at > 0);
        assert_eq!(metadata.phase, "3.5.1");
        assert_eq!(metadata.determinism, DeterminismLevel::Deterministic);
    }

    #[test]
    fn test_bcib_sequence_registry() {
        // Test AR-3: BCIBSequenceRegistry
        let mut registry = BCIBSequenceRegistry::new();
        assert!(registry.is_empty());

        // Create and register a sequence
        let instructions = vec![BCIBInstruction::Context(ContextInstruction::LoadContext {
            path: "data.users".to_string(),
            location: test_location(),
        })];
        let sequence = BCIBSequence::new(instructions);
        let sequence_id = sequence.metadata.sequence_id.clone();

        let registered_id = registry.register(sequence);
        assert_eq!(registered_id, sequence_id);
        assert_eq!(registry.len(), 1);
        assert!(!registry.is_empty());

        // Test retrieval
        assert!(registry.contains(&sequence_id));
        let retrieved = registry.get(&sequence_id);
        assert!(retrieved.is_some());

        // Test sequence IDs
        let ids = registry.sequence_ids();
        assert_eq!(ids.len(), 1);
        assert!(ids.contains(&sequence_id));
    }

    #[test]
    fn test_debug_instruction_sequence_references() {
        // Test AR-3: Debug instructions use sequence ID references
        let explain = DebugInstruction::Explain {
            target_sequence_id: "test-sequence-123".to_string(),
            location: test_location(),
        };
        assert!(explain.validate().is_ok());

        let dry_run = DebugInstruction::DryRun {
            target_sequence_id: "test-sequence-456".to_string(),
            location: test_location(),
        };
        assert!(dry_run.validate().is_ok());

        // Invalid - empty sequence ID
        let invalid_explain = DebugInstruction::Explain {
            target_sequence_id: "".to_string(),
            location: test_location(),
        };
        assert!(invalid_explain.validate().is_err());
    }

    #[test]
    fn test_flat_instruction_graph_example() {
        // Test AR-1: Flat instruction graph example
        // t0 = LOAD_FIELD("age")
        // t1 = LOAD_LITERAL(18)
        // t2 = CMP_GT(t0, t1)
        // APPLY_FILTER_BOOL(t2)

        let instructions = vec![
            BCIBInstruction::Query(QueryInstruction::LoadField {
                field: "age".to_string(),
                target_register: 0,
                location: test_location(),
            }),
            BCIBInstruction::Query(QueryInstruction::LoadLiteral {
                value: Value::Number(18.0),
                target_register: 1,
                location: test_location(),
            }),
            BCIBInstruction::Query(QueryInstruction::Compare {
                left: OperandRef::TempRegister(0),
                operator: ComparisonOp::GreaterThan,
                right: OperandRef::TempRegister(1),
                target_register: 2,
                location: test_location(),
            }),
            BCIBInstruction::Query(QueryInstruction::ApplyFilterBool {
                filter_register: 2,
                location: test_location(),
            }),
        ];

        let sequence = BCIBSequence::new(instructions);
        assert!(sequence.validate().is_ok());

        // Verify flat structure (no nested expressions)
        assert_eq!(sequence.instructions.len(), 4);

        // Each instruction should be atomic
        for instruction in &sequence.instructions {
            assert!(instruction.validate().is_ok());
        }
    }

    #[test]
    fn test_contextual_capabilities() {
        // Test AR-4: Contextual capabilities
        let read_users = Capability::Read {
            context: "data.users".to_string(),
        };
        let read_logs = Capability::Read {
            context: "fs.logs".to_string(),
        };
        let system_status = Capability::System {
            scope: SystemScope::Status,
        };
        let system_agents = Capability::System {
            scope: SystemScope::Agents,
        };

        // Different contexts should be different capabilities
        assert_ne!(read_users, read_logs);
        assert_ne!(system_status, system_agents);

        // Test system scopes
        assert_eq!(SystemScope::Status, SystemScope::Status);
        assert_ne!(SystemScope::Status, SystemScope::Agents);
    }
}
