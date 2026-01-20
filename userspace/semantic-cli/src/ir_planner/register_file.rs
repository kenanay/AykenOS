//! Register File Implementation
//! 
//! **Created By:** Kenan AY
//! **Date:** 16 Ocak 2026
//! **Architectural Reference:** C3 - Register Model Specification
//! 
//! Virtual register storage with late binding and type safety.
//! Implements HashMap-based register storage for Gate C simplicity.

use crate::execution_plan::RegisterId;
use crate::bcib::Value;
use crate::context::ContextData;
use super::ExecutionError;
use std::collections::HashMap;

/// Register file for virtual register storage
#[derive(Debug, Clone)]
pub struct RegisterFile {
    /// Register storage (virtual register ID -> value)
    registers: HashMap<RegisterId, RegisterValue>,
    /// Allocation order for debugging
    allocation_order: Vec<RegisterId>,
    /// Type constraints for registers
    type_constraints: HashMap<RegisterId, ValueType>,
}

impl RegisterFile {
    /// Create new empty register file
    pub fn new() -> Self {
        Self {
            registers: HashMap::new(),
            allocation_order: Vec::new(),
            type_constraints: HashMap::new(),
        }
    }
    
    /// Set register value with type checking
    /// 
    /// **Architectural Reference:** C3 Section "Single Assignment Semantics"
    pub fn set_register(&mut self, register_id: RegisterId, value: RegisterValue) -> Result<(), RegisterFileError> {
        // Check if register already assigned (single assignment rule)
        if self.registers.contains_key(&register_id) {
            return Err(RegisterFileError::RegisterAlreadyAssigned { register: register_id });
        }
        
        // Check type constraints
        if let Some(expected_type) = self.type_constraints.get(&register_id) {
            let actual_type = value.value_type();
            if !self.is_type_compatible(expected_type, &actual_type) {
                return Err(RegisterFileError::TypeConstraintViolation { 
                    register: register_id,
                    expected: expected_type.clone(),
                    actual: actual_type,
                });
            }
        }
        
        // Store register value
        self.registers.insert(register_id, value);
        
        // Track allocation order
        if !self.allocation_order.contains(&register_id) {
            self.allocation_order.push(register_id);
        }
        
        Ok(())
    }
    
    /// Update register value (for ContextData mutations like filtering)
    /// 
    /// **Gate C Pragmatic:** Allows updating ContextData registers for filter operations
    /// This is a controlled exception to single-assignment for context transformations
    pub fn update_register(&mut self, register_id: RegisterId, value: RegisterValue) -> Result<(), RegisterFileError> {
        // Only allow updates for ContextData registers
        if let Some(existing) = self.registers.get(&register_id) {
            match (existing, &value) {
                (RegisterValue::ContextData(_), RegisterValue::ContextData(_)) => {
                    // Allow ContextData → ContextData updates (filter, transform, etc.)
                    self.registers.insert(register_id, value);
                    Ok(())
                },
                _ => {
                    // All other types follow strict single-assignment
                    Err(RegisterFileError::RegisterAlreadyAssigned { register: register_id })
                }
            }
        } else {
            // Register not assigned yet, use normal set
            self.set_register(register_id, value)
        }
    }
    
    /// Get register value
    pub fn get_register(&self, register_id: RegisterId) -> Result<&RegisterValue, RegisterFileError> {
        self.registers.get(&register_id)
            .ok_or(RegisterFileError::UndefinedRegister { register: register_id })
    }
    
    /// Check if register is defined
    pub fn is_register_defined(&self, register_id: RegisterId) -> bool {
        self.registers.contains_key(&register_id)
    }
    
    /// Set type constraint for register
    pub fn set_type_constraint(&mut self, register_id: RegisterId, value_type: ValueType) {
        self.type_constraints.insert(register_id, value_type);
    }
    
    /// Clear all registers
    pub fn clear(&mut self) {
        self.registers.clear();
        self.allocation_order.clear();
        self.type_constraints.clear();
    }
    
    /// Get all defined registers
    pub fn defined_registers(&self) -> Vec<RegisterId> {
        self.allocation_order.clone()
    }
    
    /// Clone register state for replay
    pub fn clone_state(&self) -> HashMap<RegisterId, RegisterValue> {
        self.registers.clone()
    }
    
