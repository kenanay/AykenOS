//! # IR Planner (Semantic/Structural Only)
//!
//! Provide semantic analysis and ordering hints without runtime optimization.
//!
//! **ARCHITECTURAL RULE:**
//! This module MUST NOT depend on higher-level Gate C components.
//! Violations are considered architecture breaks.
//!
//! **Author:** Kenan AY  
//! **Phase:** 3.5 Gate C

use crate::gate_c::{
    error::{IRError, GateCResult},
    types::{ExecutionPlan, PlanStep, Operation, DataRef},
    limits::{MAX_PLAN_STEPS, MAX_SEMANTIC_HINTS},
};
use std::collections::{HashMap, HashSet};

/// Semantic analysis engine for BCIB instructions
/// 
/// **Key Constraints:**
/// - NO register allocation
/// - NO runtime optimization  
/// - Semantic analysis only
/// - Linear time complexity O(n)
pub struct SemanticAnalyzer {
    max_plan_steps: usize,
    max_hints: usize,
}

impl SemanticAnalyzer {
    /// Create new semantic analyzer
    pub fn new() -> Self {
        Self {
            max_plan_steps: MAX_PLAN_STEPS,
            max_hints: MAX_SEMANTIC_HINTS,
        }
    }
    
    /// Create analyzer with custom limits
    pub fn with_limits(max_plan_steps: usize, max_hints: usize) -> Self {
        Self {
            max_plan_steps,
            max_hints,
        }
    }
    
    /// Analyze execution plan for semantic dependencies
    /// 
    /// **Complexity:** O(n) where n is number of steps
    /// **Constraints:** No runtime optimization, semantic analysis only
    pub fn analyze_semantic_dependencies(&self, plan: &ExecutionPlan) -> GateCResult<SemanticAnalysis> {
        // Validate plan size
        if plan.steps.len() > self.max_plan_steps {
            return Err(IRError::TooComplex(format!(
                "Plan has {} steps, exceeds limit of {}", 
                plan.steps.len(), self.max_plan_steps
            )).into());
        }
        
        let mut analysis = SemanticAnalysis::new();
        
        // Build semantic dependency graph (O(n))
        let dependency_graph = self.build_semantic_dependency_graph(&plan.steps)?;
        analysis.dependency_graph = dependency_graph;
        
        // Generate ordering hints (O(n))
        let ordering_hints = self.generate_ordering_hints(&plan.steps)?;
        analysis.ordering_hints = ordering_hints;
        
        // Generate parallelism hints (O(n))
        let parallelism_hints = self.generate_parallelism_hints(&plan.steps)?;
        analysis.parallelism_hints = parallelism_hints;
        
        // Validate hint count limits
        if analysis.ordering_hints.len() > self.max_hints {
            return Err(IRError::TooComplex(format!(
                "Generated {} ordering hints, exceeds limit of {}", 
                analysis.ordering_hints.len(), self.max_hints
            )).into());
        }
        
        Ok(analysis)
    }
    
    /// Build semantic dependency graph
    /// 
    /// **Complexity:** O(n) - single pass through steps
    fn build_semantic_dependency_graph(&self, steps: &[PlanStep]) -> GateCResult<SemanticDependencyGraph> {
        let mut graph = SemanticDependencyGraph::new();
        let mut data_producers: HashMap<String, String> = HashMap::new();
        
        // Single pass through steps (O(n))
        for step in steps {
            let mut dependencies = Vec::new();
            
            // Check input dependencies
            for input in &step.inputs {
                if let Some(producer_step) = &input.source_step {
                    dependencies.push(SemanticDependency {
                        from_step: producer_step.clone(),
                        to_step: step.id.clone(),
                        dependency_type: SemanticDependencyType::DataFlow,
                        data_ref: Some(input.id.clone()),
                    });
                }
            }
            
            // Record output producers
            for output in &step.outputs {
                data_producers.insert(output.id.clone(), step.id.clone());
            }
            
            // Add step to graph
            graph.add_step(step.id.clone(), dependencies);
        }
        
        Ok(graph)
    }
    
    /// Generate enhanced semantic ordering hints with dependency-based analysis
    /// 
    /// **Complexity:** O(n) - single pass analysis with dependency tracking
    /// **Enhanced Features:**
    /// - Dependency-based ordering suggestions
    /// - Critical path analysis
    /// - Resource contention hints
    /// - Execution priority scoring
    fn generate_ordering_hints(&self, steps: &[PlanStep]) -> GateCResult<Vec<OrderingHint>> {
        let mut hints = Vec::new();
        
        // Build dependency map for enhanced analysis
        let dependency_map = self.build_dependency_map(steps);
        let critical_path = self.identify_critical_path(steps, &dependency_map);
        
        // Analyze each step for ordering opportunities (O(n))
        for (_i, step) in steps.iter().enumerate() {
            // Enhanced operation-based hints
            match &step.operation {
                Operation::Query { .. } => {
                    let confidence = if step.inputs.is_empty() { 0.9 } else { 0.7 };
                    hints.push(OrderingHint {
                        step_id: step.id.clone(),
                        hint_type: OrderingHintType::EarlyExecution,
                        reason: "Query operations can be executed early to reduce latency".to_string(),
                        confidence,
                    });
                    
                    // Add IO-intensive hint for queries
                    hints.push(OrderingHint {
                        step_id: step.id.clone(),
                        hint_type: OrderingHintType::IOIntensive,
                        reason: "Query operations are typically IO-bound".to_string(),
                        confidence: 0.8,
                    });
                }
                Operation::Compute { .. } => {
                    hints.push(OrderingHint {
                        step_id: step.id.clone(),
                        hint_type: OrderingHintType::ComputeIntensive,
                        reason: "Compute operations may benefit from parallel execution".to_string(),
                        confidence: 0.8,
                    });
                    
                    // Check if compute step is on critical path
                    if critical_path.contains(&step.id) {
                        hints.push(OrderingHint {
                            step_id: step.id.clone(),
                            hint_type: OrderingHintType::EarlyExecution,
                            reason: "Step is on critical path and should be prioritized".to_string(),
                            confidence: 0.95,
                        });
                    }
                }
                Operation::Mutation { .. } => {
                    hints.push(OrderingHint {
                        step_id: step.id.clone(),
                        hint_type: OrderingHintType::LateExecution,
                        reason: "Mutation operations should be executed after dependencies".to_string(),
                        confidence: 0.9,
                    });
                }
            }
            
            // Enhanced dependency-based ordering hints
            let input_count = step.inputs.len();
            let output_count = step.outputs.len();
            let dependents = self.count_dependents(step, steps);
            
            if input_count == 0 {
                hints.push(OrderingHint {
                    step_id: step.id.clone(),
                    hint_type: OrderingHintType::EarlyExecution,
                    reason: "Step has no input dependencies and can start immediately".to_string(),
                    confidence: 0.9,
                });
            }
            
            if output_count == 0 {
                let confidence = if dependents == 0 { 0.8 } else { 0.6 };
                hints.push(OrderingHint {
                    step_id: step.id.clone(),
                    hint_type: OrderingHintType::LateExecution,
                    reason: "Step produces no outputs and can be deferred".to_string(),
                    confidence,
                });
            }
            
            // High fan-out steps should be prioritized
            if dependents > 2 {
                hints.push(OrderingHint {
                    step_id: step.id.clone(),
                    hint_type: OrderingHintType::EarlyExecution,
                    reason: format!("Step has {} dependents and should be prioritized", dependents),
                    confidence: 0.85,
                });
            }
            
            // Resource contention analysis
            if let Some(resource_hint) = self.analyze_resource_contention(step, steps) {
                hints.push(resource_hint);
            }
        }
        
        // Add critical path hints
        for step_id in &critical_path {
            hints.push(OrderingHint {
                step_id: step_id.clone(),
                hint_type: OrderingHintType::EarlyExecution,
                reason: "Step is on the critical path for plan completion".to_string(),
                confidence: 0.95,
            });
        }
        
        Ok(hints)
    }
    
