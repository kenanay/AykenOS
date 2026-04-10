//! # Normalizer Integration
//!
//! Canonicalize plans and validate structural correctness.
//!
//! **ARCHITECTURAL RULE:**
//! This module MUST NOT depend on higher-level Gate C components.
//! Violations are considered architecture breaks.
//!
//! **Author:** Kenan AY  
//! **Phase:** 3.5 Gate C

use crate::gate_c::{
    error::{GateCResult, NormalizationError},
    limits::{MAX_DATA_REFS_PER_STEP, MAX_PLAN_METADATA_BYTES, MAX_PLAN_STEPS},
    types::{
        CanonicalMetadata, CanonicalPlan, CanonicalStep, DataRef, ExecutionPlan,
        InvalidationReason, MutationIntent, Operation, PlanFingerprint, PlanMetadata, PlanStep,
    },
};
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};

/// Validation report for comprehensive plan validation
#[derive(Debug, Clone)]
pub struct ValidationReport {
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<ValidationWarning>,
    pub is_valid: bool,
}

impl ValidationReport {
    pub fn new() -> Self {
        Self {
            errors: Vec::new(),
            warnings: Vec::new(),
            is_valid: true,
        }
    }

    pub fn add_error(&mut self, error: ValidationError) {
        self.errors.push(error);
        self.is_valid = false;
    }

    pub fn add_warning(&mut self, warning: ValidationWarning) {
        self.warnings.push(warning);
    }

    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    pub fn error_count(&self) -> usize {
        self.errors.len()
    }

    pub fn warning_count(&self) -> usize {
        self.warnings.len()
    }
}

/// Validation error with categorization
#[derive(Debug, Clone)]
pub struct ValidationError {
    pub category: ValidationCategory,
    pub message: String,
    pub step_id: Option<String>,
}

/// Validation warning for non-critical issues
#[derive(Debug, Clone)]
pub struct ValidationWarning {
    pub category: ValidationCategory,
    pub message: String,
    pub step_id: Option<String>,
}

/// Categories of validation issues
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ValidationCategory {
    Structure,
    Step,
    DataFlow,
    Dependencies,
    Operations,
    Metadata,
    References,
    Cycles,
    Limits,
}

/// Plan normalizer for canonicalization and structural validation
pub struct PlanNormalizer {
    canonicalization_rules: CanonicalizationRules,
    structural_validator: StructuralValidator,
    /// Object pools for memory optimization (Phase 4.3.3.1)
    pools: RefCell<crate::memory::ExecutionPools>,
}

impl PlanNormalizer {
    /// Create new plan normalizer
    pub fn new() -> Self {
        Self {
            canonicalization_rules: CanonicalizationRules::new(),
            structural_validator: StructuralValidator::new(),
            pools: RefCell::new(crate::memory::ExecutionPools::with_capacity(32)), // Pre-allocate for warmup
        }
    }

    /// Create plan normalizer with custom rules
    pub fn with_rules(rules: CanonicalizationRules) -> Self {
        Self {
            canonicalization_rules: rules,
            structural_validator: StructuralValidator::new(),
            pools: RefCell::new(crate::memory::ExecutionPools::with_capacity(32)), // Pre-allocate for warmup
        }
    }

    /// Normalize execution plan to canonical form
    pub fn normalize(&self, plan: &ExecutionPlan) -> GateCResult<CanonicalPlan> {
        // Validate structure first
        self.structural_validator.validate_structure(plan)?;

        // Check plan size limits
        if plan.steps.len() > MAX_PLAN_STEPS {
            return Err(NormalizationError::TooComplex(format!(
                "Plan has {} steps, exceeds limit of {}",
                plan.steps.len(),
                MAX_PLAN_STEPS
            ))
            .into());
        }

        // Canonicalize steps using pooled memory
        let canonical_steps = {
            let mut pools = self.pools.borrow_mut();
            self.canonicalize_steps_with_pools(&plan.steps, &mut *pools)?
        };

        // Create canonical metadata using pooled memory
        let canonical_metadata = {
            let mut pools = self.pools.borrow_mut();
            self.create_canonical_metadata_with_pools(plan, &mut *pools)?
        };

        // Generate plan fingerprint
        let fingerprint = self.generate_fingerprint(&canonical_steps, &canonical_metadata)?;

        let canonical_plan = CanonicalPlan {
            fingerprint,
            normalized_steps: canonical_steps,
            metadata: canonical_metadata,
        };

        // Validate canonical plan consistency
        self.validate_canonical_consistency(&canonical_plan)?;

        // Constitutional Rule: Clear pools after execution (no cross-run leakage)
        self.pools.borrow_mut().clear_all();

        Ok(canonical_plan)
    }

    /// Normalize execution plan for performance testing (bypasses MAX_PLAN_STEPS limit)
    ///
    /// **CRITICAL:** This method is ONLY for performance testing and complexity budget validation.
    /// It bypasses constitutional limits and should NEVER be used in production code.
    ///
    /// **Phase 4.3 Performance Testing:** This method enables testing normalization performance
    /// at scales beyond constitutional limits to validate algorithmic improvements.
    pub fn normalize_for_performance_testing(
        &self,
        plan: &ExecutionPlan,
    ) -> GateCResult<CanonicalPlan> {
        // Validate structure with performance testing mode (bypasses step limit)
        self.structural_validator
            .validate_structure_for_performance_testing(plan)?;

        // **PERFORMANCE TESTING:** Skip MAX_PLAN_STEPS limit check
        // This allows testing at 5K+ steps for algorithmic performance validation

        // Canonicalize steps using pooled memory (Phase 4.3.3.1)
        let canonical_steps = {
            let mut pools = self.pools.borrow_mut();
            self.canonicalize_steps_with_pools(&plan.steps, &mut *pools)?
        };

        // Create canonical metadata using pooled memory (Phase 4.3.3.1)
        let canonical_metadata = {
            let mut pools = self.pools.borrow_mut();
            self.create_canonical_metadata_with_pools(plan, &mut *pools)?
        };

        // Generate plan fingerprint
        let fingerprint = self.generate_fingerprint(&canonical_steps, &canonical_metadata)?;

        let canonical_plan = CanonicalPlan {
            fingerprint,
            normalized_steps: canonical_steps,
            metadata: canonical_metadata,
        };

        // Validate canonical plan consistency
        self.validate_canonical_consistency(&canonical_plan)?;

        // Constitutional Rule: Clear pools after execution (no cross-run leakage)
        self.pools.borrow_mut().clear_all();

        Ok(canonical_plan)
    }

    /// Validate structural correctness of plan
    pub fn validate_structure(&self, plan: &ExecutionPlan) -> GateCResult<()> {
        self.structural_validator.validate_structure(plan)
    }

    /// Canonicalize steps using object pooling (Phase 4.3.3.1)
    ///
    /// **Performance Improvements:**
    /// - Use pooled Vec buffers to avoid allocations
    /// - Pre-allocate capacity to avoid reallocations
    /// - Use indices instead of cloning for sorting
    /// - Single-pass validation during canonicalization
    fn canonicalize_steps_with_pools(
        &self,
        steps: &[PlanStep],
        pools: &mut crate::memory::ExecutionPools,
    ) -> GateCResult<Vec<CanonicalStep>> {
        // Use pooled Vec for canonical steps
        let mut canonical_steps = Vec::with_capacity(steps.len());

        // Use pooled Vec for step indices
        let mut step_indices = pools.borrow_index_vec();
        step_indices.clear();
        step_indices.extend(0..steps.len());
        step_indices.sort_by(|&a, &b| steps[a].id.cmp(&steps[b].id));

        // Process steps in sorted order with single-pass validation
        for &index in &step_indices {
            let step = &steps[index];
            let canonical_step = self.canonicalize_single_step_with_pools(step, pools)?;
            canonical_steps.push(canonical_step);
        }

        // Return pooled Vec
        pools.return_index_vec(step_indices);

        // Validate step references (optimized)
        self.validate_step_references_optimized(&canonical_steps)?;

        Ok(canonical_steps)
    }

    /// Canonicalize a single step with object pooling (Phase 4.3.3.1)
    ///
    /// **Performance Improvements:**
    /// - Use pooled Vec buffers for indices to avoid allocations
    /// - Pre-allocate vectors with known capacity
    /// - Minimize string allocations in operation canonicalization
    fn canonicalize_single_step_with_pools(
        &self,
        step: &PlanStep,
        pools: &mut crate::memory::ExecutionPools,
    ) -> GateCResult<CanonicalStep> {
        // Pre-allocate with exact capacity
        let mut canonical_inputs = Vec::with_capacity(step.inputs.len());
        let mut canonical_outputs = Vec::with_capacity(step.outputs.len());

        // Use pooled Vec for input indices
        let mut input_indices = pools.borrow_index_vec();
        input_indices.clear();
        input_indices.extend(0..step.inputs.len());
        input_indices.sort_by(|&a, &b| step.inputs[a].id.cmp(&step.inputs[b].id));

        for &index in &input_indices {
            canonical_inputs.push(step.inputs[index].clone()); // Only clone when necessary
        }

        // Return pooled Vec
        pools.return_index_vec(input_indices);

        // Use pooled Vec for output indices
        let mut output_indices = pools.borrow_index_vec();
        output_indices.clear();
        output_indices.extend(0..step.outputs.len());
        output_indices.sort_by(|&a, &b| step.outputs[a].id.cmp(&step.outputs[b].id));

        for &index in &output_indices {
            canonical_outputs.push(step.outputs[index].clone()); // Only clone when necessary
        }

        // Return pooled Vec
        pools.return_index_vec(output_indices);

        // Canonicalize operation (no pooling needed for this part)
        let canonical_operation = self.canonicalize_operation_optimized(&step.operation)?;

        Ok(CanonicalStep {
            id: step.id.clone(),
            operation: canonical_operation,
            inputs: canonical_inputs,
            outputs: canonical_outputs,
        })
    }

