use super::{IRPlanner, SemanticAnalysis};
use crate::gate_c::error::GateCResult;
use crate::gate_c::types::ExecutionPlan;

pub struct OptimizedIRExecutor {
    inner: IRPlanner,
}

impl OptimizedIRExecutor {
    pub fn new() -> Self {
        Self {
            inner: IRPlanner::new(),
        }
    }

    pub fn analyze_plan(&self, plan: &ExecutionPlan) -> GateCResult<SemanticAnalysis> {
        self.inner.analyze_plan(plan)
    }
}

impl Default for OptimizedIRExecutor {
    fn default() -> Self {
        Self::new()
    }
}