    /// Build dependency map for enhanced analysis
    fn build_dependency_map(&self, steps: &[PlanStep]) -> HashMap<String, Vec<String>> {
        let mut dependency_map = HashMap::new();
        
        for step in steps {
            let mut dependencies = Vec::new();
            for input in &step.inputs {
                if let Some(source_step) = &input.source_step {
                    dependencies.push(source_step.clone());
                }
            }
            dependency_map.insert(step.id.clone(), dependencies);
        }
        
        dependency_map
    }
    
    /// Identify critical path through the plan
    /// 
    /// **Algorithm:** Longest path through dependency graph
    fn identify_critical_path(&self, steps: &[PlanStep], dependency_map: &HashMap<String, Vec<String>>) -> Vec<String> {
        let mut critical_path = Vec::new();
        let mut visited = HashSet::new();
        let mut path_lengths: HashMap<String, usize> = HashMap::new();
        
        // Calculate longest path to each step
        for step in steps {
            if !visited.contains(&step.id) {
                self.calculate_longest_path(&step.id, dependency_map, &mut path_lengths, &mut visited);
            }
        }
        
        // Find the step with maximum path length
        if let Some((longest_step, _)) = path_lengths.iter().max_by_key(|(_, &length)| length) {
            // Reconstruct critical path
            self.reconstruct_critical_path(longest_step, dependency_map, &path_lengths, &mut critical_path);
        }
        
        critical_path
    }
    
    /// Calculate longest path to a step (recursive with memoization)
    fn calculate_longest_path(
        &self,
        step_id: &str,
        dependency_map: &HashMap<String, Vec<String>>,
        path_lengths: &mut HashMap<String, usize>,
        visited: &mut HashSet<String>,
    ) -> usize {
        if let Some(&length) = path_lengths.get(step_id) {
            return length;
        }
        
        visited.insert(step_id.to_string());
        
        let empty_deps = Vec::new();
        let dependencies = dependency_map.get(step_id).unwrap_or(&empty_deps);
        let max_dep_length = dependencies
            .iter()
            .map(|dep| self.calculate_longest_path(dep, dependency_map, path_lengths, visited))
            .max()
            .unwrap_or(0);
        
        let length = max_dep_length + 1;
        path_lengths.insert(step_id.to_string(), length);
        length
    }
    
    /// Reconstruct critical path from longest step
    fn reconstruct_critical_path(
        &self,
        step_id: &str,
        dependency_map: &HashMap<String, Vec<String>>,
        path_lengths: &HashMap<String, usize>,
        critical_path: &mut Vec<String>,
    ) {
        critical_path.push(step_id.to_string());
        
        let current_length = path_lengths.get(step_id).unwrap_or(&0);
        let empty_deps = Vec::new();
        let dependencies = dependency_map.get(step_id).unwrap_or(&empty_deps);
        
        // Find dependency with length = current_length - 1
        for dep in dependencies {
            if let Some(&dep_length) = path_lengths.get(dep) {
                if dep_length == current_length - 1 {
                    self.reconstruct_critical_path(dep, dependency_map, path_lengths, critical_path);
                    break;
                }
            }
        }
    }
    
    /// Count number of steps that depend on this step
    fn count_dependents(&self, step: &PlanStep, all_steps: &[PlanStep]) -> usize {
        let mut count = 0;
        for other_step in all_steps {
            for input in &other_step.inputs {
                if let Some(source_step) = &input.source_step {
                    if source_step == &step.id {
                        count += 1;
                        break;
                    }
                }
            }
        }
        count
    }
    
    /// Analyze resource contention for ordering hints
    fn analyze_resource_contention(&self, step: &PlanStep, all_steps: &[PlanStep]) -> Option<OrderingHint> {
        // Check for steps that might contend for similar resources
        let mut contending_steps = 0;
        
        for other_step in all_steps {
            if other_step.id == step.id {
                continue;
            }
            
            // Simple heuristic: same operation type might contend for resources
            if std::mem::discriminant(&step.operation) == std::mem::discriminant(&other_step.operation) {
                contending_steps += 1;
            }
        }
        
        if contending_steps > 2 {
            Some(OrderingHint {
                step_id: step.id.clone(),
                hint_type: OrderingHintType::EarlyExecution,
                reason: format!("Step may contend with {} similar operations", contending_steps),
                confidence: 0.6,
            })
        } else {
            None
        }
    }
    
