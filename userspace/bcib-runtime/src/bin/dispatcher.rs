use bcib::{BcibBuffer, BcibInstruction};
use bcib_runtime::{BcibExecutor, BcibGraph};
use dsl_parser::{DslParser, ParseError};

fn build_bcib_from_dsl(commands: &[&str]) -> Result<Vec<u8>, ParseError> {
    let mut parser = DslParser::new();
    let mut buf = BcibBuffer::new();

    for cmd in commands {
        let req = parser.parse_command(cmd)?;
        match req.command {
            dsl_parser::Command::SelectContext { .. } => {
                // Context selection maps to a NOP for now; runtime would track ctx separately.
                buf.add(BcibInstruction::nop());
            }
            dsl_parser::Command::Create { .. } => {
                buf.add(BcibInstruction::data_create(1, 1)); // placeholder indices
            }
            dsl_parser::Command::Add { .. } => {
                buf.add(BcibInstruction::data_add(1, 2));
            }
            dsl_parser::Command::Query { .. } => {
                buf.add(BcibInstruction::data_query(1, 3));
            }
            dsl_parser::Command::Render => {
                buf.add(BcibInstruction::ui_render(4));
            }
            dsl_parser::Command::AiAsk { .. } => {
                buf.add(BcibInstruction::ai_ask(5));
            }
            dsl_parser::Command::Info => {
                buf.add(BcibInstruction::nop());
            }
            dsl_parser::Command::Batch(items) => {
                // Each batch item is parsed individually; map to nop for now.
                for _ in items {
                    buf.add(BcibInstruction::nop());
                }
            }
        }
    }

    buf.add(BcibInstruction::end());
    Ok(buf.encode())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Example DSL script to drive BCIB encoding
    let cmds = ["> data.users", ">> create schema=id:int,name:string", ">> add {\"id\":1}", ">> query filter=\"id>0\""];

    let bcib_bytes = build_bcib_from_dsl(&cmds)?;
    println!("Built BCIB buffer ({} bytes)", bcib_bytes.len());

    // Only submit to kernel when explicitly requested (avoids int 0x80 on host dev boxes)
    let should_submit = std::env::var("RUN_IN_QEMU").map(|v| v == "1").unwrap_or(false);
    if !should_submit {
        println!("RUN_IN_QEMU!=1; skipping syscall submission. Bytes ready for submit_execution.");
        return Ok(());
    }

    let mut exec = BcibExecutor::new();
    let graph = BcibGraph::new(&bcib_bytes);
    let exec_id = exec.submit_execution(&graph)?;
    println!("Submitted execution_id={}", exec_id);

    let status = exec.wait_result(exec_id, 0)?;
    println!("wait_result returned status {}", status);
    Ok(())
}
