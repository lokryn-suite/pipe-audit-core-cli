use clap::Parser;
use pipe_audit::cli::{AuthCommands, Cli, Commands};
use pipe_audit::commands;
use pipe_audit::logging;

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();

    logging::init_logging();

    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Validate { file }) => {
            commands::validate::run(&file).await;
        }
        Some(Commands::Run { all }) => {
            commands::run::run(all).await;
        }
        Some(Commands::Auth { auth_command }) => match auth_command {
            AuthCommands::List => {
                commands::auth::list().await;
            }
            AuthCommands::Test { profile } => {
                commands::auth::test(&profile).await;
            }
        },
        None => eprintln!("No command provided. Try --help."),
    }
}
