// Constitutional Module: Confidence Adjustment
// Deterministic, rule-based adjustments only.

//! Confidence adjustment based on refactor outcomes (no learning).

use crate::arh::refactor_outcome::Effectiveness;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AdjustmentRule {
    pub positive_delta: i8,
    pub neutral_delta: i8,
    pub negative_delta: i8,
    pub min_confidence: u8,
    pub max_confidence: u8,
}

impl AdjustmentRule {
    pub fn default() -> Self {
        Self {
            positive_delta: 5,
            neutral_delta: 0,
            negative_delta: -10,
            min_confidence: 0,
            max_confidence: 100,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AdjustmentResult {
    pub adjusted_confidence: u8,
    pub delta: i8,
}

pub fn adjust_confidence(
    base_confidence: u8,
    outcome: Effectiveness,
    rules: AdjustmentRule,
) -> AdjustmentResult {
    let delta = match outcome {
        Effectiveness::Positive => rules.positive_delta,
        Effectiveness::Neutral => rules.neutral_delta,
        Effectiveness::Negative => rules.negative_delta,
    };
    let adjusted = clamp_confidence(base_confidence, delta, rules.min_confidence, rules.max_confidence);
    AdjustmentResult {
        adjusted_confidence: adjusted,
        delta,
    }
}

fn clamp_confidence(base: u8, delta: i8, min: u8, max: u8) -> u8 {
    let base_i16 = base as i16;
    let adjusted = base_i16 + delta as i16;
    if adjusted < min as i16 {
        min
    } else if adjusted > max as i16 {
        max
    } else {
        adjusted as u8
    }
}
