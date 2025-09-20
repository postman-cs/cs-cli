//! Gong-specific TUI runner that manages the complete retrieval workflow
//!
//! This module integrates the common TUI app with Gong-specific retrieval logic,
//! providing a seamless experience from customer selection to results display.

use crate::common::auth::hybrid_cookie_storage::set_sync_preference;
use crate::common::cli::args::{ContentType, ParsedCommand};
use crate::common::cli::tui_app::{draw_tui, RetrievalMessage, RetrievalResults, TuiApp};
use crate::gong::api::client::HttpClientPool;
use crate::gong::api::customer::GongCustomerSearchRetriever;
use crate::gong::auth::GongAuthenticator;
use crate::gong::cli::{load_config, save_config};
use crate::gong::config::AppConfig;
use crate::gong::retriever::TeamCallsRetriever;
use crate::{Result, CsCliError};

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use tracing::{info, warn};

/// Type alias for customer suggestion provider function
type SuggestionProvider = Box<dyn Fn(&str) -> Vec<String> + Send>;

/// Run the complete Gong TUI experience
pub async fn run_gong_tui(config: AppConfig) -> Result<ParsedCommand> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Initialize app state - check if this is first run for sync choice
    let mut app = TuiApp::new();
    let mut last_input = String::new();
    let mut suggestion_provider: Option<SuggestionProvider> = None;
    let mut auth_task_running = false; // Start as false, will be set when auth starts

    // Channel for retrieval progress and authentication
    let (retrieval_tx, mut retrieval_rx) = mpsc::unbounded_channel::<RetrievalMessage>();

    // Allow the TUI to start with its default state (Authenticating)
    // This ensures the proper authentication animation sequence is shown

    // Check if user has previously made a sync choice
    let has_existing_preference = match get_sync_preference_for_check().await {
        Ok(preference) => {
            app.sync_enabled = preference;
            app.sync_choice_made = true; // Skip choice if already made previously
            true
        }
        Err(_) => {
            // First run - user needs to choose
            app.sync_enabled = false; // Default to local-only
            app.sync_choice_made = false; // Require user choice
            false
        }
    };

    // Only show choice UI on first run
    if has_existing_preference {
        // Authentication will start automatically since choice is already made
    } else {
        // Authentication will wait for user choice
    }

    // Main event loop
    let result = loop {
        // Check for retrieval messages
        while let Ok(msg) = retrieval_rx.try_recv() {
            // Handle authentication success by creating suggestion provider
            if let RetrievalMessage::AuthSuccess = msg {
                auth_task_running = false; // Authentication completed successfully
                                           // Create suggestion provider now that we're authenticated
                match create_suggestion_provider(config.clone()).await {
                    Ok(provider) => {
                        suggestion_provider = Some(provider);
                        app.state = crate::common::cli::tui_app::AppState::CustomerSelection;
                    }
                    Err(e) => {
                        app.handle_retrieval_message(RetrievalMessage::AuthFailed(format!(
                            "Failed to initialize search: {e}"
                        )));
                        continue;
                    }
                }
            }

            // Handle authentication failure
            if let RetrievalMessage::AuthFailed(_) = msg {
                auth_task_running = false; // Authentication task completed (with failure)
            }

            app.handle_retrieval_message(msg);
        }

        // Start authentication only after user makes storage choice
        if app.state == crate::common::cli::tui_app::AppState::Authenticating
            && !auth_task_running
            && app.sync_choice_made
        {
            // Save current sync preference
            if let Err(e) = set_sync_preference(app.sync_enabled).await {
                eprintln!("Failed to save sync preference: {e}");
            }

            // Clear any invalid stored session data before starting
            if let Err(e) = clear_invalid_session_data().await {
                eprintln!("Failed to clear invalid session data: {e}");
            }

            if !auth_task_running {
                auth_task_running = true;
                let auth_tx = retrieval_tx.clone();
                let auth_config = config.clone();
                tokio::spawn(async move {
                    run_authentication(auth_config, auth_tx).await;
                });
            }
        }

        // Handle authentication mode toggle (save preference when user changes it)
        if app.sync_enabled != get_current_sync_preference().await {
            if let Err(e) = set_sync_preference(app.sync_enabled).await {
                eprintln!("Failed to save sync preference: {e}");
            }
        }

        // Update suggestions if in customer selection
        if app.state == crate::common::cli::tui_app::AppState::CustomerSelection {
            if let Some(ref provider) = suggestion_provider {
                if app.input != last_input && app.input.len() >= 2 {
                    let suggestions = provider(&app.input);
                    app.update_suggestions(suggestions);
                    last_input = app.input.clone();
                } else if app.input.len() < 2 {
                    app.update_suggestions(Vec::new());
                }
            }
        }

        // Check if we need to start retreival
        if app.state == crate::common::cli::tui_app::AppState::Initializing {
            app.state = crate::common::cli::tui_app::AppState::Retrieving;

            // Get the parsed command
            let command = app.get_parsed_command();

            // Start retreival in background
            let tx = retrieval_tx.clone();
            let config_clone = config.clone();
            tokio::spawn(async move {
                run_retrieval(command, config_clone, tx).await;
            });
        }

        // Update animations before drawing
        app.update_animations();

        // Only redraw if animation is dirty or we have other state changes
        if app.animation_dirty || app.state != crate::common::cli::tui_app::AppState::Retrieving {
            terminal.draw(|f| draw_tui(f, &mut app))?;
            app.animation_dirty = false; // Reset dirty flag after drawing
        }

        // Handle input with proper frame timing for smooth animations
        // Target 60 FPS = ~16.67ms per frame
        let frame_duration = Duration::from_millis(16);
        
        if event::poll(frame_duration)? {
            match event::read()? {
                Event::Key(key) => {
                    if key.kind == KeyEventKind::Press {
                        // Handle authentication retry before normal input processing
                        if matches!(
                            app.state,
                            crate::common::cli::tui_app::AppState::AuthenticationFailed(_)
                        ) {
                            if matches!(key.code, KeyCode::Char('r') | KeyCode::Enter)
                                && !auth_task_running
                            {
                                // Start new authentication task only if none is running
                                if !auth_task_running {
                                    auth_task_running = true;
                                    app.state = crate::common::cli::tui_app::AppState::Authenticating;
                                    app.auth_progress = 0.0;
                                    app.auth_error = None;

                                    let auth_tx = retrieval_tx.clone();
                                    let auth_config = config.clone();
                                    tokio::spawn(async move {
                                        run_authentication(auth_config, auth_tx).await;
                                    });
                                }
                                continue; // Skip normal input processing
                            } else if key.code == KeyCode::Esc {
                                break Ok(app.get_parsed_command());
                            }
                        }

                        if app.handle_input(key.code) {
                            // Exit requested
                            break Ok(app.get_parsed_command());
                        }
                    }
                }
                Event::Mouse(mouse) => {
                    if app.handle_mouse(mouse) {
                        // Exit requested
                        break Ok(app.get_parsed_command());
                    }
                }
                _ => {}
            }
        }
    };

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    result
}

