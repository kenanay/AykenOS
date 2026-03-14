use crate::errors::VerifierRuntimeError;
use serde::Serialize;
use serde_json::Value;

pub fn canonicalize_json_bytes(bytes: &[u8]) -> Result<Vec<u8>, VerifierRuntimeError> {
    let value: Value = serde_json::from_slice(bytes)
        .map_err(|error| VerifierRuntimeError::json("parse json", error))?;
    canonicalize_json_value(&value)
}

pub fn canonicalize_json<T: Serialize>(value: &T) -> Result<Vec<u8>, VerifierRuntimeError> {
    let json_value = serde_json::to_value(value)
        .map_err(|error| VerifierRuntimeError::json("serialize json", error))?;
    canonicalize_json_value(&json_value)
}

pub fn canonicalize_json_value(value: &Value) -> Result<Vec<u8>, VerifierRuntimeError> {
    let mut output = String::new();
    write_value(value, &mut output)?;
    output.push('\n');
    Ok(output.into_bytes())
}

fn write_value(value: &Value, output: &mut String) -> Result<(), VerifierRuntimeError> {
    match value {
        Value::Null => output.push_str("null"),
        Value::Bool(boolean) => output.push_str(if *boolean { "true" } else { "false" }),
        Value::Number(number) => {
            output.push_str(
                &serde_json::to_string(number)
                    .map_err(|error| VerifierRuntimeError::json("canonicalize number", error))?,
            );
        }
        Value::String(string) => {
            output.push_str(
                &serde_json::to_string(string)
                    .map_err(|error| VerifierRuntimeError::json("canonicalize string", error))?,
            );
        }
        Value::Array(values) => {
            output.push('[');
            for (index, item) in values.iter().enumerate() {
                if index > 0 {
                    output.push(',');
                }
                write_value(item, output)?;
            }
            output.push(']');
        }
        Value::Object(map) => {
            output.push('{');
            let mut keys: Vec<&str> = map.keys().map(String::as_str).collect();
            keys.sort_unstable();
            for (index, key) in keys.iter().enumerate() {
                if index > 0 {
                    output.push(',');
                }
                output.push_str(
                    &serde_json::to_string(key)
                        .map_err(|error| VerifierRuntimeError::json("canonicalize key", error))?,
                );
                output.push(':');
                write_value(&map[*key], output)?;
            }
            output.push('}');
        }
    }
    Ok(())
}
