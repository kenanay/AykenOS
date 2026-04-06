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
    if args.len() < 2 {
        eprintln!("usage: obs-cli <snapshot.json>");
        process::exit(1);
    }
    let path = &args[1];
    let bytes = match std::fs::read(path) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("error reading {}: {}", path, e);
            process::exit(2);
        }
    };
    let snapshot = match parser::parse_snapshot(&bytes) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("{}", e);
            process::exit(e.exit_code());
        }
    };
    let output = formatter::format_snapshot(&snapshot);
    print!("{}", output);
}
