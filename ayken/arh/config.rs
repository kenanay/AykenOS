// Constitutional Module: ARH Config
// Configuration may only tighten behavior, never relax it.
// Any attempt to weaken safety must fail validation.

use crate::arh::fix_preferences::{FixPreferences, PatternPriorities, SafetyConstraints};
use std::collections::HashSet;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConfidenceThresholds {
    pub safe_autofix_min: u8,
    pub assisted_fix_min: u8,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProjectProfile {
    Default,
    KernelSensitive,
    PerformanceCritical,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ArhConfig {
    pub confidence_thresholds: ConfidenceThresholds,
    pub fix_preferences: FixPreferences,
    pub pattern_priorities: PatternPriorities,
    pub safety_constraints: SafetyConstraints,
    pub project_profile: ProjectProfile,
}

impl ArhConfig {
    pub fn defaults() -> Self {
        Self {
            confidence_thresholds: ConfidenceThresholds {
                safe_autofix_min: 95,
                assisted_fix_min: 85,
            },
            fix_preferences: FixPreferences::default(),
            pattern_priorities: PatternPriorities::default(),
            safety_constraints: SafetyConstraints::hard_locked(),
            project_profile: ProjectProfile::Default,
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.confidence_thresholds.safe_autofix_min < 95 {
            return Err("safe_autofix_min cannot be lower than 95".to_string());
        }
        if self.confidence_thresholds.assisted_fix_min < 85 {
            return Err("assisted_fix_min cannot be lower than 85".to_string());
        }
        if self.safety_constraints.allow_kernel_fixes {
            return Err("allow_kernel_fixes must be false".to_string());
        }
        if self.safety_constraints.allow_cross_module {
            return Err("allow_cross_module must be false".to_string());
        }
        if self.safety_constraints.allow_design_hint_enforcement {
            return Err("allow_design_hint_enforcement must be false".to_string());
        }
        validate_disabled_lists(&self.fix_preferences)?;
        self.pattern_priorities.validate()?;
        Ok(())
    }

    /// Apply tightening-only thresholds to a candidate config.
    pub fn tighten(base: &ArhConfig, override_cfg: &ArhConfig) -> ArhConfig {
        let safe_autofix_min = base
            .confidence_thresholds
            .safe_autofix_min
            .max(override_cfg.confidence_thresholds.safe_autofix_min);
        let assisted_fix_min = base
            .confidence_thresholds
            .assisted_fix_min
            .max(override_cfg.confidence_thresholds.assisted_fix_min);

        let fix_preferences = merge_preferences(&base.fix_preferences, &override_cfg.fix_preferences);
        let pattern_priorities = merge_pattern_priorities(&base.pattern_priorities, &override_cfg.pattern_priorities);
        let project_profile = tighten_profile(&base.project_profile, &override_cfg.project_profile);

        ArhConfig {
            confidence_thresholds: ConfidenceThresholds {
                safe_autofix_min,
                assisted_fix_min,
            },
            fix_preferences,
            pattern_priorities,
            safety_constraints: base.safety_constraints.clone(),
            project_profile,
        }
    }
}

fn merge_preferences(base: &FixPreferences, override_cfg: &FixPreferences) -> FixPreferences {
    let mut disabled_rules: HashSet<String> = base.disabled_rules.iter().cloned().collect();
    for rule in &override_cfg.disabled_rules {
        disabled_rules.insert(rule.clone());
    }

    let mut disabled_patterns: HashSet<String> = base.disabled_patterns.iter().cloned().collect();
    for pattern in &override_cfg.disabled_patterns {
        disabled_patterns.insert(pattern.clone());
    }

    FixPreferences {
        prefer_safe_over_assisted: base.prefer_safe_over_assisted || override_cfg.prefer_safe_over_assisted,
        disabled_rules: disabled_rules.into_iter().collect(),
        disabled_patterns: disabled_patterns.into_iter().collect(),
    }
}

fn merge_pattern_priorities(base: &PatternPriorities, override_cfg: &PatternPriorities) -> PatternPriorities {
    // Fail-closed policy: duplicate pattern_id entries are rejected in validation.
    let mut overrides = base.overrides.clone();
    overrides.extend(override_cfg.overrides.clone());
    PatternPriorities { overrides }
}

fn profile_rank(profile: &ProjectProfile) -> u8 {
    match profile {
        ProjectProfile::Default => 0,
        ProjectProfile::PerformanceCritical => 1,
        ProjectProfile::KernelSensitive => 2,
    }
}

fn tighten_profile(base: &ProjectProfile, override_cfg: &ProjectProfile) -> ProjectProfile {
    if profile_rank(override_cfg) > profile_rank(base) {
        override_cfg.clone()
    } else {
        base.clone()
    }
}

fn validate_disabled_lists(preferences: &FixPreferences) -> Result<(), String> {
    let max_items = 256usize;
    let max_len = 128usize;

    if preferences.disabled_rules.len() > max_items || preferences.disabled_patterns.len() > max_items {
        return Err("disabled_rules/disabled_patterns exceeds maximum length".to_string());
    }

    let mut seen: HashSet<String> = HashSet::new();
    for rule in &preferences.disabled_rules {
        if rule.trim().is_empty() {
            return Err("disabled_rules contains empty entry".to_string());
        }
        if rule.len() > max_len {
            return Err("disabled_rules entry too long".to_string());
        }
        if !seen.insert(rule.clone()) {
            return Err("disabled_rules contains duplicates".to_string());
        }
    }

    let mut seen_patterns: HashSet<String> = HashSet::new();
    for pattern in &preferences.disabled_patterns {
        if pattern.trim().is_empty() {
            return Err("disabled_patterns contains empty entry".to_string());
        }
        if pattern.len() > max_len {
            return Err("disabled_patterns entry too long".to_string());
        }
        if !seen_patterns.insert(pattern.clone()) {
            return Err("disabled_patterns contains duplicates".to_string());
        }
    }

    Ok(())
}
