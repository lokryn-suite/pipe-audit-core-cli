use crate::contracts::SchemaContracts;

pub async fn run(file: &str) {
    let path = format!("contracts/{}", file);

    match std::fs::read_to_string(&path) {
        Ok(content) => match toml::from_str::<SchemaContracts>(&content) {
            Ok(_) => println!("✅ {} is a valid contract TOML", file),
            Err(e) => eprintln!("❌ Parse error in {}: {e}", file),
        },
        Err(e) => eprintln!("❌ Could not read {}: {e}", path),
    }
}
