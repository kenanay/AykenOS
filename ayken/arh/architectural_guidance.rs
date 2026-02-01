// Constitutional Module: ArchitecturalGuidance
// This module MUST NOT mutate code, generate patches, or apply edits.
// All outputs are educational and advisory-only.
// Forbidden behaviors: file writes, patch emission, workspace edits, auto-apply.

//! Architectural guidance content (advisory-only, non-enforcing).

use crate::arh::design_hint_engine::ViolationType;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GuidanceSection {
    pub title: String,
    pub body: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ArchitecturalGuidance {
    pub architectural_intent: String,
    pub sections: Vec<GuidanceSection>,
}

impl ArchitecturalGuidance {
    pub fn for_violation(violation: ViolationType, is_kernel: bool) -> Self {
        match violation {
            ViolationType::AllocGlobal => Self {
                architectural_intent: "Preserve allocation locality and deterministic resource boundaries.".to_string(),
                sections: alloc_global_sections(is_kernel),
            },
            ViolationType::DeterminismRng => Self {
                architectural_intent: "Maintain deterministic behavior by controlling randomness sources.".to_string(),
                sections: determinism_rng_sections(is_kernel),
            },
        }
    }
}

fn alloc_global_sections(is_kernel: bool) -> Vec<GuidanceSection> {
    let mut sections = vec![
        GuidanceSection {
            title: "Why this violation exists".to_string(),
            body: vec![
                "Global allocation hides lifetime boundaries and breaks locality assumptions.".to_string(),
                "Unscoped allocation undermines deterministic memory behavior.".to_string(),
            ],
        },
        GuidanceSection {
            title: "Architectural intent behind the rule".to_string(),
            body: vec![
                "Encourage explicit ownership and bounded lifetimes for allocation.".to_string(),
                "Make allocation costs visible and attributable.".to_string(),
            ],
        },
        GuidanceSection {
            title: "Valid architectural options".to_string(),
            body: vec![
                "Subsystem-scoped arena allocation.".to_string(),
                "Region allocation with explicit teardown boundaries.".to_string(),
                "Thread-local allocators with bounded reuse.".to_string(),
            ],
        },
    ];

    if is_kernel {
        sections.push(GuidanceSection {
            title: "Kernel risk notice".to_string(),
            body: vec![
                "Kernel allocation scope changes can destabilize memory safety guarantees.".to_string(),
                "Prefer reversible steps and staged rollout with review.".to_string(),
            ],
        });
    }

    sections
}

fn determinism_rng_sections(is_kernel: bool) -> Vec<GuidanceSection> {
    let mut sections = vec![
        GuidanceSection {
            title: "Why this violation exists".to_string(),
            body: vec![
                "Unseeded randomness breaks deterministic replay and auditability.".to_string(),
                "Implicit RNG sources introduce hidden nondeterminism.".to_string(),
            ],
        },
        GuidanceSection {
            title: "Architectural intent behind the rule".to_string(),
            body: vec![
                "Make randomness explicit and traceable through seeded providers.".to_string(),
                "Preserve deterministic testing and simulation.".to_string(),
            ],
        },
        GuidanceSection {
            title: "Valid architectural options".to_string(),
            body: vec![
                "SeededRng trait injection at subsystem boundaries.".to_string(),
                "Deterministic RNG wrapper with documented seed policy.".to_string(),
                "Test-only RNG providers with explicit seed management.".to_string(),
            ],
        },
    ];

    if is_kernel {
        sections.push(GuidanceSection {
            title: "Kernel risk notice".to_string(),
            body: vec![
                "Kernel RNG usage impacts system-wide determinism guarantees.".to_string(),
                "Require architecture review before changing RNG boundaries.".to_string(),
            ],
        });
    }

    sections
}
