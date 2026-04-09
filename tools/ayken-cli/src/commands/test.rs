use crate::cli::TestArgs;
use crate::core::{env, error::AykenError, policy, process};
use std::env as std_env;

pub fn run(args: TestArgs, _json: bool) -> Result<(), AykenError> {
    let snapshot = env::snapshot();
    let ci = std_env::var("CI").ok().as_deref() == Some("true");
    policy::ci_forbid_experimental(ci, args.experimental)?;
    let effective_cc = policy::enforce_toolchain_policy(&snapshot, args.experimental)?;

    process::run_command(
        "cargo",
        &["test"],
        Some(&args.workspace),
        &[("CC", effective_cc.as_str())],
    )
}
