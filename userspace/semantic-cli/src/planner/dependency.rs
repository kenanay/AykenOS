//! Dependency analysis for execution plan steps
//!
//! This module provides functionality for analyzing dependencies between
//! execution plan steps, ensuring proper ordering and resource coordination.

use crate::types::*;
use crate::error::PlanningError;
use std::collections::{HashMap, HashSet, VecDeque};
use tracing::debug;

/// Dependency analyzer for execution plan steps
pub struct DependencyAnalyzer {
    /// Configuration for dependency analysis
    config: DependencyAnalysisConfig,
}

/// Configuration for dependency analysis
#[derive(Debug, Clone)]
pub struct DependencyAnalysisConfig {
    /// Whether to detect circular dependencies
    pub detect_circular_dependencies: bool,
    /// Whether to optimize dependency chains
    pub optimize_chains: bool,
    /// Maximum dependency chain length
    pub max_chain_length: usize,
    /// Whether to allow parallel execution
    pub allow_parallel_execution: bool,
}

impl Default for DependencyAnalysisConfig {
    fn default() -> Self {
        Self {
            detect_circular_dependencies: true,
            optimize_chains: true,
            max_chain_length: 20,
            allow_parallel_execution: true,
        }
    }
}

/// Dependency analysis result
#[derive(Debug, Clone)]
pub struct DependencyAnalysisResult {
    /// Identified dependencies
    pub dependencies: Vec<Dependency>,
    /// Execution order based on dependencies
    pub execution_order: Vec<StepId>,
    /// Steps that can be executed in parallel
    pub parallel_groups: Vec<Vec<StepId>>,
    /// Circular dependencies found (if any)
    pub circular_dependencies: Vec<Vec<StepId>>,
    /// Critical path through the dependencies
    pub critical_path: Vec<StepId>,
}

/// Dependency graph for analysis
#[derive(Debug, Clone)]
struct DependencyGraph {
    /// Adjacency list representation
    adjacency_list: HashMap<StepId, Vec<StepId>>,
    /// Reverse adjacency list (dependents)
    reverse_adjacency_list: HashMap<StepId, Vec<StepId>>,
    /// Step information
    steps: HashMap<StepId, PlanStep>,
}

impl DependencyAnalyzer {
    /// Create a new dependency analyzer
    pub fn new() -> Self {
        Self {
            config: DependencyAnalysisConfig::default(),
        }
    }

    /// Create analyzer with custom configuration
    pub fn with_config(config: DependencyAnalysisConfig) -> Self {
        Self { config }
    }

    /// Analyze dependencies between execution steps
    pub async fn analyze_dependencies(&self, steps: &[PlanStep]) -> Result<Vec<Dependency>, PlanningError> {
        debug!("Analyzing dependencies for {} steps", steps.len());

        let mut dependencies = Vec::new();

        // Build condition maps for efficient lookup
        let postcondition_map = self.build_postcondition_map(steps);
        let _precondition_map = self.build_precondition_map(steps);

        // Find explicit dependencies based on pre/postconditions
        dependencies.extend(self.find_condition_dependencies(steps, &postcondition_map)?);

        // Find implicit dependencies based on step semantics
        dependencies.extend(self.find_semantic_dependencies(steps)?);

        // Find resource dependencies
        dependencies.extend(self.find_resource_dependencies(steps)?);

        // Optimize dependencies if configured
        if self.config.optimize_chains {
            dependencies = self.optimize_dependency_chains(dependencies)?;
        }

        // Validate dependencies
        if let Err(validation_error) = self.validate_dependencies(&dependencies, steps).await {
            return Err(PlanningError::ValidationFailed { reason: validation_error });
        }

        debug!("Found {} dependencies", dependencies.len());
        Ok(dependencies)
    }

    /// Perform comprehensive dependency analysis
    pub async fn analyze_comprehensive(&self, steps: &[PlanStep]) -> Result<DependencyAnalysisResult, PlanningError> {
        debug!("Performing comprehensive dependency analysis");

        // Get basic dependencies
        let dependencies = self.analyze_dependencies(steps).await?;

        // Build dependency graph
        let graph = self.build_dependency_graph(steps, &dependencies)?;

        // Check for circular dependencies
        let circular_dependencies = if self.config.detect_circular_dependencies {
            self.detect_circular_dependencies(&graph)?
        } else {
            Vec::new()
        };

        // Calculate execution order
        let execution_order = self.calculate_execution_order(&graph)?;

        // Find parallel execution groups
        let parallel_groups = if self.config.allow_parallel_execution {
            self.find_parallel_groups(&graph, &execution_order)?
        } else {
            Vec::new()
        };

        // Calculate critical path
        let critical_path = self.calculate_critical_path(&graph, steps)?;

        Ok(DependencyAnalysisResult {
            dependencies,
            execution_order,
            parallel_groups,
            circular_dependencies,
            critical_path,
        })
    }

