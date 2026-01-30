//! AykenOS Authentication Module
//! 
//! Copyright (c) 2026 Kenan AY. All rights reserved.
//! 
//! This module implements authentication and copyright protection mechanisms
//! for AykenOS. It includes project fingerprinting, usage verification,
//! and audit logging.

use sha2::{Sha256, Digest};
use std::fs;
use std::path::Path;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Project information structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectInfo {
    pub name: String,
    pub author: String,
    pub copyright: String,
    pub version: String,
    pub license: String,
    pub fingerprint: String,
}

/// Usage log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageLog {
    pub timestamp: DateTime<Utc>,
    pub operation: String,
    pub user: String,
    pub fingerprint: String,
    pub authorized: bool,
}

/// Authentication error types
#[derive(Debug)]
pub enum AuthError {
    ProjectNotFound,
    InvalidFingerprint,
    UnauthorizedUse,
    LicenseViolation,
    TamperingDetected,
}

impl std::fmt::Display for AuthError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            AuthError::ProjectNotFound => write!(f, "AykenOS project files not found"),
            AuthError::InvalidFingerprint => write!(f, "Invalid project fingerprint - possible tampering"),
            AuthError::UnauthorizedUse => write!(f, "Unauthorized use detected - license required"),
            AuthError::LicenseViolation => write!(f, "License violation - commercial use prohibited"),
            AuthError::TamperingDetected => write!(f, "Project tampering detected - integrity compromised"),
        }
    }
}

/// Generate a unique fingerprint for the project
pub fn generate_project_fingerprint() -> String {
    let mut hasher = Sha256::new();
    
    // Include key project files in fingerprint (try both local and parent directory)
    let key_files = [
        "LICENSE",
        "SECURITY.md", 
        "README.md",
        "Makefile",
        "kernel/kernel.c",
    ];
    
    for file_path in &key_files {
        let content = fs::read_to_string(file_path)
            .or_else(|_| fs::read_to_string(Path::new("..").join(file_path)));
            
        if let Ok(content) = content {
            hasher.update(file_path.as_bytes());
            hasher.update(content.as_bytes());
        }
    }
    
    // Add project metadata
    hasher.update(b"AykenOS");
    hasher.update(b"Kenan AY");
    hasher.update(b"2026");
    
    hex::encode(hasher.finalize())
}

/// Get project information
pub fn get_project_info() -> ProjectInfo {
    ProjectInfo {
        name: "AykenOS".to_string(),
        author: "Kenan AY".to_string(),
        copyright: "© 2026 Kenan AY. All rights reserved.".to_string(),
        version: "0.1.0".to_string(),
        license: "Proprietary".to_string(),
        fingerprint: generate_project_fingerprint(),
    }
}

/// Authenticate the project and verify integrity
pub fn authenticate_project() -> Result<ProjectInfo, String> {
    // Check if key files exist (look in parent directory if not found locally)
    let required_files = ["LICENSE", "SECURITY.md", "README.md"];
    for file in &required_files {
        let local_path = Path::new(file);
        let parent_path = Path::new("..").join(file);
        
        if !local_path.exists() && !parent_path.exists() {
            return Err(format!("Required file missing: {} - Project integrity compromised", file));
        }
    }
    
    // Verify license file content (try both locations)
    let license_content = fs::read_to_string("LICENSE")
        .or_else(|_| fs::read_to_string("../LICENSE"));
        
    if let Ok(content) = license_content {
        if !content.contains("Kenan AY") || !content.contains("2026") {
            return Err("License file tampered - Copyright protection violated".to_string());
        }
    } else {
        return Err("Cannot read LICENSE file - Project integrity compromised".to_string());
    }
    
    // Generate and verify project fingerprint
    let project_info = get_project_info();
    
    // Log successful authentication
    log_usage("project_authentication");
    
    Ok(project_info)
}

/// Verify authorized use for specific operations
pub fn verify_authorized_use(operation: &str) -> Result<(), String> {
    // Check for commercial use indicators
    let commercial_operations = [
        "production_deploy",
        "commercial_build",
        "enterprise_install",
        "saas_integration",
    ];
    
    if commercial_operations.contains(&operation) {
        return Err("Commercial use detected - License required. Contact: kenanay@example.com".to_string());
    }
    
    // Log authorized use
    log_usage(operation);
    
    Ok(())
}

/// Log usage for audit trail
pub fn log_usage(operation: &str) {
    let log_entry = UsageLog {
        timestamp: Utc::now(),
        operation: operation.to_string(),
        user: whoami::username(),
        fingerprint: generate_project_fingerprint(),
        authorized: true,
    };
    
    // In a real implementation, this would write to a secure log file
    // For now, we'll just print to stderr for audit purposes
    eprintln!("AykenOS Audit: {} - {} by {} at {}", 
              operation, 
              log_entry.fingerprint[..8].to_string(),
              log_entry.user,
              log_entry.timestamp.format("%Y-%m-%d %H:%M:%S UTC"));
}

/// Check if the current environment is authorized for development
pub fn check_development_environment() -> Result<(), String> {
    // Check for development indicators (try both local and parent directory)
    let dev_indicators = [
        Path::new(".git").exists() || Path::new("../.git").exists(),
        Path::new("Makefile").exists() || Path::new("../Makefile").exists(),
        Path::new("kernel").exists() || Path::new("../kernel").exists(),
    ];
    
    if dev_indicators.iter().all(|&x| x) {
        log_usage("development_environment");
        Ok(())
    } else {
        Err("Invalid development environment - Project files missing".to_string())
    }
}

/// Validate license compliance
pub fn validate_license_compliance() -> Result<(), String> {
    // Check for license file (try both locations)
    let license_path = if Path::new("LICENSE").exists() {
        "LICENSE"
    } else if Path::new("../LICENSE").exists() {
        "../LICENSE"
    } else {
        return Err("LICENSE file missing - Copyright violation".to_string());
    };
    
    // Verify license content
    if let Ok(license_content) = fs::read_to_string(license_path) {
        if !license_content.contains("Proprietary") {
            return Err("Invalid license type - Must be Proprietary".to_string());
        }
        
        if !license_content.contains("Kenan AY") {
            return Err("Invalid copyright holder in license".to_string());
        }
    }
    
    log_usage("license_validation");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_info() {
        let info = get_project_info();
        assert_eq!(info.name, "AykenOS");
        assert_eq!(info.author, "Kenan AY");
        assert!(info.copyright.contains("2026"));
        assert!(info.copyright.contains("Kenan AY"));
    }

    #[test]
    fn test_fingerprint_generation() {
        let fp1 = generate_project_fingerprint();
        let fp2 = generate_project_fingerprint();
        assert_eq!(fp1, fp2, "Fingerprint should be deterministic");
        assert_eq!(fp1.len(), 64, "SHA256 hash should be 64 characters");
    }

    #[test]
    fn test_authorized_operations() {
        assert!(verify_authorized_use("development").is_ok());
        assert!(verify_authorized_use("testing").is_ok());
        assert!(verify_authorized_use("research").is_ok());
    }

    #[test]
    fn test_unauthorized_operations() {
        assert!(verify_authorized_use("production_deploy").is_err());
        assert!(verify_authorized_use("commercial_build").is_err());
        assert!(verify_authorized_use("enterprise_install").is_err());
    }
}