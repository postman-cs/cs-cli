//! Command line argument parsing and processing
//!
//! This module provides sophisticated argument parsing that matches the Python version's
//! flexibility, supporting both team and customer extraction modes with smart argument detection.

use clap::{Args, Parser, Subcommand};
use clap_complete::Shell;

/// CS-CLI: Customer Success Communication Extraction Tool
///
/// Extract customer communications or team calls from Gong and save as markdown files.
/// Run without arguments for interactive mode!
#[derive(Parser, Debug, Clone)]
#[command(
    name = "cs-cli",
    version,
    about = "Extract customer communications or team calls from Gong",
    long_about = "Extract customer communications or team calls from Gong and save as markdown files.

EXAMPLES:
    cs-cli                              Interactive mode
    cs-cli customer Postman 30          Get last 30 days of Postman
    cs-cli customer Wells Fargo calls 90    Get last 90 days of Wells Fargo calls
    cs-cli customer emails 7-11 365     Get last 365 days of 7-Eleven emails
    cs-cli customer \"Fortune 500\" 30 calls emails    Get calls and emails
    cs-cli team                         Get last 7 days of team calls (prompts for stream ID)
    cs-cli team 30                      Get last 30 days of team calls (prompts for stream ID)"
)]
#[derive(Default)]
pub struct CliArgs {
    /// Enable debug logging
    #[arg(long, help = "Enable debug logging")]
    pub debug: bool,

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
    /// Call stream ID (will prompt if not provided)
    #[arg(help = "Call stream ID (will prompt if not provided)")]
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

#[derive(clap::ValueEnum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContentType {
    /// Extract only calls
    Calls,
    /// Extract only emails
    Emails,
    /// Extract both calls and emails
    Both,
}

