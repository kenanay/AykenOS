//! CLI mode management and switching logic

use crate::types::*;
use crate::error::*;
use tracing::{info, debug};

/// Mode manager for handling CLI mode transitions and state
pub struct ModeManager {
    /// Current active mode
    current_mode: CLIMode,
    /// Mode-specific configurations
    mode_configs: std::collections::HashMap<CLIMode, ModeConfig>,
}

/// Configuration for a specific CLI mode
#[derive(Debug, Clone)]
pub struct ModeConfig {
    /// Whether this mode requires special initialization
    pub requires_initialization: bool,
    /// Allowed transitions from this mode
    pub allowed_transitions: Vec<CLIMode>,
    /// Mode-specific settings
    pub settings: std::collections::HashMap<String, String>,
}

impl ModeManager {
    /// Create a new mode manager
    pub fn new() -> Self {
        let mut mode_configs = std::collections::HashMap::new();
        
        // Traditional mode configuration
        mode_configs.insert(CLIMode::Traditional, ModeConfig {
            requires_initialization: false,
            allowed_transitions: vec![CLIMode::Semantic, CLIMode::Developer],
            settings: std::collections::HashMap::new(),
        });
        
        // Semantic mode configuration
        mode_configs.insert(CLIMode::Semantic, ModeConfig {
            requires_initialization: true,
            allowed_transitions: vec![CLIMode::Traditional, CLIMode::Developer],
            settings: {
                let mut settings = std::collections::HashMap::new();
                settings.insert("semantic_trigger".to_string(), "?".to_string());
                settings.insert("confidence_threshold".to_string(), "0.7".to_string());
                settings
            },
        });
        
        // Developer mode configuration
        mode_configs.insert(CLIMode::Developer, ModeConfig {
            requires_initialization: true,
            allowed_transitions: vec![CLIMode::Traditional, CLIMode::Semantic],
            settings: {
                let mut settings = std::collections::HashMap::new();
                settings.insert("dry_run_default".to_string(), "true".to_string());
                settings.insert("verbose_logging".to_string(), "true".to_string());
                settings.insert("trace_pipeline".to_string(), "true".to_string());
                settings
            },
        });

        Self {
            current_mode: CLIMode::Traditional,
            mode_configs,
        }
    }

    /// Get the current CLI mode
    pub fn current_mode(&self) -> CLIMode {
        self.current_mode
    }

    /// Check if a mode transition is allowed
    pub fn can_transition_to(&self, target_mode: CLIMode) -> bool {
        if let Some(config) = self.mode_configs.get(&self.current_mode) {
            config.allowed_transitions.contains(&target_mode)
        } else {
            false
        }
    }

    /// Validate a mode transition
    pub fn validate_transition(&self, target_mode: CLIMode) -> Result<(), ModeError> {
        // Check if transition is allowed
        if !self.can_transition_to(target_mode) {
            return Err(ModeError::InvalidTransition {
                from: self.current_mode,
                to: target_mode,
            });
        }

        // Check if target mode requires initialization
        if let Some(config) = self.mode_configs.get(&target_mode) {
            if config.requires_initialization {
                // In a full implementation, this would check if required
                // components (AI models, etc.) are available
                debug!("Mode {:?} requires initialization", target_mode);
            }
        }

        Ok(())
    }

    /// Perform mode transition
    pub fn transition_to(&mut self, target_mode: CLIMode) -> Result<ModeTransition, ModeError> {
        self.validate_transition(target_mode)?;
        
        let previous_mode = self.current_mode;
        self.current_mode = target_mode;
        
        info!("Mode transition: {:?} -> {:?}", previous_mode, target_mode);
        
        Ok(ModeTransition {
            from: previous_mode,
            to: target_mode,
            timestamp: chrono::Utc::now(),
        })
    }

    /// Get configuration for a mode
    pub fn get_mode_config(&self, mode: CLIMode) -> Option<&ModeConfig> {
        self.mode_configs.get(&mode)
    }

    /// Get setting for current mode
    pub fn get_current_setting(&self, key: &str) -> Option<&String> {
        self.mode_configs
            .get(&self.current_mode)?
            .settings
            .get(key)
    }

    /// Update setting for current mode
    pub fn update_current_setting(&mut self, key: String, value: String) {
        if let Some(config) = self.mode_configs.get_mut(&self.current_mode) {
            config.settings.insert(key, value);
        }
    }
}

/// Semantic mode specific functionality
pub struct SemanticMode {
    /// Confidence threshold for accepting intents
    pub confidence_threshold: f32,
    /// Semantic trigger character
    pub semantic_trigger: char,
    /// Whether to show alternatives for low confidence
    pub show_alternatives: bool,
}

