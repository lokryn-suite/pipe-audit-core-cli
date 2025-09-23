mod cli;
mod contracts;
mod runner;
mod commands;
mod validators;

use clap::Parser;
use cli::{Cli, Commands};

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Validate { file }) => {
            commands::validate::run(&file);
        }
        Some(Commands::Run { all }) => {
            commands::run::run(all);
        }
        None => eprintln!("No command provided. Try --help."),
    }
}
