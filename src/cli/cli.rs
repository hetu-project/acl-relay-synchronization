use super::migrate_cmd::MigrateCmd;
use super::run_cmd::RunCmd;
use crate::common::consts;
use clap::{Parser, Subcommand};

/// Main CLI structure
#[derive(Parser, Debug)]
#[command(version = consts::CLI_VERSION, about = "A simple CLI app with config and help commands", long_about = None)]
struct Cli {
    /// Subcommands for the application
    #[command(subcommand)]
    command: Option<Commands>,
}

/// Enum defining the subcommands
#[derive(Subcommand, Debug)]
enum Commands {
    /// run server
    Run(RunCmd),

    /// database migration
    Migrate(MigrateCmd),
}

/// CLI processing logic
/// This function encapsulates both parsing and command handling.
pub async fn handle_cli() {
    // Parse the CLI arguments
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Run(cmd)) => {
            cmd.run().await;
        }
        Some(Commands::Migrate(cmd)) => {
            cmd.run().await;
        }
        None => {
            panic!("need subcommand, use '--help' to get usage of subcommands")
        }
    }
}
