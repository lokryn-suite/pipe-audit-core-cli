use crate::profiles::load_profiles;
use std::path::Path;

pub async fn run() {
    let mut healthy = true;

    // Check contracts directory
    if !Path::new("contracts").exists() {
        println!("❌ contracts/ directory not found");
        healthy = false;
    } else {
        println!("✅ contracts/ directory exists");
    }

    // Check logs directory
    if !Path::new("logs").exists() {
        println!("❌ logs/ directory not found");
        healthy = false;
    } else {
        println!("✅ logs/ directory exists");
    }

    // Check profiles
    match load_profiles() {
        Ok(profiles) => {
            if profiles.is_empty() {
                println!("⚠️  No profiles configured");
            } else {
                println!("✅ {} profile(s) loaded", profiles.len());
            }
        }
        Err(_) => {
            println!("❌ Failed to load profiles");
            healthy = false;
        }
    }

    if healthy {
        println!("\n🎉 System healthy");
    } else {
        println!("\n💥 System issues detected. Check logs for details.");
    }
}
