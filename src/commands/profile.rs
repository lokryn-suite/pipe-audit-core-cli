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
            "local" => true, // Local always works if profile exists
            "azure" => test_azure_profile_internal(profile).await,
            "gcs" => test_gcs_profile_internal(profile).await,
            "sftp" => false, // Not implemented yet
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

    // Handle optional credentials
    if let (Some(access_key), Some(secret_key)) = (&profile.access_key, &profile.secret_key) {
        if !access_key.is_empty() && !secret_key.is_empty() {
            let creds = Credentials::new(
                access_key.clone(),
                secret_key.clone(),
                None,
                None,
                "profile",
            );
            s3b = s3b.credentials_provider(creds);
        }
    }

    let client = aws_sdk_s3::Client::from_conf(s3b.build());
    client.list_buckets().send().await.is_ok()
}

async fn test_azure_profile_internal(profile: &Profile) -> bool {
    // For now, only support connection string method
    if let Some(connection_string) = &profile.connection_string {
        return test_azure_connection_string(connection_string).await;
    }

    false
}

async fn test_azure_connection_string(connection_string: &str) -> bool {
    use base64::{engine::general_purpose, Engine as _};
    use hmac::{Hmac, Mac};
    use sha2::Sha256;

    let (account_name, account_key) = match parse_azure_connection_string(connection_string) {
        Ok((name, key)) => (name, key),
        Err(_) => return false,
    };

    let client = reqwest::Client::new();
    let url = format!("https://{}.blob.core.windows.net/?comp=list", account_name);
    let date = chrono::Utc::now()
        .format("%a, %d %b %Y %H:%M:%S GMT")
        .to_string();

    // Create the string to sign for Azure Storage authentication
    let string_to_sign = format!(
        "GET\n\n\n\n\n\n\n\n\n\n\n\nx-ms-date:{}\nx-ms-version:2020-04-08\n/{}/\ncomp:list",
        date, account_name
    );

    // Create HMAC signature
    let key_bytes = match general_purpose::STANDARD.decode(&account_key) {
        Ok(bytes) => bytes,
        Err(_) => return false,
    };

    let mut mac = match Hmac::<Sha256>::new_from_slice(&key_bytes) {
        Ok(mac) => mac,
        Err(_) => return false,
    };

    mac.update(string_to_sign.as_bytes());
    let signature = general_purpose::STANDARD.encode(mac.finalize().into_bytes());
    let auth_header = format!("SharedKey {}:{}", account_name, signature);

    match client
        .get(&url)
        .header("Authorization", auth_header)
        .header("x-ms-date", date)
        .header("x-ms-version", "2020-04-08")
        .send()
        .await
    {
        Ok(response) => {
            println!("Debug: Azure auth test response: {}", response.status());
            response.status().is_success()
        }
        Err(e) => {
            println!("Debug: Azure connection failed: {}", e);
            false
        }
    }
}

fn parse_azure_connection_string(
    connection_string: &str,
) -> Result<(String, String), Box<dyn std::error::Error>> {
    let mut account_name = None;
    let mut account_key = None;

    for part in connection_string.split(';') {
        if let Some(name) = part.strip_prefix("AccountName=") {
            account_name = Some(name.to_string());
        } else if let Some(key) = part.strip_prefix("AccountKey=") {
            account_key = Some(key.to_string());
        }
    }

    match (account_name, account_key) {
        (Some(name), Some(key)) => Ok((name, key)),
        _ => Err("Invalid connection string format".into()),
    }
}

async fn test_gcs_profile_internal(profile: &Profile) -> bool {
    if let Some(service_account_json) = &profile.service_account_json {
        return test_gcs_service_account(service_account_json).await;
    }
    false
}

fn parse_gcs_service_account(
    service_account_json: &str,
) -> Result<(String, String, String), Box<dyn std::error::Error>> {
    use serde_json::Value;

    let json: Value = serde_json::from_str(service_account_json)?;

    let project_id = json["project_id"]
        .as_str()
        .ok_or("Missing project_id in service account JSON")?
        .to_string();

    let client_email = json["client_email"]
        .as_str()
        .ok_or("Missing client_email in service account JSON")?
        .to_string();

    let private_key = json["private_key"]
        .as_str()
        .ok_or("Missing private_key in service account JSON")?
        .to_string();

    Ok((project_id, client_email, private_key))
}

async fn test_gcs_service_account(service_account_json: &str) -> bool {
    println!(
        "Debug: GCS service account JSON length: {}",
        service_account_json.len()
    );
    println!(
        "Debug: GCS service account JSON first 100 chars: {}",
        &service_account_json.chars().take(100).collect::<String>()
    );
    use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
    use serde_json::json;

    let (project_id, client_email, private_key) =
        match parse_gcs_service_account(service_account_json) {
            Ok((pid, email, key)) => (pid, email, key),
            Err(e) => {
                println!("Debug: GCS service account parsing failed: {}", e);
                return false;
            }
        };

    // Create JWT claims
    let now = chrono::Utc::now().timestamp();
    let claims = json!({
        "iss": client_email,
        "scope": "https://www.googleapis.com/auth/cloud-platform",
        "aud": "https://oauth2.googleapis.com/token",
        "exp": now + 3600, // 1 hour
        "iat": now
    });

    // Generate JWT token
    let header = Header::new(Algorithm::RS256);
    let encoding_key = match EncodingKey::from_rsa_pem(private_key.as_bytes()) {
        Ok(key) => key,
        Err(e) => {
            println!("Debug: GCS private key parsing failed: {}", e);
            return false;
        }
    };

    let jwt_token = match encode(&header, &claims, &encoding_key) {
        Ok(token) => token,
        Err(e) => {
            println!("Debug: GCS JWT generation failed: {}", e);
            return false;
        }
    };

    println!("Debug: GCS JWT generated successfully");

    // Exchange JWT for access token
    let client = reqwest::Client::new();

    let token_response = match client
        .post("https://oauth2.googleapis.com/token")
        .header("Content-Type", "application/x-www-form-urlencoded")
        .form(&[
            ("grant_type", "urn:ietf:params:oauth:grant-type:jwt-bearer"),
            ("assertion", &jwt_token),
        ])
        .send()
        .await
    {
        Ok(response) => response,
        Err(e) => {
            println!("Debug: GCS token exchange request failed: {}", e);
            return false;
        }
    };

    if !token_response.status().is_success() {
        println!(
            "Debug: GCS token exchange failed: {}",
            token_response.status()
        );
        return false;
    }

    let access_token = match token_response.json::<serde_json::Value>().await {
        Ok(json) => match json["access_token"].as_str() {
            Some(token) => token.to_string(),
            None => {
                println!("Debug: GCS access_token not found in response");
                return false;
            }
        },
        Err(e) => {
            println!("Debug: GCS token response parsing failed: {}", e);
            return false;
        }
    };

    // Test bucket list API
    let bucket_list_url = format!(
        "https://storage.googleapis.com/storage/v1/b?project={}",
        project_id
    );

    match client
        .get(&bucket_list_url)
        .header("Authorization", format!("Bearer {}", access_token))
        .send()
        .await
    {
        Ok(response) => {
            println!("Debug: GCS bucket list response: {}", response.status());
            response.status().is_success()
        }
        Err(e) => {
            println!("Debug: GCS bucket list failed: {}", e);
            false
        }
    }
}
