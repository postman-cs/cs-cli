mod launcher;

use cs_cli::gong::cli::run_cli;
use cs_cli::Result;
use owo_colors::OwoColorize;

#[tokio::main]
async fn main() -> Result<()> {
    // Check if we're in a terminal, self-launch if not
    launcher::ensure_terminal();

    // Initialize tracing for comprehensive async debugging
    // Use try_init to avoid panic if subscriber already set (e.g., when launched from GUI)
    let _ = tracing_subscriber::fmt::try_init();

    println!("{}", "CS-CLI - Initializing...".truecolor(255, 108, 55));
    run_cli().await
}
