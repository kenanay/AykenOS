//! # Pipeline Planning (Stateless)
//!
//! Analyze pipeline dependencies without state storage.
//!
//! **ARCHITECTURAL RULE:**
//! This module MUST NOT depend on higher-level Gate C components.
//! Violations are considered architecture breaks.
//!
//! **Author:** Kenan AY  
//! **Phase:** 3.5 Gate C

use crate::gate_c::{
    error::{GateCResult, PipelineError},
    limits::{MAX_DEPENDENCY_DEPTH, MAX_DEP_GRAPH_EDGES, MAX_PIPELINE_STEPS},
    types::{DependencyGraph, PipelinePlan, PipelineStep, StepId},
};
use std::collections::{HashMap, HashSet, VecDeque};

/// Dependency analyzer for pipeline planning
pub struct DependencyAnalyzer {
    max_steps: usize,
    max_edges: usize,
    max_depth: usize,
}

impl DependencyAnalyzer {
    /// Create new dependency analyzer with default limits
    pub fn new() -> Self {
        Self {
            max_steps: MAX_PIPELINE_STEPS,
            max_edges: MAX_DEP_GRAPH_EDGES,
            max_depth: MAX_DEPENDENCY_DEPTH,
        }
    }

    /// Create dependency analyzer with custom limits
    pub fn with_limits(max_steps: usize, max_edges: usize, max_depth: usize) -> Self {
        Self {
            max_steps,
            max_edges,
            max_depth,
        }
    }

    /// Analyze pipeline steps and build dependency graph
    pub fn analyze(&self, steps: &[PipelineStep]) -> GateCResult<DependencyGraph> {
        // Check step count limit
        if steps.len() > self.max_steps {
            return Err(PipelineError::PipelineTooLarge {
                steps: steps.len(),
                limit: self.max_steps,
            }
            .into());
        }

        // Build dependency graph from step inputs/outputs
        let mut graph = DependencyGraph {
            nodes: Vec::new(),
            edges: Vec::new(),
        };

        // Add all step IDs as nodes
        for step in steps {
            graph.nodes.push(step.id.clone());
        }

        // Build edges based on data dependencies
        let mut output_producers: HashMap<String, StepId> = HashMap::new();

        // First pass: record which steps produce which outputs
        for step in steps {
            for output in &step.outputs {
                output_producers.insert(output.id.clone(), step.id.clone());
            }
        }

        // Second pass: create edges for input dependencies
        for step in steps {
            for input in &step.inputs {
                if let Some(producer_step) = output_producers.get(&input.id) {
                    if producer_step != &step.id {
                        graph.edges.push((producer_step.clone(), step.id.clone()));
                    }
                }
            }
        }

        // Check edge count limit
        if graph.edges.len() > self.max_edges {
            return Err(PipelineError::PipelineTooLarge {
                steps: graph.edges.len(),
                limit: self.max_edges,
            }
            .into());
        }

        // Validate the graph
        self.validate_graph(&graph)?;

        Ok(graph)
    }

    /// Detect cycles in dependency graph
    pub fn detect_cycles(&self, graph: &DependencyGraph) -> GateCResult<()> {
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();

        for node in &graph.nodes {
            if !visited.contains(node) {
                if self.has_cycle_util(graph, node, &mut visited, &mut rec_stack)? {
                    return Err(PipelineError::CycleDetected(format!(
                        "Cycle detected involving node: {}",
                        node
                    ))
                    .into());
                }
            }
        }

        Ok(())
    }

