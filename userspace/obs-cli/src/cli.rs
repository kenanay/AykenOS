use std::path::PathBuf;

use crate::error::AppError;
use crate::threshold::ThresholdCondition;

#[derive(Debug, Clone)]
pub struct Flags {
    pub proofd_addr: String,
    pub timeout_ms: u64,
    pub snapshot_file: Option<PathBuf>,
    pub save_snapshot: Option<PathBuf>,
    pub diff_baseline: Option<PathBuf>,
    pub json_output: bool,
    pub fail_if: Vec<ThresholdCondition>,
}

impl Default for Flags {
    fn default() -> Self {
        Flags {
            proofd_addr: "http://127.0.0.1:7777".to_string(),
            timeout_ms: 5000,
            snapshot_file: None,
            save_snapshot: None,
            diff_baseline: None,
            json_output: false,
            fail_if: Vec::new(),
        }
    }
}

impl Flags {
    /// Parse command-line arguments into a `Flags` struct.
    ///
    /// Accepts both `--flag value` and `--flag=value` forms.
    /// Returns `AppError::Usage` on any invalid input.
    pub fn parse(args: &[String]) -> Result<Flags, AppError> {
        let mut flags = Flags::default();

        // Track whether each exclusive flag was set
        let mut proofd_addr_set = false;
        let mut snapshot_file_set = false;

        let mut i = 0;
        // Skip argv[0] (program name) if present
        if !args.is_empty() {
            i = 1;
        }

        while i < args.len() {
            let arg = &args[i];

            // Split on '=' for --flag=value form
            let (flag, inline_value) = if let Some(eq_pos) = arg.find('=') {
                let (f, v) = arg.split_at(eq_pos);
                (f, Some(&v[1..]))
            } else {
                (arg.as_str(), None)
            };

            // Helper: get the value either from inline or next arg
            macro_rules! get_value {
                ($flag_name:expr) => {{
                    if let Some(v) = inline_value {
                        i += 1;
                        v.to_string()
                    } else if i + 1 < args.len() {
                        i += 1;
                        let v = args[i].clone();
                        i += 1;
                        v
                    } else {
                        return Err(AppError::Usage(format!(
                            "{} requires a value",
                            $flag_name
                        )));
                    }
                }};
            }

            match flag {
                "--proofd-addr" => {
                    let val = get_value!("--proofd-addr");
                    flags.proofd_addr = val;
                    proofd_addr_set = true;
                }
                "--timeout-ms" => {
                    let val = get_value!("--timeout-ms");
                    flags.timeout_ms = val.parse::<u64>().map_err(|_| {
                        AppError::Usage(format!("invalid --timeout-ms value: '{}'", val))
                    })?;
                }
                "--snapshot-file" => {
                    let val = get_value!("--snapshot-file");
                    flags.snapshot_file = Some(PathBuf::from(val));
                    snapshot_file_set = true;
                }
                "--save-snapshot" => {
                    let val = get_value!("--save-snapshot");
                    flags.save_snapshot = Some(PathBuf::from(val));
                }
                "--diff" => {
                    let val = get_value!("--diff");
                    flags.diff_baseline = Some(PathBuf::from(val));
                }
                "--json" => {
                    flags.json_output = true;
                    if inline_value.is_some() {
                        return Err(AppError::Usage(
                            "--json does not take a value".to_string(),
                        ));
                    }
                    i += 1;
                }
                "--fail-if" => {
                    let val = get_value!("--fail-if");
                    let cond = ThresholdCondition::parse(&val)?;
                    flags.fail_if.push(cond);
                }
                other if other.starts_with("--") => {
                    return Err(AppError::Usage(format!("unknown flag: '{}'", other)));
                }
                _ => {
                    // Positional argument — skip for now (main.rs handles them)
                    i += 1;
                }
            }

            // Mutual exclusion check after each flag is set
            if proofd_addr_set && snapshot_file_set {
                return Err(AppError::Usage(
                    "--proofd-addr and --snapshot-file are mutually exclusive".to_string(),
                ));
            }
        }

        Ok(flags)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(parts: &[&str]) -> Vec<String> {
        // Prepend a fake argv[0]
        let mut v = vec!["obs-cli".to_string()];
        v.extend(parts.iter().map(|s| s.to_string()));
        v
    }

    #[test]
    fn no_args_returns_defaults() {
        let flags = Flags::parse(&args(&[])).unwrap();
        assert_eq!(flags.proofd_addr, "http://127.0.0.1:7777");
        assert_eq!(flags.timeout_ms, 5000);
        assert!(flags.snapshot_file.is_none());
        assert!(flags.save_snapshot.is_none());
        assert!(flags.diff_baseline.is_none());
        assert!(!flags.json_output);
        assert!(flags.fail_if.is_empty());
    }

    #[test]
    fn snapshot_file_space_form() {
        let flags = Flags::parse(&args(&["--snapshot-file", "path.json"])).unwrap();
        assert_eq!(flags.snapshot_file, Some(PathBuf::from("path.json")));
    }

    #[test]
    fn snapshot_file_equals_form() {
        let flags = Flags::parse(&args(&["--snapshot-file=path.json"])).unwrap();
        assert_eq!(flags.snapshot_file, Some(PathBuf::from("path.json")));
    }

    #[test]
    fn proofd_addr_space_form() {
        let flags = Flags::parse(&args(&["--proofd-addr", "http://localhost:9999"])).unwrap();
        assert_eq!(flags.proofd_addr, "http://localhost:9999");
    }

    #[test]
    fn proofd_addr_and_snapshot_file_mutually_exclusive() {
        let result = Flags::parse(&args(&[
            "--proofd-addr",
            "http://localhost:9999",
            "--snapshot-file",
            "path.json",
        ]));
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.exit_code(), 1);
        assert!(matches!(err, AppError::Usage(_)));
        if let AppError::Usage(msg) = err {
            assert!(
                msg.contains("mutually exclusive"),
                "expected 'mutually exclusive' in: {}",
                msg
            );
        }
    }

