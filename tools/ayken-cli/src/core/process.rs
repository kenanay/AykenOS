use crate::core::error::AykenError;
use std::process::{Command, Stdio};

pub fn run_command(
    program: &str,
    args: &[&str],
    cwd: Option<&str>,
    envs: &[(&str, &str)],
) -> Result<(), AykenError> {
    let mut cmd = Command::new(program);
    cmd.args(args)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());

    if let Some(dir) = cwd {
        cmd.current_dir(dir);
    }

    for (k, v) in envs {
        cmd.env(k, v);
    }

    let status = cmd.status()?;
    if status.success() {
        Ok(())
    } else {
        Err(AykenError::Process(format!(
            "`{} {}` exited with status {}",
            program,
            args.join(" "),
            status.code().unwrap_or(-1)
        )))
    }
}
