mod cli;
mod diff;
mod error;
mod fetcher;
mod formatter;
mod models;
mod parser;
mod printer;
mod threshold;

use std::process;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    // Step 1: Parse flags — fail fast on usage errors
    let flags = match cli::Flags::parse(&args) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("{}", e);
            process::exit(e.exit_code());
        }
    };

    // Step 2: Fetch or read snapshot bytes
    let bytes = if let Some(ref path) = flags.snapshot_file {
        match fetcher::read_snapshot_file(path) {
            Ok(b) => b,
            Err(e) => {
                eprintln!("{}", e);
                process::exit(e.exit_code());
            }
        }
    } else {
        match fetcher::fetch_from_proofd(&flags.proofd_addr, flags.timeout_ms) {
            Ok(b) => b,
            Err(e) => {
                eprintln!("{}", e);
                process::exit(e.exit_code());
            }
        }
    };

    // Step 3: Parse and validate snapshot (contract enforcement)
    let snapshot = match parser::parse_snapshot(&bytes) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("{}", e);
            process::exit(e.exit_code());
        }
    };

    // Step 4: Save snapshot if requested (before threshold — save regardless of policy)
    if let Some(ref save_path) = flags.save_snapshot {
        match printer::to_canonical_json(&snapshot) {
            Ok(json_bytes) => {
                if let Err(e) = fetcher::write_snapshot_file(save_path, &json_bytes) {
                    eprintln!("{}", e);
                    process::exit(e.exit_code());
                }
            }
            Err(e) => {
                eprintln!("{}", e);
                process::exit(e.exit_code());
            }
        }
    }

    // Step 5: Threshold enforcement — hard gate, fail-fast
    // If any condition is violated: report to stderr, produce NO stdout output, exit 4.
    if let Err(e) = threshold::evaluate_all(&flags.fail_if, &snapshot) {
        eprintln!("{}", e);
        process::exit(e.exit_code());
    }

    // Step 6: Produce output ONLY if threshold passed (format or JSON)
    if flags.json_output {
        match printer::to_canonical_json(&snapshot) {
            Ok(json_bytes) => {
                use std::io::Write;
                if let Err(e) = std::io::stdout().write_all(&json_bytes) {
                    eprintln!("error writing output: {}", e);
                    process::exit(2);
                }
            }
            Err(e) => {
                eprintln!("{}", e);
                process::exit(e.exit_code());
            }
        }
    } else {
        let output = formatter::format_snapshot(&snapshot);
        print!("{}", output);
    }
}