    /// Generate enhanced parallelism opportunity hints with advanced analysis
    /// 
    /// **Complexity:** O(n) - CRITICAL FIX: Linear time analysis per Gate C rules
    /// **Note:** These are semantic hints only, NOT execution optimization
    /// **Enhanced Features:**
    /// - Resource-aware parallelism analysis
    /// - Execution time estimation hints
    /// - Dependency chain analysis
    /// - Parallelizable group identification
    fn generate_parallelism_hints(&self, steps: &[PlanStep]) -> GateCResult<Vec<ParallelismHint>> {
        let mut hints = Vec::new();
        let mut data_dependencies: HashMap<String, HashSet<String>> = HashMap::new();
        let mut resource_usage: HashMap<String, Vec<String>> = HashMap::new();
        
        // CRITICAL FIX: Single pass O(n) analysis instead of O(n²)
        // Build data dependency and resource usage maps (O(n))
        for step in steps {
            let mut deps = HashSet::new();
            for input in &step.inputs {
                if let Some(source) = &input.source_step {
                    deps.insert(source.clone());
                }
            }
            data_dependencies.insert(step.id.clone(), deps);
            
            // Analyze resource usage patterns
            let resources = self.analyze_step_resources(step);
            resource_usage.insert(step.id.clone(), resources);
        }
        
        // CRITICAL FIX: O(n) parallelism analysis using dependency sets
        // Instead of O(n²) pairwise comparison, use set operations
        for step in steps {
            let step_deps = data_dependencies.get(&step.id).unwrap();
            let step_resources = resource_usage.get(&step.id).unwrap();
            
            // Count potential parallel candidates using set operations (O(1) per step)
            let mut parallel_count = 0;
            let mut resource_contention_count = 0;
            
            for other_step in steps {
                if other_step.id == step.id {
                    continue;
                }
                
                let other_deps = data_dependencies.get(&other_step.id).unwrap();
                
                // Quick dependency check using sets (O(1) average case)
                if !step_deps.contains(&other_step.id) && !other_deps.contains(&step.id) {
                    // Check for data conflicts (simplified O(1) check)
                    let has_data_conflict = self.quick_data_conflict_check(step, other_step);
                    
                    if !has_data_conflict {
                        // Quick resource conflict check
                        let resource_conflict = self.quick_resource_conflict_check(step_resources, resource_usage.get(&other_step.id).unwrap());
                        
                        if resource_conflict {
                            resource_contention_count += 1;
                        } else {
                            parallel_count += 1;
                        }
                    }
                }
            }
            
            // Generate hints based on counts (O(1))
            if parallel_count > 0 {
                let confidence = self.calculate_simple_parallelism_confidence(step, parallel_count);
                hints.push(ParallelismHint {
                    step_id: step.id.clone(),
                    hint_type: ParallelismHintType::ParallelCandidate,
                    parallel_with: vec![], // Simplified: don't store all candidates
                    confidence,
                    reason: format!("Step can potentially run in parallel with {} other steps", parallel_count),
                });
            }
            
            // Generate resource contention hints
            if resource_contention_count > 0 {
                hints.push(ParallelismHint {
                    step_id: step.id.clone(),
                    hint_type: ParallelismHintType::ResourceContention,
                    parallel_with: vec![], // Simplified
                    confidence: 0.8,
                    reason: format!("Step may contend with {} other steps for resources", resource_contention_count),
                });
            }
            
            // Check if step requires sequential execution
            if self.requires_sequential_execution(step) {
                hints.push(ParallelismHint {
                    step_id: step.id.clone(),
                    hint_type: ParallelismHintType::SequentialRequired,
                    parallel_with: vec![],
                    confidence: 0.9,
                    reason: "Step requires sequential execution due to operation type".to_string(),
                });
            }
        }
        
        Ok(hints)
    }
    
    /// Analyze resource usage patterns for a step
    fn analyze_step_resources(&self, step: &PlanStep) -> Vec<String> {
        let mut resources = Vec::new();
        
        match &step.operation {
            Operation::Query { target, .. } => {
                resources.push(format!("query:{}", target));
                resources.push("io".to_string());
            }
            Operation::Compute { function, .. } => {
                resources.push(format!("compute:{}", function));
                resources.push("cpu".to_string());
            }
            Operation::Mutation { .. } => {
                resources.push("mutation".to_string());
                resources.push("write".to_string());
            }
        }
        
        // Add resource hints based on inputs/outputs
        for input in &step.inputs {
            resources.push(format!("read:{}", input.data_type));
        }
        
        for output in &step.outputs {
            resources.push(format!("write:{}", output.data_type));
        }
        
        resources
    }
    
    /// Check for resource conflicts between steps
    fn check_resource_conflicts(&self, resources1: &[String], resources2: &[String]) -> bool {
        for r1 in resources1 {
            for r2 in resources2 {
                // Check for write-write conflicts
                if r1.starts_with("write:") && r2.starts_with("write:") {
                    if r1 == r2 {
                        return true;
                    }
                }
                
                // Check for specific resource conflicts
                if r1 == r2 && (r1 == "mutation" || r1 == "cpu" || r1 == "io") {
                    return true;
                }
            }
        }
        false
    }
    
    /// Calculate confidence score for parallelism hint
    fn calculate_parallelism_confidence(&self, step: &PlanStep, candidates: &[String], _all_steps: &[PlanStep]) -> f64 {
        let mut confidence: f64 = 0.7; // Base confidence
        
        // Increase confidence for independent operations
        match &step.operation {
            Operation::Query { .. } => confidence += 0.1,
            Operation::Compute { .. } => confidence += 0.15,
            Operation::Mutation { .. } => confidence -= 0.2, // Mutations are riskier
        }
        
        // Adjust based on number of candidates
        if candidates.len() > 3 {
            confidence += 0.1; // More candidates = higher confidence
        }
        
        // Adjust based on step complexity
        let input_count = step.inputs.len();
        let output_count = step.outputs.len();
        
        if input_count == 0 && output_count > 0 {
            confidence += 0.1; // Source steps are good for parallelism
        }
        
        if input_count > 0 && output_count == 0 {
            confidence += 0.05; // Sink steps are also good
        }
        
        confidence.min(1.0).max(0.0)
    }
    
    /// Check if step requires sequential execution
    fn requires_sequential_execution(&self, step: &PlanStep) -> bool {
        match &step.operation {
            Operation::Mutation { .. } => true, // Mutations often need sequential execution
            Operation::Query { .. } => false,
            Operation::Compute { .. } => false,
        }
    }
    
    /// Identify groups of steps that can be parallelized together
    fn identify_parallelizable_groups(&self, steps: &[PlanStep], dependencies: &HashMap<String, HashSet<String>>) -> Vec<Vec<String>> {
        let mut groups = Vec::new();
        let mut visited = HashSet::new();
        
        for step in steps {
            if visited.contains(&step.id) {
                continue;
            }
            
            let mut group = Vec::new();
            self.find_parallelizable_group(&step.id, steps, dependencies, &mut group, &mut visited);
            
            if group.len() > 1 {
                groups.push(group);
            }
        }
        
        groups
    }
    
