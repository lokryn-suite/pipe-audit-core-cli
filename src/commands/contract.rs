use crate::contracts::SchemaContracts;
use glob::glob;
use std::fs;

pub async fn list() {
    println!("Available contracts:");

    match glob("contracts/*.toml") {
        Ok(entries) => {
            let mut found_any = false;
            for entry in entries {
                match entry {
                    Ok(path) => {
                        if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                            println!("  - {}", name);
                            found_any = true;
                        }
                    }
                    Err(_) => {
                        eprintln!("❌ Error reading contract files. Check logs for details.");
                        return;
                    }
                }
            }
            if !found_any {
                println!("  No contracts found in contracts/ directory");
            }
        }
        Err(_) => {
            eprintln!("❌ Failed to read contracts directory. Check logs for details.");
        }
    }
}

pub async fn validate(file: &str) {
    let path = if file.ends_with(".toml") {
        format!("contracts/{}", file)
    } else {
        format!("contracts/{}.toml", file)
    };

    match fs::read_to_string(&path) {
        Ok(content) => match toml::from_str::<SchemaContracts>(&content) {
            Ok(_) => println!("✅ {} is a valid contract", file),
            Err(_) => eprintln!("❌ Invalid contract syntax. Check logs for details."),
        },
        Err(_) => eprintln!("❌ Contract file not found: {}", file),
    }
}

pub async fn show(name: &str) {
    let path = format!("contracts/{}.toml", name);

    match fs::read_to_string(&path) {
        Ok(content) => {
            println!("Contract: {}\n", name);
            println!("{}", content);
        }
        Err(_) => eprintln!("❌ Contract '{}' not found", name),
    }
}
