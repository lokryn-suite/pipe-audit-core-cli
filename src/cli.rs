use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "pipa")]
#[command(about = "Data quality engine CLI", long_about = None)]
pub struct Cli {
    #[arg(short, long)]
    pub verbose: bool,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Run data validation against contracts
    Run {
        /// Contract name (without .toml extension)
        contract: Option<String>,
        /// Run all contracts
        #[arg(long)]
        all: bool,
    },
    /// Manage contracts
    Contract {
        #[command(subcommand)]
        contract_command: ContractCommands,
    },
    /// Manage profiles
    Profile {
        #[command(subcommand)]
        profile_command: ProfileCommands,
    },
    /// System health check
    Health,
    /// Log management
    Logs {
        #[command(subcommand)]
        logs_command: LogsCommands,
    },
}

#[derive(Subcommand, Debug)]
pub enum ContractCommands {
    /// List available contracts
    List,
    /// Validate contract TOML syntax
    Validate {
        /// Contract file name
        file: String,
    },
    /// Show contract details
    Show {
        /// Contract name
        name: String,
    },
}

#[derive(Subcommand, Debug)]
pub enum ProfileCommands {
    /// List available profiles
    List,
    /// Test profile connectivity
    Test {
        /// Profile name
        profile: String,
    },
}

#[derive(Subcommand, Debug)]
pub enum LogsCommands {
    /// Verify log integrity
    Verify {
        /// Date to verify (YYYY-MM-DD format)
        #[arg(long)]
        date: Option<String>,
    },
}