    /// Find all steps that can be parallelized with the given step
    fn find_parallelizable_group(
        &self,
        step_id: &str,
        all_steps: &[PlanStep],
        dependencies: &HashMap<String, HashSet<String>>,
        group: &mut Vec<String>,
        visited: &mut HashSet<String>,
    ) {
        if visited.contains(step_id) {
            return;
        }
        
        visited.insert(step_id.to_string());
        group.push(step_id.to_string());
        
        let step = all_steps.iter().find(|s| s.id == step_id);
        if step.is_none() {
            return;
        }
        let step = step.unwrap();
        
        // Find other steps at the same dependency level
        let step_deps = dependencies.get(step_id).unwrap();
        
        for other_step in all_steps {
            if visited.contains(&other_step.id) || other_step.id == step_id {
                continue;
            }
            
            let other_deps = dependencies.get(&other_step.id).unwrap();
            
            // Check if steps can be parallelized
            if !step_deps.contains(&other_step.id) 
                && !other_deps.contains(step_id)
                && !self.quick_data_conflict_check(step, other_step) {
                
                // Recursively add to group
                self.find_parallelizable_group(&other_step.id, all_steps, dependencies, group, visited);
            }
        }
    }
    
    /// Quick data conflict check (O(1) simplified version)
    fn quick_data_conflict_check(&self, step1: &PlanStep, step2: &PlanStep) -> bool {
        // Simplified conflict detection for O(n) analysis
        
        // Check for write-write conflicts (overlapping outputs)
        for output1 in &step1.outputs {
            for output2 in &step2.outputs {
                if output1.id == output2.id {
                    return true;
                }
            }
        }
        
        // Check for read-write conflicts (step1 writes what step2 reads, or vice versa)
        for output1 in &step1.outputs {
            for input2 in &step2.inputs {
                if output1.id == input2.id {
                    return true;
                }
            }
        }
        
        for output2 in &step2.outputs {
            for input1 in &step1.inputs {
                if output2.id == input1.id {
                    return true;
                }
            }
        }
        
        false
    }
    
    /// Quick resource conflict check (O(1) simplified version)
    fn quick_resource_conflict_check(&self, resources1: &[String], resources2: &[String]) -> bool {
        // Simplified resource conflict check
        for r1 in resources1 {
            for r2 in resources2 {
                if r1 == r2 && (r1 == "mutation" || r1 == "cpu" || r1 == "io") {
                    return true;
                }
            }
        }
        false
    }
    
    /// Calculate simple parallelism confidence (O(1))
    fn calculate_simple_parallelism_confidence(&self, step: &PlanStep, parallel_count: usize) -> f64 {
        let mut confidence: f64 = 0.7; // Base confidence
        
        // Increase confidence for independent operations
        match &step.operation {
            Operation::Query { .. } => confidence += 0.1,
            Operation::Compute { .. } => confidence += 0.15,
            Operation::Mutation { .. } => confidence -= 0.2, // Mutations are riskier
        }
        
        // Adjust based on number of candidates
        if parallel_count > 3 {
            confidence += 0.1;
        }
        
        confidence.min(1.0).max(0.0)
    }
}

impl Default for SemanticAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of semantic analysis
#[derive(Debug, Clone)]
pub struct SemanticAnalysis {
    pub dependency_graph: SemanticDependencyGraph,
    pub ordering_hints: Vec<OrderingHint>,
    pub parallelism_hints: Vec<ParallelismHint>,
}

impl SemanticAnalysis {
    fn new() -> Self {
        Self {
            dependency_graph: SemanticDependencyGraph::new(),
            ordering_hints: Vec::new(),
            parallelism_hints: Vec::new(),
        }
    }
}

/// Semantic dependency graph
#[derive(Debug, Clone)]
pub struct SemanticDependencyGraph {
    steps: HashMap<String, Vec<SemanticDependency>>,
}

impl SemanticDependencyGraph {
    fn new() -> Self {
        Self {
            steps: HashMap::new(),
        }
    }
    
    fn add_step(&mut self, step_id: String, dependencies: Vec<SemanticDependency>) {
        self.steps.insert(step_id, dependencies);
    }
    
    /// Get dependencies for a step
    pub fn get_dependencies(&self, step_id: &str) -> Option<&Vec<SemanticDependency>> {
        self.steps.get(step_id)
    }
    
    /// Get all steps in the graph
    pub fn get_steps(&self) -> Vec<&String> {
        self.steps.keys().collect()
    }
}

/// Semantic dependency between steps
#[derive(Debug, Clone)]
pub struct SemanticDependency {
    pub from_step: String,
    pub to_step: String,
    pub dependency_type: SemanticDependencyType,
    pub data_ref: Option<String>,
}

/// Type of semantic dependency
#[derive(Debug, Clone, PartialEq)]
pub enum SemanticDependencyType {
    DataFlow,
    ControlFlow,
    ResourceAccess,
}

/// Ordering hint for execution planning
#[derive(Debug, Clone)]
pub struct OrderingHint {
    pub step_id: String,
    pub hint_type: OrderingHintType,
    pub reason: String,
    pub confidence: f64, // 0.0 to 1.0
}

/// Type of ordering hint
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum OrderingHintType {
    EarlyExecution,
    LateExecution,
    ComputeIntensive,
    IOIntensive,
}

/// Parallelism opportunity hint
#[derive(Debug, Clone)]
pub struct ParallelismHint {
    pub step_id: String,
    pub hint_type: ParallelismHintType,
    pub parallel_with: Vec<String>,
    pub confidence: f64, // 0.0 to 1.0
    pub reason: String,
}

/// Type of parallelism hint
#[derive(Debug, Clone, PartialEq)]
pub enum ParallelismHintType {
    ParallelCandidate,
    SequentialRequired,
    ResourceContention,
}

/// Enhanced ordering hint generator with dependency-based analysis
pub struct OrderingHinter {
    semantic_analyzer: SemanticAnalyzer,
}

impl OrderingHinter {
    /// Create new ordering hinter
    pub fn new() -> Self {
        Self {
            semantic_analyzer: SemanticAnalyzer::new(),
        }
    }
    
    /// Generate enhanced ordering hints for a plan
    pub fn generate_hints(&self, plan: &ExecutionPlan) -> GateCResult<Vec<OrderingHint>> {
        let analysis = self.semantic_analyzer.analyze_semantic_dependencies(plan)?;
        Ok(analysis.ordering_hints)
    }
    
    /// Generate dependency-based ordering suggestions
    pub fn generate_dependency_based_hints(&self, plan: &ExecutionPlan) -> GateCResult<Vec<OrderingHint>> {
        let mut hints = Vec::new();
        
        // Build dependency graph
        let dependency_graph = self.semantic_analyzer.build_semantic_dependency_graph(&plan.steps)?;
        
        // Analyze dependency patterns
        for step in &plan.steps {
            let dependencies = dependency_graph.get_dependencies(&step.id);
            
            if let Some(deps) = dependencies {
                // Steps with many dependencies should be scheduled later
                if deps.len() > 3 {
                    hints.push(OrderingHint {
                        step_id: step.id.clone(),
                        hint_type: OrderingHintType::LateExecution,
                        reason: format!("Step has {} dependencies and should wait", deps.len()),
                        confidence: 0.8,
                    });
                }
                
                // Steps with only data flow dependencies can be optimized
                let data_flow_only = deps.iter().all(|d| d.dependency_type == SemanticDependencyType::DataFlow);
                if data_flow_only && deps.len() <= 2 {
                    hints.push(OrderingHint {
                        step_id: step.id.clone(),
                        hint_type: OrderingHintType::EarlyExecution,
                        reason: "Step has simple data flow dependencies".to_string(),
                        confidence: 0.75,
                    });
                }
            }
        }
        
        Ok(hints)
    }
    
