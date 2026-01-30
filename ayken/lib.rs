//! AykenOS Authentication and Protection System
//! 
//! Copyright (c) 2026 Kenan AY. All rights reserved.
//! 
//! This module provides authentication and copyright protection for AykenOS.
//! Unauthorized use, modification, or distribution is strictly prohibited.

pub mod auth;

use std::sync::Once;

static INIT: Once = Once::new();

/// Initialize the AykenOS authentication system
/// 
/// This function must be called before using any AykenOS functionality.
/// It verifies the project authenticity and logs usage.
pub fn init() -> Result<(), String> {
    let mut result = Ok(());
    
    INIT.call_once(|| {
        match auth::authenticate_project() {
            Ok(_) => {
                println!("AykenOS Authentication: ✅ Project verified");
                auth::log_usage("system_init");
            }
            Err(e) => {
                eprintln!("AykenOS Authentication: ❌ {}", e);
                result = Err(e);
            }
        }
    });
    
    result
}

/// Verify that the current usage is authorized
pub fn verify_usage(operation: &str) -> Result<(), String> {
    auth::verify_authorized_use(operation)
}

/// Get project information
pub fn project_info() -> auth::ProjectInfo {
    auth::get_project_info()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init() {
        // This test verifies that the authentication system initializes
        let result = init();
        assert!(result.is_ok(), "Authentication system should initialize successfully");
    }

    #[test]
    fn test_project_info() {
        let info = project_info();
        assert_eq!(info.name, "AykenOS");
        assert_eq!(info.author, "Kenan AY");
        assert_eq!(info.copyright, "© 2026 Kenan AY. All rights reserved.");
    }
}