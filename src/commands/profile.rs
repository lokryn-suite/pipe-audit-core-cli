use crate::engine::{list_profiles, test_profile};

pub async fn list() {
    match list_profiles() {
        Ok((profile_list, message)) => {
            println!("{}", message);
            if profile_list.profiles.is_empty() {
                // No additional output needed
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

pub async fn test(profile_name: &str) {
    let (_result, message) = test_profile(profile_name).await;
    println!("{}", message);
}
