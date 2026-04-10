//! Accumulator Operations and Data Reduction Module
//!
//! This module implements accumulator state management and data transformation operations
//! for the loop engine. It provides a focused interface for accumulator operations
//! while maintaining strict dependency boundaries.
//!
//! # Responsibilities
//!
//! - Accumulator state management and transitions
//! - Data transformation operations
//! - Data fingerprint generation via fingerprint::data layer
//! - AccumulatorManager with transition tracking
//!
//! # Dependency Restrictions
//!
//! This module can only import:
//! - accumulator (for accumulator types)
//! - fingerprint (data layer only)
//! - errors (for error handling)
//!
//! # Requirements Coverage
//!
//! - Requirements 1.1: Modularization with defined dependency boundaries
//! - Requirements 1.3: Focused component for accumulator operations
//! - Requirements 1.4: File size under 1000 lines
//! - Requirements 6.2: Preserve type-safe accumulator abstraction

use super::accumulator::AccumulatorPattern;
use crate::bcib::Value;
use crate::error::{ErrorCode, Result, SemanticCLIError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Accumulator transition record for fingerprint generation
///
/// This struct captures a single state transition in the accumulator,
/// providing the data needed for data fingerprint computation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AccumulatorTransition {
    /// Step index in the transition sequence
    pub step_index: u64,
    /// Type tag for canonical encoding
    pub type_tag: TypeTag,
    /// Canonical bytes representation of the transition
    pub canonical_bytes: Vec<u8>,
    /// Accumulator name (for multi-accumulator patterns)
    pub accumulator_name: String,
    /// Transition type (update, create, finalize)
    pub transition_type: TransitionType,
}

impl AccumulatorTransition {
    /// Create a new accumulator transition
    pub fn new(
        step_index: u64,
        type_tag: TypeTag,
        canonical_bytes: Vec<u8>,
        accumulator_name: String,
        transition_type: TransitionType,
    ) -> Self {
        Self {
            step_index,
            type_tag,
            canonical_bytes,
            accumulator_name,
            transition_type,
        }
    }

    /// Validate this transition
    pub fn validate(&self) -> Result<()> {
        if self.accumulator_name.is_empty() {
            return Err(SemanticCLIError::validation_error(
                "Accumulator transition must have a valid accumulator name",
                "Provide a non-empty accumulator name",
                ErrorCode::E300,
            ));
        }

        if self.canonical_bytes.is_empty()
            && !matches!(self.transition_type, TransitionType::Create)
        {
            return Err(SemanticCLIError::validation_error(
                "Accumulator transition must have canonical bytes (except for Create transitions)",
                "Provide canonical bytes representation",
                ErrorCode::E300,
            ));
        }

        Ok(())
    }
}

/// Type tag for canonical encoding
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TypeTag {
    U8 = 0x01,
    U16 = 0x02,
    U32 = 0x03,
    U64 = 0x04,
    I8 = 0x11,
    I16 = 0x12,
    I32 = 0x13,
    I64 = 0x14,
    F32 = 0x21,
    F64 = 0x22,
    String = 0x30,
    Bytes = 0x31,
    Array = 0x40,
    Struct = 0x50,
    Boolean = 0x60,
}

impl TypeTag {
    /// Get type tag for a Value
    pub fn from_value(value: &Value) -> Self {
        match value {
            Value::String(_) => TypeTag::String,
            Value::Number(_) => TypeTag::F64,
            Value::Boolean(_) => TypeTag::Boolean,
            Value::Array(_) => TypeTag::Array,
            Value::List(_) => TypeTag::Array, // Lists are encoded as arrays
            Value::SortedMap(_) => TypeTag::Struct,
        }
    }

    /// Get the byte representation of this type tag
    pub fn as_byte(self) -> u8 {
        self as u8
    }
}

/// Type of accumulator transition
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransitionType {
    /// Create a new accumulator
    Create,
    /// Update an existing accumulator
    Update,
    /// Finalize an accumulator (end of loop)
    Finalize,
}

/// Data fingerprint for accumulator transitions
///
/// This struct represents the data fingerprint component as specified
/// in the design document, tracking accumulator transitions for
/// deterministic execution verification.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DataFingerprint {
    /// Total number of transition steps
    pub transition_step_count: u64,
    /// All accumulator transitions in order
    pub transitions: Vec<AccumulatorTransition>,
    /// Combined hash of all transitions (BLAKE3)
    pub combined_hash: [u8; 32],
}