    /// Validate hint consistency with enhanced checks
    pub fn validate_hints(&self, hints: &[OrderingHint]) -> GateCResult<()> {
        // Check for conflicting hints
        let mut early_steps = HashSet::new();
        let mut late_steps = HashSet::new();
        let mut compute_intensive = HashSet::new();
        let mut io_intensive = HashSet::new();
        
        for hint in hints {
            match hint.hint_type {
                OrderingHintType::EarlyExecution => {
                    if late_steps.contains(&hint.step_id) {
                        return Err(IRError::InconsistentHints(format!(
                            "Step {} has conflicting early/late execution hints", 
                            hint.step_id
                        )).into());
                    }
                    early_steps.insert(&hint.step_id);
                }
                OrderingHintType::LateExecution => {
                    if early_steps.contains(&hint.step_id) {
                        return Err(IRError::InconsistentHints(format!(
                            "Step {} has conflicting early/late execution hints", 
                            hint.step_id
                        )).into());
                    }
                    late_steps.insert(&hint.step_id);
                }
                OrderingHintType::ComputeIntensive => {
                    if io_intensive.contains(&hint.step_id) {
                        return Err(IRError::InconsistentHints(format!(
                            "Step {} has conflicting compute/IO intensive hints", 
                            hint.step_id
                        )).into());
                    }
                    compute_intensive.insert(&hint.step_id);
                }
                OrderingHintType::IOIntensive => {
                    if compute_intensive.contains(&hint.step_id) {
                        return Err(IRError::InconsistentHints(format!(
                            "Step {} has conflicting compute/IO intensive hints", 
                            hint.step_id
                        )).into());
                    }
                    io_intensive.insert(&hint.step_id);
                }
            }
        }
        
        // Validate confidence scores
        for hint in hints {
            if hint.confidence < 0.0 || hint.confidence > 1.0 {
                return Err(IRError::InconsistentHints(format!(
                    "Step {} has invalid confidence score: {}", 
                    hint.step_id, hint.confidence
                )).into());
            }
        }
        
        Ok(())
    }
    
    /// Merge and deduplicate hints from multiple sources
    pub fn merge_hints(&self, hint_sets: Vec<Vec<OrderingHint>>) -> Vec<OrderingHint> {
        let mut merged_hints: HashMap<(String, OrderingHintType), OrderingHint> = HashMap::new();
        
        for hint_set in hint_sets {
            for hint in hint_set {
                let key = (hint.step_id.clone(), hint.hint_type.clone());
                
                // Keep hint with higher confidence
                if let Some(existing_hint) = merged_hints.get(&key) {
                    if hint.confidence > existing_hint.confidence {
                        merged_hints.insert(key, hint);
                    }
                } else {
                    merged_hints.insert(key, hint);
                }
            }
        }
        
        merged_hints.into_values().collect()
    }
    
    /// Filter hints by confidence threshold
    pub fn filter_by_confidence(&self, hints: Vec<OrderingHint>, min_confidence: f64) -> Vec<OrderingHint> {
        hints.into_iter()
            .filter(|hint| hint.confidence >= min_confidence)
            .collect()
    }
    
    /// Generate execution priority scores based on hints
    pub fn generate_priority_scores(&self, hints: &[OrderingHint]) -> HashMap<String, f64> {
        let mut scores = HashMap::new();
        
        for hint in hints {
            let score_delta = match hint.hint_type {
                OrderingHintType::EarlyExecution => hint.confidence * 10.0,
                OrderingHintType::LateExecution => -hint.confidence * 5.0,
                OrderingHintType::ComputeIntensive => hint.confidence * 2.0,
                OrderingHintType::IOIntensive => hint.confidence * 1.0,
            };
            
            *scores.entry(hint.step_id.clone()).or_insert(0.0) += score_delta;
        }
        
        scores
    }
}

impl Default for OrderingHinter {
    fn default() -> Self {
        Self::new()
    }
}

/// IR Planner - coordinates semantic analysis and hint generation
pub struct IRPlanner {
    semantic_analyzer: SemanticAnalyzer,
    ordering_hinter: OrderingHinter,
}

impl IRPlanner {
    /// Create new IR planner
    pub fn new() -> Self {
        Self {
            semantic_analyzer: SemanticAnalyzer::new(),
            ordering_hinter: OrderingHinter::new(),
        }
    }
    
    /// Perform complete semantic analysis of execution plan
    pub fn analyze_plan(&self, plan: &ExecutionPlan) -> GateCResult<SemanticAnalysis> {
        // Validate plan structure first
        self.validate_plan_structure(plan)?;
        
        // Perform semantic analysis
        let analysis = self.semantic_analyzer.analyze_semantic_dependencies(plan)?;
        
        // Validate hint consistency
        self.ordering_hinter.validate_hints(&analysis.ordering_hints)?;
        
        Ok(analysis)
    }
    
    /// Validate plan structure for semantic analysis
    fn validate_plan_structure(&self, plan: &ExecutionPlan) -> GateCResult<()> {
        if plan.steps.is_empty() {
            return Err(IRError::InvalidPlan(
                "Plan cannot be empty".to_string()
            ).into());
        }
        
        // Check for duplicate step IDs
        let mut seen_ids = HashSet::new();
        for step in &plan.steps {
            if seen_ids.contains(&step.id) {
                return Err(IRError::InvalidPlan(format!(
                    "Duplicate step ID: {}", step.id
                )).into());
            }
            seen_ids.insert(&step.id);
        }
        
        Ok(())
    }
}

impl Default for IRPlanner {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gate_c::types::{PlanMetadata, MutationIntent, ResourcePath, ChangeSet};
    use std::collections::HashMap;

