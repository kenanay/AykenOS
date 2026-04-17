use crate::core::error::AykenError;
use std::process::{Command, Stdio};

#[derive(Clone, Copy)]
pub enum OutputMode {
    Inherit,
    Quiet,
}

pub struct CommandStatus {
    pub success: bool,
    pub exit_code: i32,
}

pub fn run_command(
    program: &str,
    args: &[&str],
    cwd: Option<&str>,
    envs: &[(&str, &str)],
) -> Result<(), AykenError> {
    let status = run_command_status(program, args, cwd, envs, OutputMode::Inherit)?;
    if status.success {
        Ok(())
    } else {
        Err(AykenError::Process(format!(
            "`{} {}` exited with status {}",
            program,
            args.join(" "),
            status.exit_code
        )))
    }
}

pub fn run_command_status(
    program: &str,
    args: &[&str],
    cwd: Option<&str>,
    envs: &[(&str, &str)],
    output_mode: OutputMode,
) -> Result<CommandStatus, AykenError> {
    let mut cmd = Command::new(program);
    cmd.args(args).stdin(Stdio::inherit());

    match output_mode {
        OutputMode::Inherit => {
            cmd.stdout(Stdio::inherit()).stderr(Stdio::inherit());
        }
        OutputMode::Quiet => {
            cmd.stdout(Stdio::null()).stderr(Stdio::null());
        }
    }

    if let Some(dir) = cwd {
        cmd.current_dir(dir);
    }

    for (k, v) in envs {
        cmd.env(k, v);
    }

    let status = cmd.status()?;
    Ok(CommandStatus {
        success: status.success(),
        exit_code: status.code().unwrap_or(-1),
    })
}

pub fn run_command_owned(
    program: &str,
    args: &[String],
    cwd: Option<&str>,
    envs: &[(&str, &str)],
) -> Result<(), AykenError> {
    let arg_refs: Vec<&str> = args.iter().map(String::as_str).collect();
    run_command(program, &arg_refs, cwd, envs)
}
