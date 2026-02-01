// Constitutional Module: ARH Fix Preferences
// Preferences may only influence ordering, never enforcement.

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FixPreferences {
    pub prefer_safe_over_assisted: bool,
    pub disabled_rules: Vec<String>,
    pub disabled_patterns: Vec<String>,
}

impl Default for FixPreferences {
    fn default() -> Self {
        Self {
            prefer_safe_over_assisted: true,
            disabled_rules: Vec::new(),
            disabled_patterns: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PatternPriorityOverride {
    pub pattern_id: String,
    pub priority_boost: i8,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PatternPriorities {
    pub overrides: Vec<PatternPriorityOverride>,
}

impl Default for PatternPriorities {
    fn default() -> Self {
        Self { overrides: Vec::new() }
    }
}

impl PatternPriorities {
    pub fn validate(&self) -> Result<(), String> {
        let mut seen = std::collections::HashSet::new();
        for override_entry in &self.overrides {
            if !seen.insert(override_entry.pattern_id.clone()) {
                return Err("duplicate pattern_id in overrides".to_string());
            }
            if override_entry.priority_boost < -5 || override_entry.priority_boost > 5 {
                return Err("priority_boost must be between -5 and +5".to_string());
            }
            if override_entry.priority_boost == 0 {
                return Err("priority_boost cannot be 0".to_string());
            }
            if !override_entry.pattern_id.starts_with("PATTERN::") {
                return Err("pattern_id must start with PATTERN::".to_string());
            }
            if override_entry.pattern_id.len() > 128 {
                return Err("pattern_id too long".to_string());
            }
            if override_entry.pattern_id.starts_with("PATTERN::DESIGN_HINT") && override_entry.priority_boost > 0 {
                return Err("Cannot raise DesignHint above AssistedFix".to_string());
            }
            if override_entry.pattern_id.starts_with("PATTERN::ARRE") {
                return Err("Cannot override ARRE priority".to_string());
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SafetyConstraints {
    pub allow_kernel_fixes: bool,
    pub allow_cross_module: bool,
    pub allow_design_hint_enforcement: bool,
}

impl Default for SafetyConstraints {
    fn default() -> Self {
        Self {
            allow_kernel_fixes: false,
            allow_cross_module: false,
            allow_design_hint_enforcement: false,
        }
    }
}

impl SafetyConstraints {
    pub fn hard_locked() -> Self {
        Self::default()
    }
}
