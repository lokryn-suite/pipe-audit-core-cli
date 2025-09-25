use crate::connectors::{Connector, S3Connector};
use crate::contracts::load_contract_for_file;
use crate::profiles::load_profiles;
use crate::runner;
use glob::glob;
use std::path::Path;

pub async fn run(all: bool) {
    if all {
        let profiles = load_profiles().expect("Failed to load profiles");

        for entry in glob("contracts/*.toml").expect("Failed to read glob pattern") {
            match entry {
                Ok(path) => {
                    let contract_file = path.to_string_lossy().to_string();
                    if let Err(e) = validate_with_contract(&contract_file, &profiles).await {
                        eprintln!("❌ Validation failed for {}: {e}", contract_file);
                    }
                }
                Err(e) => eprintln!("❌ Error reading contract file: {e}"),
            }
        }
    } else {
        eprintln!("No run mode specified. Try `--all`.");
    }
}

async fn validate_with_contract(
    contract_path: &str,
    profiles: &crate::profiles::Profiles,
) -> Result<(), Box<dyn std::error::Error>> {
    let path = Path::new(contract_path);
    let contracts = load_contract_for_file(path);

    let source = contracts
        .source
        .as_ref()
        .ok_or("Contract missing [source] section")?;

    let data: Vec<u8> = match source.r#type.as_str() {
        "local" => {
            let location = source.location.as_ref().ok_or("Local source missing location")?;
            println!("📂 Reading local file {}", location);
            std::fs::read(location)?
        }
        "s3" => {
            let profile_name = source.profile.as_ref().ok_or("S3 source requires profile")?;
            let profile = profiles
                .get(profile_name)
                .ok_or_else(|| format!("Profile '{}' not found", profile_name))?;
            let location = source.location.as_ref().ok_or("S3 source missing location")?;
            
            println!("🔎 Fetching {} via profile {}", location, profile_name);
            let url = url::Url::parse(location)?;
            let connector = S3Connector::from_profile_and_url(profile, &url).await?;
            let mut reader = connector.fetch(location).await?;
            let mut buffer = Vec::new();
            std::io::Read::read_to_end(&mut reader, &mut buffer)?;
            buffer
        }
        "azure" => {
            let location = source.location.as_ref().ok_or("Azure source missing location")?;
            println!("☁️ Azure fetch not yet implemented for {}", location);
            return Err("Azure connector not implemented".into());
        }
        "gcs" => {
            let location = source.location.as_ref().ok_or("GCS source missing location")?;
            println!("☁️ GCS fetch not yet implemented for {}", location);
            return Err("GCS connector not implemented".into());
        }
        "sftp" => {
            let location = source.location.as_ref().ok_or("SFTP source missing location")?;
            println!("🔐 SFTP fetch not yet implemented for {}", location);
            return Err("SFTP connector not implemented".into());
        }
        "not_moved" => {
            println!("⚠️ Source marked as not_moved, skipping");
            return Ok(());
        }
        other => return Err(format!("Unsupported source type: {}", other).into()),
    };

    let extension = source
        .location
        .as_ref()
        .and_then(|loc| Path::new(loc).extension().and_then(|s| s.to_str()))
        .unwrap_or("csv");

    runner::validate_data(&data, extension, &contracts).await?;
    println!("✅ Validation passed for {}", contract_path);
    Ok(())
}
