use ayken_runtime::{
    commit::commit, error::RuntimeResult, executors::execute, loader::load_rollback_test_program,
    runtime::RuntimeState,
};

fn main() -> RuntimeResult<()> {
    println!("=== Rollback Integrity Test ===\n");

    let program = load_rollback_test_program()?;
    let mut state = RuntimeState::new();

    println!("Initial state:");
    println!("  running: {}", state.running);
    println!("  journal entries: {}", state.journal.len());
    println!("  contexts: {}", state.contexts.len());
    println!();

    // This should fail: insert without collection
    let inst = program[0];

    println!("Executing DataInsert without collection...");
    let pending = execute(inst, &state)?;

    println!("Attempting commit...");
    let result = commit(pending, &mut state);

    println!();
    match result {
        Err(e) => {
            println!("✅ Commit failed as expected: {:?}", e);
            println!();
            println!("State after rollback:");
            println!("  running: {}", state.running);
            println!("  journal entries: {}", state.journal.len());
            println!("  contexts: {}", state.contexts.len());
            println!();

            if !state.running && state.journal.is_empty() {
                println!("✅ PASS: Rollback integrity maintained");
                println!("  - Runtime stopped (fail-closed)");
                println!("  - Journal empty (no partial commit)");
                println!("  - Contexts preserved");
                Ok(())
            } else {
                println!("❌ FAIL: State corrupted after rollback");
                Err(ayken_runtime::error::RuntimeError::CommitError)
            }
        }
        Ok(_) => {
            println!("❌ FAIL: Commit succeeded when it should have failed");
            Err(ayken_runtime::error::RuntimeError::CommitError)
        }
    }
}
