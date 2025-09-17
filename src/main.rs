mod launcher;

use cs_cli::gong::cli::run_cli;
use cs_cli::Result;
use owo_colors::OwoColorize;

#[tokio::main]
async fn main() -> Result<()> {
    // Check for updates first (silent fail if network issues)
    let _ = cs_cli::updater::check_and_update().await;

    // Check if we're in a terminal, self-launch if not
    launcher::ensure_terminal();

    // Initialize tracing for comprehensive async debugging
    let _ = tracing_subscriber::fmt::try_init();

    // Run Gong CLI (handles its own argument parsing)
    println!("{}", "CS-CLI - Initializing...".truecolor(255, 108, 55));
    run_cli().await
}
