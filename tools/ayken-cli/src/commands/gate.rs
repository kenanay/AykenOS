use crate::cli::{GateArgs, GateTarget};
use crate::commands::{risk, status};
use crate::core::{
    env,
    error::AykenError,
    output, policy,
    process::{self, OutputMode},
};
use serde::Serialize;
use std::env as std_env;

const PHASE16_GATE_CHAIN: &[&str] = &[
    "ci-gate-boundary",
    "ci-gate-hygiene",
    "ci-gate-constitutional",
    "ci-gate-governance-policy",
    "ci-gate-phase15-workstreams",
];

#[derive(Serialize)]
struct GateRunStatus {
    gate: &'static str,
    result: &'static str,
    exit_code: i32,
}

#[derive(Serialize)]
struct GateRunSummary {
    target: &'static str,
    all_pass: bool,
    gates: Vec<GateRunStatus>,
    risk: risk::AdvisoryRiskSummary,
}

pub fn run(args: GateArgs, _json: bool) -> Result<(), AykenError> {
    let snapshot = env::snapshot();
    let ci = std_env::var("CI").ok().as_deref() == Some("true");
    policy::ci_forbid_experimental(ci, args.experimental)?;
    let effective_cc = policy::enforce_toolchain_policy(&snapshot, args.experimental)?;
    let cc_env: &[(&str, &str)] = &[("CC", effective_cc.as_str())];

    match args.target {
        GateTarget::Hygiene => run_gate_chain("hygiene", &["ci-gate-hygiene"], cc_env, _json),
        GateTarget::All => run_gate_chain("all", PHASE16_GATE_CHAIN, cc_env, _json),
    }
}

fn run_gate_chain(
    target: &'static str,
    gates: &[&'static str],
    cc_env: &[(&str, &str)],
    json: bool,
) -> Result<(), AykenError> {
    let output_mode = if json {
        OutputMode::Quiet
    } else {
        OutputMode::Inherit
    };
    let authority = status::gather_authority_status();
    let advisory_risk = risk::compute_risk(&authority);
    let mut summary = GateRunSummary {
        target,
        all_pass: true,
        gates: Vec::with_capacity(gates.len()),
        risk: advisory_risk,
    };

    for gate in gates {
        let status = process::run_command_status("make", &[*gate], None, cc_env, output_mode)?;
        let result = if status.success { "PASS" } else { "FAIL" };

        summary.gates.push(GateRunStatus {
            gate,
            result,
            exit_code: status.exit_code,
        });

        if !json {
            println!("ayken gate {gate}: {result} (exit {})", status.exit_code);
        }

        if !status.success {
            summary.all_pass = false;
            if json {
                output::print_json(&summary)?;
            }
            return Err(AykenError::Process(format!(
                "`make {gate}` exited with status {}",
                status.exit_code
            )));
        }
    }

    if json {
        output::print_json(&summary)?;
    } else {
        println!(
            "ayken gate risk: {} (authority={}, confidence={})",
            summary.risk.risk_level,
            summary.risk.authority,
            summary
                .risk
                .lineage_confidence
                .map(|value| format!("{value:?}"))
                .unwrap_or_else(|| "-".to_string())
        );
    }

    Ok(())
}
