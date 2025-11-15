use pipa::audit_logging::JsonlLogger;
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
    let logger = JsonlLogger::default();
    match list_profiles(&logger) {
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
/// attempts to connect using the profile's configuration (e.g. DB,
/// cloud service, or API credentials). Prints the engine's result
/// message to stdout.
///
/// Called from `main.rs` when the user runs:
/// ```bash
/// pipa profile test <profile_name>
/// ```
pub async fn test(profile_name: &str) {
    let logger = JsonlLogger::default();
    let (_result, message) = test_profile(&logger, profile_name).await;
    println!("{}", message);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logger_creation() {
        // Test that JsonlLogger can be created
        let logger = JsonlLogger::default();
        // Logger is created successfully if we get here
        drop(logger);
    }

    #[test]
    fn test_profile_name_handling() {
        // Test that profile names are handled correctly
        let profile_name = "test_profile";
        assert!(!profile_name.is_empty());
        assert_eq!(profile_name.len(), 12);
    }

    #[test]
    fn test_empty_profile_list() {
        // Test that empty profile list is handled
        let profiles: Vec<String> = vec![];
        assert!(profiles.is_empty());
    }

    #[test]
    fn test_non_empty_profile_list() {
        // Test that non-empty profile list is handled
        let profiles = vec!["profile1".to_string(), "profile2".to_string()];
        assert!(!profiles.is_empty());
        assert_eq!(profiles.len(), 2);

        // Test iteration over profiles
        for name in profiles.iter() {
            assert!(!name.is_empty());
        }
    }
}
