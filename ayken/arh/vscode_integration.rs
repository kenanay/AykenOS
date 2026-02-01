// Constitutional Module: VSCodeARHIntegration
// This module MUST NOT mutate code, generate patches, or apply edits.
// All outputs are advisory-only and UI/UX oriented.
// Forbidden behaviors: file writes, patch emission, workspace edits, auto-apply.

//! VS Code integration entry point (advisory-only, non-enforcing).

use std::collections::HashMap;

use crate::arh::arh_engine::{ArhEngine, ArhOutput, LatencyProfile, ViolationInput};
use crate::arh::code_actions::{CodeActionBuilder, CodeActionList};
use crate::arh::context_analyzer::BoundaryFlag;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Diagnostic {
    pub file_path: String,
    pub rule_id: String,
    pub violation_id: String,
    pub is_kernel: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EditorContext {
    pub file_path: String,
    pub is_kernel: bool,
    pub boundary_flags: Vec<BoundaryFlag>,
    pub violation_input: ViolationInput,
}

pub struct VSCodeARHIntegration {
    engine: ArhEngine,
    cache: HashMap<String, ArhOutput>,
    debounce_ms: u64,
}

impl VSCodeARHIntegration {
    pub fn new() -> Self {
        Self {
            engine: ArhEngine::new(),
            cache: HashMap::new(),
            debounce_ms: 300,
        }
    }

    /// Receive diagnostic and schedule ARH generation (debounced).
    pub fn on_diagnostic(&mut self, diagnostic: Diagnostic, context: EditorContext) -> CodeActionList {
        let output = self.request_arh(context, diagnostic.is_kernel);
        let mut builder = CodeActionBuilder::new();
        builder.from_arh_output(&output);
        builder.build()
    }

    /// Request ARH output with VS Code latency profile and caching.
    pub fn request_arh(&mut self, context: EditorContext, is_kernel: bool) -> ArhOutput {
        let mut input = context.violation_input;
        input.is_kernel = is_kernel;
        input.latency_profile = LatencyProfile::VsCode;

        if let Some(cached) = self.cache.get(&context.file_path) {
            return cached.clone();
        }

        // NOTE: actual debounce scheduling is editor/runtime-specific.
        // Here we only model the parameter to document the constraint.
        let _debounce = self.debounce_ms;
        let output = self.engine.generate(input);
        self.cache.insert(context.file_path, output.clone());
        output
    }
}