    /// Dump register state for debugging
    /// 
    /// **Architectural Reference:** C3 Section "Debug Visibility"
    pub fn dump_state(&self) -> String {
        let mut output = String::new();
        output.push_str("=== Register File State ===\n");
        
        for reg_id in &self.allocation_order {
            if let Some(value) = self.registers.get(reg_id) {
                output.push_str(&format!("{:?}: {}\n", 
                    reg_id, value.debug_format()));
            } else {
                output.push_str(&format!("{:?}: <undefined>\n", reg_id));
            }
        }
        
        if self.allocation_order.is_empty() {
            output.push_str("(no registers allocated)\n");
        }
        
        output
    }
    
    /// Check type compatibility
    fn is_type_compatible(&self, expected: &ValueType, actual: &ValueType) -> bool {
        match (expected, actual) {
            // Exact type match
            (a, b) if a == b => true,
            
            // Any type can be stored in Data registers
            (ValueType::Any, _) => true,
            
            // Context types are compatible
            (ValueType::Context, ValueType::ContextData) => true,
            (ValueType::Context, ValueType::ContextReference) => true,
            
            // Filter types are compatible
            (ValueType::Filter, ValueType::FilterExpression) => true,
            (ValueType::Filter, ValueType::FilterResult) => true,
            
            // No other compatibility
            _ => false,
        }
    }
    
    /// Get register statistics
    pub fn get_statistics(&self) -> RegisterFileStatistics {
        let mut type_counts = HashMap::new();
        
        for value in self.registers.values() {
            let value_type = value.value_type();
            *type_counts.entry(value_type).or_insert(0) += 1;
        }
        
        RegisterFileStatistics {
            total_registers: self.registers.len(),
            allocated_registers: self.allocation_order.len(),
            type_distribution: type_counts,
        }
    }
}

impl Default for RegisterFile {
    fn default() -> Self {
        Self::new()
    }
}

/// Register value types for IR execution
/// 
/// **Architectural Reference:** C3 Section "Register Value System"
#[derive(Debug, Clone, PartialEq)]
pub enum RegisterValue {
    // Primitive values
    String(String),
    Number(f64),
    Boolean(bool),
    Null,
    
    // Complex values
    Array(Vec<RegisterValue>),
    Object(HashMap<String, RegisterValue>),
    
    // Context-specific values
    ContextReference(String),
    ContextData(ContextData),
    
    // Filter-specific values (simplified for Gate C)
    FilterExpression(String), // Simplified as string for Gate C
    FilterResult(bool),
}

impl RegisterValue {
    /// Get type name for debugging
    pub fn type_name(&self) -> &'static str {
        match self {
            RegisterValue::String(_) => "string",
            RegisterValue::Number(_) => "number",
            RegisterValue::Boolean(_) => "boolean",
            RegisterValue::Null => "null",
            RegisterValue::Array(_) => "array",
            RegisterValue::Object(_) => "object",
            RegisterValue::ContextReference(_) => "context_ref",
            RegisterValue::ContextData(_) => "context_data",
            RegisterValue::FilterExpression(_) => "filter_expr",
            RegisterValue::FilterResult(_) => "filter_result",
        }
    }
    
    /// Get value type for type checking
    pub fn value_type(&self) -> ValueType {
        match self {
            RegisterValue::String(_) => ValueType::String,
            RegisterValue::Number(_) => ValueType::Number,
            RegisterValue::Boolean(_) => ValueType::Boolean,
            RegisterValue::Null => ValueType::Null,
            RegisterValue::Array(_) => ValueType::Array,
            RegisterValue::Object(_) => ValueType::Object,
            RegisterValue::ContextReference(_) => ValueType::ContextReference,
            RegisterValue::ContextData(_) => ValueType::ContextData,
            RegisterValue::FilterExpression(_) => ValueType::FilterExpression,
            RegisterValue::FilterResult(_) => ValueType::FilterResult,
        }
    }
    
    /// Convert to boolean for logical operations
    pub fn as_boolean(&self) -> Result<bool, RegisterFileError> {
        match self {
            RegisterValue::Boolean(b) => Ok(*b),
            RegisterValue::FilterResult(b) => Ok(*b),
            _ => Err(RegisterFileError::TypeConversionFailed { 
                from: self.type_name().to_string(),
                to: "boolean".to_string(),
            }),
        }
    }
    
    /// Convert from BCIB Value
    pub fn from_bcib_value(value: Value) -> Result<Self, ExecutionError> {
        match value {
            Value::String(s) => Ok(RegisterValue::String(s)),
            Value::Number(n) => Ok(RegisterValue::Number(n)),
            Value::Boolean(b) => Ok(RegisterValue::Boolean(b)),
            // Collections are not supported in register file (Phase 3.1)
            // They should be handled at the loop execution level
            Value::Array(_) | Value::List(_) | Value::SortedMap(_) => {
                Err(ExecutionError::InvalidRegisterValue(
                    "Collection values cannot be stored in registers".to_string()
                ))
            }
        }
    }
    
    /// Convert to BCIB Value (if possible)
    pub fn to_bcib_value(&self) -> Option<Value> {
        match self {
            RegisterValue::String(s) => Some(Value::String(s.clone())),
            RegisterValue::Number(n) => Some(Value::Number(*n)),
            RegisterValue::Boolean(b) => Some(Value::Boolean(*b)),
            _ => None, // Complex types cannot be converted to BCIB Value
        }
    }
    
    /// Format for debug output (human-readable)
    pub fn debug_format(&self) -> String {
        match self {
            RegisterValue::String(s) => format!("string = \"{}\"", s),
            RegisterValue::Number(n) => format!("number = {}", n),
            RegisterValue::Boolean(b) => format!("boolean = {}", b),
            RegisterValue::Null => "null = null".to_string(),
            RegisterValue::Array(arr) => format!("array = [{}]", arr.len()),
            RegisterValue::Object(obj) => format!("object = {{{}}}", obj.len()),
            RegisterValue::ContextReference(r) => format!("context_ref = \"{}\"", r),
            RegisterValue::ContextData(_) => "context_data = ContextData {{ ... }}".to_string(),
            RegisterValue::FilterExpression(e) => format!("filter_expr = \"{}\"", e),
            RegisterValue::FilterResult(r) => format!("filter_result = {}", r),
        }
    }
}

