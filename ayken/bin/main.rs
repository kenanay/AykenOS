//! AykenOS Authentication CLI Tool
//! 
//! Copyright (c) 2026 Kenan AY. All rights reserved.

use ayken::auth;
use std::env;

fn main() {
    println!("AykenOS Authentication System v1.0");
    println!("Copyright (c) 2026 Kenan AY. All rights reserved.\n");

    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        print_usage();
        return;
    }

    match args[1].as_str() {
        "info" => show_project_info(),
        "verify" => verify_project(),
        "fingerprint" => show_fingerprint(),
        "check" => check_compliance(),
        "help" | "--help" | "-h" => print_usage(),
        _ => {
            eprintln!("Unknown command: {}", args[1]);
            print_usage();
        }
    }
}

fn print_usage() {
    println!("Usage: ayken-auth <command>");
    println!();
    println!("Commands:");
    println!("  info        Show project information");
    println!("  verify      Verify project authenticity");
    println!("  fingerprint Show project fingerprint");
    println!("  check       Check license compliance");
    println!("  help        Show this help message");
    println!();
    println!("For commercial licensing: kenanay@example.com");
}

fn show_project_info() {
    let info = auth::get_project_info();
    println!("Project Information:");
    println!("  Name: {}", info.name);
    println!("  Author: {}", info.author);
    println!("  Copyright: {}", info.copyright);
    println!("  Version: {}", info.version);
    println!("  License: {}", info.license);
    println!("  Fingerprint: {}", &info.fingerprint[..16]);
}

fn verify_project() {
    match auth::authenticate_project() {
        Ok(info) => {
            println!("✅ Project verification successful");
            println!("   Project: {}", info.name);
            println!("   Author: {}", info.author);
            println!("   Fingerprint: {}", &info.fingerprint[..16]);
        }
        Err(e) => {
            eprintln!("❌ Project verification failed: {}", e);
            std::process::exit(1);
        }
    }
}

fn show_fingerprint() {
    let fingerprint = auth::generate_project_fingerprint();
    println!("Project Fingerprint: {}", fingerprint);
    println!("Short: {}", &fingerprint[..16]);
}

fn check_compliance() {
    println!("Checking license compliance...");
    
    match auth::validate_license_compliance() {
        Ok(_) => println!("✅ License compliance verified"),
        Err(e) => {
            eprintln!("❌ License compliance failed: {}", e);
            std::process::exit(1);
        }
    }
    
    match auth::check_development_environment() {
        Ok(_) => println!("✅ Development environment verified"),
        Err(e) => {
            eprintln!("❌ Development environment check failed: {}", e);
            std::process::exit(1);
        }
    }
    
    println!("✅ All compliance checks passed");
}