/// Create a Gong suggestion provider (assumes authentication is already done)
async fn create_suggestion_provider(config: AppConfig) -> Result<SuggestionProvider> {
    // Initialize Gong API client with minimal pool for suggestion provider
    let mut http_config = config.http.clone();
    http_config.pool_size = 2; // Minimal pool for suggestion provider
    http_config.max_concurrency_per_client = 5; // Reduced concurrency
    let http = HttpClientPool::new_gong_pool(Some(http_config)).await?;
    let mut auth = GongAuthenticator::new(config.auth.clone()).await?;

    // Re-authenticate to get session cookies (this should be fast since cookies are cached)
    let authenticated = auth.authenticate().await?;
    if !authenticated {
        return Err(crate::common::error::types::CsCliError::Authentication(
            "Authentication failed when creating search provider".to_string(),
        ));
    }

    // Set cookies
    if let Ok(session_cookies) = auth.get_session_cookies() {
        http.set_cookies(session_cookies).await?;
    }

    // Create search client
    let search_client = Arc::new(GongCustomerSearchRetriever::new(
        Arc::new(http),
        Arc::new(auth),
        Some(config),
    )?);

    Ok(Box::new(move |input: &str| {
        if input.len() < 2 {
            return Vec::new();
        }

        let client = Arc::clone(&search_client);
        let input_str = input.to_string();

        // Use the existing async context instead of creating a nested runtime
        let search_future = async move {
            match tokio::time::timeout(
                Duration::from_millis(800),
                client.search_customers(&input_str),
            )
            .await
            {
                Ok(Ok(results)) => results.into_iter().map(|r| r.name).collect(),
                _ => Vec::new(),
            }
        };

        // Since we're in a sync context but the caller is async, we need to spawn
        // the task on the current runtime instead of creating a new one
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(search_future)
        })
    }))
}

