use pipa::profile::{list_profiles, test_profile};

/// List all available profiles in the project.
///
/// Delegates to `pipa::profile::list_profiles()`, which queries
/// the engine for configured profiles. Prints a summary message
/// and each profile name to stdout.
///
/// Called from `main.rs` when the user runs:
/// ```bash
/// pipa profile list
/// ```
pub async fn list() {
    match list_profiles() {
        Ok((profile_list, message)) => {
            // Print engine-provided summary message
            println!("{}", message);

            // Print each profile name if any exist
            if profile_list.profiles.is_empty() {
                // No profiles found — nothing else to print
            } else {
                for name in profile_list.profiles.iter() {
                    println!("  - {}", name);
                }
            }
        }
        Err(_) => {
            eprintln!("❌ Failed to load profiles. Check logs for details.");
        }
    }
}

/// Test connectivity for a specific profile by name.
///
/// Delegates to `pipa::profile::test_profile(profile_name)`, which
/// attempts to connect using the profile’s configuration (e.g. DB,
/// cloud service, or API credentials). Prints the engine’s result
/// message to stdout.
///
/// Called from `main.rs` when the user runs:
/// ```bash
/// pipa profile test <profile_name>
/// ```
pub async fn test(profile_name: &str) {
    let (_result, message) = test_profile(profile_name).await;
    println!("{}", message);
}
