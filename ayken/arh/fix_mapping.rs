// Constitutional Module: FixMapping
// This module MUST NOT mutate code, generate patches, or apply edits.
// All outputs are advisory-only.
// Forbidden behaviors: file writes, patch emission, workspace edits, auto-apply.

//! Canonical mapping from rule violations to eligible hint types.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HintType {
    AssistedFix,
    DesignHint,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FixMappingResult {
    pub allowed: Vec<HintType>,
    pub forbidden: Vec<HintType>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FixMapping;

impl FixMapping {
    pub fn default() -> Self {
        Self {}
    }

    /// Determine canonical hint mapping for a given rule id.
    pub fn for_violation(&self, rule_id: &str, is_kernel: bool) -> FixMappingResult {
        let mut allowed = Vec::new();
        let mut forbidden = Vec::new();

        if is_kernel {
            allowed.push(HintType::DesignHint);
            forbidden.push(HintType::AssistedFix);
            return FixMappingResult { allowed, forbidden };
        }

        if rule_id == "ALLOC.GLOBAL" {
            allowed.push(HintType::DesignHint);
            allowed.push(HintType::AssistedFix);
        } else if rule_id == "DETERMINISM.RNG" {
            allowed.push(HintType::DesignHint);
            forbidden.push(HintType::AssistedFix);
        } else {
            allowed.push(HintType::DesignHint);
        }

        FixMappingResult { allowed, forbidden }
    }
}