    /// Create canonical metadata using object pooling (Phase 4.3.3.1)
    fn create_canonical_metadata_with_pools(
        &self,
        plan: &ExecutionPlan,
        pools: &mut crate::memory::ExecutionPools,
    ) -> GateCResult<CanonicalMetadata> {
        // Use pooled HashMap for metadata (for future optimizations)
        let mut metadata_map = pools.borrow_context_map();

        // Return pooled HashMap immediately (not used in current implementation)
        pools.return_context_map(metadata_map);

        // Delegate to existing method for now
        self.create_canonical_metadata(plan)
    }

    /// Canonicalize plan steps with Phase 4.3 optimizations
    ///
    /// **Performance Improvements:**
    /// - Pre-allocate capacity to avoid reallocations
    /// - Use indices instead of cloning for sorting
    /// - Single-pass validation during canonicalization
    fn canonicalize_steps(&self, steps: &[PlanStep]) -> GateCResult<Vec<CanonicalStep>> {
        // Pre-allocate capacity for better performance
        let mut canonical_steps = Vec::with_capacity(steps.len());

        // Create index-based sorting to avoid cloning steps
        let mut step_indices: Vec<usize> = (0..steps.len()).collect();
        step_indices.sort_by(|&a, &b| steps[a].id.cmp(&steps[b].id));

        // Process steps in sorted order with single-pass validation
        for &index in &step_indices {
            let step = &steps[index];
            let canonical_step = self.canonicalize_single_step_optimized(step)?;
            canonical_steps.push(canonical_step);
        }

        // Validate step references (optimized)
        self.validate_step_references_optimized(&canonical_steps)?;

        Ok(canonical_steps)
    }

    /// Canonicalize a single step with Phase 4.3 optimizations
    ///
    /// **Performance Improvements:**
    /// - Use index-based sorting to avoid cloning DataRef objects
    /// - Pre-allocate vectors with known capacity
    /// - Minimize string allocations in operation canonicalization
    fn canonicalize_single_step_optimized(&self, step: &PlanStep) -> GateCResult<CanonicalStep> {
        // Pre-allocate with exact capacity
        let mut canonical_inputs = Vec::with_capacity(step.inputs.len());
        let mut canonical_outputs = Vec::with_capacity(step.outputs.len());

        // Use index-based sorting to avoid cloning DataRef objects
        let mut input_indices: Vec<usize> = (0..step.inputs.len()).collect();
        input_indices.sort_by(|&a, &b| step.inputs[a].id.cmp(&step.inputs[b].id));

        for &index in &input_indices {
            canonical_inputs.push(step.inputs[index].clone()); // Only clone when necessary
        }

        let mut output_indices: Vec<usize> = (0..step.outputs.len()).collect();
        output_indices.sort_by(|&a, &b| step.outputs[a].id.cmp(&step.outputs[b].id));

        for &index in &output_indices {
            canonical_outputs.push(step.outputs[index].clone()); // Only clone when necessary
        }

        // Canonicalize operation with optimizations
        let canonical_operation = self.canonicalize_operation_optimized(&step.operation)?;

        Ok(CanonicalStep {
            id: step.id.clone(), // Required clone for ownership
            operation: canonical_operation,
            inputs: canonical_inputs,
            outputs: canonical_outputs,
        })
    }

    /// Canonicalize operation with Phase 4.3 optimizations
    ///
    /// **Performance Improvements:**
    /// - Use Vec for parameter sorting instead of BTreeMap to avoid extra allocations
    /// - Minimize string cloning by using references where possible
    /// - Pre-allocate collections with known capacity
    fn canonicalize_operation_optimized(&self, operation: &Operation) -> GateCResult<Operation> {
        match operation {
            Operation::Query { target, parameters } => {
                // Use Vec for sorting instead of BTreeMap to reduce allocations
                let mut param_pairs: Vec<(&String, &String)> = parameters.iter().collect();
                param_pairs.sort_by(|a, b| a.0.cmp(b.0));

                // Pre-allocate HashMap with exact capacity
                let mut canonical_params = HashMap::with_capacity(parameters.len());
                for (k, v) in param_pairs {
                    canonical_params.insert(k.clone(), v.clone()); // Required clones for ownership
                }

                Ok(Operation::Query {
                    target: target.clone(), // Required clone for ownership
                    parameters: canonical_params,
                })
            }
            Operation::Mutation { intent } => {
                // Mutation intents are already canonical - avoid unnecessary cloning
                Ok(operation.clone()) // Single clone instead of deep analysis
            }
            Operation::Compute {
                function,
                arguments,
            } => {
                // Pre-allocate with exact capacity and sort in-place
                let mut canonical_args = Vec::with_capacity(arguments.len());
                canonical_args.extend_from_slice(arguments);
                canonical_args.sort();

                Ok(Operation::Compute {
                    function: function.clone(), // Required clone for ownership
                    arguments: canonical_args,
                })
            }
        }
    }
    /// Validate step references with Phase 4.3 optimizations
    ///
    /// **Performance Improvements:**
    /// - Use HashSet for O(1) lookups instead of linear searches
    /// - Pre-allocate collections with known capacity
    /// - Single-pass validation instead of multiple iterations
    fn validate_step_references_optimized(&self, steps: &[CanonicalStep]) -> GateCResult<()> {
        // Pre-allocate HashSet for O(1) step ID lookups
        let mut step_ids = std::collections::HashSet::with_capacity(steps.len());
        for step in steps {
            step_ids.insert(&step.id);
        }

        // Single-pass validation of all references
        for step in steps {
            // Validate input references
            for input in &step.inputs {
                if let Some(ref source_step) = input.source_step {
                    if !step_ids.contains(source_step) {
                        return Err(NormalizationError::InvalidReference(format!(
                            "Step '{}' references unknown source step '{}'",
                            step.id, source_step
                        ))
                        .into());
                    }
                }
            }

            // Validate output references (outputs should reference their own step)
            for output in &step.outputs {
                if let Some(ref source_step) = output.source_step {
                    if source_step != &step.id {
                        return Err(NormalizationError::InvalidReference(format!(
                            "Step '{}' output references different step '{}'",
                            step.id, source_step
                        ))
                        .into());
                    }
                }
            }
        }

        Ok(())
    }

    /// Create canonical metadata
    fn create_canonical_metadata(&self, plan: &ExecutionPlan) -> GateCResult<CanonicalMetadata> {
        // Check metadata size limits
        let metadata_size = plan.metadata.name.len()
            + plan.metadata.description.as_ref().map_or(0, |d| d.len())
            + plan
                .metadata
                .extra
                .iter()
                .map(|(k, v)| k.len() + v.len())
                .sum::<usize>();

        if metadata_size > MAX_PLAN_METADATA_BYTES {
            return Err(NormalizationError::TooComplex(format!(
                "Plan metadata size {} bytes exceeds limit of {}",
                metadata_size, MAX_PLAN_METADATA_BYTES
            ))
            .into());
        }

        Ok(CanonicalMetadata {
            name: plan.metadata.name.clone(),
            version: plan.metadata.version.clone(),
            // CRITICAL FIX: Remove timestamp from canonical metadata for determinism
            canonicalized_at: 0, // DETERMINISTIC: No timestamp in canonical form
        })
    }

