use std::error::Error as StdError;
use std::fmt::{Display, Formatter};
use std::io;

#[derive(Debug)]
pub enum VerifierRuntimeError {
    Io {
        context: String,
        source: io::Error,
    },
    Json {
        context: String,
        source: serde_json::Error,
    },
    Config {
        context: String,
    },
}

impl VerifierRuntimeError {
    pub fn io(context: impl Into<String>, source: io::Error) -> Self {
        Self::Io {
            context: context.into(),
            source,
        }
    }

    pub fn json(context: impl Into<String>, source: serde_json::Error) -> Self {
        Self::Json {
            context: context.into(),
            source,
        }
    }

    pub fn config(context: impl Into<String>) -> Self {
        Self::Config {
            context: context.into(),
        }
    }
}

impl Display for VerifierRuntimeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            VerifierRuntimeError::Io { context, source } => {
                write!(f, "I/O error during {context}: {source}")
            }
            VerifierRuntimeError::Json { context, source } => {
                write!(f, "JSON error during {context}: {source}")
            }
            VerifierRuntimeError::Config { context } => {
                write!(f, "Configuration error: {context}")
            }
        }
    }
}

impl StdError for VerifierRuntimeError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            VerifierRuntimeError::Io { source, .. } => Some(source),
            VerifierRuntimeError::Json { source, .. } => Some(source),
            VerifierRuntimeError::Config { .. } => None,
        }
    }
}