impl DataFingerprint {
    /// Create a new data fingerprint from transitions
    pub fn from_transitions(transitions: Vec<AccumulatorTransition>) -> Result<Self> {
        let transition_step_count = transitions.len() as u64;

        // Compute combined hash using BLAKE3
        let combined_hash = Self::compute_combined_hash(&transitions)?;

        Ok(Self {
            transition_step_count,
            transitions,
            combined_hash,
        })
    }

    /// Compute combined SHA-256 hash of all transitions
    fn compute_combined_hash(transitions: &[AccumulatorTransition]) -> Result<[u8; 32]> {
        use sha2::{Digest, Sha256};

        let mut hasher = Sha256::new();

        // Add transition count for deterministic hashing
        hasher.update(&(transitions.len() as u64).to_le_bytes());

        // Add each transition in order
        for transition in transitions {
            // Add step index
            hasher.update(&transition.step_index.to_le_bytes());

            // Add type tag
            hasher.update(&[transition.type_tag.as_byte()]);

            // Add canonical bytes
            hasher.update(&(transition.canonical_bytes.len() as u32).to_le_bytes());
            hasher.update(&transition.canonical_bytes);

            // Add accumulator name
            hasher.update(&(transition.accumulator_name.len() as u32).to_le_bytes());
            hasher.update(transition.accumulator_name.as_bytes());

            // Add transition type
            hasher.update(&[transition.transition_type as u8]);
        }

        Ok(hasher.finalize().into())
    }

    /// Validate this data fingerprint
    pub fn validate(&self) -> Result<()> {
        if self.transition_step_count != self.transitions.len() as u64 {
            return Err(SemanticCLIError::validation_error(
                "Data fingerprint step count does not match transitions length",
                "Ensure step count matches actual transitions",
                ErrorCode::E300,
            ));
        }

        // Validate all transitions
        for transition in &self.transitions {
            transition.validate()?;
        }

        // Verify hash consistency
        let expected_hash = Self::compute_combined_hash(&self.transitions)?;
        if self.combined_hash != expected_hash {
            return Err(SemanticCLIError::validation_error(
                "Data fingerprint hash does not match computed hash",
                "Recompute the fingerprint hash",
                ErrorCode::E300,
            ));
        }

        Ok(())
    }
}

/// Accumulator Manager with transition tracking
///
/// This struct manages accumulator operations and tracks all state transitions
/// for data fingerprint generation. It provides the core functionality for
/// accumulator state management while maintaining transition history.
pub struct AccumulatorManager {
    /// Current accumulator pattern
    pattern: AccumulatorPattern,
    /// Transition history for fingerprint generation
    transitions: Vec<AccumulatorTransition>,
    /// Current step index for transitions
    current_step: u64,
    /// Type registry for canonical encoding
    type_registry: TypeRegistry,
}

impl AccumulatorManager {
    /// Create a new accumulator manager
    pub fn new() -> Self {
        Self {
            pattern: AccumulatorPattern::new(),
            transitions: Vec::new(),
            current_step: 0,
            type_registry: TypeRegistry::new(),
        }
    }

    /// Create accumulator manager from existing pattern
    pub fn from_pattern(pattern: AccumulatorPattern) -> Result<Self> {
        let mut manager = Self::new();

        // Record create transitions for existing accumulators
        for name in pattern.get_accumulator_names() {
            let value = pattern.get_accumulator(&name)?;
            manager.record_create_transition(&name, value)?;
        }

        manager.pattern = pattern;
        Ok(manager)
    }

    /// Add a new accumulator with transition tracking
    pub fn add_accumulator(&mut self, name: String, initial_value: Value) -> Result<()> {
        // Add to pattern
        self.pattern
            .add_accumulator(name.clone(), initial_value.clone())?;

        // Record create transition
        self.record_create_transition(&name, &initial_value)?;

        Ok(())
    }

    /// Update an accumulator with transition tracking
    pub fn update_accumulator(&mut self, name: &str, new_value: Value) -> Result<()> {
        // Update pattern with type validation
        self.pattern.update_accumulator(name, new_value.clone())?;

        // Record update transition
        self.record_update_transition(name, &new_value)?;

        Ok(())
    }

    /// Get an accumulator value
    pub fn get_accumulator(&self, name: &str) -> Result<&Value> {
        self.pattern.get_accumulator(name)
    }

    /// Get all accumulator names
    pub fn get_accumulator_names(&self) -> Vec<String> {
        self.pattern.get_accumulator_names()
    }