    #[test]
    fn timeout_ms_valid() {
        let flags = Flags::parse(&args(&["--timeout-ms", "3000"])).unwrap();
        assert_eq!(flags.timeout_ms, 3000);
    }

    #[test]
    fn timeout_ms_invalid_string() {
        let result = Flags::parse(&args(&["--timeout-ms", "abc"]));
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.exit_code(), 1);
        assert!(matches!(err, AppError::Usage(_)));
        if let AppError::Usage(msg) = err {
            assert!(
                msg.contains("invalid --timeout-ms value: 'abc'"),
                "unexpected message: {}",
                msg
            );
        }
    }

    #[test]
    fn json_flag() {
        let flags = Flags::parse(&args(&["--json"])).unwrap();
        assert!(flags.json_output);
    }

    #[test]
    fn fail_if_valid_condition() {
        let flags = Flags::parse(&args(&["--fail-if", "conflict_count>0"])).unwrap();
        assert_eq!(flags.fail_if.len(), 1);
    }

    #[test]
    fn fail_if_unknown_field_returns_usage() {
        let result = Flags::parse(&args(&["--fail-if", "unknown_field>0"]));
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.exit_code(), 1);
        assert!(matches!(err, AppError::Usage(_)));
    }

    #[test]
    fn unknown_flag_returns_usage() {
        let result = Flags::parse(&args(&["--bogus"]));
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.exit_code(), 1);
        if let AppError::Usage(msg) = err {
            assert!(
                msg.contains("unknown flag: '--bogus'"),
                "unexpected message: {}",
                msg
            );
        }
    }

    #[test]
    fn proofd_addr_equals_form() {
        let flags = Flags::parse(&args(&["--proofd-addr=http://localhost:9999"])).unwrap();
        assert_eq!(flags.proofd_addr, "http://localhost:9999");
    }

    #[test]
    fn timeout_ms_equals_form() {
        let flags = Flags::parse(&args(&["--timeout-ms=3000"])).unwrap();
        assert_eq!(flags.timeout_ms, 3000);
    }

    #[test]
    fn multiple_fail_if_conditions() {
        let flags = Flags::parse(&args(&[
            "--fail-if",
            "conflict_count>0",
            "--fail-if",
            "total_incidents>=5",
        ]))
        .unwrap();
        assert_eq!(flags.fail_if.len(), 2);
    }
}
