//! # REPL Semantic Visibility
//!
//! Provide plan visualization and explanation within capability bounds.
//!
//! **ARCHITECTURAL RULE:**
//! This module MUST NOT depend on higher-level Gate C components.
//! Violations are considered architecture breaks.
//!
//! **Author:** Kenan AY  
//! **Phase:** 3.5 Gate C

use crate::gate_c::{
    error::GateCResult,
    types::{ExecutionPlan, PlanStep, Operation, DataRef, Dependency, MutationIntent, InvalidationReason},
    limits::{MAX_RENDER_NODES, MAX_INSPECT_OUTPUT_BYTES},
    security_ops::{RedactionEngine, CapabilityScope},
};
use std::collections::{HashMap, HashSet};
use serde::{Serialize, Deserialize};

/// Plan visualization for REPL with bounded rendering
pub struct REPLVisualizer {
    /// Visualization configuration
    config: VisualizationConfig,
    /// Redaction engine for capability-based filtering
    redaction_engine: RedactionEngine,
}

/// Visualization configuration
#[derive(Debug, Clone)]
pub struct VisualizationConfig {
    /// Maximum nodes to render
    pub max_render_nodes: usize,
    /// Maximum output size in bytes
    pub max_output_size: usize,
    /// Enable detailed view
    pub detailed_view: bool,
    /// Enable dry-run mode
    pub dry_run_mode: bool,
    /// Show dependency graph
    pub show_dependencies: bool,
}

impl Default for VisualizationConfig {
    fn default() -> Self {
        Self {
            max_render_nodes: MAX_RENDER_NODES,
            max_output_size: MAX_INSPECT_OUTPUT_BYTES,
            detailed_view: true,
            dry_run_mode: true,
            show_dependencies: true,
        }
    }
}

/// Bounded renderer for plan visualization
pub struct BoundedRenderer {
    /// Rendering configuration
    config: RenderingConfig,
    /// Node count tracker
    node_count: usize,
    /// Output size tracker
    output_size: usize,
}

/// Rendering configuration
#[derive(Debug, Clone)]
pub struct RenderingConfig {
    /// Maximum nodes to render
    pub max_nodes: usize,
    /// Maximum output size
    pub max_output_size: usize,
    /// Rendering style
    pub style: RenderingStyle,
    /// Include metadata
    pub include_metadata: bool,
}

/// Rendering style options
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RenderingStyle {
    /// Compact text representation
    Compact,
    /// Detailed text representation
    Detailed,
    /// Tree-like structure
    Tree,
    /// Summary view
    Summary,
}

impl Default for RenderingConfig {
    fn default() -> Self {
        Self {
            max_nodes: MAX_RENDER_NODES,
            max_output_size: MAX_INSPECT_OUTPUT_BYTES,
            style: RenderingStyle::Detailed,
            include_metadata: true,
        }
    }
}

/// Plan visualization result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanVisualization {
    /// Plan identifier
    pub plan_id: String,
    /// Rendered content
    pub content: String,
    /// Visualization metadata
    pub metadata: VisualizationMetadata,
    /// Summary information
    pub summary: PlanSummary,
}

/// Visualization metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualizationMetadata {
    /// Number of nodes rendered
    pub nodes_rendered: usize,
    /// Total nodes in plan
    pub total_nodes: usize,
    /// Output size in bytes
    pub output_size: usize,
    /// Whether output was truncated
    pub truncated: bool,
    /// Rendering duration in milliseconds
    pub render_duration_ms: u64,
    /// Rendering style used
    pub style: String,
}

/// Plan summary information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanSummary {
    /// Total number of steps
    pub total_steps: usize,
    /// Number of query operations
    pub query_operations: usize,
    /// Number of mutation operations
    pub mutation_operations: usize,
    /// Number of compute operations
    pub compute_operations: usize,
    /// Number of dependencies
    pub total_dependencies: usize,
    /// Complexity score
    pub complexity_score: f64,
}

/// Dry-run preview result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DryRunPreview {
    /// Plan identifier
    pub plan_id: String,
    /// Preview content
    pub preview: String,
    /// Execution flow preview
    pub execution_flow: Vec<StepPreview>,
    /// Data flow preview
    pub data_flow: Vec<DataFlowEdge>,
    /// Preview metadata
    pub metadata: PreviewMetadata,
}

/// Step preview information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepPreview {
    /// Step identifier
    pub step_id: String,
    /// Step description
    pub description: String,
    /// Operation type
    pub operation_type: String,
    /// Input data references
    pub inputs: Vec<String>,
    /// Output data references
    pub outputs: Vec<String>,
    /// Dependencies
    pub dependencies: Vec<String>,
}

/// Data flow edge
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataFlowEdge {
    /// Source step
    pub from_step: String,
    /// Target step
    pub to_step: String,
    /// Data reference
    pub data_ref: String,
    /// Data type
    pub data_type: String,
}

/// Preview metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreviewMetadata {
    /// Preview generation time
    pub generated_at: u64,
    /// Preview duration
    pub generation_duration_ms: u64,
    /// Number of steps previewed
    pub steps_previewed: usize,
    /// Whether preview was truncated
    pub truncated: bool,
}

/// Semantic explanation for REPL
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticExplanation {
    /// Plan identifier
    pub plan_id: String,
    /// Explanation content
    pub explanation: String,
    /// Interactive explanation sections
    pub sections: Vec<ExplanationSection>,
    /// Explanation metadata
    pub metadata: ExplanationMetadata,
}

/// Explanation section for interactive exploration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExplanationSection {
    /// Section identifier
    pub section_id: String,
    /// Section title
    pub title: String,
    /// Section content
    pub content: String,
    /// Section type
    pub section_type: ExplanationSectionType,
    /// Related step IDs
    pub related_steps: Vec<String>,
    /// Subsections
    pub subsections: Vec<ExplanationSection>,
}

/// Types of explanation sections
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExplanationSectionType {
    /// Overview of the entire plan
    Overview,
    /// Data flow explanation
    DataFlow,
    /// Operation sequence explanation
    OperationSequence,
    /// Dependency analysis
    DependencyAnalysis,
    /// Performance characteristics
    PerformanceCharacteristics,
    /// Security considerations
    SecurityConsiderations,
}

/// Explanation metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExplanationMetadata {
    /// Generation timestamp
    pub generated_at: u64,
    /// Generation duration in milliseconds
    pub generation_duration_ms: u64,
    /// Number of sections generated
    pub sections_generated: usize,
    /// Total explanation size in bytes
    pub explanation_size: usize,
    /// Whether explanation was truncated
    pub truncated: bool,
    /// Capability scope used for redaction
    pub capability_scope: String,
}

/// Interactive explanation features
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractiveFeatures {
    /// Available drill-down sections
    pub drill_down_sections: Vec<String>,
    /// Step-by-step navigation
    pub step_navigation: Vec<NavigationStep>,
    /// Cross-references between sections
    pub cross_references: HashMap<String, Vec<String>>,
}

/// Navigation step for interactive exploration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NavigationStep {
    /// Step identifier
    pub step_id: String,
    /// Step title
    pub title: String,
    /// Step description
    pub description: String,
    /// Related sections
    pub related_sections: Vec<String>,
}

impl REPLVisualizer {
    /// Create new REPL visualizer
    pub fn new() -> Self {
        Self {
            config: VisualizationConfig::default(),
            redaction_engine: RedactionEngine::new(),
        }
    }
    
    /// Create REPL visualizer with custom configuration
    pub fn with_config(config: VisualizationConfig) -> Self {
        Self {
            config,
            redaction_engine: RedactionEngine::new(),
        }
    }
    
    /// Add capability filter for redaction
    pub fn add_capability_filter(&mut self, capability: String, scope: CapabilityScope) {
        self.redaction_engine.add_capability_filter(capability, scope);
    }
    
