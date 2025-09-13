use cs_cli::cli::run_cli;
use cs_cli::Result;
#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing for comprehensive async debugging
    tracing_subscriber::fmt::init();

    println!("CS-CLI Rust - Initializing...");
    run_cli().await
}