    /// Canonicalize dependency graph ordering
    pub fn canonicalize_order(&self, graph: &DependencyGraph) -> GateCResult<Vec<StepId>> {
        // Perform topological sort
        let mut in_degree: HashMap<StepId, usize> = HashMap::new();
        let mut adj_list: HashMap<StepId, Vec<StepId>> = HashMap::new();

        // Initialize in-degree and adjacency list
        for node in &graph.nodes {
            in_degree.insert(node.clone(), 0);
            adj_list.insert(node.clone(), Vec::new());
        }

        // Build adjacency list and calculate in-degrees
        for (from, to) in &graph.edges {
            adj_list.get_mut(from).unwrap().push(to.clone());
            *in_degree.get_mut(to).unwrap() += 1;
        }

        // Kahn's algorithm for topological sorting
        let mut queue = VecDeque::new();
        let mut result = Vec::new();

        // Add all nodes with in-degree 0 to queue
        for (node, &degree) in &in_degree {
            if degree == 0 {
                queue.push_back(node.clone());
            }
        }

        // Sort nodes with same in-degree for deterministic ordering
        let mut queue_vec: Vec<_> = queue.into_iter().collect();
        queue_vec.sort();
        queue = queue_vec.into();

        while let Some(node) = queue.pop_front() {
            result.push(node.clone());

            // Reduce in-degree of adjacent nodes
            let mut next_nodes = Vec::new();
            if let Some(neighbors) = adj_list.get(&node) {
                for neighbor in neighbors {
                    let degree = in_degree.get_mut(neighbor).unwrap();
                    *degree -= 1;
                    if *degree == 0 {
                        next_nodes.push(neighbor.clone());
                    }
                }
            }

            // Sort next nodes for deterministic ordering
            next_nodes.sort();
            for next_node in next_nodes {
                queue.push_back(next_node);
            }
        }

        // Check if all nodes were processed (no cycles)
        if result.len() != graph.nodes.len() {
            return Err(PipelineError::CycleDetected(
                "Topological sort failed - cycle detected".to_string(),
            )
            .into());
        }

        Ok(result)
    }

    /// Validate dependency graph structure
    fn validate_graph(&self, graph: &DependencyGraph) -> GateCResult<()> {
        // Check for cycles
        self.detect_cycles(graph)?;

        // Check dependency depth
        let depth = self.calculate_max_depth(graph)?;
        if depth > self.max_depth {
            return Err(PipelineError::AmbiguousOrdering(format!(
                "Dependency depth {} exceeds maximum {}",
                depth, self.max_depth
            ))
            .into());
        }

        // Validate all edge references exist as nodes
        let node_set: HashSet<_> = graph.nodes.iter().collect();
        for (from, to) in &graph.edges {
            if !node_set.contains(from) {
                return Err(PipelineError::InvalidStepReference(format!(
                    "Edge references non-existent node: {}",
                    from
                ))
                .into());
            }
            if !node_set.contains(to) {
                return Err(PipelineError::InvalidStepReference(format!(
                    "Edge references non-existent node: {}",
                    to
                ))
                .into());
            }
        }

        Ok(())
    }

    /// Utility function for cycle detection using DFS
    fn has_cycle_util(
        &self,
        graph: &DependencyGraph,
        node: &StepId,
        visited: &mut HashSet<StepId>,
        rec_stack: &mut HashSet<StepId>,
    ) -> GateCResult<bool> {
        visited.insert(node.clone());
        rec_stack.insert(node.clone());

        // Find all neighbors of current node
        for (from, to) in &graph.edges {
            if from == node {
                if !visited.contains(to) {
                    if self.has_cycle_util(graph, to, visited, rec_stack)? {
                        return Ok(true);
                    }
                } else if rec_stack.contains(to) {
                    return Ok(true);
                }
            }
        }

        rec_stack.remove(node);
        Ok(false)
    }

    /// Calculate maximum dependency depth
    fn calculate_max_depth(&self, graph: &DependencyGraph) -> GateCResult<usize> {
        let mut max_depth = 0;
        let mut visited = HashSet::new();

        for node in &graph.nodes {
            if !visited.contains(node) {
                let depth = self.calculate_depth_from_node(graph, node, &mut visited, 0)?;
                max_depth = max_depth.max(depth);
            }
        }

        Ok(max_depth)
    }

    /// Calculate depth from a specific node
    fn calculate_depth_from_node(
        &self,
        graph: &DependencyGraph,
        node: &StepId,
        visited: &mut HashSet<StepId>,
        current_depth: usize,
    ) -> GateCResult<usize> {
        visited.insert(node.clone());
        let mut max_depth = current_depth;

        // Find all outgoing edges from this node
        for (from, to) in &graph.edges {
            if from == node && !visited.contains(to) {
                let depth =
                    self.calculate_depth_from_node(graph, to, visited, current_depth + 1)?;
                max_depth = max_depth.max(depth);
            }
        }

        Ok(max_depth)
    }
}

impl Default for DependencyAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

/// Stateless pipeline planner
pub struct PipelinePlanner {
    analyzer: DependencyAnalyzer,
    canonicalizer: OrderCanonicalizer,
}

