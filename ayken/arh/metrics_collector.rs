// Constitutional Module: Metrics Collector
// Metrics collection is fail-closed and local-only. No disk/network/telemetry.

use std::collections::BTreeMap;

use crate::arh::analytics::FixApplicationEvent;

pub trait MetricsSink {
    fn record_fix_event(&mut self, event: FixApplicationEvent);
    fn record_hint_generated(&mut self, pattern_id: &str);
    fn record_violation_seen(&mut self, rule_id: &str);
}

pub struct InMemoryMetricsCollector {
    pub fix_events: Vec<FixApplicationEvent>,
    pub pattern_usage: BTreeMap<String, usize>,
    pub violation_frequency: BTreeMap<String, usize>,
}

impl InMemoryMetricsCollector {
    pub fn new() -> Self {
        Self {
            fix_events: Vec::new(),
            pattern_usage: BTreeMap::new(),
            violation_frequency: BTreeMap::new(),
        }
    }
}

impl MetricsSink for InMemoryMetricsCollector {
    fn record_fix_event(&mut self, event: FixApplicationEvent) {
        self.fix_events.push(event);
    }

    fn record_hint_generated(&mut self, pattern_id: &str) {
        *self.pattern_usage.entry(pattern_id.to_string()).or_insert(0) += 1;
    }

    fn record_violation_seen(&mut self, rule_id: &str) {
        *self.violation_frequency.entry(rule_id.to_string()).or_insert(0) += 1;
    }
}