/// Create initial GitHub gist for sync
async fn create_initial_gist(access_token: &str) -> Result<()> {
    use octocrab::Octocrab;
    use crate::common::auth::github::{GIST_DESCRIPTION, GIST_FILENAME};
    
    // Create GitHub client
    let client = Octocrab::builder()
        .personal_token(access_token.to_string())
        .build()
        .map_err(|e| CsCliError::GitHubOAuth(format!("Failed to create GitHub client: {}", e)))?;

    // Create initial empty gist
    let gist = client
        .gists()
        .create()
        .description(GIST_DESCRIPTION)
        .public(false) // Private gist
        .file(GIST_FILENAME, "# CS-CLI Session Data\n\nThis gist stores encrypted session data for cross-device synchronization.\n\nCreated: ")
        .send()
        .await
        .map_err(|e| CsCliError::GitHubOAuth(format!("Failed to create gist: {}", e)))?;

    tracing::info!("Created initial gist with ID: {}", gist.id);

    // Save gist configuration
    use crate::common::auth::github::gist_config_manager::GistConfigManager;
    
    let config_manager = GistConfigManager::new()
        .map_err(|e| CsCliError::GitHubOAuth(format!("Failed to create config manager: {}", e)))?;
    
    // Get GitHub user info
    let user = client.current().user().await
        .map_err(|e| CsCliError::GitHubOAuth(format!("Failed to get user info: {}", e)))?;
    
    // Create config with token hash
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(access_token.as_bytes());
    let token_hash = format!("{:x}", hasher.finalize());
    
    let config = crate::common::auth::github::gist_config_manager::GistConfig::new(
        gist.id,
        user.login,
        token_hash,
    );
    
    config_manager.save(&config)
        .map_err(|e| CsCliError::GitHubOAuth(format!("Failed to save gist config: {}", e)))?;

    Ok(())
}

