use bcib::{BcibBuffer, BcibInstruction};
use bcib_runtime::{BcibExecutor, BcibGraph};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("BCIB Execution Submission Demo");
    println!("==============================");

    // Create a simple BCIB graph
    let mut buf = BcibBuffer::new();
    buf.add(BcibInstruction::data_create(1, 1));
    buf.add(BcibInstruction::data_add(1, 2));
    buf.add(BcibInstruction::end());

    let bcib_bytes = buf.encode();
    println!("Created BCIB graph with {} bytes", bcib_bytes.len());

    // Create executor and graph
    let mut executor = BcibExecutor::new();
    let graph = BcibGraph::new(&bcib_bytes);

    // Validate the graph
    match graph.validate() {
        Ok(()) => println!("✓ BCIB graph validation passed"),
        Err(e) => {
            println!("✗ BCIB graph validation failed: {}", e);
            return Err(e.into());
        }
    }

    // Check if we should actually submit to kernel
    let should_submit = std::env::var("RUN_IN_QEMU")
        .map(|v| v == "1")
        .unwrap_or(false);

    if !should_submit {
        println!("RUN_IN_QEMU!=1; skipping actual syscall submission");
        println!("Graph is ready for submit_execution() call");
        return Ok(());
    }

    let target_context_id = std::env::var("AYKEN_TARGET_CONTEXT_ID")
        .map_err(|_| "AYKEN_TARGET_CONTEXT_ID is required when RUN_IN_QEMU=1")?
        .parse::<u64>()
        .map_err(|_| "AYKEN_TARGET_CONTEXT_ID must parse as u64")?;

    // Submit execution to Ring0
    println!("Submitting execution to Ring0...");
    match executor.submit_execution(&graph, target_context_id) {
        Ok(execution_id) => {
            println!(
                "✓ Execution submitted successfully with ID: {}",
                execution_id
            );

            // Wait for result
            match executor.wait_result(execution_id, 1000) {
                Ok(status) => println!("✓ Execution completed with status: {}", status),
                Err(e) => println!("✗ Wait result failed: {}", e),
            }
        }
        Err(e) => {
            println!("✗ Execution submission failed: {}", e);
            return Err(e.into());
        }
    }

    Ok(())
}
