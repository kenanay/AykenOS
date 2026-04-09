use super::{IRPlanner, SemanticAnalysis};
use crate::gate_c::error::GateCResult;
use crate::gate_c::types::ExecutionPlan;

pub struct OptimizedSemanticAnalyzer {
    inner: IRPlanner,
}

impl OptimizedSemanticAnalyzer {
    pub fn new() -> Self {
        Self {
            inner: IRPlanner::new(),
        }
    }

    pub fn analyze_plan(&self, plan: &ExecutionPlan) -> GateCResult<SemanticAnalysis> {
        self.inner.analyze_plan(plan)
    }
}

impl Default for OptimizedSemanticAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}
