use clap::{Args, Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(
    name = "ayken",
    version,
    about = "AykenOS controlled toolchain entrypoint"
)]
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
    /// Show combined closure and verified-head authority state
    Status(StatusArgs),
    /// Show advisory risk interpretation without changing authority
    Risk(RiskArgs),
    /// Run a CI gate
    Gate(GateArgs),
    /// Observe or verify closure authority
    Closure(ClosureArgs),
    /// Verify CI-backed development head authority
    Head(HeadArgs),
    /// Thin BCIB orchestration over existing verifier/runtime surfaces
    Bcib(BcibArgs),
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
pub struct StatusArgs {}

#[derive(Args, Debug)]
pub struct RiskArgs {}

#[derive(Args, Debug)]
pub struct GateArgs {
    #[command(subcommand)]
    pub target: GateTarget,

    /// Allow experimental toolchain. Forbidden in CI.
    #[arg(long)]
    pub experimental: bool,
}

#[derive(Subcommand, Debug)]
pub enum GateTarget {
    /// Run the hygiene gate only
    Hygiene,
    /// Run the Phase-16 minimal gate chain
    All,
}

#[derive(Args, Debug)]
pub struct ClosureArgs {
    #[command(subcommand)]
    pub target: ClosureTarget,
}

#[derive(Subcommand, Debug)]
pub enum ClosureTarget {
    /// Advisory closure observation surface
    Status,
    /// Binding closure authority verification surface
    Verify,
}

#[derive(Args, Debug)]
pub struct HeadArgs {
    #[command(subcommand)]
    pub target: HeadTarget,
}

#[derive(Subcommand, Debug)]
pub enum HeadTarget {
    /// Binding verified-head authority verification surface
    Verify,
    /// Advisory nearest-verified-ancestor diagnostics surface
    Lineage,
}

#[derive(Args, Debug)]
pub struct BcibArgs {
    #[command(subcommand)]
    pub target: BcibTarget,
}

#[derive(Subcommand, Debug)]
pub enum BcibTarget {
    /// Verify a proof bundle via the existing proof-verifier binary
    Verify(BcibVerifyArgs),
    /// Compute a SHA-256 digest for a BCIB artifact
    Hash(BcibPathArgs),
    /// Inspect a BCIB artifact without claiming authority
    Inspect(BcibPathArgs),
}

#[derive(Args, Debug)]
pub struct BcibVerifyArgs {
    /// Path to the proof bundle root
    pub bundle_path: String,

    /// Path to the trust policy JSON
    #[arg(long)]
    pub policy: String,

    /// Path to the producer registry snapshot JSON
    #[arg(long)]
    pub registry: String,
}

#[derive(Args, Debug)]
pub struct BcibPathArgs {
    /// Path to the BCIB artifact
    pub path: String,
}