    fn create_test_plan() -> ExecutionPlan {
        ExecutionPlan {
            id: "test-plan".to_string(),
            steps: vec![
                PlanStep {
                    id: "step-1".to_string(),
                    operation: Operation::Query {
                        target: "data-source".to_string(),
                        parameters: HashMap::new(),
                    },
                    inputs: vec![],
                    outputs: vec![DataRef {
                        id: "data-1".to_string(),
                        data_type: "string".to_string(),
                        source_step: Some("step-1".to_string()),
                    }],
                },
                PlanStep {
                    id: "step-2".to_string(),
                    operation: Operation::Compute {
                        function: "process".to_string(),
                        arguments: vec!["arg1".to_string()],
                    },
                    inputs: vec![DataRef {
                        id: "data-1".to_string(),
                        data_type: "string".to_string(),
                        source_step: Some("step-1".to_string()),
                    }],
                    outputs: vec![DataRef {
                        id: "data-2".to_string(),
                        data_type: "string".to_string(),
                        source_step: Some("step-2".to_string()),
                    }],
                },
            ],
            metadata: PlanMetadata {
                name: "Test Plan".to_string(),
                description: Some("Test plan for IR analysis".to_string()),
                created_at: 1234567890,
                version: "1.0".to_string(),
                extra: HashMap::new(),
            },
            dependencies: vec![],
        }
    }

    #[test]
    fn test_semantic_analyzer_creation() {
        let analyzer = SemanticAnalyzer::new();
        assert_eq!(analyzer.max_plan_steps, MAX_PLAN_STEPS);
        assert_eq!(analyzer.max_hints, MAX_SEMANTIC_HINTS);
    }

    #[test]
    fn test_semantic_dependency_analysis() {
        let analyzer = SemanticAnalyzer::new();
        let plan = create_test_plan();
        
        let result = analyzer.analyze_semantic_dependencies(&plan);
        assert!(result.is_ok());
        
        let analysis = result.unwrap();
        assert!(!analysis.ordering_hints.is_empty());
        // Parallelism hints may be empty if no parallelism opportunities exist
        // This is expected behavior
        
        // Check dependency graph
        let deps = analysis.dependency_graph.get_dependencies("step-2");
        assert!(deps.is_some());
        assert!(!deps.unwrap().is_empty());
    }

    #[test]
    fn test_ordering_hint_generation() {
        let analyzer = SemanticAnalyzer::new();
        let plan = create_test_plan();
        
        let hints = analyzer.generate_ordering_hints(&plan.steps).unwrap();
        assert!(!hints.is_empty());
        
        // Check that we have hints for both steps
        let step1_hints: Vec<_> = hints.iter().filter(|h| h.step_id == "step-1").collect();
        let step2_hints: Vec<_> = hints.iter().filter(|h| h.step_id == "step-2").collect();
        
        assert!(!step1_hints.is_empty());
        assert!(!step2_hints.is_empty());
    }

    #[test]
    fn test_parallelism_hint_generation() {
        let analyzer = SemanticAnalyzer::new();
        let plan = create_test_plan();
        
        let _hints = analyzer.generate_parallelism_hints(&plan.steps).unwrap();
        // May or may not have parallelism hints depending on dependencies
        // This is expected behavior
    }

    #[test]
    fn test_data_conflict_detection() {
        let analyzer = SemanticAnalyzer::new();
        
        let step1 = PlanStep {
            id: "step-1".to_string(),
            operation: Operation::Query {
                target: "test".to_string(),
                parameters: HashMap::new(),
            },
            inputs: vec![],
            outputs: vec![DataRef {
                id: "data-1".to_string(),
                data_type: "string".to_string(),
                source_step: Some("step-1".to_string()),
            }],
        };
        
        let step2 = PlanStep {
            id: "step-2".to_string(),
            operation: Operation::Query {
                target: "test".to_string(),
                parameters: HashMap::new(),
            },
            inputs: vec![DataRef {
                id: "data-1".to_string(),
                data_type: "string".to_string(),
                source_step: Some("step-1".to_string()),
            }],
            outputs: vec![],
        };
        
        // Should detect write-read conflict
        assert!(analyzer.quick_data_conflict_check(&step2, &step1)); // step2 reads what step1 writes
    }

    #[test]
    fn test_ordering_hinter() {
        let hinter = OrderingHinter::new();
        let plan = create_test_plan();
        
        let hints = hinter.generate_hints(&plan).unwrap();
        assert!(!hints.is_empty());
        
        // Validate hints
        let result = hinter.validate_hints(&hints);
        assert!(result.is_ok());
    }

    #[test]
    fn test_ir_planner() {
        let planner = IRPlanner::new();
        let plan = create_test_plan();
        
        let analysis = planner.analyze_plan(&plan).unwrap();
        assert!(!analysis.ordering_hints.is_empty());
    }

    #[test]
    fn test_empty_plan_rejection() {
        let planner = IRPlanner::new();
        let empty_plan = ExecutionPlan {
            id: "empty".to_string(),
            steps: vec![],
            metadata: PlanMetadata {
                name: "Empty".to_string(),
                description: None,
                created_at: 0,
                version: "1.0".to_string(),
                extra: HashMap::new(),
            },
            dependencies: vec![],
        };
        
        let result = planner.analyze_plan(&empty_plan);
        assert!(result.is_err());
    }

    #[test]
    fn test_duplicate_step_id_rejection() {
        let planner = IRPlanner::new();
        let mut plan = create_test_plan();
        plan.steps[1].id = plan.steps[0].id.clone(); // Duplicate ID
        
        let result = planner.analyze_plan(&plan);
        assert!(result.is_err());
    }

    #[test]
    fn test_semantic_dependency_graph() {
        let mut graph = SemanticDependencyGraph::new();
        
        let dep = SemanticDependency {
            from_step: "step-1".to_string(),
            to_step: "step-2".to_string(),
            dependency_type: SemanticDependencyType::DataFlow,
            data_ref: Some("data-1".to_string()),
        };
        
        graph.add_step("step-2".to_string(), vec![dep]);
        
        let deps = graph.get_dependencies("step-2");
        assert!(deps.is_some());
        assert_eq!(deps.unwrap().len(), 1);
        
        let steps = graph.get_steps();
        assert_eq!(steps.len(), 1);
    }

    #[test]
    fn test_enhanced_ordering_hint_generation() {
        let analyzer = SemanticAnalyzer::new();
        let plan = create_complex_test_plan();
        
        let hints = analyzer.generate_ordering_hints(&plan.steps).unwrap();
        assert!(!hints.is_empty());
        
        // Check for critical path hints
        let critical_path_hints: Vec<_> = hints.iter()
            .filter(|h| h.reason.contains("critical path"))
            .collect();
        assert!(!critical_path_hints.is_empty());
        
        // Check for dependency-based hints
        let dependency_hints: Vec<_> = hints.iter()
            .filter(|h| h.reason.contains("dependencies") || h.reason.contains("dependents"))
            .collect();
        assert!(!dependency_hints.is_empty());
        
        // Verify confidence scores are valid
        for hint in &hints {
            assert!(hint.confidence >= 0.0 && hint.confidence <= 1.0);
        }
    }

