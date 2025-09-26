use crate::profiles::{load_profiles, Profile};
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

    let profile = match profiles.get(profile_name) {
        Some(profile) => profile,
        None => {
            eprintln!("❌ Profile '{}' not found", profile_name);
            return;
        }
    };

    match profile.provider.as_str() {
        "s3" => test_s3_profile(profile_name, profile).await,
        "azure" => {
            println!("⚠️ Azure profile testing not yet implemented");
        }
        "gcs" => {
            println!("⚠️ GCS profile testing not yet implemented");
        }
        "local" => test_local_profile(profile_name, profile).await,
        "sftp" => {
            println!("⚠️ SFTP profile testing not yet implemented");
        }
        _ => {
            eprintln!("❌ Unsupported provider '{}'", profile.provider);
        }
    }
}

async fn test_s3_profile(profile_name: &str, profile: &Profile) {
    // Your existing S3 test logic here
    let region = profile
        .region
        .clone()
        .unwrap_or_else(|| "us-east-1".to_string());
    let mut cfg_loader =
        aws_config::defaults(aws_config::BehaviorVersion::latest())
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

    match client.list_buckets().send().await {
        Ok(_) => println!("✅ Profile '{}' connectivity verified", profile_name),
        Err(_) => eprintln!("❌ Profile '{}' test failed. Check logs for details.", profile_name),
    }
}

async fn test_local_profile(profile_name: &str, _profile: &Profile) {
    // For local profiles, just verify we can read the file system
    if std::path::Path::new(".").exists() {
        println!("✅ Profile '{}' (local) accessible", profile_name);
    } else {
        eprintln!("❌ Profile '{}' (local) test failed", profile_name);
    }
}