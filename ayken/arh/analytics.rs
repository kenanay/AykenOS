// Constitutional Module: Analytics
// Analytics must never mutate code, enforce decisions, or profile identities.

use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FixOutcome {
    Applied,
    RolledBack,
    Failed,
    Skipped,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FixApplicationEvent {
    pub timestamp: String,
    pub module_id: String,
    pub is_kernel: bool,
    pub outcome: FixOutcome,
    pub violation_count: usize,
    pub pattern_ids: Vec<String>,
    pub duration_ms: u64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AnalyticsSnapshot {
    pub total_fixes: usize,
    pub success_rate: f32,
    pub rollback_rate: f32,
    pub kernel_fix_attempts: usize,
    pub avg_duration_ms: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ArchitecturalTrend {
    pub timestamp: String,
    pub violation_count: usize,
    pub constitutional_conflicts: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PatternEffectiveness {
    pub pattern_id: String,
    pub applied_count: usize,
    pub rollback_count: usize,
}

pub struct ARHAnalytics;

impl ARHAnalytics {
    pub fn snapshot(events: &[FixApplicationEvent]) -> AnalyticsSnapshot {
        let total_fixes = events.len();
        let applied = events.iter().filter(|e| e.outcome == FixOutcome::Applied).count();
        let rolled_back = events.iter().filter(|e| e.outcome == FixOutcome::RolledBack).count();
        let kernel_fix_attempts = events.iter().filter(|e| e.is_kernel).count();
        let avg_duration_ms = if total_fixes == 0 {
            0
        } else {
            let sum: u64 = events.iter().map(|e| e.duration_ms).sum();
            sum / total_fixes as u64
        };

        AnalyticsSnapshot {
            total_fixes,
            success_rate: if total_fixes == 0 { 0.0 } else { applied as f32 / total_fixes as f32 },
            rollback_rate: if total_fixes == 0 { 0.0 } else { rolled_back as f32 / total_fixes as f32 },
            kernel_fix_attempts,
            avg_duration_ms,
        }
    }

    pub fn pattern_effectiveness(events: &[FixApplicationEvent]) -> Vec<PatternEffectiveness> {
        let mut map: BTreeMap<String, PatternEffectiveness> = BTreeMap::new();
        for event in events {
            for pattern_id in &event.pattern_ids {
                let entry = map.entry(pattern_id.clone()).or_insert(PatternEffectiveness {
                    pattern_id: pattern_id.clone(),
                    applied_count: 0,
                    rollback_count: 0,
                });
                match event.outcome {
                    FixOutcome::Applied => entry.applied_count += 1,
                    FixOutcome::RolledBack => entry.rollback_count += 1,
                    _ => {}
                }
            }
        }
        map.into_values().collect()
    }
}
