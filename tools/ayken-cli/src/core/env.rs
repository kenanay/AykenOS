use serde::Serialize;
use std::env;

#[derive(Debug, Serialize)]
pub struct EnvSnapshot {
    pub cc: Option<String>,
    pub rustup_toolchain: Option<String>,
    pub rustflags: Option<String>,
    pub cargo_target_dir: Option<String>,
    pub path_contains_ayken: bool,
}

pub fn snapshot() -> EnvSnapshot {
    let path = env::var("PATH").unwrap_or_default();
    let path_contains_ayken = path
        .split(':')
        .any(|s| s.to_ascii_lowercase().contains("ayken"));

    EnvSnapshot {
        cc: env::var("CC").ok(),
        rustup_toolchain: env::var("RUSTUP_TOOLCHAIN").ok(),
        rustflags: env::var("RUSTFLAGS").ok(),
        cargo_target_dir: env::var("CARGO_TARGET_DIR").ok(),
        path_contains_ayken,
    }
}
