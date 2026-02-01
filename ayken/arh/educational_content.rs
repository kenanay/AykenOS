// Constitutional Module: EducationalContent
// This module MUST NOT mutate code, generate patches, or apply edits.
// All outputs are educational and advisory-only.
// Forbidden behaviors: file writes, patch emission, workspace edits, auto-apply.

//! Educational content for architectural guidance (advisory-only).

use crate::arh::design_hint_engine::ViolationType;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EducationNote {
    pub title: String,
    pub content: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EducationalContent {
    pub notes: Vec<EducationNote>,
}

impl EducationalContent {
    pub fn for_violation(violation: ViolationType) -> Self {
        match violation {
            ViolationType::AllocGlobal => Self {
                notes: alloc_global_notes(),
            },
            ViolationType::DeterminismRng => Self {
                notes: determinism_rng_notes(),
            },
        }
    }
}

fn alloc_global_notes() -> Vec<EducationNote> {
    vec![
        EducationNote {
            title: "Allocation locality".to_string(),
            content: vec![
                "Locality improves cache behavior and predictability.".to_string(),
                "Global allocation hides ownership and lifetimes.".to_string(),
            ],
        },
        EducationNote {
            title: "Anti-patterns".to_string(),
            content: vec![
                "Using a global arena for unrelated subsystems.".to_string(),
                "Relying on implicit global allocators in hot paths.".to_string(),
            ],
        },
        EducationNote {
            title: "Why the constitution forbids this".to_string(),
            content: vec![
                "Unbounded allocation breaks deterministic resource governance.".to_string(),
            ],
        },
    ]
}

fn determinism_rng_notes() -> Vec<EducationNote> {
    vec![
        EducationNote {
            title: "Determinism and seeds".to_string(),
            content: vec![
                "Seeded randomness enables reproducible behavior.".to_string(),
                "Explicit seed policies are audit-friendly.".to_string(),
            ],
        },
        EducationNote {
            title: "Anti-patterns".to_string(),
            content: vec![
                "Seeding RNGs from wall clock time.".to_string(),
                "Hidden RNG usage through globals.".to_string(),
            ],
        },
        EducationNote {
            title: "Why the constitution forbids this".to_string(),
            content: vec![
                "Nondeterminism erodes auditability and test reliability.".to_string(),
            ],
        },
    ]
}
