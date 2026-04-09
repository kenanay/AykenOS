//! Lightweight compatibility pools for semantic-cli execution and Gate C normalization.

use crate::ir_planner::register_file::RegisterFile;
use crate::ir_planner::replay::ReplayRecorder;
use crate::ir_planner::ExecutionState;
use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct PoolStats {
    pub borrows: usize,
    pub returns: usize,
    pub clears: usize,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PoolSizes {
    pub index_vec_pool: usize,
    pub context_map_pool: usize,
    pub register_file_pool: usize,
    pub replay_recorder_pool: usize,
    pub execution_state_pool: usize,
}

#[derive(Debug, Default)]
pub struct ExecutionPools {
    index_vec_pool: Vec<Vec<usize>>,
    context_map_pool: Vec<HashMap<String, String>>,
    register_file_pool: Vec<RegisterFile>,
    replay_recorder_pool: Vec<ReplayRecorder>,
    execution_state_pool: Vec<ExecutionState>,
    stats: PoolStats,
}

impl ExecutionPools {
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            index_vec_pool: Vec::with_capacity(capacity),
            context_map_pool: Vec::with_capacity(capacity),
            register_file_pool: Vec::with_capacity(capacity),
            replay_recorder_pool: Vec::with_capacity(capacity),
            execution_state_pool: Vec::with_capacity(capacity),
            stats: PoolStats::default(),
        }
    }

    pub fn stats(&self) -> &PoolStats {
        &self.stats
    }

    pub fn pool_sizes(&self) -> PoolSizes {
        PoolSizes {
            index_vec_pool: self.index_vec_pool.len(),
            context_map_pool: self.context_map_pool.len(),
            register_file_pool: self.register_file_pool.len(),
            replay_recorder_pool: self.replay_recorder_pool.len(),
            execution_state_pool: self.execution_state_pool.len(),
        }
    }

    pub fn borrow_index_vec(&mut self) -> Vec<usize> {
        self.stats.borrows += 1;
        self.index_vec_pool.pop().unwrap_or_default()
    }

    pub fn return_index_vec(&mut self, mut value: Vec<usize>) {
        value.clear();
        self.stats.returns += 1;
        self.index_vec_pool.push(value);
    }

    pub fn borrow_context_map(&mut self) -> HashMap<String, String> {
        self.stats.borrows += 1;
        self.context_map_pool.pop().unwrap_or_default()
    }

    pub fn return_context_map(&mut self, mut value: HashMap<String, String>) {
        value.clear();
        self.stats.returns += 1;
        self.context_map_pool.push(value);
    }

    pub fn borrow_execution_state(&mut self) -> ExecutionState {
        self.stats.borrows += 1;
        self.execution_state_pool.pop().unwrap_or_default()
    }

    pub fn borrow_register_file(&mut self) -> RegisterFile {
        self.stats.borrows += 1;
        self.register_file_pool.pop().unwrap_or_default()
    }

    pub fn borrow_replay_recorder(&mut self) -> ReplayRecorder {
        self.stats.borrows += 1;
        self.replay_recorder_pool.pop().unwrap_or_default()
    }

    pub fn clear_all(&mut self) {
        self.stats.clears += 1;
        for file in &mut self.register_file_pool {
            file.clear();
        }
        for recorder in &mut self.replay_recorder_pool {
            recorder.clear();
        }
        self.index_vec_pool.clear();
        self.context_map_pool.clear();
        self.register_file_pool.clear();
        self.replay_recorder_pool.clear();
        self.execution_state_pool.clear();
    }
}

pub mod allocation_optimizer {
    #[derive(Debug, Clone, Default)]
    pub struct AllocationOptimizer {
        pub allocations_avoided: usize,
        pub bytes_saved: usize,
    }

    impl AllocationOptimizer {
        pub fn new() -> Self {
            Self::default()
        }

        pub fn record_format_allocation_avoided(&mut self, bytes_saved: usize) {
            self.allocations_avoided += 1;
            self.bytes_saved += bytes_saved;
        }
    }
}