/// Value types for type checking
/// 
/// **Architectural Reference:** C3 Section "Type Safety Rules"
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ValueType {
    // Primitive types
    String,
    Number,
    Boolean,
    Null,
    
    // Complex types
    Array,
    Object,
    
    // Context types
    ContextReference,
    ContextData,
    
    // Filter types
    FilterExpression,
    FilterResult,
    
    // Meta types
    Any,        // Can hold any type (for Data registers)
    Context,    // Can hold any context type
    Filter,     // Can hold any filter type
}

/// Register file statistics
#[derive(Debug, Clone)]
pub struct RegisterFileStatistics {
    pub total_registers: usize,
    pub allocated_registers: usize,
    pub type_distribution: HashMap<ValueType, usize>,
}

/// Register file errors
#[derive(Debug, Clone, thiserror::Error)]
pub enum RegisterFileError {
    #[error("Undefined register: {register:?}")]
    UndefinedRegister { register: RegisterId },
    
    #[error("Register {register:?} already assigned (single assignment violation)")]
    RegisterAlreadyAssigned { register: RegisterId },
    
    #[error("Type constraint violation for register {register:?}: expected {expected:?}, got {actual:?}")]
    TypeConstraintViolation { 
        register: RegisterId, 
        expected: ValueType, 
        actual: ValueType 
    },
    
    #[error("Type conversion failed: cannot convert {from} to {to}")]
    TypeConversionFailed { from: String, to: String },
    
    #[error("Register file operation failed: {reason}")]
    OperationFailed { reason: String },
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_register_file_creation() {
        let register_file = RegisterFile::new();
        assert_eq!(register_file.registers.len(), 0);
        assert_eq!(register_file.allocation_order.len(), 0);
    }
    
    #[test]
    fn test_register_assignment() {
        let mut register_file = RegisterFile::new();
        
        let result = register_file.set_register(0, RegisterValue::String("test".to_string()));
        assert!(result.is_ok());
        
        let value = register_file.get_register(0).unwrap();
        match value {
            RegisterValue::String(s) => assert_eq!(s, "test"),
            _ => panic!("Expected string value"),
        }
    }
    
    #[test]
    fn test_single_assignment_violation() {
        let mut register_file = RegisterFile::new();
        
        // First assignment should succeed
        let result1 = register_file.set_register(0, RegisterValue::String("test1".to_string()));
        assert!(result1.is_ok());
        
        // Second assignment should fail
        let result2 = register_file.set_register(0, RegisterValue::String("test2".to_string()));
        assert!(result2.is_err());
        
        match result2.unwrap_err() {
            RegisterFileError::RegisterAlreadyAssigned { register } => {
                assert_eq!(register, 0);
            },
            _ => panic!("Expected RegisterAlreadyAssigned error"),
        }
    }
    
