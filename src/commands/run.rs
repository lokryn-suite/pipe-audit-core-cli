use glob::glob;
use crate::runner;

/// Run all validations
pub fn run(all: bool) {
    if all {
        // Find all CSV files in data/
        for entry in glob("data/*.csv").expect("Failed to read glob pattern") {
            match entry {
                Ok(path) => {
                    let file = path.to_string_lossy().to_string();
                    println!("🔎 Validating {}", file);
                    if let Err(e) = runner::validate_file(&file) {
                        eprintln!("❌ Validation failed for {}: {e}", file);
                    }
                }
                Err(e) => eprintln!("❌ Error reading file: {e}"),
            }
        }
    } else {
        eprintln!("No run mode specified. Try `--all`.");
    }
}
