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

    // Step 1b: Reject --json + --diff combination (ambiguous output mode)
    if flags.json_output && flags.diff_baseline.is_some() {
        eprintln!("error: --json and --diff cannot be used together");
        process::exit(1);
    }

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

    // Step 4: Save snapshot if requested (before threshold — save raw evidence regardless of policy)
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
    // Violations → stderr only, NO stdout output, exit 4.
    if let Err(e) = threshold::evaluate_all(&flags.fail_if, &snapshot) {
        eprintln!("{}", e);
        process::exit(e.exit_code());
    }

    // Step 6: Produce output ONLY if threshold passed
    if let Some(ref baseline_path) = flags.diff_baseline {
        // Diff mode: load baseline, compute diff, format
        let baseline_bytes = match fetcher::read_snapshot_file(baseline_path) {
            Ok(b) => b,
            Err(e) => {
                eprintln!("{}", e);
                process::exit(e.exit_code());
            }
        };
        let baseline = match parser::parse_snapshot(&baseline_bytes) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("{}", e);
                process::exit(e.exit_code());
            }
        };
        let d = diff::compute_diff(&baseline, &snapshot);
        print!("{}", diff::format_diff(&d));
    } else if flags.json_output {
        // JSON mode: canonical JSON to stdout
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
        // Default: human-readable formatted output
        let output = formatter::format_snapshot(&snapshot);
        print!("{}", output);
    }
}