    /// Visualize execution plan
    pub fn visualize_plan(&self, plan: &ExecutionPlan) -> GateCResult<PlanVisualization> {
        // CONSTITUTIONAL FIX: Use deterministic timing instead of actual timing
        
        // CRITICAL FIX: Add summary view fallback for large plans
        let use_summary_view = plan.steps.len() > self.config.max_render_nodes / 2 ||
                              self.estimate_plan_size(plan) > self.config.max_output_size / 2;
        
        // Create bounded renderer with appropriate style
        let mut renderer = BoundedRenderer::new(RenderingConfig {
            max_nodes: self.config.max_render_nodes,
            max_output_size: self.config.max_output_size,
            style: if use_summary_view {
                RenderingStyle::Summary // CRITICAL FIX: Use summary for large plans
            } else if self.config.detailed_view { 
                RenderingStyle::Detailed 
            } else { 
                RenderingStyle::Compact 
            },
            include_metadata: true,
        });
        
        // Generate plan summary
        let summary = self.generate_plan_summary(plan)?;
        
        // Render plan content
        let content = renderer.render_plan(plan)?;
        
        // CONSTITUTIONAL FIX: Use deterministic duration instead of actual timing
        use crate::gate_c::deterministic::deterministic_duration_ms;
        let render_duration_ms = deterministic_duration_ms("repl_visualization", &plan.id);
        
        let metadata = VisualizationMetadata {
            nodes_rendered: renderer.node_count(),
            total_nodes: plan.steps.len(),
            output_size: content.len(),
            truncated: renderer.was_truncated(),
            render_duration_ms,
            style: match renderer.config.style {
                RenderingStyle::Compact => "Compact".to_string(),
                RenderingStyle::Detailed => "Detailed".to_string(),
                RenderingStyle::Tree => "Tree".to_string(),
                RenderingStyle::Summary => "Summary".to_string(),
            },
        };
        
        Ok(PlanVisualization {
            plan_id: plan.id.clone(),
            content,
            metadata,
            summary,
        })
    }
    
    /// Generate dry-run preview
    pub fn dry_run_preview(&self, plan: &ExecutionPlan) -> GateCResult<DryRunPreview> {
        // CONSTITUTIONAL FIX: Use deterministic timing instead of actual timing
        
        // Generate execution flow preview
        let execution_flow = self.generate_execution_flow_preview(plan)?;
        
        // Generate data flow preview
        let data_flow = self.generate_data_flow_preview(plan)?;
        
        // Generate preview content
        let preview = self.generate_preview_content(plan, &execution_flow, &data_flow)?;
        
        let metadata = PreviewMetadata {
            // DETERMINISM FIX: Use deterministic timestamp based on plan content
            generated_at: crate::gate_c::deterministic::deterministic_timestamp_from_plan_id(&plan.id),
            // DETERMINISM FIX: Use deterministic duration based on plan content
            generation_duration_ms: crate::gate_c::deterministic::deterministic_duration_ms("preview_generation", &plan.id),
            steps_previewed: execution_flow.len(),
            truncated: execution_flow.len() < plan.steps.len(),
        };
        
        Ok(DryRunPreview {
            plan_id: plan.id.clone(),
            preview,
            execution_flow,
            data_flow,
            metadata,
        })
    }
    
    /// Generate plan summary
    fn generate_plan_summary(&self, plan: &ExecutionPlan) -> GateCResult<PlanSummary> {
        let mut query_ops = 0;
        let mut mutation_ops = 0;
        let mut compute_ops = 0;
        
        for step in &plan.steps {
            match &step.operation {
                Operation::Query { .. } => query_ops += 1,
                Operation::Mutation { .. } => mutation_ops += 1,
                Operation::Compute { .. } => compute_ops += 1,
            }
        }
        
        // Calculate complexity score
        let complexity_score = self.calculate_complexity_score(plan);
        
        Ok(PlanSummary {
            total_steps: plan.steps.len(),
            query_operations: query_ops,
            mutation_operations: mutation_ops,
            compute_operations: compute_ops,
            total_dependencies: plan.dependencies.len(),
            complexity_score,
        })
    }
    
    /// Calculate plan complexity score
    fn calculate_complexity_score(&self, plan: &ExecutionPlan) -> f64 {
        let step_count = plan.steps.len() as f64;
        let dependency_count = plan.dependencies.len() as f64;
        
        // Count data references
        let mut total_data_refs = 0;
        for step in &plan.steps {
            total_data_refs += step.inputs.len() + step.outputs.len();
        }
        
        // Simple complexity formula
        let base_complexity = step_count * 1.0;
        let dependency_complexity = dependency_count * 0.5;
        let data_complexity = (total_data_refs as f64) * 0.1;
        
        base_complexity + dependency_complexity + data_complexity
    }
    
    /// Estimate plan rendering size for summary fallback decision
    fn estimate_plan_size(&self, plan: &ExecutionPlan) -> usize {
        let mut estimated_size = 0;
        
        // Base plan info
        estimated_size += plan.id.len() + plan.metadata.name.len();
        
        // Estimate step content size
        for step in &plan.steps {
            estimated_size += step.id.len() + 50; // Base step overhead
            
            match &step.operation {
                Operation::Query { target, parameters } => {
                    estimated_size += target.len() + parameters.len() * 20;
                }
                Operation::Compute { function, arguments } => {
                    estimated_size += function.len() + arguments.len() * 15;
                }
                Operation::Mutation { .. } => {
                    estimated_size += 100; // Mutation overhead
                }
            }
            
            // Data references
            estimated_size += (step.inputs.len() + step.outputs.len()) * 25;
        }
        
        // Dependencies
        estimated_size += plan.dependencies.len() * 30;
        
        estimated_size
    }
    
    /// Generate execution flow preview
    fn generate_execution_flow_preview(&self, plan: &ExecutionPlan) -> GateCResult<Vec<StepPreview>> {
        let mut previews = Vec::new();
        
        for step in &plan.steps {
            let description = self.generate_step_description(step);
            let operation_type = match &step.operation {
                Operation::Query { .. } => "Query".to_string(),
                Operation::Mutation { .. } => "Mutation".to_string(),
                Operation::Compute { .. } => "Compute".to_string(),
            };
            
            let inputs: Vec<String> = step.inputs.iter().map(|r| r.id.clone()).collect();
            let outputs: Vec<String> = step.outputs.iter().map(|r| r.id.clone()).collect();
            
            // Find dependencies (steps that produce our inputs)
            let mut dependencies = Vec::new();
            for input in &step.inputs {
                if let Some(source_step) = &input.source_step {
                    dependencies.push(source_step.clone());
                }
            }
            
            previews.push(StepPreview {
                step_id: step.id.clone(),
                description,
                operation_type,
                inputs,
                outputs,
                dependencies,
            });
            
            // Limit preview size
            if previews.len() >= self.config.max_render_nodes {
                break;
            }
        }
        
        Ok(previews)
    }
    
    /// Generate data flow preview
    fn generate_data_flow_preview(&self, plan: &ExecutionPlan) -> GateCResult<Vec<DataFlowEdge>> {
        let mut edges = Vec::new();
        
        for step in &plan.steps {
            for input in &step.inputs {
                if let Some(source_step) = &input.source_step {
                    edges.push(DataFlowEdge {
                        from_step: source_step.clone(),
                        to_step: step.id.clone(),
                        data_ref: input.id.clone(),
                        data_type: input.data_type.clone(),
                    });
                }
            }
        }
        
        Ok(edges)
    }
    
    /// Generate step description
    fn generate_step_description(&self, step: &PlanStep) -> String {
        match &step.operation {
            Operation::Query { target, parameters } => {
                if parameters.is_empty() {
                    format!("Query data from {}", target)
                } else {
                    format!("Query data from {} with {} parameters", target, parameters.len())
                }
            }
            Operation::Mutation { intent } => {
                match intent {
                    MutationIntent::InvalidateIntent { target, reason } => {
                        format!("Invalidate resource {} (reason: {})", 
                               target, 
                               match reason {
                                   InvalidationReason::Obsolete => "obsolete",
                                   InvalidationReason::Conflict => "conflict", 
                                   InvalidationReason::ConstraintViolation => "constraint violation",
                                   InvalidationReason::Custom(s) => s,
                               })
                    }
                    MutationIntent::UpdateIntent { target, changes } => {
                        format!("Update resource {} ({} updates, {} removals)", 
                               target, changes.updates.len(), changes.removals.len())
                    }
                    MutationIntent::CreateIntent { path, content } => {
                        format!("Create resource {} (type: {})", path, content.content_type)
                    }
                }
            }
            Operation::Compute { function, arguments } => {
                format!("Execute function '{}' with {} arguments", function, arguments.len())
            }
        }
    }
    
    /// Generate explanation content from sections
    fn generate_explanation_content(&self, plan: &ExecutionPlan, sections: &[ExplanationSection]) -> GateCResult<(String, bool)> {
        let mut content = String::new();
        
        content.push_str(&format!("=== Semantic Explanation: {} ===\n\n", plan.id));
        
        for section in sections {
            content.push_str(&format!("## {}\n\n", section.title));
            content.push_str(&section.content);
            content.push_str("\n\n");
            
            // Add subsections if any
            for subsection in &section.subsections {
                content.push_str(&format!("### {}\n\n", subsection.title));
                content.push_str(&subsection.content);
                content.push_str("\n\n");
            }
        }
        
        // Check if truncation is needed
        let was_truncated = content.len() > self.config.max_output_size;
        
        // Ensure size limits
        if was_truncated {
            content.truncate(self.config.max_output_size - 50);
            content.push_str("\n\n... (explanation truncated due to size limits)");
        }
        
        Ok((content, was_truncated))
    }
    
