// filepath: /Volumes/External/developyr/source/data-quality/pipe_audit/src/profiles.rs
use serde::Deserialize;
use std::collections::HashMap;
use std::env;

#[derive(Debug, Deserialize)]
pub struct Profile {
    pub provider: String,
    pub endpoint: Option<String>,
    pub region: Option<String>,
    pub access_key: String,
    pub secret_key: String,
    pub path_style: Option<bool>,
    pub use_ssl: Option<bool>,
}

pub type Profiles = HashMap<String, Profile>;

pub fn load_profiles() -> Result<Profiles, Box<dyn std::error::Error>> {
    let content = std::fs::read_to_string("profiles.toml")?;
    let mut profiles: Profiles = toml::from_str(&content)?;

    // Substitute env vars
    for profile in profiles.values_mut() {
        profile.access_key = substitute_env(&profile.access_key);
        profile.secret_key = substitute_env(&profile.secret_key);
    }

    Ok(profiles)
}

fn substitute_env(value: &str) -> String {
    if value.starts_with("${") && value.ends_with('}') {
        let var = &value[2..value.len() - 1];
        env::var(var).unwrap_or_else(|_| value.to_string())
    } else {
        value.to_string()
    }
}
