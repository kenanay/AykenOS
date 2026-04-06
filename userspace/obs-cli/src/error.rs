/// Application error type for obs-cli.
///
/// Each variant maps to a specific exit code:
/// - `Usage`     → 1  (bad flags, unknown fields, invalid arguments)
/// - `Http`      → 2  (non-200 HTTP response)
/// - `Io`        → 2  (connection failure, file I/O)
/// - `Parse`     → 3  (malformed JSON, missing required field)
/// - `Schema`    → 3  (float detected, epistemic boundary violation)
/// - `Threshold` → 4  (one or more --fail-if conditions violated)
#[derive(Debug, Clone, PartialEq)]
pub enum AppError {
    Usage(String),
    Http(u16, String),
    Io(String),
    Parse(String),
    Schema(String),
    Threshold(Vec<String>),
}

impl AppError {
    /// Returns the process exit code for this error.
    pub fn exit_code(&self) -> i32 {
        match self {
            AppError::Usage(_) => 1,
            AppError::Http(_, _) => 2,
            AppError::Io(_) => 2,
            AppError::Parse(_) => 3,
            AppError::Schema(_) => 3,
            AppError::Threshold(_) => 4,
        }
    }

    /// Returns a human-readable error message for this error.
    pub fn message(&self) -> String {
        match self {
            AppError::Usage(msg) => format!("usage error: {}", msg),
            AppError::Http(status, body) => {
                format!("HTTP error {}: {}", status, body)
            }
            AppError::Io(msg) => format!("I/O error: {}", msg),
            AppError::Parse(msg) => format!("parse error: {}", msg),
            AppError::Schema(msg) => format!("schema error: {}", msg),
            AppError::Threshold(violations) => violations.join("\n"),
        }
    }
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message())
    }
}