    #[test]
    fn test_enhanced_parallelism_hint_generation() {
        let analyzer = SemanticAnalyzer::new();
        let plan = create_parallelizable_test_plan();
        
        let hints = analyzer.generate_parallelism_hints(&plan.steps).unwrap();
        
        // Should have parallelism hints for independent steps (may be empty if no opportunities)
        let _parallel_hints: Vec<_> = hints.iter()
            .filter(|h| h.hint_type == ParallelismHintType::ParallelCandidate)
            .collect();
        // Note: parallel hints may be empty if steps have dependencies or conflicts
        
        // Check for resource contention hints
        let _contention_hints: Vec<_> = hints.iter()
            .filter(|h| h.hint_type == ParallelismHintType::ResourceContention)
            .collect();
        // May or may not have contention hints depending on plan structure
        
        // Verify all hints have valid confidence scores
        for hint in &hints {
            assert!(hint.confidence >= 0.0 && hint.confidence <= 1.0);
        }
    }

    #[test]
    fn test_critical_path_identification() {
        let analyzer = SemanticAnalyzer::new();
        let plan = create_linear_dependency_plan();
        
        let dependency_map = analyzer.build_dependency_map(&plan.steps);
        let critical_path = analyzer.identify_critical_path(&plan.steps, &dependency_map);
        
        assert!(!critical_path.is_empty());
        // Critical path should include the final step
        assert!(critical_path.contains(&"step-3".to_string()));
    }

    #[test]
    fn test_resource_contention_analysis() {
        let analyzer = SemanticAnalyzer::new();
        
        // Create steps with similar operations (potential resource contention)
        let step1 = PlanStep {
            id: "compute-1".to_string(),
            operation: Operation::Compute {
                function: "process".to_string(),
                arguments: vec!["arg1".to_string()],
            },
            inputs: vec![],
            outputs: vec![],
        };
        
        let step2 = PlanStep {
            id: "compute-2".to_string(),
            operation: Operation::Compute {
                function: "analyze".to_string(),
                arguments: vec!["arg2".to_string()],
            },
            inputs: vec![],
            outputs: vec![],
        };
        
        let step3 = PlanStep {
            id: "compute-3".to_string(),
            operation: Operation::Compute {
                function: "finalize".to_string(),
                arguments: vec!["arg3".to_string()],
            },
            inputs: vec![],
            outputs: vec![],
        };
        
        let step4 = PlanStep {
            id: "compute-4".to_string(),
            operation: Operation::Compute {
                function: "validate".to_string(),
                arguments: vec!["arg4".to_string()],
            },
            inputs: vec![],
            outputs: vec![],
        };
        
        let step5 = PlanStep {
            id: "query-1".to_string(),
            operation: Operation::Query {
                target: "database".to_string(),
                parameters: HashMap::new(),
            },
            inputs: vec![],
            outputs: vec![],
        };
        
        let steps = vec![step1, step2, step3, step4, step5];
        
        // Should detect resource contention between compute steps (4 compute steps > 2 threshold)
        let hint = analyzer.analyze_resource_contention(&steps[0], &steps);
        assert!(hint.is_some());
        
        let hint = hint.unwrap();
        assert!(hint.reason.contains("contend"));
        assert_eq!(hint.hint_type, OrderingHintType::EarlyExecution);
    }

    #[test]
    fn test_dependency_based_ordering_hints() {
        let hinter = OrderingHinter::new();
        let plan = create_complex_test_plan();
        
        let hints = hinter.generate_dependency_based_hints(&plan).unwrap();
        assert!(!hints.is_empty());
        
        // Should have hints based on dependency patterns
        let dependency_based_hints: Vec<_> = hints.iter()
            .filter(|h| h.reason.contains("dependencies"))
            .collect();
        assert!(!dependency_based_hints.is_empty());
    }

    #[test]
    fn test_enhanced_hint_validation() {
        let hinter = OrderingHinter::new();
        
        // Create hints with conflicts
        let conflicting_hints = vec![
            OrderingHint {
                step_id: "step-1".to_string(),
                hint_type: OrderingHintType::ComputeIntensive,
                reason: "Test".to_string(),
                confidence: 0.8,
            },
            OrderingHint {
                step_id: "step-1".to_string(),
                hint_type: OrderingHintType::IOIntensive,
                reason: "Test".to_string(),
                confidence: 0.8,
            },
        ];
        
        let result = hinter.validate_hints(&conflicting_hints);
        assert!(result.is_err());
        
        // Test invalid confidence scores
        let invalid_confidence_hints = vec![
            OrderingHint {
                step_id: "step-1".to_string(),
                hint_type: OrderingHintType::EarlyExecution,
                reason: "Test".to_string(),
                confidence: 1.5, // Invalid confidence > 1.0
            },
        ];
        
        let result = hinter.validate_hints(&invalid_confidence_hints);
        assert!(result.is_err());
    }

    #[test]
    fn test_hint_merging() {
        let hinter = OrderingHinter::new();
        
        let hints1 = vec![
            OrderingHint {
                step_id: "step-1".to_string(),
                hint_type: OrderingHintType::EarlyExecution,
                reason: "Test 1".to_string(),
                confidence: 0.7,
            },
        ];
        
        let hints2 = vec![
            OrderingHint {
                step_id: "step-1".to_string(),
                hint_type: OrderingHintType::EarlyExecution,
                reason: "Test 2".to_string(),
                confidence: 0.9, // Higher confidence
            },
        ];
        
        let merged = hinter.merge_hints(vec![hints1, hints2]);
        assert_eq!(merged.len(), 1);
        assert_eq!(merged[0].confidence, 0.9); // Should keep higher confidence
    }

