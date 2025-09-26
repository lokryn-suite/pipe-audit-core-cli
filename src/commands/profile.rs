use crate::profiles::{load_profiles, Profile, Profiles};
use aws_sdk_s3::config::Credentials;

pub async fn list() {
    match load_profiles() {
        Ok(profiles) => {
            if profiles.is_empty() {
                println!("No profiles configured");
            } else {
                println!("Available profiles:");
                for (name, profile) in profiles.iter() {
                    println!("  - {} ({})", name, profile.provider);
                }
            }
        }
        Err(_) => eprintln!("❌ Failed to load profiles. Check logs for details."),
    }
}

pub async fn test(profile_name: &str) {
    let profiles = match load_profiles() {
        Ok(profiles) => profiles,
        Err(_) => {
            eprintln!("❌ Failed to load profiles. Check logs for details.");
            return;
        }
    };

    if test_profile_internal(profile_name, &profiles).await {
        println!("✅ Profile '{}' connectivity verified", profile_name);
    } else {
        eprintln!(
            "❌ Profile '{}' test failed. Check logs for details.",
            profile_name
        );
    }
}

// Extracted for reuse in file movement
pub async fn test_profile_internal(profile_name: &str, profiles: &Profiles) -> bool {
    if let Some(profile) = profiles.get(profile_name) {
        match profile.provider.as_str() {
            "s3" => test_s3_profile_internal(profile).await,
            "local" => true,  // Local always works if profile exists
            "azure" => false, // Not implemented yet
            "gcs" => false,   // Not implemented yet
            "sftp" => false,  // Not implemented yet
            _ => false,
        }
    } else {
        false
    }
}

async fn test_s3_profile_internal(profile: &Profile) -> bool {
    let region = profile
        .region
        .clone()
        .unwrap_or_else(|| "us-east-1".to_string());
    let mut cfg_loader = aws_config::defaults(aws_config::BehaviorVersion::latest())
        .region(aws_config::Region::new(region));

    if let Some(endpoint) = &profile.endpoint {
        cfg_loader = cfg_loader.endpoint_url(endpoint);
    }

    let base = cfg_loader.load().await;
    let mut s3b = aws_sdk_s3::config::Builder::from(&base);

    if profile.path_style.unwrap_or(false) {
        s3b = s3b.force_path_style(true);
    }

    if !profile.access_key.is_empty() && !profile.secret_key.is_empty() {
        let creds = Credentials::new(
            profile.access_key.clone(),
            profile.secret_key.clone(),
            None,
            None,
            "profile",
        );
        s3b = s3b.credentials_provider(creds);
    }

    let client = aws_sdk_s3::Client::from_conf(s3b.build());
    client.list_buckets().send().await.is_ok()
}
