//! Deterministic regression tests (property-style invariants).
//! Guarantees: duplicate work keys fail closed; incremental recompute logic.

#[cfg(test)]
mod regression_tests {
    use crate::arh::incremental_analysis::IncrementalIndex;
    use crate::arh::parallel_processing::{ParallelProcessor, WorkItem, WorkResult};
    use crate::arh::performance::{
        fallback_hint_policy, FallbackPolicy, PerfProfile, PerfStage, PerformanceMonitor,
    };

    #[test]
    fn duplicate_work_keys_fail_closed() {
        let processor = ParallelProcessor;
        let items = vec![
            WorkItem { key: "k".to_string(), is_kernel: false },
            WorkItem { key: "k".to_string(), is_kernel: false },
        ];
        let result = processor.process_deterministic(&items, |i| WorkResult {
            key: i.key.clone(),
            output: "x".to_string(),
        });
        assert!(result.is_err(), "duplicate keys must fail closed");
    }

    #[test]
    fn incremental_recompute_only_on_changes() {
        let mut idx = IncrementalIndex::new();
        let file = "f";
        let violation = "v";
        let rule = "R";
        let cfg = "c";
        let profile = PerfProfile::VsCodeRealtime;
        let hash = "h";

        assert!(idx.should_recompute(file, hash, violation, rule, cfg, profile));
        idx.update(file, hash, violation, rule, cfg, profile, "out1");
        assert!(!idx.should_recompute(file, hash, violation, rule, cfg, profile));
        assert!(idx.should_recompute(file, hash, violation, "R2", cfg, profile));
        assert!(idx.should_recompute(file, hash, violation, rule, "cfg2", profile));
        assert!(idx.should_recompute(file, hash, violation, rule, cfg, PerfProfile::CiBatch));
        assert!(idx.should_recompute(file, hash, "v2", rule, cfg, profile));

        assert_eq!(idx.output_change_count(file), 0);
        idx.update(file, hash, violation, rule, cfg, profile, "out1");
        assert_eq!(idx.output_change_count(file), 0);
        idx.update(file, hash, violation, rule, cfg, profile, "out2");
        assert_eq!(idx.output_change_count(file), 1);
    }

    #[test]
    fn performance_budget_exceeded_triggers_fallback() {
        let mut monitor = PerformanceMonitor::new(PerfProfile::VsCodeRealtime);
        monitor.record(PerfStage::Pattern, 1000, Some(2048));
        let report = monitor.report();
        assert!(report.exceeded, "budget overage must be reported");
        assert!(
            !report.exceeded_reasons.is_empty(),
            "exceeded reasons must be reported"
        );
        assert_eq!(
            fallback_hint_policy(report.exceeded),
            FallbackPolicy::CacheOrEmpty
        );
    }
}
