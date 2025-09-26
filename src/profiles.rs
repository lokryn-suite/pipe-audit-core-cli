// src/profiles.rs
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;

#[derive(Debug, Deserialize, Clone)]
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
    let content = fs::read_to_string("profiles.toml")?;
    let profiles: Profiles = toml::from_str(&content)?;
    
    // Handle environment variable expansion
    let mut expanded_profiles = HashMap::new();
    for (name, mut profile) in profiles {
        profile.access_key = expand_env_vars(&profile.access_key);
        profile.secret_key = expand_env_vars(&profile.secret_key);
        expanded_profiles.insert(name, profile);
    }
    
    Ok(expanded_profiles)
}

fn expand_env_vars(value: &str) -> String {
    // Simple env var expansion for ${VAR_NAME}
    if value.starts_with("${") && value.ends_with("}") {
        let var_name = &value[2..value.len()-1];
        std::env::var(var_name).unwrap_or_else(|_| value.to_string())
    } else {
        value.to_string()
    }
}