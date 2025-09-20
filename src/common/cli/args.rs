//! Platform-agnostic command line argument parsing and processing
//!
//! This module provides common argument structures that can be extended
//! by platform-specific implementations.

use clap::{Args, Parser, Subcommand};
use clap_complete::Shell;
use std::fmt;

/// CS-CLI: Customer Success Communication Extraction Tool
///
/// Extract customer communications from various platforms and save as markdown files.
/// Run without arguments for interactive mode!
#[derive(Parser, Debug, Clone)]
#[command(
    name = "cs-cli",
    version,
    about = "Extract customer communications from various platforms with cross-device session sync",
    long_about = "Extract customer communications from various platforms and save as markdown files.

CROSS-DEVICE SYNC:
    Session data is securely synced across devices via encrypted GitHub gists.
    You'll only need to authenticate once across all your devices.

EXAMPLES:
    cs-cli                              Interactive mode
    cs-cli customer Postman 30          Get last 30 days of Postman
    cs-cli customer Wells Fargo calls 90    Get last 90 days of Wells Fargo calls
    cs-cli customer emails 7-11 365     Get last 365 days of 7-Eleven emails
    cs-cli customer \"Fortune 500\" 30 calls emails    Get calls and emails
    cs-cli team                         Get last 7 days of team calls
    cs-cli team 30                      Get last 30 days of team calls
    
SYNC MANAGEMENT:
    cs-cli --sync-status                Show current sync status
    cs-cli --reset-sync                 Reset sync and re-authenticate
    cs-cli --local-only                 Disable sync for this run

KEYCHAIN (macOS):
    cs-cli --keychain-password=yourpass customer Postman    Provide password via CLI
    cs-cli customer Postman                                 Will prompt for password automatically"
)]
#[derive(Default)]
pub struct CliArgs {
    /// Enable debug logging
    #[arg(long, help = "Enable debug logging")]
    pub debug: bool,

    /// macOS keychain password (optional, will prompt if needed)
    #[arg(long, help = "macOS keychain password for browser cookie access")]
    pub keychain_password: Option<String>,

    /// Force disable cross-device sync (local-only mode)
    #[arg(long, help = "Disable cross-device sync and use local storage only")]
    pub local_only: bool,

    /// Reset sync configuration and re-authenticate with GitHub
    #[arg(long, help = "Reset cross-device sync and re-authenticate")]
    pub reset_sync: bool,

    /// Show current sync status
    #[arg(long, help = "Display current cross-device sync status")]
    pub sync_status: bool,

    /// Subcommand or flexible arguments
    #[command(subcommand)]
    pub command: Option<Command>,

    /// Flexible arguments for backwards compatibility (when no subcommand provided)
    #[arg(trailing_var_arg = true, allow_hyphen_values = true, hide = true)]
    pub raw_args: Vec<String>,
}

#[derive(Subcommand, Debug, Clone)]
pub enum Command {
    /// Extract customer communications
    Customer(CustomerArgs),
    /// Extract team calls from call stream
    Team(TeamArgs),
    /// Generate shell completion scripts
    Completion(CompletionArgs),
}

#[derive(Args, Debug, Clone)]
pub struct CompletionArgs {
    /// Shell to generate completion for
    #[arg(value_enum)]
    pub shell: Shell,
}

#[derive(Args, Debug, Clone)]
pub struct CustomerArgs {
    /// Customer name to search for
    #[arg(help = "Customer/company name to search for")]
    pub name: String,

    /// Number of days back to search
    #[arg(
        short = 'd',
        long = "days",
        help = "Number of days back to search (default: 90)"
    )]
    pub days: Option<u32>,

    /// Start date (YYYY-MM-DD format)
    #[arg(long = "from", help = "Start date (YYYY-MM-DD format)")]
    pub from_date: Option<String>,

    /// End date (YYYY-MM-DD format)
    #[arg(long = "to", help = "End date (YYYY-MM-DD format)")]
    pub to_date: Option<String>,

    /// Content type to extract
    #[arg(short = 'c', long = "content", value_enum, help = "What to extract")]
    pub content: Option<ContentType>,

    /// Extract only emails
    #[arg(long = "emails-only", help = "Extract only emails (no calls)")]
    pub emails_only: bool,

    /// Fetch full email body content
    #[arg(long = "fetch-bodies", help = "Fetch full email body content")]
    pub fetch_email_bodies: bool,
}

#[derive(Args, Debug, Clone)]
pub struct TeamArgs {
    /// Call stream ID (optional, will prompt if not provided)
    #[arg(help = "Call stream ID from the platform")]
    pub stream_id: Option<String>,

    /// Number of days back to search
    #[arg(
        short = 'd',
        long = "days",
        help = "Number of days back to search (default: 7)"
    )]
    pub days: Option<u32>,

    /// Start date (YYYY-MM-DD format)
    #[arg(long = "from", help = "Start date (YYYY-MM-DD format)")]
    pub from_date: Option<String>,

    /// End date (YYYY-MM-DD format)
    #[arg(long = "to", help = "End date (YYYY-MM-DD format)")]
    pub to_date: Option<String>,
}

/// Content type to extract
#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum ContentType {
    /// Extract only calls
    Calls,
    /// Extract only emails
    Emails,
    /// Extract both calls and emails
    Both,
}