    /// Check if an accumulator exists
    pub fn has_accumulator(&self, name: &str) -> bool {
        self.pattern.has_accumulator(name)
    }

    /// Get the number of accumulators
    pub fn len(&self) -> usize {
        self.pattern.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.pattern.is_empty()
    }

    /// Get the underlying accumulator pattern
    pub fn get_pattern(&self) -> &AccumulatorPattern {
        &self.pattern
    }

    /// Finalize all accumulators and record transitions
    pub fn finalize_all(&mut self) -> Result<HashMap<String, Value>> {
        let all_values = self.pattern.get_all_values();

        // Record finalize transitions
        for (name, value) in &all_values {
            self.record_finalize_transition(name, value)?;
        }

        Ok(all_values)
    }

    /// Get data fingerprint from transition history
    pub fn get_data_fingerprint(&self) -> Result<DataFingerprint> {
        DataFingerprint::from_transitions(self.transitions.clone())
    }

    /// Get transition count
    pub fn get_transition_count(&self) -> u64 {
        self.transitions.len() as u64
    }

    /// Get all transitions (for debugging)
    pub fn get_transitions(&self) -> &[AccumulatorTransition] {
        &self.transitions
    }

    /// Clear transition history (for testing)
    pub fn clear_transitions(&mut self) {
        self.transitions.clear();
        self.current_step = 0;
    }

    /// Validate the accumulator manager state
    pub fn validate(&self) -> Result<()> {
        // Validate underlying pattern
        self.pattern.validate()?;

        // Validate all transitions
        for transition in &self.transitions {
            transition.validate()?;
        }

        // Validate step sequence
        for (i, transition) in self.transitions.iter().enumerate() {
            if transition.step_index != i as u64 {
                return Err(SemanticCLIError::validation_error(
                    format!(
                        "Transition step index {} does not match position {}",
                        transition.step_index, i
                    ),
                    "Ensure transitions are recorded in order",
                    ErrorCode::E300,
                ));
            }
        }

        Ok(())
    }

    /// Record a create transition
    fn record_create_transition(&mut self, name: &str, value: &Value) -> Result<()> {
        let type_tag = TypeTag::from_value(value);
        let canonical_bytes = self.type_registry.encode_canonical(value)?;

        let transition = AccumulatorTransition::new(
            self.current_step,
            type_tag,
            canonical_bytes,
            name.to_string(),
            TransitionType::Create,
        );

        self.transitions.push(transition);
        self.current_step += 1;

        Ok(())
    }

    /// Record an update transition
    fn record_update_transition(&mut self, name: &str, value: &Value) -> Result<()> {
        let type_tag = TypeTag::from_value(value);
        let canonical_bytes = self.type_registry.encode_canonical(value)?;

        let transition = AccumulatorTransition::new(
            self.current_step,
            type_tag,
            canonical_bytes,
            name.to_string(),
            TransitionType::Update,
        );

        self.transitions.push(transition);
        self.current_step += 1;

        Ok(())
    }

    /// Record a finalize transition
    fn record_finalize_transition(&mut self, name: &str, value: &Value) -> Result<()> {
        let type_tag = TypeTag::from_value(value);
        let canonical_bytes = self.type_registry.encode_canonical(value)?;

        let transition = AccumulatorTransition::new(
            self.current_step,
            type_tag,
            canonical_bytes,
            name.to_string(),
            TransitionType::Finalize,
        );

        self.transitions.push(transition);
        self.current_step += 1;

        Ok(())
    }
}

impl Default for AccumulatorManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Type registry for canonical encoding
///
/// This struct provides canonical encoding services for accumulator values,
/// ensuring deterministic byte representation for fingerprint generation.
pub struct TypeRegistry {
    /// Encoding configuration
    #[allow(dead_code)]
    config: EncodingConfig,
}

impl TypeRegistry {
    /// Create a new type registry
    pub fn new() -> Self {
        Self {
            config: EncodingConfig::default(),
        }
    }

