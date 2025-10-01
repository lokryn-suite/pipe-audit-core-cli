use crate::logging::schema::Executor;
use glob::glob;
use hostname;
use pipa::engine::contracts::run_contract_validation;
use std::path::Path;
use whoami;

pub async fn run_all() {
    let hostname = hostname::get()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    let executor = Executor {
        user: whoami::username(),
        host: hostname,
    };

    for entry in glob("contracts/*.toml").expect("Failed to read glob pattern") {
        match entry {
            Ok(path) => {
                let contract_name = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("unknown");

                match run_contract_validation(contract_name, &executor, true).await {
                    Ok((outcome, message)) => {
                        println!("{}", message);
                        if !outcome.passed {
                            eprintln!(
                                "⚠️  Validation completed with failures for {}",
                                contract_name
                            );
                        }
                    }
                    Err(_) => {
                        eprintln!(
                            "❌ Validation failed for {}. Check logs for details.",
                            contract_name
                        );
                    }
                }
            }
            Err(_) => eprintln!("❌ Error reading contract files. Check logs for details."),
        }
    }
}

pub async fn run_single(contract_name: &str) {
    let hostname = hostname::get()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    let executor = Executor {
        user: whoami::username(),
        host: hostname,
    };

    if !Path::new(&format!("contracts/{}.toml", contract_name)).exists() {
        eprintln!(
            "❌ Contract '{}' not found. Use 'pipa contract list' to see available contracts.",
            contract_name
        );
        return;
    }

    match run_contract_validation(contract_name, &executor, true).await {
        Ok((outcome, message)) => {
            println!("{}", message);
            if !outcome.passed {
                eprintln!(
                    "⚠️  Validation completed with {} failures out of {} checks",
                    outcome.fail_count,
                    outcome.pass_count + outcome.fail_count
                );
            }
        }
        Err(_) => {
            eprintln!(
                "❌ Validation failed for {}. Check logs for details.",
                contract_name
            );
        }
    }
}
