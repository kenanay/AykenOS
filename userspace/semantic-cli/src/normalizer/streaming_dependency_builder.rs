#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct IndexedDependencyGraph {
    pub instruction_count: usize,
}

pub struct StreamingDependencyBuilder {
    graph: IndexedDependencyGraph,
}

impl StreamingDependencyBuilder {
    pub fn new(_expected_instruction_count: usize) -> Self {
        Self {
            graph: IndexedDependencyGraph::default(),
        }
    }

    pub fn build_for_instruction_count(&mut self, instruction_count: usize) -> IndexedDependencyGraph {
        self.graph = IndexedDependencyGraph { instruction_count };
        self.graph.clone()
    }
}
