//! B-MODE Core Types
//!
//! This module defines types specific to B-MODE constitutional analysis.

use crate::types::*;
use serde::{Deserialize, Serialize};

/// Rule type for constitutional rules (B-MODE)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RuleType {
    JITAllocationImmutability,
    NativeCacheDeterministicDisable,
    AuthorityHierarchyEnforcement,
}

/// Enforcement level for constitutional rules (B-MODE)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EnforcementLevel {
    Component,
    System,
    Constitutional,
    Administrative,
}

/// Operation type for constitutional analysis (B-MODE)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OperationType {
    AllocationRewrite,
    CodeGeneration,
    CodeOptimization,
}

/// B-MODE specification report
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BModeSpecificationReport {
    pub component: ComponentId,
    pub analysis_timestamp: LogicalTimestamp,
    pub violations: Vec<crate::errors::SpecificationViolation>,
    pub findings: Vec<crate::errors::SpecificationFinding>,
    pub compliance_score: f64,
}

impl BModeSpecificationReport {
    pub fn new(component: ComponentId) -> Self {
        Self {
            component,
            analysis_timestamp: DeterministicClock::new().now(),
            violations: Vec::new(),
            findings: Vec::new(),
            compliance_score: 1.0,
        }
    }
}

/// JIT Operation for B-MODE analysis
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum JITOperation {
    AllocationRewrite {
        original: AllocationDecision,
        proposed: AllocationDecision,
    },
    CodeGeneration {
        register_accesses: Vec<RegisterAccess>,
        bounds_checking_enabled: bool,
    },
    CodeOptimization {
        optimization_type: String,
        affected_registers: Vec<PhysicalRegisterId>,
    },
}