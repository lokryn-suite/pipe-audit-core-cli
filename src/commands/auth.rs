use crate::profiles::load_profiles;
use aws_sdk_s3::config::Credentials;

pub async fn list() {
    match load_profiles() {
        Ok(profiles) => {
            println!("Available profiles:");
            for name in profiles.keys() {
                println!("  - {}", name);
            }
        }
        Err(e) => eprintln!("❌ Failed to load profiles: {e}"),
    }
}

pub async fn test(profile_name: &str) {
    match load_profiles() {
        Ok(profiles) => {
            if let Some(profile) = profiles.get(profile_name) {
                if profile.provider == "s3" {
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
                        Err(e) => eprintln!("❌ Profile '{}' test failed: {e}", profile_name),
                    }
                } else {
                    eprintln!("❌ Unsupported provider '{}'", profile.provider);
                }
            } else {
                eprintln!("❌ Profile '{}' not found", profile_name);
            }
        }
        Err(e) => eprintln!("❌ Failed to load profiles: {e}"),
    }
}
