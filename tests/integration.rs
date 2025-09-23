
use data_quality::engine::validate_dataframe;
use data_quality::contracts::SchemaContracts;
use polars::prelude::*;

#[test]
fn end_to_end_validation() {
    // Build a tiny DataFrame
    let df = df![
        "id" => &[1, 2, 2],
        "region_id" => &[10, 20, 20]
    ].unwrap();

    // Minimal contract (could also load from TOML)
    let contract = SchemaContracts {
        name: "test".into(),
        version: "0.1.0".into(),
        file: None,
        columns: vec![],
        compound_unique: Some(vec![data_quality::contracts::CompoundUnique {
            columns: vec!["id".into(), "region_id".into()],
        }]),
    };

    // Run validation
    let result = validate_dataframe(&df, &contract);

    assert!(result.is_ok());
}
