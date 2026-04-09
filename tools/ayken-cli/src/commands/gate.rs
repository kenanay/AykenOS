use crate::cli::GateArgs;
use crate::core::{env, error::AykenError, policy, process};
use std::env as std_env;

pub fn run(args: GateArgs, _json: bool) -> Result<(), AykenError> {
    let snapshot = env::snapshot();
    let ci = std_env::var("CI").ok().as_deref() == Some("true");
    policy::ci_forbid_experimental(ci, args.experimental)?;
    let effective_cc = policy::enforce_toolchain_policy(&snapshot, args.experimental)?;
    let cc_env: &[(&str, &str)] = &[("CC", effective_cc.as_str())];

    match args.target.as_str() {
        "hygiene" => process::run_command("make", &["ci-gate-hygiene"], None, cc_env),
        "all" => {
            process::run_command("make", &["ci-gate-hygiene"], None, cc_env)?;
            process::run_command("make", &["ci-gate-boundary"], None, cc_env)?;
            process::run_command("make", &["ci-gate-constitutional"], None, cc_env)
        }
        other => Err(AykenError::Policy(format!(
            "unsupported gate target: `{other}`. Valid: hygiene | all"
        ))),
    }
}
