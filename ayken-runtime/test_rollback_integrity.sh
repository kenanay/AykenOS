#!/usr/bin/env bash
set -euo pipefail

echo "=== Rollback Integrity Test ==="
echo

# Create test program
cat > /tmp/test_rollback.rs << 'EOF'
use ayken_runtime::{
    commit::commit, error::RuntimeResult, executors::execute, loader::load_rollback_test_program,
    runtime::RuntimeState, types,
};

fn main() {
    println!("Testing rollback integrity...");
    
    let program = load_rollback_test_program().unwrap();
    let mut state = RuntimeState::new();
    
    // This should fail: insert without collection
    let inst = program[0];
    
    let pending = execute(inst, &state).unwrap();
    let result = commit(pending, &mut state);
    
    match result {
        Err(_) => {
            println!("✅ Commit failed as expected");
            println!("  running: {}", state.running);
            println!("  journal entries: {}", state.journal.len());
            println!("  contexts: {}", state.contexts.len());
            
            if !state.running && state.journal.is_empty() {
                println!("\n✅ PASS: Rollback integrity maintained");
                std::process::exit(0);
            } else {
                println!("\n❌ FAIL: State corrupted after rollback");
                std::process::exit(1);
            }
        }
        Ok(_) => {
            println!("❌ Commit succeeded when it should have failed");
            std::process::exit(1);
        }
    }
}
EOF

# Compile and run
cd ayken-runtime
rustc --edition 2021 \
    --extern ayken_runtime=target/debug/libayken_runtime.rlib \
    -L target/debug/deps \
    /tmp/test_rollback.rs \
    -o /tmp/test_rollback 2>&1 | grep -v "warning:" || true

/tmp/test_rollback
