// Constitutional Module: DesignHintEngine
// This module MUST NOT mutate code, generate patches, or apply edits.
// All outputs are educational and advisory-only.
// Forbidden behaviors: file writes, patch emission, workspace edits, auto-apply.

//! Design hint orchestration (advisory-only, non-enforcing).

use crate::arh::architectural_guidance::{ArchitecturalGuidance, GuidanceSection};
use crate::arh::implementation_roadmaps::ImplementationRoadmap;
use crate::arh::educational_content::EducationalContent;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ViolationType {
    AllocGlobal,
    DeterminismRng,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DesignHintRequest {
    pub violation: ViolationType,
    pub is_kernel: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DesignHintOutput {
    pub violation: ViolationType,
    pub architectural_intent: String,
    pub design_options: Vec<String>,
    pub trade_offs: TradeOffMatrix,
    pub roadmap: ImplementationRoadmap,
    pub misapplication_risks: Vec<String>,
    pub educational_notes: EducationalContent,
    pub related_arre_patterns: Vec<String>,
    pub kernel_risk_notice: Option<String>,
    pub sections: Vec<GuidanceSection>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TradeOffMatrixRow {
    pub option: String,
    pub performance: String,
    pub safety: String,
    pub complexity: String,
    pub reversibility: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TradeOffMatrix {
    pub rows: Vec<TradeOffMatrixRow>,
}

pub struct DesignHintEngine;

impl DesignHintEngine {
    /// Generate design hints without mutating any source or enforcing decisions.
    pub fn generate(&self, request: DesignHintRequest) -> DesignHintOutput {
        let guidance = ArchitecturalGuidance::for_violation(request.violation, request.is_kernel);
        let roadmap = ImplementationRoadmap::for_violation(request.violation);
        let education = EducationalContent::for_violation(request.violation);

        let (trade_offs, options, misapplication, arre_patterns) = match request.violation {
            ViolationType::AllocGlobal => (
                alloc_global_tradeoffs(),
                alloc_global_options(),
                alloc_global_misapplication_risks(),
                alloc_global_arre_patterns(),
            ),
            ViolationType::DeterminismRng => (
                determinism_rng_tradeoffs(),
                determinism_rng_options(),
                determinism_rng_misapplication_risks(),
                determinism_rng_arre_patterns(),
            ),
        };

        DesignHintOutput {
            violation: request.violation,
            architectural_intent: guidance.architectural_intent.clone(),
            design_options: options,
            trade_offs,
            roadmap,
            misapplication_risks: misapplication,
            educational_notes: education,
            related_arre_patterns: arre_patterns,
            kernel_risk_notice: if request.is_kernel {
                Some("Kernel context: elevated risk; avoid irreversible changes without architecture review.".to_string())
            } else {
                None
            },
            sections: guidance.sections,
        }
    }
}

fn alloc_global_options() -> Vec<String> {
    vec![
        "Arena allocator scoped per subsystem".to_string(),
        "Region allocator with explicit lifetime boundaries".to_string(),
        "Thread-local allocator with bounded reuse".to_string(),
    ]
}

fn determinism_rng_options() -> Vec<String> {
    vec![
        "SeededRng trait injected at boundary".to_string(),
        "Deterministic RNG wrapper with explicit seed provenance".to_string(),
        "Test-only RNG provider with stable seed policy".to_string(),
    ]
}

fn alloc_global_tradeoffs() -> TradeOffMatrix {
    TradeOffMatrix {
        rows: vec![
            TradeOffMatrixRow {
                option: "Arena allocator scoped per subsystem".to_string(),
                performance: "High (cache locality)".to_string(),
                safety: "Medium (lifetime discipline required)".to_string(),
                complexity: "Medium".to_string(),
                reversibility: "Medium".to_string(),
            },
            TradeOffMatrixRow {
                option: "Region allocator with explicit boundaries".to_string(),
                performance: "Medium".to_string(),
                safety: "High".to_string(),
                complexity: "High".to_string(),
                reversibility: "Low".to_string(),
            },
            TradeOffMatrixRow {
                option: "Thread-local allocator with reuse".to_string(),
                performance: "Medium-High".to_string(),
                safety: "Medium".to_string(),
                complexity: "Medium".to_string(),
                reversibility: "High".to_string(),
            },
        ],
    }
}

fn determinism_rng_tradeoffs() -> TradeOffMatrix {
    TradeOffMatrix {
        rows: vec![
            TradeOffMatrixRow {
                option: "SeededRng trait injection".to_string(),
                performance: "Medium".to_string(),
                safety: "High (deterministic)".to_string(),
                complexity: "Medium".to_string(),
                reversibility: "High".to_string(),
            },
            TradeOffMatrixRow {
                option: "Deterministic RNG wrapper".to_string(),
                performance: "Medium".to_string(),
                safety: "High".to_string(),
                complexity: "Low-Medium".to_string(),
                reversibility: "High".to_string(),
            },
            TradeOffMatrixRow {
                option: "Test-only RNG provider".to_string(),
                performance: "High (tests only)".to_string(),
                safety: "Medium".to_string(),
                complexity: "Low".to_string(),
                reversibility: "High".to_string(),
            },
        ],
    }
}

fn alloc_global_misapplication_risks() -> Vec<String> {
    vec![
        "Arena lifetime mismatch causing retained memory".to_string(),
        "Global allocation pressure hiding leaks".to_string(),
        "Cache locality regressions if arena scope is too broad".to_string(),
    ]
}

fn determinism_rng_misapplication_risks() -> Vec<String> {
    vec![
        "Seed reuse across contexts causing correlated randomness".to_string(),
        "Hidden non-determinism when seeds are derived from time".to_string(),
        "Test determinism lost if seed policy is not documented".to_string(),
    ]
}

fn alloc_global_arre_patterns() -> Vec<String> {
    vec![
        "ARRE::ALLOC::ARENA_SCOPE".to_string(),
        "ARRE::ALLOC::REGION_LIFETIME".to_string(),
    ]
}

fn determinism_rng_arre_patterns() -> Vec<String> {
    vec![
        "ARRE::DETERMINISM::SEEDED_RNG".to_string(),
        "ARRE::DETERMINISM::RNG_PROVIDER".to_string(),
    ]
}