impl PipelinePlanner {
    /// Create new pipeline planner
    pub fn new() -> Self {
        Self {
            analyzer: DependencyAnalyzer::new(),
            canonicalizer: OrderCanonicalizer::new(),
        }
    }

    /// Create pipeline planner with custom analyzer
    pub fn with_analyzer(analyzer: DependencyAnalyzer) -> Self {
        Self {
            analyzer,
            canonicalizer: OrderCanonicalizer::new(),
        }
    }

    /// Plan pipeline from steps (stateless operation)
    pub fn plan(&self, steps: Vec<PipelineStep>) -> GateCResult<PipelinePlan> {
        // DETERMINISM FIX: Calculate timestamp before moving steps
        let created_at =
            crate::gate_c::deterministic::deterministic_timestamp_from_plan_id("pipeline_plan");

        // Analyze dependencies
        let dependencies = self.analyzer.analyze(&steps)?;

        // Canonicalize ordering
        let canonical_order = self.analyzer.canonicalize_order(&dependencies)?;

        // Validate canonical ordering is unambiguous
        self.canonicalizer
            .validate_ordering(&canonical_order, &dependencies)?;

        // Create pipeline plan
        let pipeline_plan = PipelinePlan {
            steps,
            dependencies,
            metadata: crate::gate_c::types::PipelineMetadata {
                name: "Generated Pipeline".to_string(),
                description: Some("Auto-generated pipeline from dependency analysis".to_string()),
                created_at,
            },
        };

        Ok(pipeline_plan)
    }
}

impl Default for PipelinePlanner {
    fn default() -> Self {
        Self::new()
    }
}

/// Order canonicalizer for deterministic pipeline ordering
pub struct OrderCanonicalizer {
    // Configuration for canonicalization rules
}

impl OrderCanonicalizer {
    /// Create new order canonicalizer
    pub fn new() -> Self {
        Self {}
    }

    /// Validate that ordering is unambiguous
    pub fn validate_ordering(&self, order: &[StepId], graph: &DependencyGraph) -> GateCResult<()> {
        // Check that the ordering respects all dependencies
        let position_map: HashMap<_, _> = order.iter().enumerate().map(|(i, id)| (id, i)).collect();

        for (from, to) in &graph.edges {
            let from_pos = position_map.get(from).ok_or_else(|| {
                PipelineError::InvalidStepReference(format!("Step not found in order: {}", from))
            })?;
            let to_pos = position_map.get(to).ok_or_else(|| {
                PipelineError::InvalidStepReference(format!("Step not found in order: {}", to))
            })?;

            if from_pos >= to_pos {
                return Err(PipelineError::AmbiguousOrdering(format!(
                    "Dependency violation: {} (pos {}) should come before {} (pos {})",
                    from, from_pos, to, to_pos
                ))
                .into());
            }
        }

        Ok(())
    }
}

