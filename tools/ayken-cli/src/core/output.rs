use crate::core::error::AykenError;
use serde::Serialize;
use std::io::{self, Write};

pub fn print_json<T: Serialize>(value: &T) -> Result<(), AykenError> {
    let text = serde_json::to_string_pretty(value)?;
    println!("{text}");
    io::stdout().flush()?;
    Ok(())
}
