// Constitutional Module: ImplementationRoadmaps
// This module MUST NOT mutate code, generate patches, or apply edits.
// All outputs are educational and advisory-only.
// Forbidden behaviors: file writes, patch emission, workspace edits, auto-apply.

//! Implementation roadmaps (advisory-only, non-enforcing).

use crate::arh::design_hint_engine::ViolationType;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EffortLevel {
    Low,
    Medium,
    High,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RoadmapStep {
    pub title: String,
    pub description: String,
    pub effort: EffortLevel,
    pub decision_point: String,
    pub misapplication_risk: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ImplementationRoadmap {
    pub steps: Vec<RoadmapStep>,
}

impl ImplementationRoadmap {
    pub fn for_violation(violation: ViolationType) -> Self {
        match violation {
            ViolationType::AllocGlobal => Self {
                steps: alloc_global_steps(),
            },
            ViolationType::DeterminismRng => Self {
                steps: determinism_rng_steps(),
            },
        }
    }
}

fn alloc_global_steps() -> Vec<RoadmapStep> {
    vec![
        RoadmapStep {
            title: "Inventory allocation hotspots".to_string(),
            description: "Map where global allocation is currently used and why.".to_string(),
            effort: EffortLevel::Low,
            decision_point: "Is allocation scope bound to a subsystem?".to_string(),
            misapplication_risk: "Missing hotspots leads to partial fixes.".to_string(),
        },
        RoadmapStep {
            title: "Select an arena or region boundary".to_string(),
            description: "Define explicit lifetime boundaries for allocation.".to_string(),
            effort: EffortLevel::Medium,
            decision_point: "Is the boundary reversible without data loss?".to_string(),
            misapplication_risk: "Over-broad scope retains memory unnecessarily.".to_string(),
        },
        RoadmapStep {
            title: "Introduce allocator interface".to_string(),
            description: "Expose allocation through a controlled interface.".to_string(),
            effort: EffortLevel::High,
            decision_point: "Does the interface preserve locality?".to_string(),
            misapplication_risk: "Interface sprawl can reintroduce global usage.".to_string(),
        },
    ]
}

fn determinism_rng_steps() -> Vec<RoadmapStep> {
    vec![
        RoadmapStep {
            title: "Define seed policy".to_string(),
            description: "Document deterministic seed provenance.".to_string(),
            effort: EffortLevel::Low,
            decision_point: "Is seed generation reproducible?".to_string(),
            misapplication_risk: "Time-derived seeds reintroduce nondeterminism.".to_string(),
        },
        RoadmapStep {
            title: "Introduce RNG provider interface".to_string(),
            description: "Route RNG usage through an injected provider.".to_string(),
            effort: EffortLevel::Medium,
            decision_point: "Can provider be swapped for tests?".to_string(),
            misapplication_risk: "Implicit globals bypass the provider.".to_string(),
        },
        RoadmapStep {
            title: "Audit call sites".to_string(),
            description: "Verify all randomness flows through the provider.".to_string(),
            effort: EffortLevel::High,
            decision_point: "Is the RNG boundary enforced consistently?".to_string(),
            misapplication_risk: "Hidden RNG usage undermines determinism.".to_string(),
        },
    ]
}
