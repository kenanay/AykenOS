// Constitutional Module: Fix Modes
// This module MUST NOT mutate code.
// It defines mode semantics and validation only.

use crate::arh::confidence_calculator::AutomationEligibility;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FixMode {
    Safe,
    Preview,
    Report,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FixModeConfig {
    pub min_confidence: u8,
    pub max_risk: u8,
}

impl FixModeConfig {
    pub fn safe_defaults() -> Self {
        Self {
            min_confidence: 90,
            max_risk: 30,
        }
    }
}

pub fn validate_mode_flags(safe: bool, preview: bool, report: bool) -> Result<FixMode, String> {
    let count = safe as u8 + preview as u8 + report as u8;
    if count != 1 {
        return Err("Exactly one of --safe, --preview, or --report must be specified".to_string());
    }
    if safe {
        Ok(FixMode::Safe)
    } else if preview {
        Ok(FixMode::Preview)
    } else {
        Ok(FixMode::Report)
    }
}

pub fn automation_allowed(
    eligibility: AutomationEligibility,
    confidence: u8,
    risk: u8,
    config: &FixModeConfig,
) -> bool {
    eligibility == AutomationEligibility::Yes && confidence >= config.min_confidence && risk < config.max_risk
}