impl Default for OrderCanonicalizer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gate_c::types::{DataRef, Operation};
    use std::collections::HashMap;

    fn create_test_step(id: &str, inputs: Vec<&str>, outputs: Vec<&str>) -> PipelineStep {
        PipelineStep {
            id: id.to_string(),
            operation: Operation::Query {
                target: "test".to_string(),
                parameters: HashMap::new(),
            },
            inputs: inputs
                .into_iter()
                .map(|s| DataRef {
                    id: s.to_string(),
                    data_type: "test".to_string(),
                    source_step: None,
                })
                .collect(),
            outputs: outputs
                .into_iter()
                .map(|s| DataRef {
                    id: s.to_string(),
                    data_type: "test".to_string(),
                    source_step: Some(id.to_string()),
                })
                .collect(),
        }
    }

    #[test]
    fn test_dependency_analyzer_creation() {
        let analyzer = DependencyAnalyzer::new();
        assert_eq!(analyzer.max_steps, MAX_PIPELINE_STEPS);
        assert_eq!(analyzer.max_edges, MAX_DEP_GRAPH_EDGES);
    }

    #[test]
    fn test_simple_dependency_analysis() {
        let analyzer = DependencyAnalyzer::new();

        let steps = vec![
            create_test_step("step1", vec![], vec!["data1"]),
            create_test_step("step2", vec!["data1"], vec!["data2"]),
        ];

        let result = analyzer.analyze(&steps);
        assert!(result.is_ok());

        let graph = result.unwrap();
        assert_eq!(graph.nodes.len(), 2);
        assert_eq!(graph.edges.len(), 1);
        assert_eq!(graph.edges[0], ("step1".to_string(), "step2".to_string()));
    }

    #[test]
    fn test_cycle_detection() {
        let analyzer = DependencyAnalyzer::new();

        // Create a cycle: step1 -> step2 -> step1
        let steps = vec![
            create_test_step("step1", vec!["data2"], vec!["data1"]),
            create_test_step("step2", vec!["data1"], vec!["data2"]),
        ];

        let graph_result = analyzer.analyze(&steps);

        // The analyze method should detect the cycle and return an error
        assert!(graph_result.is_err());

        match graph_result.unwrap_err() {
            crate::gate_c::error::GateCError::Pipeline(PipelineError::CycleDetected(_)) => {
                // Expected - cycle detected during analysis
            }
            _ => panic!("Expected CycleDetected error during analysis"),
        }
    }

    #[test]
    fn test_topological_sort() {
        let analyzer = DependencyAnalyzer::new();

        let steps = vec![
            create_test_step("step1", vec![], vec!["data1"]),
            create_test_step("step2", vec!["data1"], vec!["data2"]),
            create_test_step("step3", vec!["data2"], vec!["data3"]),
        ];

        let graph = analyzer.analyze(&steps).unwrap();
        let order = analyzer.canonicalize_order(&graph).unwrap();

        assert_eq!(order, vec!["step1", "step2", "step3"]);
    }

    #[test]
    fn test_pipeline_planner() {
        let planner = PipelinePlanner::new();

        let steps = vec![
            create_test_step("step1", vec![], vec!["data1"]),
            create_test_step("step2", vec!["data1"], vec!["data2"]),
        ];

        let result = planner.plan(steps);
        assert!(result.is_ok());

        let pipeline = result.unwrap();
        assert_eq!(pipeline.steps.len(), 2);
        assert_eq!(pipeline.dependencies.nodes.len(), 2);
        assert_eq!(pipeline.dependencies.edges.len(), 1);
    }

    #[test]
    fn test_pipeline_too_large() {
        let analyzer = DependencyAnalyzer::with_limits(2, 10, 5);

        let steps = vec![
            create_test_step("step1", vec![], vec!["data1"]),
            create_test_step("step2", vec![], vec!["data2"]),
            create_test_step("step3", vec![], vec!["data3"]), // Exceeds limit of 2
        ];

        let result = analyzer.analyze(&steps);
        assert!(result.is_err());

        match result.unwrap_err() {
            crate::gate_c::error::GateCError::Pipeline(PipelineError::PipelineTooLarge {
                steps,
                limit,
            }) => {
                assert_eq!(steps, 3);
                assert_eq!(limit, 2);
            }
            _ => panic!("Expected PipelineTooLarge error"),
        }
    }

    #[test]
    fn test_deterministic_ordering() {
        let analyzer = DependencyAnalyzer::new();

        // Create steps with no dependencies (should be sorted by name)
        let steps = vec![
            create_test_step("step_c", vec![], vec!["data_c"]),
            create_test_step("step_a", vec![], vec!["data_a"]),
            create_test_step("step_b", vec![], vec!["data_b"]),
        ];

        let graph = analyzer.analyze(&steps).unwrap();
        let order1 = analyzer.canonicalize_order(&graph).unwrap();
        let order2 = analyzer.canonicalize_order(&graph).unwrap();

        // Should be deterministic
        assert_eq!(order1, order2);
        // Should be sorted alphabetically when no dependencies
        assert_eq!(order1, vec!["step_a", "step_b", "step_c"]);
    }

    #[test]
    fn test_order_canonicalizer() {
        let canonicalizer = OrderCanonicalizer::new();

        let graph = DependencyGraph {
            nodes: vec!["step1".to_string(), "step2".to_string()],
            edges: vec![("step1".to_string(), "step2".to_string())],
        };

        // Valid ordering
        let valid_order = vec!["step1".to_string(), "step2".to_string()];
        let result = canonicalizer.validate_ordering(&valid_order, &graph);
        assert!(result.is_ok());

        // Invalid ordering (violates dependency)
        let invalid_order = vec!["step2".to_string(), "step1".to_string()];
        let result = canonicalizer.validate_ordering(&invalid_order, &graph);
        assert!(result.is_err());
    }
}
