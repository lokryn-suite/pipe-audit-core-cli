use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct CompoundUnique {
    pub columns: Vec<String>,
}