/// Run authentication process with progress updates
async fn run_authentication(config: AppConfig, tx: mpsc::UnboundedSender<RetrievalMessage>) {
    // Send initial progress - step 1
    tx.send(RetrievalMessage::AuthProgress(
        0.1,
        "Initializing authentication system...".to_string(),
    ))
    .ok();

    // Check if GitHub sync is enabled and perform OAuth first if needed
    if get_current_sync_preference().await {
        tx.send(RetrievalMessage::AuthProgress(
            0.15,
            "Preparing GitHub sync...".to_string(),
        ))
        .ok();

        // Check if we already have a valid GitHub token (consolidated check)
        use std::process::Command;
        let token_check = Command::new("security")
            .args([
                "find-generic-password",
                "-s",
                "com.postman.cs-cli.github-token",
                "-a",
                "oauth-access-token",
                "-w", // Return password only
            ])
            .output();

        let (needs_oauth, _has_valid_gist_config) = match token_check {
            Ok(output) if output.status.success() => {
                let token = String::from_utf8(output.stdout)
                    .map(|s| s.trim().to_string())
                    .unwrap_or_default();

                tracing::debug!("Found GitHub token in keychain, length: {}", token.len());
                
                if token.is_empty() {
                    tracing::info!("GitHub token is empty, need OAuth");
                    (true, false) // Empty token, need OAuth, no valid gist config
                } else {
                    // We have a token, verify it's still valid by trying to create a client
                    use octocrab::Octocrab;
                    match Octocrab::builder().personal_token(token.clone()).build() {
                        Ok(client) => {
                            // Try a simple API call to verify token validity
                            match client.current().user().await {
                                Ok(_) => {
                                    // Token is valid, check if we also have gist config
                                    use crate::common::auth::github::gist_config_manager::GistConfigManager;
                                    let has_gist_config = if let Ok(config_manager) = GistConfigManager::new() {
                                        config_manager.load().map(|opt| opt.is_some()).unwrap_or(false)
                                    } else {
                                        false
                                    };

                                    if has_gist_config {
                                        tx.send(RetrievalMessage::AuthProgress(
                                            0.3,
                                            "GitHub sync already configured, continuing...".to_string(),
                                        ))
                                        .ok();
                                        tracing::info!("Valid GitHub token and gist config found, skipping OAuth");
                                        return; // Early return - completely skip OAuth
                                    } else {
                                        tracing::info!("Valid GitHub token found but no gist config, will create gist");
                                        (false, false) // Token is valid but need to create gist
                                    }
                                }
                                Err(e) => {
                                    tracing::warn!(
                                        "GitHub token validation failed: {}, will re-authenticate",
                                        e
                                    );
                                    (true, false) // Token is invalid, need OAuth
                                }
                            }
                        }
                        Err(_) => (true, false), // Failed to create client, need OAuth
                    }
                }
            }
            Ok(output) => {
                tracing::info!("GitHub token check failed with status: {}", output.status);
                let stderr = String::from_utf8_lossy(&output.stderr);
                tracing::info!("GitHub token check stderr: {}", stderr);
                (true, false) // No token found or error, need OAuth
            }
            Err(e) => {
                tracing::info!("GitHub token check command failed: {}", e);
                (true, false) // No token found or error, need OAuth
            }
        };

        // Clean up stale gist config if token was invalid
        if needs_oauth {
            use crate::common::auth::github::gist_config_manager::GistConfigManager;
            if let Ok(config_manager) = GistConfigManager::new() {
                if let Ok(Some(_)) = config_manager.load() {
                    tracing::info!("Removing stale gist configuration due to invalid token");
                    if let Err(e) = config_manager.remove() {
                        tracing::warn!("Failed to remove stale gist config: {}", e);
                    }
                }
            }
        }

        if needs_oauth {
            tx.send(RetrievalMessage::AuthProgress(
                0.2,
                "Setting up GitHub sync...".to_string(),
            ))
            .ok();

            // Initialize GitHub OAuth flow and get token
            use crate::common::auth::github_oauth_flow::GitHubOAuthFlow;

            let mut oauth_flow = match GitHubOAuthFlow::new() {
                Ok(flow) => flow,
                Err(e) => {
                    tx.send(RetrievalMessage::AuthProgress(
                        0.0,
                        format!("Failed to initialize GitHub OAuth: {}", e),
                    ))
                    .ok();
                    return;
                }
            };
            match oauth_flow.authenticate().await {
                Ok(access_token) => {
                    tx.send(RetrievalMessage::AuthProgress(
                        0.3,
                        "GitHub authorization successful, storing token...".to_string(),
                    ))
                    .ok();

                    // Store the GitHub token in keychain for later use
                    use std::process::Command;
                    let output = Command::new("security")
                        .args([
                            "add-generic-password",
                            "-s",
                            "com.postman.cs-cli.github-token",
                            "-a",
                            "oauth-access-token",
                            "-w",
                            &access_token,
                            "-U", // Update if exists
                        ])
                        .output();

                    match output {
                        Ok(out) if out.status.success() => {
                            tracing::info!("GitHub token stored successfully in keychain");
                            tx.send(RetrievalMessage::AuthProgress(
                                0.4,
                                "Creating GitHub gist for sync...".to_string(),
                            ))
                            .ok();

                            // Create initial gist for sync
                            if let Err(e) = create_initial_gist(&access_token).await {
                                tracing::warn!("Failed to create initial gist: {}", e);
                                // Continue anyway - gist creation failure is not fatal
                            } else {
                                tracing::info!("Initial GitHub gist created successfully");
                            }

                            tx.send(RetrievalMessage::AuthProgress(
                                0.5,
                                "GitHub sync ready, continuing with authentication...".to_string(),
                            ))
                            .ok();
                        }
                        Ok(out) => {
                            let stderr = String::from_utf8_lossy(&out.stderr);
                            tracing::warn!("Failed to store GitHub token: {}", stderr);
                            // Continue anyway - token storage failure is not fatal
                        }
                        Err(e) => {
                            tracing::warn!("Failed to run security command: {}", e);
                            // Continue anyway - token storage failure is not fatal
                        }
                    }
                }
                Err(e) => {
                    tx.send(RetrievalMessage::AuthFailed(format!(
                        "GitHub authorization failed: {e}. Continuing with local storage only."
                    )))
                    .ok();
                    // Continue with authentication even if GitHub OAuth fails
                    // User can still use local storage
                }
            }
        }
    }

    // Initialize Gong authenticator
    let mut auth = match GongAuthenticator::new(config.auth.clone()).await {
        Ok(auth) => auth,
        Err(e) => {
            tx.send(RetrievalMessage::AuthFailed(format!(
                "Failed to initialize authenticator: {e}"
            )))
            .ok();
            return;
        }
    };

    tx.send(RetrievalMessage::AuthProgress(
        0.5,
        "Retreiving browser sessions...".to_string(),
    ))
    .ok();

    // Check if already authenticated first
    if auth.is_authenticated() {
        tx.send(RetrievalMessage::AuthProgress(
            1.0,
            "Authentication successful!".to_string(),
        ))
        .ok();
        tx.send(RetrievalMessage::AuthSuccess).ok();
        return;
    }

    // Perform authentication
    match auth.authenticate().await {
        Ok(true) => {
            tx.send(RetrievalMessage::AuthProgress(
                1.0,
                "Authentication successful!".to_string(),
            ))
            .ok();
            tx.send(RetrievalMessage::AuthSuccess).ok();
        }
        Ok(false) => {
            tx.send(RetrievalMessage::AuthFailed(
                "Authentication failed. Please ensure you're logged into Gong in your browser."
                    .to_string(),
            ))
            .ok();
        }
        Err(e) => {
            tx.send(RetrievalMessage::AuthFailed(format!(
                "Authentication error: {e}"
            )))
            .ok();
        }
    }
}