impl std::fmt::Display for ContentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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
    /// This handles the flexible argument parsing that matches the Python version,
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
    /// This implements the sophisticated parsing logic from the Python version:
    /// - Team vs customer mode detection
    /// - Flexible argument order
    /// - Content type detection
    /// - Hyphen-aware number parsing for customer names like "7 - 11"
    fn parse_raw_arguments(&self) -> crate::Result<ParsedCommand> {
        let args = &self.raw_args;

        if args.is_empty() {
            return Ok(ParsedCommand::Interactive);
        }

        // Check if first argument is 'team'
        if args[0].to_lowercase() == "team" {
            return self.parse_team_args(&args[1..]);
        }

        // Otherwise, parse as customer command
        self.parse_customer_args(args)
    }

    /// Parse team command arguments
    fn parse_team_args(&self, args: &[String]) -> crate::Result<ParsedCommand> {
        let mut days = None;
        let mut stream_id = None;

        // Look for a number (days parameter) or stream ID
        for arg in args {
            if let Ok(d) = arg.parse::<u32>() {
                if d > 0 && d < 10000 {
                    // Reasonable days range
                    days = Some(d);
                } else {
                    // Might be a stream ID
                    stream_id = Some(arg.clone());
                }
            } else if !arg.is_empty() {
                // Non-numeric argument is likely a stream ID
                stream_id = Some(arg.clone());
            }
        }

        Ok(ParsedCommand::Team {
            stream_id,
            days: days.or(Some(7)), // Default to 7 days for team
            from_date: None,
            to_date: None,
        })
    }

    /// Parse customer command arguments with flexible order and content detection
    ///
    /// Supports arguments in any order:
    /// - "Postman 30 calls"
    /// - "30 calls Postman"
    /// - "calls Postman 30"
    /// - "7 - 11 365 emails" (hyphen-aware)
    fn parse_customer_args(&self, args: &[String]) -> crate::Result<ParsedCommand> {
        // Step 1: Extract content keywords
        let mut content_keywords = Vec::new();
        let mut remaining_args = Vec::new();

        for arg in args {
            let arg_lower = arg.to_lowercase();
            if matches!(arg_lower.as_str(), "calls" | "emails" | "call" | "email") {
                // Convert singular to plural
                let content_word = if arg_lower == "call" {
                    "calls".to_string()
                } else if arg_lower == "email" {
                    "emails".to_string()
                } else {
                    arg_lower
                };
                content_keywords.push(content_word);
            } else {
                remaining_args.push(arg.clone());
            }
        }

        // Step 2: Smart number detection with hyphen awareness
        let mut days = None;
        let mut days_index = None;

        // Search from right to left to find the last number that's not part of a hyphenated name
        for i in (0..remaining_args.len()).rev() {
            if let Ok(d) = remaining_args[i].parse::<u32>() {
                if d == 0 || d > 10000 {
                    continue; // Skip unreasonable values
                }

                // Check if this is part of a hyphenated customer name
                let mut is_hyphenated = false;

                // Check if preceded by a hyphen (with potential number before it)
                if i > 0
                    && matches!(remaining_args[i - 1].as_str(), "-" | "–" | "—")
                    && i > 1
                    && remaining_args[i - 2].parse::<u32>().is_ok()
                {
                    is_hyphenated = true;
                }

                // Check if followed by a hyphen and another number
                if i < remaining_args.len().saturating_sub(2)
                    && matches!(remaining_args[i + 1].as_str(), "-" | "–" | "—")
                    && remaining_args[i + 2].parse::<u32>().is_ok()
                {
                    is_hyphenated = true;
                }

                if !is_hyphenated {
                    days = Some(d);
                    days_index = Some(i);
                    break;
                }
            }
        }

        // Step 3: Everything else is the customer name
        let customer_parts: Vec<String> = match days_index {
            Some(idx) => {
                let mut parts = Vec::new();
                parts.extend_from_slice(&remaining_args[..idx]);
                parts.extend_from_slice(&remaining_args[idx + 1..]);
                parts
            }
            None => remaining_args,
        };

        let customer_name = customer_parts.join(" ");

        if customer_name.is_empty() {
            return Err(crate::error::CsCliError::InvalidArguments {
                message: "Customer name is required".to_string(),
            });
        }

        // Step 4: Determine content type
        let content_type = match content_keywords.as_slice() {
            [] => ContentType::Both, // Default to both if not specified
            keywords
                if keywords.contains(&"calls".to_string())
                    && keywords.contains(&"emails".to_string()) =>
            {
                ContentType::Both
            }
            keywords if keywords.contains(&"calls".to_string()) => ContentType::Calls,
            keywords if keywords.contains(&"emails".to_string()) => ContentType::Emails,
            _ => ContentType::Both,
        };

        let emails_only = content_type == ContentType::Emails;

        Ok(ParsedCommand::Customer {
            name: customer_name,
            days: days.or(Some(90)), // Default to 90 days for customers
            from_date: None,
            to_date: None,
            content_type,
            emails_only,
            fetch_email_bodies: matches!(content_type, ContentType::Emails | ContentType::Both),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_customer_parsing_standard_order() {
        let args = CliArgs {
            debug: false,
            command: None,
            raw_args: vec!["Postman".to_string(), "30".to_string(), "calls".to_string()],
        };

        match args.parse_command().unwrap() {
            ParsedCommand::Customer {
                name,
                days,
                content_type,
                ..
            } => {
                assert_eq!(name, "Postman");
                assert_eq!(days, Some(30));
                assert_eq!(content_type, ContentType::Calls);
            }
            _ => panic!("Expected customer command"),
        }
    }

    #[test]
    fn test_customer_parsing_flexible_order() {
        let args = CliArgs {
            debug: false,
            command: None,
            raw_args: vec![
                "emails".to_string(),
                "30".to_string(),
                "Postman".to_string(),
            ],
        };

        match args.parse_command().unwrap() {
            ParsedCommand::Customer {
                name,
                days,
                content_type,
                ..
            } => {
                assert_eq!(name, "Postman");
                assert_eq!(days, Some(30));
                assert_eq!(content_type, ContentType::Emails);
            }
            _ => panic!("Expected customer command"),
        }
    }

    #[test]
    fn test_customer_parsing_hyphenated_name() {
        let args = CliArgs {
            debug: false,
            command: None,
            raw_args: vec![
                "7".to_string(),
                "-".to_string(),
                "11".to_string(),
                "365".to_string(),
                "emails".to_string(),
            ],
        };

        match args.parse_command().unwrap() {
            ParsedCommand::Customer {
                name,
                days,
                content_type,
                ..
            } => {
                assert_eq!(name, "7 - 11");
                assert_eq!(days, Some(365));
                assert_eq!(content_type, ContentType::Emails);
            }
            _ => panic!("Expected customer command"),
        }
    }

    #[test]
    fn test_team_parsing() {
        let args = CliArgs {
            debug: false,
            command: None,
            raw_args: vec!["team".to_string(), "14".to_string()],
        };

        match args.parse_command().unwrap() {
            ParsedCommand::Team { days, .. } => {
                assert_eq!(days, Some(14));
            }
            _ => panic!("Expected team command"),
        }
    }

    #[test]
    fn test_interactive_mode() {
        let args = CliArgs {
            debug: false,
            command: None,
            raw_args: vec![],
        };

        match args.parse_command().unwrap() {
            ParsedCommand::Interactive => {}
            _ => panic!("Expected interactive mode"),
        }
    }

    #[test]
    fn test_both_content_types() {
        let args = CliArgs {
            debug: false,
            command: None,
            raw_args: vec![
                "Postman".to_string(),
                "calls".to_string(),
                "emails".to_string(),
                "30".to_string(),
            ],
        };

        match args.parse_command().unwrap() {
            ParsedCommand::Customer {
                name,
                days,
                content_type,
                ..
            } => {
                assert_eq!(name, "Postman");
                assert_eq!(days, Some(30));
                assert_eq!(content_type, ContentType::Both);
            }
            _ => panic!("Expected customer command"),
        }
    }
}
