use crate::engine::{get_contract, list_contracts, validate_contract};
use std::fs;

pub async fn list() {
    match list_contracts() {
        Ok((contract_list, message)) => {
            println!("{}", message);
            for name in contract_list.contracts {
                println!("  - {}", name);
            }
        }
        Err(_) => {
            eprintln!("❌ Failed to read contracts directory. Check logs for details.");
        }
    }
}

pub async fn validate(file: &str) {
    let (_validation, message) = validate_contract(file);
    println!("{}", message);
}

pub async fn show(name: &str) {
    let (contract_info, message) = get_contract(name);
    println!("{}", message);

    if contract_info.exists {
        let path = format!("contracts/{}.toml", name);
        match fs::read_to_string(&path) {
            Ok(content) => {
                println!("📄 Contract: {}", name);
                println!("\n{}", content);
            }
            Err(_) => eprintln!("❌ Failed to read contract content"),
        }
    }
}
