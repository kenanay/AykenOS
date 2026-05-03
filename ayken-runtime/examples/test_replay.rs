use ayken_runtime::{
    commit::commit,
    executors::execute,
    loader::load_demo_program,
    replay::replay_journal,
    runtime::{hash_hex, RuntimeState},
};

fn main() {
    println!("=== Journal Replay Test ===\n");

    let program = load_demo_program().unwrap();
    let mut state = RuntimeState::new();

    println!("Executing program...");
    for inst in program {
        if !state.running {
            break;
        }

        let pending = execute(inst, &state).unwrap();
        commit(pending, &mut state).unwrap();
        state.add_trace(&inst);
        state.pc += 1;
    }

    println!("  journal entries: {}", state.journal.len());
    println!("  contexts: {}", state.contexts.len());
    println!("  result_buffers: {}", state.result_buffers.len());
    println!();

    let original_hash = state.canonical_state_hash_v2();
    println!("Original state hash:");
    println!("  {}", hash_hex(&original_hash));
    println!();

    println!("Replaying journal...");
    let replayed = replay_journal(&state.journal).unwrap();
    let replay_hash = replayed.canonical_state_hash_v2();

    println!("Replayed state hash:");
    println!("  {}", hash_hex(&replay_hash));
    println!();

    if original_hash == replay_hash {
        println!("✅ PASS: Journal replay produced identical state");
        println!("  - Journal is source of truth");
        println!("  - State is deterministically derived");
        println!("  - BLAKE3 hash verified");
    } else {
        println!("❌ FAIL: Replay mismatch");
        println!("  original: {}", hash_hex(&original_hash));
        println!("  replayed: {}", hash_hex(&replay_hash));
        std::process::exit(1);
    }
}
