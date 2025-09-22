mod contracts;
mod validators;
mod engine;
mod output;
mod cli;
use polars::prelude::*;

fn main() -> PolarsResult<()> {
    // Build options
    let options = CsvReadOptions::default()
        .with_has_header(true)                // first row is header
        .with_infer_schema_length(Some(100)); // peek 100 rows for schema inference

    // Create a reader from file path with options
    let df = options
        .try_into_reader_with_file_path(Some("data/regions.csv".into()))?
        .finish()?;

    println!("{:?}", df.head(Some(5)));
    Ok(())
}



