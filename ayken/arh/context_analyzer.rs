// Constitutional Module: ContextAnalyzer
// This module MUST NOT mutate code, generate patches, or apply edits.
// All outputs are advisory-only.
// Forbidden behaviors: file writes, patch emission, workspace edits, auto-apply.

//! Context analysis (scope, dependencies, boundaries; advisory-only).

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ViolationScope {
    Local,
    Module,
    CrossModule,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UsagePattern {
    HotPath,
    InitPath,
    TestOnly,
    Unknown,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BoundaryFlag {
    UserspaceToKernel,
    KernelToUserspace,
    SharedAbiOrFfi,
    ModuleBoundaryViolation,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ContextSummary {
    pub scope: ViolationScope,
    pub usage_pattern: UsagePattern,
    pub boundary_flags: Vec<BoundaryFlag>,
    pub dependency_graph_depth: u32,
}

pub struct ContextAnalyzer;

impl ContextAnalyzer {
    /// Analyze violation context deterministically.
    pub fn analyze(&self, scope_depth: u32, usage: UsagePattern, boundary_flags: Vec<BoundaryFlag>) -> ContextSummary {
        let scope = if scope_depth == 0 {
            ViolationScope::Local
        } else if scope_depth == 1 {
            ViolationScope::Module
        } else {
            ViolationScope::CrossModule
        };

        ContextSummary {
            scope,
            usage_pattern: usage,
            boundary_flags,
            dependency_graph_depth: scope_depth,
        }
    }

    /// Advisory-only signal: boundary flags imply automation lock.
    pub fn automation_lock_required(&self, summary: &ContextSummary) -> bool {
        summary.boundary_flags.iter().any(is_boundary_lock)
    }
}

fn is_boundary_lock(flag: &BoundaryFlag) -> bool {
    matches!(
        flag,
        BoundaryFlag::UserspaceToKernel
            | BoundaryFlag::KernelToUserspace
            | BoundaryFlag::SharedAbiOrFfi
            | BoundaryFlag::ModuleBoundaryViolation
    )
}
