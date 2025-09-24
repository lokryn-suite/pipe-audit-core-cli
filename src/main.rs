use clap::Parser;
use pipe_audit::cli::{Cli, Commands};
use pipe_audit::commands;
use pipe_audit::logging;

#[tokio::main]
async fn main() {
    logging::init_logging();

    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Validate { file }) => {
            commands::validate::run(&file).await;
        }
        Some(Commands::Run { all }) => {
            commands::run::run(all).await;
        }
        None => eprintln!("No command provided. Try --help."),
    }
}