/// Run the retrieval process with progress updates
async fn run_retrieval(
    command: ParsedCommand,
    config: AppConfig,
    tx: mpsc::UnboundedSender<RetrievalMessage>,
) {
    // Initialize retriever with quiet mode enabled (no console output)
    let config_clone = config.clone();
    let mut retriever = TeamCallsRetriever::new(config);
    retriever.set_quiet(true); // Suppress console output
    let mut cli_config = load_config();

    // Setup phase - skip authentication since it's already done
    tx.send(RetrievalMessage::Phase(
        "Initializing retrieval system...".to_string(),
    ))
    .ok();
    tx.send(RetrievalMessage::Progress(0.1)).ok();

    // Setup without authentication (since TUI already authenticated)
    match setup_retriever_without_auth(&mut retriever, config_clone).await {
        Ok(_) => {
            tx.send(RetrievalMessage::SubTask(
                "Retrieval system ready".to_string(),
            ))
            .ok();
            tx.send(RetrievalMessage::Progress(0.2)).ok();
        }
        Err(e) => {
            tx.send(RetrievalMessage::Error(format!("Setup failed: {e}")))
                .ok();
            return;
        }
    }

    // Retreive based on command type
    let mut total_calls = 0;
    let mut total_emails = 0;
    let mut saved_files = Vec::new();
    let mut output_dir = String::new();

    match command {
        ParsedCommand::Customer {
            ref name,
            days,
            content_type,
            emails_only,
            fetch_email_bodies,
            ..
        } => {
            let names = [name.clone()];

            let days = days.unwrap_or(90);
            let first_customer = names[0].clone();

            // Process each customer
            for (idx, customer_name) in names.iter().enumerate() {
                let phase = format!(
                    "Processing customer {}/{}: {}",
                    idx + 1,
                    names.len(),
                    customer_name
                );
                tx.send(RetrievalMessage::Phase(phase)).ok();

                let base_progress = 0.2 + (0.7 * idx as f64 / names.len() as f64);
                tx.send(RetrievalMessage::Progress(base_progress)).ok();

                // Retreive communications
                tx.send(RetrievalMessage::SubTask(format!(
                    "Searching for {customer_name}"
                )))
                .ok();

                let (calls, emails, resolved_name) = match retriever
                    .retrieve_customer_communications(
                        customer_name,
                        days,
                        matches!(content_type, ContentType::Both | ContentType::Emails),
                        emails_only,
                        fetch_email_bodies,
                    )
                    .await
                {
                    Ok(result) => result,
                    Err(e) => {
                        tx.send(RetrievalMessage::SubTask(format!(
                            "Failed to retrieve for {customer_name}: {e}"
                        )))
                        .ok();
                        continue;
                    }
                };

                tx.send(RetrievalMessage::CallsFound(calls.len())).ok();
                tx.send(RetrievalMessage::EmailsFound(emails.len())).ok();

                total_calls += calls.len();
                total_emails += emails.len();

                // Save results
                if !calls.is_empty() && !emails_only {
                    tx.send(RetrievalMessage::SubTask(
                        "Saving call transcripts...".to_string(),
                    ))
                    .ok();

                    match retriever.save_calls_as_markdown_with_resolved_name(
                        &calls,
                        Some(&first_customer),
                        Some(&resolved_name),
                    ) {
                        Ok(files) => {
                            for file in &files {
                                if let Some(name) = file.file_name() {
                                    tx.send(RetrievalMessage::FileSaved(
                                        name.to_string_lossy().to_string(),
                                    ))
                                    .ok();
                                }
                            }
                            if let Some(parent) = files.first().and_then(|f| f.parent()) {
                                output_dir = parent.display().to_string();
                            }
                            saved_files.extend(files);
                        }
                        Err(e) => {
                            tx.send(RetrievalMessage::SubTask(format!(
                                "Failed to save calls: {e}"
                            )))
                            .ok();
                        }
                    }
                }

                if !emails.is_empty() {
                    tx.send(RetrievalMessage::SubTask("Saving emails...".to_string()))
                        .ok();

                    match retriever.save_emails_as_markdown(&emails, &first_customer) {
                        Ok(files) => {
                            for file in &files {
                                if let Some(name) = file.file_name() {
                                    tx.send(RetrievalMessage::FileSaved(
                                        name.to_string_lossy().to_string(),
                                    ))
                                    .ok();
                                }
                            }
                            if output_dir.is_empty() {
                                if let Some(parent) = files.first().and_then(|f| f.parent()) {
                                    output_dir = parent.display().to_string();
                                }
                            }
                            saved_files.extend(files);
                        }
                        Err(e) => {
                            tx.send(RetrievalMessage::SubTask(format!(
                                "Failed to save emails: {e}"
                            )))
                            .ok();
                        }
                    }
                }

                let progress = 0.2 + (0.7 * (idx + 1) as f64 / names.len() as f64);
                tx.send(RetrievalMessage::Progress(progress)).ok();
            }
        }

        ParsedCommand::MultipleCustomers {
            ref names,
            days,
            content_type,
            emails_only,
            fetch_email_bodies,
            ..
        } => {
            let days = days.unwrap_or(90);
            let first_customer = names[0].clone();

            // Process each customer
            for (idx, customer_name) in names.iter().enumerate() {
                let phase = format!(
                    "Processing customer {}/{}: {}",
                    idx + 1,
                    names.len(),
                    customer_name
                );
                tx.send(RetrievalMessage::Phase(phase)).ok();

                let base_progress = 0.2 + (0.7 * idx as f64 / names.len() as f64);
                tx.send(RetrievalMessage::Progress(base_progress)).ok();

                // Retreive communications
                tx.send(RetrievalMessage::SubTask(format!(
                    "Searching for {customer_name}"
                )))
                .ok();

                let (calls, emails, resolved_name) = match retriever
                    .retrieve_customer_communications(
                        customer_name,
                        days,
                        matches!(content_type, ContentType::Both | ContentType::Emails),
                        emails_only,
                        fetch_email_bodies,
                    )
                    .await
                {
                    Ok(result) => result,
                    Err(e) => {
                        tx.send(RetrievalMessage::SubTask(format!(
                            "Failed to retrieve for {customer_name}: {e}"
                        )))
                        .ok();
                        continue;
                    }
                };

                tx.send(RetrievalMessage::CallsFound(calls.len())).ok();
                tx.send(RetrievalMessage::EmailsFound(emails.len())).ok();

                total_calls += calls.len();
                total_emails += emails.len();

                // Save results
                if !calls.is_empty() && !emails_only {
                    tx.send(RetrievalMessage::SubTask(
                        "Saving call transcripts...".to_string(),
                    ))
                    .ok();

                    match retriever.save_calls_as_markdown_with_resolved_name(
                        &calls,
                        Some(&first_customer),
                        Some(&resolved_name),
                    ) {
                        Ok(files) => {
                            for file in &files {
                                if let Some(name) = file.file_name() {
                                    tx.send(RetrievalMessage::FileSaved(
                                        name.to_string_lossy().to_string(),
                                    ))
                                    .ok();
                                }
                            }
                            if let Some(parent) = files.first().and_then(|f| f.parent()) {
                                output_dir = parent.display().to_string();
                            }
                            saved_files.extend(files);
                        }
                        Err(e) => {
                            tx.send(RetrievalMessage::SubTask(format!(
                                "Failed to save calls: {e}"
                            )))
                            .ok();
                        }
                    }
                }

                if !emails.is_empty() {
                    tx.send(RetrievalMessage::SubTask("Saving emails...".to_string()))
                        .ok();

                    match retriever.save_emails_as_markdown(&emails, &first_customer) {
                        Ok(files) => {
                            for file in &files {
                                if let Some(name) = file.file_name() {
                                    tx.send(RetrievalMessage::FileSaved(
                                        name.to_string_lossy().to_string(),
                                    ))
                                    .ok();
                                }
                            }
                            if output_dir.is_empty() {
                                if let Some(parent) = files.first().and_then(|f| f.parent()) {
                                    output_dir = parent.display().to_string();
                                }
                            }
                            saved_files.extend(files);
                        }
                        Err(e) => {
                            tx.send(RetrievalMessage::SubTask(format!(
                                "Failed to save emails: {e}"
                            )))
                            .ok();
                        }
                    }
                }

                let progress = 0.2 + (0.7 * (idx + 1) as f64 / names.len() as f64);
                tx.send(RetrievalMessage::Progress(progress)).ok();
            }
        }

        ParsedCommand::Team {
            stream_id, days, ..
        } => {
            tx.send(RetrievalMessage::Phase(
                "Retrieving team calls...".to_string(),
            ))
            .ok();

            let stream_id = match stream_id {
                Some(id) => id,
                None => {
                    tx.send(RetrievalMessage::Error(
                        "No stream ID provided".to_string(),
                    ))
                    .ok();
                    return;
                }
            };

            let days = days.unwrap_or(7);

            match retriever
                .retrieve_team_calls(&stream_id, Some(days), None, None)
                .await
            {
                Ok(calls) => {
                    tx.send(RetrievalMessage::CallsFound(calls.len())).ok();
                    total_calls = calls.len();

                    if !calls.is_empty() {
                        tx.send(RetrievalMessage::SubTask(
                            "Saving team calls...".to_string(),
                        ))
                        .ok();

                        match retriever.save_calls_as_markdown_with_resolved_name(
                            &calls,
                            Some("Team"),
                            Some("Team"),
                        ) {
                            Ok(files) => {
                                if let Some(parent) = files.first().and_then(|f| f.parent()) {
                                    output_dir = parent.display().to_string();
                                }
                                saved_files.extend(files);
                            }
                            Err(e) => {
                                tx.send(RetrievalMessage::Error(format!(
                                    "Failed to save calls: {e}"
                                )))
                                .ok();
                                return;
                            }
                        }
                    }

                    // Save config
                    cli_config.team_call_stream_id = Some(stream_id);
                    save_config(&cli_config).ok();
                }
                Err(e) => {
                    tx.send(RetrievalMessage::Error(format!(
                        "Failed to retrieve team calls: {e}"
                    )))
                    .ok();
                    return;
                }
            }
        }
        _ => {}
    }

    // Cleanup
    retriever.cleanup().await;

    // Send completion
    tx.send(RetrievalMessage::Progress(1.0)).ok();
    tx.send(RetrievalMessage::Complete(RetrievalResults {
        total_calls,
        total_emails,
        files_saved: saved_files.len(),
        output_directory: output_dir,
    }))
    .ok();
}

