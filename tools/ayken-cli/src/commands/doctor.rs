use crate::cli::DoctorArgs;
use crate::core::{env, error::AykenError, output, policy};
use serde::Serialize;
use std::env as std_env;

#[derive(Serialize)]
struct DoctorReport {
    effective_cc: String,
    ci: bool,
    experimental: bool,
    env: env::EnvSnapshot,
    status: &'static str,
}

pub fn run(args: DoctorArgs, json: bool) -> Result<(), AykenError> {
    let snapshot = env::snapshot();
    let ci = std_env::var("CI").ok().as_deref() == Some("true");
    policy::ci_forbid_experimental(ci, args.experimental)?;
    let effective_cc = policy::enforce_toolchain_policy(&snapshot, args.experimental)?;

    let report = DoctorReport {
        effective_cc,
        ci,
        experimental: args.experimental,
        env: snapshot,
        status: "ok",
    };

    if json {
        output::print_json(&report)
    } else {
        println!("ayken doctor");
        println!("  status         : {}", report.status);
        println!("  effective_cc   : {}", report.effective_cc);
        println!("  ci             : {}", report.ci);
        println!("  experimental   : {}", report.experimental);
        println!("  env.CC         : {:?}", report.env.cc);
        println!("  rust_toolchain : {:?}", report.env.rustup_toolchain);
        println!("  RUSTFLAGS      : {:?}", report.env.rustflags);
        println!("  CARGO_TARGET_DIR: {:?}", report.env.cargo_target_dir);
        println!("  PATH has ayken : {}", report.env.path_contains_ayken);
        Ok(())
    }
}
