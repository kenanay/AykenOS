// bcib_simple.rs
// Simplified BCIB structure for Phase-16A submission pipeline
// This is a minimal subset for testing the submission/replay/audit pipeline

use serde::{Deserialize, Serialize};

/// Simplified BCIB structure
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BCIB {
    pub instructions: Vec<BCIBInstruction>,
}

/// Simplified BCIB instruction set for Phase-16A
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BCIBInstruction {
    /// Query data from context
    DataQuery {
        target: BCIBOperand,
        context: String,
        filter: Option<String>,
    },
    
    /// Create data in context (mutation)
    DataCreate {
        target: BCIBOperand,
        context: String,
        data: String,
    },
    
    /// End execution and return result
    End {
        result: BCIBOperand,
    },
    
    /// Emit trace for observability
    TraceEmit {
        message: String,
    },
    
    /// No operation (should never appear in production)
    Nop,
}

/// BCIB operand (register or literal)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BCIBOperand {
    Register(usize),
    Literal(String),
}