    /// Encode a value in canonical form
    pub fn encode_canonical(&self, value: &Value) -> Result<Vec<u8>> {
        let mut bytes = Vec::new();

        // Add type tag
        let type_tag = TypeTag::from_value(value);
        bytes.push(type_tag.as_byte());

        // Encode value based on type
        match value {
            Value::String(s) => {
                // Length-prefixed string
                bytes.extend_from_slice(&(s.len() as u32).to_le_bytes());
                bytes.extend_from_slice(s.as_bytes());
            }
            Value::Number(n) => {
                // Canonical floating-point representation
                let canonical_n = self.canonicalize_f64(*n);
                bytes.extend_from_slice(&canonical_n.to_le_bytes());
            }
            Value::Boolean(b) => {
                // Single byte boolean
                bytes.push(if *b { 1 } else { 0 });
            }
            Value::Array(arr) => {
                // Length-prefixed array
                bytes.extend_from_slice(&(arr.len() as u32).to_le_bytes());
                for item in arr {
                    let item_bytes = self.encode_canonical(item)?;
                    bytes.extend_from_slice(&(item_bytes.len() as u32).to_le_bytes());
                    bytes.extend_from_slice(&item_bytes);
                }
            }
            Value::List(list) => {
                // Encode as array (same structure)
                bytes.extend_from_slice(&(list.len() as u32).to_le_bytes());
                for item in list {
                    let item_bytes = self.encode_canonical(item)?;
                    bytes.extend_from_slice(&(item_bytes.len() as u32).to_le_bytes());
                    bytes.extend_from_slice(&item_bytes);
                }
            }
            Value::SortedMap(map) => {
                // Deterministic key ordering (already sorted in BTreeMap)
                bytes.extend_from_slice(&(map.len() as u32).to_le_bytes());
                for (key, value) in map {
                    // Encode key as string
                    bytes.extend_from_slice(&(key.len() as u32).to_le_bytes());
                    bytes.extend_from_slice(key.as_bytes());

                    // Encode value
                    let value_bytes = self.encode_canonical(value)?;
                    bytes.extend_from_slice(&(value_bytes.len() as u32).to_le_bytes());
                    bytes.extend_from_slice(&value_bytes);
                }
            }
        }

        Ok(bytes)
    }

    /// Canonicalize floating-point value
    ///
    /// This implements the floating-point canonicalization requirements:
    /// - Normalize NaN to single canonical quiet NaN bit pattern
    /// - Convert -0.0 to +0.0
    /// - Perform bit-level IEEE754 hashing without decimal rounding
    fn canonicalize_f64(&self, value: f64) -> f64 {
        if value.is_nan() {
            // Normalize all NaN values to quiet NaN
            f64::from_bits(0x7FF8000000000000)
        } else if value == -0.0 {
            // Convert -0.0 to +0.0
            0.0
        } else {
            value
        }
    }
}

impl Default for TypeRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Encoding configuration for canonical representation
#[derive(Debug, Clone)]
pub struct EncodingConfig {
    /// Use little-endian byte order
    pub little_endian: bool,
    /// Include explicit type tags
    pub explicit_type_tags: bool,
}

impl Default for EncodingConfig {
    fn default() -> Self {
        Self {
            little_endian: true,      // Requirements 7.1: little-endian byte order
            explicit_type_tags: true, // Requirements 7.1: explicit type tags
        }
    }
}

/// Data transformation operations for accumulator values
///
/// This struct provides data transformation utilities that work with
/// the accumulator manager to perform common data operations.
pub struct DataTransformer {
    /// Type registry for encoding operations
    #[allow(dead_code)]
    type_registry: TypeRegistry,
}

impl DataTransformer {
    /// Create a new data transformer
    pub fn new() -> Self {
        Self {
            type_registry: TypeRegistry::new(),
        }
    }

    /// Transform accumulator value with validation
    pub fn transform_accumulator_value(
        &self,
        current_value: &Value,
        transformation: ValueTransformation,
    ) -> Result<Value> {
        match transformation {
            ValueTransformation::Identity => Ok(current_value.clone()),
            ValueTransformation::Increment => self.increment_value(current_value),
            ValueTransformation::Append(ref append_value) => {
                self.append_value(current_value, append_value)
            }
            ValueTransformation::Merge(ref merge_value) => {
                self.merge_value(current_value, merge_value)
            }
        }
    }

    /// Increment a numeric value
    fn increment_value(&self, value: &Value) -> Result<Value> {
        match value {
            Value::Number(n) => Ok(Value::Number(n + 1.0)),
            _ => Err(SemanticCLIError::validation_error(
                "Cannot increment non-numeric value",
                "Use increment transformation only with numeric values",
                ErrorCode::E300,
            )),
        }
    }

