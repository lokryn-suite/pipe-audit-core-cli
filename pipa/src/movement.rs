//! File movement orchestration.
//!
//! This module handles writing validated data to its final destination
//! (success path) or to quarantine (failure path). It supports multiple
//! backends (local filesystem, S3, Azure, GCS) and integrates with
//! configured profiles for authentication.
//!
//! Responsibilities:
//! - Validate profile connectivity before movement.
//! - Generate unique filenames with timestamps (and quarantine suffix).
//! - Serialize Polars DataFrames into CSV or Parquet.
//! - Write data via the appropriate connector.
//!
//! ## Supported types
//! - `"local"`: local filesystem
//! - `"s3"`: Amazon S3
//! - `"azure"`: Azure Blob Storage
//! - `"gcs"`: Google Cloud Storage
//! - `"not_moved"`: skip movement
//!
//! ## Usage
//! - Call `FileMovement::write_success_data` after validation passes.
//! - Call `FileMovement::write_quarantine_data` after validation fails.
//! - Use `FileMovement::validate_profiles` to pre‑check connectivity.

use crate::connectors::{AzureConnector, GCSConnector, S3Connector};
use crate::contracts::schema::{Destination, Quarantine, Source};
use crate::engine::profiles::test_profile_internal;
use crate::profiles::Profiles;
use anyhow::{Result, anyhow, bail};
use chrono::Utc;
use polars::prelude::{CsvWriter, DataFrame, ParquetWriter};
use polars_io::SerWriter;
use std::io::Cursor;
use std::path::Path;
use url::Url;

/// File movement orchestrator.
pub struct FileMovement;