    /// Validate dependencies against steps
    pub async fn validate_dependencies(
        &self,
        dependencies: &[Dependency],
        steps: &[PlanStep],
    ) -> Result<(), String> {
        debug!("Validating {} dependencies", dependencies.len());

        let step_ids: HashSet<StepId> = steps.iter().map(|s| s.id).collect();

        for dependency in dependencies {
            // Check that both steps exist
            if !step_ids.contains(&dependency.prerequisite) {
                return Err(format!("Prerequisite step not found: {}", dependency.prerequisite));
            }
            if !step_ids.contains(&dependency.dependent) {
                return Err(format!("Dependent step not found: {}", dependency.dependent));
            }

            // Check for self-dependency
            if dependency.prerequisite == dependency.dependent {
                return Err(format!("Step cannot depend on itself: {}", dependency.prerequisite));
            }
        }

        // Check for circular dependencies if configured
        if self.config.detect_circular_dependencies {
            let graph = self.build_dependency_graph(steps, dependencies)
                .map_err(|e| format!("Failed to build dependency graph: {:?}", e))?;
            let circular_deps = self.detect_circular_dependencies(&graph)
                .map_err(|e| format!("Failed to detect circular dependencies: {:?}", e))?;
            if !circular_deps.is_empty() {
                return Err(format!("Circular dependencies detected: {} cycles", circular_deps.len()));
            }
        }

        Ok(())
    }