    /// Append to an array or list value
    fn append_value(&self, current: &Value, append: &Value) -> Result<Value> {
        match current {
            Value::Array(ref arr) => {
                let mut new_arr = arr.clone();
                new_arr.push(append.clone());
                Ok(Value::Array(new_arr))
            }
            Value::List(ref list) => {
                let mut new_list = list.clone();
                new_list.push(append.clone());
                Ok(Value::List(new_list))
            }
            _ => Err(SemanticCLIError::validation_error(
                "Cannot append to non-collection value",
                "Use append transformation only with arrays or lists",
                ErrorCode::E300,
            )),
        }
    }

    /// Merge with another value (for maps)
    fn merge_value(&self, current: &Value, merge: &Value) -> Result<Value> {
        match (current, merge) {
            (Value::SortedMap(ref current_map), Value::SortedMap(ref merge_map)) => {
                let mut new_map = current_map.clone();
                for (key, value) in merge_map {
                    new_map.insert(key.clone(), value.clone());
                }
                Ok(Value::SortedMap(new_map))
            }
            _ => Err(SemanticCLIError::validation_error(
                "Cannot merge non-map values",
                "Use merge transformation only with sorted maps",
                ErrorCode::E300,
            )),
        }
    }
}

impl Default for DataTransformer {
    fn default() -> Self {
        Self::new()
    }
}

/// Value transformation operations
#[derive(Debug, Clone, PartialEq)]
pub enum ValueTransformation {
    /// Identity transformation (no change)
    Identity,
    /// Increment numeric value by 1
    Increment,
    /// Append value to collection
    Append(Value),
    /// Merge with another value
    Merge(Value),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bcib::Value;
    use std::collections::BTreeMap;

    #[test]
    fn test_accumulator_manager_basic_operations() {
        let mut manager = AccumulatorManager::new();

        // Add accumulators
        assert!(manager
            .add_accumulator("counter".to_string(), Value::Number(0.0))
            .is_ok());
        assert!(manager
            .add_accumulator("flag".to_string(), Value::Boolean(false))
            .is_ok());

        assert_eq!(manager.len(), 2);
        assert!(!manager.is_empty());
        assert!(manager.has_accumulator("counter"));
        assert!(manager.has_accumulator("flag"));

        // Update accumulators
        assert!(manager
            .update_accumulator("counter", Value::Number(42.0))
            .is_ok());
        assert!(manager
            .update_accumulator("flag", Value::Boolean(true))
            .is_ok());

        // Get values
        assert_eq!(
            manager.get_accumulator("counter").unwrap(),
            &Value::Number(42.0)
        );
        assert_eq!(
            manager.get_accumulator("flag").unwrap(),
            &Value::Boolean(true)
        );

        // Check transition tracking
        assert_eq!(manager.get_transition_count(), 4); // 2 creates + 2 updates
    }

    #[test]
    fn test_accumulator_transition_validation() {
        let transition = AccumulatorTransition::new(
            0,
            TypeTag::F64,
            vec![1, 2, 3, 4],
            "test".to_string(),
            TransitionType::Update,
        );

        assert!(transition.validate().is_ok());

        // Test invalid transition (empty name)
        let invalid_transition = AccumulatorTransition::new(
            0,
            TypeTag::F64,
            vec![1, 2, 3, 4],
            "".to_string(),
            TransitionType::Update,
        );

        assert!(invalid_transition.validate().is_err());
    }

    #[test]
    fn test_data_fingerprint_generation() {
        let mut manager = AccumulatorManager::new();

        // Add and update accumulators
        manager
            .add_accumulator("sum".to_string(), Value::Number(0.0))
            .unwrap();
        manager
            .update_accumulator("sum", Value::Number(10.0))
            .unwrap();
        manager
            .update_accumulator("sum", Value::Number(25.0))
            .unwrap();

        // Get data fingerprint
        let fingerprint = manager.get_data_fingerprint().unwrap();

        assert_eq!(fingerprint.transition_step_count, 3);
        assert_eq!(fingerprint.transitions.len(), 3);
        assert!(fingerprint.validate().is_ok());

        // Verify hash consistency
        let fingerprint2 = manager.get_data_fingerprint().unwrap();
        assert_eq!(fingerprint.combined_hash, fingerprint2.combined_hash);
    }