    /// Apply capability-based redaction to explanation sections
    fn redact_explanation_sections(&self, sections: &[ExplanationSection]) -> GateCResult<Vec<ExplanationSection>> {
        let mut redacted_sections = Vec::new();
        
        for section in sections {
            // Apply redaction based on section type and capability scope
            let redacted_content = match section.section_type {
                ExplanationSectionType::SecurityConsiderations => {
                    // Security sections require higher capability scope
                    self.redaction_engine.redact_security_content(&section.content)?
                }
                _ => {
                    // Other sections use standard redaction
                    self.redaction_engine.redact_explanation(&section.content)?
                }
            };
            
            let mut redacted_section = section.clone();
            redacted_section.content = redacted_content;
            
            // Recursively redact subsections
            if !section.subsections.is_empty() {
                redacted_section.subsections = self.redact_explanation_sections(&section.subsections)?;
            }
            
            redacted_sections.push(redacted_section);
        }
        
        Ok(redacted_sections)
    }
    
    /// Determine execution order based on dependencies
    fn determine_execution_order(&self, plan: &ExecutionPlan) -> GateCResult<Vec<String>> {
        let mut order = Vec::new();
        let mut visited = HashSet::new();
        let mut visiting = HashSet::new();
        
        // Build dependency map
        let mut dependencies: HashMap<String, Vec<String>> = HashMap::new();
        for step in &plan.steps {
            dependencies.insert(step.id.clone(), Vec::new());
        }
        
        for dep in &plan.dependencies {
            dependencies.entry(dep.to.clone())
                .or_insert_with(Vec::new)
                .push(dep.from.clone());
        }
        
        // Topological sort with cycle detection
        fn visit(step_id: &str, 
                dependencies: &HashMap<String, Vec<String>>,
                visited: &mut HashSet<String>,
                visiting: &mut HashSet<String>,
                order: &mut Vec<String>) -> GateCResult<()> {
            
            if visiting.contains(step_id) {
                return Err(crate::gate_c::error::GateCError::Pipeline(
                    crate::gate_c::error::PipelineError::CycleDetected(
                        format!("Circular dependency detected involving step: {}", step_id)
                    )
                ));
            }
            
            if visited.contains(step_id) {
                return Ok(());
            }
            
            visiting.insert(step_id.to_string());
            
            if let Some(deps) = dependencies.get(step_id) {
                for dep in deps {
                    visit(dep, dependencies, visited, visiting, order)?;
                }
            }
            
            visiting.remove(step_id);
            visited.insert(step_id.to_string());
            order.push(step_id.to_string());
            
            Ok(())
        }
        
        for step in &plan.steps {
            if !visited.contains(&step.id) {
                visit(&step.id, &dependencies, &mut visited, &mut visiting, &mut order)?;
            }
        }
        
        Ok(order)
    }
    
    /// Identify parallel execution groups
    fn identify_parallel_groups(&self, plan: &ExecutionPlan) -> GateCResult<Vec<Vec<String>>> {
        let execution_order = self.determine_execution_order(plan)?;
        let mut groups = Vec::new();
        let mut current_group = Vec::new();
        
        // Build dependency map for quick lookup
        let mut dependents: HashMap<String, HashSet<String>> = HashMap::new();
        for dep in &plan.dependencies {
            dependents.entry(dep.from.clone())
                .or_insert_with(HashSet::new)
                .insert(dep.to.clone());
        }
        
        for step_id in execution_order {
            // Check if this step can run in parallel with current group
            let can_parallelize = current_group.iter().all(|group_step| {
                // No direct dependency between steps
                !dependents.get(group_step).map_or(false, |deps| deps.contains(&step_id)) &&
                !dependents.get(&step_id).map_or(false, |deps| deps.contains(group_step))
            });
            
            if can_parallelize && !current_group.is_empty() {
                current_group.push(step_id);
            } else {
                if !current_group.is_empty() {
                    groups.push(current_group);
                }
                current_group = vec![step_id];
            }
        }
        
        if !current_group.is_empty() {
            groups.push(current_group);
        }
        
        Ok(groups)
    }
    
    /// Find critical path through the plan
    fn find_critical_path(&self, plan: &ExecutionPlan) -> GateCResult<Vec<String>> {
        // Build dependency graph
        let mut graph: HashMap<String, Vec<String>> = HashMap::new();
        let mut reverse_graph: HashMap<String, Vec<String>> = HashMap::new();
        
        for step in &plan.steps {
            graph.insert(step.id.clone(), Vec::new());
            reverse_graph.insert(step.id.clone(), Vec::new());
        }
        
        for dep in &plan.dependencies {
            graph.entry(dep.from.clone())
                .or_insert_with(Vec::new)
                .push(dep.to.clone());
            reverse_graph.entry(dep.to.clone())
                .or_insert_with(Vec::new)
                .push(dep.from.clone());
        }
        
        // Find longest path (critical path)
        let mut distances: HashMap<String, usize> = HashMap::new();
        let mut predecessors: HashMap<String, Option<String>> = HashMap::new();
        
        // Initialize distances
        for step in &plan.steps {
            distances.insert(step.id.clone(), 0);
            predecessors.insert(step.id.clone(), None);
        }
        
        // Topological sort and longest path calculation
        let execution_order = self.determine_execution_order(plan)?;
        
        for step_id in &execution_order {
            if let Some(deps) = reverse_graph.get(step_id) {
                for dep in deps {
                    let new_distance = distances.get(dep).unwrap_or(&0) + 1;
                    if new_distance > *distances.get(step_id).unwrap_or(&0) {
                        distances.insert(step_id.clone(), new_distance);
                        predecessors.insert(step_id.clone(), Some(dep.clone()));
                    }
                }
            }
        }
        
        // Find the step with maximum distance (end of critical path)
        let end_step = distances.iter()
            .max_by_key(|(_, &distance)| distance)
            .map(|(step_id, _)| step_id.clone())
            .unwrap_or_else(|| plan.steps[0].id.clone());
        
        // Reconstruct critical path
        let mut path = Vec::new();
        let mut current = Some(end_step);
        
        while let Some(step_id) = current {
            path.push(step_id.clone());
            current = predecessors.get(&step_id).and_then(|pred| pred.clone());
        }
        
        path.reverse();
        Ok(path)
    }
    
    /// Find dependency bottlenecks
    fn find_dependency_bottlenecks(&self, plan: &ExecutionPlan) -> GateCResult<Vec<String>> {
        let mut fan_in: HashMap<String, usize> = HashMap::new();
        let mut fan_out: HashMap<String, usize> = HashMap::new();
        
        // Initialize counters
        for step in &plan.steps {
            fan_in.insert(step.id.clone(), 0);
            fan_out.insert(step.id.clone(), 0);
        }
        
        // Count dependencies
        for dep in &plan.dependencies {
            *fan_out.entry(dep.from.clone()).or_insert(0) += 1;
            *fan_in.entry(dep.to.clone()).or_insert(0) += 1;
        }
        
        // Identify bottlenecks (high fan-in or fan-out)
        let mut bottlenecks = Vec::new();
        let threshold = (plan.dependencies.len() as f64 / plan.steps.len() as f64 * 2.0) as usize;
        
        for step in &plan.steps {
            let in_count = fan_in.get(&step.id).unwrap_or(&0);
            let out_count = fan_out.get(&step.id).unwrap_or(&0);
            
            if *in_count > threshold || *out_count > threshold {
                bottlenecks.push(step.id.clone());
            }
        }
        
        Ok(bottlenecks)
    }
    
    /// Estimate maximum parallel width
    fn estimate_parallel_width(&self, plan: &ExecutionPlan) -> GateCResult<usize> {
        let parallel_groups = self.identify_parallel_groups(plan)?;
        Ok(parallel_groups.iter().map(|group| group.len()).max().unwrap_or(1))
    }
    
    /// Find sections related to a specific step
    fn find_related_sections(&self, step: &PlanStep, sections: &[ExplanationSection]) -> Vec<String> {
        sections.iter()
            .filter(|section| section.related_steps.contains(&step.id))
            .map(|section| section.section_id.clone())
            .collect()
    }
    
    /// Generate cross-references between sections
    fn generate_cross_references(&self, sections: &[ExplanationSection]) -> HashMap<String, Vec<String>> {
        let mut cross_refs = HashMap::new();
        
        for section in sections {
            let mut refs = Vec::new();
            
            // Find sections that share related steps
            for other_section in sections {
                if section.section_id != other_section.section_id {
                    let shared_steps: HashSet<_> = section.related_steps.iter()
                        .filter(|step| other_section.related_steps.contains(step))
                        .collect();
                    
                    if !shared_steps.is_empty() {
                        refs.push(other_section.section_id.clone());
                    }
                }
            }
            
            cross_refs.insert(section.section_id.clone(), refs);
        }
        
        cross_refs
    }
    
