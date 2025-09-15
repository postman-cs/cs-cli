mod launcher;

use cs_cli::gong::cli::run_cli;
use cs_cli::common::self_update;
use cs_cli::Result;
use owo_colors::OwoColorize;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "cs-cli")]
#[command(about = "CS-CLI: Customer Success Deep Research Tool", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Self-update operations
    #[command(name = "self")]
    SelfUpdate {
        #[command(subcommand)]
        command: SelfCommands,
    },
}

#[derive(Subcommand)]
enum SelfCommands {
    /// Check for and install updates
    Update {
        #[arg(long, help = "Update channel (stable, beta)")]
        channel: Option<String>,
        #[arg(long, help = "Skip confirmation prompts")]
        assume_yes: bool,
        #[arg(long, help = "Run in background mode (minimal output)")]
        background: bool,
    },
    /// Roll back to the previous version
    Rollback,
    /// List installed versions
    Versions,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Check if we're in a terminal, self-launch if not
    launcher::ensure_terminal();

    // Initialize tracing for comprehensive async debugging
    let _ = tracing_subscriber::fmt::try_init();

    // Parse CLI arguments
    let cli = Cli::parse();

    // Handle self-update commands first (before Gong initialization)
    if let Some(Commands::SelfUpdate { command }) = cli.command {
        match command {
            SelfCommands::Update { channel, assume_yes, background } => {
                return self_update::run(channel, assume_yes, background).await
                    .map_err(|e| cs_cli::CsCliError::UpdateError(e.to_string()));
            }
            SelfCommands::Rollback => {
                return self_update::rollback()
                    .map_err(|e| cs_cli::CsCliError::UpdateError(e.to_string()));
            }
            SelfCommands::Versions => {
                return self_update::list_versions()
                    .map_err(|e| cs_cli::CsCliError::UpdateError(e.to_string()));
            }
        }
    }

    // Default to Gong CLI if no self-update command
    println!("{}", "CS-CLI - Initializing...".truecolor(255, 108, 55));
    run_cli().await
}