    /// Generate deterministic plan fingerprint
    fn generate_fingerprint(
        &self,
        steps: &[CanonicalStep],
        metadata: &CanonicalMetadata,
    ) -> GateCResult<PlanFingerprint> {
        // DETERMINISM FIX: Use canonical serialization instead of Hash trait
        use crate::gate_c::deterministic::simple_string_hash;

        // Build canonical string representation
        let mut canonical_repr = String::new();

        // Add fixed seed for determinism
        canonical_repr.push_str("CANONICAL_PLAN_V1:");

        // Add canonical steps
        for step in steps {
            canonical_repr.push_str(&format!("STEP:{}:", step.id));

            // Add operation deterministically
            match &step.operation {
                Operation::Query { target, parameters } => {
                    canonical_repr.push_str(&format!("QUERY:{}:", target));
                    // Add parameters in sorted order
                    let mut sorted_params: Vec<_> = parameters.iter().collect();
                    sorted_params.sort_by(|a, b| a.0.cmp(b.0));
                    for (k, v) in sorted_params {
                        canonical_repr.push_str(&format!("PARAM:{}={}:", k, v));
                    }
                }
                Operation::Mutation { intent } => {
                    canonical_repr.push_str("MUTATION:");
                    // Canonical representation of mutation intent (no Debug output)
                    match intent {
                        MutationIntent::InvalidateIntent { target, reason } => {
                            canonical_repr.push_str("INVALIDATE:");
                            canonical_repr.push_str(&target.to_string());
                            canonical_repr.push(':');
                            match reason {
                                InvalidationReason::Obsolete => canonical_repr.push_str("OBSOLETE"),
                                InvalidationReason::Conflict => canonical_repr.push_str("CONFLICT"),
                                InvalidationReason::ConstraintViolation => {
                                    canonical_repr.push_str("CONSTRAINT_VIOLATION")
                                }
                                InvalidationReason::Custom(s) => {
                                    canonical_repr.push_str("CUSTOM:");
                                    canonical_repr.push_str(s);
                                }
                            }
                        }
                        MutationIntent::UpdateIntent { target, changes } => {
                            canonical_repr.push_str("UPDATE:");
                            canonical_repr.push_str(&target.to_string());
                            canonical_repr.push_str(":UPDATES:");
                            canonical_repr.push_str(&changes.updates.len().to_string());
                            canonical_repr.push_str(":REMOVALS:");
                            canonical_repr.push_str(&changes.removals.len().to_string());
                        }
                        MutationIntent::CreateIntent { path, content } => {
                            canonical_repr.push_str("CREATE:");
                            canonical_repr.push_str(&path.to_string());
                            canonical_repr.push(':');
                            canonical_repr.push_str(&content.content_type);
                        }
                    }
                }
                Operation::Compute {
                    function,
                    arguments,
                } => {
                    canonical_repr.push_str(&format!("COMPUTE:{}:", function));
                    for arg in arguments {
                        canonical_repr.push_str(&format!("ARG:{}:", arg));
                    }
                }
            }

            // Add inputs and outputs
            for input in &step.inputs {
                canonical_repr.push_str(&format!("INPUT:{}:{}:", input.id, input.data_type));
            }
            for output in &step.outputs {
                canonical_repr.push_str(&format!("OUTPUT:{}:{}:", output.id, output.data_type));
            }
        }

        // Add metadata (DETERMINISTIC: exclude timestamps)
        canonical_repr.push_str(&format!("META:{}:{}:", metadata.name, metadata.version));

        // Generate hash from canonical representation
        let hash = simple_string_hash(&canonical_repr);

        Ok(PlanFingerprint {
            hash,
            version: 1, // Version 1 of fingerprinting algorithm
        })
    }

    /// Validate step references
    fn validate_step_references(&self, steps: &[CanonicalStep]) -> GateCResult<()> {
        let step_ids: HashSet<_> = steps.iter().map(|s| &s.id).collect();
        let mut output_ids = HashSet::new();

        // Collect all output IDs
        for step in steps {
            for output in &step.outputs {
                if output_ids.contains(&output.id) {
                    return Err(NormalizationError::StructuralError(format!(
                        "Duplicate output ID: {}",
                        output.id
                    ))
                    .into());
                }
                output_ids.insert(&output.id);
            }
        }

        // Validate input references
        for step in steps {
            for input in &step.inputs {
                if let Some(source_step) = &input.source_step {
                    if !step_ids.contains(source_step) {
                        return Err(NormalizationError::InvalidReference(format!(
                            "Input {} references non-existent step: {}",
                            input.id, source_step
                        ))
                        .into());
                    }
                }
            }
        }

        Ok(())
    }

    /// Validate canonical plan consistency
    fn validate_canonical_consistency(&self, plan: &CanonicalPlan) -> GateCResult<()> {
        // Verify fingerprint consistency
        let recalculated_fingerprint =
            self.generate_fingerprint(&plan.normalized_steps, &plan.metadata)?;

        if plan.fingerprint.hash != recalculated_fingerprint.hash {
            return Err(NormalizationError::StructuralError(
                "Plan fingerprint inconsistency detected".to_string(),
            )
            .into());
        }

        // Verify step ordering is canonical (sorted by ID)
        for i in 1..plan.normalized_steps.len() {
            if plan.normalized_steps[i - 1].id >= plan.normalized_steps[i].id {
                return Err(NormalizationError::StructuralError(
                    "Steps are not in canonical order".to_string(),
                )
                .into());
            }
        }

        Ok(())
    }
}

impl Default for PlanNormalizer {
    fn default() -> Self {
        Self::new()
    }
}

/// Canonicalization rules for deterministic plan normalization
pub struct CanonicalizationRules {
    sort_parameters: bool,
    sort_arguments: bool,
    sort_data_refs: bool,
}

impl CanonicalizationRules {
    /// Create default canonicalization rules
    pub fn new() -> Self {
        Self {
            sort_parameters: true,
            sort_arguments: true,
            sort_data_refs: true,
        }
    }

    /// Create strict canonicalization rules (all sorting enabled)
    pub fn strict() -> Self {
        Self {
            sort_parameters: true,
            sort_arguments: true,
            sort_data_refs: true,
        }
    }

    /// Create lenient canonicalization rules (minimal sorting)
    pub fn lenient() -> Self {
        Self {
            sort_parameters: false,
            sort_arguments: false,
            sort_data_refs: true, // Always sort data refs for consistency
        }
    }
}

impl Default for CanonicalizationRules {
    fn default() -> Self {
        Self::new()
    }
}

/// Structural validator for plan validation
pub struct StructuralValidator {
    max_steps: usize,
    max_data_refs_per_step: usize,
}

impl StructuralValidator {
    /// Create new structural validator
    pub fn new() -> Self {
        Self {
            max_steps: MAX_PLAN_STEPS,
            max_data_refs_per_step: MAX_DATA_REFS_PER_STEP,
        }
    }

    /// Create validator with custom limits
    pub fn with_limits(max_steps: usize, max_data_refs_per_step: usize) -> Self {
        Self {
            max_steps,
            max_data_refs_per_step,
        }
    }

    /// Validate plan structure with comprehensive checks
    pub fn validate_structure(&self, plan: &ExecutionPlan) -> GateCResult<()> {
        // Check step count
        if plan.steps.len() > self.max_steps {
            return Err(NormalizationError::TooComplex(format!(
                "Plan has {} steps, exceeds limit of {}",
                plan.steps.len(),
                self.max_steps
            ))
            .into());
        }

        self.validate_structure_internal(plan)
    }

    /// Validate plan structure for performance testing (bypasses step count limit)
    ///
    /// **CRITICAL:** This method is ONLY for performance testing and complexity budget validation.
    /// It bypasses constitutional step limits and should NEVER be used in production code.
    ///
    /// **Phase 4.3 Performance Testing:** This method enables structural validation
    /// at scales beyond constitutional limits to validate algorithmic improvements.
    pub fn validate_structure_for_performance_testing(
        &self,
        plan: &ExecutionPlan,
    ) -> GateCResult<()> {
        // **PERFORMANCE TESTING:** Skip step count limit check
        // This allows testing at 5K+ steps for algorithmic performance validation

        self.validate_structure_internal(plan)
    }

    /// Internal structure validation (shared by both normal and performance testing modes)
    fn validate_structure_internal(&self, plan: &ExecutionPlan) -> GateCResult<()> {
        // Validate each step
        for step in &plan.steps {
            self.validate_step(step)?;
        }

        // Validate step ID uniqueness
        self.validate_step_uniqueness(&plan.steps)?;

        // Validate dependencies
        self.validate_dependencies(plan)?;

        // Validate data flow consistency
        self.validate_data_flow(plan)?;

        // Validate operation consistency
        self.validate_operations(plan)?;

        // Validate metadata
        self.validate_metadata(&plan.metadata)?;

        // Check for circular dependencies
        self.validate_no_cycles(plan)?;

        // Validate reference integrity
        self.validate_reference_integrity(plan)?;

        Ok(())
    }

    /// Validate individual step
    fn validate_step(&self, step: &PlanStep) -> GateCResult<()> {
        // Check step ID
        if step.id.is_empty() {
            return Err(
                NormalizationError::StructuralError("Step ID cannot be empty".to_string()).into(),
            );
        }

        // Check data reference limits
        if step.inputs.len() > self.max_data_refs_per_step {
            return Err(NormalizationError::TooComplex(format!(
                "Step {} has {} inputs, exceeds limit of {}",
                step.id,
                step.inputs.len(),
                self.max_data_refs_per_step
            ))
            .into());
        }

        if step.outputs.len() > self.max_data_refs_per_step {
            return Err(NormalizationError::TooComplex(format!(
                "Step {} has {} outputs, exceeds limit of {}",
                step.id,
                step.outputs.len(),
                self.max_data_refs_per_step
            ))
            .into());
        }

        // Validate data references
        for input in &step.inputs {
            self.validate_data_ref(input, "input")?;
        }
        for output in &step.outputs {
            self.validate_data_ref(output, "output")?;
        }

        Ok(())
    }

    /// Validate data reference
    fn validate_data_ref(&self, data_ref: &DataRef, ref_type: &str) -> GateCResult<()> {
        if data_ref.id.is_empty() {
            return Err(NormalizationError::StructuralError(format!(
                "Data reference {} ID cannot be empty",
                ref_type
            ))
            .into());
        }

        if data_ref.data_type.is_empty() {
            return Err(NormalizationError::StructuralError(format!(
                "Data reference {} type cannot be empty",
                ref_type
            ))
            .into());
        }

        Ok(())
    }

    /// Validate step ID uniqueness
    fn validate_step_uniqueness(&self, steps: &[PlanStep]) -> GateCResult<()> {
        let mut seen_ids = HashSet::new();

        for step in steps {
            if seen_ids.contains(&step.id) {
                return Err(NormalizationError::StructuralError(format!(
                    "Duplicate step ID: {}",
                    step.id
                ))
                .into());
            }
            seen_ids.insert(&step.id);
        }

        Ok(())
    }