    /// Generate preview content
    fn generate_preview_content(&self, plan: &ExecutionPlan, 
                               execution_flow: &[StepPreview], 
                               data_flow: &[DataFlowEdge]) -> GateCResult<String> {
        let mut content = String::new();
        
        content.push_str(&format!("=== Dry-Run Preview: {} ===\n\n", plan.id));
        
        // Plan overview
        content.push_str("## Plan Overview\n");
        content.push_str(&format!("- Total Steps: {}\n", plan.steps.len()));
        content.push_str(&format!("- Dependencies: {}\n", plan.dependencies.len()));
        content.push_str(&format!("- Data Flow Edges: {}\n\n", data_flow.len()));
        
        // Execution flow
        content.push_str("## Execution Flow\n");
        for (i, preview) in execution_flow.iter().enumerate() {
            content.push_str(&format!("{}. {} ({})\n", i + 1, preview.step_id, preview.operation_type));
            content.push_str(&format!("   Description: {}\n", preview.description));
            if !preview.inputs.is_empty() {
                content.push_str(&format!("   Inputs: {}\n", preview.inputs.join(", ")));
            }
            if !preview.outputs.is_empty() {
                content.push_str(&format!("   Outputs: {}\n", preview.outputs.join(", ")));
            }
            content.push('\n');
        }
        
        // Data flow
        if !data_flow.is_empty() {
            content.push_str("## Data Flow\n");
            for edge in data_flow {
                content.push_str(&format!("{} -> {} ({})\n", 
                                        edge.from_step, edge.to_step, edge.data_ref));
            }
        }
        
        Ok(content)
    }
    
    /// Generate semantic explanation for plan
    pub fn generate_semantic_explanation(&self, plan: &ExecutionPlan) -> GateCResult<SemanticExplanation> {
        // CONSTITUTIONAL FIX: Use deterministic timing instead of actual timing
        
        // Generate explanation sections
        let mut sections = Vec::new();
        
        // Overview section
        sections.push(self.generate_overview_section(plan)?);
        
        // Data flow section
        sections.push(self.generate_data_flow_section(plan)?);
        
        // Operation sequence section
        sections.push(self.generate_operation_sequence_section(plan)?);
        
        // Dependency analysis section
        sections.push(self.generate_dependency_analysis_section(plan)?);
        
        // Performance characteristics section
        sections.push(self.generate_performance_section(plan)?);
        
        // Security considerations section (capability-filtered)
        sections.push(self.generate_security_section(plan)?);
        
        // Generate main explanation content
        let (explanation, content_was_truncated) = self.generate_explanation_content(plan, &sections)?;
        
        // Apply capability-based redaction
        let redacted_explanation = self.redaction_engine.redact_explanation(&explanation)?;
        let redacted_sections = self.redact_explanation_sections(&sections)?;
        
        let metadata = ExplanationMetadata {
            // DETERMINISM FIX: Use deterministic timestamp based on plan content
            generated_at: crate::gate_c::deterministic::deterministic_timestamp_from_plan_id(&plan.id),
            // DETERMINISM FIX: Use deterministic duration based on plan content
            generation_duration_ms: crate::gate_c::deterministic::deterministic_duration_ms("explanation_generation", &plan.id),
            sections_generated: redacted_sections.len(),
            explanation_size: redacted_explanation.len(),
            truncated: content_was_truncated || redacted_explanation.len() >= self.config.max_output_size,
            capability_scope: "default".to_string(), // TODO: Get from redaction engine
        };
        
        Ok(SemanticExplanation {
            plan_id: plan.id.clone(),
            explanation: redacted_explanation,
            sections: redacted_sections,
            metadata,
        })
    }
    
    /// Generate interactive explanation features
    pub fn generate_interactive_features(&self, plan: &ExecutionPlan, 
                                       explanation: &SemanticExplanation) -> GateCResult<InteractiveFeatures> {
        // Generate drill-down sections
        let drill_down_sections: Vec<String> = explanation.sections
            .iter()
            .map(|section| section.section_id.clone())
            .collect();
        
        // Generate step navigation
        let mut step_navigation = Vec::new();
        for (i, step) in plan.steps.iter().enumerate() {
            step_navigation.push(NavigationStep {
                step_id: step.id.clone(),
                title: format!("Step {}: {}", i + 1, step.id),
                description: self.generate_step_description(step),
                related_sections: self.find_related_sections(step, &explanation.sections),
            });
        }
        
        // Generate cross-references
        let cross_references = self.generate_cross_references(&explanation.sections);
        
        Ok(InteractiveFeatures {
            drill_down_sections,
            step_navigation,
            cross_references,
        })
    }
    
    /// Generate overview explanation section
    fn generate_overview_section(&self, plan: &ExecutionPlan) -> GateCResult<ExplanationSection> {
        let mut content = String::new();
        
        content.push_str(&format!("This execution plan '{}' contains {} steps ", 
                                plan.id, plan.steps.len()));
        content.push_str(&format!("with {} dependencies between them.\n\n", plan.dependencies.len()));
        
        // Analyze operation types
        let mut query_count = 0;
        let mut mutation_count = 0;
        let mut compute_count = 0;
        
        for step in &plan.steps {
            match &step.operation {
                Operation::Query { .. } => query_count += 1,
                Operation::Mutation { .. } => mutation_count += 1,
                Operation::Compute { .. } => compute_count += 1,
            }
        }
        
        content.push_str("The plan consists of:\n");
        if query_count > 0 {
            content.push_str(&format!("- {} query operations for data retrieval\n", query_count));
        }
        if mutation_count > 0 {
            content.push_str(&format!("- {} mutation operations for data modification\n", mutation_count));
        }
        if compute_count > 0 {
            content.push_str(&format!("- {} compute operations for data processing\n", compute_count));
        }
        
        // Analyze complexity
        let complexity = self.calculate_complexity_score(plan);
        content.push_str(&format!("\nComplexity score: {:.2}\n", complexity));
        
        if complexity < 5.0 {
            content.push_str("This is a simple plan with straightforward execution flow.");
        } else if complexity < 15.0 {
            content.push_str("This is a moderately complex plan with some interdependencies.");
        } else {
            content.push_str("This is a complex plan with significant interdependencies and data flow.");
        }
        
        Ok(ExplanationSection {
            section_id: "overview".to_string(),
            title: "Plan Overview".to_string(),
            content,
            section_type: ExplanationSectionType::Overview,
            related_steps: plan.steps.iter().map(|s| s.id.clone()).collect(),
            subsections: vec![],
        })
    }
    
    /// Generate data flow explanation section
    fn generate_data_flow_section(&self, plan: &ExecutionPlan) -> GateCResult<ExplanationSection> {
        let mut content = String::new();
        
        content.push_str("Data flows through the plan as follows:\n\n");
        
        // Analyze data flow patterns
        let mut data_producers: HashMap<String, Vec<String>> = HashMap::new();
        let mut data_consumers: HashMap<String, Vec<String>> = HashMap::new();
        
        for step in &plan.steps {
            // Track outputs (data production)
            for output in &step.outputs {
                data_producers.entry(output.id.clone())
                    .or_insert_with(Vec::new)
                    .push(step.id.clone());
            }
            
            // Track inputs (data consumption)
            for input in &step.inputs {
                data_consumers.entry(input.id.clone())
                    .or_insert_with(Vec::new)
                    .push(step.id.clone());
            }
        }
        
        // Describe data flow chains
        for (data_id, producers) in &data_producers {
            if let Some(consumers) = data_consumers.get(data_id) {
                content.push_str(&format!("Data '{}' is produced by {} and consumed by {}\n",
                    data_id,
                    producers.join(", "),
                    consumers.join(", ")
                ));
            } else {
                content.push_str(&format!("Data '{}' is produced by {} but not consumed (output data)\n",
                    data_id, producers.join(", ")));
            }
        }
        
        // Identify data flow patterns
        content.push_str("\nData Flow Patterns:\n");
        if data_producers.len() > data_consumers.len() {
            content.push_str("- This plan generates more data than it consumes (data expansion pattern)\n");
        } else if data_producers.len() < data_consumers.len() {
            content.push_str("- This plan consumes more data than it produces (data aggregation pattern)\n");
        } else {
            content.push_str("- This plan has balanced data production and consumption\n");
        }
        
        Ok(ExplanationSection {
            section_id: "data_flow".to_string(),
            title: "Data Flow Analysis".to_string(),
            content,
            section_type: ExplanationSectionType::DataFlow,
            related_steps: plan.steps.iter().map(|s| s.id.clone()).collect(),
            subsections: vec![],
        })
    }
    
