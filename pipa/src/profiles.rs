//! Profile management for external storage providers.
//!
//! Profiles define how PipeAudit connects to external systems (S3, Azure,
//! GCP, etc.) for reading/writing validated data. They are loaded from
//! a `profiles.toml` file at runtime and may contain environment-variable
//! references for sensitive values.
//!
//! ## Example `profiles.toml`
//! ```toml
//! [s3_profile]
//! provider = "s3"
//! region = "us-east-1"
//! access_key = "${AWS_ACCESS_KEY_ID}"
//! secret_key = "${AWS_SECRET_ACCESS_KEY}"
//!
//! [azure_profile]
//! provider = "azure"
//! account_name = "${AZURE_ACCOUNT_NAME}"
//! client_id = "${AZURE_CLIENT_ID}"
//! client_secret = "${AZURE_CLIENT_SECRET}"
//! tenant_id = "${AZURE_TENANT_ID}"
//! ```
//!
//! ## Behavior
//! - Profiles are deserialized from TOML into `Profile` structs.
//! - Any field wrapped in `${VAR}` is expanded from the environment.
//! - Profiles are stored in a `HashMap<String, Profile>` keyed by profile name.
//!
//! ## Usage
//! - Call `load_profiles()` to load and expand all profiles.
//! - Pass the resulting `Profiles` into file movement or connector logic.

use serde::Deserialize;
use std::collections::HashMap;
use std::fs;

/// A single profile definition for connecting to an external provider.
///
/// Fields are provider-specific. Only the relevant subset is used depending
/// on the `provider` value (`"s3"`, `"azure"`, `"gcp"`, etc.).
#[derive(Debug, Deserialize, Clone)]
pub struct Profile {
    pub provider: String,
    pub endpoint: Option<String>,

    // --- S3 specific fields ---
    pub region: Option<String>,
    pub access_key: Option<String>,
    pub secret_key: Option<String>,
    pub path_style: Option<bool>,

    //pub use_ssl: Option<bool>, TODO: Implement

    // --- Azure specific fields ---
    pub account_name: Option<String>,
    pub connection_string: Option<String>,

    // --- GCP specific fields ---
    pub service_account_json: Option<String>,
}

/// A collection of profiles, keyed by profile name (from TOML section headers).
pub type Profiles = HashMap<String, Profile>;

/// Expand an optional string field using a provided updater function.
///
/// Used to replace `${VAR}` placeholders with environment variables.
fn expand_optional_field<F>(field: &mut Option<String>, updater: F)
where
    F: FnOnce(&str) -> String,
{
    if let Some(ref value) = field.clone() {
        *field = Some(updater(value));
    }
}

/// Load all profiles from `profiles.toml`, expanding environment variables.
///
/// # Returns
/// * `Profiles` - a map of profile name → `Profile`.
///
/// # Errors
/// Returns an error if the file cannot be read or parsed.
pub fn load_profiles() -> Result<Profiles, Box<dyn std::error::Error>> {
    let content = fs::read_to_string("profiles.toml")?;
    let mut profiles: Profiles = toml::from_str(&content)?;

    // Expand environment variables in sensitive fields
    for (_name, profile) in profiles.iter_mut() {
        expand_optional_field(&mut profile.access_key, expand_env_vars);
        expand_optional_field(&mut profile.secret_key, expand_env_vars);
        expand_optional_field(&mut profile.connection_string, expand_env_vars);
        expand_optional_field(&mut profile.account_name, expand_env_vars);
        expand_optional_field(&mut profile.service_account_json, expand_env_vars);
    }
    Ok(profiles)
}

/// Expand `${VAR}` placeholders into environment variable values.
///
/// If the variable is not set, the original string is returned unchanged.
fn expand_env_vars(value: &str) -> String {
    if value.starts_with("${") && value.ends_with("}") {
        let var_name = &value[2..value.len() - 1];
        match std::env::var(var_name) {
            Ok(env_value) => env_value,
            Err(_e) => value.to_string(),
        }
    } else {
        value.to_string()
    }
}
