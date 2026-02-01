// Constitutional Module: Performance
// Performance optimizations must not change behavior or outputs.
// No heuristic shortcuts; only measurement and budget enforcement.

use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum PerfProfile {
    VsCodeRealtime,
    CiBatch,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PerformanceBudget {
    pub max_ms: u64,
    pub max_memory_kb: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum PerfStage {
    Pattern,
    Context,
    Semantic,
    Orchestration,
}

impl PerfStage {
    pub fn as_str(&self) -> &'static str {
        match self {
            PerfStage::Pattern => "pattern",
            PerfStage::Context => "context",
            PerfStage::Semantic => "semantic",
            PerfStage::Orchestration => "orchestration",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PerformanceSample {
    pub stage: PerfStage,
    pub elapsed_ms: u64,
    pub memory_kb: usize,
}

impl PerformanceSample {
    pub fn label(&self) -> &'static str {
        self.stage.as_str()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PerformanceReport {
    pub profile: PerfProfile,
    pub budget: PerformanceBudget,
    pub samples: Vec<PerformanceSample>,
    pub exceeded: bool,
    pub exceeded_reasons: Vec<String>,
}

pub struct PerformanceMonitor {
    budget: PerformanceBudget,
    profile: PerfProfile,
    samples: Vec<PerformanceSample>,
}

impl PerformanceMonitor {
    pub fn new(profile: PerfProfile) -> Self {
        let budget = match profile {
            PerfProfile::VsCodeRealtime => PerformanceBudget {
                max_ms: 200,
                max_memory_kb: 1024,
            },
            PerfProfile::CiBatch => PerformanceBudget {
                max_ms: 2000,
                max_memory_kb: 8192,
            },
        };
        Self {
            budget,
            profile,
            samples: Vec::new(),
        }
    }

    pub fn record(&mut self, stage: PerfStage, elapsed_ms: u64, memory_kb: Option<usize>) {
        let memory_kb = memory_kb.unwrap_or(self.budget.max_memory_kb + 1);
        self.samples.push(PerformanceSample {
            stage,
            elapsed_ms,
            memory_kb,
        });
    }

    pub fn report(self) -> PerformanceReport {
        let stage_budgets = ordered_stage_budget(self.profile);
        let total_elapsed: u64 = self.samples.iter().map(|s| s.elapsed_ms).sum();
        let max_memory: usize = self.samples.iter().map(|s| s.memory_kb).max().unwrap_or(0);

        let mut exceeded_reasons = Vec::new();
        let stage_exceeded = self.samples.iter().any(|s| {
            match stage_budgets.get(&s.stage) {
                Some(budget) => {
                    let exceeded = s.elapsed_ms > budget.max_ms || s.memory_kb > budget.max_memory_kb;
                    if exceeded {
                        exceeded_reasons.push(format!("stage {} exceeded", s.label()));
                    }
                    exceeded
                }
                None => {
                    exceeded_reasons.push(format!("stage {} has no budget", s.label()));
                    true
                }
            }
        });
        let total_exceeded = total_elapsed > self.budget.max_ms || max_memory > self.budget.max_memory_kb;
        if total_exceeded {
            exceeded_reasons.push("total budget exceeded".to_string());
        }
        let exceeded = stage_exceeded || total_exceeded;
        PerformanceReport {
            profile: self.profile,
            budget: self.budget,
            samples: self.samples,
            exceeded,
            exceeded_reasons,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FallbackPolicy {
    Normal,
    CacheOrEmpty,
}

pub fn fallback_hint_policy(exceeded: bool) -> FallbackPolicy {
    if exceeded {
        FallbackPolicy::CacheOrEmpty
    } else {
        FallbackPolicy::Normal
    }
}

pub fn ordered_stage_budget(profile: PerfProfile) -> BTreeMap<PerfStage, PerformanceBudget> {
    let mut map = BTreeMap::new();
    match profile {
        PerfProfile::VsCodeRealtime => {
            map.insert(
                PerfStage::Pattern,
                PerformanceBudget { max_ms: 30, max_memory_kb: 256 },
            );
            map.insert(
                PerfStage::Context,
                PerformanceBudget { max_ms: 30, max_memory_kb: 256 },
            );
            map.insert(
                PerfStage::Semantic,
                PerformanceBudget { max_ms: 30, max_memory_kb: 256 },
            );
            map.insert(
                PerfStage::Orchestration,
                PerformanceBudget { max_ms: 30, max_memory_kb: 256 },
            );
        }
        PerfProfile::CiBatch => {
            map.insert(
                PerfStage::Pattern,
                PerformanceBudget { max_ms: 300, max_memory_kb: 2048 },
            );
            map.insert(
                PerfStage::Context,
                PerformanceBudget { max_ms: 300, max_memory_kb: 2048 },
            );
            map.insert(
                PerfStage::Semantic,
                PerformanceBudget { max_ms: 300, max_memory_kb: 2048 },
            );
            map.insert(
                PerfStage::Orchestration,
                PerformanceBudget { max_ms: 300, max_memory_kb: 2048 },
            );
        }
    }
    map
}