    /// Generate operation sequence explanation section
    fn generate_operation_sequence_section(&self, plan: &ExecutionPlan) -> GateCResult<ExplanationSection> {
        let mut content = String::new();
        
        content.push_str("The operations execute in the following semantic order:\n\n");
        
        // Build execution order based on dependencies
        let execution_order = self.determine_execution_order(plan)?;
        
        for (i, step_id) in execution_order.iter().enumerate() {
            if let Some(step) = plan.steps.iter().find(|s| &s.id == step_id) {
                content.push_str(&format!("{}. {} - {}\n", 
                    i + 1, 
                    step.id, 
                    self.generate_step_description(step)
                ));
                
                // Explain why this step comes at this position
                let dependencies: Vec<String> = step.inputs.iter()
                    .filter_map(|input| input.source_step.as_ref())
                    .cloned()
                    .collect();
                
                if !dependencies.is_empty() {
                    content.push_str(&format!("   Depends on: {}\n", dependencies.join(", ")));
                }
            }
        }
        
        // Analyze parallelization opportunities
        content.push_str("\nParallelization Opportunities:\n");
        let parallel_groups = self.identify_parallel_groups(plan)?;
        for (i, group) in parallel_groups.iter().enumerate() {
            if group.len() > 1 {
                content.push_str(&format!("- Group {}: {} can execute in parallel\n", 
                    i + 1, group.join(", ")));
            }
        }
        
        Ok(ExplanationSection {
            section_id: "operation_sequence".to_string(),
            title: "Operation Sequence".to_string(),
            content,
            section_type: ExplanationSectionType::OperationSequence,
            related_steps: execution_order,
            subsections: vec![],
        })
    }
    
    /// Generate dependency analysis section
    fn generate_dependency_analysis_section(&self, plan: &ExecutionPlan) -> GateCResult<ExplanationSection> {
        let mut content = String::new();
        
        content.push_str("Dependency Analysis:\n\n");
        
        // Analyze dependency types and patterns
        let mut data_deps = 0;
        let mut control_deps = 0;
        let mut resource_deps = 0;
        
        for dep in &plan.dependencies {
            match dep.dependency_type {
                crate::gate_c::types::DependencyType::Data => data_deps += 1,
                crate::gate_c::types::DependencyType::Control => control_deps += 1,
                crate::gate_c::types::DependencyType::Resource => resource_deps += 1,
            }
        }
        
        content.push_str(&format!("Total dependencies: {}\n", plan.dependencies.len()));
        content.push_str(&format!("- Data dependencies: {}\n", data_deps));
        content.push_str(&format!("- Control dependencies: {}\n", control_deps));
        content.push_str(&format!("- Resource dependencies: {}\n", resource_deps));
        
        // Identify critical path
        let critical_path = self.find_critical_path(plan)?;
        content.push_str(&format!("\nCritical Path: {}\n", critical_path.join(" -> ")));
        content.push_str("This is the longest dependency chain that determines minimum execution time.\n");
        
        // Identify dependency bottlenecks
        let bottlenecks = self.find_dependency_bottlenecks(plan)?;
        if !bottlenecks.is_empty() {
            content.push_str("\nDependency Bottlenecks:\n");
            for bottleneck in bottlenecks {
                content.push_str(&format!("- {}: High fan-in/fan-out dependency node\n", bottleneck));
            }
        }
        
        Ok(ExplanationSection {
            section_id: "dependency_analysis".to_string(),
            title: "Dependency Analysis".to_string(),
            content,
            section_type: ExplanationSectionType::DependencyAnalysis,
            related_steps: critical_path,
            subsections: vec![],
        })
    }
    
    /// Generate performance characteristics section
    fn generate_performance_section(&self, plan: &ExecutionPlan) -> GateCResult<ExplanationSection> {
        let mut content = String::new();
        
        content.push_str("Performance Characteristics:\n\n");
        
        // Analyze computational complexity
        let step_count = plan.steps.len();
        let dependency_count = plan.dependencies.len();
        
        content.push_str(&format!("Plan Size: {} steps, {} dependencies\n", step_count, dependency_count));
        
        // Estimate execution characteristics
        let parallel_width = self.estimate_parallel_width(plan)?;
        content.push_str(&format!("Maximum Parallelism: {} concurrent operations\n", parallel_width));
        
        let critical_path_length = self.find_critical_path(plan)?.len();
        content.push_str(&format!("Critical Path Length: {} steps\n", critical_path_length));
        
        // Resource usage analysis
        let mut total_inputs = 0;
        let mut total_outputs = 0;
        for step in &plan.steps {
            total_inputs += step.inputs.len();
            total_outputs += step.outputs.len();
        }
        
        content.push_str(&format!("Data Movement: {} inputs, {} outputs\n", total_inputs, total_outputs));
        
        // Performance recommendations
        content.push_str("\nPerformance Recommendations:\n");
        if parallel_width > 1 {
            content.push_str(&format!("- Consider parallel execution with {} threads\n", parallel_width));
        }
        if critical_path_length > 10 {
            content.push_str("- Long critical path may benefit from optimization\n");
        }
        if total_inputs > total_outputs * 2 {
            content.push_str("- High input/output ratio suggests data aggregation workload\n");
        }
        
        Ok(ExplanationSection {
            section_id: "performance".to_string(),
            title: "Performance Characteristics".to_string(),
            content,
            section_type: ExplanationSectionType::PerformanceCharacteristics,
            related_steps: vec![],
            subsections: vec![],
        })
    }
    
    /// Generate security considerations section
    fn generate_security_section(&self, plan: &ExecutionPlan) -> GateCResult<ExplanationSection> {
        let mut content = String::new();
        
        content.push_str("Security Considerations:\n\n");
        
        // Analyze operation security implications
        let mut has_mutations = false;
        let mut has_queries = false;
        let mut has_compute = false;
        
        for step in &plan.steps {
            match &step.operation {
                Operation::Query { .. } => has_queries = true,
                Operation::Mutation { .. } => has_mutations = true,
                Operation::Compute { .. } => has_compute = true,
            }
        }
        
        content.push_str("Operation Security Profile:\n");
        if has_queries {
            content.push_str("- Contains data access operations (requires read permissions)\n");
        }
        if has_mutations {
            content.push_str("- Contains data modification operations (requires write permissions)\n");
        }
        if has_compute {
            content.push_str("- Contains computation operations (requires execute permissions)\n");
        }
        
        // Data flow security analysis
        content.push_str("\nData Flow Security:\n");
        let external_inputs = plan.steps.iter()
            .flat_map(|s| &s.inputs)
            .filter(|input| input.source_step.is_none())
            .count();
        
        if external_inputs > 0 {
            content.push_str(&format!("- {} external data inputs (validate input sources)\n", external_inputs));
        }
        
        let external_outputs = plan.steps.iter()
            .flat_map(|s| &s.outputs)
            .count();
        
        if external_outputs > 0 {
            content.push_str(&format!("- {} data outputs (consider output sanitization)\n", external_outputs));
        }
        
        // Capability requirements
        content.push_str("\nRequired Capabilities:\n");
        content.push_str("- PLAN_EXECUTION: Basic plan execution capability\n");
        if has_queries {
            content.push_str("- DATA_READ: Data access capability\n");
        }
        if has_mutations {
            content.push_str("- DATA_WRITE: Data modification capability\n");
        }
        if has_compute {
            content.push_str("- COMPUTE_EXECUTE: Computation capability\n");
        }
        
        Ok(ExplanationSection {
            section_id: "security".to_string(),
            title: "Security Considerations".to_string(),
            content,
            section_type: ExplanationSectionType::SecurityConsiderations,
            related_steps: vec![],
            subsections: vec![],
        })
    }
}

impl Default for REPLVisualizer {
    fn default() -> Self {
        Self::new()
    }
}

impl BoundedRenderer {
    /// Create new bounded renderer
    pub fn new(config: RenderingConfig) -> Self {
        Self {
            config,
            node_count: 0,
            output_size: 0,
        }
    }
    
    /// Get current node count
    pub fn node_count(&self) -> usize {
        self.node_count
    }
    
    /// Check if output was truncated
    pub fn was_truncated(&self) -> bool {
        self.node_count >= self.config.max_nodes || 
        self.output_size >= self.config.max_output_size
    }
    
