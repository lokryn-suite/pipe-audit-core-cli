use crate::contracts::SchemaContracts;
use crate::drivers::get_driver;
use crate::engine::validate_dataframe;
use anyhow::{Context, Result};

#[cfg(feature = "file-management")]
use crate::movement::FileMovement;
#[cfg(feature = "file-management")]
use crate::profiles::load_profiles;

pub async fn validate_data(
    data: &[u8],
    extension: &str,
    contracts: &SchemaContracts,
) -> Result<()> {
    println!(
        "🔍 Starting validation with {} bytes, extension: {}",
        data.len(),
        extension
    );

    // Step 1: Early profile validation (feature-flagged)
    #[cfg(feature = "file-management")]
    let (_source_valid, dest_valid, quarantine_valid) = {
        let profiles = match load_profiles() {
            Ok(profiles) => profiles,
            Err(_) => {
                eprintln!("❌ Failed to load profiles. Check logs for details.");
                return Ok(());
            }
        };
        FileMovement::validate_profiles(
            contracts.source.as_ref(),
            contracts.destination.as_ref(),
            contracts.quarantine.as_ref(),
            &profiles,
        )
        .await
    };

    #[cfg(feature = "file-management")]
    {
        if !dest_valid && contracts.destination.is_some() {
            println!(
                "⚠️ Destination profile invalid. Validation will proceed without file movement."
            );
        }
        if !quarantine_valid && contracts.quarantine.is_some() {
            println!("⚠️ Quarantine profile invalid. Failed files will not be moved.");
        }
    }

    // Step 2: Parse and validate data
    let driver =
        get_driver(extension).context("Failed to find a suitable driver for the extension")?;
    println!("✅ Found driver for extension: {}", extension);

    let df = driver
        .load(data)
        .context("Failed to parse data from memory")?;
    println!(
        "✅ Parsed DataFrame with {} rows, {} columns",
        df.height(),
        df.width()
    );

    let validation_result = validate_dataframe(&df, contracts);
    let validation_passed = match validation_result {
        Ok(passed) => passed,
        Err(_) => {
            println!("❌ Validation engine error");
            false
        }
    };

    if validation_passed {
        println!("✅ Validation completed - All checks passed");
    } else {
        println!("❌ Validation completed - Some checks failed");
    }

    // Step 3: File movement based on validation result (feature-flagged)
    #[cfg(feature = "file-management")]
    {
        let profiles = match load_profiles() {
            Ok(profiles) => profiles,
            Err(_) => {
                eprintln!("❌ Failed to load profiles for file movement.");
                return Ok(());
            }
        };

        let original_location = contracts
            .source
            .as_ref()
            .and_then(|s| s.location.as_ref())
            .map(|s| s.as_str())
            .unwrap_or("unknown");

        if validation_passed && dest_valid {
            if let Some(destination) = &contracts.destination {
                match FileMovement::write_success_data(
                    &df,
                    original_location,
                    destination,
                    &profiles,
                )
                .await
                {
                    Ok(_) => println!("✅ Data written to destination"),
                    Err(e) => eprintln!("❌ Failed to write to destination: {}", e),
                }
            }
        } else if !validation_passed && quarantine_valid {
            if let Some(quarantine) = &contracts.quarantine {
                match FileMovement::write_quarantine_data(
                    &df,
                    original_location,
                    quarantine,
                    &profiles,
                )
                .await
                {
                    Ok(_) => println!("⚠️ Data quarantined"),
                    Err(e) => eprintln!("❌ Failed to write to quarantine: {}", e),
                }
            }
        }
    }

    Ok(())
}