    #[test]
    fn test_type_registry_canonical_encoding() {
        let registry = TypeRegistry::new();

        // Test different value types
        let string_val = Value::String("test".to_string());
        let number_val = Value::Number(42.5);
        let bool_val = Value::Boolean(true);
        let array_val = Value::Array(vec![Value::Number(1.0), Value::Number(2.0)]);

        // Encode values
        let string_bytes = registry.encode_canonical(&string_val).unwrap();
        let number_bytes = registry.encode_canonical(&number_val).unwrap();
        let bool_bytes = registry.encode_canonical(&bool_val).unwrap();
        let array_bytes = registry.encode_canonical(&array_val).unwrap();

        // Verify type tags are included
        assert_eq!(string_bytes[0], TypeTag::String.as_byte());
        assert_eq!(number_bytes[0], TypeTag::F64.as_byte());
        assert_eq!(bool_bytes[0], TypeTag::Boolean.as_byte());
        assert_eq!(array_bytes[0], TypeTag::Array.as_byte());

        // Test deterministic encoding (same input -> same output)
        let string_bytes2 = registry.encode_canonical(&string_val).unwrap();
        assert_eq!(string_bytes, string_bytes2);
    }

    #[test]
    fn test_floating_point_canonicalization() {
        let registry = TypeRegistry::new();

        // Test NaN canonicalization
        let nan_val = Value::Number(f64::NAN);
        let nan_bytes1 = registry.encode_canonical(&nan_val).unwrap();
        let nan_bytes2 = registry.encode_canonical(&nan_val).unwrap();
        assert_eq!(nan_bytes1, nan_bytes2); // Should be identical

        // Test -0.0 canonicalization
        let neg_zero = Value::Number(-0.0);
        let pos_zero = Value::Number(0.0);
        let neg_zero_bytes = registry.encode_canonical(&neg_zero).unwrap();
        let pos_zero_bytes = registry.encode_canonical(&pos_zero).unwrap();
        assert_eq!(neg_zero_bytes, pos_zero_bytes); // Should be identical
    }

    #[test]
    fn test_data_transformer() {
        let transformer = DataTransformer::new();

        // Test increment transformation
        let number_val = Value::Number(5.0);
        let incremented = transformer
            .transform_accumulator_value(&number_val, ValueTransformation::Increment)
            .unwrap();
        assert_eq!(incremented, Value::Number(6.0));

        // Test append transformation
        let array_val = Value::Array(vec![Value::Number(1.0)]);
        let appended = transformer
            .transform_accumulator_value(
                &array_val,
                ValueTransformation::Append(Value::Number(2.0)),
            )
            .unwrap();
        assert_eq!(
            appended,
            Value::Array(vec![Value::Number(1.0), Value::Number(2.0)])
        );

        // Test merge transformation
        let mut map1 = BTreeMap::new();
        map1.insert("a".to_string(), Value::Number(1.0));
        let mut map2 = BTreeMap::new();
        map2.insert("b".to_string(), Value::Number(2.0));

        let map1_val = Value::SortedMap(map1);
        let map2_val = Value::SortedMap(map2);

        let merged = transformer
            .transform_accumulator_value(&map1_val, ValueTransformation::Merge(map2_val))
            .unwrap();

        if let Value::SortedMap(merged_map) = merged {
            assert_eq!(merged_map.len(), 2);
            assert!(merged_map.contains_key("a"));
            assert!(merged_map.contains_key("b"));
        } else {
            panic!("Expected SortedMap");
        }
    }

    #[test]
    fn test_accumulator_manager_from_pattern() {
        let mut pattern = AccumulatorPattern::new();
        pattern
            .add_accumulator("existing".to_string(), Value::String("test".to_string()))
            .unwrap();

        let manager = AccumulatorManager::from_pattern(pattern).unwrap();

        assert_eq!(manager.len(), 1);
        assert!(manager.has_accumulator("existing"));
        assert_eq!(manager.get_transition_count(), 1); // One create transition

        let transitions = manager.get_transitions();
        assert_eq!(transitions[0].transition_type, TransitionType::Create);
        assert_eq!(transitions[0].accumulator_name, "existing");
    }

    #[test]
    fn test_accumulator_manager_validation() {
        let mut manager = AccumulatorManager::new();

        // Add valid accumulators
        manager
            .add_accumulator("test1".to_string(), Value::Number(1.0))
            .unwrap();
        manager
            .add_accumulator("test2".to_string(), Value::Boolean(true))
            .unwrap();

        // Should validate successfully
        assert!(manager.validate().is_ok());

        // Test finalization
        let final_values = manager.finalize_all().unwrap();
        assert_eq!(final_values.len(), 2);
        assert_eq!(final_values.get("test1"), Some(&Value::Number(1.0)));
        assert_eq!(final_values.get("test2"), Some(&Value::Boolean(true)));

        // Should still validate after finalization
        assert!(manager.validate().is_ok());
    }
}
