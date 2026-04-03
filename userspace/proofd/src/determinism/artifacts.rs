use crate::{write_bytes_if_absent_or_same, ServiceError};
use serde::Serialize;
use serde_json::Value;
use std::path::Path;

use super::canonical_json::{canonicalize, canonicalize_value};

pub(crate) fn write_canonical_json_file_if_absent_or_same<T>(
    path: &Path,
    value: &T,
    write_error: &'static str,
    conflict_error: &'static str,
) -> Result<(), ServiceError>
where
    T: Serialize,
{
    let bytes = canonicalize(value).map_err(|_| ServiceError::Runtime(write_error))?;
    write_bytes_if_absent_or_same(path, &bytes, write_error, conflict_error)
}

pub(crate) fn write_canonical_json_value_if_absent_or_same(
    path: &Path,
    value: &Value,
    write_error: &'static str,
    conflict_error: &'static str,
) -> Result<(), ServiceError> {
    let bytes = canonicalize_value(value).map_err(|_| ServiceError::Runtime(write_error))?;
    write_bytes_if_absent_or_same(path, &bytes, write_error, conflict_error)
}
