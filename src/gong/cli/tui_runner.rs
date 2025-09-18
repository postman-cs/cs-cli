//! Gong-specific TUI runner that manages the complete extraction workflow
//!
//! This module integrates the common TUI app with Gong-specific extraction logic,
//! providing a seamless experience from customer selection to results display.

use crate::common::cli::args::{ContentType, ParsedCommand};
use crate::common::cli::tui_app::{draw_tui, ExtractionMessage, ExtractionResults, TuiApp};
use crate::gong::api::client::HttpClientPool;
use crate::gong::api::customer::GongCustomerSearchClient;
use crate::gong::auth::GongAuthenticator;
use crate::gong::cli::{load_config, save_config};
use crate::gong::config::AppConfig;
use crate::gong::extractor::TeamCallsExtractor;
use crate::Result;

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

    // Initialize app state - starts in Authenticating state
    let mut app = TuiApp::new();
    let mut last_input = String::new();
    let mut suggestion_provider: Option<SuggestionProvider> = None;
    let mut auth_task_running = true; // Track if authentication task is running

    // Channel for extraction progress and authentication
    let (extraction_tx, mut extraction_rx) = mpsc::unbounded_channel::<ExtractionMessage>();

    // Start authentication process
    let auth_tx = extraction_tx.clone();
    let auth_config = config.clone();
    tokio::spawn(async move {
        run_authentication(auth_config, auth_tx).await;
    });

    // Main event loop
    let result = loop {
        // Check for extraction messages
        while let Ok(msg) = extraction_rx.try_recv() {
            // Handle authentication success by creating suggestion provider
            if let ExtractionMessage::AuthSuccess = msg {
                auth_task_running = false; // Authentication completed successfully
                                           // Create suggestion provider now that we're authenticated
                match create_suggestion_provider(config.clone()).await {
                    Ok(provider) => {
                        suggestion_provider = Some(provider);
                        app.state = crate::common::cli::tui_app::AppState::CustomerSelection;
                    }
                    Err(e) => {
                        app.handle_extraction_message(ExtractionMessage::AuthFailed(format!(
                            "Failed to initialize search: {e}"
                        )));
                        continue;
                    }
                }
            }

            // Handle authentication failure
            if let ExtractionMessage::AuthFailed(_) = msg {
                auth_task_running = false; // Authentication task completed (with failure)
            }

            app.handle_extraction_message(msg);
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

        // Check if we need to start extraction
        if app.state == crate::common::cli::tui_app::AppState::Initializing {
            app.state = crate::common::cli::tui_app::AppState::Extracting;

            // Get the parsed command
            let command = app.get_parsed_command();

            // Start extraction in background
            let tx = extraction_tx.clone();
            let config_clone = config.clone();
            tokio::spawn(async move {
                run_extraction(command, config_clone, tx).await;
            });
        }

        // Draw UI
        terminal.draw(|f| draw_tui(f, &app))?;

        // Handle input
        if event::poll(Duration::from_millis(100))? {
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
                                auth_task_running = true;
                                app.state = crate::common::cli::tui_app::AppState::Authenticating;
                                app.auth_progress = 0.0;
                                app.auth_error = None;

                                let auth_tx = extraction_tx.clone();
                                let auth_config = config.clone();
                                tokio::spawn(async move {
                                    run_authentication(auth_config, auth_tx).await;
                                });
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
    // Initialize Gong API client
    let http = HttpClientPool::new_gong_pool(Some(config.http.clone())).await?;
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
    let search_client = Arc::new(GongCustomerSearchClient::new(
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

        // Create a new thread with its own runtime for the API call
        let handle = std::thread::spawn(move || {
            // Create a new runtime in this thread
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async move {
                match tokio::time::timeout(
                    Duration::from_millis(800),
                    client.search_customers(&input_str),
                )
                .await
                {
                    Ok(Ok(results)) => results.into_iter().map(|r| r.name).collect(),
                    _ => Vec::new(),
                }
            })
        });

        // Wait for the thread to complete with a timeout
        handle.join().unwrap_or_default()
    }))
}

