// Constitutional Module: CLI Fix Command
// This module MUST NOT silently mutate code.
// All outputs are advisory-only unless explicitly allowed by mode.
// Forbidden behaviors: auto-apply without approval, kernel autofix.

pub mod fix_command;
pub mod fix_modes;
pub mod fix_application;
pub mod fix_reporting;
pub mod system_status;
pub mod adn_commands;

pub use fix_command::*;
pub use fix_modes::*;
pub use fix_application::*;
pub use fix_reporting::*;
pub use system_status::*;
pub use adn_commands::*;
