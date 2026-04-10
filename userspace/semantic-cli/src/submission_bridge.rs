//! Submission Bridge module
//!
//! This module bridges Semantic CLI to BCIB submission via orchestrator.
//! Semantic CLI submits execution plans, does not execute directly.
//!
//! **ARCHITECTURAL CORRECTION:** Renamed from "execution_bridge" to "submission_bridge"
//! to clarify that this module SUBMITS plans but does NOT execute them.

#![allow(dead_code)]

use crate::bcib_simple::BCIB;
use crate::canonical_query::CanonicalQueryBinding;
use crate::error::SemanticCLIError;
use serde::{Deserialize, Serialize};

/// Capability required for submission
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Capability {
    pub name: String,
    pub scope: String,
    pub resource: String,
    pub reason: String,
}

/// Submission input
#[derive(Debug, Clone)]
pub struct SubmissionInput {
    pub canonical_command: String,
    pub canonical_binding: CanonicalQueryBinding,
    pub bcib: BCIB,
    pub declared_capabilities: Vec<Capability>,
}

/// Submission result
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SubmissionResult {
    pub submission_id: String,
    pub status: String,
    pub result: Option<String>,
}

/// Submit adapter trait
pub trait SubmitAdapter {
    fn submit(&self, input: SubmissionInput) -> Result<SubmissionResult, SemanticCLIError>;
}

pub struct SubmissionBridge;