impl FileMovement {
    /// Validate connectivity for source, destination, and quarantine profiles.
    ///
    /// Returns a tuple of booleans: `(source_valid, dest_valid, quarantine_valid)`.
    pub async fn validate_profiles(
        source: Option<&Source>,
        destination: Option<&Destination>,
        quarantine: Option<&Quarantine>,
        profiles: &Profiles,
    ) -> (bool, bool, bool) {
        let source_valid = Self::test_profile_connectivity(
            source.and_then(|s| s.profile.as_ref()),
            source.map(|s| s.r#type.as_str()),
            profiles,
        )
        .await;

        let dest_valid = Self::test_profile_connectivity(
            destination.and_then(|d| d.profile.as_ref()),
            destination.map(|d| d.r#type.as_str()),
            profiles,
        )
        .await;

        let quarantine_valid = Self::test_profile_connectivity(
            quarantine.and_then(|q| q.profile.as_ref()),
            quarantine.map(|q| q.r#type.as_str()),
            profiles,
        )
        .await;

        (source_valid, dest_valid, quarantine_valid)
    }

    /// Internal helper: test connectivity for a given profile/type.
    async fn test_profile_connectivity(
        profile_name: Option<&String>,
        destination_type: Option<&str>,
        profiles: &Profiles,
    ) -> bool {
        match destination_type {
            Some("local") | Some("not_moved") => true,
            _ => {
                if let Some(name) = profile_name {
                    test_profile_internal(name, profiles).await
                } else {
                    false
                }
            }
        }
    }

    /// Generate a unique filename with timestamp and optional quarantine suffix.
    fn generate_filename(
        original_location: &str,
        is_quarantine: bool,
        format_override: Option<&str>,
    ) -> String {
        let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
        let path = Path::new(original_location);
        let stem = path.file_stem().unwrap_or_default().to_string_lossy();

        let extension = format_override
            .or_else(|| path.extension().and_then(|s| s.to_str()))
            .unwrap_or("csv");

        if is_quarantine {
            format!("{}_{}_quarantine.{}", stem, timestamp, extension)
        } else {
            format!("{}_{}.{}", stem, timestamp, extension)
        }
    }

    /// Write validated data to the configured **destination**.
    pub async fn write_success_data(
        df: &DataFrame,
        original_location: &str,
        destination: &Destination,
        profiles: &Profiles,
    ) -> Result<()> {
        let filename =
            Self::generate_filename(original_location, false, destination.format.as_deref());
        let format = destination.format.as_deref().unwrap_or("csv");
        let data = Self::serialize_dataframe(df, format)?;

        let write_config = Source {
            r#type: destination.r#type.clone(),
            location: Some(Self::build_destination_path(
                destination.location.as_ref().unwrap(),
                &filename,
            )),
            profile: destination.profile.clone(),
        };

        Self::write_data_via_connector(&data, &write_config, profiles).await
    }

    /// Write failed data to the configured **quarantine**.
    pub async fn write_quarantine_data(
        df: &DataFrame,
        original_location: &str,
        quarantine: &Quarantine,
        profiles: &Profiles,
    ) -> Result<()> {
        let filename =
            Self::generate_filename(original_location, true, quarantine.format.as_deref());
        let format = quarantine.format.as_deref().unwrap_or("csv");
        let data = Self::serialize_dataframe(df, format)?;

        let write_config = Source {
            r#type: quarantine.r#type.clone(),
            location: Some(Self::build_destination_path(
                quarantine.location.as_ref().unwrap(),
                &filename,
            )),
            profile: quarantine.profile.clone(),
        };

        Self::write_data_via_connector(&data, &write_config, profiles).await
    }

    /// Build a full destination path by appending filename to base location.
    fn build_destination_path(base_location: &str, filename: &str) -> String {
        if base_location.ends_with('/') {
            format!("{}{}", base_location, filename)
        } else {
            format!("{}/{}", base_location, filename)
        }
    }

    /// Write serialized data to the configured backend.
    async fn write_data_via_connector(
        data: &[u8],
        config: &Source,
        profiles: &Profiles,
    ) -> Result<()> {
        match config.r#type.as_str() {
            "local" => {
                let location = config.location.as_ref().unwrap();
                if let Some(parent) = Path::new(location).parent() {
                    std::fs::create_dir_all(parent)?;
                }
                std::fs::write(location, data)?;
                println!("📁 Wrote {} bytes to {}", data.len(), location);
                Ok(())
            }
            "s3" => {
                let profile_name = config.profile.as_ref().unwrap();
                let profile = profiles
                    .get(profile_name)
                    .ok_or_else(|| anyhow!("Profile '{}' not found", profile_name))?;
                let location = config.location.as_ref().unwrap();
                let url = Url::parse(location)?;
                let connector = S3Connector::from_profile_and_url(profile, &url).await?;
                connector.put_object_from_url(location, data).await?;
                println!("📤 Wrote {} bytes to {}", data.len(), location);
                Ok(())
            }
            "azure" => {
                let profile_name = config.profile.as_ref().unwrap();
                let profile = profiles
                    .get(profile_name)
                    .ok_or_else(|| anyhow!("Profile '{}' not found", profile_name))?;
                let location = config.location.as_ref().unwrap();
                let url = Url::parse(location)?;
                let connector = AzureConnector::from_profile_and_url(profile, &url).await?;
                connector.put_object_from_url(location, data).await?;
                println!("☁️ Wrote {} bytes to {}", data.len(), location);
                Ok(())
            }
            "gcs" => {
                let profile_name = config.profile.as_ref().unwrap();
                let profile = profiles
                    .get(profile_name)
                    .ok_or_else(|| anyhow!("Profile '{}' not found", profile_name))?;
                let location = config.location.as_ref().unwrap();
                let url = Url::parse(location)?;
                let connector = GCSConnector::from_profile_and_url(profile, &url).await?;
                connector.put_object_from_url(location, data).await?;
                println!("☁️ Wrote {} bytes to {}", data.len(), location);
                Ok(())
            }
            "not_moved" => {
                println!("📄 Marked as not_moved, skipping write");
                Ok(())
            }
            _ => bail!("Unsupported type: {}", config.r#type),
        }
    }
    /// Serialize a DataFrame into the requested format (CSV or Parquet).
    fn serialize_dataframe(df: &DataFrame, format: &str) -> Result<Vec<u8>> {
        match format.to_lowercase().as_str() {
            "csv" => {
                let mut buffer = Vec::new();
                let mut cursor = Cursor::new(&mut buffer);
                let mut df_clone = df.clone();
                CsvWriter::new(&mut cursor)
                    .include_header(true)
                    .finish(&mut df_clone)?;
                Ok(buffer)
            }
            "parquet" => {
                let mut buffer = Vec::new();
                let mut cursor = Cursor::new(&mut buffer);
                let mut df_clone = df.clone();
                ParquetWriter::new(&mut cursor).finish(&mut df_clone)?;
                Ok(buffer)
            }
            _ => bail!("Unsupported output format: {}", format),
        }
    }
}