/// Setup retriever without authentication (assumes auth is already done)
async fn setup_retriever_without_auth(
    retriever: &mut TeamCallsRetriever,
    config: AppConfig,
) -> Result<()> {
    // Initialize Gong HTTP client and auth with minimal pool for authentication
    let mut http_config = config.http.clone();
    http_config.pool_size = 2; // Minimal pool for authentication
    http_config.max_concurrency_per_client = 5; // Reduced concurrency
    let http = HttpClientPool::new_gong_pool(Some(http_config)).await?;
    let mut auth = GongAuthenticator::new(config.auth.clone()).await?;

    // Re-authenticate to get session cookies (this should be fast since cookies are cached)
    let authenticated = auth.authenticate().await?;
    if !authenticated {
        return Err(crate::common::error::types::CsCliError::Authentication(
            "Failed to re-authenticate for retrieval setup".to_string(),
        ));
    }

    // Set cookies
    if let Ok(session_cookies) = auth.get_session_cookies() {
        http.set_cookies(session_cookies).await?;
    }

    // Create Arc references for sharing
    let http_arc = Arc::new(http);
    let auth_arc = Arc::new(auth);

    // Initialize the retriever components using the new method
    retriever.setup_with_auth(http_arc, auth_arc).await?;

    Ok(())
}
/// Get current sync preference
async fn get_current_sync_preference() -> bool {
    use dirs;

    let config_dir = match dirs::config_dir() {
        Some(dir) => dir.join("cs-cli"),
        None => return false,
    };

    let preference_path = config_dir.join("sync-preference");
    if preference_path.exists() {
        if let Ok(content) = std::fs::read_to_string(preference_path) {
            return content.trim() == "enabled";
        }
    }
    false
}