    /// Build postcondition map for efficient lookup
    fn build_postcondition_map<'a>(&self, steps: &'a [PlanStep]) -> HashMap<String, Vec<&'a PlanStep>> {
        let mut map = HashMap::new();
        
        for step in steps {
            for postcondition in &step.postconditions {
                map.entry(postcondition.description.clone())
                   .or_insert_with(Vec::new)
                   .push(step);
            }
        }
        
        map
    }

    /// Build precondition map for efficient lookup
    fn build_precondition_map<'a>(&self, steps: &'a [PlanStep]) -> HashMap<String, Vec<&'a PlanStep>> {
        let mut map = HashMap::new();
        
        for step in steps {
            for precondition in &step.preconditions {
                map.entry(precondition.description.clone())
                   .or_insert_with(Vec::new)
                   .push(step);
            }
        }
        
        map
    }

    /// Find dependencies based on pre/postconditions
    fn find_condition_dependencies(
        &self,
        steps: &[PlanStep],
        postcondition_map: &HashMap<String, Vec<&PlanStep>>,
    ) -> Result<Vec<Dependency>, PlanningError> {
        let mut dependencies = Vec::new();

        for step in steps {
            for precondition in &step.preconditions {
                if let Some(providers) = postcondition_map.get(&precondition.description) {
                    for provider in providers {
                        if provider.id != step.id {
                            dependencies.push(Dependency {
                                prerequisite: provider.id,
                                dependent: step.id,
                                dependency_type: DependencyType::Success,
                            });
                        }
                    }
                }
            }
        }

        Ok(dependencies)
    }

    /// Find semantic dependencies based on step types
    fn find_semantic_dependencies(&self, steps: &[PlanStep]) -> Result<Vec<Dependency>, PlanningError> {
        let mut dependencies = Vec::new();

        // Define semantic dependency rules
        let dependency_rules = self.get_semantic_dependency_rules();

        for i in 0..steps.len() {
            for j in i + 1..steps.len() {
                let step_a = &steps[i];
                let step_b = &steps[j];

                // Check if step_a should come before step_b
                if self.should_step_precede(step_a, step_b, &dependency_rules) {
                    dependencies.push(Dependency {
                        prerequisite: step_a.id,
                        dependent: step_b.id,
                        dependency_type: DependencyType::Success,
                    });
                }
                // Check if step_b should come before step_a
                else if self.should_step_precede(step_b, step_a, &dependency_rules) {
                    dependencies.push(Dependency {
                        prerequisite: step_b.id,
                        dependent: step_a.id,
                        dependency_type: DependencyType::Success,
                    });
                }
            }
        }

        Ok(dependencies)
    }

    /// Get semantic dependency rules
    fn get_semantic_dependency_rules(&self) -> HashMap<String, Vec<String>> {
        let mut rules = HashMap::new();

        // Validation steps should come before execution steps
        rules.insert("validate_".to_string(), vec![
            "execute_".to_string(),
            "perform_".to_string(),
            "apply_".to_string(),
        ]);

        // Backup steps should come before modification steps
        rules.insert("backup_".to_string(), vec![
            "apply_".to_string(),
            "modify_".to_string(),
            "delete_".to_string(),
        ]);

        // Preparation steps should come before execution
        rules.insert("prepare_".to_string(), vec![
            "execute_".to_string(),
            "perform_".to_string(),
            "start_".to_string(),
        ]);

        // Configuration steps should come before dependent operations
        rules.insert("configure_".to_string(), vec![
            "start_".to_string(),
            "initialize_".to_string(),
        ]);

        // Cleanup steps should come after main operations
        rules.insert("cleanup_".to_string(), vec![]);

        rules
    }

    /// Check if one step should precede another based on semantic rules
    fn should_step_precede(
        &self,
        step_a: &PlanStep,
        step_b: &PlanStep,
        rules: &HashMap<String, Vec<String>>,
    ) -> bool {
        for (prefix, dependent_prefixes) in rules {
            if step_a.command.starts_with(prefix) {
                for dependent_prefix in dependent_prefixes {
                    if step_b.command.starts_with(dependent_prefix) {
                        return true;
                    }
                }
            }
        }
        false
    }

    /// Find resource dependencies
    fn find_resource_dependencies(&self, steps: &[PlanStep]) -> Result<Vec<Dependency>, PlanningError> {
        let mut dependencies = Vec::new();

        // Group steps by resource usage patterns
        let file_operations = self.find_steps_with_pattern(steps, &["file_", "backup_", "restore_"]);
        let config_operations = self.find_steps_with_pattern(steps, &["config", "apply_config"]);
        let process_operations = self.find_steps_with_pattern(steps, &["process_", "start_", "stop_"]);

        // File operations should be serialized if they affect the same resources
        dependencies.extend(self.create_serialization_dependencies(&file_operations));

        // Configuration operations should be serialized
        dependencies.extend(self.create_serialization_dependencies(&config_operations));

        // Process operations should be carefully ordered
        dependencies.extend(self.create_serialization_dependencies(&process_operations));

        Ok(dependencies)
    }

    /// Find steps matching command patterns
    fn find_steps_with_pattern<'a>(&self, steps: &'a [PlanStep], patterns: &[&str]) -> Vec<&'a PlanStep> {
        steps.iter()
            .filter(|step| patterns.iter().any(|pattern| step.command.contains(pattern)))
            .collect()
    }

    /// Create serialization dependencies for a group of steps
    fn create_serialization_dependencies(&self, steps: &[&PlanStep]) -> Vec<Dependency> {
        let mut dependencies = Vec::new();

        for i in 0..steps.len() {
            for j in i + 1..steps.len() {
                dependencies.push(Dependency {
                    prerequisite: steps[i].id,
                    dependent: steps[j].id,
                    dependency_type: DependencyType::Success,
                });
            }
        }

        dependencies
    }

    /// Optimize dependency chains
    fn optimize_dependency_chains(&self, mut dependencies: Vec<Dependency>) -> Result<Vec<Dependency>, PlanningError> {
        // Remove redundant transitive dependencies
        dependencies = self.remove_transitive_dependencies(dependencies)?;

        // Merge compatible dependencies
        dependencies = self.merge_compatible_dependencies(dependencies)?;

        Ok(dependencies)
    }

    /// Remove transitive dependencies (A->B, B->C, A->C => remove A->C)
    fn remove_transitive_dependencies(&self, dependencies: Vec<Dependency>) -> Result<Vec<Dependency>, PlanningError> {
        let mut graph: HashMap<StepId, HashSet<StepId>> = HashMap::new();
        
        // Build adjacency list
        for dep in &dependencies {
            graph.entry(dep.prerequisite)
                 .or_insert_with(HashSet::new)
                 .insert(dep.dependent);
        }

        let mut optimized = Vec::new();

        for dep in dependencies {
            // Check if this dependency is transitive
            if !self.is_transitive_dependency(&dep, &graph) {
                optimized.push(dep);
            }
        }

        Ok(optimized)
    }

    /// Check if a dependency is transitive
    fn is_transitive_dependency(&self, dep: &Dependency, graph: &HashMap<StepId, HashSet<StepId>>) -> bool {
        // Check if there's a path from prerequisite to dependent through other nodes
        if let Some(direct_deps) = graph.get(&dep.prerequisite) {
            for intermediate in direct_deps {
                if *intermediate != dep.dependent {
                    if let Some(indirect_deps) = graph.get(intermediate) {
                        if indirect_deps.contains(&dep.dependent) {
                            return true; // Found transitive path
                        }
                    }
                }
            }
        }
        false
    }

    /// Merge compatible dependencies
    fn merge_compatible_dependencies(&self, dependencies: Vec<Dependency>) -> Result<Vec<Dependency>, PlanningError> {
        // For now, just return as-is. In a full implementation, this would
        // merge dependencies with the same prerequisite/dependent pairs
        Ok(dependencies)
    }

    /// Build dependency graph
    fn build_dependency_graph(
        &self,
        steps: &[PlanStep],
        dependencies: &[Dependency],
    ) -> Result<DependencyGraph, PlanningError> {
        let mut adjacency_list = HashMap::new();
        let mut reverse_adjacency_list = HashMap::new();
        let mut step_map = HashMap::new();

        // Initialize with all steps
        for step in steps {
            adjacency_list.insert(step.id, Vec::new());
            reverse_adjacency_list.insert(step.id, Vec::new());
            step_map.insert(step.id, step.clone());
        }

        // Add dependencies
        for dep in dependencies {
            adjacency_list.entry(dep.prerequisite)
                          .or_insert_with(Vec::new)
                          .push(dep.dependent);
            
            reverse_adjacency_list.entry(dep.dependent)
                                  .or_insert_with(Vec::new)
                                  .push(dep.prerequisite);
        }

        Ok(DependencyGraph {
            adjacency_list,
            reverse_adjacency_list,
            steps: step_map,
        })
    }

    /// Detect circular dependencies using DFS
    fn detect_circular_dependencies(&self, graph: &DependencyGraph) -> Result<Vec<Vec<StepId>>, PlanningError> {
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();
        let mut cycles = Vec::new();

        for step_id in graph.steps.keys() {
            if !visited.contains(step_id) {
                if let Some(cycle) = self.dfs_detect_cycle(*step_id, graph, &mut visited, &mut rec_stack, &mut Vec::new()) {
                    cycles.push(cycle);
                }
            }
        }

        Ok(cycles)
    }

    /// DFS helper for cycle detection
    fn dfs_detect_cycle(
        &self,
        node: StepId,
        graph: &DependencyGraph,
        visited: &mut HashSet<StepId>,
        rec_stack: &mut HashSet<StepId>,
        path: &mut Vec<StepId>,
    ) -> Option<Vec<StepId>> {
        visited.insert(node);
        rec_stack.insert(node);
        path.push(node);

        if let Some(neighbors) = graph.adjacency_list.get(&node) {
            for &neighbor in neighbors {
                if !visited.contains(&neighbor) {
                    if let Some(cycle) = self.dfs_detect_cycle(neighbor, graph, visited, rec_stack, path) {
                        return Some(cycle);
                    }
                } else if rec_stack.contains(&neighbor) {
                    // Found cycle
                    let cycle_start = path.iter().position(|&x| x == neighbor).unwrap();
                    return Some(path[cycle_start..].to_vec());
                }
            }
        }

        rec_stack.remove(&node);
        path.pop();
        None
    }

    /// Calculate execution order using topological sort
    fn calculate_execution_order(&self, graph: &DependencyGraph) -> Result<Vec<StepId>, PlanningError> {
        let mut in_degree = HashMap::new();
        let mut queue = VecDeque::new();
        let mut result = Vec::new();

        // Calculate in-degrees
        for step_id in graph.steps.keys() {
            in_degree.insert(*step_id, 0);
        }

        for (_, dependents) in &graph.adjacency_list {
            for &dependent in dependents {
                *in_degree.get_mut(&dependent).unwrap() += 1;
            }
        }

        // Add nodes with no incoming edges to queue
        for (&step_id, &degree) in &in_degree {
            if degree == 0 {
                queue.push_back(step_id);
            }
        }

        // Process queue
        while let Some(current) = queue.pop_front() {
            result.push(current);

            if let Some(dependents) = graph.adjacency_list.get(&current) {
                for &dependent in dependents {
                    let degree = in_degree.get_mut(&dependent).unwrap();
                    *degree -= 1;
                    if *degree == 0 {
                        queue.push_back(dependent);
                    }
                }
            }
        }

        // Check if all nodes were processed (no cycles)
        if result.len() != graph.steps.len() {
            return Err(PlanningError::CircularDependency);
        }

        Ok(result)
    }

    /// Find parallel execution groups
    fn find_parallel_groups(
        &self,
        graph: &DependencyGraph,
        execution_order: &[StepId],
    ) -> Result<Vec<Vec<StepId>>, PlanningError> {
        let mut groups = Vec::new();
        let mut processed = HashSet::new();

        for &step_id in execution_order {
            if processed.contains(&step_id) {
                continue;
            }

            // Find all steps that can run in parallel with this one
            let mut parallel_group = vec![step_id];
            processed.insert(step_id);

            // Look for steps that have no dependencies between them
            for &other_step in execution_order {
                if processed.contains(&other_step) {
                    continue;
                }

                if self.can_run_in_parallel(step_id, other_step, graph) {
                    parallel_group.push(other_step);
                    processed.insert(other_step);
                }
            }

            groups.push(parallel_group);
        }

        Ok(groups)
    }

    /// Check if two steps can run in parallel
    fn can_run_in_parallel(&self, step_a: StepId, step_b: StepId, graph: &DependencyGraph) -> bool {
        // Steps can run in parallel if there's no dependency path between them
        !self.has_dependency_path(step_a, step_b, graph) && 
        !self.has_dependency_path(step_b, step_a, graph)
    }

    /// Check if there's a dependency path from one step to another
    fn has_dependency_path(&self, from: StepId, to: StepId, graph: &DependencyGraph) -> bool {
        let mut visited = HashSet::new();
        self.dfs_path_exists(from, to, graph, &mut visited)
    }

    /// DFS to check if path exists
    fn dfs_path_exists(
        &self,
        current: StepId,
        target: StepId,
        graph: &DependencyGraph,
        visited: &mut HashSet<StepId>,
    ) -> bool {
        if current == target {
            return true;
        }

        visited.insert(current);

        if let Some(neighbors) = graph.adjacency_list.get(&current) {
            for &neighbor in neighbors {
                if !visited.contains(&neighbor) && self.dfs_path_exists(neighbor, target, graph, visited) {
                    return true;
                }
            }
        }

        false
    }

    /// Calculate critical path (longest path through dependencies)
    fn calculate_critical_path(
        &self,
        graph: &DependencyGraph,
        _steps: &[PlanStep],
    ) -> Result<Vec<StepId>, PlanningError> {
        let mut distances = HashMap::new();
        let mut predecessors = HashMap::new();

        // Initialize distances
        for step_id in graph.steps.keys() {
            distances.insert(*step_id, 0);
        }

        // Calculate longest paths (critical path)
        let execution_order = self.calculate_execution_order(graph)?;
        
        for &step_id in &execution_order {
            let step_duration = graph.steps.get(&step_id)
                .map(|s| s.timeout.as_secs())
                .unwrap_or(0);

            if let Some(dependents) = graph.adjacency_list.get(&step_id) {
                for &dependent in dependents {
                    let new_distance = distances[&step_id] + step_duration;
                    if new_distance > distances[&dependent] {
                        distances.insert(dependent, new_distance);
                        predecessors.insert(dependent, step_id);
                    }
                }
            }
        }

        // Find the step with maximum distance (end of critical path)
        let end_step = distances.iter()
            .max_by_key(|(_, &distance)| distance)
            .map(|(&step_id, _)| step_id)
            .ok_or(PlanningError::NoCriticalPath)?;

        // Reconstruct critical path
        let mut critical_path = Vec::new();
        let mut current = end_step;
        
        loop {
            critical_path.push(current);
            if let Some(&predecessor) = predecessors.get(&current) {
                current = predecessor;
            } else {
                break;
            }
        }

        critical_path.reverse();
        Ok(critical_path)
    }
}

impl Default for DependencyAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}