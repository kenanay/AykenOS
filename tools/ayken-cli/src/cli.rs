use clap::{Args, Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "ayken", version, about = "AykenOS controlled toolchain entrypoint")]
pub struct AykenCli {
    /// Output results as JSON
    #[arg(long, global = true)]
    pub json: bool,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Check environment, toolchain, and policy
    Doctor(DoctorArgs),
    /// Run cargo check with enforced toolchain policy
    Check(CheckArgs),
    /// Run cargo test with enforced toolchain policy
    Test(TestArgs),
    /// Run a CI gate
    Gate(GateArgs),
    /// Show closure readiness status
    Closure(ClosureArgs),
}

#[derive(Args, Debug)]
pub struct DoctorArgs {
    /// Allow experimental toolchain (CC=ayken). Forbidden in CI.
    #[arg(long)]
    pub experimental: bool,
}

#[derive(Args, Debug)]
pub struct CheckArgs {
    /// Allow experimental toolchain. Forbidden in CI.
    #[arg(long)]
    pub experimental: bool,

    /// Workspace directory to run cargo check in
    #[arg(long, default_value = "userspace")]
    pub workspace: String,
}

#[derive(Args, Debug)]
pub struct TestArgs {
    /// Allow experimental toolchain. Forbidden in CI.
    #[arg(long)]
    pub experimental: bool,

    /// Workspace directory to run cargo test in
    #[arg(long, default_value = "userspace")]
    pub workspace: String,
}

#[derive(Args, Debug)]
pub struct GateArgs {
    /// Gate to run: hygiene | all
    pub target: String,

    /// Allow experimental toolchain. Forbidden in CI.
    #[arg(long)]
    pub experimental: bool,
}

#[derive(Args, Debug)]
pub struct ClosureArgs {
    /// Closure sub-command: status
    pub target: String,
}
