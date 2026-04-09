use thiserror::Error;

#[derive(Debug, Error)]
pub enum AykenError {
    #[error("policy violation: {0}")]
    Policy(String),

    #[error("process failed: {0}")]
    Process(String),

    #[error("io error: {0}")]
    Io(String),

    #[error("serialization error: {0}")]
    Serialization(String),
}

impl AykenError {
    pub fn exit_code(&self) -> i32 {
        match self {
            AykenError::Policy(_) => 2,
            AykenError::Process(_) => 3,
            AykenError::Io(_) => 4,
            AykenError::Serialization(_) => 5,
        }
    }
}

impl From<std::io::Error> for AykenError {
    fn from(value: std::io::Error) -> Self {
        AykenError::Io(value.to_string())
    }
}

impl From<serde_json::Error> for AykenError {
    fn from(value: serde_json::Error) -> Self {
        AykenError::Serialization(value.to_string())
    }
}
