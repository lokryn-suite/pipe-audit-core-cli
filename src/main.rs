
use clap::Parser;
use data_quality::cli::{Cli, Commands};
use data_quality::logging;
use data_quality::commands;

fn main() {
    logging::init_logging();

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
