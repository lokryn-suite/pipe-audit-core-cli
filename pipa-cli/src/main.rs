use clap::Parser;              // CLI argument parsing
use pipa::contract;            // Public API surface (contracts)
use pipa::profile;             // Public API surface (profiles)
use pipa::run;                 // Public API surface (runner)
use pipa::logs;                // Public API surface (logs)
use pipa::health;              // Public API surface (health checks)
use pipa::init;                // Public API surface (project init)

mod cli;                       // Local CLI definitions (structs/enums)
mod commands;                  // Local command implementations

use cli::{Cli, Commands, ContractCommands, LogsCommands, ProfileCommands};


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    
    dotenv::dotenv().ok();         // Load environment variables from .env
    logging::init_logging();       // Initialize logging system
    let cli = Cli::parse();        // Parse CLI args into `Cli` struct


    match cli.command {
        Some(Commands::Run { contract, all }) => {
            if all && contract.is_some() {
                eprintln!("❌ Cannot specify both contract name and --all");
                std::process::exit(1);
            }
            if !all && contract.is_none() {
                eprintln!("❌ Must specify either contract name or --all");
                std::process::exit(1);
            }

            if all {
                commands::run::run_all().await;
            } else if let Some(name) = contract {
                commands::run::run_single(&name).await;
            }
        }
        Some(Commands::Contract { contract_command }) => match contract_command {
            ContractCommands::List => commands::contract::list().await,
            ContractCommands::Validate { file } => commands::contract::validate(&file).await,
            ContractCommands::Show { name } => commands::contract::show(&name).await,
        },
        Some(Commands::Profile { profile_command }) => match profile_command {
            ProfileCommands::List => commands::profile::list().await,
            ProfileCommands::Test { profile } => commands::profile::test(&profile).await,
        },
        Some(Commands::Health) => commands::health::run().await,

        Some(Commands::Logs { logs_command }) => match logs_command {
            LogsCommands::Verify { date, all } => {
                commands::logs::verify(date.as_deref(), all).await;
            }
        },
        Some(Commands::Init) => commands::init::init_project().await,
        None => {
            println!("No command specified. Use --help for usage information.");
        }
    }

    Ok(())
}