/// Run authentication process with progress updates
async fn run_authentication(config: AppConfig, tx: mpsc::UnboundedSender<ExtractionMessage>) {
    // Send initial progress
    tx.send(ExtractionMessage::AuthProgress(
        0.1,
        "Initializing authentication system...".to_string(),
    ))
    .ok();

    // Initialize Gong authenticator
    let mut auth = match GongAuthenticator::new(config.auth.clone()).await {
        Ok(auth) => auth,
        Err(e) => {
            tx.send(ExtractionMessage::AuthFailed(format!(
                "Failed to initialize authenticator: {e}"
            )))
            .ok();
            return;
        }
    };

    tx.send(ExtractionMessage::AuthProgress(
        0.3,
        "Checking keychain access...".to_string(),
    ))
    .ok();

    // Ensure keychain access (this may prompt for password)
    use crate::common::auth::smart_keychain::SmartKeychainManager;
    if let Ok(keychain_manager) = SmartKeychainManager::new() {
        if let Err(e) = keychain_manager.ensure_keychain_access() {
            tx.send(ExtractionMessage::AuthFailed(format!(
                "Keychain access failed: {e}"
            )))
            .ok();
            return;
        }
    }

    tx.send(ExtractionMessage::AuthProgress(
        0.5,
        "Extracting browser cookies...".to_string(),
    ))
    .ok();

    // Check if already authenticated first
    if auth.is_authenticated() {
        tx.send(ExtractionMessage::AuthProgress(
            1.0,
            "Authentication successful!".to_string(),
        ))
        .ok();
        tx.send(ExtractionMessage::AuthSuccess).ok();
        return;
    }

    // Perform authentication
    match auth.authenticate().await {
        Ok(true) => {
            tx.send(ExtractionMessage::AuthProgress(
                1.0,
                "Authentication successful!".to_string(),
            ))
            .ok();
            tx.send(ExtractionMessage::AuthSuccess).ok();
        }
        Ok(false) => {
            tx.send(ExtractionMessage::AuthFailed(
                "Authentication failed. Please ensure you're logged into Gong in your browser."
                    .to_string(),
            ))
            .ok();
        }
        Err(e) => {
            tx.send(ExtractionMessage::AuthFailed(format!(
                "Authentication error: {e}"
            )))
            .ok();
        }
    }
}

