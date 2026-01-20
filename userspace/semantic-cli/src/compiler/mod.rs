//! Command Compiler and Security Validation
//!
//! This module provides the command compilation pipeline that converts execution plans
//! into validated, secure commands ready for execution. It implements strict security
//! boundaries and policy enforcement as required by Phase 3.3.

pub mod validation;
pub mod security;
pub mod compiler;
pub mod policy;

pub use compiler::{CommandCompiler, CompilerConfig};
pub use validation::*;
pub use security::*;
pub use policy::*;