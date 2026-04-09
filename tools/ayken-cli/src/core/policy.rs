use crate::core::env::EnvSnapshot;
use crate::core::error::AykenError;

/// Returns the effective CC to use, enforcing toolchain policy.
/// CC=ayken is forbidden unless --experimental is explicitly set.
pub fn enforce_toolchain_policy(
    snapshot: &EnvSnapshot,
    experimental: bool,
) -> Result<String, AykenError> {
    match snapshot.cc.as_deref() {
        Some("ayken") if !experimental => Err(AykenError::Policy(
            "CC=ayken is forbidden unless --experimental is explicitly provided. \
             See ayken/STATUS.md."
                .to_string(),
        )),
        Some("ayken") => Ok("ayken".to_string()),
        Some(other) if !other.trim().is_empty() => Ok(other.to_string()),
        _ => Ok("clang".to_string()),
    }
}

/// Experimental mode is forbidden in CI.
pub fn ci_forbid_experimental(ci: bool, experimental: bool) -> Result<(), AykenError> {
    if ci && experimental {
        return Err(AykenError::Policy(
            "experimental mode is forbidden in CI (CI=true)".to_string(),
        ));
    }
    Ok(())
}
