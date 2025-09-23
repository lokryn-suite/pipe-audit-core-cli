use clap::{Parser, Subcommand};

/// My Program: a structured CLI example
#[derive(Parser, Debug)]
#[command(name = "data-quality")]
#[command(about = "Data quality engine CLI", long_about = None)]
pub struct Cli {
    /// Activate verbose mode
    #[arg(short, long)]
    pub verbose: bool,

    /// Subcommands like `init`, `run`, etc.
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Initialize something
    Validate {
        /// Optional name
        file: String,
    },
    /// Run contracts against data files
    Run {
        /// Run all contracts in the data folder
        #[arg(long)]
        all: bool,
    }
}