impl SemanticMode {
    /// Create new semantic mode with default settings
    pub fn new() -> Self {
        Self {
            confidence_threshold: 0.7,
            semantic_trigger: '?',
            show_alternatives: true,
        }
    }

    /// Check if input should be processed semantically
    pub fn should_process_semantically(&self, input: &str) -> bool {
        input.trim_start().starts_with(self.semantic_trigger) || 
        input.trim().is_empty() == false
    }

    /// Check if confidence is acceptable
    pub fn is_confidence_acceptable(&self, confidence: f32) -> bool {
        confidence >= self.confidence_threshold
    }

    /// Generate clarification request for low confidence
    pub fn generate_clarification(&self, intent: &Intent) -> ClarificationRequest {
        ClarificationRequest {
            original_input: intent.raw_input.clone(),
            confidence: intent.confidence,
            alternatives: intent.alternatives.clone(),
            suggested_rephrasing: self.suggest_rephrasing(&intent.raw_input),
        }
    }

    fn suggest_rephrasing(&self, input: &str) -> Vec<String> {
        // Simple rephrasing suggestions
        vec![
            format!("Could you mean: '{} --help'?", input),
            format!("Did you want to: 'show {}'?", input),
            format!("Perhaps: 'list {}'?", input),
        ]
    }
}

/// Developer mode specific functionality
pub struct DeveloperMode {
    /// Whether dry-run is enabled by default
    pub dry_run_enabled: bool,
    /// Tracing level for debugging
    pub tracing_level: TracingLevel,
    /// Debug flags
    pub debug_flags: Vec<String>,
    /// Whether to show internal pipeline steps
    pub show_pipeline_steps: bool,
}

impl DeveloperMode {
    /// Create new developer mode with default settings
    pub fn new() -> Self {
        Self {
            dry_run_enabled: true,
            tracing_level: TracingLevel::Debug,
            debug_flags: vec!["pipeline".to_string(), "parsing".to_string()],
            show_pipeline_steps: true,
        }
    }

    /// Check if a debug flag is enabled
    pub fn is_debug_flag_enabled(&self, flag: &str) -> bool {
        self.debug_flags.contains(&flag.to_string())
    }

    /// Enable a debug flag
    pub fn enable_debug_flag(&mut self, flag: String) {
        if !self.debug_flags.contains(&flag) {
            self.debug_flags.push(flag);
        }
    }

    /// Disable a debug flag
    pub fn disable_debug_flag(&mut self, flag: &str) {
        self.debug_flags.retain(|f| f != flag);
    }

    /// Generate debug information for an intent
    pub fn generate_debug_info(&self, intent: &Intent) -> DebugInfo {
        DebugInfo {
            intent_id: intent.id,
            parsing_details: ParsingDetails {
                confidence: intent.confidence,
                action_detected: intent.action.clone(),
                targets_found: intent.targets.len(),
                parameters_extracted: intent.parameters.len(),
            },
            pipeline_trace: Vec::new(), // Would be populated during execution
            performance_metrics: PerformanceMetrics::default(),
        }
    }
}

/// Clarification request for ambiguous input
#[derive(Debug, Clone)]
pub struct ClarificationRequest {
    /// Original user input
    pub original_input: String,
    /// Confidence score of the best interpretation
    pub confidence: f32,
    /// Alternative interpretations
    pub alternatives: Vec<Intent>,
    /// Suggested rephrasing options
    pub suggested_rephrasing: Vec<String>,
}

/// Debug information for developer mode
#[derive(Debug, Clone)]
pub struct DebugInfo {
    /// Intent ID being debugged
    pub intent_id: uuid::Uuid,
    /// Parsing details
    pub parsing_details: ParsingDetails,
    /// Pipeline execution trace
    pub pipeline_trace: Vec<PipelineStep>,
    /// Performance metrics
    pub performance_metrics: PerformanceMetrics,
}

/// Parsing details for debugging
#[derive(Debug, Clone)]
pub struct ParsingDetails {
    /// Confidence score
    pub confidence: f32,
    /// Detected action type
    pub action_detected: ActionType,
    /// Number of targets found
    pub targets_found: usize,
    /// Number of parameters extracted
    pub parameters_extracted: usize,
}

/// Pipeline step for tracing
#[derive(Debug, Clone)]
pub struct PipelineStep {
    /// Step name
    pub name: String,
    /// Duration of this step
    pub duration: std::time::Duration,
    /// Input to this step
    pub input: String,
    /// Output from this step
    pub output: String,
}

impl Default for ModeManager {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for SemanticMode {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for DeveloperMode {
    fn default() -> Self {
        Self::new()
    }
}