/// Check if sync preference exists and return it, or error if no preference set
async fn get_sync_preference_for_check() -> Result<bool> {
    use dirs;

    let config_dir = match dirs::config_dir() {
        Some(dir) => dir.join("cs-cli"),
        None => return Err(crate::common::error::types::CsCliError::Configuration(
            "Unable to determine config directory".to_string(),
        )),
    };

    let preference_path = config_dir.join("sync-preference");
    if preference_path.exists() {
        let content = std::fs::read_to_string(preference_path)
            .map_err(|e| crate::common::error::types::CsCliError::Configuration(
                format!("Failed to read sync preference: {e}")
            ))?;
        Ok(content.trim() == "enabled")
    } else {
        Err(crate::common::error::types::CsCliError::Configuration(
            "No sync preference found - first run".to_string(),
        ))
    }
}

/// Clear invalid session data to force fresh authentication
async fn clear_invalid_session_data() -> Result<()> {
    use crate::common::auth::hybrid_cookie_storage::delete_cookies_hybrid;

    info!("Clearing invalid session data before retry...");
    match delete_cookies_hybrid().await {
        Ok(()) => {
            info!("Successfully cleared invalid session data");
            Ok(())
        }
        Err(e) => {
            warn!("Failed to clear session data: {}", e);
            // Non-fatal - continue with retry anyway
            Ok(())
        }
    }
}