impl fmt::Display for ContentType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ContentType::Calls => write!(f, "calls"),
            ContentType::Emails => write!(f, "emails"),
            ContentType::Both => write!(f, "calls and emails"),
        }
    }
}

/// Parsed command with smart detection
#[derive(Debug, Clone)]
pub enum ParsedCommand {
    /// Customer extraction mode
    Customer {
        name: String,
        days: Option<u32>,
        from_date: Option<String>,
        to_date: Option<String>,
        content_type: ContentType,
        emails_only: bool,
        fetch_email_bodies: bool,
    },
    /// Multiple customers extraction mode
    MultipleCustomers {
        names: Vec<String>,
        days: Option<u32>,
        from_date: Option<String>,
        to_date: Option<String>,
        content_type: ContentType,
        emails_only: bool,
        fetch_email_bodies: bool,
    },
    /// Team extraction mode
    Team {
        stream_id: Option<String>,
        days: Option<u32>,
        from_date: Option<String>,
        to_date: Option<String>,
    },
    /// Generate shell completion
    Completion { shell: Shell },
    /// Interactive mode (no arguments provided)
    Interactive,
}

impl CliArgs {
    /// Parse and process arguments with smart detection
    ///
    /// This handles the flexible argument parsing,
    /// supporting argument order flexibility and hyphen-aware parsing.
    pub fn parse_command(&self) -> crate::Result<ParsedCommand> {
        // If we have a proper subcommand, use it
        if let Some(command) = &self.command {
            return match command {
                Command::Customer(args) => Ok(ParsedCommand::Customer {
                    name: args.name.clone(),
                    days: args.days,
                    from_date: args.from_date.clone(),
                    to_date: args.to_date.clone(),
                    content_type: args.content.unwrap_or(ContentType::Both),
                    emails_only: args.emails_only,
                    fetch_email_bodies: args.fetch_email_bodies,
                }),
                Command::Team(args) => Ok(ParsedCommand::Team {
                    stream_id: args.stream_id.clone(),
                    days: args.days,
                    from_date: args.from_date.clone(),
                    to_date: args.to_date.clone(),
                }),
                Command::Completion(args) => Ok(ParsedCommand::Completion { shell: args.shell }),
            };
        }

        // Handle raw arguments with smart parsing (backwards compatibility)
        if self.raw_args.is_empty() {
            return Ok(ParsedCommand::Interactive);
        }

        self.parse_raw_arguments()
    }

    /// Smart parsing of raw arguments to maintain backwards compatibility
    ///
    /// This implements sophisticated parsing logic:
    /// - Team vs customer mode detection
    /// - Flexible argument order
    /// - Content type detection
    /// - Hyphen-aware number parsing for customer names like "7 - 11"
    fn parse_raw_arguments(&self) -> crate::Result<ParsedCommand> {
        let args = &self.raw_args;

        if args.is_empty() {
            return Ok(ParsedCommand::Interactive);
        }

        // Check if first arg is "team"
        if args[0].to_lowercase() == "team" {
            let days = if args.len() > 1 {
                args[1].parse::<u32>().ok()
            } else {
                None
            };

            return Ok(ParsedCommand::Team {
                stream_id: None,
                days,
                from_date: None,
                to_date: None,
            });
        }

        // Otherwise, it's customer mode with flexible parsing
        let mut customer_parts = Vec::new();
        let mut days: Option<u32> = None;
        let mut content_type = ContentType::Both;
        let mut explicit_content = false;
        let mut emails_only = false;

        let mut i = 0;
        while i < args.len() {
            let arg = &args[i];
            let lower = arg.to_lowercase();

            // Check for content type keywords
            if lower == "calls" || lower == "call" {
                content_type = ContentType::Calls;
                explicit_content = true;
                i += 1;
                continue;
            } else if lower == "emails" || lower == "email" {
                if explicit_content && content_type == ContentType::Calls {
                    // Both calls and emails were specified
                    content_type = ContentType::Both;
                } else {
                    content_type = ContentType::Emails;
                    emails_only = true;
                }
                explicit_content = true;
                i += 1;
                continue;
            } else if lower == "both" {
                content_type = ContentType::Both;
                explicit_content = true;
                i += 1;
                continue;
            }

            // Try to parse as number for days
            if days.is_none() {
                if let Ok(d) = arg.parse::<u32>() {
                    // Check if this could be part of a hyphenated customer name
                    if i + 2 < args.len() && args[i + 1] == "-" {
                        // This is part of a name like "7 - 11"
                        customer_parts.push(arg.clone());
                        customer_parts.push(args[i + 1].clone());
                        customer_parts.push(args[i + 2].clone());
                        i += 3;
                        continue;
                    }
                    days = Some(d);
                    i += 1;
                    continue;
                }
            }

            // Everything else is part of the customer name
            customer_parts.push(arg.clone());
            i += 1;
        }

        // Join customer parts with spaces
        let customer_name = customer_parts.join(" ");

        if customer_name.is_empty() {
            return Err(crate::CsCliError::InvalidArguments {
                message: "Customer name is required".to_string(),
            });
        }

        // Determine fetch_email_bodies based on content type
        let fetch_email_bodies = matches!(content_type, ContentType::Emails | ContentType::Both);

        Ok(ParsedCommand::Customer {
            name: customer_name,
            days,
            from_date: None,
            to_date: None,
            content_type,
            emails_only,
            fetch_email_bodies,
        })
    }
}
