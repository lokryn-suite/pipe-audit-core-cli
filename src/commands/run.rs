use crate::connectors::{Connector, S3Connector};
use crate::contracts::load_contract_for_file;
use crate::profiles::load_profiles;
use crate::runner;
use glob::glob;
use std::path::Path;

pub async fn run(all: bool) {
    if all {
        let profiles = load_profiles().expect("Failed to load profiles");

        for entry in glob("data/*.csv").expect("Failed to read glob pattern") {
            match entry {
                Ok(path) => {
                    let file = path.to_string_lossy().to_string();
                    if let Err(e) = validate_with_contract(&file, &profiles).await {
                        eprintln!("❌ Validation failed for {}: {e}", file);
                    }
                }
                Err(e) => eprintln!("❌ Error reading file: {e}"),
            }
        }
    } else {
        eprintln!("No run mode specified. Try `--all`.");
    }
}

async fn validate_with_contract(
    file: &str,
    profiles: &crate::profiles::Profiles,
) -> Result<(), Box<dyn std::error::Error>> {
    let path = Path::new(file);
    let contracts = load_contract_for_file(path);

    let data = if let Some(profile_name) = &contracts.source.profile {
        if let Some(profile) = profiles.get(profile_name) {
            let url = url::Url::parse(&contracts.source.location.as_ref().unwrap())?;
            match contracts.source.r#type.as_str() {
                "s3" => {
                    let connector = S3Connector::from_profile_and_url(profile, &url).await?;
                    let mut reader = connector.fetch(&url.path()[1..]).await?;
                    let mut buffer = String::new();
                    std::io::Read::read_to_string(&mut reader, &mut buffer)?;
                    buffer
                }
                "local" => {
                    std::fs::read_to_string(file)?
                }
                _ => return Err("Unsupported source type".into()),
            }
        } else {
            return Err(format!("Profile '{}' not found", profile_name).into());
        }
    } else {
        std::fs::read_to_string(file)?
    };

    runner::validate_data(&data, &contracts).await?;

    println!("✅ Validation passed for {}", file);
    Ok(())
}
