use clap::{Parser, Subcommand};

/// Root CLI parser for the `pipa` data quality engine.
///
/// This struct defines the top-level CLI interface. It is parsed
/// automatically by `clap` when `Cli::parse()` is called in `main.rs`.
#[derive(Parser, Debug)]
#[command(name = "pipa")]
#[command(
    about = "Pipe Audit Data Quality CLI",
    long_about = "Pipa validates data contracts, manages profiles, verifies logs, \
                  and checks system health. Use it to ensure your data pipelines \
                  meet quality standards and are fully auditable."
)]
pub struct Cli {
    /// Increase output verbosity (-v / --verbose).
    /// This flag can be checked in `main.rs` or logging setup
    /// to adjust log levels globally.
    #[arg(short, long)]
    pub verbose: bool,

    /// The top-level command to execute.
    /// If no command is provided, `main.rs` will print a help message.
    #[command(subcommand)]
    pub command: Option<Commands>,
}

/// Top-level CLI commands.
///
/// Each variant corresponds to a subcommand handled in `main.rs`,
/// which then delegates to the appropriate function in `commands::*`.
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Run data validation against contracts.
    ///
    /// Either a single contract can be specified by name,
    /// or `--all` can be used to run every contract.
    Run {
        /// Contract name (without `.toml` extension).
        contract: Option<String>,

        /// Run all contracts in the project.
        #[arg(long)]
        all: bool,
    },

    /// Manage contracts (list, validate, show).
    Contract {
        #[command(subcommand)]
        contract_command: ContractCommands,
    },

    /// Manage profiles (list, test connectivity).
    Profile {
        #[command(subcommand)]
        profile_command: ProfileCommands,
    },

    /// Run a system health check.
    ///
    /// This typically verifies environment setup, connectors,
    /// and other prerequisites.
    Health,

    /// Manage logs (verify integrity).
    Logs {
        #[command(subcommand)]
        logs_command: LogsCommands,
    },

    /// Initialize a new project in the current directory.
    Init,
}

/// Contract-related subcommands.
///
/// These are dispatched from `Commands::Contract` in `main.rs`.
#[derive(Subcommand, Debug)]
pub enum ContractCommands {
    /// List available contracts in the project.
    List,

    /// Validate contract TOML syntax.
    ///
    /// Ensures the file is well-formed and can be parsed.
    Validate {
        /// Contract file name (e.g. `my_contract.toml`).
        file: String,
    },

    /// Show contract details by name.
    ///
    /// Prints metadata and validation rules for inspection.
    Show {
        /// Contract name (without `.toml` extension).
        name: String,
    },
}

/// Profile-related subcommands.
///
/// These are dispatched from `Commands::Profile` in `main.rs`.
#[derive(Subcommand, Debug)]
pub enum ProfileCommands {
    /// List available profiles.
    List,

    /// Test profile connectivity.
    ///
    /// Useful for verifying credentials and endpoints.
    Test {
        /// Profile name to test.
        profile: String,
    },
}

/// Log-related subcommands.
///
/// These are dispatched from `Commands::Logs` in `main.rs`.
#[derive(Subcommand, Debug)]
pub enum LogsCommands {
    /// Verify log integrity.
    ///
    /// Can check a specific date or all sealed logs.
    Verify {
        /// Date to verify (YYYY-MM-DD format).
        #[arg(long)]
        date: Option<String>,

        /// Verify all sealed logs.
        #[arg(long)]
        all: bool,
    },
}
