use ayken_runtime::{
    commit::commit, error::RuntimeResult, executors::execute, loader::load_demo_program,
    runtime::RuntimeState, types,
};

fn main() {
    println!("=== Ayken Runtime v0.2 ===");
    println!("Based on:");
    println!("  - ABDF v0.1 Final Spec");
    println!("  - BCIB Opcode Set v0.1");
    println!("  - BCIB Execution Semantics v0.1");
    println!();

    let program = load_demo_program().unwrap();
    println!("Loaded {} instructions", program.len());

    let mut state = RuntimeState::new();

    match run_program(&program, &mut state) {
        Ok(_) => {
            println!("✅ Ayken Runtime completed successfully");
            println!();
            println!("Final State:");
            println!("  contexts: {}", state.contexts.len());
            println!("  result_buffers: {}", state.result_buffers.len());
            state.print_trace();
        }
        Err(e) => {
            println!("❌ Runtime failed: {:?}", e);
        }
    }
}

/// Main execution loop: fetch → validate → execute → commit
fn run_program(
    instructions: &[types::BcibInstruction],
    state: &mut RuntimeState,
) -> RuntimeResult<()> {
    while state.running && state.pc < instructions.len() {
        let inst = instructions[state.pc];

        // Validate
        if let Err(e) = validate(&inst, state) {
            return Err(state.fail_closed(e));
        }

        // Execute (produces PendingOp)
        let pending = execute(inst, state)?;

        // Commit (applies PendingOp atomically)
        commit(pending, state)?;

        // Add trace
        state.add_trace(&inst);

        state.pc += 1;
    }

    Ok(())
}

fn validate(inst: &types::BcibInstruction, state: &RuntimeState) -> RuntimeResult<()> {
    use types::ContextStatus;

    let ctx = state.current_context()?;

    if ctx.status == ContextStatus::Failed {
        return Err(ayken_runtime::error::RuntimeError::ContextError);
    }

    if inst.arg_count > 64 {
        return Err(ayken_runtime::error::RuntimeError::ValidationError);
    }

    Ok(())
}
