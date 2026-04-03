use proof_verifier::canonical::jcs::{
    canonicalize_json as canonicalize_json_impl,
    canonicalize_json_value as canonicalize_json_value_impl,
};
use proof_verifier::Error as VerifierError;
use serde::Serialize;
use serde_json::Value;

pub(crate) fn canonicalize<T>(value: &T) -> Result<Vec<u8>, VerifierError>
where
    T: Serialize,
{
    canonicalize_json_impl(value)
}

pub(crate) fn canonicalize_value(value: &Value) -> Result<Vec<u8>, VerifierError> {
    canonicalize_json_value_impl(value)
}
