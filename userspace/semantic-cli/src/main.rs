//! Semantic CLI Main Entry Point
//!
//! This is the main executable for the Semantic CLI REPL.
//! Phase 3.5.1.a - Gate B implementation.

use semantic_cli::repl::MinimalREPL;
use semantic_cli::error::Result;

fn main() -> Result<()> {
    // Create and run the REPL
    let mut repl = MinimalREPL::new()?;
    repl.run()?;
    
    Ok(())
}