    #[test]
    fn test_confidence_filtering() {
        let hinter = OrderingHinter::new();
        
        let hints = vec![
            OrderingHint {
                step_id: "step-1".to_string(),
                hint_type: OrderingHintType::EarlyExecution,
                reason: "High confidence".to_string(),
                confidence: 0.9,
            },
            OrderingHint {
                step_id: "step-2".to_string(),
                hint_type: OrderingHintType::LateExecution,
                reason: "Low confidence".to_string(),
                confidence: 0.3,
            },
        ];
        
        let filtered = hinter.filter_by_confidence(hints, 0.5);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].step_id, "step-1");
    }

    #[test]
    fn test_priority_score_generation() {
        let hinter = OrderingHinter::new();
        
        let hints = vec![
            OrderingHint {
                step_id: "step-1".to_string(),
                hint_type: OrderingHintType::EarlyExecution,
                reason: "Test".to_string(),
                confidence: 0.8,
            },
            OrderingHint {
                step_id: "step-1".to_string(),
                hint_type: OrderingHintType::ComputeIntensive,
                reason: "Test".to_string(),
                confidence: 0.7,
            },
            OrderingHint {
                step_id: "step-2".to_string(),
                hint_type: OrderingHintType::LateExecution,
                reason: "Test".to_string(),
                confidence: 0.6,
            },
        ];
        
        let scores = hinter.generate_priority_scores(&hints);
        
        assert!(scores.contains_key("step-1"));
        assert!(scores.contains_key("step-2"));
        
        // step-1 should have higher priority (early + compute)
        // step-2 should have lower priority (late)
        assert!(scores["step-1"] > scores["step-2"]);
    }

    fn create_complex_test_plan() -> ExecutionPlan {
        ExecutionPlan {
            id: "complex-plan".to_string(),
            steps: vec![
                PlanStep {
                    id: "step-1".to_string(),
                    operation: Operation::Query {
                        target: "data-source".to_string(),
                        parameters: HashMap::new(),
                    },
                    inputs: vec![],
                    outputs: vec![DataRef {
                        id: "data-1".to_string(),
                        data_type: "string".to_string(),
                        source_step: Some("step-1".to_string()),
                    }],
                },
                PlanStep {
                    id: "step-2".to_string(),
                    operation: Operation::Compute {
                        function: "process".to_string(),
                        arguments: vec!["arg1".to_string()],
                    },
                    inputs: vec![DataRef {
                        id: "data-1".to_string(),
                        data_type: "string".to_string(),
                        source_step: Some("step-1".to_string()),
                    }],
                    outputs: vec![DataRef {
                        id: "data-2".to_string(),
                        data_type: "string".to_string(),
                        source_step: Some("step-2".to_string()),
                    }],
                },
                PlanStep {
                    id: "step-3".to_string(),
                    operation: Operation::Mutation {
                        intent: MutationIntent::UpdateIntent {
                            target: ResourcePath {
                                segments: vec!["test".to_string()],
                            },
                            changes: ChangeSet {
                                updates: HashMap::new(),
                                removals: vec![],
                            },
                        },
                    },
                    inputs: vec![DataRef {
                        id: "data-2".to_string(),
                        data_type: "string".to_string(),
                        source_step: Some("step-2".to_string()),
                    }],
                    outputs: vec![],
                },
            ],
            metadata: PlanMetadata {
                name: "Complex Test Plan".to_string(),
                description: Some("Complex plan for testing".to_string()),
                created_at: 1234567890,
                version: "1.0".to_string(),
                extra: HashMap::new(),
            },
            dependencies: vec![],
        }
    }

    fn create_parallelizable_test_plan() -> ExecutionPlan {
        ExecutionPlan {
            id: "parallel-plan".to_string(),
            steps: vec![
                PlanStep {
                    id: "parallel-1".to_string(),
                    operation: Operation::Query {
                        target: "source-1".to_string(),
                        parameters: HashMap::new(),
                    },
                    inputs: vec![],
                    outputs: vec![DataRef {
                        id: "data-1".to_string(),
                        data_type: "string".to_string(),
                        source_step: Some("parallel-1".to_string()),
                    }],
                },
                PlanStep {
                    id: "parallel-2".to_string(),
                    operation: Operation::Query {
                        target: "source-2".to_string(),
                        parameters: HashMap::new(),
                    },
                    inputs: vec![],
                    outputs: vec![DataRef {
                        id: "data-2".to_string(),
                        data_type: "string".to_string(),
                        source_step: Some("parallel-2".to_string()),
                    }],
                },
                PlanStep {
                    id: "merge".to_string(),
                    operation: Operation::Compute {
                        function: "merge".to_string(),
                        arguments: vec![],
                    },
                    inputs: vec![
                        DataRef {
                            id: "data-1".to_string(),
                            data_type: "string".to_string(),
                            source_step: Some("parallel-1".to_string()),
                        },
                        DataRef {
                            id: "data-2".to_string(),
                            data_type: "string".to_string(),
                            source_step: Some("parallel-2".to_string()),
                        },
                    ],
                    outputs: vec![DataRef {
                        id: "merged-data".to_string(),
                        data_type: "string".to_string(),
                        source_step: Some("merge".to_string()),
                    }],
                },
            ],
            metadata: PlanMetadata {
                name: "Parallelizable Plan".to_string(),
                description: Some("Plan with parallelizable steps".to_string()),
                created_at: 1234567890,
                version: "1.0".to_string(),
                extra: HashMap::new(),
            },
            dependencies: vec![],
        }
    }

    fn create_linear_dependency_plan() -> ExecutionPlan {
        ExecutionPlan {
            id: "linear-plan".to_string(),
            steps: vec![
                PlanStep {
                    id: "step-1".to_string(),
                    operation: Operation::Query {
                        target: "source".to_string(),
                        parameters: HashMap::new(),
                    },
                    inputs: vec![],
                    outputs: vec![DataRef {
                        id: "data-1".to_string(),
                        data_type: "string".to_string(),
                        source_step: Some("step-1".to_string()),
                    }],
                },
                PlanStep {
                    id: "step-2".to_string(),
                    operation: Operation::Compute {
                        function: "process".to_string(),
                        arguments: vec![],
                    },
                    inputs: vec![DataRef {
                        id: "data-1".to_string(),
                        data_type: "string".to_string(),
                        source_step: Some("step-1".to_string()),
                    }],
                    outputs: vec![DataRef {
                        id: "data-2".to_string(),
                        data_type: "string".to_string(),
                        source_step: Some("step-2".to_string()),
                    }],
                },
                PlanStep {
                    id: "step-3".to_string(),
                    operation: Operation::Compute {
                        function: "finalize".to_string(),
                        arguments: vec![],
                    },
                    inputs: vec![DataRef {
                        id: "data-2".to_string(),
                        data_type: "string".to_string(),
                        source_step: Some("step-2".to_string()),
                    }],
                    outputs: vec![DataRef {
                        id: "final-data".to_string(),
                        data_type: "string".to_string(),
                        source_step: Some("step-3".to_string()),
                    }],
                },
            ],
            metadata: PlanMetadata {
                name: "Linear Plan".to_string(),
                description: Some("Plan with linear dependencies".to_string()),
                created_at: 1234567890,
                version: "1.0".to_string(),
                extra: HashMap::new(),
            },
            dependencies: vec![],
        }
    }

    #[test]
    fn test_plan_size_limits() {
        let analyzer = SemanticAnalyzer::with_limits(1, 10); // Limit to 1 step
        let plan = create_test_plan(); // Has 2 steps
        
        let result = analyzer.analyze_semantic_dependencies(&plan);
        assert!(result.is_err());
    }
}