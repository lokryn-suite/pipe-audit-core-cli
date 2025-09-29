//! Resource limits for free tier vs cloud
use crate::error::ValidationResult;
use std::env;

#[derive(Debug, Clone)]
pub struct Limits {
    pub max_contracts: usize,
    pub max_profiles: usize,
    pub max_file_size_bytes: usize,
    pub log_retention_days: u32,
}

impl Limits {
    /// Load limits from environment variables
    pub fn from_env() -> Self {
        Self {
            max_contracts: env::var("MAX_CONTRACTS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(usize::MAX), // No limit by default
            max_profiles: env::var("MAX_PROFILES")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(usize::MAX),
            max_file_size_bytes: env::var("MAX_FILE_SIZE_MB")
                .ok()
                .and_then(|v| v.parse::<usize>().ok())
                .map(|mb| mb * 1024 * 1024)
                .unwrap_or(usize::MAX),
            log_retention_days: env::var("LOG_RETENTION_DAYS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(u32::MAX),
        }
    }

    /// Free tier limits
    pub fn free_tier() -> Self {
        Self {
            max_contracts: 10,
            max_profiles: 3,
            max_file_size_bytes: 100 * 1024 * 1024, // 100MB
            log_retention_days: 30,
        }
    }

    pub fn check_file_size(&self, size: usize) -> ValidationResult<()> {
        if size > self.max_file_size_bytes {
            Err(crate::error::ValidationError::FileTooLarge {
                size,
                max: self.max_file_size_bytes,
            })
        } else {
            Ok(())
        }
    }
}