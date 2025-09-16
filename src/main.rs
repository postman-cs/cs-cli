mod launcher;

use cs_cli::gong::cli::run_cli;
use cs_cli::Result;
use owo_colors::OwoColorize;
use clap::Parser;

#[derive(Parser)]
#[command(name = "cs-cli")]
#[command(about = "CS-CLI: Customer Success Deep Research Tool", long_about = None)]
#[command(version)]
struct Cli {}

#[tokio::main]
async fn main() -> Result<()> {
    // Check for updates first (silent fail if network issues)
    let _ = cs_cli::updater::check_and_update().await;

    // Check if we're in a terminal, self-launch if not
    launcher::ensure_terminal();

    // Initialize tracing for comprehensive async debugging
    let _ = tracing_subscriber::fmt::try_init();

    // Parse CLI arguments
    let _cli = Cli::parse();

    // Run Gong CLI
    println!("{}", "CS-CLI - Initializing...".truecolor(255, 108, 55));
    run_cli().await
}
