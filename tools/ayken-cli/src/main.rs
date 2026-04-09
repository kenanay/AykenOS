mod cli;
mod commands;
mod core;

use clap::Parser;
use cli::{AykenCli, Command};

fn main() {
    let cli = AykenCli::parse();

    let result = match cli.command {
        Command::Doctor(args) => commands::doctor::run(args, cli.json),
        Command::Check(args) => commands::check::run(args, cli.json),
        Command::Test(args) => commands::test::run(args, cli.json),
        Command::Gate(args) => commands::gate::run(args, cli.json),
        Command::Closure(args) => commands::closure::run(args, cli.json),
        Command::Bcib(args) => commands::bcib::run(args, cli.json),
    };

    if let Err(err) = result {
        eprintln!("error: {err}");
        std::process::exit(err.exit_code());
    }
}