/// Run the extraction process with progress updates
async fn run_extraction(
    command: ParsedCommand,
    config: AppConfig,
    tx: mpsc::UnboundedSender<ExtractionMessage>,
) {
    // Initialize extractor with quiet mode enabled (no console output)
    let config_clone = config.clone();
    let mut extractor = TeamCallsExtractor::new(config);
    extractor.set_quiet(true); // Suppress console output
    let mut cli_config = load_config();

    // Setup phase - skip authentication since it's already done
    tx.send(ExtractionMessage::Phase(
        "Initializing extraction system...".to_string(),
    ))
    .ok();
    tx.send(ExtractionMessage::Progress(0.1)).ok();

    // Setup without authentication (since TUI already authenticated)
    match setup_extractor_without_auth(&mut extractor, config_clone).await {
        Ok(_) => {
            tx.send(ExtractionMessage::SubTask(
                "Extraction system ready".to_string(),
            ))
            .ok();
            tx.send(ExtractionMessage::Progress(0.2)).ok();
        }
        Err(e) => {
            tx.send(ExtractionMessage::Error(format!("Setup failed: {e}")))
                .ok();
            return;
        }
    }

    // Extract based on command type
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
                tx.send(ExtractionMessage::Phase(phase)).ok();

                let base_progress = 0.2 + (0.7 * idx as f64 / names.len() as f64);
                tx.send(ExtractionMessage::Progress(base_progress)).ok();

                // Extract communications
                tx.send(ExtractionMessage::SubTask(format!(
                    "Searching for {customer_name}"
                )))
                .ok();

                let (calls, emails, resolved_name) = match extractor
                    .extract_customer_communications(
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
                        tx.send(ExtractionMessage::SubTask(format!(
                            "Failed to extract for {customer_name}: {e}"
                        )))
                        .ok();
                        continue;
                    }
                };

                tx.send(ExtractionMessage::CallsFound(calls.len())).ok();
                tx.send(ExtractionMessage::EmailsFound(emails.len())).ok();

                total_calls += calls.len();
                total_emails += emails.len();

                // Save results
                if !calls.is_empty() && !emails_only {
                    tx.send(ExtractionMessage::SubTask(
                        "Saving call transcripts...".to_string(),
                    ))
                    .ok();

                    match extractor.save_calls_as_markdown_with_resolved_name(
                        &calls,
                        Some(&first_customer),
                        Some(&resolved_name),
                    ) {
                        Ok(files) => {
                            for file in &files {
                                if let Some(name) = file.file_name() {
                                    tx.send(ExtractionMessage::FileSaved(
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
                            tx.send(ExtractionMessage::SubTask(format!(
                                "Failed to save calls: {e}"
                            )))
                            .ok();
                        }
                    }
                }

                if !emails.is_empty() {
                    tx.send(ExtractionMessage::SubTask("Saving emails...".to_string()))
                        .ok();

                    match extractor.save_emails_as_markdown(&emails, &first_customer) {
                        Ok(files) => {
                            for file in &files {
                                if let Some(name) = file.file_name() {
                                    tx.send(ExtractionMessage::FileSaved(
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
                            tx.send(ExtractionMessage::SubTask(format!(
                                "Failed to save emails: {e}"
                            )))
                            .ok();
                        }
                    }
                }

                let progress = 0.2 + (0.7 * (idx + 1) as f64 / names.len() as f64);
                tx.send(ExtractionMessage::Progress(progress)).ok();
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
                tx.send(ExtractionMessage::Phase(phase)).ok();

                let base_progress = 0.2 + (0.7 * idx as f64 / names.len() as f64);
                tx.send(ExtractionMessage::Progress(base_progress)).ok();

                // Extract communications
                tx.send(ExtractionMessage::SubTask(format!(
                    "Searching for {customer_name}"
                )))
                .ok();

                let (calls, emails, resolved_name) = match extractor
                    .extract_customer_communications(
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
                        tx.send(ExtractionMessage::SubTask(format!(
                            "Failed to extract for {customer_name}: {e}"
                        )))
                        .ok();
                        continue;
                    }
                };

                tx.send(ExtractionMessage::CallsFound(calls.len())).ok();
                tx.send(ExtractionMessage::EmailsFound(emails.len())).ok();

                total_calls += calls.len();
                total_emails += emails.len();

                // Save results
                if !calls.is_empty() && !emails_only {
                    tx.send(ExtractionMessage::SubTask(
                        "Saving call transcripts...".to_string(),
                    ))
                    .ok();

                    match extractor.save_calls_as_markdown_with_resolved_name(
                        &calls,
                        Some(&first_customer),
                        Some(&resolved_name),
                    ) {
                        Ok(files) => {
                            for file in &files {
                                if let Some(name) = file.file_name() {
                                    tx.send(ExtractionMessage::FileSaved(
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
                            tx.send(ExtractionMessage::SubTask(format!(
                                "Failed to save calls: {e}"
                            )))
                            .ok();
                        }
                    }
                }

                if !emails.is_empty() {
                    tx.send(ExtractionMessage::SubTask("Saving emails...".to_string()))
                        .ok();

                    match extractor.save_emails_as_markdown(&emails, &first_customer) {
                        Ok(files) => {
                            for file in &files {
                                if let Some(name) = file.file_name() {
                                    tx.send(ExtractionMessage::FileSaved(
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
                            tx.send(ExtractionMessage::SubTask(format!(
                                "Failed to save emails: {e}"
                            )))
                            .ok();
                        }
                    }
                }

                let progress = 0.2 + (0.7 * (idx + 1) as f64 / names.len() as f64);
                tx.send(ExtractionMessage::Progress(progress)).ok();
            }
        }

        ParsedCommand::Team {
            stream_id, days, ..
        } => {
            tx.send(ExtractionMessage::Phase(
                "Extracting team calls...".to_string(),
            ))
            .ok();

            let stream_id = match stream_id {
                Some(id) => id,
                None => {
                    tx.send(ExtractionMessage::Error(
                        "No stream ID provided".to_string(),
                    ))
                    .ok();
                    return;
                }
            };

            let days = days.unwrap_or(7);

            match extractor
                .extract_team_calls(&stream_id, Some(days), None, None)
                .await
            {
                Ok(calls) => {
                    tx.send(ExtractionMessage::CallsFound(calls.len())).ok();
                    total_calls = calls.len();

                    if !calls.is_empty() {
                        tx.send(ExtractionMessage::SubTask(
                            "Saving team calls...".to_string(),
                        ))
                        .ok();

                        match extractor.save_calls_as_markdown_with_resolved_name(
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
                                tx.send(ExtractionMessage::Error(format!(
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
                    tx.send(ExtractionMessage::Error(format!(
                        "Failed to extract team calls: {e}"
                    )))
                    .ok();
                    return;
                }
            }
        }
        _ => {}
    }

    // Cleanup
    tx.send(ExtractionMessage::Phase("Finalizing...".to_string()))
        .ok();
    extractor.cleanup().await;

    // Send completion
    tx.send(ExtractionMessage::Progress(1.0)).ok();
    tx.send(ExtractionMessage::Complete(ExtractionResults {
        total_calls,
        total_emails,
        files_saved: saved_files.len(),
        output_directory: output_dir,
    }))
    .ok();
}

/// Setup extractor without authentication (assumes auth is already done)
async fn setup_extractor_without_auth(
    extractor: &mut TeamCallsExtractor,
    config: AppConfig,
) -> Result<()> {
    // Initialize Gong HTTP client and auth
    let http = HttpClientPool::new_gong_pool(Some(config.http.clone())).await?;
    let mut auth = GongAuthenticator::new(config.auth.clone()).await?;

    // Re-authenticate to get session cookies (this should be fast since cookies are cached)
    let authenticated = auth.authenticate().await?;
    if !authenticated {
        return Err(crate::common::error::types::CsCliError::Authentication(
            "Failed to re-authenticate for extraction setup".to_string(),
        ));
    }

    // Set cookies
    if let Ok(session_cookies) = auth.get_session_cookies() {
        http.set_cookies(session_cookies).await?;
    }

    // Create Arc references for sharing
    let http_arc = Arc::new(http);
    let auth_arc = Arc::new(auth);

    // Initialize the extractor components using the new method
    extractor.setup_with_auth(http_arc, auth_arc).await?;

    Ok(())
}