    /// Render execution plan
    pub fn render_plan(&mut self, plan: &ExecutionPlan) -> GateCResult<String> {
        match self.config.style {
            RenderingStyle::Compact => self.render_compact(plan),
            RenderingStyle::Detailed => self.render_detailed(plan),
            RenderingStyle::Tree => self.render_tree(plan),
            RenderingStyle::Summary => self.render_summary(plan),
        }
    }
    
    /// Render plan in compact format
    fn render_compact(&mut self, plan: &ExecutionPlan) -> GateCResult<String> {
        let mut content = String::new();
        
        content.push_str(&format!("Plan: {} ({} steps)\n", plan.id, plan.steps.len()));
        
        for step in &plan.steps {
            if self.should_truncate() {
                content.push_str("... (truncated)\n");
                break;
            }
            
            let op_type = match &step.operation {
                Operation::Query { .. } => "Q",
                Operation::Mutation { .. } => "M", 
                Operation::Compute { .. } => "C",
            };
            
            content.push_str(&format!("- {} [{}]\n", step.id, op_type));
            self.node_count += 1;
            self.output_size += content.len();
        }
        
        Ok(content)
    }
    
    /// Render plan in detailed format
    fn render_detailed(&mut self, plan: &ExecutionPlan) -> GateCResult<String> {
        let mut content = String::new();
        
        content.push_str(&format!("=== Execution Plan: {} ===\n\n", plan.id));
        
        if self.config.include_metadata {
            content.push_str("## Metadata\n");
            content.push_str(&format!("- Name: {}\n", plan.metadata.name));
            if let Some(desc) = &plan.metadata.description {
                content.push_str(&format!("- Description: {}\n", desc));
            }
            content.push_str(&format!("- Version: {}\n", plan.metadata.version));
            content.push_str(&format!("- Created: {}\n\n", plan.metadata.created_at));
        }
        
        content.push_str("## Steps\n");
        for (i, step) in plan.steps.iter().enumerate() {
            if self.should_truncate() {
                content.push_str("... (remaining steps truncated)\n");
                break;
            }
            
            content.push_str(&format!("### {}. {}\n", i + 1, step.id));
            
            match &step.operation {
                Operation::Query { target, parameters } => {
                    content.push_str(&format!("**Type:** Query\n"));
                    content.push_str(&format!("**Target:** {}\n", target));
                    if !parameters.is_empty() {
                        content.push_str(&format!("**Parameters:** {} items\n", parameters.len()));
                    }
                }
                Operation::Mutation { intent } => {
                    content.push_str(&format!("**Type:** Mutation\n"));
                    content.push_str(&format!("**Intent:** {:?}\n", intent));
                }
                Operation::Compute { function, arguments } => {
                    content.push_str(&format!("**Type:** Compute\n"));
                    content.push_str(&format!("**Function:** {}\n", function));
                    content.push_str(&format!("**Arguments:** {} items\n", arguments.len()));
                }
            }
            
            if !step.inputs.is_empty() {
                content.push_str(&format!("**Inputs:** {}\n", 
                    step.inputs.iter().map(|r| r.id.clone()).collect::<Vec<_>>().join(", ")));
            }
            
            if !step.outputs.is_empty() {
                content.push_str(&format!("**Outputs:** {}\n", 
                    step.outputs.iter().map(|r| r.id.clone()).collect::<Vec<_>>().join(", ")));
            }
            
            content.push('\n');
            self.node_count += 1;
            self.output_size = content.len();
        }
        
        Ok(content)
    }
    
    /// Render plan in tree format
    fn render_tree(&mut self, plan: &ExecutionPlan) -> GateCResult<String> {
        let mut content = String::new();
        
        content.push_str(&format!("Plan: {}\n", plan.id));
        
        // Build dependency tree
        let mut dependency_map: HashMap<String, Vec<String>> = HashMap::new();
        let mut all_steps: HashSet<String> = HashSet::new();
        
        for step in &plan.steps {
            all_steps.insert(step.id.clone());
            for input in &step.inputs {
                if let Some(source_step) = &input.source_step {
                    dependency_map.entry(source_step.clone())
                        .or_insert_with(Vec::new)
                        .push(step.id.clone());
                }
            }
        }
        
        // Find root steps (no dependencies)
        let mut root_steps = Vec::new();
        for step in &plan.steps {
            let has_dependencies = step.inputs.iter().any(|input| input.source_step.is_some());
            if !has_dependencies {
                root_steps.push(step.id.clone());
            }
        }
        
        // Render tree
        for root in &root_steps {
            if self.should_truncate() {
                content.push_str("... (truncated)\n");
                break;
            }
            self.render_tree_node(&mut content, root, &dependency_map, 0)?;
        }
        
        Ok(content)
    }
    
    /// Render tree node recursively
    fn render_tree_node(&mut self, content: &mut String, step_id: &str, 
                       dependency_map: &HashMap<String, Vec<String>>, 
                       depth: usize) -> GateCResult<()> {
        if self.should_truncate() {
            return Ok(());
        }
        
        let indent = "  ".repeat(depth);
        content.push_str(&format!("{}├─ {}\n", indent, step_id));
        self.node_count += 1;
        self.output_size = content.len();
        
        if let Some(children) = dependency_map.get(step_id) {
            for child in children {
                self.render_tree_node(content, child, dependency_map, depth + 1)?;
            }
        }
        
        Ok(())
    }
    
    /// Render plan summary
    fn render_summary(&mut self, plan: &ExecutionPlan) -> GateCResult<String> {
        let mut content = String::new();
        
        content.push_str(&format!("=== Plan Summary: {} ===\n\n", plan.id));
        
        // Count operations by type
        let mut query_count = 0;
        let mut mutation_count = 0;
        let mut compute_count = 0;
        
        for step in &plan.steps {
            match &step.operation {
                Operation::Query { .. } => query_count += 1,
                Operation::Mutation { .. } => mutation_count += 1,
                Operation::Compute { .. } => compute_count += 1,
            }
        }
        
        content.push_str(&format!("**Total Steps:** {}\n", plan.steps.len()));
        content.push_str(&format!("**Query Operations:** {}\n", query_count));
        content.push_str(&format!("**Mutation Operations:** {}\n", mutation_count));
        content.push_str(&format!("**Compute Operations:** {}\n", compute_count));
        content.push_str(&format!("**Dependencies:** {}\n", plan.dependencies.len()));
        
        // Calculate total data references
        let mut total_inputs = 0;
        let mut total_outputs = 0;
        for step in &plan.steps {
            total_inputs += step.inputs.len();
            total_outputs += step.outputs.len();
        }
        
        content.push_str(&format!("**Total Inputs:** {}\n", total_inputs));
        content.push_str(&format!("**Total Outputs:** {}\n", total_outputs));
        
        self.node_count = 1; // Summary counts as one node
        self.output_size = content.len();
        
        Ok(content)
    }
    
