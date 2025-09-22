// src/contracts.rs
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(tag = "rule")] // tells Serde to look at "rule" in TOML
pub enum Contract {
    Unique { column: String },
    NotNull { column: String },
    Range { column: String, min: i64, max: i64 },
    Pattern { column: String, pattern: String },
}
