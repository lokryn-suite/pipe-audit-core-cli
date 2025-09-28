// src/profiles.rs
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;

#[derive(Debug, Deserialize, Clone)]

pub struct Profile {
    pub provider: String,
    pub endpoint: Option<String>,

    // S3 Specific fields
    pub region: Option<String>,
    pub access_key: Option<String>,
    pub secret_key: Option<String>,
    pub path_style: Option<bool>,
    pub use_ssl: Option<bool>,

    // Azure specific fields
    pub account_name: Option<String>,
    pub client_id: Option<String>,
    pub client_secret: Option<String>,
    pub tenant_id: Option<String>,
    pub connection_string: Option<String>,

    //gcp specific fields
    pub service_account_json: Option<String>,
}

pub type Profiles = HashMap<String, Profile>;

fn expand_optional_field<F>(field: &mut Option<String>, updater: F)
where
    F: FnOnce(&str) -> String,
{
    if let Some(ref value) = field.clone() {
        *field = Some(updater(value));
    }
}

pub fn load_profiles() -> Result<Profiles, Box<dyn std::error::Error>> {
    let content = fs::read_to_string("profiles.toml")?;
    let mut profiles: Profiles = toml::from_str(&content)?;

    for (_name, profile) in profiles.iter_mut() {
        expand_optional_field(&mut profile.access_key, expand_env_vars);
        expand_optional_field(&mut profile.secret_key, expand_env_vars);
        expand_optional_field(&mut profile.connection_string, expand_env_vars);
        expand_optional_field(&mut profile.account_name, expand_env_vars);
        expand_optional_field(&mut profile.service_account_json, expand_env_vars);
    }
    Ok(profiles)
}

fn expand_env_vars(value: &str) -> String {
    if value.starts_with("${") && value.ends_with("}") {
        let var_name = &value[2..value.len() - 1];

        match std::env::var(var_name) {
            Ok(env_value) => env_value,
            Err(_e) => value.to_string(),
        }
    } else {
        println!("Debug: Value doesn't match pattern, returning as-is");
        value.to_string()
    }
}