    /// Check if rendering should be truncated
    fn should_truncate(&self) -> bool {
        self.node_count >= self.config.max_nodes || 
        self.output_size >= self.config.max_output_size
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gate_c::types::{PlanMetadata, DependencyType};
    use std::collections::HashMap;

    fn create_test_plan() -> ExecutionPlan {
        ExecutionPlan {
            id: "test-visualization-plan".to_string(),
            steps: vec![
                PlanStep {
                    id: "step-1".to_string(),
                    operation: Operation::Query {
                        target: "database".to_string(),
                        parameters: {
                            let mut params = HashMap::new();
                            params.insert("table".to_string(), "users".to_string());
                            params
                        },
                    },
                    inputs: vec![],
                    outputs: vec![DataRef {
                        id: "user-data".to_string(),
                        data_type: "json".to_string(),
                        source_step: Some("step-1".to_string()),
                    }],
                },
                PlanStep {
                    id: "step-2".to_string(),
                    operation: Operation::Compute {
                        function: "process_users".to_string(),
                        arguments: vec!["filter".to_string()],
                    },
                    inputs: vec![DataRef {
                        id: "user-data".to_string(),
                        data_type: "json".to_string(),
                        source_step: Some("step-1".to_string()),
                    }],
                    outputs: vec![DataRef {
                        id: "processed-data".to_string(),
                        data_type: "json".to_string(),
                        source_step: Some("step-2".to_string()),
                    }],
                },
            ],
            metadata: PlanMetadata {
                name: "Visualization Test Plan".to_string(),
                description: Some("Plan for testing visualization".to_string()),
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
    fn test_repl_visualizer_creation() {
        let visualizer = REPLVisualizer::new();
        assert_eq!(visualizer.config.max_render_nodes, MAX_RENDER_NODES);
        assert_eq!(visualizer.config.max_output_size, MAX_INSPECT_OUTPUT_BYTES);
        assert!(visualizer.config.detailed_view);
        assert!(visualizer.config.dry_run_mode);
    }

    #[test]
    fn test_repl_visualizer_with_config() {
        let config = VisualizationConfig {
            max_render_nodes: 100,
            max_output_size: 1000,
            detailed_view: false,
            dry_run_mode: false,
            show_dependencies: false,
        };
        
        let visualizer = REPLVisualizer::with_config(config);
        assert_eq!(visualizer.config.max_render_nodes, 100);
        assert_eq!(visualizer.config.max_output_size, 1000);
        assert!(!visualizer.config.detailed_view);
        assert!(!visualizer.config.dry_run_mode);
    }

    #[test]
    fn test_plan_visualization() {
        let visualizer = REPLVisualizer::new();
        let plan = create_test_plan();
        
        let result = visualizer.visualize_plan(&plan);
        assert!(result.is_ok());
        
        let visualization = result.unwrap();
        assert_eq!(visualization.plan_id, "test-visualization-plan");
        assert!(!visualization.content.is_empty());
        assert_eq!(visualization.metadata.total_nodes, 2);
        // Note: render_duration_ms is u64, always >= 0
        
        // Check summary
        assert_eq!(visualization.summary.total_steps, 2);
        assert_eq!(visualization.summary.query_operations, 1);
        assert_eq!(visualization.summary.compute_operations, 1);
        assert_eq!(visualization.summary.mutation_operations, 0);
    }

    #[test]
    fn test_dry_run_preview() {
        let visualizer = REPLVisualizer::new();
        let plan = create_test_plan();
        
        let result = visualizer.dry_run_preview(&plan);
        assert!(result.is_ok());
        
        let preview = result.unwrap();
        assert_eq!(preview.plan_id, "test-visualization-plan");
        assert!(!preview.preview.is_empty());
        assert_eq!(preview.execution_flow.len(), 2);
        assert_eq!(preview.data_flow.len(), 1);
        
        // Check execution flow
        assert_eq!(preview.execution_flow[0].step_id, "step-1");
        assert_eq!(preview.execution_flow[0].operation_type, "Query");
        assert_eq!(preview.execution_flow[1].step_id, "step-2");
        assert_eq!(preview.execution_flow[1].operation_type, "Compute");
        
        // Check data flow
        assert_eq!(preview.data_flow[0].from_step, "step-1");
        assert_eq!(preview.data_flow[0].to_step, "step-2");
        assert_eq!(preview.data_flow[0].data_ref, "user-data");
    }

    #[test]
    fn test_bounded_renderer_compact() {
        let config = RenderingConfig {
            max_nodes: 10,
            max_output_size: 1000,
            style: RenderingStyle::Compact,
            include_metadata: false,
        };
        
        let mut renderer = BoundedRenderer::new(config);
        let plan = create_test_plan();
        
        let result = renderer.render_plan(&plan);
        assert!(result.is_ok());
        
        let content = result.unwrap();
        assert!(content.contains("test-visualization-plan"));
        assert!(content.contains("step-1"));
        assert!(content.contains("step-2"));
        assert_eq!(renderer.node_count(), 2);
    }

    #[test]
    fn test_bounded_renderer_detailed() {
        let config = RenderingConfig {
            max_nodes: 10,
            max_output_size: 5000,
            style: RenderingStyle::Detailed,
            include_metadata: true,
        };
        
        let mut renderer = BoundedRenderer::new(config);
        let plan = create_test_plan();
        
        let result = renderer.render_plan(&plan);
        assert!(result.is_ok());
        
        let content = result.unwrap();
        assert!(content.contains("Execution Plan"));
        assert!(content.contains("Metadata"));
        assert!(content.contains("Visualization Test Plan"));
        assert!(content.contains("Query"));
        assert!(content.contains("Compute"));
    }

    #[test]
    fn test_bounded_renderer_tree() {
        let config = RenderingConfig {
            max_nodes: 10,
            max_output_size: 1000,
            style: RenderingStyle::Tree,
            include_metadata: false,
        };
        
        let mut renderer = BoundedRenderer::new(config);
        let plan = create_test_plan();
        
        let result = renderer.render_plan(&plan);
        assert!(result.is_ok());
        
        let content = result.unwrap();
        assert!(content.contains("├─"));
        assert!(content.contains("step-1"));
    }

    #[test]
    fn test_bounded_renderer_summary() {
        let config = RenderingConfig {
            max_nodes: 10,
            max_output_size: 1000,
            style: RenderingStyle::Summary,
            include_metadata: false,
        };
        
        let mut renderer = BoundedRenderer::new(config);
        let plan = create_test_plan();
        
        let result = renderer.render_plan(&plan);
        assert!(result.is_ok());
        
        let content = result.unwrap();
        assert!(content.contains("Plan Summary"));
        assert!(content.contains("**Total Steps:** 2"));
        assert!(content.contains("**Query Operations:** 1"));
        assert!(content.contains("**Compute Operations:** 1"));
        assert!(content.contains("**Mutation Operations:** 0"));
    }

    #[test]
    fn test_rendering_limits() {
        let config = RenderingConfig {
            max_nodes: 1, // Very small limit
            max_output_size: 1000,
            style: RenderingStyle::Compact,
            include_metadata: false,
        };
        
        let mut renderer = BoundedRenderer::new(config);
        let plan = create_test_plan();
        
        let result = renderer.render_plan(&plan);
        assert!(result.is_ok());
        
        let content = result.unwrap();
        assert!(renderer.was_truncated());
        assert!(content.contains("truncated") || renderer.node_count() <= 1);
    }

    #[test]
    fn test_plan_summary_generation() {
        let visualizer = REPLVisualizer::new();
        let plan = create_test_plan();
        
        let result = visualizer.generate_plan_summary(&plan);
        assert!(result.is_ok());
        
        let summary = result.unwrap();
        assert_eq!(summary.total_steps, 2);
        assert_eq!(summary.query_operations, 1);
        assert_eq!(summary.compute_operations, 1);
        assert_eq!(summary.mutation_operations, 0);
        assert_eq!(summary.total_dependencies, 1);
        assert!(summary.complexity_score > 0.0);
    }

    #[test]
    fn test_complexity_score_calculation() {
        let visualizer = REPLVisualizer::new();
        let plan = create_test_plan();
        
        let complexity = visualizer.calculate_complexity_score(&plan);
        
        // Should be > 0 and reasonable
        assert!(complexity > 0.0);
        assert!(complexity < 100.0); // Reasonable upper bound for test plan
    }

    #[test]
    fn test_execution_flow_preview_generation() {
        let visualizer = REPLVisualizer::new();
        let plan = create_test_plan();
        
        let result = visualizer.generate_execution_flow_preview(&plan);
        assert!(result.is_ok());
        
        let flow = result.unwrap();
        assert_eq!(flow.len(), 2);
        
        // Check first step
        assert_eq!(flow[0].step_id, "step-1");
        assert_eq!(flow[0].operation_type, "Query");
        assert!(flow[0].inputs.is_empty());
        assert_eq!(flow[0].outputs, vec!["user-data"]);
        
        // Check second step
        assert_eq!(flow[1].step_id, "step-2");
        assert_eq!(flow[1].operation_type, "Compute");
        assert_eq!(flow[1].inputs, vec!["user-data"]);
        assert_eq!(flow[1].outputs, vec!["processed-data"]);
        assert_eq!(flow[1].dependencies, vec!["step-1"]);
    }

    #[test]
    fn test_data_flow_preview_generation() {
        let visualizer = REPLVisualizer::new();
        let plan = create_test_plan();
        
        let result = visualizer.generate_data_flow_preview(&plan);
        assert!(result.is_ok());
        
        let flow = result.unwrap();
        assert_eq!(flow.len(), 1);
        
        assert_eq!(flow[0].from_step, "step-1");
        assert_eq!(flow[0].to_step, "step-2");
        assert_eq!(flow[0].data_ref, "user-data");
        assert_eq!(flow[0].data_type, "json");
    }

    #[test]
    fn test_step_description_generation() {
        let visualizer = REPLVisualizer::new();
        
        // Test query description
        let query_step = PlanStep {
            id: "test".to_string(),
            operation: Operation::Query {
                target: "database".to_string(),
                parameters: HashMap::new(),
            },
            inputs: vec![],
            outputs: vec![],
        };
        
        let desc = visualizer.generate_step_description(&query_step);
        assert_eq!(desc, "Query data from database");
        
        // Test compute description
        let compute_step = PlanStep {
            id: "test".to_string(),
            operation: Operation::Compute {
                function: "process".to_string(),
                arguments: vec!["arg1".to_string(), "arg2".to_string()],
            },
            inputs: vec![],
            outputs: vec![],
        };
        
        let desc = visualizer.generate_step_description(&compute_step);
        assert_eq!(desc, "Execute function 'process' with 2 arguments");
    }

    #[test]
    fn test_preview_content_generation() {
        let visualizer = REPLVisualizer::new();
        let plan = create_test_plan();
        
        let execution_flow = visualizer.generate_execution_flow_preview(&plan).unwrap();
        let data_flow = visualizer.generate_data_flow_preview(&plan).unwrap();
        
        let result = visualizer.generate_preview_content(&plan, &execution_flow, &data_flow);
        assert!(result.is_ok());
        
        let content = result.unwrap();
        assert!(content.contains("Dry-Run Preview"));
        assert!(content.contains("Plan Overview"));
        assert!(content.contains("Execution Flow"));
        assert!(content.contains("Data Flow"));
        assert!(content.contains("step-1"));
        assert!(content.contains("step-2"));
    }

    #[test]
    fn test_semantic_explanation_generation() {
        let visualizer = REPLVisualizer::new();
        let plan = create_test_plan();
        
        let result = visualizer.generate_semantic_explanation(&plan);
        assert!(result.is_ok());
        
        let explanation = result.unwrap();
        assert_eq!(explanation.plan_id, "test-visualization-plan");
        assert!(!explanation.explanation.is_empty());
        assert_eq!(explanation.sections.len(), 6); // Overview, DataFlow, OperationSequence, DependencyAnalysis, Performance, Security
        // Note: generation_duration_ms is u64, always >= 0
        assert!(explanation.metadata.sections_generated > 0);
    }

    #[test]
    fn test_explanation_sections() {
        let visualizer = REPLVisualizer::new();
        let plan = create_test_plan();
        
        // Test overview section
        let overview = visualizer.generate_overview_section(&plan).unwrap();
        assert_eq!(overview.section_type, ExplanationSectionType::Overview);
        assert!(overview.content.contains("execution plan"));
        assert!(overview.content.contains("2 steps"));
        
        // Test data flow section
        let data_flow = visualizer.generate_data_flow_section(&plan).unwrap();
        assert_eq!(data_flow.section_type, ExplanationSectionType::DataFlow);
        assert!(data_flow.content.contains("Data flows"));
        
        // Test operation sequence section
        let op_sequence = visualizer.generate_operation_sequence_section(&plan).unwrap();
        assert_eq!(op_sequence.section_type, ExplanationSectionType::OperationSequence);
        assert!(op_sequence.content.contains("semantic order"));
        
        // Test dependency analysis section
        let dep_analysis = visualizer.generate_dependency_analysis_section(&plan).unwrap();
        assert_eq!(dep_analysis.section_type, ExplanationSectionType::DependencyAnalysis);
        assert!(dep_analysis.content.contains("Dependency Analysis"));
        
        // Test performance section
        let performance = visualizer.generate_performance_section(&plan).unwrap();
        assert_eq!(performance.section_type, ExplanationSectionType::PerformanceCharacteristics);
        assert!(performance.content.contains("Performance Characteristics"));
        
        // Test security section
        let security = visualizer.generate_security_section(&plan).unwrap();
        assert_eq!(security.section_type, ExplanationSectionType::SecurityConsiderations);
        assert!(security.content.contains("Security Considerations"));
    }

    #[test]
    fn test_interactive_features_generation() {
        let visualizer = REPLVisualizer::new();
        let plan = create_test_plan();
        let explanation = visualizer.generate_semantic_explanation(&plan).unwrap();
        
        let result = visualizer.generate_interactive_features(&plan, &explanation);
        assert!(result.is_ok());
        
        let features = result.unwrap();
        assert_eq!(features.drill_down_sections.len(), 6);
        assert_eq!(features.step_navigation.len(), 2);
        assert!(!features.cross_references.is_empty());
        
        // Check step navigation
        assert_eq!(features.step_navigation[0].step_id, "step-1");
        assert_eq!(features.step_navigation[1].step_id, "step-2");
    }

    #[test]
    fn test_execution_order_determination() {
        let visualizer = REPLVisualizer::new();
        let plan = create_test_plan();
        
        let result = visualizer.determine_execution_order(&plan);
        assert!(result.is_ok());
        
        let order = result.unwrap();
        assert_eq!(order.len(), 2);
        assert_eq!(order[0], "step-1"); // step-1 has no dependencies, should come first
        assert_eq!(order[1], "step-2"); // step-2 depends on step-1, should come second
    }

    #[test]
    fn test_parallel_groups_identification() {
        let visualizer = REPLVisualizer::new();
        let plan = create_test_plan();
        
        let result = visualizer.identify_parallel_groups(&plan);
        assert!(result.is_ok());
        
        let groups = result.unwrap();
        assert!(!groups.is_empty());
        // In our test plan, step-2 depends on step-1, so they can't be parallel
        assert!(groups.iter().all(|group| group.len() == 1));
    }

    #[test]
    fn test_critical_path_finding() {
        let visualizer = REPLVisualizer::new();
        let plan = create_test_plan();
        
        let result = visualizer.find_critical_path(&plan);
        assert!(result.is_ok());
        
        let path = result.unwrap();
        assert!(!path.is_empty());
        assert!(path.contains(&"step-1".to_string()));
        assert!(path.contains(&"step-2".to_string()));
    }

    #[test]
    fn test_dependency_bottlenecks() {
        let visualizer = REPLVisualizer::new();
        let plan = create_test_plan();
        
        let result = visualizer.find_dependency_bottlenecks(&plan);
        assert!(result.is_ok());
        
        let bottlenecks = result.unwrap();
        // Our simple test plan shouldn't have bottlenecks
        assert!(bottlenecks.is_empty());
    }

    #[test]
    fn test_parallel_width_estimation() {
        let visualizer = REPLVisualizer::new();
        let plan = create_test_plan();
        
        let result = visualizer.estimate_parallel_width(&plan);
        assert!(result.is_ok());
        
        let width = result.unwrap();
        assert!(width >= 1);
    }

    #[test]
    fn test_explanation_size_limits() {
        let mut config = VisualizationConfig::default();
        config.max_output_size = 100; // Very small limit
        
        let visualizer = REPLVisualizer::with_config(config);
        let plan = create_test_plan();
        
        let result = visualizer.generate_semantic_explanation(&plan);
        assert!(result.is_ok());
        
        let explanation = result.unwrap();
        assert!(explanation.explanation.len() <= 150); // Should be truncated with message
        assert!(explanation.metadata.truncated);
    }

    #[test]
    fn test_capability_based_redaction() {
        let mut visualizer = REPLVisualizer::new();
        visualizer.add_capability_filter("test_capability".to_string(), CapabilityScope::Read);
        
        let plan = create_test_plan();
        
        let result = visualizer.generate_semantic_explanation(&plan);
        assert!(result.is_ok());
        
        let explanation = result.unwrap();
        // Should still generate explanation but with redaction applied
        assert!(!explanation.explanation.is_empty());
        assert!(!explanation.sections.is_empty());
    }

    #[test]
    fn test_cross_references_generation() {
        let visualizer = REPLVisualizer::new();
        
        // Create sections with overlapping related steps
        let sections = vec![
            ExplanationSection {
                section_id: "section1".to_string(),
                title: "Section 1".to_string(),
                content: "Content 1".to_string(),
                section_type: ExplanationSectionType::Overview,
                related_steps: vec!["step-1".to_string(), "step-2".to_string()],
                subsections: vec![],
            },
            ExplanationSection {
                section_id: "section2".to_string(),
                title: "Section 2".to_string(),
                content: "Content 2".to_string(),
                section_type: ExplanationSectionType::DataFlow,
                related_steps: vec!["step-2".to_string()],
                subsections: vec![],
            },
        ];
        
        let cross_refs = visualizer.generate_cross_references(&sections);
        
        // section1 and section2 should cross-reference each other (both have step-2)
        assert!(cross_refs.get("section1").unwrap().contains(&"section2".to_string()));
        assert!(cross_refs.get("section2").unwrap().contains(&"section1".to_string()));
    }

    #[test]
    fn test_explanation_content_generation() {
        let visualizer = REPLVisualizer::new();
        let plan = create_test_plan();
        
        let sections = vec![
            ExplanationSection {
                section_id: "test_section".to_string(),
                title: "Test Section".to_string(),
                content: "Test content".to_string(),
                section_type: ExplanationSectionType::Overview,
                related_steps: vec![],
                subsections: vec![],
            },
        ];
        
        let result = visualizer.generate_explanation_content(&plan, &sections);
        assert!(result.is_ok());
        
        let (content, _) = result.unwrap();
        assert!(content.contains("Semantic Explanation"));
        assert!(content.contains("Test Section"));
        assert!(content.contains("Test content"));
    }
}