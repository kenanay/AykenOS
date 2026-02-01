// Constitutional Module: PatternMatcher
// This module MUST NOT mutate code, generate patches, or apply edits.
// All outputs are advisory-only.
// Forbidden behaviors: file writes, patch emission, workspace edits, auto-apply.

//! Pattern matcher (structure-aware, deterministic, advisory-only).

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PatternToken {
    RuleTag(String),
    AstNode(String),
    Identifier(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PatternMatch {
    pub pattern_id: String,
    pub description: String,
    pub complexity_cost: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PatternMatchResult {
    pub matches: Vec<PatternMatch>,
    pub total_complexity_cost: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PatternLibrary {
    pub version: String,
}

pub struct PatternMatcher;

impl PatternMatcher {
    /// Deterministically match known patterns based on a structured input token stream.
    /// NOTE: This is advisory-only and does not mutate code.
    pub fn match_patterns(&self, tokens: &[PatternToken], library: &PatternLibrary) -> PatternMatchResult {
        let mut matches = Vec::new();
        matches.extend(match_alloc_global(tokens, library));
        matches.extend(match_determinism_rng(tokens, library));

        let total_complexity_cost = matches.iter().map(|m| m.complexity_cost).sum();
        PatternMatchResult { matches, total_complexity_cost }
    }
}

fn match_alloc_global(tokens: &[PatternToken], _library: &PatternLibrary) -> Vec<PatternMatch> {
    let mut result = Vec::new();
    for token in tokens {
        if let PatternToken::RuleTag(tag) = token {
            if tag == "ALLOC.GLOBAL" {
                result.push(PatternMatch {
                    pattern_id: "PATTERN::ALLOC::GLOBAL".to_string(),
                    description: "Global allocation usage detected".to_string(),
                    complexity_cost: 2,
                });
            }
        }
    }
    result
}

fn match_determinism_rng(tokens: &[PatternToken], _library: &PatternLibrary) -> Vec<PatternMatch> {
    let mut result = Vec::new();
    for token in tokens {
        if let PatternToken::RuleTag(tag) = token {
            if tag == "DETERMINISM.RNG" {
                result.push(PatternMatch {
                    pattern_id: "PATTERN::DETERMINISM::RNG".to_string(),
                    description: "Non-deterministic RNG usage detected".to_string(),
                    complexity_cost: 2,
                });
            }
        }
    }
    result
}