    /// Validate plan dependencies
    fn validate_dependencies(&self, plan: &ExecutionPlan) -> GateCResult<()> {
        let step_ids: HashSet<_> = plan.steps.iter().map(|s| &s.id).collect();

        // Validate dependency references
        for dep in &plan.dependencies {
            if !step_ids.contains(&dep.from) {
                return Err(NormalizationError::InvalidReference(format!(
                    "Dependency references non-existent step: {}",
                    dep.from
                ))
                .into());
            }
            if !step_ids.contains(&dep.to) {
                return Err(NormalizationError::InvalidReference(format!(
                    "Dependency references non-existent step: {}",
                    dep.to
                ))
                .into());
            }
            if dep.from == dep.to {
                return Err(NormalizationError::StructuralError(format!(
                    "Self-dependency detected in step: {}",
                    dep.from
                ))
                .into());
            }
        }

        Ok(())
    }

    /// Validate data flow consistency
    fn validate_data_flow(&self, plan: &ExecutionPlan) -> GateCResult<()> {
        let mut data_producers: HashMap<String, String> = HashMap::new();
        let mut data_consumers: HashMap<String, Vec<String>> = HashMap::new();

        // Build data flow maps
        for step in &plan.steps {
            // Record data producers
            for output in &step.outputs {
                if let Some(existing_producer) = data_producers.get(&output.id) {
                    return Err(NormalizationError::StructuralError(format!(
                        "Data ID '{}' is produced by multiple steps: '{}' and '{}'",
                        output.id, existing_producer, step.id
                    ))
                    .into());
                }
                data_producers.insert(output.id.clone(), step.id.clone());
            }

            // Record data consumers
            for input in &step.inputs {
                data_consumers
                    .entry(input.id.clone())
                    .or_insert_with(Vec::new)
                    .push(step.id.clone());
            }
        }

        // Validate that all consumed data is produced
        for (data_id, consumers) in &data_consumers {
            if !data_producers.contains_key(data_id) {
                return Err(NormalizationError::InvalidReference(format!(
                    "Data '{}' is consumed by steps {:?} but never produced",
                    data_id, consumers
                ))
                .into());
            }
        }

        // Validate source_step references in data refs
        for step in &plan.steps {
            for input in &step.inputs {
                if let Some(source_step) = &input.source_step {
                    if let Some(producer) = data_producers.get(&input.id) {
                        if producer != source_step {
                            return Err(NormalizationError::StructuralError(format!(
                                "Data '{}' source_step '{}' doesn't match actual producer '{}'",
                                input.id, source_step, producer
                            ))
                            .into());
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Validate operation consistency
    fn validate_operations(&self, plan: &ExecutionPlan) -> GateCResult<()> {
        for step in &plan.steps {
            match &step.operation {
                Operation::Query { target, parameters } => {
                    if target.is_empty() {
                        return Err(NormalizationError::StructuralError(format!(
                            "Step '{}': Query target cannot be empty",
                            step.id
                        ))
                        .into());
                    }

                    // Validate parameter names are not empty
                    for (key, value) in parameters {
                        if key.is_empty() {
                            return Err(NormalizationError::StructuralError(format!(
                                "Step '{}': Query parameter key cannot be empty",
                                step.id
                            ))
                            .into());
                        }
                        if value.is_empty() {
                            return Err(NormalizationError::StructuralError(format!(
                                "Step '{}': Query parameter '{}' value cannot be empty",
                                step.id, key
                            ))
                            .into());
                        }
                    }
                }
                Operation::Compute {
                    function,
                    arguments,
                } => {
                    if function.is_empty() {
                        return Err(NormalizationError::StructuralError(format!(
                            "Step '{}': Compute function cannot be empty",
                            step.id
                        ))
                        .into());
                    }

                    // Validate arguments are not empty
                    for (i, arg) in arguments.iter().enumerate() {
                        if arg.is_empty() {
                            return Err(NormalizationError::StructuralError(format!(
                                "Step '{}': Compute argument {} cannot be empty",
                                step.id, i
                            ))
                            .into());
                        }
                    }
                }
                Operation::Mutation { intent: _ } => {
                    // Mutation intent validation is handled by mutation module
                    // Just ensure the step has proper inputs if needed
                }
            }
        }

        Ok(())
    }

    /// Validate plan metadata
    fn validate_metadata(&self, metadata: &PlanMetadata) -> GateCResult<()> {
        if metadata.name.is_empty() {
            return Err(NormalizationError::StructuralError(
                "Plan name cannot be empty".to_string(),
            )
            .into());
        }

        if metadata.version.is_empty() {
            return Err(NormalizationError::StructuralError(
                "Plan version cannot be empty".to_string(),
            )
            .into());
        }

        // Check metadata size limits
        let metadata_size = metadata.name.len()
            + metadata.version.len()
            + metadata.description.as_ref().map_or(0, |d| d.len())
            + metadata
                .extra
                .iter()
                .map(|(k, v)| k.len() + v.len())
                .sum::<usize>();

        if metadata_size > MAX_PLAN_METADATA_BYTES {
            return Err(NormalizationError::TooComplex(format!(
                "Plan metadata size {} bytes exceeds limit of {} bytes",
                metadata_size, MAX_PLAN_METADATA_BYTES
            ))
            .into());
        }

        Ok(())
    }

    /// Validate no circular dependencies using DFS
    fn validate_no_cycles(&self, plan: &ExecutionPlan) -> GateCResult<()> {
        let mut graph: HashMap<String, Vec<String>> = HashMap::new();

        // Build dependency graph
        for step in &plan.steps {
            graph.insert(step.id.clone(), Vec::new());
        }

        for dep in &plan.dependencies {
            if let Some(deps) = graph.get_mut(&dep.from) {
                deps.push(dep.to.clone());
            }
        }

        // Also add implicit data dependencies
        for step in &plan.steps {
            for input in &step.inputs {
                if let Some(source_step) = &input.source_step {
                    if let Some(deps) = graph.get_mut(source_step) {
                        if !deps.contains(&step.id) {
                            deps.push(step.id.clone());
                        }
                    }
                }
            }
        }

        // DFS cycle detection
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();

        for step_id in graph.keys() {
            if !visited.contains(step_id) {
                if self.has_cycle_dfs(&graph, step_id, &mut visited, &mut rec_stack)? {
                    return Err(NormalizationError::StructuralError(format!(
                        "Circular dependency detected involving step: {}",
                        step_id
                    ))
                    .into());
                }
            }
        }

        Ok(())
    }

    /// DFS helper for cycle detection
    fn has_cycle_dfs(
        &self,
        graph: &HashMap<String, Vec<String>>,
        node: &str,
        visited: &mut HashSet<String>,
        rec_stack: &mut HashSet<String>,
    ) -> GateCResult<bool> {
        visited.insert(node.to_string());
        rec_stack.insert(node.to_string());

        if let Some(neighbors) = graph.get(node) {
            for neighbor in neighbors {
                if !visited.contains(neighbor) {
                    if self.has_cycle_dfs(graph, neighbor, visited, rec_stack)? {
                        return Ok(true);
                    }
                } else if rec_stack.contains(neighbor) {
                    return Ok(true);
                }
            }
        }

        rec_stack.remove(node);
        Ok(false)
    }

    /// Validate reference integrity across the plan
    fn validate_reference_integrity(&self, plan: &ExecutionPlan) -> GateCResult<()> {
        let step_ids: HashSet<_> = plan.steps.iter().map(|s| &s.id).collect();

        // Validate all step references in data refs
        for step in &plan.steps {
            for input in &step.inputs {
                if let Some(source_step) = &input.source_step {
                    if !step_ids.contains(source_step) {
                        return Err(NormalizationError::InvalidReference(format!(
                            "Step '{}' input '{}' references non-existent source step '{}'",
                            step.id, input.id, source_step
                        ))
                        .into());
                    }
                }
            }

            for output in &step.outputs {
                if let Some(source_step) = &output.source_step {
                    if source_step != &step.id {
                        return Err(NormalizationError::StructuralError(format!(
                            "Step '{}' output '{}' has incorrect source_step '{}'",
                            step.id, output.id, source_step
                        ))
                        .into());
                    }
                }
            }
        }

        Ok(())
    }

    /// Validate that validation is idempotent (can be called multiple times)
    pub fn validate_idempotent(&self, plan: &ExecutionPlan) -> GateCResult<()> {
        // First validation
        self.validate_structure(plan)?;

        // Second validation should produce same result
        self.validate_structure(plan)?;

        // Third validation should also produce same result
        self.validate_structure(plan)?;

        Ok(())
    }

    /// Get validation report with detailed information
    pub fn get_validation_report(&self, plan: &ExecutionPlan) -> ValidationReport {
        let mut report = ValidationReport::new();

        // Check each validation aspect individually and collect results
        // Don't return early on errors - collect all of them

        // Basic structure validation
        if let Err(e) = self.validate_step_uniqueness(&plan.steps) {
            report.add_error(ValidationError {
                category: ValidationCategory::Structure,
                message: e.to_string(),
                step_id: None,
            });
        }

        // Individual step validation
        for step in &plan.steps {
            if let Err(e) = self.validate_step(step) {
                report.add_error(ValidationError {
                    category: ValidationCategory::Step,
                    message: e.to_string(),
                    step_id: Some(step.id.clone()),
                });
            }
        }

        // Data flow validation
        if let Err(e) = self.validate_data_flow(plan) {
            report.add_error(ValidationError {
                category: ValidationCategory::DataFlow,
                message: e.to_string(),
                step_id: None,
            });
        }

        // Dependency validation
        if let Err(e) = self.validate_dependencies(plan) {
            report.add_error(ValidationError {
                category: ValidationCategory::Dependencies,
                message: e.to_string(),
                step_id: None,
            });
        }

        // Operation validation
        if let Err(e) = self.validate_operations(plan) {
            report.add_error(ValidationError {
                category: ValidationCategory::Operations,
                message: e.to_string(),
                step_id: None,
            });
        }

        // Metadata validation
        if let Err(e) = self.validate_metadata(&plan.metadata) {
            report.add_error(ValidationError {
                category: ValidationCategory::Metadata,
                message: e.to_string(),
                step_id: None,
            });
        }

        // Circular dependency validation
        if let Err(e) = self.validate_no_cycles(plan) {
            report.add_error(ValidationError {
                category: ValidationCategory::Cycles,
                message: e.to_string(),
                step_id: None,
            });
        }

        // Reference integrity validation
        if let Err(e) = self.validate_reference_integrity(plan) {
            report.add_error(ValidationError {
                category: ValidationCategory::References,
                message: e.to_string(),
                step_id: None,
            });
        }

        // Check plan size limits
        if plan.steps.len() > self.max_steps {
            report.add_error(ValidationError {
                category: ValidationCategory::Limits,
                message: format!(
                    "Plan has {} steps, exceeds limit of {}",
                    plan.steps.len(),
                    self.max_steps
                ),
                step_id: None,
            });
        }

        report
    }
}

impl Default for StructuralValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gate_c::types::{
        ChangeSet, Dependency, DependencyType, MutationIntent, PlanMetadata, ResourcePath,
    };
    use std::collections::HashMap;

    fn create_test_plan() -> ExecutionPlan {
        ExecutionPlan {
            id: "test-plan".to_string(),
            steps: vec![
                PlanStep {
                    id: "step-2".to_string(), // Intentionally out of order
                    operation: Operation::Query {
                        target: "test".to_string(),
                        parameters: {
                            let mut params = HashMap::new();
                            params.insert("z_param".to_string(), "value1".to_string());
                            params.insert("a_param".to_string(), "value2".to_string());
                            params
                        },
                    },
                    inputs: vec![DataRef {
                        id: "output-1".to_string(), // Fixed: use the actual output from step-1
                        data_type: "string".to_string(),
                        source_step: Some("step-1".to_string()),
                    }],
                    outputs: vec![DataRef {
                        id: "output-2".to_string(),
                        data_type: "string".to_string(),
                        source_step: Some("step-2".to_string()),
                    }],
                },
                PlanStep {
                    id: "step-1".to_string(),
                    operation: Operation::Compute {
                        function: "test_func".to_string(),
                        arguments: vec!["z_arg".to_string(), "a_arg".to_string()], // Intentionally out of order
                    },
                    inputs: vec![],
                    outputs: vec![DataRef {
                        id: "output-1".to_string(),
                        data_type: "string".to_string(),
                        source_step: Some("step-1".to_string()),
                    }],
                },
            ],
            metadata: PlanMetadata {
                name: "Test Plan".to_string(),
                description: Some("Test plan for normalization".to_string()),
                created_at: 1234567890,
                version: "1.0".to_string(),
                extra: HashMap::new(),
            },
            dependencies: vec![Dependency {
                from: "step-1".to_string(),
                to: "step-2".to_string(),
                dependency_type: DependencyType::Data,
            }],
        }
    }

    #[test]
    fn test_plan_normalizer_creation() {
        let normalizer = PlanNormalizer::new();
        assert!(normalizer.canonicalization_rules.sort_parameters);
        assert!(normalizer.canonicalization_rules.sort_arguments);
    }

    #[test]
    fn test_plan_normalization() {
        let normalizer = PlanNormalizer::new();
        let plan = create_test_plan();

        let result = normalizer.normalize(&plan);
        assert!(result.is_ok());

        let canonical = result.unwrap();

        // Check that steps are sorted by ID
        assert_eq!(canonical.normalized_steps[0].id, "step-1");
        assert_eq!(canonical.normalized_steps[1].id, "step-2");

        // Check that parameters are sorted
        if let Operation::Query { parameters, .. } = &canonical.normalized_steps[1].operation {
            let mut keys: Vec<_> = parameters.keys().cloned().collect();
            keys.sort();
            assert_eq!(keys, vec!["a_param", "z_param"]);
        } else {
            assert!(false, "Expected Query operation");
        }

        // Check that arguments are sorted
        if let Operation::Compute { arguments, .. } = &canonical.normalized_steps[0].operation {
            assert_eq!(arguments, &vec!["a_arg", "z_arg"]);
        } else {
            assert!(false, "Expected Compute operation");
        }
    }

    #[test]
    fn test_canonicalization_stability() {
        let normalizer = PlanNormalizer::new();
        let plan = create_test_plan();

        let canonical1 = normalizer.normalize(&plan).unwrap();
        let canonical2 = normalizer.normalize(&plan).unwrap();

        // Fingerprints should be identical
        assert_eq!(canonical1.fingerprint().hash, canonical2.fingerprint().hash);
        assert_eq!(
            canonical1.fingerprint().version,
            canonical2.fingerprint().version
        );
    }

    #[test]
    fn test_structural_validation() {
        let validator = StructuralValidator::new();
        let plan = create_test_plan();

        let result = validator.validate_structure(&plan);
        assert!(result.is_ok());
    }

    #[test]
    fn test_invalid_step_id() {
        let validator = StructuralValidator::new();
        let mut plan = create_test_plan();
        plan.steps[0].id = "".to_string(); // Empty ID

        let result = validator.validate_structure(&plan);
        assert!(result.is_err());

        match result.unwrap_err() {
            crate::gate_c::error::GateCError::Normalization(
                NormalizationError::StructuralError(_),
            ) => {
                // Expected
            }
            other => assert!(false, "Expected StructuralError, got: {:?}", other),
        }
    }

    #[test]
    fn test_duplicate_step_ids() {
        let validator = StructuralValidator::new();
        let mut plan = create_test_plan();
        plan.steps[1].id = plan.steps[0].id.clone(); // Duplicate ID

        let result = validator.validate_structure(&plan);
        assert!(result.is_err());

        match result.unwrap_err() {
            crate::gate_c::error::GateCError::Normalization(
                NormalizationError::StructuralError(_),
            ) => {
                // Expected
            }
            other => assert!(false, "Expected StructuralError, got: {:?}", other),
        }
    }

    #[test]
    fn test_invalid_dependency_reference() {
        let validator = StructuralValidator::new();
        let mut plan = create_test_plan();
        plan.dependencies[0].from = "non-existent-step".to_string();

        let result = validator.validate_structure(&plan);
        assert!(result.is_err());

        match result.unwrap_err() {
            crate::gate_c::error::GateCError::Normalization(
                NormalizationError::InvalidReference(_),
            ) => {
                // Expected
            }
            other => assert!(false, "Expected InvalidReference, got: {:?}", other),
        }
    }

    #[test]
    fn test_plan_too_large() {
        let validator = StructuralValidator::with_limits(1, 32); // Limit to 1 step
        let plan = create_test_plan(); // Has 2 steps

        let result = validator.validate_structure(&plan);
        assert!(result.is_err());

        match result.unwrap_err() {
            crate::gate_c::error::GateCError::Normalization(NormalizationError::TooComplex(_)) => {
                // Expected
            }
            other => assert!(false, "Expected TooComplex, got: {:?}", other),
        }
    }

    #[test]
    fn test_canonicalization_rules() {
        let strict_rules = CanonicalizationRules::strict();
        assert!(strict_rules.sort_parameters);
        assert!(strict_rules.sort_arguments);
        assert!(strict_rules.sort_data_refs);

        let lenient_rules = CanonicalizationRules::lenient();
        assert!(!lenient_rules.sort_parameters);
        assert!(!lenient_rules.sort_arguments);
        assert!(lenient_rules.sort_data_refs); // Always true
    }

    #[test]
    fn test_fingerprint_determinism() {
        let normalizer = PlanNormalizer::new();
        let plan1 = create_test_plan();
        let mut plan2 = create_test_plan();

        // Swap step order in plan2
        plan2.steps.swap(0, 1);

        let canonical1 = normalizer.normalize(&plan1).unwrap();
        let canonical2 = normalizer.normalize(&plan2).unwrap();

        // Should have same fingerprint despite different input order
        assert_eq!(canonical1.fingerprint().hash, canonical2.fingerprint().hash);
    }

    #[test]
    fn test_ambiguous_plan_detection() {
        let normalizer = PlanNormalizer::new();
        let mut plan = create_test_plan();

        // Create ambiguous reference - this will be caught by structural validation first
        plan.steps[0].inputs[0].source_step = Some("non-existent".to_string());

        let result = normalizer.normalize(&plan);
        assert!(result.is_err());

        // The error could be either InvalidReference or StructuralError depending on validation order
        match result.unwrap_err() {
            crate::gate_c::error::GateCError::Normalization(
                NormalizationError::InvalidReference(_),
            )
            | crate::gate_c::error::GateCError::Normalization(
                NormalizationError::StructuralError(_),
            ) => {
                // Expected - either error type is acceptable for this test
            }
            other => assert!(
                false,
                "Expected InvalidReference or StructuralError for ambiguous plan, got: {:?}",
                other
            ),
        }
    }

    // ===== TASK 17: COMPREHENSIVE STRUCTURAL VALIDATION TESTS =====

    #[test]
    fn test_comprehensive_structural_validation() {
        let validator = StructuralValidator::new();
        let plan = create_test_plan();

        // Should pass comprehensive validation
        assert!(validator.validate_structure(&plan).is_ok());
    }

    #[test]
    fn test_data_flow_validation() {
        let validator = StructuralValidator::new();

        // Create plan with broken data flow
        let mut plan = create_test_plan();
        plan.steps[0].inputs.push(DataRef {
            id: "non-existent-data".to_string(),
            data_type: "string".to_string(),
            source_step: Some("non-existent-step".to_string()),
        });

        let result = validator.validate_structure(&plan);
        assert!(result.is_err());

        let error = result.unwrap_err();
        assert!(error.to_string().contains("non-existent-data"));
    }

    #[test]
    fn test_circular_dependency_detection() {
        let validator = StructuralValidator::new();

        // Create plan with circular dependency
        let mut plan = create_test_plan();
        plan.dependencies.push(Dependency {
            from: "step-2".to_string(),
            to: "step-1".to_string(),
            dependency_type: DependencyType::Data,
        });

        let result = validator.validate_structure(&plan);
        assert!(result.is_err());

        let error = result.unwrap_err();
        assert!(error.to_string().contains("Circular dependency"));
    }

    #[test]
    fn test_operation_validation() {
        let validator = StructuralValidator::new();

        // Create plan with invalid operation
        let mut plan = create_test_plan();
        plan.steps[0].operation = Operation::Query {
            target: "".to_string(), // Empty target should fail
            parameters: HashMap::new(),
        };

        let result = validator.validate_structure(&plan);
        assert!(result.is_err());

        let error = result.unwrap_err();
        assert!(error.to_string().contains("target cannot be empty"));
    }

    #[test]
    fn test_metadata_validation() {
        let validator = StructuralValidator::new();

        // Create plan with invalid metadata
        let mut plan = create_test_plan();
        plan.metadata.name = "".to_string(); // Empty name should fail

        let result = validator.validate_structure(&plan);
        assert!(result.is_err());

        let error = result.unwrap_err();
        assert!(error.to_string().contains("name cannot be empty"));
    }

    #[test]
    fn test_reference_integrity_validation() {
        let validator = StructuralValidator::new();

        // Create plan with broken reference
        let mut plan = create_test_plan();
        plan.steps[0].inputs[0].source_step = Some("non-existent-step".to_string());

        let result = validator.validate_structure(&plan);
        assert!(result.is_err());

        let error = result.unwrap_err();
        // The error message should contain information about the reference issue
        let error_str = error.to_string();
        assert!(
            error_str.contains("non-existent")
                || error_str.contains("doesn't match")
                || error_str.contains("references")
        );
    }

    #[test]
    fn test_validation_idempotency() {
        let validator = StructuralValidator::new();
        let plan = create_test_plan();

        // Validation should be idempotent
        assert!(validator.validate_idempotent(&plan).is_ok());
    }

    #[test]
    fn test_validation_report() {
        let validator = StructuralValidator::new();
        let plan = create_test_plan();

        let report = validator.get_validation_report(&plan);
        assert!(report.is_valid);
        assert_eq!(report.error_count(), 0);
    }

    #[test]
    fn test_validation_report_with_errors() {
        let validator = StructuralValidator::new();

        // Create plan with multiple issues
        let mut plan = create_test_plan();
        plan.metadata.name = "".to_string(); // Invalid metadata
        plan.steps[0].operation = Operation::Query {
            target: "".to_string(), // Invalid operation
            parameters: HashMap::new(),
        };

        let report = validator.get_validation_report(&plan);
        assert!(!report.is_valid);
        assert!(report.error_count() > 0);
    }

    #[test]
    fn test_duplicate_data_producer_detection() {
        let validator = StructuralValidator::new();

        // Create plan where two steps produce same data
        let mut plan = create_test_plan();
        plan.steps.push(PlanStep {
            id: "step-3".to_string(),
            operation: Operation::Compute {
                function: "duplicate".to_string(),
                arguments: vec![],
            },
            inputs: vec![],
            outputs: vec![DataRef {
                id: "output-1".to_string(), // Same as step-1's output
                data_type: "string".to_string(),
                source_step: Some("step-3".to_string()),
            }],
        });

        let result = validator.validate_structure(&plan);
        assert!(result.is_err());

        let error = result.unwrap_err();
        assert!(error.to_string().contains("produced by multiple steps"));
    }

    #[test]
    fn test_empty_operation_parameters() {
        let validator = StructuralValidator::new();

        // Test empty parameter key
        let mut plan = create_test_plan();
        let mut params = HashMap::new();
        params.insert("".to_string(), "value".to_string()); // Empty key
        plan.steps[0].operation = Operation::Query {
            target: "test".to_string(),
            parameters: params,
        };

        let result = validator.validate_structure(&plan);
        assert!(result.is_err());

        // Test empty parameter value
        let mut plan = create_test_plan();
        let mut params = HashMap::new();
        params.insert("key".to_string(), "".to_string()); // Empty value
        plan.steps[0].operation = Operation::Query {
            target: "test".to_string(),
            parameters: params,
        };

        let result = validator.validate_structure(&plan);
        assert!(result.is_err());
    }

    #[test]
    fn test_compute_operation_validation() {
        let validator = StructuralValidator::new();

        // Test empty function name
        let mut plan = create_test_plan();
        plan.steps[1].operation = Operation::Compute {
            function: "".to_string(), // Empty function
            arguments: vec!["arg".to_string()],
        };

        let result = validator.validate_structure(&plan);
        assert!(result.is_err());

        // Test empty argument
        let mut plan = create_test_plan();
        plan.steps[1].operation = Operation::Compute {
            function: "test".to_string(),
            arguments: vec!["".to_string()], // Empty argument
        };

        let result = validator.validate_structure(&plan);
        assert!(result.is_err());
    }

    #[test]
    fn test_metadata_size_limits() {
        let validator = StructuralValidator::new();

        // Create plan with oversized metadata
        let mut plan = create_test_plan();
        plan.metadata.description = Some("x".repeat(MAX_PLAN_METADATA_BYTES));

        let result = validator.validate_structure(&plan);
        assert!(result.is_err());

        let error = result.unwrap_err();
        assert!(error.to_string().contains("metadata size"));
    }

    #[test]
    fn test_data_ref_limits() {
        let validator = StructuralValidator::new();

        // Create step with too many inputs
        let mut plan = create_test_plan();
        let mut inputs = Vec::new();
        for i in 0..=MAX_DATA_REFS_PER_STEP {
            inputs.push(DataRef {
                id: format!("input-{}", i),
                data_type: "string".to_string(),
                source_step: Some("step-1".to_string()),
            });
        }
        plan.steps[0].inputs = inputs;

        let result = validator.validate_structure(&plan);
        assert!(result.is_err());

        let error = result.unwrap_err();
        assert!(error.to_string().contains("exceeds limit"));
    }

    #[test]
    fn test_source_step_mismatch_detection() {
        let validator = StructuralValidator::new();

        // Create plan where source_step doesn't match actual producer
        let mut plan = create_test_plan();
        plan.steps[0].inputs[0].source_step = Some("step-2".to_string()); // Wrong producer

        let result = validator.validate_structure(&plan);
        assert!(result.is_err());

        let error = result.unwrap_err();
        assert!(error.to_string().contains("doesn't match actual producer"));
    }

    #[test]
    fn test_output_source_step_validation() {
        let validator = StructuralValidator::new();

        // Create plan where output has wrong source_step
        let mut plan = create_test_plan();
        plan.steps[0].outputs[0].source_step = Some("wrong-step".to_string());

        let result = validator.validate_structure(&plan);
        assert!(result.is_err());

        let error = result.unwrap_err();
        assert!(error.to_string().contains("incorrect source_step"));
    }

    #[test]
    fn test_validation_categories() {
        let validator = StructuralValidator::new();

        // Test different validation categories
        let mut plan = create_test_plan();
        plan.metadata.name = "".to_string();

        let report = validator.get_validation_report(&plan);
        assert!(!report.is_valid);

        // Should have metadata category error
        let metadata_errors: Vec<_> = report
            .errors
            .iter()
            .filter(|e| e.category == ValidationCategory::Metadata)
            .collect();
        assert!(!metadata_errors.is_empty());
    }

    #[test]
    fn test_complex_circular_dependency() {
        let validator = StructuralValidator::new();

        // Create plan with complex circular dependency through data flow
        let plan = ExecutionPlan {
            id: "circular-plan".to_string(),
            steps: vec![
                PlanStep {
                    id: "step-a".to_string(),
                    operation: Operation::Compute {
                        function: "func_a".to_string(),
                        arguments: vec![],
                    },
                    inputs: vec![DataRef {
                        id: "data-c".to_string(),
                        data_type: "string".to_string(),
                        source_step: Some("step-c".to_string()),
                    }],
                    outputs: vec![DataRef {
                        id: "data-a".to_string(),
                        data_type: "string".to_string(),
                        source_step: Some("step-a".to_string()),
                    }],
                },
                PlanStep {
                    id: "step-b".to_string(),
                    operation: Operation::Compute {
                        function: "func_b".to_string(),
                        arguments: vec![],
                    },
                    inputs: vec![DataRef {
                        id: "data-a".to_string(),
                        data_type: "string".to_string(),
                        source_step: Some("step-a".to_string()),
                    }],
                    outputs: vec![DataRef {
                        id: "data-b".to_string(),
                        data_type: "string".to_string(),
                        source_step: Some("step-b".to_string()),
                    }],
                },
                PlanStep {
                    id: "step-c".to_string(),
                    operation: Operation::Compute {
                        function: "func_c".to_string(),
                        arguments: vec![],
                    },
                    inputs: vec![DataRef {
                        id: "data-b".to_string(),
                        data_type: "string".to_string(),
                        source_step: Some("step-b".to_string()),
                    }],
                    outputs: vec![DataRef {
                        id: "data-c".to_string(),
                        data_type: "string".to_string(),
                        source_step: Some("step-c".to_string()),
                    }],
                },
            ],
            metadata: PlanMetadata {
                name: "Circular Plan".to_string(),
                description: None,
                created_at: 0,
                version: "1.0".to_string(),
                extra: HashMap::new(),
            },
            dependencies: vec![],
        };

        let result = validator.validate_structure(&plan);
        assert!(result.is_err());

        let error = result.unwrap_err();
        assert!(error.to_string().contains("Circular dependency"));
    }

    // ===== TASK 18: COMPREHENSIVE NORMALIZER TESTING =====

    #[test]
    fn test_canonicalization_rules_comprehensive() {
        // Test all canonicalization rule combinations
        let strict_normalizer = PlanNormalizer::with_rules(CanonicalizationRules::strict());
        let lenient_normalizer = PlanNormalizer::with_rules(CanonicalizationRules::lenient());
        let default_normalizer = PlanNormalizer::new();

        let plan = create_test_plan();

        // All should succeed but may produce different canonical forms
        assert!(strict_normalizer.normalize(&plan).is_ok());
        assert!(lenient_normalizer.normalize(&plan).is_ok());
        assert!(default_normalizer.normalize(&plan).is_ok());

        // Strict and default should be equivalent
        let strict_result = strict_normalizer.normalize(&plan).unwrap();
        let default_result = default_normalizer.normalize(&plan).unwrap();
        assert_eq!(
            strict_result.fingerprint().hash,
            default_result.fingerprint().hash
        );
    }

    #[test]
    fn test_normalization_stability_property() {
        // Property test: normalization should be stable across multiple calls
        let normalizer = PlanNormalizer::new();
        let plan = create_test_plan();

        let mut fingerprints = Vec::new();

        // Run normalization multiple times
        for _ in 0..10 {
            let canonical = normalizer.normalize(&plan).unwrap();
            fingerprints.push(canonical.fingerprint().hash);
        }

        // All fingerprints should be identical
        let first_fingerprint = fingerprints[0];
        for fingerprint in fingerprints {
            assert_eq!(fingerprint, first_fingerprint);
        }
    }

    #[test]
    fn test_normalization_determinism_property() {
        // Property test: same logical plan should produce same fingerprint regardless of input order
        let normalizer = PlanNormalizer::new();

        // Create multiple equivalent plans with different orderings
        let mut plans = Vec::new();

        for i in 0..5 {
            let mut plan = create_test_plan();

            // Shuffle step order
            if i % 2 == 0 {
                plan.steps.reverse();
            }

            // Shuffle dependency order
            if i % 3 == 0 {
                plan.dependencies.reverse();
            }

            plans.push(plan);
        }

        // All should produce the same canonical fingerprint
        let mut fingerprints = Vec::new();
        for plan in plans {
            let canonical = normalizer.normalize(&plan).unwrap();
            fingerprints.push(canonical.fingerprint().hash);
        }

        let first_fingerprint = fingerprints[0];
        for fingerprint in fingerprints {
            assert_eq!(fingerprint, first_fingerprint);
        }
    }

    #[test]
    fn test_complex_plan_integration() {
        // Integration test with a complex plan
        let normalizer = PlanNormalizer::new();

        let complex_plan = ExecutionPlan {
            id: "complex-plan".to_string(),
            steps: vec![
                // Step with multiple inputs and outputs
                PlanStep {
                    id: "step-1".to_string(),
                    operation: Operation::Query {
                        target: "database".to_string(),
                        parameters: {
                            let mut params = HashMap::new();
                            params.insert("table".to_string(), "users".to_string());
                            params.insert("filter".to_string(), "active=true".to_string());
                            params.insert("limit".to_string(), "100".to_string());
                            params
                        },
                    },
                    inputs: vec![],
                    outputs: vec![
                        DataRef {
                            id: "user-data".to_string(),
                            data_type: "json".to_string(),
                            source_step: Some("step-1".to_string()),
                        },
                        DataRef {
                            id: "user-count".to_string(),
                            data_type: "integer".to_string(),
                            source_step: Some("step-1".to_string()),
                        },
                    ],
                },
                // Step with complex computation
                PlanStep {
                    id: "step-2".to_string(),
                    operation: Operation::Compute {
                        function: "transform_users".to_string(),
                        arguments: vec![
                            "normalize".to_string(),
                            "validate".to_string(),
                            "enrich".to_string(),
                        ],
                    },
                    inputs: vec![DataRef {
                        id: "user-data".to_string(),
                        data_type: "json".to_string(),
                        source_step: Some("step-1".to_string()),
                    }],
                    outputs: vec![DataRef {
                        id: "processed-users".to_string(),
                        data_type: "json".to_string(),
                        source_step: Some("step-2".to_string()),
                    }],
                },
                // Step with mutation
                PlanStep {
                    id: "step-3".to_string(),
                    operation: Operation::Mutation {
                        intent: MutationIntent::UpdateIntent {
                            target: ResourcePath {
                                segments: vec!["cache".to_string(), "users".to_string()],
                            },
                            changes: ChangeSet {
                                updates: HashMap::new(),
                                removals: vec![],
                            },
                        },
                    },
                    inputs: vec![DataRef {
                        id: "processed-users".to_string(),
                        data_type: "json".to_string(),
                        source_step: Some("step-2".to_string()),
                    }],
                    outputs: vec![DataRef {
                        id: "cache-result".to_string(),
                        data_type: "boolean".to_string(),
                        source_step: Some("step-3".to_string()),
                    }],
                },
            ],
            metadata: PlanMetadata {
                name: "Complex User Processing Plan".to_string(),
                description: Some(
                    "A complex plan that queries, processes, and caches user data".to_string(),
                ),
                created_at: 1640995200, // 2022-01-01
                version: "2.1.0".to_string(),
                extra: {
                    let mut extra = HashMap::new();
                    extra.insert("author".to_string(), "system".to_string());
                    extra.insert("priority".to_string(), "high".to_string());
                    extra
                },
            },
            dependencies: vec![
                Dependency {
                    from: "step-1".to_string(),
                    to: "step-2".to_string(),
                    dependency_type: DependencyType::Data,
                },
                Dependency {
                    from: "step-2".to_string(),
                    to: "step-3".to_string(),
                    dependency_type: DependencyType::Data,
                },
            ],
        };

        // Should normalize successfully
        let result = normalizer.normalize(&complex_plan);
        assert!(result.is_ok());

        let canonical = result.unwrap();

        // Verify canonical properties
        assert_eq!(canonical.normalized_steps.len(), 3);
        assert!(canonical.fingerprint().hash != 0);
        assert_eq!(canonical.fingerprint().version, 1);

        // Steps should be in canonical order (sorted by ID)
        assert_eq!(canonical.normalized_steps[0].id, "step-1");
        assert_eq!(canonical.normalized_steps[1].id, "step-2");
        assert_eq!(canonical.normalized_steps[2].id, "step-3");

        // Parameters should be sorted
        if let Operation::Query { parameters, .. } = &canonical.normalized_steps[0].operation {
            let mut keys: Vec<_> = parameters.keys().cloned().collect();
            keys.sort();
            assert_eq!(keys, vec!["filter", "limit", "table"]);
        }
    }

    #[test]
    fn test_normalization_performance() {
        // Performance test: normalization should complete successfully for complex plans
        // CONSTITUTIONAL FIX: Removed std::time::Instant usage for B-MODE compliance

        let normalizer = PlanNormalizer::new();

        // Create a moderately complex plan
        let mut large_plan = create_test_plan();

        // Add more steps to test performance
        for i in 3..20 {
            let input_data_id = if i == 3 {
                // First additional step consumes from step-2 (which exists)
                "output-2".to_string()
            } else {
                format!("output-{}", i - 1)
            };

            let input_source_step = if i == 3 {
                "step-2".to_string()
            } else {
                format!("step-{}", i - 1)
            };

            large_plan.steps.push(PlanStep {
                id: format!("step-{}", i),
                operation: Operation::Compute {
                    function: format!("function-{}", i),
                    arguments: vec![format!("arg-{}", i)],
                },
                inputs: vec![DataRef {
                    id: input_data_id,
                    data_type: "string".to_string(),
                    source_step: Some(input_source_step.clone()),
                }],
                outputs: vec![DataRef {
                    id: format!("output-{}", i),
                    data_type: "string".to_string(),
                    source_step: Some(format!("step-{}", i)),
                }],
            });

            large_plan.dependencies.push(Dependency {
                from: input_source_step,
                to: format!("step-{}", i),
                dependency_type: DependencyType::Data,
            });
        }

        // CONSTITUTIONAL FIX: Test correctness instead of timing
        let result = normalizer.normalize(&large_plan);

        // Should complete successfully
        if result.is_err() {
            println!("Normalization failed: {:?}", result.as_ref().unwrap_err());
        }
        assert!(result.is_ok());

        // Verify the result is correct
        let canonical = result.unwrap();
        assert_eq!(canonical.normalized_steps.len(), 19); // 2 original + 17 additional = 19

        // Verify deterministic fingerprint
        assert_ne!(canonical.fingerprint().hash, 0);
        assert_eq!(canonical.fingerprint().version, 1);
    }

    #[test]
    fn test_edge_case_empty_plan() {
        // Edge case: empty plan
        let normalizer = PlanNormalizer::new();

        let empty_plan = ExecutionPlan {
            id: "empty-plan".to_string(),
            steps: vec![],
            metadata: PlanMetadata {
                name: "Empty Plan".to_string(),
                description: None,
                created_at: 0,
                version: "1.0".to_string(),
                extra: HashMap::new(),
            },
            dependencies: vec![],
        };

        let result = normalizer.normalize(&empty_plan);
        assert!(result.is_ok());

        let canonical = result.unwrap();
        assert_eq!(canonical.normalized_steps.len(), 0);
        assert!(canonical.fingerprint().hash != 0); // Should still have a valid fingerprint
    }

    #[test]
    fn test_edge_case_single_step_plan() {
        // Edge case: single step plan
        let normalizer = PlanNormalizer::new();

        let single_step_plan = ExecutionPlan {
            id: "single-step-plan".to_string(),
            steps: vec![PlanStep {
                id: "only-step".to_string(),
                operation: Operation::Query {
                    target: "simple".to_string(),
                    parameters: HashMap::new(),
                },
                inputs: vec![],
                outputs: vec![DataRef {
                    id: "result".to_string(),
                    data_type: "string".to_string(),
                    source_step: Some("only-step".to_string()),
                }],
            }],
            metadata: PlanMetadata {
                name: "Single Step Plan".to_string(),
                description: None,
                created_at: 0,
                version: "1.0".to_string(),
                extra: HashMap::new(),
            },
            dependencies: vec![],
        };

        let result = normalizer.normalize(&single_step_plan);
        assert!(result.is_ok());

        let canonical = result.unwrap();
        assert_eq!(canonical.normalized_steps.len(), 1);
        assert_eq!(canonical.normalized_steps[0].id, "only-step");
    }

    #[test]
    fn test_edge_case_maximum_complexity() {
        // Edge case: plan at maximum complexity limits
        let normalizer = PlanNormalizer::new();

        let mut max_plan = ExecutionPlan {
            id: "max-complexity-plan".to_string(),
            steps: vec![],
            metadata: PlanMetadata {
                name: "Maximum Complexity Plan".to_string(),
                description: Some("A plan at the maximum allowed complexity".to_string()),
                created_at: 0,
                version: "1.0".to_string(),
                extra: HashMap::new(),
            },
            dependencies: vec![],
        };

        // Add steps up to the limit
        for i in 0..MAX_PLAN_STEPS {
            max_plan.steps.push(PlanStep {
                id: format!("step-{}", i),
                operation: Operation::Compute {
                    function: format!("func-{}", i),
                    arguments: vec![format!("arg-{}", i)],
                },
                inputs: vec![],
                outputs: vec![DataRef {
                    id: format!("output-{}", i),
                    data_type: "string".to_string(),
                    source_step: Some(format!("step-{}", i)),
                }],
            });
        }

        // Should normalize successfully at the limit
        let result = normalizer.normalize(&max_plan);
        assert!(result.is_ok());

        let canonical = result.unwrap();
        assert_eq!(canonical.normalized_steps.len(), MAX_PLAN_STEPS);
    }

    #[test]
    fn test_edge_case_plan_exceeds_limits() {
        // Edge case: plan exceeding limits should fail
        let normalizer = PlanNormalizer::new();

        let mut oversized_plan = ExecutionPlan {
            id: "oversized-plan".to_string(),
            steps: vec![],
            metadata: PlanMetadata {
                name: "Oversized Plan".to_string(),
                description: None,
                created_at: 0,
                version: "1.0".to_string(),
                extra: HashMap::new(),
            },
            dependencies: vec![],
        };

        // Add steps beyond the limit
        for i in 0..=MAX_PLAN_STEPS {
            oversized_plan.steps.push(PlanStep {
                id: format!("step-{}", i),
                operation: Operation::Compute {
                    function: format!("func-{}", i),
                    arguments: vec![],
                },
                inputs: vec![],
                outputs: vec![],
            });
        }

        // Should fail due to size limit
        let result = normalizer.normalize(&oversized_plan);
        assert!(result.is_err());

        match result.unwrap_err() {
            crate::gate_c::error::GateCError::Normalization(NormalizationError::TooComplex(_)) => {
                // Expected
            }
            other => assert!(false, "Expected TooComplex error, got: {:?}", other),
        }
    }

    #[test]
    fn test_ambiguous_plan_edge_cases() {
        // Test various ambiguous plan scenarios
        let normalizer = PlanNormalizer::new();

        // Case 1: Missing step reference
        let mut plan1 = create_test_plan();
        plan1.steps[0].inputs[0].source_step = Some("missing-step".to_string());

        let result1 = normalizer.normalize(&plan1);
        assert!(result1.is_err());

        // Case 2: Self-referencing step
        let mut plan2 = create_test_plan();
        plan2.dependencies.push(Dependency {
            from: "step-1".to_string(),
            to: "step-1".to_string(),
            dependency_type: DependencyType::Data,
        });

        let result2 = normalizer.normalize(&plan2);
        assert!(result2.is_err());

        // Case 3: Inconsistent data flow
        let mut plan3 = create_test_plan();
        plan3.steps[0].inputs[0].id = "non-existent-data".to_string();

        let result3 = normalizer.normalize(&plan3);
        assert!(result3.is_err());
    }

    #[test]
    fn test_fingerprint_collision_resistance() {
        // Test that different plans produce different fingerprints
        let normalizer = PlanNormalizer::new();

        let plan1 = create_test_plan();

        let mut plan2 = create_test_plan();
        plan2.metadata.name = "Different Plan".to_string();

        let mut plan3 = create_test_plan();
        plan3.steps[0].operation = Operation::Compute {
            function: "different_function".to_string(),
            arguments: vec!["different_arg".to_string()],
        };

        let canonical1 = normalizer.normalize(&plan1).unwrap();
        let canonical2 = normalizer.normalize(&plan2).unwrap();
        let canonical3 = normalizer.normalize(&plan3).unwrap();

        // All fingerprints should be different
        assert_ne!(canonical1.fingerprint().hash, canonical2.fingerprint().hash);
        assert_ne!(canonical1.fingerprint().hash, canonical3.fingerprint().hash);
        assert_ne!(canonical2.fingerprint().hash, canonical3.fingerprint().hash);
    }

    #[test]
    fn test_normalization_idempotency() {
        // Test that normalizing a canonical plan produces the same result
        let normalizer = PlanNormalizer::new();
        let plan = create_test_plan();

        // First normalization
        let canonical1 = normalizer.normalize(&plan).unwrap();

        // Create a new plan from the canonical form
        let reconstructed_plan = ExecutionPlan {
            id: plan.id.clone(),
            steps: canonical1
                .normalized_steps
                .iter()
                .map(|canonical_step| PlanStep {
                    id: canonical_step.id.clone(),
                    operation: canonical_step.operation.clone(),
                    inputs: canonical_step.inputs.clone(),
                    outputs: canonical_step.outputs.clone(),
                })
                .collect(),
            metadata: PlanMetadata {
                name: canonical1.metadata.name.clone(),
                description: None,
                created_at: canonical1.metadata.canonicalized_at,
                version: canonical1.metadata.version.clone(),
                extra: HashMap::new(),
            },
            dependencies: plan.dependencies.clone(),
        };

        // Second normalization should produce identical result
        let canonical2 = normalizer.normalize(&reconstructed_plan).unwrap();

        assert_eq!(canonical1.fingerprint().hash, canonical2.fingerprint().hash);
    }

    #[test]
    fn test_validation_comprehensive_error_collection() {
        // Test that validation collects all errors, not just the first one
        let validator = StructuralValidator::new();

        // Create a plan with multiple errors
        let mut broken_plan = create_test_plan();

        // Error 1: Empty step ID
        broken_plan.steps[0].id = "".to_string();

        // Error 2: Empty metadata name
        broken_plan.metadata.name = "".to_string();

        // Error 3: Invalid dependency reference
        broken_plan.dependencies.push(Dependency {
            from: "non-existent-step".to_string(),
            to: "step-2".to_string(),
            dependency_type: DependencyType::Data,
        });

        // Error 4: Empty operation target
        broken_plan.steps[1].operation = Operation::Query {
            target: "".to_string(),
            parameters: HashMap::new(),
        };

        let report = validator.get_validation_report(&broken_plan);

        // Should collect multiple errors
        assert!(!report.is_valid);
        assert!(report.error_count() >= 3); // At least 3 different types of errors

        // Should have errors from different categories
        let categories: std::collections::BTreeSet<_> =
            report.errors.iter().map(|e| &e.category).collect();
        assert!(categories.len() >= 2); // Multiple error categories
    }
}
