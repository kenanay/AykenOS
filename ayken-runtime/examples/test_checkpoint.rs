use ayken_runtime::{
    checkpoint::replay_from_checkpoint,
    commit::commit,
    executors::execute,
    loader::load_demo_program,
    runtime::{hash_hex, RuntimeState},
    types::Opcode,
};

fn main() {
    println!("=== Checkpoint + Compaction Test ===\n");

    let program = load_demo_program().unwrap();
    let mut state = RuntimeState::new();

    println!("Phase 1: Execute program");
    for inst in &program {
        if !state.running {
            break;
        }

        let pending = execute(*inst, &state).unwrap();
        commit(pending, &mut state).unwrap();
        state.add_trace(inst);
        state.pc += 1;
    }

    println!("  journal entries: {}", state.journal.len());
    println!("  contexts: {}", state.contexts.len());
    println!("  result_buffers: {}", state.result_buffers.len());

    let before_checkpoint_hash = state.canonical_state_hash_v2();
    println!("  state hash: {}", hash_hex(&before_checkpoint_hash));
    println!();

    println!("Phase 2: Create checkpoint");
    let checkpoint = state.checkpoint();
    println!("  checkpoint_id: {}", checkpoint.checkpoint_id);
    println!("  checkpoint pc: {}", checkpoint.pc);
    println!("  checkpoint hash: {}", hash_hex(&checkpoint.state_hash));
    println!("  journal after checkpoint: {}", state.journal.len());
    println!();

    println!("Phase 3: Replay from checkpoint (empty tail)");
    let replayed = replay_from_checkpoint(&checkpoint, &[]).unwrap();
    let replay_hash = replayed.canonical_state_hash_v2();
    println!("  replayed hash: {}", hash_hex(&replay_hash));
    println!();

    if checkpoint.state_hash == replay_hash {
        println!("✅ PASS: Checkpoint replay with empty tail");
    } else {
        println!("❌ FAIL: Checkpoint replay mismatch");
        println!("  checkpoint: {}", hash_hex(&checkpoint.state_hash));
        println!("  replayed:   {}", hash_hex(&replay_hash));
        std::process::exit(1);
    }

    println!();
    println!("Phase 4: Execute new operation after checkpoint");
    let new_inst = ayken_runtime::types::BcibInstruction {
        opcode: Opcode::DataInsert,
        flags: 0,
        arg_start: 0,
        arg_count: 0,
    };

    let pending = execute(new_inst, &state).unwrap();
    commit(pending, &mut state).unwrap();
    state.add_trace(&new_inst);
    state.pc += 1;

    println!("  journal after new op: {}", state.journal.len());
    let after_new_op_hash = state.canonical_state_hash_v2();
    println!("  state hash: {}", hash_hex(&after_new_op_hash));
    println!();

    println!("Phase 5: Replay from checkpoint + tail journal");
    let replayed_with_tail = replay_from_checkpoint(&checkpoint, &state.journal).unwrap();
    let replay_with_tail_hash = replayed_with_tail.canonical_state_hash_v2();
    println!("  replayed hash: {}", hash_hex(&replay_with_tail_hash));
    println!();

    if after_new_op_hash == replay_with_tail_hash {
        println!("✅ PASS: Checkpoint + tail journal replay");
        println!("  - Checkpoint compaction works");
        println!("  - Incremental replay works");
        println!("  - Long-running durability foundation ready");
    } else {
        println!("❌ FAIL: Checkpoint + tail replay mismatch");
        println!("  current:  {}", hash_hex(&after_new_op_hash));
        println!("  replayed: {}", hash_hex(&replay_with_tail_hash));
        std::process::exit(1);
    }
}