    #[test]
    fn test_undefined_register_access() {
        let register_file = RegisterFile::new();
        
        let result = register_file.get_register(999);
        assert!(result.is_err());
        
        match result.unwrap_err() {
            RegisterFileError::UndefinedRegister { register } => {
                assert_eq!(register, 999);
            },
            _ => panic!("Expected UndefinedRegister error"),
        }
    }
    
    #[test]
    fn test_type_constraints() {
        let mut register_file = RegisterFile::new();
        
        // Set type constraint
        register_file.set_type_constraint(0, ValueType::String);
        
        // Valid assignment
        let result1 = register_file.set_register(0, RegisterValue::String("test".to_string()));
        assert!(result1.is_ok());
        
        // Clear and try invalid assignment
        register_file.clear();
        register_file.set_type_constraint(1, ValueType::String);
        
        let result2 = register_file.set_register(1, RegisterValue::Number(42.0));
        assert!(result2.is_err());
    }
    
    #[test]
    fn test_register_value_conversions() {
        // Test BCIB value conversion
        let bcib_value = Value::Number(42.0);
        let register_value = RegisterValue::from_bcib_value(bcib_value).unwrap();
        
        match register_value {
            RegisterValue::Number(n) => assert_eq!(n, 42.0),
            _ => panic!("Expected number value"),
        }
        
        // Test back conversion
        let back_to_bcib = register_value.to_bcib_value().unwrap();
        match back_to_bcib {
            Value::Number(n) => assert_eq!(n, 42.0),
            _ => panic!("Expected number value"),
        }
    }
    
    #[test]
    fn test_boolean_conversion() {
        let bool_value = RegisterValue::Boolean(true);
        assert_eq!(bool_value.as_boolean().unwrap(), true);
        
        let filter_result = RegisterValue::FilterResult(false);
        assert_eq!(filter_result.as_boolean().unwrap(), false);
        
        let string_value = RegisterValue::String("test".to_string());
        assert!(string_value.as_boolean().is_err());
    }
    
    #[test]
    fn test_register_file_dump() {
        let mut register_file = RegisterFile::new();
        
        register_file.set_register(0, RegisterValue::String("test".to_string())).unwrap();
        register_file.set_register(1, RegisterValue::Number(42.0)).unwrap();
        
        let dump = register_file.dump_state();
        assert!(dump.contains("Register File State"));
        assert!(dump.contains("string = \"test\""));
        assert!(dump.contains("number = 42"));
    }
    
    #[test]
    fn test_register_statistics() {
        let mut register_file = RegisterFile::new();
        
        register_file.set_register(0, RegisterValue::String("test".to_string())).unwrap();
        register_file.set_register(1, RegisterValue::Number(42.0)).unwrap();
        register_file.set_register(2, RegisterValue::Boolean(true)).unwrap();
        
        let stats = register_file.get_statistics();
        assert_eq!(stats.total_registers, 3);
        assert_eq!(stats.allocated_registers, 3);
        assert_eq!(stats.type_distribution.get(&ValueType::String), Some(&1));
        assert_eq!(stats.type_distribution.get(&ValueType::Number), Some(&1));
        assert_eq!(stats.type_distribution.get(&ValueType::Boolean), Some(&1));
    }
    
    #[test]
    fn test_type_compatibility() {
        let register_file = RegisterFile::new();
        
        // Exact match
        assert!(register_file.is_type_compatible(&ValueType::String, &ValueType::String));
        
        // Any type compatibility
        assert!(register_file.is_type_compatible(&ValueType::Any, &ValueType::String));
        assert!(register_file.is_type_compatible(&ValueType::Any, &ValueType::Number));
        
        // Context type compatibility
        assert!(register_file.is_type_compatible(&ValueType::Context, &ValueType::ContextData));
        assert!(register_file.is_type_compatible(&ValueType::Context, &ValueType::ContextReference));
        
        // Filter type compatibility
        assert!(register_file.is_type_compatible(&ValueType::Filter, &ValueType::FilterExpression));
        assert!(register_file.is_type_compatible(&ValueType::Filter, &ValueType::FilterResult));
        
        // Incompatible types
        assert!(!register_file.is_type_compatible(&ValueType::String, &ValueType::Number));
        assert!(!register_file.is_type_compatible(&ValueType::Context, &ValueType::String));
    }
}